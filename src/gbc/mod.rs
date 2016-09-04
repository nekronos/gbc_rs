pub mod cart;
pub mod cpu;
pub mod interconnect;

mod opcode;

#[derive(Debug)]
pub enum Model {
    Gb,
    Cgb,
}
