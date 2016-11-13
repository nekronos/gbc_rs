mod mbc1;
mod mbc3;

use self::mbc1::Mbc1;
use self::mbc3::Mbc3;

#[derive(Debug)]
pub struct RamInfo {
    size: u32,
    bank_count: u32,
}

impl RamInfo {
    pub fn new(size: u32, bank_count: u32) -> RamInfo {
        RamInfo {
            size: size,
            bank_count: bank_count,
        }
    }
}

#[derive(Debug)]
pub struct MbcInfo {
    mbc_type: MbcType,
    ram_info: Option<RamInfo>,
    has_batt: bool,
}

impl MbcInfo {
    pub fn new(mbc_type: MbcType, ram_info: Option<RamInfo>, has_batt: bool) -> MbcInfo {
        MbcInfo {
            mbc_type: mbc_type,
            ram_info: ram_info,
            has_batt: has_batt,
        }
    }
}

#[derive(Debug)]
pub enum MbcType {
    None,
    Mbc1,
    Mbc3,
    Mbc5,
}

pub trait Mbc {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, val: u8);
}

pub fn new_mbc(mbc_info: MbcInfo) -> Box<Mbc> {
    match mbc_info.mbc_type {
        MbcType::None => Box::new(RomOnly {}),
        MbcType::Mbc1 => Box::new(Mbc1::new(mbc_info)),
        MbcType::Mbc3 => Box::new(Mbc3::new(mbc_info)),
        _ => panic!("Unsupported cart type"),
    }
}

struct RomOnly;

impl Mbc for RomOnly {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8 {
        rom[addr as usize]
    }

    #[allow(unused_variables)]
    fn write(&mut self, addr: u16, val: u8) {}

    #[allow(unused_variables)]
    fn read_ram(&self, addr: u16) -> u8 {
        0
    }

    #[allow(unused_variables)]
    fn write_ram(&mut self, addr: u16, val: u8) {}
}