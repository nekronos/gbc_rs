pub mod cart;
pub mod cpu;
pub mod interconnect;
pub mod display;

mod disassembler;
mod registers;
mod opcode;
mod ram;

#[derive(Debug)]
pub enum GameboyType {
    Cgb,
    Gb,
}
