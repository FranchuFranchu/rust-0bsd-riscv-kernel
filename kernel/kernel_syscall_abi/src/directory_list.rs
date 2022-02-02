use alloc::{string::String, vec::Vec};

use flat_bytes::Flat;

#[derive(Flat)]
#[repr(u8)]
pub enum DirectoryWritePacketHeader {
    Create, // Followed by directory entry
    Delete, // Followed by name
}

#[derive(Flat)]
pub struct DirectoryEntry {
    name: String,
    attributes: Vec<DirectoryAttribute>,
}

#[derive(Flat)]
#[repr(u8)]
pub enum DirectoryAttribute {
    PermissionFlags(PermissionFlags),
}

type PermissionFlags = u8;
