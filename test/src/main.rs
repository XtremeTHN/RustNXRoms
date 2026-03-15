use binrw::BinRead;
use positioned_io::ReadAt;
use std::{
    fs::File,
    io::{Read, Seek, Write},
};

use log::info;

use nxroms::{
    formats::{
        cnmt, nacp::{Nacp, TitleLanguage}, nca::{self, Nca}, xci::Xci
    },
    fs::{
        pfs::{PFSHeader, PartitionFs},
        romfs::RomFs,
    },
    keyring::Keyring,
};

fn list_romfs_files(rom_fs: RomFs) {
    info!("Listing romfs files...");
    for (index, file) in rom_fs.files.iter().enumerate() {
        let name = String::from_utf8(file.name.clone()).expect("error while decoding name");
        info!("{}: {}", index, name);
    }
}

fn print_info<T: BinRead + PFSHeader, R: ReadAt + Read + Seek>(
    pfs: PartitionFs<T>,
    part: R,
) {
    let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
    keyring.parse().expect("error while parsing keyring");

    for (index, entry) in pfs.header.entry_table().iter().enumerate() {
        let name = pfs.get_name_for_entry(entry).expect("failed to get name:");
        info!("{}", name);

        let mut r = pfs.open_entry(entry, &part);

        let splitted = name.split(".").collect::<Vec<&str>>();
        let ext = splitted.last();
        if ext == Some(&"xml") {
            let mut out = File::create("out.xml").expect("fail");
            let mut buf = vec![];
            r.read_to_end(&mut buf);
            out.write_all(&buf);
        }
        
        if ext != Some(&"nca") {
            continue;
        }

        let mut nca = Nca::new(&keyring, &mut r).expect("err");
        match nca.header.content_type {
            nca::ContentType::Meta => {
                info!("found meta: {}", name);
                let mut fs = nca.open_fs(0, &mut r).expect("fail");
                let cnmt_pfs = PartitionFs::new_pfs0_header(&mut fs).expect("fail");

                let mut stream = cnmt_pfs.open_entry(&cnmt_pfs.header.entry_table[0], &mut fs);                    
                
                let cnmt_head = cnmt::PackagedContentMetaHeader::read(&mut stream).expect("fail");
                println!("{:#?}", cnmt_head);

                if cnmt_head.content_meta_type == cnmt::ContentMetaType::Patch {
                    info!("this package is an update");
                } else if cnmt_head.content_meta_type == cnmt::ContentMetaType::Application {
                    info!("this package is an application");
                } else if cnmt_head.content_meta_type == cnmt::ContentMetaType::AddOnContent {
                    info!("this package is a dlc");
                } else {
                    info!("unknown package type: {:?}", cnmt_head.content_meta_type);
                }

                let extended = cnmt::AddOnContentMetaExtendedHeader::read(&mut stream).expect("fail");
                println!("{:#?}", extended);
            }

            nca::ContentType::Control => {
                info!("found control: {}", name);
                let mut fs = nca.open_fs(0, &mut r).expect("err");
                let rom_fs = RomFs::new(&mut fs).expect("err");

                let mut raw_nacp = rom_fs.open_file(&rom_fs.files[0], &mut fs);
                let nacp = Nacp::read(&mut raw_nacp).expect("fail to parse nacp");
                
                let lang = TitleLanguage::from_system_locale().unwrap();

                info!("selected language: {:?}", lang);

                let title = &nacp.titles[lang as usize];
                info!("Title: {}", title.name().unwrap());
                info!("Version: {}", nacp.version().unwrap());
            }

            _ => {
                continue;
            }
        }
    }
}

fn xci_test() {
    let mut file = File::open("/home/axel/Projects/d/ori.xci").expect("er");
    let mut xci = Xci::new(&mut file).expect("err");

    let mut part = xci
        .open_partition("secure".to_string(), &file)
        .expect("err");
    let pfs = xci.open_partition_fs(&mut part).expect("");

    print_info(pfs, part);
}

fn nsp_test() {
    let mut file = File::open("undertale.nsp").expect("failed");
    let pfs = PartitionFs::new_pfs0_header(&mut file).expect("failed");
    let mut keyring = Keyring::new(String::from("~/.switch/prod.keys"));
    keyring.parse().expect("fail");

    print_info(pfs, &mut file);
}

fn main() {
    let env = env_logger::Env::default().filter_or("LIFT_LOG", "info");
    env_logger::init_from_env(env);
    nsp_test();
}
