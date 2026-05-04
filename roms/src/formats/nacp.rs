//! This format contains language entries in UTF-8
//!
//! More info: <https://switchbrew.org/wiki/NACP>
//!
//! Example:
//! ```
//! use nxroms::BinRead;
//! use nxroms::formats::nacp::Nacp;
//! use nxroms::formats::nca::Nca;
//! use nxroms::fs::romfs::RomFs;
//! use nxroms::keyring::Keyring;
//! use std::fs::File;
//!
//! fn main() {
//!     let mut file = File::open("00000000000000.nca").expect("failed to open nca");
//!
//!     let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
//!     keyring.parse().expect("failed to parse keyring");
//!
//!     // In this example im gonna assume this nca is the control nca
//!     let mut nca = Nca::new(&keyring, &mut file).expect("failed to parse nca");
//!
//!     let mut stream = nca.open_fs(0, &mut file).expect("failed to open fs");
//!
//!     let romfs = RomFs::new(&mut stream).expect("failed to construct romfs");
//!     let nacp_entry = romfs
//!         .files()
//!         .nth(0)
//!         .expect("no files")
//!         .expect("failed to parse file");
//!
//!     let mut raw_nacp = romfs.open_file(&nacp_entry, &mut stream);
//!     let nacp = Nacp::read(&mut raw_nacp).expect("failed to parse nacp");
//!
//!     // Do things with nacp
//! }
//! ```

use std::string::FromUtf8Error;
use strum_macros::Display;

use binrw::BinRead;

fn strip(bytes: &mut Vec<u8>) {
    bytes.retain(|&b| b != 0x00);
}

/// Represents a title language, used in the NACP format
#[derive(BinRead, PartialEq, Debug, Display, Eq, Clone, Copy)]
#[br(repr(u8))]
#[br(little)]
pub enum TitleLanguage {
    AmericanEnglish = 0,
    BritishEnglish = 1,
    Japanese = 2,
    French = 3,
    German = 4,
    LatinAmericanSpanish = 5,
    Spanish = 6,
    Italian = 7,
    Dutch = 8,
    CanadianFrench = 9,
    Portuguese = 10,
    Russian = 11,
    Korean = 12,
    TraditionalChinese = 13,
    SimplifiedChinese = 14,
    BrazilianPortuguese = 15,
    #[cfg(feature = "glib")]
    Automatic = 16,
}

#[derive(thiserror::Error, Debug)]
pub enum TitleLanguageErrors {
    #[error("This language is not supported by Nintendo.")]
    LanguageNotSupported,
}

#[cfg(feature = "glib")]
impl TitleLanguage {
    /// Attempts to determine the system locale and convert it to a TitleLanguage variant.
    pub fn from_system_locale() -> Result<Self, TitleLanguageErrors> {
        let languages = glib::language_names();
        let raw_lang = languages.first().map(|s| s.as_ref()).unwrap_or("en_US");

        let parts = raw_lang.split(".").collect::<Vec<&str>>();
        let language = parts[0];

        match language {
            "en_US" => Ok(Self::AmericanEnglish),
            "en_GB" => Ok(Self::BritishEnglish),
            "ja" | "ja_JP" => Ok(Self::Japanese),
            "fr" | "fr_FR" => Ok(Self::French),
            "de" | "de_DE" => Ok(Self::German),
            "es_419" => Ok(Self::LatinAmericanSpanish),
            "es_ES" => Ok(Self::Spanish),
            "it" | "it_IT" => Ok(Self::Italian),
            "nl" | "nl_NL" => Ok(Self::Dutch),
            "fr_CA" => Ok(Self::CanadianFrench),
            "pt_PT" => Ok(Self::Portuguese),
            "ru" | "ru_RU" => Ok(Self::Russian),
            "ko" | "ko_KR" => Ok(Self::Korean),
            "zh_TW" | "zh_HK" => Ok(Self::TraditionalChinese),
            "zh_CN" | "zh_SG" => Ok(Self::SimplifiedChinese),
            "pt_BR" => Ok(Self::BrazilianPortuguese),
            _ => Err(TitleLanguageErrors::LanguageNotSupported),
        }
    }
}

impl TryFrom<i32> for TitleLanguage {
    type Error = TitleLanguageErrors;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::AmericanEnglish),
            1 => Ok(Self::BritishEnglish),
            2 => Ok(Self::Japanese),
            3 => Ok(Self::French),
            4 => Ok(Self::German),
            5 => Ok(Self::LatinAmericanSpanish),
            6 => Ok(Self::Spanish),
            7 => Ok(Self::Italian),
            8 => Ok(Self::Dutch),
            9 => Ok(Self::CanadianFrench),
            10 => Ok(Self::Portuguese),
            11 => Ok(Self::Russian),
            12 => Ok(Self::Korean),
            13 => Ok(Self::TraditionalChinese),
            14 => Ok(Self::SimplifiedChinese),
            15 => Ok(Self::BrazilianPortuguese),
            #[cfg(feature = "glib")]
            16 => Ok(Self::Automatic),
            _ => Err(TitleLanguageErrors::LanguageNotSupported),
        }
    }
}

/// Represents a title entry in the NACP format
#[derive(BinRead, Debug)]
#[br(little)]
pub struct Title {
    /// The name of the title, in bytes. Some bytes might be null.
    #[br(count = 0x200)]
    pub raw_name: Vec<u8>,
    /// The publisher of the title, in bytes. Some bytes might be null.
    #[br(count = 0x100)]
    pub raw_publisher: Vec<u8>,
}

impl Title {
    /// Returns the name of the title as a String, stripping all null bytes.
    pub fn name(&self) -> Result<String, FromUtf8Error> {
        let mut n = self.raw_name.clone();
        strip(&mut n);
        String::from_utf8(n)
    }

    /// Returns the publisher of the title as a String, stripping all null bytes.
    pub fn publisher(&self) -> Result<String, FromUtf8Error> {
        let mut n = self.raw_publisher.clone();
        strip(&mut n);
        String::from_utf8(n)
    }
}

/// A format used in the control nca, containing title information and other metadata about the application.
#[derive(BinRead, Debug)]
#[br(little)]
pub struct Nacp {
    /// A vector of all available title languages. Some of them might be empty.
    #[br(count = 16)]
    pub titles: Vec<Title>,

    /// The version of the application, in bytes.
    #[br(count = 0x10, seek_before = std::io::SeekFrom::Start(0x3060))]
    pub raw_version: Vec<u8>,
}

impl Nacp {
    /// Returns the version of the application as a String, stripping all null bytes.
    pub fn version(&self) -> Result<String, FromUtf8Error> {
        let mut n = self.raw_version.clone();
        strip(&mut n);

        String::from_utf8(n)
    }
}
