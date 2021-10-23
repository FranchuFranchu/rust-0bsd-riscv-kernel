//! The functions here are tasks that can be run to make sure that complex kernel tasks
//! won't crash

use alloc::{collections::BTreeSet, vec::Vec};
use core::{
    ops::{BitAnd, BitXor},
    pin::Pin,
    task::Context,
};

use crate::{
    asm::do_supervisor_syscall_0,
    cpu,
    drivers::{traits::block::GenericBlockDevice, virtio::VirtioDriver},
    external_interrupt::ExternalInterruptHandler,
    fdt, process,
};

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
        loop {
            let inode = ext2.get_path("/file2").await.unwrap().unwrap();
            let mut handle = ext2.inode_handle(inode).await.unwrap();
            use kernel_io::Read;
            let t = handle.read_to_end_new().await.unwrap();
            println!("File contents: {:?}", core::str::from_utf8(&t.1).unwrap());
        }
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
