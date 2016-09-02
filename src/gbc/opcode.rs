
#[derive(Debug)]
pub enum Opcode {
    Nop = 0x00,
    Jp = 0xc3,
}

pub fn to_opcode(x: u8) -> Opcode {
    match x {
        0x00 => Opcode::Nop,
        0xc3 => Opcode::Jp,
        _ => panic!("Opcode not implemented: {:?}", x),
    }
}
