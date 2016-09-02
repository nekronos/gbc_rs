

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

#[derive(Debug)]
pub enum StatusFlag {
    Zero,
    Subtract,
    HalfCarry,
    Carry,
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

    pub fn is_set(&self, flag: StatusFlag) -> bool {
        match flag {
            StatusFlag::Zero => (self.f & 0x80) != 0,
            StatusFlag::Subtract => (self.f & 0x40) != 0,
            StatusFlag::HalfCarry => (self.f & 0x20) != 0,
            StatusFlag::Carry => (self.f & 0x10) != 0,
        }
    }
}
