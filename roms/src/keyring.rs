//! `prod.keys` parser
//!
//! ```
//! use nxroms::keyring::Keyring;
//!
//! fn main() {
//!     let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
//!
//!     match keyring.parse() {
//!         Ok(()) => {
//!             /// Now you can use the prod keys
//!         }
//!         Err(err) => {
//!             eprintln!("Failed to parse keys: {}", err);
//!         }
//!     }
//! }
//! ```

use dirs::home_dir;
use hex::{FromHexError, decode};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyringErrors {
    #[error("Couldn't decode value: {0}")]
    HexDecoding(#[from] FromHexError),

    #[error("Couldn't decode key: {0}")]
    Utf8Decoding(#[from] FromUtf8Error),

    #[error("Failed to read: {0}")]
    Read(#[from] std::io::Error),

    #[error("Couldn't get home directory")]
    HomeDir,
}

/// A struct holding important keys.
/// Only the nca keys are stored
// TODO: Add support for all keys
#[derive(Default, Debug, Clone)]
pub struct Keyring {
    pub key_area_application: Vec<Vec<u8>>,
    pub key_area_ocean: Vec<Vec<u8>>,
    pub key_area_system: Vec<Vec<u8>>,
    pub header_key: Vec<u8>,
    path: PathBuf,
}

impl Keyring {
    /// Constructs a keyring with the given path.<br>
    /// You should use the [parse](Keyring::parse) method after construting
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Populates `self` with supported keys
    pub fn parse(&mut self) -> Result<(), KeyringErrors> {
        let path = if self.path.starts_with("~") {
            if let Some(home) = home_dir() {
                self.path
                    .to_string_lossy()
                    .replace("~", &home.to_string_lossy())
            } else {
                return Err(KeyringErrors::HomeDir);
            }
        } else {
            self.path.to_string_lossy().to_string()
        };

        let mut file = File::open(path)?;

        let mut buf = vec![];
        file.read_to_end(&mut buf)?;

        for raw_line in buf.split(|&b| b == b'\n') {
            let unsplitted = String::from_utf8(raw_line.to_vec())?;
            let line = unsplitted.split_once('=');

            if line.is_none() {
                continue;
            }

            let (key, val) = {
                let (_key, _val) = line.unwrap();

                (_key.replace(" ", ""), _val.replace(" ", ""))
            };

            if key.starts_with("key_area_key_application_") {
                self.key_area_application.push(decode(val).expect("err"));
                continue;
            }

            if key.starts_with("key_area_key_ocean_") {
                self.key_area_ocean.push(decode(val).expect("err"));
                continue;
            }

            if key.starts_with("key_area_key_system_") {
                self.key_area_system.push(decode(val).expect("err"));
                continue;
            }

            if key == "header_key" {
                self.header_key = decode(val).expect("err");
            }
        }

        Ok(())
    }
}
