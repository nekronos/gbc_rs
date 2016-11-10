use std::fmt;
use std::fmt::Debug;
use std::string::String;
use std::boxed::Box;

use super::mbc::Mbc;
use super::GameboyType;

pub struct Cart {
    bytes: Box<[u8]>,
    mbc: Box<Mbc>,
}

#[derive(Debug)]
pub enum CartType {
    Rom,
    RomMbc1,
    RomMbc5RamBatt,
    Unsupported,
}

#[derive(Debug)]
pub enum DestinationCode {
    Japanese,
    NonJapanese,
}

impl Cart {
    pub fn new(bytes: Box<[u8]>) -> Cart {
        let mbc = super::mbc::new_mbc(Cart::get_cart_type(&bytes));
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

    pub fn cart_type(&self) -> CartType {
        Cart::get_cart_type(&self.bytes)
    }

    fn get_cart_type(bytes: &Box<[u8]>) -> CartType {
        match bytes[0x0147] {
            0x00 => CartType::Rom,
            0x01 => CartType::RomMbc1,
            0x1b => CartType::RomMbc5RamBatt,
            _ => CartType::Unsupported,
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

    pub fn ram_size(&self) -> u32 {
        match self.bytes[0x0149] {
            0 => 0,
            1 => 1024 * 2,
            2 => 1024 * 8,
            3 => 1024 * 32,
            4 => 1024 * 128,
            _ => panic!("Unsupported ram size: {:x}", self.bytes[0x0149]),
        }
    }

    pub fn ram_bank_count(&self) -> u32 {
        match self.bytes[0x0149] {
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
}

impl Debug for Cart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Cart {{
    title: {}
    type: {:?}
    size: {:?}
    bank_count: {:?}
    ram_size: {:?}
    ram_bank_count: {:?}
    destination_code: {:?}
}}",
               self.title(),
               self.cart_type(),
               self.rom_size(),
               self.rom_bank_count(),
               self.ram_size(),
               self.ram_bank_count(),
               self.destination_code())
    }
}
