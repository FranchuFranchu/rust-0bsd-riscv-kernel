use alloc::{string::String, vec::Vec};

use flat_bytes::Flat;

#[derive(Flat)]
#[repr(u8)]
pub enum DirectoryWritePacketHeader {
    Create(DirectoryEntry),
    Delete(String),
}

#[derive(Flat)]
pub struct DirectoryEntry {
    pub name: String,
    pub attributes: Vec<DirectoryAttribute>,
}

#[derive(Flat)]
#[repr(u8)]
pub enum DirectoryAttribute {
    PermissionFlags(PermissionFlags),
    Inode(u64),
}

type PermissionFlags = u8;
