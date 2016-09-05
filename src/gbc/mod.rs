pub mod cart;
pub mod cpu;
pub mod interconnect;

mod registers;
mod opcode;
mod ram;

#[derive(Debug)]
pub enum Model {
    Gb,
    Cgb,
}
