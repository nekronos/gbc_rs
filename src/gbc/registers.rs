use super::GameboyType;
use std::fmt;
use std::fmt::Debug;

#[derive(Copy, Clone)]
pub enum Reg8 {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl Registers {
    pub fn new(gb_type: GameboyType) -> Registers {
        Registers {
            a: match gb_type {
                GameboyType::Cgb => 0x11,
                GameboyType::Dmg => 0x01,
            },
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x0100,

            // 0xb0
            zero: true,
            subtract: false,
            half_carry: true,
            carry: true,
        }
    }

    #[inline(always)]
    pub fn read_u8(&self, reg: Reg8) -> u8 {
        use self::Reg8::*;
        match reg {
            A => self.a,
            F => self.get_flags(),
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }

    #[inline(always)]
    pub fn read_u16(&self, reg: Reg16) -> u16 {
        use self::Reg8::*;
        use self::Reg16::*;
        match reg {
            AF => ((self.read_u8(A) as u16) << 8) | self.read_u8(F) as u16,
            BC => ((self.read_u8(B) as u16) << 8) | self.read_u8(C) as u16,
            DE => ((self.read_u8(D) as u16) << 8) | self.read_u8(E) as u16,
            HL => ((self.read_u8(H) as u16) << 8) | self.read_u8(L) as u16,
            SP => self.sp,
        }
    }

    #[inline(always)]
    pub fn write_u8(&mut self, reg: Reg8, value: u8) {
        use self::Reg8::*;
        match reg {
            A => self.a = value,
            F => self.set_flags(value),
            B => self.b = value,
            C => self.c = value,
            D => self.d = value,
            E => self.e = value,
            H => self.h = value,
            L => self.l = value,
        }
    }

    #[inline(always)]
    pub fn write_u16(&mut self, reg: Reg16, value: u16) {
        use self::Reg8::*;
        use self::Reg16::*;
        let high = (value >> 8) as u8;
        let low = value as u8;
        match reg {
            AF => {
                self.write_u8(A, high);
                self.write_u8(F, low)
            }

            BC => {
                self.write_u8(B, high);
                self.write_u8(C, low)
            }

            DE => {
                self.write_u8(D, high);
                self.write_u8(E, low)
            }

            HL => {
                self.write_u8(H, high);
                self.write_u8(L, low)
            }

            SP => self.sp = value,
        }
    }

    #[inline(always)]
    fn get_flags(&self) -> u8 {
        let mut flags = 0;
        if self.zero {
            flags |= 0b1000_0000
        }
        if self.subtract {
            flags |= 0b0100_0000
        }
        if self.half_carry {
            flags |= 0b0010_0000
        }
        if self.carry {
            flags |= 0b0001_0000
        }
        flags
    }

    #[inline(always)]
    fn set_flags(&mut self, flags: u8) {
        self.zero = (flags & 0b1000_0000) != 0;
        self.subtract = (flags & 0b0100_0000) != 0;
        self.half_carry = (flags & 0b0010_0000) != 0;
        self.carry = (flags & 0b0001_0000) != 0;
    }
}

impl Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Registers {{
    af: {:04X}
    bc: {:04X}
    de: {:04X}
    hl: {:04X}
    sp: {:04X}
    pc: {:04X}
    zero: {:#?}
    subtract: {:#?}
    half_carry: {:#?}
    carry: {:#?}
}}",
               self.read_u16(Reg16::AF),
               self.read_u16(Reg16::BC),
               self.read_u16(Reg16::DE),
               self.read_u16(Reg16::HL),
               self.sp,
               self.pc,
               self.zero,
               self.subtract,
               self.half_carry,
               self.carry)
    }
}
