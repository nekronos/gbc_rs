use std::string::String;
use std::boxed::Box;
use super::GameboyType;

#[derive(Debug)]
pub struct Cart {
    bytes: Box<[u8]>,
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

impl Cart {
    pub fn new(bytes: Box<[u8]>) -> Cart {
        Cart { bytes: bytes }
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
        match self.bytes[0x0147] {
            0x01 => CartType::RomMbc1,
            0x1b => CartType::RomMbc5RamBatt,
            _ => CartType::Unsupported,
        }
    }

    pub fn rom_size(&self) -> u32 {
        match self.bytes[0x0148] {
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
        self.bytes[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.bytes[addr as usize] = val
    }
}
