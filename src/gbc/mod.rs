pub mod cart;
pub mod cpu;
pub mod ppu;
pub mod spu;
pub mod interconnect;

mod disassembler;
mod registers;
mod opcode;
mod timer;

#[derive(Debug)]
pub enum GameboyType {
    Cgb,
    Gb,
}

#[derive(Debug,Copy,Clone)]
pub enum CpuSpeed {
    Normal,
    Double,
}

impl CpuSpeed {
    pub fn value(self) -> u32 {
        match self {
            CpuSpeed::Normal => 4_194_304,
            CpuSpeed::Double => 8_388_608,
        }
    }
}

pub enum Interrupt {
    VBlank,
    LCDStat,
    TimerOverflow,
    Serial,
    Joypad,
}
