//! File system with a magic value of `PFS0`
//!
//! Example:
//! ```
//! use nxroms::fs::pfs::{PartitionFs, PFSHeader};
//! use std::io::Read;
//! use std::fs::File;
//!
//! fn main() {
//!     let mut file = File::open("rom.nsp").expect("err");
//!     let pfs = PartitionFs::new_pfs0(&mut file).expect("err");
//!
//!     println!("Listing pfs0 files:");
//!     for (index, entry) in pfs.header.entry_table().iter().enumerate() {
//!         let name = pfs.get_name_for_entry(entry).expect("name should be valid");
//!
//!         println!("\t{}: {}", index, name);
//!     }
//!}
//! ```

use binrw::BinRead;
use positioned_io::ReadAt;
use std::io::{Read, Seek};
use std::string::FromUtf8Error;
use thiserror::Error;

use crate::readers::FileRegion;

/// Errors that can happen when using [`PartitionFs`]
#[derive(Error, Debug)]
pub enum PartitionFsErrors {
    #[error("Failed to parse pfs")]
    CorruptPfs(#[from] binrw::Error),
    #[error("Failed to decode from bytes")]
    DecodingError(#[from] FromUtf8Error),
    #[error("Failed to find null terminator in string")]
    NullTerminatorError,
}

/// A partition file system entry
#[derive(BinRead, Debug, Clone, Copy)]
#[br(little)]
pub struct PartitionFsEntry {
    _offset: u64,
    _size: u64,
    #[br(pad_after = 4)]
    _string_offset: u32,
}

impl PFSEntry for PartitionFsEntry {
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

/// A struct holding information of a PFS0 entry
#[derive(BinRead, Debug)]
#[br(little, magic = b"PFS0")]
pub struct PartitionFsHeader {
    /// The count of pfs entries
    entry_count: u32,

    /// The size of the string table.
    #[br(pad_after = 4)]
    string_table_size: u32,

    /// The entry table
    #[br(count = entry_count)]
    pub entry_table: Vec<PartitionFsEntry>,

    #[br(count = string_table_size)]
    _string_table: Vec<u8>,

    /// The position where the raw data is
    #[br(calc = entry_count as u64 * size_of_val(&entry_table) as u64 + string_table_size as u64 + 0x10)]
    pub raw_data_pos: u64,
}

pub trait PFSHeader {
    /// The entry type
    type Entry: PFSEntry;

    /// The position where the raw data is
    fn raw_data_pos(&self) -> u64;
    /// A vector of bytes containing all the names of the fs entries. Every name ends with a nul
    /// terminator
    fn string_table(&self) -> &[u8];
    /// The entry table
    fn entry_table(&self) -> &Vec<Self::Entry>;
}

pub trait PFSEntry {
    /// The offset where the entry name is
    fn string_offset(&self) -> u32;

    /// The size of the entry
    fn size(&self) -> u64;

    /// The offset relative to [raw_data_pos](PFSHeader::raw_data_pos)
    fn offset(&self) -> u64;
}

impl PFSHeader for PartitionFsHeader {
    type Entry = PartitionFsEntry;

    fn raw_data_pos(&self) -> u64 {
        self.raw_data_pos
    }

    fn string_table(&self) -> &[u8] {
        &self._string_table
    }

    fn entry_table(&self) -> &Vec<Self::Entry> {
        &self.entry_table
    }
}

/// A fs which has a magic value of `PFS0`
#[derive(Debug)]
pub struct PartitionFs<T: BinRead + PFSHeader> {
    pub header: T,
}

impl<T: BinRead + PFSHeader> PartitionFs<T> {
    /// Constructs a `PFS`. The header can be a [`PartitionFsHeader`] or a [`HashPartitionFsHeader`](super::hfs::HashPartitionFsHeader) or any struct implementing [`PFSHeader`] and `BinRead`.
    pub fn new(header: T) -> Self {
        Self { header }
    }

    /// Gets the name of the provided entry.
    pub fn get_name_for_entry<E: PFSEntry>(&self, entry: &E) -> Result<String, PartitionFsErrors> {
        let slice = &self.header.string_table()[entry.string_offset() as usize..];

        match slice.iter().position(|&b| b == 0) {
            Some(pos) => Ok(String::from_utf8(slice[..pos].to_vec())?),
            None => Err(PartitionFsErrors::NullTerminatorError),
        }
    }

    /// Returns a FileRegion representing the provided entry
    pub fn open_entry<R: ReadAt, E: PFSEntry>(&self, entry: &E, stream: R) -> FileRegion<R> {
        FileRegion::new(
            stream,
            entry.offset() + self.header.raw_data_pos(),
            entry.size(),
        )
    }
}

impl PartitionFs<PartitionFsHeader> {
    /// Constructs a partition file system. Use this to construct pfs with magic values of PFS0
    pub fn new_pfs0<R: Read + Seek>(
        stream: &mut R,
    ) -> Result<PartitionFs<PartitionFsHeader>, PartitionFsErrors> {
        let h = PartitionFsHeader::read(stream)?;

        Ok(PartitionFs::<PartitionFsHeader>::new(h))
    }
}
