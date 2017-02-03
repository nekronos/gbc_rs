use super::ppu::Ppu;
use super::spu::Spu;
use super::cart::Cart;
use super::timer::Timer;
use super::gamepad::Gamepad;
use super::GameboyType;

const ZRAM_SIZE: usize = 0x7f;
const RAM_SIZE: usize = 1024 * 32;

pub struct Interconnect {
    gameboy_type: GameboyType,
    pub cart: Cart,
    ppu: Ppu,
    spu: Spu,
    timer: Timer,
    gamepad: Gamepad,
    ram: Box<[u8]>,
    zram: Box<[u8]>,
    svbk: u8,
    ppu_dma: u8,
    pub int_enable: u8,
    pub int_flags: u8,
    ram_offset: usize,
}

impl Interconnect {
    pub fn new(gameboy_type: GameboyType,
               cart: Cart,
               ppu: Ppu,
               spu: Spu,
               gamepad: Gamepad)
               -> Interconnect {
        Interconnect {
            gameboy_type: gameboy_type,
            cart: cart,
            ppu: ppu,
            spu: spu,
            timer: Timer::new(),
            gamepad: gamepad,
            ram: vec![0; RAM_SIZE].into_boxed_slice(),
            zram: vec![0; ZRAM_SIZE].into_boxed_slice(),
            svbk: 0,
            ppu_dma: 0,
            int_enable: 0,
            int_flags: 0,
            ram_offset: 0,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000...0x7fff => self.cart.read(addr),
            0x8000...0x9fff => self.ppu.read(addr),
            0xa000...0xbfff => self.cart.read_ram(addr),
            0xc000...0xcfff => self.ram[(addr - 0xc000) as usize],
            0xd000...0xdfff => self.ram[(addr - 0xc000) as usize + self.ram_offset],
            0xe000...0xfdff => self.read(addr - 0xe000 + 0xc000),

            0xff00 => self.gamepad.read(),

            0xff01...0xff02 => {
                // serial IO
                0
            }
            0xff04...0xff07 => self.timer.read(addr),

            0xff10...0xff3f => self.spu.read(addr),

            0xff0f => self.int_flags,

            0xff46 => self.ppu_dma,

            0xfe00...0xfeff | 0xff40...0xff45 | 0xff47...0xff4b | 0xff68...0xff69 | 0xff4f => {
                self.ppu.read(addr)
            }

            0xff4d => 0, // Speedswitch
            0xff70 => self.svbk,
            0xff80...0xfffe => self.zram[(addr - 0xff80) as usize],
            0xffff => self.int_enable,
            _ => panic!("Read: addr not in range: 0x{:x}", addr),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000...0x7fff => self.cart.write(addr, val),
            0x8000...0x9fff => self.ppu.write(addr, val),
            0xa000...0xbfff => self.cart.write_ram(addr, val),
            0xc000...0xcfff => self.ram[(addr - 0xc000) as usize] = val,
            0xd000...0xdfff => self.ram[(addr - 0xc000) as usize + self.ram_offset] = val,
            0xe000...0xfdff => self.write(addr - 0xe000 + 0xc000, val),

            0xff00 => self.gamepad.write(val),

            0xff01...0xff02 => {
                // serial IO
                if addr == 0xff01 {
                    // print!("{}", val as char)
                }
            }
            0xff04...0xff07 => self.timer.write(addr, val),

            0xff10...0xff3f => self.spu.write(addr, val),

            0xff0f => self.int_flags = val,

            0xff46 => {
                self.ppu_dma = val;
                self.ppu_dma_transfer()
            }

            0xfe00...0xfeff | 0xff40...0xff45 | 0xff47...0xff4b | 0xff68...0xff69 | 0xff4f => {
                self.ppu.write(addr, val)
            }

            0xff4d => {} // Speedswitch
            0xff70 => {
                self.svbk = val & 0b111;
                self.update_ram_offset()
            }

            0xff7f => {} // TETRIS writes to this address for some reason

            0xff80...0xfffe => self.zram[(addr - 0xff80) as usize] = val,
            0xffff => self.int_enable = val,
            _ => panic!("Write: addr not in range: 0x{:x} - val: 0x{:x}", addr, val),
        }
    }

    pub fn cycle_flush(&mut self, cycle_count: u32) {

        let ppu_ints = self.ppu.cycle_flush(cycle_count);
        let timer_ints = self.timer.cycle_flush(cycle_count);
        let gamepad_ints = self.gamepad.cycle_flush(cycle_count);

        let interrupts = ppu_ints | timer_ints | gamepad_ints;

        self.int_flags |= interrupts.bits
    }

    fn ppu_dma_transfer(&mut self) {
        let dma_start = (self.ppu_dma as u16) << 8;
        let dma_end = dma_start | 0x009f;

        // if dma_start > 0x7fff && dma_end < 0xc000 {
        // panic!("Illegal DMA address range: 0x{:x} - 0x{:x}",
        // dma_start,
        // dma_end);
        // }

        let mut oam = vec![0; super::ppu::OAM_SIZE].into_boxed_slice();

        for a in dma_start..dma_end {
            oam[(a - dma_start) as usize] = self.read(a)
        }

        self.ppu.oam_dma_transfer(oam)
    }

    fn update_ram_offset(&mut self) {
        self.ram_offset = self.svbk as usize * 0x1000
    }
}
