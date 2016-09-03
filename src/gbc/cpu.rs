use super::memory::Memory;

use std::u8;

#[derive(Debug)]
pub struct Cpu {
    a: u8,
    // f: u8,
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
            // f: 0,
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
        self.sp = 0xfffe;
        self.pc = 0x0100;
    }

    pub fn execute_instruction(&mut self, memory: &mut Memory) {

        let opcode = self.fetch_operand_u8(&memory);

        match opcode {
            0x00 => println!("nop"),

            0xc3 => {
                self.pc = self.fetch_operand_u16(&memory);
            }

            0xfe => {
                let operand = self.fetch_operand_u8(&memory);

                self.subtract = true;

                if self.a < operand {
                    self.carry = true
                }

                if self.a == operand {
                    self.zero = true
                }

                let x = self.a.wrapping_sub(operand);
                if (x & 0x0f) > (self.a & 0x0f) {
                    self.half_carry = true
                }

            }
            _ => panic!("Opcode not implemented: 0x{0:x}", opcode),
        }

        println!("pc: 0x{0:x}", self.pc);

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
}
