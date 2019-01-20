pub mod cart;
pub mod cpu;
pub mod ppu;
pub mod spu;
pub mod interconnect;
pub mod gamepad;
pub mod console;

mod disassembler;
mod registers;
mod opcode;
mod timer;
mod mbc;

#[derive(Debug,Copy,Clone)]
pub enum GameboyType {
    Cgb,
    Dmg,
}

#[allow(dead_code)]
#[derive(Debug,Copy,Clone)]
pub enum CpuClock {
    Normal,
    Double,
}

impl CpuClock {
    #[allow(dead_code)]
    pub fn value(self) -> u32 {
        match self {
            CpuClock::Normal => 4_194_304,
            CpuClock::Double => 8_388_608,
        }
    }
}

bitflags! {
    pub flags Interrupts: u8 {
        const INT_VBLANK = 0b00001,
        const INT_LCDSTAT = 0b00010,
        const INT_TIMEROVERFLOW = 0b00100,
        const INT_SERIAL = 0b01000,
        const INT_JOYPAD = 0b10000,
    }
}
