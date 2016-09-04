
enum Reg8
{
	A,
	F,
	B,
	C,
	D,
	E,
	H,
	L,
}

enum Reg16 {
	AF,
	BC,
	DE,
	HL,
}

#[derive(Debug)]
struct Registers {
	a: u8,
	f: u8,
	b: u8,
	c: u8,
	d: u8,
	e: u8,
	h: u8,
	l: u8,
}

impl Registers {

	fn read_u8(&self, reg: Reg8) -> u8 {
		use self::Reg8*
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

	fn read_u16(&self, reg: Reg16) -> u16 {
		use self::Reg16*;
		match reg {
			AF => to_u16(self.A, self.F),
			BC => to_u16(self.B, self.C),
			DE => to_u16(self.D, self.E),
			HL => to_u16(self.H, self.L),
		}
	}
	
	fn to_u16(u8 high, u8 low) -> u16 {
		((high as u16) << 8) | (low as u16)
	}

}