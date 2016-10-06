use std::string::String;
use std::boxed::Box;
use super::GameboyType;

pub struct Cart {
    bytes: Box<[u8]>,
    mbc: Box<Mbc>,
}

#[derive(Debug)]
pub enum CartType {
    RomMbc1,
    RomMbc5RamBatt,
    Unsupported,
}

#[derive(Debug)]
pub enum DestinationCode {
    Japanese,
    NonJapanese,
}

trait Mbc {
    fn read(&self, cart: &Cart, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

#[derive(Debug)]
struct Mbc1 {
    bank_select: u8,
}

impl Mbc1 {
    pub fn new() -> Mbc1 {
        Mbc1 { bank_select: 0x01 }
    }
}

impl Mbc for Mbc1 {
    fn read(&self, cart: &Cart, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => cart.bytes[addr as usize],
            0x4000...0x7fff => {
                let addr = addr - 0x4000;
                let offset = (self.bank_select as u16) * 0x4000;
                let addr = addr + offset;
                cart.bytes[addr as usize]
            }
            _ => panic!("Mbc1::read: address out of range 0x{:x}", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x2000...0x3fff => {
                println!("Switching to bank: 0x{:x}", val);
                self.bank_select = val | 0x01
            } 
            _ => panic!("Mbc1::write address out of range 0x{:x}", addr),
        }
    }
}

impl Cart {
    pub fn new(bytes: Box<[u8]>) -> Cart {
        let mbc = {
            match Cart::get_cart_type(&bytes) {
                CartType::RomMbc1 => Box::new(Mbc1::new()),
                _ => panic!("Unsupported cart type"),
            }
        };
        Cart {
            bytes: bytes,
            mbc: mbc,
        }
    }

    pub fn title(&self) -> String {
        let mut title = Vec::new();
        let mut offset = 0x0134;

        while offset <= 0x0142 {
            let byte = self.bytes[offset];

            if byte == 0x00 {
                break;
            }

            title.push(byte);
            offset = offset + 1;
        }

        String::from_utf8(title).unwrap()
    }

    pub fn cart_type(&self) -> CartType {
        Cart::get_cart_type(&self.bytes)
    }

    fn get_cart_type(bytes: &Box<[u8]>) -> CartType {
        match bytes[0x0147] {
            0x01 => CartType::RomMbc1,
            0x1b => CartType::RomMbc5RamBatt,
            _ => CartType::Unsupported,
        }
    }

    pub fn rom_size(&self) -> u32 {
        match self.bytes[0x0148] {
            0x00 => 1024 * 32,
            0x01 => 1024 * 64,
            0x05 => 1024 * 1024,
            _ => panic!("Unsupported rom size"),
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
            _ => panic!("Unsupported ram size"),
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
            _ => GameboyType::Gb,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.mbc.read(self, addr)
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.mbc.write(addr, val)
    }
}
