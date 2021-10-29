//! The functions here are tasks that can be run to make sure that complex kernel tasks
//! won't crash

use aligned::Aligned;
use alloc::{collections::BTreeSet, vec::Vec};
use core::{
    ops::{BitAnd, BitXor},
    pin::Pin,
    task::Context,
};
use core::iter::FromIterator;
use core::mem::size_of;
use core::mem::MaybeUninit;
use alloc::alloc::Layout;

use crate::{asm::do_supervisor_syscall_0, cpu::{self, Registers}, drivers::{traits::block::GenericBlockDevice, virtio::VirtioDriver}, external_interrupt::ExternalInterruptHandler, fdt, paging::{EntryBits, Paging}, process};

// random-ish function I just made up
fn twist(value: &mut usize) -> usize {
    *value = value
        .wrapping_add(
            #[cfg(target_arch = "riscv64")]
            {
                0x902392093222
            },
            #[cfg(target_arch = "riscv32")]
            {
                0x90233423
            },
        )
        .bitxor(0b10101110101)
        .bitand(0xFF);
    *value
}

pub fn boxed_slice_with_alignment<T: Clone>(size: usize, align: usize, initialize: &T) -> alloc::boxed::Box<[T]> {
    unsafe { 
        let ptr: *mut MaybeUninit<T> = alloc::alloc::alloc(Layout::from_size_align(size * size_of::<T>(), align).unwrap()) as *mut MaybeUninit<T>;
        for i in 0..size {
            *ptr.add(i) = MaybeUninit::new(initialize.clone())
        }
        alloc::boxed::Box::from_raw(core::slice::from_raw_parts_mut(ptr as *mut T, size))
        
        
    }
}

pub fn test_task() {
    // Calculate primes
    let mut sieve = Vec::new();
    let mut not_removed = BTreeSet::new();
    for i in 0..2000 {
        sieve.push(false);
        if i > 1 {
            not_removed.insert(i);
        }
    }
    for idx in 2..sieve.len() {
        if sieve[idx] {
            continue;
        }
        let mut jdx = idx * 2;
        while jdx < 2000 {
            sieve[jdx] = true;
            jdx += idx;
        }
        for maybe_prime_idx in 2..idx {
            if !sieve[maybe_prime_idx] && not_removed.contains(&maybe_prime_idx) {
                println!("Prime: {}", maybe_prime_idx);
                not_removed.remove(&maybe_prime_idx);
            }
        }
    }
    loop {}
}

pub fn test_task_2() {
    // Allocate tons of memory
    let twisted_value = 0;
    let mut vector_vec = Vec::with_capacity(10);
    for i in 0..70 {
        let mut v: Vec<usize> = Vec::with_capacity(twisted_value);
        v.resize(twisted_value, 0);
        for i in v.iter_mut() {
            *i = i as *mut usize as usize;
        }
        vector_vec.push(v);
    }
    for v in vector_vec.iter() {
        for i in v.iter() {
            assert!(*i == i as *const usize as usize);
        }
    }
    drop(vector_vec);

    use crate::timeout::TimeoutFuture;
    // On QEMU, 10_000_000 timebase is 1 second
    let mut future = TimeoutFuture {
        for_time: cpu::get_time() + 10_000_000,
    };
    let waker = process::Process::this().write().construct_waker();
    use core::future::Future;

    info!("Scheduling timeout..");

    // Poll the future until it resolves
    while TimeoutFuture::poll(Pin::new(&mut future), &mut Context::from_waker(&waker))
        == core::task::Poll::Pending
    {
        // Trigger a "yield" smode-to-smode syscall
        trigger_yield_syscall();
    }

    info!("Timeout finished");
}

pub fn test_task_3() {
    {
        info!("Waiting");
        use crate::lock::shared::Mutex;

        let m = Mutex::new(0);

        let m1 = m.lock();

        drop(m1);
        let m2 = m.lock();
    }
    use alloc::sync::Arc;

    use crate::lock::shared::RwLock;

    let exec = crate::future::Executor::new();
    let block = async {
        // First, wait until the device setup is done
        info!("Waiting");
        crate::device_setup::is_done_future().await;
        info!("Waited");
        // Get the block device

        let block_device: Arc<RwLock<dyn GenericBlockDevice + Send + Sync + Unpin>> = {
            let guard = fdt::root().read();
            let block_device_node = guard.get("soc/virtio_mmio@10008000").unwrap();
            let lock = block_device_node.kernel_struct.read();
            let bd = lock
                .as_ref()
                .unwrap()
                .downcast_ref::<(VirtioDriver, Option<ExternalInterruptHandler>)>();

            let block_device = if let VirtioDriver::Block(bd) = &bd.as_ref().unwrap().0 {
                bd
            } else {
                panic!("Block device not found!");
            };
            block_device.clone()
        };

        /*
        let mut v: Vec<u8> = Vec::new();
        v.resize(512, 0);

        v[0] = 65;
        v[1] = 67;

        //println!("Write: {:?}", v);

        let request = block_device
            .lock()
            .create_request(0, v.into_boxed_slice(), true);

        let buf = request.await;
        info!("Read {:?}", buf);

        let mut v: Vec<u8> = Vec::new();
        v.resize(512, 0);

        // Read the block again
        let request = block_device
            .lock()
            .create_request(0, v.into_boxed_slice(), false);

        let buf = request.await;
        */

        use crate::filesystem::ext2::code::Ext2;

        let ext2 = Ext2::new(&block_device);

        ext2.load_superblock().await.unwrap();

        info!("{:?}", ext2.read_inode(2).await.unwrap());
        
        
        let inode = ext2.get_path("/main").await.unwrap().unwrap();
        let mut handle = ext2.inode_handle(inode).await.unwrap();
        use kernel_io::Read;
        let t = handle.read_to_end_new().await.unwrap();
        let mut new_page_table = Box::new(crate::paging::Table::zeroed());
        
        
        
        let mut root_table = crate::paging::sv39::RootTable(&mut new_page_table);
        
        
        let elf_file = elf_rs::Elf::from_bytes(&t.1);
        let mut allocated_segments = Vec::new();
        if let elf_rs::Elf::Elf64(e) = elf_file.unwrap() {
            println!("{:?} header: {:?}", e, e.header());

            for p in e.program_header_iter() {
                // This is our buffer with the program's code
                let mut segment = boxed_slice_with_alignment(p.ph.memsz() as usize, 4096, &0u8);
                println!("{:x}", p.ph.vaddr());
                println!("{:?}", segment);
                info!("{:x}", &segment[0] as *const u8 as usize);
                // Copy segment to buffer
                segment.copy_from_slice(p.segment());
                //root_table.map(&segment[0] as *const u8 as usize, p.ph.vaddr() as usize, (p.ph.memsz() as usize).max(4096), EntryBits::EXECUTE | EntryBits::VALID | EntryBits::READ);
                root_table.map(&segment[0] as *const u8 as usize, p.ph.vaddr() as usize, (p.ph.memsz() as usize).max(4096), EntryBits::EXECUTE | EntryBits::VALID | EntryBits::READ | EntryBits::USER);
                allocated_segments.push(segment);
            }

            for s in e.section_header_iter() {
                //println!("{:x?}", s);
            }

            let s = e.lookup_section(".text");
            //println!("s {:?}", s);
            
            
            unsafe { println!("{:x}", root_table.0.entries[0].as_table_mut()[0].as_table_mut()[32].value); }
            process::new_process_int(e.header().entry_point() as usize, 0, |process| {
                crate::paging::map_critical_kernel_address_space(&mut root_table, &*process.trap_frame as *const _ as usize);
                // Create a buffer with the program's stack
                let program_stack = boxed_slice_with_alignment(4096, 4096, &0u8);
                root_table.map(&program_stack[0] as *const _ as usize, 0x40000, 4096, EntryBits::VALID | EntryBits::READ | EntryBits::WRITE | EntryBits::USER);
                process.trap_frame.general_registers[Registers::Sp.idx()] = 0x40800;
                core::mem::forget(program_stack);
                process.is_supervisor = false;
                process.trap_frame.satp = (&*root_table.0 as *const crate::paging::Table as usize)  >> 12  | cpu::csr::SATP_SV39;
            });
        };
        core::mem::forget(allocated_segments);
        core::mem::forget(root_table);
        core::mem::forget(new_page_table);
        
        //crate::sbi::shutdown(0);
    };
    let block = Box::pin(block);
    let mut block = Box::new(block);
    use alloc::boxed::Box;
    //exec.push_future(block);
    // TODO maybe use Some(task) in the future?

    let waker = crate::process::Process::this().read().construct_waker();
    let mut context = Context::from_waker(&waker);
    use core::future::Future;
    while core::task::Poll::Pending == Pin::new(&mut block).poll(&mut context) {
        unsafe { do_supervisor_syscall_0(2) };
    }

    info!("Ending");
}

#[inline]
fn trigger_yield_syscall() {
    unsafe {
        asm!(r"
			li a7, 2
			# Trigger a timer interrupt
			csrr t0, sip
			# Set SSIP
			ori t0, t0, 2
			csrw sip, t0
		", out("a7") _, out("t0") _)
    }
}
