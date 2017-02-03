mod mbc1;
mod mbc3;
mod mbc5;

use self::mbc1::Mbc1;
use self::mbc3::Mbc3;
use self::mbc5::Mbc5;

#[derive(Debug,Copy,Clone)]
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

    fn make_ram(self, save_ram: Option<Box<[u8]>>) -> Box<[u8]> {
        match save_ram {
            Some(ram) => {
                if ram.len() == self.size as usize {
                    ram
                } else {
                    panic!("save_ram size mismatch - expected {:#?}, but got: {:#?}",
                           self.size,
                           ram.len());
                }
            }
            None => vec![0; self.size as usize].into_boxed_slice(),
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
    Mbc2,
    Mbc3,
    Mbc5,
}

pub trait Mbc {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    fn read_ram(&self, addr: u16) -> u8;
    fn write_ram(&mut self, addr: u16, val: u8);
    fn copy_ram(&self) -> Option<Box<[u8]>>;
}

pub fn new_mbc(mbc_info: MbcInfo, ram: Option<Box<[u8]>>) -> Box<Mbc> {
    match mbc_info.mbc_type {
        MbcType::None => Box::new(RomOnly {}),
        MbcType::Mbc1 => Box::new(Mbc1::new(mbc_info, ram)),
        MbcType::Mbc3 => Box::new(Mbc3::new(mbc_info, ram)),
        MbcType::Mbc5 => Box::new(Mbc5::new(mbc_info, ram)),
        _ => panic!("{:?} not implemented!", mbc_info.mbc_type),
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

    fn copy_ram(&self) -> Option<Box<[u8]>> {
        None
    }
}
