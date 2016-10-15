use super::interconnect::Interconnect;
use super::registers::{Registers, Reg8, Reg16};
use super::opcode::{CB_OPCODE_TIMES, OPCODE_TIMES, OPCODE_COND_TIMES};
use super::GameboyType;

use std::u8;
use std::u16;

#[allow(dead_code)]
const CLOCK_SPEED: u32 = 4_194_304;

pub struct Cpu {
    reg: Registers,
    interconnect: Interconnect,
    ime: bool,
    int_flags: u8,
    int_enable: u8,
    int_pending: bool,
}

struct ZMem;
struct Imm8;
struct Imm16;

#[derive(Copy,Clone)]
struct Mem<T: Src<u16>>(T);

#[allow(dead_code)]
enum Cond {
    Uncond,
    Zero,
    Carry,
    NotZero,
    NotCarry,
}

impl Cond {
    fn is_true(self, cpu: &Cpu) -> bool {
        use self::Cond::*;
        match self {
            Uncond => true,
            Zero => cpu.reg.zero,
            Carry => cpu.reg.carry,
            NotZero => !cpu.reg.zero,
            NotCarry => !cpu.reg.carry,
        }
    }
}

#[derive(Debug)]
enum Timing {
    Default,
    Cond,
    Cb(u32),
}

trait Src<T> {
    fn read(self, cpu: &mut Cpu) -> T;
}

trait Dst<T> {
    fn write(self, cpu: &mut Cpu, val: T);
}

impl Dst<u8> for Reg8 {
    fn write(self, cpu: &mut Cpu, val: u8) {
        cpu.reg.write_u8(self, val)
    }
}

impl Dst<u16> for Reg16 {
    fn write(self, cpu: &mut Cpu, val: u16) {
        cpu.reg.write_u16(self, val)
    }
}

impl Src<u8> for Reg8 {
    fn read(self, cpu: &mut Cpu) -> u8 {
        cpu.reg.read_u8(self)
    }
}

impl Src<u8> for Imm8 {
    fn read(self, cpu: &mut Cpu) -> u8 {
        cpu.fetch_u8()
    }
}

impl Src<u16> for Reg16 {
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.reg.read_u16(self)
    }
}

impl Src<u16> for Imm16 {
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.fetch_u16()
    }
}

impl Src<u8> for ZMem {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let offset = cpu.fetch_u8() as u16;
        let addr = 0xff00 + offset;
        cpu.read(addr)
    }
}

impl Dst<u8> for ZMem {
    fn write(self, cpu: &mut Cpu, val: u8) {
        let offset = cpu.fetch_u8() as u16;
        let addr = 0xff00 + offset;
        cpu.write(addr, val)
    }
}

impl Dst<u8> for Mem<Reg16> {
    fn write(self, cpu: &mut Cpu, val: u8) {
        let Mem(reg) = self;
        let addr = reg.read(cpu);
        cpu.write(addr, val)
    }
}

impl Dst<u8> for Mem<Imm16> {
    fn write(self, cpu: &mut Cpu, val: u8) {
        let Mem(imm) = self;
        let addr = imm.read(cpu);
        cpu.write(addr, val)
    }
}

impl Src<u8> for Mem<Imm16> {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let Mem(imm) = self;
        let addr = imm.read(cpu);
        cpu.read(addr)
    }
}

impl Dst<u16> for Mem<Imm16> {
    fn write(self, cpu: &mut Cpu, val: u16) {
        let Mem(imm) = self;
        let addr = imm.read(cpu);
        let l = val as u8;
        let h = (val >> 8) as u8;
        cpu.write(addr, l);
        cpu.write(addr + 1, h)
    }
}

impl Src<u8> for Mem<Reg16> {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let Mem(reg) = self;
        let addr = reg.read(cpu);
        cpu.read(addr)
    }
}


impl Cpu {
    pub fn new(gb_type: GameboyType, interconnect: Interconnect) -> Cpu {
        Cpu {
            reg: Registers::new(gb_type),
            interconnect: interconnect,
            ime: true,
            int_flags: 0,
            int_enable: 0,
            int_pending: false,
        }
    }

    pub fn step(&mut self) -> u32 {
        let elapsed_cycles = {
            self.handle_interrupt() + self.execute_instruction()
        };
        self.interconnect.cycle_flush(elapsed_cycles);
        elapsed_cycles
    }

    fn handle_interrupt(&mut self) -> u32 {
        if !self.int_pending && !self.ime {
            return 0;
        }

        let ints = self.int_flags & self.int_enable;
        if ints == 0 {
            return 0;
        }

        self.int_enable = 0;
        self.ime = false;

        let int = ints.trailing_zeros();
        let int_handler = {
            match int {
                0 => 0x0040,// VBLANK
                1 => 0x0048,// LCDC STATUS
                2 => 0x0050,// TIMER OVERFLOW
                3 => 0x0058,// SERIAL TRANSFER COMPLETE
                4 => 0x0060,// P10-P13 INPUT SIGNAL
                _ => panic!("Invalid interrupt {:x}", int),
            }
        };

        self.int_flags = 0x01 << int;

        let pc = self.reg.pc;
        self.push_u16(pc);

        self.reg.pc = int_handler;

        4 // It takes 4 cycles to handle the interrupt
    }

    fn execute_instruction(&mut self) -> u32 {

        let pc = self.reg.pc;
        // println!("{}",
        // super::disassembler::disassemble(pc, self.interconnect));

        let opcode = self.fetch_u8();

        use super::registers::Reg8::*;
        use super::registers::Reg16::*;
        use self::Cond::*;

        let timing = {
            match opcode {
                0x00 => Timing::Default,
                0x01 => self.ld(BC, Imm16),
                0x02 => self.ld(Mem(BC), A),
                0x03 => self.inc_16(BC),
                0x04 => self.inc_8(B),
                0x05 => self.dec_8(B),
                0x06 => self.ld(B, Imm8),
                0x07 => self.rlca(),
                0x08 => self.ld(Mem(Imm16), SP),
                0x09 => self.add_16(HL, BC),
                0x0a => self.ld(A, Mem(BC)),
                0x0b => self.dec_16(BC),
                0x0c => self.inc_8(C),
                0x0d => self.dec_8(C),
                0x0e => self.ld(C, Imm8),
                0x0f => self.rrca(),
                0x10 => self.stop(),
                0x11 => self.ld(DE, Imm16),
                0x12 => self.ld(Mem(DE), A),
                0x13 => self.inc_16(DE),
                0x14 => self.inc_8(D),
                0x15 => self.dec_8(D),
                0x16 => self.ld(D, Imm8),
                0x17 => self.rla(),
                0x18 => self.jr(Uncond, Imm8),
                0x19 => self.add_16(HL, DE),
                0x1a => self.ld(A, Mem(DE)),
                0x1b => self.dec_16(DE),
                0x1c => self.inc_8(E),
                0x1d => self.dec_8(E),
                0x1e => self.ld(E, Imm8),
                0x1f => self.rra(),
                0x20 => self.jr(NotZero, Imm8),
                0x21 => self.ld(HL, Imm16),
                0x22 => self.ldi(Mem(HL), A, HL),
                0x23 => self.inc_16(HL),
                0x24 => self.inc_8(H),
                0x25 => self.dec_8(H),
                0x26 => self.ld(H, Imm8),
                0x27 => self.daa(),
                0x28 => self.jr(Zero, Imm8),
                0x29 => self.add_16(HL, HL),
                0x2a => self.ldi(A, Mem(HL), HL),
                0x2b => self.dec_16(HL),
                0x2c => self.inc_8(L),
                0x2d => self.dec_8(L),
                0x2e => self.ld(L, Imm8),
                0x2f => self.cpl(),
                0x30 => self.jr(NotCarry, Imm8),
                0x31 => self.ld(SP, Imm16),
                0x32 => self.ldd(Mem(HL), A, HL),
                0x33 => self.inc_16(SP),
                0x34 => self.inc_8(Mem(HL)),
                0x35 => self.dec_8(Mem(HL)),
                0x36 => self.ld(Mem(HL), Imm8),
                0x37 => self.scf(),
                0x38 => self.jr(Carry, Imm8),
                0x39 => self.add_16(HL, SP),
                0x3a => self.ldd(A, Mem(HL), HL),
                0x3b => self.dec_16(SP),
                0x3c => self.inc_8(A),
                0x3d => self.dec_8(A),
                0x3e => self.ld(A, Imm8),
                0x3f => self.ccf(),
                0x40 => self.ld(B, B),
                0x41 => self.ld(B, C),
                0x42 => self.ld(B, D),
                0x43 => self.ld(B, E),
                0x44 => self.ld(B, H),
                0x45 => self.ld(B, L),
                0x46 => self.ld(B, Mem(HL)),
                0x47 => self.ld(B, A),
                0x48 => self.ld(C, B),
                0x49 => self.ld(C, C),
                0x4a => self.ld(C, D),
                0x4b => self.ld(C, E),
                0x4c => self.ld(C, H),
                0x4d => self.ld(C, L),
                0x4e => self.ld(C, Mem(HL)),
                0x4f => self.ld(C, A),
                0x50 => self.ld(D, B),
                0x51 => self.ld(D, C),
                0x52 => self.ld(D, D),
                0x53 => self.ld(D, E),
                0x54 => self.ld(D, H),
                0x55 => self.ld(D, L),
                0x56 => self.ld(D, Mem(HL)),
                0x57 => self.ld(D, A),
                0x58 => self.ld(E, B),
                0x59 => self.ld(E, C),
                0x5a => self.ld(E, D),
                0x5b => self.ld(E, E),
                0x5c => self.ld(E, H),
                0x5d => self.ld(E, L),
                0x5e => self.ld(E, Mem(HL)),
                0x5f => self.ld(E, A),
                0x60 => self.ld(H, B),
                0x61 => self.ld(H, C),
                0x62 => self.ld(H, D),
                0x63 => self.ld(H, E),
                0x64 => self.ld(H, H),
                0x65 => self.ld(H, L),
                0x66 => self.ld(H, Mem(HL)),
                0x67 => self.ld(H, A),
                0x68 => self.ld(L, B),
                0x69 => self.ld(L, C),
                0x6a => self.ld(L, D),
                0x6b => self.ld(L, E),
                0x6c => self.ld(L, H),
                0x6d => self.ld(L, L),
                0x6e => self.ld(L, Mem(HL)),
                0x6f => self.ld(L, A),
                0x70 => self.ld(Mem(HL), B),
                0x71 => self.ld(Mem(HL), C),
                0x72 => self.ld(Mem(HL), D),
                0x73 => self.ld(Mem(HL), E),
                0x74 => self.ld(Mem(HL), H),
                0x75 => self.ld(Mem(HL), L),
                0x77 => self.ld(Mem(HL), A),
                0x78 => self.ld(A, B),
                0x79 => self.ld(A, C),
                0x7a => self.ld(A, D),
                0x7b => self.ld(A, E),
                0x7c => self.ld(A, H),
                0x7d => self.ld(A, L),
                0x7e => self.ld(A, Mem(HL)),
                0x7f => self.ld(A, A),
                0x80 => self.add_8(A, B),
                0x81 => self.add_8(A, C),
                0x82 => self.add_8(A, D),
                0x83 => self.add_8(A, E),
                0x84 => self.add_8(A, H),
                0x85 => self.add_8(A, L),
                0x86 => self.add_8(A, Mem(HL)),
                0x87 => self.add_8(A, A),
                0x88 => self.adc(A, B),
                0x89 => self.adc(A, C),
                0x8a => self.adc(A, D),
                0x8b => self.adc(A, E),
                0x8c => self.adc(A, H),
                0x8d => self.adc(A, L),
                0x8e => self.adc(A, Mem(HL)),
                0x8f => self.adc(A, A),
                0x90 => self.sub_8(A, B),
                0x91 => self.sub_8(A, C),
                0x92 => self.sub_8(A, D),
                0x93 => self.sub_8(A, E),
                0x94 => self.sub_8(A, H),
                0x95 => self.sub_8(A, L),
                0x96 => self.sub_8(A, Mem(HL)),
                0x97 => self.sub_8(A, A),
                0x98 => self.sbc(A, B),
                0x99 => self.sbc(A, C),
                0x9a => self.sbc(A, D),
                0x9b => self.sbc(A, E),
                0x9c => self.sbc(A, H),
                0x9d => self.sbc(A, L),
                0x9e => self.sbc(A, Mem(HL)),
                0x9f => self.sbc(A, A),
                0xa0 => self.and(B),
                0xa1 => self.and(C),
                0xa2 => self.and(D),
                0xa3 => self.and(E),
                0xa4 => self.and(H),
                0xa5 => self.and(L),
                0xa6 => self.and(Mem(HL)),
                0xa7 => self.and(A),
                0xa8 => self.xor(B),
                0xa9 => self.xor(C),
                0xaa => self.xor(D),
                0xab => self.xor(E),
                0xac => self.xor(H),
                0xad => self.xor(L),
                0xae => self.xor(Mem(HL)),
                0xaf => self.xor(A),
                0xb0 => self.or(B),
                0xb1 => self.or(C),
                0xb2 => self.or(D),
                0xb3 => self.or(E),
                0xb4 => self.or(H),
                0xb5 => self.or(L),
                0xb6 => self.or(Mem(HL)),
                0xb7 => self.or(A),
                0xb8 => self.cp(B),
                0xb9 => self.cp(C),
                0xba => self.cp(D),
                0xbb => self.cp(E),
                0xbc => self.cp(H),
                0xbd => self.cp(L),
                0xbe => self.cp(Mem(HL)),
                0xbf => self.cp(A),
                0xc0 => self.ret(NotZero),
                0xc1 => self.pop(BC),
                0xc2 => self.jp(NotZero, Imm16),
                0xc3 => self.jp(Uncond, Imm16),
                0xc4 => self.call(NotZero, Imm16),
                0xc5 => self.push(BC),
                0xc6 => self.add_8(A, Imm8),
                0xc8 => self.ret(Zero),
                0xc9 => self.ret(Uncond),
                0xca => self.jp(Zero, Imm16),
                0xcb => self.execute_cb_instruction(),
                0xcc => self.call(Zero, Imm16),
                0xcd => self.call(Uncond, Imm16),
                0xce => self.adc(A, Imm8),
                0xd0 => self.ret(NotCarry),
                0xd1 => self.pop(DE),
                0xd2 => self.jp(NotCarry, Imm16),
                0xd4 => self.call(NotCarry, Imm16),
                0xd5 => self.push(DE),
                0xd6 => self.sub_8(A, Imm8),
                0xd8 => self.ret(Carry),
                0xda => self.jp(Carry, Imm16),
                0xdc => self.call(Carry, Imm16),
                0xde => self.sbc(A, Imm8),
                0xe0 => self.ld(ZMem, A),
                0xe1 => self.pop(HL),
                0xe5 => self.push(HL),
                0xe6 => self.and(Imm8),
                0xe8 => self.add_sp(Imm8),
                0xe9 => self.jp(Uncond, HL),
                0xea => self.ld(Mem(Imm16), A),
                0xee => self.xor(Imm8),
                0xf0 => self.ld(A, ZMem),
                0xf1 => self.pop(AF),
                0xf3 => self.di(),
                0xf5 => self.push(AF),
                0xf6 => self.or(Imm8),
                0xf8 => self.ld_hl_sp(),
                0xf9 => self.ld(SP, HL),
                0xfa => self.ld(A, Mem(Imm16)),
                0xfb => self.ei(),
                0xfe => self.cp(Imm8),

                _ => {
                    println!("\n");
                    println!("{}",
                             super::disassembler::disassemble(pc, &self.interconnect));
                    println!("{:#?}", self.reg);
                    panic!("Opcode not implemented: 0x{:x}", opcode);
                }
            }
        };

        match timing {
            Timing::Default => OPCODE_TIMES[opcode as usize] as u32,
            Timing::Cond => OPCODE_COND_TIMES[opcode as usize] as u32,
            Timing::Cb(x) => x,
        }
    }

    fn execute_cb_instruction(&mut self) -> Timing {

        let opcode = self.fetch_u8();

        use super::registers::Reg8::*;
        use super::registers::Reg16::*;

        match opcode {

            0x00 => self.rlc(B),
            0x01 => self.rlc(C),
            0x02 => self.rlc(D),
            0x03 => self.rlc(E),
            0x04 => self.rlc(H),
            0x05 => self.rlc(L),
            0x06 => self.rlc(Mem(HL)),
            0x07 => self.rlc(A),
            0x08 => self.rrc(B),
            0x09 => self.rrc(C),
            0x0a => self.rrc(D),
            0x0b => self.rrc(E),
            0x0c => self.rrc(H),
            0x0d => self.rrc(L),
            0x0e => self.rrc(Mem(HL)),
            0x0f => self.rrc(A),
            0x10 => self.rl(B),
            0x11 => self.rl(C),
            0x12 => self.rl(D),
            0x13 => self.rl(E),
            0x14 => self.rl(H),
            0x15 => self.rl(L),
            0x16 => self.rl(Mem(HL)),
            0x17 => self.rl(A),
            0x18 => self.rr(B),
            0x19 => self.rr(C),
            0x1a => self.rr(D),
            0x1b => self.rr(E),
            0x1c => self.rr(H),
            0x1d => self.rr(L),
            0x1e => self.rr(Mem(HL)),
            0x1f => self.rr(A),
            0x20 => self.sla(B),
            0x21 => self.sla(C),
            0x22 => self.sla(D),
            0x23 => self.sla(E),
            0x24 => self.sla(H),
            0x25 => self.sla(L),
            0x26 => self.sla(Mem(HL)),
            0x27 => self.sla(A),
            0x28 => self.sra(B),
            0x29 => self.sra(C),
            0x2a => self.sra(D),
            0x2b => self.sra(E),
            0x2c => self.sra(H),
            0x2d => self.sra(L),
            0x2e => self.sra(Mem(HL)),
            0x2f => self.sra(A),
            0x30 => self.swap_8(B),
            0x31 => self.swap_8(C),
            0x32 => self.swap_8(D),
            0x33 => self.swap_8(E),
            0x34 => self.swap_8(H),
            0x35 => self.swap_8(L),
            0x36 => self.swap_8(Mem(HL)),
            0x37 => self.swap_8(A),
            0x38 => self.srl(B),
            0x39 => self.srl(C),
            0x3a => self.srl(D),
            0x3b => self.srl(E),
            0x3c => self.srl(H),
            0x3d => self.srl(L),
            0x3e => self.srl(Mem(HL)),
            0x3f => self.srl(A),
            0x40 => self.bit(0, B),
            0x41 => self.bit(0, C),
            0x42 => self.bit(0, D),
            0x43 => self.bit(0, E),
            0x44 => self.bit(0, H),
            0x45 => self.bit(0, L),
            0x46 => self.bit(0, Mem(HL)),
            0x47 => self.bit(0, A),
            0x48 => self.bit(1, B),
            0x49 => self.bit(1, C),
            0x4a => self.bit(1, D),
            0x4b => self.bit(1, E),
            0x4c => self.bit(1, H),
            0x4d => self.bit(1, L),
            0x4e => self.bit(1, Mem(HL)),
            0x4f => self.bit(1, A),
            0x50 => self.bit(2, B),
            0x51 => self.bit(2, C),
            0x52 => self.bit(2, D),
            0x53 => self.bit(2, E),
            0x54 => self.bit(2, H),
            0x55 => self.bit(2, L),
            0x56 => self.bit(2, Mem(HL)),
            0x57 => self.bit(2, A),
            0x58 => self.bit(3, B),
            0x59 => self.bit(3, C),
            0x5a => self.bit(3, D),
            0x5b => self.bit(3, E),
            0x5c => self.bit(3, H),
            0x5d => self.bit(3, L),
            0x5e => self.bit(3, Mem(HL)),
            0x5f => self.bit(3, A),
            0x60 => self.bit(4, B),
            0x61 => self.bit(4, C),
            0x62 => self.bit(4, D),
            0x63 => self.bit(4, E),
            0x64 => self.bit(4, H),
            0x65 => self.bit(4, L),
            0x66 => self.bit(4, Mem(HL)),
            0x67 => self.bit(4, A),
            0x68 => self.bit(5, B),
            0x69 => self.bit(5, C),
            0x6a => self.bit(5, D),
            0x6b => self.bit(5, E),
            0x6c => self.bit(5, H),
            0x6d => self.bit(5, L),
            0x6e => self.bit(5, Mem(HL)),
            0x6f => self.bit(5, A),
            0x70 => self.bit(6, B),
            0x71 => self.bit(6, C),
            0x72 => self.bit(6, D),
            0x73 => self.bit(6, E),
            0x74 => self.bit(6, H),
            0x75 => self.bit(6, L),
            0x76 => self.bit(6, Mem(HL)),
            0x77 => self.bit(6, A),
            0x78 => self.bit(7, B),
            0x79 => self.bit(7, C),
            0x7a => self.bit(7, D),
            0x7b => self.bit(7, E),
            0x7c => self.bit(7, H),
            0x7d => self.bit(7, L),
            0x7e => self.bit(7, Mem(HL)),
            0x7f => self.bit(7, A),
            0x80 => self.res(0, B),
            0x81 => self.res(0, C),
            0x82 => self.res(0, D),
            0x83 => self.res(0, E),
            0x84 => self.res(0, H),
            0x85 => self.res(0, L),
            0x86 => self.res(0, Mem(HL)),
            0x87 => self.res(0, A),
            0x88 => self.res(1, B),
            0x89 => self.res(1, C),
            0x8a => self.res(1, D),
            0x8b => self.res(1, E),
            0x8c => self.res(1, H),
            0x8d => self.res(1, L),
            0x8e => self.res(1, Mem(HL)),
            0x8f => self.res(1, A),
            0x90 => self.res(2, B),
            0x91 => self.res(2, C),
            0x92 => self.res(2, D),
            0x93 => self.res(2, E),
            0x94 => self.res(2, H),
            0x95 => self.res(2, L),
            0x96 => self.res(2, Mem(HL)),
            0x97 => self.res(2, A),
            0x98 => self.res(3, B),
            0x99 => self.res(3, C),
            0x9a => self.res(3, D),
            0x9b => self.res(3, E),
            0x9c => self.res(3, H),
            0x9d => self.res(3, L),
            0x9e => self.res(3, Mem(HL)),
            0x9f => self.res(3, A),
            0xa0 => self.res(4, B),
            0xa1 => self.res(4, C),
            0xa2 => self.res(4, D),
            0xa3 => self.res(4, E),
            0xa4 => self.res(4, H),
            0xa5 => self.res(4, L),
            0xa6 => self.res(4, Mem(HL)),
            0xa7 => self.res(4, A),
            0xa8 => self.res(5, B),
            0xa9 => self.res(5, C),
            0xaa => self.res(5, D),
            0xab => self.res(5, E),
            0xac => self.res(5, H),
            0xad => self.res(5, L),
            0xae => self.res(5, Mem(HL)),
            0xaf => self.res(5, A),
            0xb0 => self.res(6, B),
            0xb1 => self.res(6, C),
            0xb2 => self.res(6, D),
            0xb3 => self.res(6, E),
            0xb4 => self.res(6, H),
            0xb5 => self.res(6, L),
            0xb6 => self.res(6, Mem(HL)),
            0xb7 => self.res(6, A),
            0xb8 => self.res(7, B),
            0xb9 => self.res(7, C),
            0xba => self.res(7, D),
            0xbb => self.res(7, E),
            0xbc => self.res(7, H),
            0xbd => self.res(7, L),
            0xbe => self.res(7, Mem(HL)),
            0xbf => self.res(7, A),
            0xc0 => self.set(0, B),
            0xc1 => self.set(0, C),
            0xc2 => self.set(0, D),
            0xc3 => self.set(0, E),
            0xc4 => self.set(0, H),
            0xc5 => self.set(0, L),
            0xc6 => self.set(0, Mem(HL)),
            0xc7 => self.set(0, A),
            0xc8 => self.set(1, B),
            0xc9 => self.set(1, C),
            0xca => self.set(1, D),
            0xcb => self.set(1, E),
            0xcc => self.set(1, H),
            0xcd => self.set(1, L),
            0xce => self.set(1, Mem(HL)),
            0xcf => self.set(1, A),
            0xd0 => self.set(2, B),
            0xd1 => self.set(2, C),
            0xd2 => self.set(2, D),
            0xd3 => self.set(2, E),
            0xd4 => self.set(2, H),
            0xd5 => self.set(2, L),
            0xd6 => self.set(2, Mem(HL)),
            0xd7 => self.set(2, A),
            0xd8 => self.set(3, B),
            0xd9 => self.set(3, C),
            0xda => self.set(3, D),
            0xdb => self.set(3, E),
            0xdc => self.set(3, H),
            0xdd => self.set(3, L),
            0xde => self.set(3, Mem(HL)),
            0xdf => self.set(3, A),
            0xe0 => self.set(4, B),
            0xe1 => self.set(4, C),
            0xe2 => self.set(4, D),
            0xe3 => self.set(4, E),
            0xe4 => self.set(4, H),
            0xe5 => self.set(4, L),
            0xe6 => self.set(4, Mem(HL)),
            0xe7 => self.set(4, A),
            0xe8 => self.set(5, B),
            0xe9 => self.set(5, C),
            0xea => self.set(5, D),
            0xeb => self.set(5, E),
            0xec => self.set(5, H),
            0xed => self.set(5, L),
            0xee => self.set(5, Mem(HL)),
            0xef => self.set(5, A),
            0xf0 => self.set(6, B),
            0xf1 => self.set(6, C),
            0xf2 => self.set(6, D),
            0xf3 => self.set(6, E),
            0xf4 => self.set(6, H),
            0xf5 => self.set(6, L),
            0xf6 => self.set(6, Mem(HL)),
            0xf7 => self.set(6, A),
            0xf8 => self.set(7, B),
            0xf9 => self.set(7, C),
            0xfa => self.set(7, D),
            0xfb => self.set(7, E),
            0xfc => self.set(7, H),
            0xfd => self.set(7, L),
            0xfe => self.set(7, Mem(HL)),
            0xff => self.set(7, A),

            _ => {
                let pc = self.reg.pc - 2;
                println!("\n");
                println!("{}",
                         super::disassembler::disassemble(pc, &self.interconnect));
                println!("{:#?}", self.reg);
                panic!("CB opcode not implemented: 0x{:x}", opcode)
            }
        };

        Timing::Cb(CB_OPCODE_TIMES[opcode as usize] as u32)
    }

    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0xff0f => self.int_flags,
            0xffff => self.int_enable,
            _ => self.interconnect.read(addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0xff0f => {
                self.int_flags = val;
                self.int_pending = true
            }
            0xffff => self.int_enable = val,
            _ => self.interconnect.write(addr, val),
        }
    }

    fn stop(&self) -> Timing {
        // http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
        //
        // Instruction STOP has according to manuals opcode 10 00 and
        // thus is 2 bytes long. Anyhow it seems there is no reason for
        // it so some assemblers code it simply as one byte instruction 10
        //
        Timing::Default
    }

    fn call<S: Src<u16>>(&mut self, cond: Cond, src: S) -> Timing {
        let new_pc = src.read(self);
        if cond.is_true(self) {
            let ret = self.reg.pc;
            self.push_u16(ret);
            self.reg.pc = new_pc;
            Timing::Cond
        } else {
            Timing::Default
        }
    }

    fn ret(&mut self, cond: Cond) -> Timing {
        if cond.is_true(self) {
            let new_pc = self.pop_u16();
            self.reg.pc = new_pc;
            Timing::Cond
        } else {
            Timing::Default
        }
    }

    fn ld<T, D: Dst<T>, S: Src<T>>(&mut self, dst: D, src: S) -> Timing {
        let value = src.read(self);
        dst.write(self, value);
        Timing::Default
    }

    fn ldi<T, D: Dst<T>, S: Src<T>>(&mut self, dst: D, src: S, inc: Reg16) -> Timing {
        let t = self.ld(dst, src);
        self.inc_16(inc);
        t
    }

    fn ldd<T, D: Dst<T>, S: Src<T>>(&mut self, dst: D, src: S, dec: Reg16) -> Timing {
        let t = self.ld(dst, src);
        self.dec_16(dec);
        t
    }

    fn jp<S: Src<u16>>(&mut self, cond: Cond, src: S) -> Timing {
        let new_pc = src.read(self);
        if cond.is_true(self) {
            self.reg.pc = new_pc;
            Timing::Cond
        } else {
            Timing::Default
        }
    }

    fn jr<S: Src<u8>>(&mut self, cond: Cond, src: S) -> Timing {
        let offset = (src.read(self) as i8) as i16;
        if cond.is_true(self) {
            let pc = self.reg.pc as i16;
            let new_pc = (pc + offset) as u16;
            self.reg.pc = new_pc;
            Timing::Cond
        } else {
            Timing::Default
        }
    }

    fn and<S: Src<u8>>(&mut self, src: S) -> Timing {
        let a = src.read(self);
        let r = a & self.reg.a;
        self.reg.a = r;
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = true;
        self.reg.carry = false;
        Timing::Default
    }

    fn sbc<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(&mut self, dst: D, src: S) -> Timing {
        let a = dst.read(self) as i16;
        let b = src.read(self) as i16;
        let c = if self.reg.carry { 1 } else { 0 };
        let r = a.wrapping_sub(b).wrapping_sub(c);
        dst.write(self, r as u8);
        self.reg.zero = (r as u8) == 0;
        self.reg.subtract = true;
        self.reg.carry = r < 0;
        self.reg.half_carry = ((a & 0x0f) - (b & 0x0f) - c) < 0;
        Timing::Default
    }

    fn adc<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(&mut self, dst: D, src: S) -> Timing {
        let a = dst.read(self) as u16;
        let b = src.read(self) as u16;
        let c = if self.reg.carry { 1 } else { 0 };
        let r = a + b + c;
        dst.write(self, r as u8);
        self.reg.zero = (r as u8) == 0;
        self.reg.subtract = false;
        self.reg.half_carry = ((a & 0x0f) + (b & 0x0f) + c) > 0x0f;
        self.reg.carry = r > 0x00ff;
        Timing::Default
    }

    fn add_sp<S: Src<u8>>(&mut self, src: S) -> Timing {
        let new_sp = self.offset_sp();
        self.reg.sp = new_sp;
        Timing::Default
    }

    fn ld_hl_sp(&mut self) -> Timing {
        let sp = self.offset_sp();
        self.reg.h = (sp >> 8) as u8;
        self.reg.l = sp as u8;
        Timing::Default
    }

    fn offset_sp(&mut self) -> u16 {
        let offset = (Imm8.read(self) as i8) as i32;
        let sp = (self.reg.sp as i16) as i32;
        let r = sp + offset;
        self.reg.zero = false;
        self.reg.subtract = false;
        self.reg.carry = ((sp ^ offset ^ (r & 0xffff)) & 0x100) == 0x100;
        self.reg.half_carry = ((sp ^ offset ^ (r & 0xffff)) & 0x10) == 0x10;
        r as u16
    }

    fn add_8<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(&mut self, dst: D, src: S) -> Timing {
        let a = dst.read(self) as u16;
        let b = src.read(self) as u16;
        let r = a + b;
        let c = a ^ b ^ r;
        dst.write(self, r as u8);
        self.reg.zero = (r as u8) == 0;
        self.reg.subtract = false;
        self.reg.half_carry = (c & 0x0010) != 0;
        self.reg.carry = (c & 0x0100) != 0;
        Timing::Default
    }

    fn add_16<D: Dst<u16> + Src<u16> + Copy, S: Src<u16>>(&mut self, dst: D, src: S) -> Timing {
        let a = dst.read(self) as u32;
        let b = src.read(self) as u32;
        let r = a + b;
        dst.write(self, r as u16);
        self.reg.subtract = false;
        self.reg.carry = (r & 0x10000) != 0;
        self.reg.half_carry = ((a ^ b ^ (r & 0xffff)) & 0x1000) != 0;
        Timing::Default
    }

    fn sub_8<D: Dst<u8> + Src<u8> + Copy, S: Src<u8>>(&mut self, dst: D, src: S) -> Timing {
        let a = dst.read(self) as u16;
        let b = src.read(self) as u16;
        let r = a.wrapping_sub(b);
        let c = a ^ b ^ r;
        dst.write(self, r as u8);
        self.reg.zero = (r as u8) == 0;
        self.reg.subtract = true;
        self.reg.half_carry = (c & 0x0010) != 0;
        self.reg.carry = (c & 0x0100) != 0;
        Timing::Default
    }

    fn rrca(&mut self) -> Timing {
        // RRCA is the same as RRC, only it does not affect the zero flag
        let z = self.reg.zero;
        self.rrc(Reg8::A);
        self.reg.zero = z;
        Timing::Default
    }

    fn rla(&mut self) -> Timing {
        // RLA is the same as RL, only it does not affect the zero flag
        let z = self.reg.zero;
        self.rl(Reg8::A);
        self.reg.zero = z;
        Timing::Default
    }

    fn rra(&mut self) -> Timing {
        // RRA is the same as RR, only it does not affect the zero flag
        let z = self.reg.zero;
        self.rr(Reg8::A);
        self.reg.zero = z;
        Timing::Default
    }

    fn rlca(&mut self) -> Timing {
        // RLCA is the same as RLC, only it does not affect the zero flag
        let z = self.reg.zero;
        self.rlc(Reg8::A);
        self.reg.zero = z;
        Timing::Default
    }

    fn rlc<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = a.rotate_left(1);
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = (a & 0x80) != 0
    }

    fn rl<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = a << 1;
        let r = if self.reg.carry { r | 0x01 } else { r };
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = (a & 0x80) != 0
    }

    fn rr<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = a >> 1;
        let r = if self.reg.carry { r | 0x80 } else { r };
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = (a & 0x01) != 0;
    }

    fn rrc<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = a.rotate_right(1);
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = (a & 0x01) != 0
    }

    fn sla<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = a << 1;
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = (a & 0x80) != 0
    }

    fn sra<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = a >> 1;
        let r = (a & 0x80) | r;
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = (a & 0x01) != 0
    }

    fn daa(&mut self) -> Timing {
        let mut a = self.reg.a as u16;
        let n = self.reg.subtract;
        let c = self.reg.carry;
        let h = self.reg.half_carry;

        if n {
            if c {
                a = a.wrapping_sub(0x60)
            }
            if h {
                a = a.wrapping_sub(0x06)
            }
        } else {
            if c || ((a & 0xff) > 0x99) {
                a = a + 0x60;
                self.reg.carry = true
            }
            if h || ((a & 0x0f) > 0x09) {
                a = a + 0x06
            }
        }
        self.reg.zero = (a as u8) == 0;
        self.reg.half_carry = false;
        self.reg.a = a as u8;
        Timing::Default
    }

    fn scf(&mut self) -> Timing {
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = true;
        Timing::Default
    }

    fn ccf(&mut self) -> Timing {
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = !self.reg.carry;
        Timing::Default
    }

    fn bit<S: Src<u8>>(&mut self, bit: u8, src: S) {
        let a = src.read(self) >> bit;
        self.reg.zero = (a & 0x01) == 0;
        self.reg.subtract = false;
        self.reg.half_carry = true;
    }

    fn srl<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = a >> 1;
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = (a & 0x01) != 0;
    }

    fn res<L: Src<u8> + Dst<u8> + Copy>(&mut self, bit: u8, loc: L) {
        let a = loc.read(self);
        let r = a & !(0x01 << bit);
        loc.write(self, r)
    }

    fn set<L: Src<u8> + Dst<u8> + Copy>(&mut self, bit: u8, loc: L) {
        let a = loc.read(self);
        let r = a | (0x01 << bit);
        loc.write(self, r)
    }

    fn swap_8<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) {
        let a = loc.read(self);
        let r = (a << 4) | (a >> 4);
        loc.write(self, r);
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = false
    }

    fn xor<S: Src<u8>>(&mut self, src: S) -> Timing {
        let a = src.read(self);
        let r = self.reg.a ^ a;
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = false;
        self.reg.a = r;
        Timing::Default
    }

    fn or<S: Src<u8>>(&mut self, src: S) -> Timing {
        let a = src.read(self);
        let r = self.reg.a | a;
        self.reg.zero = r == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = false;
        self.reg.a = r;
        Timing::Default
    }

    fn cpl(&mut self) -> Timing {
        let a = self.reg.a;
        self.reg.a = !a;
        self.reg.subtract = true;
        self.reg.half_carry = true;
        Timing::Default
    }

    fn cp<S: Src<u8>>(&mut self, src: S) -> Timing {
        let a = self.reg.a;
        let value = src.read(self);
        self.reg.subtract = true;
        self.reg.carry = a < value;
        self.reg.zero = a == value;
        self.reg.half_carry = (a.wrapping_sub(value) & 0xf) > (a & 0xf);
        Timing::Default
    }

    fn inc_8<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) -> Timing {
        let value = loc.read(self);
        let result = value.wrapping_add(1);
        loc.write(self, result);
        self.reg.zero = result == 0;
        self.reg.subtract = false;
        self.reg.half_carry = (result & 0x0f) == 0x00;
        Timing::Default
    }

    fn inc_16<L: Dst<u16> + Src<u16> + Copy>(&mut self, loc: L) -> Timing {
        // No condition bits are affected for 16 bit inc
        let value = loc.read(self);
        loc.write(self, value.wrapping_add(1));
        Timing::Default
    }

    fn dec_8<L: Dst<u8> + Src<u8> + Copy>(&mut self, loc: L) -> Timing {
        let value = loc.read(self);
        let result = value.wrapping_sub(1);
        loc.write(self, result);
        self.reg.zero = result == 0;
        self.reg.subtract = true;
        self.reg.half_carry = (result & 0x0f) == 0x0f;
        Timing::Default
    }

    fn dec_16<L: Dst<u16> + Src<u16> + Copy>(&mut self, loc: L) -> Timing {
        // No condition bits are affected for 16 bit dec
        let value = loc.read(self);
        loc.write(self, value.wrapping_sub(1));
        Timing::Default
    }

    fn push<S: Src<u16>>(&mut self, src: S) -> Timing {
        let value = src.read(self);
        self.push_u16(value);
        Timing::Default
    }

    fn pop<D: Dst<u16>>(&mut self, dst: D) -> Timing {
        let value = self.pop_u16();
        dst.write(self, value);
        Timing::Default
    }

    fn di(&mut self) -> Timing {
        self.ime = false;
        Timing::Default
    }

    fn ei(&mut self) -> Timing {
        self.ime = true;
        Timing::Default
    }

    fn fetch_u8(&mut self) -> u8 {
        let pc = self.reg.pc;
        let value = self.read(pc);
        self.reg.pc = pc.wrapping_add(1);
        value
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch_u8() as u16;
        let high = self.fetch_u8() as u16;
        (high << 8) | low
    }

    fn push_u8(&mut self, value: u8) {
        let sp = self.reg.sp.wrapping_sub(1);
        self.write(sp, value);
        self.reg.sp = sp
    }

    fn push_u16(&mut self, value: u16) {
        self.push_u8((value >> 8) as u8);
        self.push_u8(value as u8);
    }

    fn pop_u8(&mut self) -> u8 {
        let sp = self.reg.sp;
        let value = self.read(sp);
        self.reg.sp = sp.wrapping_add(1);
        value
    }

    fn pop_u16(&mut self) -> u16 {
        let low = self.pop_u8() as u16;
        let high = self.pop_u8() as u16;
        (high << 8) | low
    }
}
