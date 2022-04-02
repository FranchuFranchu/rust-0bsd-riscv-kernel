use alloc::vec;

use kernel_syscall_abi::directory_list::{
    DirectoryAttribute, DirectoryEntry, DirectoryWritePacketHeader,
};

use super::structures::OwnedDirectoryEntry;

pub fn ext2_entry_to_user_entry(entry: OwnedDirectoryEntry) -> DirectoryEntry {
    let entry = DirectoryEntry {
        name: entry.name,
        attributes: vec![DirectoryAttribute::Inode(entry.inode as u64)],
    };

    entry
}

fn apply_user_entry_to_ext2_entry(entry: &DirectoryWritePacketHeader) {}
