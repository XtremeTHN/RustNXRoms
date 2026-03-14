use binrw::BinRead;

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
    DataPatch = 0x84
}

#[derive(BinRead, Debug, PartialEq, Eq)]
#[br(little, repr = u8)]
pub enum ContentMetaAttributes {
    IncludesExFatDriver = 0,
    Rebootless = 1,
    Compacted = 2,
    ProperProgramExists = 3,
    _4 = 4,
    _5 = 5
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
    pub req_sys_version: u32
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct AddOnContentMetaExtendedHeader {
    pub app_id: u64,
    #[br(pad_after = 0xC)]
    pub required_app_version: u32,
}