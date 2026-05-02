//! A partition file system with hashes for every file (hashes not supported)
//! 
//! HFS can be found in the partitions of a xci:
//! 
//! ```
//! use nxroms::formats::xci::Xci;
//! use std::fs::File;
//! 
//! fn main() {
//!     let mut rom = File::open("rom.xci").expect("fail");
//! 
//!     let mut xci = Xci::new(&mut rom).expect("fail");
//! 
//!     let mut part = xci.open_partition(String::from("secure"), &mut rom).expect("fail");
//! 
//!     let hfs = xci.open_partition_fs(&mut part).expect("fail");
//! 
//!     // Do things with it
//! }
//! ```
//! 
//! If you want to open a HFS manually you can do this:
//! ```
//! use nxroms::fs::hfs::HashPartitionFsHeader;
//! use nxroms::fs::pfs::PartitionFs;
//! use nxroms::BinRead;
//! 
//! use std::fs::File;
//! 
//! fn main() {
//!     // Replace with the buffer containing the HFS
//!     let mut file = File::open("fs.hfs").expect("fail");
//! 
//!     let hfs_header = HashPartitionFsHeader::read(&mut file).expect("fail");
//!     let mut pfs = PartitionFs::new(hfs_header).expect("fail");
//! 
//!     // Do things with it
//! }
//! ```

use binrw::BinRead;

use super::pfs::{PFSEntry, PFSHeader};


/// A struct holding information of a HFS0 entry
#[derive(BinRead, Debug, Clone, Copy)]
#[br(little)]
pub struct HFSEntry {
    _offset: u64,
    _size: u64,
    #[br(pad_after = 0x2C)]
    _string_offset: u32,
}

impl PFSEntry for HFSEntry {
    fn string_offset(&self) -> u32 {
        self._string_offset
    }

    fn size(&self) -> u64 {
        self._size
    }

    fn offset(&self) -> u64 {
        self._offset
    }
}

/// The HFS header
#[derive(BinRead, Debug)]
#[br(little, magic = b"HFS0")]
pub struct HashPartitionFsHeader {
    entry_count: u32,
    #[br(pad_after = 4)]
    string_table_size: u32,

    #[br(count = entry_count)]
    pub entry_table: Vec<HFSEntry>,

    #[br(count = string_table_size)]
    string_table: Vec<u8>,

    // HFSEntry is 0x40 bytes, not 0x18 like PFS0
    #[br(calc = entry_count as u64 * 0x40 + string_table_size as u64 + 0x10)]
    pub raw_data_pos: u64,
}

impl PFSHeader for HashPartitionFsHeader {
    type Entry = HFSEntry;
    fn raw_data_pos(&self) -> u64 {
        self.raw_data_pos
    }

    fn string_table(&self) -> &[u8] {
        &self.string_table
    }

    fn entry_table(&self) -> &Vec<Self::Entry> {
        &self.entry_table
    }
}
