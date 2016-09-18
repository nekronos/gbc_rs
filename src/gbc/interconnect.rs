use super::ppu::Ppu;
use super::cart::Cart;

const ZRAM_SIZE: usize = 0x7f;
const RAM_SIZE: usize = 1024 * 32;

pub struct Interconnect {
    cart: Cart,
    ppu: Ppu,
    ram: [u8; RAM_SIZE],
    zram: [u8; ZRAM_SIZE],
}

impl Interconnect {
    pub fn new(cart: Cart, ppu: Ppu) -> Interconnect {
        Interconnect {
            cart: cart,
            ppu: ppu,
            ram: [0; RAM_SIZE],
            zram: [0; ZRAM_SIZE],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x3fff => self.cart.read(addr),
            0xc000...0xdfff => self.ram[(addr - 0xc000) as usize],
            0xff00 => {
                // joypad
                0
            }
            0xff01...0xff02 => {
                // serial IO
                0
            }
            0xff40...0xff4b => self.ppu.read(addr),
            0xff4d => 0, // Speedswitch
            0xff80...0xfffe => self.zram[(addr - 0xff80) as usize],
            _ => panic!("Read: addr not in range: 0x{:x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x3fff => self.cart.write(addr, val),
            0xc000...0xdfff => self.ram[(addr - 0xc000) as usize] = val,
            0xff00 => {
                // joypad
            }
            0xff01...0xff02 => {
                // serial IO
            }
            0xff40...0xff4b => self.ppu.write(addr, val),
            0xff4d => {} // Speedswitch
            0xff80...0xfffe => self.zram[(addr - 0xff80) as usize] = val,
            _ => panic!("Write: addr not in range: 0x{:x} - val: 0x{:x}", addr, val),
        }
    }

    pub fn cycle_flush(&mut self, cycle_count: u64) {}
}
