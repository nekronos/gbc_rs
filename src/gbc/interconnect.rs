use super::ppu::Ppu;
use super::spu::Spu;
use super::cart::Cart;
use super::timer::Timer;
use super::Interrupt;

const ZRAM_SIZE: usize = 0x7f;
const RAM_SIZE: usize = 1024 * 32;

pub struct Interconnect {
    cart: Cart,
    ppu: Ppu,
    spu: Spu,
    timer: Timer,
    ram: [u8; RAM_SIZE],
    zram: [u8; ZRAM_SIZE],
    pub int_enable: u8,
    pub int_flags: u8,
}

impl Interconnect {
    pub fn new(cart: Cart, ppu: Ppu, spu: Spu) -> Interconnect {
        Interconnect {
            cart: cart,
            ppu: ppu,
            spu: spu,
            timer: Timer::new(),
            ram: [0; RAM_SIZE],
            zram: [0; ZRAM_SIZE],
            int_enable: 0,
            int_flags: 0,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7fff => self.cart.read(addr),
            0xc000...0xdfff => self.ram[(addr - 0xc000) as usize],

            0x8000...0x9fff => {
                // VRAM
                0
            }

            0xff00 => {
                // joypad
                0
            }
            0xff01...0xff02 => {
                // serial IO
                0
            }
            0xff04...0xff07 => self.timer.read(addr),
            0xff0f => self.int_flags,
            0xff40...0xff4b | 0xff68...0xff69 => self.ppu.read(addr),
            0xff4d => 0, // Speedswitch
            0xff80...0xfffe => self.zram[(addr - 0xff80) as usize],
            0xffff => self.int_enable,
            _ => panic!("Read: addr not in range: 0x{:x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x7fff => self.cart.write(addr, val),
            0xc000...0xdfff => self.ram[(addr - 0xc000) as usize] = val,
            0x8000...0x97ff => {
                // Character Data
            }
            0x9800...0x9bff => {
                // BG Display Data 1
            }
            0xff00 => {
                // joypad
            }
            0xff01...0xff02 => {
                // serial IO
                if addr == 0xff01 {
                    print!("{}", val as char)
                }
            }
            0xff04...0xff07 => self.timer.write(addr, val),
            0xff0f => self.int_flags = val,
            0xff24...0xff26 => self.spu.write(addr, val),
            0xff40...0xff4b | 0xff68...0xff69 => self.ppu.write(addr, val),
            0xff4d => {} // Speedswitch
            0xff4f => {} // VBK, vram bank select
            0xff80...0xfffe => self.zram[(addr - 0xff80) as usize] = val,
            0xffff => self.int_enable = val,
            _ => panic!("Write: addr not in range: 0x{:x} - val: 0x{:x}", addr, val),
        }
    }

    pub fn cycle_flush(&mut self, cycle_count: u32) {
        if let Some(int) = self.ppu.cycle_flush(cycle_count) {
            self.request_interrupt(int)
        }

        if let Some(int) = self.timer.cycle_flush(cycle_count) {
            self.request_interrupt(int)
        }
    }

    fn request_interrupt(&mut self, int: Interrupt) {
        use super::Interrupt::*;
        self.int_flags |= {
            match int {
                VBlank => 0b0_0001,
                LCDStat => 0b0_0010,
                TimerOverflow => 0b0_0100,
                Serial => 0b0_1000,
                Joypad => 0b1_0000,
            }
        }
    }
}
