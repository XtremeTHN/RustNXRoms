//! A metadata file format
//!
//! More info: <https://switchbrew.org/wiki/CNMT>
//!
//! Example:
//! ```
//! use nxroms::BinRead;
//! use nxroms::formats::cnmt::PackagedContentMetaHeader;
//! use nxroms::formats::nca::Nca;
//! use nxroms::fs::pfs::PartitionFs;
//! use nxroms::keyring::Keyring;
//! use std::fs::File;
//!
//! fn main() {
//!     let mut file = File::open("00000000000000.nca").expect("failed to open nca");
//!
//!     let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
//!     keyring.parse().expect("failed to parse keyring");
//!
//!     // In this example im gonna assume this nca is the meta nca
//!     let mut nca = Nca::new(&keyring, &mut file).expect("failed to parse nca");
//!
//!     let mut stream = nca.open_fs(0, &mut file).expect("failed to open fs");
//!     let cnmt_pfs = PartitionFs::new_pfs0(&mut stream).expect("failed to construct pfs");
//!
//!     let mut cnmt = cnmt_pfs.open_entry(&cnmt_pfs.header.entry_table[0], &mut stream);
//!
//!     let cnmt_header = PackagedContentMetaHeader::read(&mut cnmt).expect("failed to parse ");
//!
//!     // Do things with cnmt_header
//! }
//! ```

use binrw::BinRead;

/// The content meta type
#[derive(BinRead, Debug, PartialEq, Eq)]
#[br(little, repr = u8)]
pub enum ContentMetaType {
    Invalid = 0,
    SystemProgram = 0x01,
    SystemData = 0x02,
    SystemUpdate = 0x03,
    BootImagePackage = 0x04,
    BootImagePackageSafe = 0x05,
    Application = 0x80,
    Patch = 0x81,
    AddOnContent = 0x82,
    Delta = 0x83,
    DataPatch = 0x84,
}

#[derive(BinRead, Debug, PartialEq, Eq)]
#[br(little, repr = u8)]
pub enum ContentMetaAttributes {
    IncludesExFatDriver = 0,
    Rebootless = 1,
    Compacted = 2,
    ProperProgramExists = 3,
    _4 = 4,
    _5 = 5,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PackagedContentMetaHeader {
    pub id: u64,
    pub version: u32,
    pub content_meta_type: ContentMetaType,
    #[br(pad_before = 0x1)]
    pub ext_header_size: u16,
    pub content_count: u16,
    pub content_meta_count: u16,
    pub content_meta_attributes: ContentMetaAttributes,
    #[br(pad_before = 0x3, pad_after = 0x4)]
    pub req_sys_version: u32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct AddOnContentMetaExtendedHeader {
    pub app_id: u64,
    #[br(pad_after = 0xC)]
    pub required_app_version: u32,
}
