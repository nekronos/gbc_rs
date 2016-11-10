use super::cart::CartType;

pub trait Mbc {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
}

pub fn new_mbc(cart_type: CartType) -> Box<Mbc> {
    match cart_type {
        CartType::Rom => Box::new(RomOnly {}),
        CartType::RomMbc1 => Box::new(Mbc1::new()),
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
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => rom[addr as usize],
            0x4000...0x7fff => {
                let addr = addr - 0x4000;
                let offset = (self.bank_select as u16) * 0x4000;
                let addr = addr + offset;
                rom[addr as usize]
            }
            _ => panic!("Mbc1::read: address out of range 0x{:x}", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x2000...0x3fff => self.bank_select = if (val & 0xf) == 0 { val | 0x01 } else { val },
            _ => panic!("Mbc1::write address out of range 0x{:x}", addr),
        }
    }
}
