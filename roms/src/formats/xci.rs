//! Nintendo Switch XCI format
//!
//! Example:
//! ```
//! use nxroms::formats::xci::Xci;
//! use std::fs::File;
//!
//! fn main() {
//!     let mut file = File::open("rom.xci").expect("failed to open file");
//!
//!     let mut xci = Xci::new(&mut file).expect("failed to read XCI");
//!
//!     // Do things with the XCI
//! }
//! ```
//! More info: <https://switchbrew.org/wiki/XCI>
use binrw::BinRead;

use crate::{
    fs::{
        hfs::HashPartitionFsHeader,
        pfs::{PFSEntry, PartitionFs, PartitionFsErrors},
    },
    readers::FileRegion,
};
use positioned_io::ReadAt;
use std::io::{Read, Seek, SeekFrom};

#[derive(BinRead, Debug)]
#[br(repr(u8))]
pub enum CardSize {
    _1GB = 0xFA,
    _2GB = 0xF8,
    _4GB = 0xF0,
    _8GB = 0xE0,
    _16GB = 0xE1,
    _32GB = 0xE2,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct XciHeader {
    #[br(count = 4, seek_before = SeekFrom::Start(0x100))]
    pub magic: Vec<u8>,

    #[br(seek_before = SeekFrom::Start(0x10C))]
    pub title_key_dec_index: u8,
    pub rom_size: CardSize,
    pub version: u8,

    #[br(seek_before = SeekFrom::Start(0x130))]
    pub hfs_header_offset: u64,
    pub hfs_header_size: u64,
}

#[derive(thiserror::Error, Debug)]
pub enum XciErrors {
    #[error("Invalid/corrupted xci: {0}")]
    CorruptXci(#[from] binrw::Error),
    #[error("Invalid magic: {0:?}")]
    InvalidMagic(Vec<u8>),
    #[error("Partition not found: {0}")]
    PartitionNotFound(String),
    #[error("PartitionFs error: {0}")]
    PartitionFsError(#[from] PartitionFsErrors),
}

#[derive(strum_macros::Display)]
pub enum XciPartition {
    Normal,
    Logo,
    Update,
    Secure,
}

/// Represents an XCI file
#[derive(Debug)]
pub struct Xci {
    pub header: XciHeader,
    pub root_hfs: PartitionFs<HashPartitionFsHeader>,
}

impl Xci {
    /// Creates a new XCI from a stream
    pub fn new<T: ReadAt + Read + Seek>(stream: &mut T) -> Result<Xci, XciErrors> {
        let h = XciHeader::read(stream)?;

        if h.magic != b"HEAD" {
            return Err(XciErrors::InvalidMagic(h.magic));
        }

        stream.seek(SeekFrom::Start(h.hfs_header_offset)).unwrap();
        let hfs_header = HashPartitionFsHeader::read(stream)?;
        let root_hfs = PartitionFs::<HashPartitionFsHeader>::new(hfs_header)?;

        Ok(Self {
            header: h,
            root_hfs,
        })
    }

    /// Opens a partition as a FileRegion
    pub fn open_partition<T: ReadAt + Read + Seek>(
        &mut self,
        partition: XciPartition,
        stream: T,
    ) -> Result<FileRegion<T>, XciErrors> {
        let part_string = partition.to_string();

        for entry in self.root_hfs.header.entry_table.iter() {
            let name = self.root_hfs.get_name_for_entry(entry)?;

            if name != part_string {
                continue;
            }

            let r = FileRegion::new(
                stream,
                self.header.hfs_header_offset + self.root_hfs.header.raw_data_pos + entry.offset(),
                entry.size(),
            );

            return Ok(r);
        }

        Err(XciErrors::PartitionNotFound(part_string))
    }

    /// Opens a partition as a PartitionFs
    pub fn open_partition_fs<T: ReadAt + Read + Seek>(
        &mut self,
        partition: &mut FileRegion<T>,
    ) -> Result<PartitionFs<HashPartitionFsHeader>, XciErrors> {
        let hfs_header = HashPartitionFsHeader::read(partition)?;
        let hfs = PartitionFs::<HashPartitionFsHeader>::new(hfs_header)?;

        Ok(hfs)
    }
}
