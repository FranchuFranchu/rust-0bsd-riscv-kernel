use alloc::vec::Vec;

use kernel_error_macro::KError;
use kernel_syscall_abi::process_egg::ProcessEggError;

use crate::process_egg::ProcessEgg;

#[derive(KError, Debug)]
pub enum ElfFileReadingError {
    ProcessEgg(ProcessEggError),
    ElfRs(elf_rs::Error),
    File((usize, [usize; 2])),
}

pub fn process_egg_from_elf_file(file: &crate::Handle) -> Result<ProcessEgg, ElfFileReadingError> {
    let mut fc = Vec::new();
    let mut buffer = Vec::new();
    buffer.resize(4096, 0);
    let mut buffer = buffer.into_boxed_slice();

    todo!();
    loop {
        let read = file.read(&mut buffer, &[])?;
        fc.extend_from_slice(&buffer[..read]);
        if read == 0 {
            break;
        }
    }

    let mut egg_handle = ProcessEgg::new()?;

    let elf_file = elf_rs::Elf::from_bytes(&fc);
    if let elf_rs::Elf::Elf64(e) = elf_file? {
        for p in e.program_header_iter() {
            if p.ph.memsz() as usize == 0 {
                continue;
            }
            // This is our buffer with the program's code
            //root_table.map(&segment[0] as *const u8 as usize, p.ph.vaddr() as usize, (p.ph.memsz() as usize).max(4096), EntryBits::EXECUTE | EntryBits::VALID | EntryBits::READ);
            egg_handle.set_memory(p.ph.vaddr() as usize, p.segment());
        }
        egg_handle.set_start_address(e.header().entry_point() as usize)
    }

    Ok(egg_handle)
}
