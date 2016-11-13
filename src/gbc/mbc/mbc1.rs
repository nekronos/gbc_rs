use super::Mbc;
use super::MbcInfo;

#[derive(Debug)]
pub struct Mbc1 {
    bank_select: u8,
    ram: Option<Box<[u8]>>,
}

impl Mbc1 {
    pub fn new(mbc_info: MbcInfo) -> Mbc1 {
        Mbc1 {
            bank_select: 0x01,
            ram: if let Some(ram_info) = mbc_info.ram_info {
                Some(vec![0; ram_info.size as usize].into_boxed_slice())
            } else {
                None
            },
        }
    }
}

impl Mbc for Mbc1 {
    fn read(&self, rom: &Box<[u8]>, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => rom[addr as usize],
            0x4000...0x7fff => {
                rom[(addr as u32 - 0x4000 + self.bank_select as u32 * 0x4000) as usize]
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

    #[allow(unused_variables)]
    fn read_ram(&self, addr: u16) -> u8 {
        0
    }

    #[allow(unused_variables)]
    fn write_ram(&mut self, addr: u16, val: u8) {}
}