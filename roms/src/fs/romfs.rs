//! A filesystem found in ncas
//! 
//! You can find more information here: https://www.3dbrew.org/wiki/RomFS
//! 
//! Example:
//! ```
//! use nxroms::fs::romfs::RomFs;
//! use nxroms::formats::nca::Nca;
//! use nxroms::keyring::Keyring;
//! use std::fs::File;
//! 
//! 
//! fn main() {
//!     let mut file = File::open("00000000000000.nca").expect("fail to open nca");
//!     
//!     let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
//!     keyring.parse();
//! 
//!     // In this example im gonna assume this nca is the control nca
//!     let mut nca = Nca::new(&keyring, &mut file).expect("fail to parse nca");
//! 
//!     let mut fs = nca.open_fs(0, &mut file).expect("fail to open fs 0");
//!     let romfs = RomFs::new(&mut fs).expect("fail to construct RomFs");
//! 
//!     let first_file = romfs.open_file(romfs.files.first().expect("no files"), &mut fs);
//! 
//!     // Do things with first_file
//! }
//! ```

use std::{
    io::{Cursor, Read, Seek},
    string::FromUtf8Error,
};

use binrw::BinRead;
use positioned_io::ReadAt;

use crate::readers::FileRegion;

/// The header of the RomFs
#[derive(BinRead, Debug)]
#[br(little)]
pub struct RomFsHeader {
    #[br(assert(header_size == 80))]
    pub header_size: u64,

    pub dir_hash_table_offset: u64,
    pub dir_hash_table_size: u64,

    pub dir_meta_table_offset: u64,
    pub dir_meta_table_size: u64,

    pub file_hash_table_offset: u64,
    pub file_hash_table_size: u64,

    pub file_meta_table_offset: u64,
    pub file_meta_table_size: u64,

    pub data_offset: u64,
}


/// Information of a file in a romfs
#[derive(BinRead)]
#[br(little)]
pub struct RomFsFileEntry {
    pub parent: u32,
    /// The offset of the sibling
    pub sibling: u32,
    /// The file data offset relative to [data_offset](RomFsHeader::data_offset)
    pub offset: u64,
    /// The size of the file
    pub size: u64,
    /// The has of the file
    pub hash: u32,
    /// The string size of the file name
    pub name_size: u32,

    /// The name of the file in bytes
    #[br(count = name_size)]
    pub _name: Vec<u8>,
}

impl RomFsFileEntry {
    /// Returns a string containing the file name
    pub fn name(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self._name.clone())
    }
}

pub struct RomFsFileIter<'a> {
    meta_table: &'a [u8],
    current_offset: Option<u32>
}

impl<'a> Iterator for RomFsFileIter<'a> {
    type Item = Result<RomFsFileEntry, RomFsErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.current_offset?;

        let slice = &self.meta_table[offset as usize..];

        let mut cur = Cursor::new(slice);
        let f = RomFsFileEntry::read(&mut cur);

        match f {
            Ok(file) => {
                self.current_offset = if file.sibling != u32::MAX {
                    Some(file.sibling)
                } else {
                    None
                };

                Some(Ok(file))
            }
            Err(e) => {
                self.current_offset = None;
                Some(Err(RomFsErrors::CorruptRomFs(e)))
            }
        }
    }
}

// #[derive(BinRead, Debug)]
// #[br(little)]
// pub struct RomFsDirectoryEntry {
//     pub parent: u32,
//     pub sibling: u32,
//     pub child: u32,
//     pub file: u32,
//     pub hash: u32,
//     pub name_size: u32,

//     #[br(count = name_size)]
//     pub name: Vec<u8>,
// }

#[derive(thiserror::Error, Debug)]
pub enum RomFsErrors {
    #[error("The romfs is invalid/corrupted")]
    CorruptRomFs(#[from] binrw::Error),
    #[error("Failed to read: {0:?}")]
    Read(#[from] std::io::Error),
}

/// A romfs. Only files are supported
// TODO: add directories support
pub struct RomFs {
    pub header: RomFsHeader,
    pub meta_table: Vec<u8>
    // pub files: Vec<RomFsFileEntry>,
}

impl RomFs {
    pub fn new<T: ReadAt + Read + Seek>(stream: &mut T) -> Result<Self, RomFsErrors> {
        let header = RomFsHeader::read(stream)?;
        let mut meta_table = vec![0u8; header.file_meta_table_size as usize];

        stream.read_at(header.file_meta_table_offset, &mut meta_table)?;

        let r = RomFs {
            header,
            meta_table
        };

        Ok(r)
    }

    pub fn files(&self) -> RomFsFileIter<'_> {
        RomFsFileIter { meta_table: &self.meta_table, current_offset: Some(0) }
    }

    /// Opens a romfs file entry 
    pub fn open_file<T: ReadAt>(&self, file: &RomFsFileEntry, stream: T) -> FileRegion<T> {
        FileRegion::new(stream, self.header.data_offset + file.offset, file.size)
    }
}
