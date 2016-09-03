pub mod cart;
pub mod cpu;
pub mod interconnect;
pub mod speed_switch;

mod opcode;

#[derive(Debug)]
pub enum Model {
    Gb,
    Cgb,
}
