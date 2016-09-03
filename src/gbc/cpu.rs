use super::memory::Memory;
use super::opcode;
use std::u8;

#[derive(Debug)]
pub struct Cpu {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    sp: u16,
    pc: u16,

    zero: bool,
    subtract: bool,
    half_carry: bool,
    carry: bool,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            zero: false,
            subtract: false,
            half_carry: false,
            carry: false,
        }
    }

    pub fn reset(&mut self) {
        // TODO: find out if the reset state matters (except for sp and pc)
        // 0x11 for CGB, 0x01 for GB
        self.a = 0x11;
        self.set_flags(0xb0);
        self.b = 0x00;
        self.c = 0x13;
        self.d = 0x00;
        self.e = 0xd8;
        self.h = 0x01;
        self.l = 0x4d;
        self.sp = 0xfffe;
        self.pc = 0x0100;

    }

    pub fn execute_instruction(&mut self, memory: &mut Memory) {

        let opcode = self.fetch_operand_u8(&memory);

        match opcode {
            0x00 => { /* NOP */ },

            0xc3 => self.jp(&memory),

            0xfe => {
                let operand = self.fetch_operand_u8(&memory);
                self.compare(operand)
            },

            0x20 => {
                let operand = self.fetch_operand_u8(&memory);
                self.jr_nz(operand)
            },

            0xf0 => self.ldh_a(&memory),

            _ => panic!("Opcode not implemented: {0:x}", opcode),
        }

        println!("0x{0:x}", self.pc);
    }

    fn fetch_operand_u8(&mut self, memory: &Memory) -> u8 {
        let operand = memory.read(self.pc);
        self.pc = self.pc + 1;
        operand
    }

    fn fetch_operand_u16(&mut self, memory: &Memory) -> u16 {
        let low = self.fetch_operand_u8(&memory) as u16;
        let high = self.fetch_operand_u8(&memory) as u16;
        (high << 8) | low
    }

    fn jp(&mut self, memory: &Memory) {
        self.pc = self.fetch_operand_u16(&memory)
    }

    fn compare(&mut self, value: u8) {
        self.subtract = true;
        self.carry = self.a < value;
        self.zero = self.a == value;
        self.half_carry = (self.a.wrapping_sub(value) & 0xf) > (self.a & 0xf);
    }

    fn jr_nz(&mut self, offset: u8) {
        let offset = offset as u16;
        if !self.zero {
            if (offset & 0x80) != 0 {
                self.pc = self.pc - offset
            } else {
                self.pc = self.pc + offset
            }
        }
    }

    fn ldh_a(&mut self, memory: & Memory) {
        let offset = self.fetch_operand_u8(&memory) as u16;
        let address = 0xff00 + offset;
        self.a = memory.read(address);
    }

    fn set_flags(&mut self, flags: u8) {
        self.zero = (flags & 0x80) != 0;
        self.subtract = (flags & 0x40) != 0;
        self.half_carry = (flags & 0x20) != 0;
        self.carry = (flags & 0x10) != 0;
    }

}
