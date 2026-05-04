//! A rust crate for parsing a variety of nintendo switch formats.
//!
//! You can check out the source code of <https://github.com/XtremeTHN/Lift> for an example of how to use this crate.
//!
//! Example (with glib feature):
//! ```
//! use binrw::BinRead;
//! use positioned_io::ReadAt;
//! use std::{
//!     fs::File,
//!     io::{Read, Seek},
//! };
//!
//! use log::info;
//!
//! use nxroms::{
//!     formats::{
//!         nacp::{Nacp, TitleLanguage},
//!         nca::{self, Nca},
//!     },
//!     fs::{
//!         pfs::{PFSHeader, PartitionFs},
//!         romfs::RomFs,
//!     },
//!     keyring::Keyring,
//! };
//!
//! fn print_info<T: BinRead + PFSHeader, R: ReadAt + Read + Seek>(pfs: PartitionFs<T>, part: R) {
//!     let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
//!     keyring.parse().expect("error while parsing keyring");
//!
//!     for entry in pfs.header.entry_table().iter() {
//!         let name = pfs.get_name_for_entry(entry).expect("failed to get name:");
//!         info!("{}", name);
//!
//!         let mut r = pfs.open_entry(entry, &part);
//!
//!         let splitted = name.split(".").collect::<Vec<&str>>();
//!         let ext = splitted.last();
//!
//!         if ext != Some(&"nca") {
//!             continue;
//!         }
//!
//!         let mut nca = Nca::new(&keyring, &mut r).expect("err");
//!         match nca.header.content_type {
//!             nca::ContentType::Control => {
//!                 info!("found control: {}", name);
//!                 let mut fs = nca.open_fs(0, &mut r).expect("err");
//!                 let rom_fs = RomFs::new(&mut fs).expect("err");
//!
//!                 let first_file = rom_fs
//!                     .files()
//!                     .nth(0)
//!                     .expect("no files")
//!                     .expect("failed to parse file");
//!                 let mut raw_nacp = rom_fs.open_file(&first_file, &mut fs);
//!                 let nacp = Nacp::read(&mut raw_nacp).expect("fail to parse nacp");
//!
//!                 let lang = TitleLanguage::from_system_locale().unwrap();
//!
//!                 info!("selected language: {:?}", lang);
//!
//!                 let title = &nacp.titles[lang as usize];
//!                 info!("Title: {}", title.name().unwrap());
//!                 info!("Version: {}", nacp.version().unwrap());
//!             }
//!
//!             _ => {
//!                 continue;
//!             }
//!         }
//!     }
//! }
//!
//! fn nsp_test() {
//!     let mut file = File::open("rom.nsp").expect("failed");
//!     let pfs = PartitionFs::new_pfs0(&mut file).expect("failed");
//!     let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
//!     keyring.parse().expect("fail");
//!
//!     print_info(pfs, &mut file);
//! }
//!
//! fn main() {
//!     let env = env_logger::Env::default().filter_or("LIFT_LOG", "info");
//!     env_logger::init_from_env(env);
//!     nsp_test();
//! }
//! ```

pub mod formats;
pub mod fs;

pub mod crypto;
pub mod keyring;
pub mod readers;

pub use binrw::BinRead;
pub use positioned_io::ReadAt;
