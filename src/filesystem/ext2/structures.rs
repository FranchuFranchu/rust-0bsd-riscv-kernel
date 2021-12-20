use core::str::Utf8Error;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Superblock {
    /// 32bit value indicating the total number of inodes, both used and free,
    /// in the file system.  This value must be lower or equal to
    /// (s_inodes_per_group * number of block groups).  It must be equal to the
    /// sum of the inodes defined in each block group.
    pub inodes_count: u32,

    /// 32bit value indicating the total number of blocks in the system including
    /// all used, free and reserved. This value must be lower or equal to
    /// (s_blocks_per_group * number of block groups). It can be lower than
    /// the previous calculation if the last block group has a smaller number of
    /// blocks than s_blocks_per_group du to volume size.  It must be equal to
    /// the sum of the blocks defined in each block group.
    pub blocks_count: u32,

    /// 32bit value indicating the total number of  blocks  reserved  for  the
    /// usage of the super user.  This is most useful if  for  some  reason  a
    /// user, maliciously or not, fill the file system to capacity; the  super
    /// user will have this specified amount of free blocks at his disposal so
    /// he can edit and save configuration files.
    pub r_blocks_count: u32,

    /// 32bit value indicating the total number of free blocks, including  the
    /// number of reserved blocks (see
    /// s_r_blocks_count).  This is a  sum
    /// of all free blocks of all the block groups.
    pub free_blocks_count: u32,

    /// 32bit value indicating the total number of free inodes.  This is a sum
    /// of all free inodes of all the block groups.
    pub free_inodes_count: u32,

    /// 32bit value identifying the first data block, in other word the id  of
    /// the block containing the superblock structure.
    pub first_data_block: u32,

    /// The block size is computed using this 32bit value  as  the  number  of
    /// bits to shift left the value 1024.  This value may only be non-negative.
    pub log_block_size: u32,

    /// The fragment size is computed using this 32bit value as the number  of
    /// bits to shift left the value 1024.  Note that a negative  value  would
    /// shift the bit right rather than left.
    pub log_frag_size: u32,

    /// 32bit value indicating the total number  of  blocks  per  group.  This
    /// value in combination with
    /// s_first_data_block can  be  used
    /// to determine the block groups boundaries.  Due to volume size boundaries,
    /// the last block group might have a smaller number of blocks than what is
    /// specified in this field.
    pub blocks_per_group: u32,

    /// 32bit value indicating the total number of fragments per group.  It is
    /// also used to determine the size of the block bitmap  of
    /// each block group.
    pub frags_per_group: u32,

    /// 32bit value indicating the total number of inodes per group.  This  is
    /// also used to determine the size of the inode bitmap  of
    /// each block group.  Note that you cannot have more than
    /// (block size in bytes * 8) inodes per group as the inode bitmap
    /// must fit within a single block. This value must be a perfect multiple
    /// of the number of inodes that can fit in a block
    /// ((1024<<s_log_block_size)/s_inode_size).
    pub inodes_per_group: u32,

    /// Unix time, as defined by POSIX, of the last time the file  system  was
    /// mounted.
    pub mtime: u32,

    /// Unix time, as defined by POSIX, of the last write access to  the  file
    /// system.
    pub wtime: u32,

    /// 16bit value indicating how many  time  the  file  system  was  mounted
    /// since the last time it was fully verified.
    pub mnt_count: u16,

    /// 16bit value indicating the maximum  number  of  times  that  the  file
    /// system may be mounted before a full check is performed.
    pub max_mnt_count: u16,

    /// 16bit value  identifying  the  file  system  as  Ext2.  The  value  is
    /// currently fixed to EXT2_SUPER_MAGIC of value 0xEF53.
    pub magic: u16,

    /// 16bit value indicating the file system state.  When the file system is
    /// mounted, this state is set  to  EXT2_ERROR_FS.  After the
    /// file system was cleanly unmounted, this value is set to EXT2_VALID_FS.
    pub state: u16,

    /// 16bit value indicating what the file system driver should do  when  an
    /// error is detected.  The following values have been defined:
    pub errors: u16,

    /// 16bit value identifying the minor revision level within its
    /// revision level.
    pub minor_rev_level: u16,

    /// Unix time, as defined by POSIX, of the last file system check.
    pub lastcheck: u32,

    /// Maximum Unix time interval, as defined by POSIX, allowed between file
    /// system checks.
    pub checkinterval: u32,

    /// 32bit identifier of the os that created the file system.  Defined
    /// values are:
    pub creator_os: u32,

    /// 32bit revision level value.
    pub rev_level: u32,

    /// 16bit value used as the default user id for reserved blocks.
    pub def_resuid: u16,

    /// 16bit value used as the default group id for reserved blocks.
    pub def_resgid: u16,

    /// 32bit value used as index to the  first inode  useable  for  standard
    /// files. In revision 0, the first	non-reserved inode is fixed to
    /// 11 (EXT2_GOOD_OLD_FIRST_INO). In revision 1 and later
    /// this value may be set to any value.
    pub first_ino: u32,

    /// 16bit value indicating the size of the inode structure. In revision 0, this
    /// value is always 128 (EXT2_GOOD_OLD_INODE_SIZE). In revision 1
    /// and later, this value must be a perfect power of 2 and must be smaller or
    /// equal to the block size (1<<s_log_block_size).
    pub inode_size: u16,

    /// 16bit value used to indicate the block group number hosting this
    /// superblock structure.  This can be used to rebuild the file system
    /// from any superblock backup.
    pub block_group_nr: u16,

    /// 32bit bitmask of compatible features.  The file system implementation
    /// is free to support them or not without risk of damaging the meta-data.
    pub feature_compat: u32,

    /// 32bit bitmask of incompatible features.  The file system
    /// implementation should refuse to mount the file system if any of
    /// the indicated feature is unsupported.
    pub feature_incompat: u32,

    /// 32bit bitmask of “read-only” features.  The file system
    /// implementation should mount as read-only if any of the indicated
    /// feature is unsupported.
    pub feature_ro_compat: u32,

    /// 128bit value used as the volume id.  This should, as much as possible,
    /// be unique for each file system formatted.
    pub uuid: [u8; 16],

    /// 16 bytes volume name, mostly unusued.  A valid volume name would consist
    /// of only ISO-Latin-1 characters and be 0 terminated.
    pub volume_name: [u8; 16],

    /// 64 bytes directory path where the file system was last mounted.  While
    /// not normally used, it could serve for auto-finding the mountpoint when
    /// not indicated on the command line. Again the path should be zero
    /// terminated for compatibility reasons.  Valid path is constructed from
    /// ISO-Latin-1 characters.
    pub last_mounted: [u8; 64],

    /// 32bit value used by compression algorithms to determine the compression
    /// method(s) used.
    pub algo_bitmap: u32,

    /// 8-bit value representing the number of blocks the implementation should
    /// attempt to pre-allocate when creating a new regular file.
    pub prealloc_blocks: u8,

    /// 8-bit value representing the number of blocks the implementation should
    /// attempt to pre-allocate when creating a new directory.
    pub prealloc_dir_blocks: u8,

    pub _pad_1: [u8; 2],
    /// 16-byte value containing the uuid of the journal superblock.  See Ext3
    /// Journaling for more information.
    pub journal_uuid: [u8; 16],

    /// 32-bit inode number of the journal file.  See Ext3 Journaling for more
    /// information.
    pub journal_inum: u32,

    /// 32-bit device number of the journal file.  See Ext3 Journaling for more
    /// information.
    pub journal_dev: u32,

    /// 32-bit inode number, pointing to the first inode in the list of inodes
    /// to delete.  See Ext3 Journaling for more information.
    pub last_orphan: u32,

    /// An array of 4 32bit values containing the seeds used for the hash
    /// algorithm for directory indexing.
    pub hash_seed: [u32; 4],

    /// An 8bit value containing the default hash version used for directory indexing.
    pub def_hash_version: u8,

    pub _pad_2: [u8; 3],
    /// A 32bit value containing the default mount options for this file system. TODO: Add more information here!
    pub default_mount_options: u32,

    /// A 32bit value indicating the block group ID of the first meta block group.  TODO: Research if this is an Ext3-only extension.
    pub first_meta_bg: u32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct BlockGroupDescriptor {
    /// 32bit block id of the first block of the
    /// “block bitmap”
    /// for the group represented.
    pub block_bitmap: u32,

    /// 32bit block id of the first block of the
    /// “inode bitmap”
    /// for the group represented.
    pub inode_bitmap: u32,

    /// 32bit block id of the first block of the
    /// “inode table”
    /// for the group represented.
    pub inode_table: u32,

    /// 16bit value indicating the total number of free blocks for
    /// the represented group.
    pub free_blocks_count: u16,

    /// 16bit value indicating the total number of free inodes for
    /// the represented group.
    pub free_inodes_count: u16,

    /// 16bit value indicating the number of inodes allocated to
    /// directories for the represented group.
    pub used_dirs_count: u16,

    /// 16bit value used for padding the structure on a 32bit boundary.
    pub pad: u16,

    /// 12 bytes of reserved space for future revisions.
    pub reserved: [u8; 12],
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct Inode {
    /// 16bit value used to indicate the format of the described file and the
    /// access rights.  Here are the possible values, which can be combined
    /// in various ways:
    pub mode: u16,

    /// 16bit user id associated with the file.
    pub uid: u16,

    /// In revision 0, (signed) 32bit value indicating the size of the file in
    /// bytes.  In revision 1 and later revisions, and only for regular files, this
    /// represents the lower 32-bit of the file size; the upper 32-bit is located
    /// in the i_dir_acl.
    pub size: u32,

    /// 32bit value representing the number of seconds since january 1st 1970
    /// of the last time this inode was accessed.
    pub atime: u32,

    /// 32bit value representing the number of seconds since january 1st 1970, of
    /// when the inode was created.
    pub ctime: u32,

    /// 32bit value representing the number of seconds since january 1st 1970,
    /// of the last time this inode was modified.
    pub mtime: u32,

    /// 32bit value representing the number of seconds since january 1st 1970, of
    /// when the inode was deleted.
    pub dtime: u32,

    /// 16bit value of the POSIX group having access to this file.
    pub gid: u16,

    /// 16bit value indicating how many times this particular inode is linked
    /// (referred to). Most files will have a link count of 1.  Files with hard
    /// links pointing to them will have an additional count for each hard link.
    pub links_count: u16,

    /// 32-bit value representing the total number of 512-bytes blocks reserved to contain the
    /// data of this inode, regardless if these blocks are used or not.  The block
    /// numbers of these reserved blocks are contained in the
    /// i_block array.
    pub blocks: u32,

    /// 32bit value indicating how the ext2 implementation should behave when
    /// accessing the data for this inode.
    pub flags: u32,

    pub osd1: u32,
    /// 15 x 32bit block numbers pointing to the blocks containing the data for
    /// this inode. The first 12 blocks are direct blocks.  The 13th entry in this
    /// array is the block number of the first indirect block; which is a block
    /// containing an array of block ID containing the data.  Therefore, the 13th
    /// block of the file will be the first block ID contained in the indirect block.
    /// With a 1KiB block size, blocks 13 to 268 of the file data are contained
    /// in this indirect block.
    pub block: [u32; 15],

    /// 32bit value used to indicate the file version (used by NFS).
    pub generation: u32,

    /// 32bit value indicating the block number containing the extended
    /// attributes. In revision 0 this value is always 0.
    pub file_acl: u32,

    /// In revision 0 this 32bit value is always 0.  In revision 1, for regular
    /// files this 32bit value contains the high 32 bits of the 64bit file size.
    pub dir_acl: u32,

    /// 32bit value indicating the location of the file fragment.
    pub faddr: u32,

    pub osd2: u32,
    pub osd3: u16,
}

#[derive(Debug)]
#[repr(C)]
pub struct DirectoryEntry {
    /// 32bit inode number of the file entry.  A value of 0 indicate that the entry
    /// is not used.
    pub inode: u32,

    /// 16bit unsigned displacement to the next directory entry from the start of the
    /// current directory entry. This field must have a value at least equal to the
    /// length of the current record.
    pub rec_len: u16,

    /// 8bit unsigned value indicating how many bytes of character data are contained in the name.
    pub name_len: u8,

    /// 8bit unsigned value used to indicate file type.
    pub file_type: u8,

    pub name: [u8; 0],
}

#[derive(Debug)]
pub struct OwnedDirectoryEntry {
    /// 32bit inode number of the file entry.  A value of 0 indicate that the entry
    /// is not used.
    pub inode: u32,

    /// 8bit unsigned value used to indicate file type.
    pub file_type: u8,

    pub name: alloc::string::String,
}

impl From<(&DirectoryEntry, &str)> for OwnedDirectoryEntry {
    fn from(other: (&DirectoryEntry, &str)) -> Self {
        OwnedDirectoryEntry {
            inode: other.0.inode,
            file_type: other.0.file_type,
            name: alloc::string::String::from(other.1),
        }
    }
}

impl OwnedDirectoryEntry {
    pub unsafe fn from_get_name(other: &DirectoryEntry) -> Result<Self, Utf8Error> {
        Ok(Self::from((other, core::str::from_utf8(other.get_name())?)))
    }
}

impl DirectoryEntry {
    pub unsafe fn get_name(&self) -> &[u8] {
        core::slice::from_raw_parts(self.name.as_ptr(), self.name_len.into())
    }
}

struct IndexedDirectoryRoot {
    pub _pad_1: [u8; 9],
    pub padding: u16,
    pub padding2: u8,

    pub _pad_2: [u8; 10],
    pub padding_2: u16,

    pub _pad_3: [u8; 4],
    /// 8bit value representing the hash version used in this indexed directory.
    pub hash_version: u8,

    /// 8bit length of the indexed directory information structure (dx_root);
    /// currently equal to 8.
    pub info_length: u8,

    /// 8bit value indicating how many indirect levels of indexing are present in
    /// this hash.
    pub indirect_levels: u8,
}
struct IndexedDirectoryEntryCountandLimit {
    /// 16bit value representing the total number of indexed directory entries that
    /// fit within the block, after removing the other structures, but including
    /// the count/limit entry.
    pub limit: u16,

    /// 16bit value representing the total number of indexed directory entries
    /// present in the block. TODO: Research if this value includes the count/limit entry.
    pub count: u16,
}
