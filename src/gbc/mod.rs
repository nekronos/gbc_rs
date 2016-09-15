pub mod cart;
pub mod cpu;
pub mod ppu;
pub mod interconnect;

mod disassembler;
mod registers;
mod opcode;

#[derive(Debug)]
pub enum GameboyType {
    Cgb,
    Gb,
}
