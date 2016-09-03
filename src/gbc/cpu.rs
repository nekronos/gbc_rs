use super::interconnect::Interconnect;
use super::Model;
use std::u8;

#[derive(Debug)]
pub struct Cpu<'a> {
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

    interconnect: &'a Interconnect<'a>,
}

impl<'a> Cpu<'a> {
    pub fn new(interconnect: &'a Interconnect) -> Cpu<'a> {
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
            interconnect: interconnect,
        }
    }

    pub fn reset(&mut self, model: Model) {
        // TODO: find out if the reset state matters (except for sp and pc)
        // 0x11 for CGB, 0x01 for GB

        self.a = match model {
            Model::Gb => 0x01,
            Model::Cgb => 0x11,
        };

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

    pub fn execute_instruction(&mut self) {

        let opcode = self.fetch_u8();

        match opcode {
            // NOP
            0x00 => {}

            // JP NZ,r8
            0x20 => {
                let offset = self.fetch_u8() as u16;
                if !self.zero {
                    if (offset & 0x80) != 0 {
                        self.pc = self.pc - offset
                    } else {
                        self.pc = self.pc + offset
                    }
                }
            }

            // JP a16
            0xc3 => self.pc = self.fetch_u16(),

            // LDH A,(a8)
            0xf0 => {
                let offset = self.fetch_u8() as u16;
                let address = 0xff00 + offset;
                self.a = self.load(address)
            }

            // CP d8
            0xfe => {
                let operand = self.fetch_u8();
                self.compare(operand)
            }

            _ => panic!("Opcode not implemented: 0x{0:x}", opcode),
        }

        println!("0x{0:x}", self.pc);
    }

    fn load(&self, address: u16) -> u8 {
        self.interconnect.read(address)
    }

    fn fetch_u8(&mut self) -> u8 {
        let operand = self.interconnect.read(self.pc);
        self.pc = self.pc + 1;
        operand
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch_u8() as u16;
        let high = self.fetch_u8() as u16;
        (high << 8) | low
    }

    fn compare(&mut self, value: u8) {
        self.subtract = true;
        self.carry = self.a < value;
        self.zero = self.a == value;
        self.half_carry = (self.a.wrapping_sub(value) & 0xf) > (self.a & 0xf);
    }

    fn set_flags(&mut self, flags: u8) {
        self.zero = (flags & 0x80) != 0;
        self.subtract = (flags & 0x40) != 0;
        self.half_carry = (flags & 0x20) != 0;
        self.carry = (flags & 0x10) != 0;
    }
}
