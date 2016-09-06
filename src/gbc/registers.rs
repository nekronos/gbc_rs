
#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Reg16 {
    AF,
    BC,
    DE,
    HL,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Flag {
    Z,
    N,
    H,
    C,
}

impl Flag {
    pub fn mask(self) -> u8 {
        use self::Flag::*;
        match self {
            Z => 0b1000_0000,
            N => 0b0100_0000,
            H => 0b0010_0000,
            C => 0b0001_0000,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

#[allow(dead_code)]
impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x11, // 0x01 for GB, 0x11 for CGB
            f: 0xb0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xd8,
            h: 0x01,
            l: 0x4d,
            sp: 0xfffe,
            pc: 0x0100,
        }
    }

    pub fn read_u8(&self, reg: Reg8) -> u8 {
        use self::Reg8::*;
        match reg {
            A => self.a,
            F => self.f,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }

    pub fn read_u16(&self, reg: Reg16) -> u16 {
        use self::Reg16::*;
        match reg {
            AF => ((self.a as u16) << 8) | self.f as u16,
            BC => ((self.b as u16) << 8) | self.c as u16,
            DE => ((self.d as u16) << 8) | self.e as u16,
            HL => ((self.h as u16) << 8) | self.l as u16,
        }
    }

    pub fn write_u8(&mut self, reg: Reg8, value: u8) {
        use self::Reg8::*;
        match reg {
            A => self.a = value,
            F => self.f = value,
            B => self.b = value,
            C => self.c = value,
            D => self.d = value,
            E => self.e = value,
            H => self.h = value,
            L => self.l = value,
        }
    }

    pub fn write_u16(&mut self, reg: Reg16, value: u16) {
        use self::Reg16::*;
        let high = (value >> 8) as u8;
        let low = value as u8;
        match reg {
            AF => {
                self.a = high;
                self.f = low
            }

            BC => {
                self.b = high;
                self.c = low
            }

            DE => {
                self.d = high;
                self.e = low
            }

            HL => {
                self.h = high;
                self.l = low
            }
        }
    }

    pub fn set_flag(&mut self, flag: Flag) {
        self.f = self.f | flag.mask()
    }

    pub fn clear_flag(&mut self, flag: Flag) {
        self.f = self.f & !flag.mask()
    }

    pub fn is_flag_set(&self, flag: Flag) -> bool {
        (self.f & flag.mask()) != 0
    }

    pub fn set_flag_value(&mut self, flag: Flag, value: bool) {
        if value {
            self.set_flag(flag)
        } else {
            self.clear_flag(flag)
        }
    }

}
