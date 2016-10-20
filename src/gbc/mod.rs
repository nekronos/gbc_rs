pub mod cart;
pub mod cpu;
pub mod ppu;
pub mod spu;
pub mod interconnect;
pub mod gamepad;

mod disassembler;
mod registers;
mod opcode;
mod timer;

#[derive(Debug)]
pub enum GameboyType {
    Cgb,
    Gb,
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

#[allow(dead_code)]
pub enum Interrupt {
    VBlank,
    LCDStat,
    TimerOverflow,
    Serial,
    Joypad,
}
