use std::fmt;
use std::fmt::Debug;
use std::string::String;
use std::boxed::Box;

use super::mbc::Mbc;
use super::mbc::MbcType;
use super::mbc::RamInfo;
use super::mbc::MbcInfo;
use super::GameboyType;

pub struct Cart {
    bytes: Box<[u8]>,
    mbc: Box<Mbc>,
}

#[derive(Debug)]
pub enum DestinationCode {
    Japanese,
    NonJapanese,
}

impl Cart {
    pub fn new(bytes: Box<[u8]>) -> Cart {
        let mbc_info = Cart::get_mbc_info(&bytes);
        let mbc = super::mbc::new_mbc(mbc_info);
        Cart {
            bytes: bytes,
            mbc: mbc,
        }
    }

    pub fn title(&self) -> String {
        let mut title = Vec::new();
        for i in 0x0134..0x0143 {
            title.push(self.bytes[i]);
        }
        String::from_utf8(title).unwrap()
    }

    pub fn mbc_info(&self) -> MbcInfo {
        Cart::get_mbc_info(&self.bytes)
    }

    fn get_mbc_info(bytes: &Box<[u8]>) -> MbcInfo {
        let ram_info = if Cart::get_ram_size(&bytes) != 0 {
            Some(RamInfo::new(Cart::get_ram_size(&bytes), Cart::get_ram_bank_count(&bytes)))
        } else {
            None
        };
        match bytes[0x0147] {
            0x00 => MbcInfo::new(MbcType::None, ram_info, false),
            0x01 => MbcInfo::new(MbcType::Mbc1, ram_info, false),
            0x02 => MbcInfo::new(MbcType::Mbc1, ram_info, false),
            0x03 => MbcInfo::new(MbcType::Mbc1, ram_info, true),
            0x13 => MbcInfo::new(MbcType::Mbc3, ram_info, true),
            0x19 => MbcInfo::new(MbcType::Mbc5, ram_info, false),
            0x1b => MbcInfo::new(MbcType::Mbc5, ram_info, true),
            _ => panic!("Unsupported mbc_info: 0x{:x}", bytes[0x0147]),
        }
    }

    pub fn rom_size(&self) -> u32 {
        match self.bytes[0x0148] {
            0 => 1024 * 32,
            1 => 1024 * 64,
            2 => 1024 * 128,
            3 => 1024 * 256,
            4 => 1024 * 512,
            5 => 1024 * 1024,
            6 => 1024 * 1024 * 2,
            _ => panic!("Unsupported rom size: {:x}", self.bytes[0x0148]),
        }
    }

    pub fn rom_bank_count(&self) -> u32 {
        self.rom_size() / (1024 * 16)
    }

    #[allow(dead_code)]
    pub fn ram_size(&self) -> u32 {
        Cart::get_ram_size(&self.bytes)
    }

    fn get_ram_size(bytes: &Box<[u8]>) -> u32 {
        match bytes[0x149] {
            0 => 0,
            1 => 1024 * 2,
            2 => 1024 * 8,
            3 => 1024 * 32,
            4 => 1024 * 128,
            _ => panic!("Unsupported ram size: {:x}", bytes[0x0149]),
        }
    }

    #[allow(dead_code)]
    pub fn ram_bank_count(&self) -> u32 {
        Cart::get_ram_bank_count(&self.bytes)
    }

    fn get_ram_bank_count(bytes: &Box<[u8]>) -> u32 {
        match bytes[0x0149] {
            0 => 0,
            1 | 2 => 1,
            3 => 4,
            4 => 16,
            _ => panic!("Unsupported ram size"),
        }
    }

    pub fn destination_code(&self) -> DestinationCode {
        match self.bytes[0x014a] {
            0 => DestinationCode::Japanese,
            1 => DestinationCode::NonJapanese,
            _ => panic!("Unsupported destination code"),
        }
    }

    #[allow(dead_code)]
    pub fn gameboy_type(&self) -> GameboyType {
        match self.bytes[0x0143] {
            // TODO: confirm that this is correct
            0x80 | 0xc0 => GameboyType::Cgb,
            _ => GameboyType::Dmg,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(&self.bytes, addr)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.mbc.write(addr, val)
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        self.mbc.read_ram(addr)
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        self.mbc.write_ram(addr, val)
    }
}

impl Debug for Cart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Cart {{
    title: {},
    mbc_info: {:?},
    size: {:?},
    bank_count: {:?},
    destination_code: {:?},
}}",
               self.title(),
               self.mbc_info(),
               self.rom_size(),
               self.rom_bank_count(),
               self.destination_code())
    }
}
