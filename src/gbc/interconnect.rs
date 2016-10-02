use super::ppu::Ppu;
use super::spu::Spu;
use super::cart::Cart;

const ZRAM_SIZE: usize = 0x7f;
const RAM_SIZE: usize = 1024 * 32;

pub struct Interconnect {
    cart: Cart,
    ppu: Ppu,
    spu: Spu,
    ram: [u8; RAM_SIZE],
    zram: [u8; ZRAM_SIZE],
    tima: u8,
    tma: u8,
    tac: u8,
}

impl Interconnect {
    pub fn new(cart: Cart, ppu: Ppu, spu: Spu) -> Interconnect {
        Interconnect {
            cart: cart,
            ppu: ppu,
            spu: spu,
            ram: [0; RAM_SIZE],
            zram: [0; ZRAM_SIZE],
            tima: 0,
            tma: 0,
            tac: 0,
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
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            0xff40...0xff4b | 0xff68...0xff69 => self.ppu.read(addr),
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
            0xff05 => self.tima = val,
            0xff06 => self.tma = val,
            0xff07 => self.write_tac(val),
            0xff24...0xff26 => self.spu.write(addr, val),
            0xff40...0xff4b | 0xff68...0xff69 => self.ppu.write(addr, val),
            0xff4d => {} // Speedswitch
            0xff4f => {} // VBK, vram bank select
            0xff80...0xfffe => self.zram[(addr - 0xff80) as usize] = val,
            _ => panic!("Write: addr not in range: 0x{:x} - val: 0x{:x}", addr, val),
        }
    }

    pub fn cycle_flush(&mut self, cycle_count: u32) {
        self.ppu.cycle_flush(cycle_count)
    }

    fn write_tac(&mut self, val: u8) {
        self.tac = val
    }
}
