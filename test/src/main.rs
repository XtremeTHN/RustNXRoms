use clap::Parser;
use nxroms::formats::cnmt::{self, PackagedContentMetaHeader};
use nxroms::formats::nacp::{Nacp, TitleLanguage};
use nxroms::formats::nca::{ContentType, Nca};
use nxroms::formats::xci::{Xci, XciPartition};
use nxroms::fs::pfs::{PFSHeader, PartitionFs};
use nxroms::fs::romfs::RomFs;
use nxroms::keyring::Keyring;
use nxroms::{BinRead, ReadAt};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
}

type GenericResult<T> = Result<T, Box<dyn Error>>;

#[derive(Default)]
struct RomInfo {
    title: Option<String>,
    version: Option<String>,
    publisher: Option<String>,
    application_type: Option<String>,
}

fn process<T, S>(pfs: PartitionFs<T>, stream: &mut S) -> GenericResult<RomInfo>
where
    T: BinRead + PFSHeader,
    S: ReadAt + Read + Seek,
{
    let mut keyring = Keyring::new("~/.switch/prod.keys");
    keyring.parse()?;

    let mut info = RomInfo::default();

    for entry in pfs.header.entry_table().iter() {
        let name = pfs.get_name_for_entry(entry)?;
        let ext = name.split(".").last();

        if ext != Some("nca") {
            println!("Ignoring \"{}\" as it's not a nca", name);
            continue;
        }

        let mut raw_nca = pfs.open_entry(entry, &stream);
        let mut nca = Nca::new(&keyring, &mut raw_nca)?;

        match nca.header.content_type {
            ContentType::Control => {
                println!("Found control nca: \"{}\"", name);
                let mut fs = nca.open_fs(0, &mut raw_nca)?;
                let romfs = RomFs::new(&mut fs)?;

                let first = romfs.files().nth(0);

                match first {
                    Some(f) => {
                        let nacp_entry = f?;
                        let mut raw_nacp = romfs.open_file(&nacp_entry, &mut fs);

                        let nacp = Nacp::read(&mut raw_nacp)?;

                        let language = TitleLanguage::from_system_locale()?;

                        let title = &nacp.titles[language as usize];

                        info.title = title.name().ok();
                        info.version = nacp.version().ok();
                        info.publisher = title.publisher().ok();
                    }
                    None => continue,
                }
            }

            ContentType::Meta => {
                println!("Found meta nca: \"{}\"", name);
                let mut fs = nca.open_fs(0, &mut raw_nca)?;
                let cnmt_pfs = PartitionFs::new_pfs0(&mut fs)?;

                let mut raw_cnmt = cnmt_pfs.open_entry(&cnmt_pfs.header.entry_table[0], &mut fs);
                let cnmt_header = PackagedContentMetaHeader::read(&mut raw_cnmt)?;

                info.application_type = Some(cnmt_header.content_meta_type.to_string());
            }
            _ => {}
        }
    }

    Ok(info)
}

fn process_nsp(file: &mut File) -> GenericResult<RomInfo> {
    let pfs = PartitionFs::new_pfs0(file)?;
    process(pfs, file)
}
fn process_xci(file: &mut File) -> GenericResult<RomInfo> {
    let mut xci = Xci::new(file)?;
    let mut fs = xci.open_partition(XciPartition::Secure, file)?;
    let secure = xci.open_partition_fs(&mut fs)?;

    process(secure, &mut fs)
}

fn main() {
    let args = Args::parse();

    let mut file = File::open(args.input.clone()).expect("failed to open rom");

    let extension = args.input.extension();

    match extension {
        Some(ext) => {
            let result = {
                if ext == "nsp" {
                    process_nsp(&mut file)
                } else if ext == "xci" {
                    process_xci(&mut file)
                } else {
                    eprintln!("Invalid extension: {}", ext.to_string_lossy());
                    std::process::exit(2);
                }
            };

            match result {
                Ok(info) => {
                    let placeholder = String::from("Unknown");
                    println!("\nRom information:");
                    println!("\tTitle: {}", info.title.as_ref().unwrap_or(&placeholder));
                    println!(
                        "\tPublisher: {}",
                        info.publisher.as_ref().unwrap_or(&placeholder)
                    );
                    println!(
                        "\tVersion: {}",
                        info.version.as_ref().unwrap_or(&placeholder)
                    );
                    println!(
                        "\tApplication Type: {}",
                        info.application_type.as_ref().unwrap_or(&placeholder)
                    );
                }
                Err(e) => {
                    eprintln!("Failed to parse rom: {}", e);
                    std::process::exit(3);
                }
            }
        }

        None => {
            eprintln!("Couldn't detect rom type");
            std::process::exit(1);
        }
    }
}
