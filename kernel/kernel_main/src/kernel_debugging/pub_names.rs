use gimli::{DebugPubNames, EndianSlice, NativeEndian};

extern "C" {}

pub fn get_debug_pub_names() -> DebugPubNames<EndianSlice<'static, NativeEndian>> {
    let start = 0; // unsafe { &__debug_pubnames_start as *const u32 as usize };
    let end = 0; // unsafe { (&__debug_pubnames_end as *const u32 ) as usize};
    println!("linked to:{:x} {:x}", start, end);
    let slice = unsafe { core::slice::from_raw_parts(start as *const u8, end - start) };
    DebugPubNames::new(slice, NativeEndian)
}

pub fn test() {
    let a = get_debug_pub_names();
    let mut entries = a.items();
    println!("{:?}", entries.next());
    while let Some(e) = entries.next().unwrap() {
        println!("E {:?}", e);
    }
    panic!()
}

pub fn address_to_symbol_and_offset(_address: usize) {}
