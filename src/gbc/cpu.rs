use super::memory::Memory;
use super::opcode::Opcode;
use super::opcode::Opcode::*;

#[derive(Debug)]
pub struct Cpu {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn reset(&mut self) {
        // TODO: find out if the reset state matters (except for sp and pc)
        self.sp = 0xfffe;
        self.pc = 0x0100;
    }

    pub fn execute_instruction(&mut self, memory: &mut Memory) {

        let opcode = self.fetch_opcode(&memory);

        match opcode {
            Nop => println!("nop"),
            Jp => println!("jp"),
        }

    }

    fn fetch_opcode(&mut self, memory: &Memory) -> Opcode {
        let opcode = super::opcode::to_opcode(memory.read(self.pc));
        self.pc = self.pc + 1;
        opcode
    }
}
