//! Handle backend that allows processes to create "egg" child processes and set them up before executing them

use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc};

use flat_bytes::Flat;
use kernel_as_register::EncodedError;
use kernel_cpu::csr::SATP_SV39;

use crate::{
    handle::HandleBackend,
    lock::future::rwlock::RwLock,
    paging::{
        EntryBits::{RWX, USER, VALID},
        Paging,
    },
    process::{new_process, Process},
    virtual_buffers,
};

pub struct ProcessEgg {
    name: String,
    start_address: usize,
    root_table: Box<crate::paging::Table>,
}

pub struct ProcessEggBackend {
    handle_eggs: RwLock<BTreeMap<usize, RwLock<ProcessEgg>>>,
}

#[derive(Flat)]
#[repr(u8)]
pub enum ProcessEggPacketHeader {
    Entry(usize),
    Memory(usize),
    Name(),
    Hatch,
}

#[async_trait]
impl<'this> HandleBackend for ProcessEggBackend {
    fn create_singleton() -> alloc::sync::Arc<dyn HandleBackend + Send + Sync + 'static>
    where
        Self: Sized,
    {
        Arc::new(Self {
            handle_eggs: RwLock::new(BTreeMap::new()),
        })
    }

    async fn open(&self, fd_id: &usize, _options: &[usize]) -> Result<usize, EncodedError> {
        let mut egg = ProcessEgg {
            root_table: Box::new(crate::paging::Table::zeroed()),
            start_address: 0,
            name: String::new(),
        };

        let mut root_table = crate::paging::sv39::RootTable(&mut egg.root_table);

        virtual_buffers::initialize_root_table(&mut root_table);

        root_table.map(0x80000000, 0x80000000, 0x80000000, VALID | RWX);

        self.handle_eggs
            .write()
            .await
            .insert(*fd_id, RwLock::new(egg));

        Ok(0)
    }

    fn name(&self) -> &'static str {
        "ProcessEggBackend"
    }

    async fn read(
        &self,
        _fd_id: &usize,
        _buf: &mut [u8],
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        Ok(0)
    }
    async fn write(
        &self,
        fd_id: &usize,
        buf: &[u8],
        _options: &[usize],
    ) -> Result<usize, EncodedError> {
        let (header, size) = ProcessEggPacketHeader::deserialize_with_size(buf).unwrap();
        let btreemap_lock = self.handle_eggs.read().await;
        let mut egg = btreemap_lock.get(fd_id).unwrap().write().await;
        let data = &buf[size..];
        use ProcessEggPacketHeader::*;
        match header {
            Memory(address) => {
                let page_offset = address % 4096;
                let page_aligned_address = address - page_offset;
                println!("offset {:?}", page_offset);
                let mut table = crate::paging::sv39::RootTable(&mut *egg.root_table);
                let iter1 =
                    (page_aligned_address..(page_aligned_address + data.len())).step_by(4096);
                let iter2 = iter1.clone();
                let mut get_slice_for_page = |page_number| -> &'static mut [u8] {
                    if let Ok(physical_address) = unsafe { table.query_physical_address(page_number) } {
                        unsafe { core::slice::from_raw_parts_mut(physical_address as *mut _, 4096) }
                    } else {
                        let mut slice = kernel_util::boxed_slice_with_alignment(4096, 4096, &0u8);
                        let addr = &mut slice[0] as *mut u8;
                        table.map(addr as usize, page_number, 0x4096, RWX | VALID | USER);
                        core::mem::forget(slice);
                        unsafe { core::slice::from_raw_parts_mut(addr, 4096) }
                    }
                };
                for (page_number, next_page_number) in iter1.zip(iter2) {
                    let index_in_data = page_number - page_aligned_address;
                    println!("pg {:x}", page_number);
                    assert!(page_number < 0x80000000);
                    let this_page = get_slice_for_page(page_number);
                    let next_page = get_slice_for_page(next_page_number);
                    this_page[page_offset..4096.min(data.len() + page_offset)].copy_from_slice(
                        &data[index_in_data..(index_in_data + 4096 - page_offset).min(data.len())],
                    );
                    if data.len() > index_in_data + page_offset + 4096 {
                        next_page[..4096 - page_offset].copy_from_slice(
                            &data[index_in_data + 4096 - page_offset
                                ..(index_in_data + 4096).min(data.len())],
                        );
                    }
                }
            }
            Entry(v) => {
                egg.start_address = v;
            }
            Hatch => {
                drop(egg);
                drop(btreemap_lock);

                let egg = self
                    .handle_eggs
                    .write()
                    .await
                    .remove(fd_id)
                    .unwrap()
                    .into_inner();
                new_process(move |process: &mut Process| {
                    let process: &mut Process = process;
                    process.name = Some(egg.name);
                    println!("{:?}", unsafe {
                        egg.root_table[0].as_table()[0].as_table()[0x10]
                    });
                    let addr = &(*egg.root_table) as *const _;
                    println!("Addr {:?}", addr);
                    core::mem::forget(egg.root_table);
                    process.trap_frame.pc = egg.start_address;
                    process.trap_frame.satp = (addr as usize >> 12) | SATP_SV39;
                });
            }
            _ => panic!(),
        }
        Ok(buf.len())
    }
}
