use super::interconnect::Interconnect;
use super::registers::{Registers, Reg8, Reg16};
use super::opcode::{CB_OPCODE_TIMES, OPCODE_TIMES, OPCODE_COND_TIMES};
use super::GameboyType;

use std::u8;
use std::u16;

#[allow(dead_code)]
const CLOCK_SPEED: u32 = 4_194_304;

pub struct Cpu<'a> {
    reg: Registers,
    interconnect: &'a mut Interconnect,
    ime: bool,
    int_flags: u8,
    int_enable: u8,
    int_pending: bool,
}

struct ZMem;
struct Imm8;
struct Imm16;
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

impl Src<u8> for Mem<Reg16> {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let Mem(reg) = self;
        let addr = reg.read(cpu);
        cpu.read(addr)
    }
}


impl<'a> Cpu<'a> {
    pub fn new(gb_type: GameboyType, interconnect: &'a mut Interconnect) -> Cpu {
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
                0 => 0x0040,    // VBLANK
                1 => 0x0048,    // LCDC STATUS
                2 => 0x0050,    // TIMER OVERFLOW
                3 => 0x0058,    // SERIAL TRANSFER COMPLETE
                4 => 0x0060,    // P10-P13 INPUT SIGNAL
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
        println!("{}",
                 super::disassembler::disassemble(pc, self.interconnect));

        let opcode = self.fetch_u8();

        use super::registers::Reg8::*;
        use super::registers::Reg16::*;
        use self::Cond::*;

        let timing = {
            match opcode {
                0x00 => Timing::Default,                    // NOP
                0x01 => self.ld(BC, Imm16),                 // LD BC,d16
                0x03 => self.inc_u16(BC),                   // INC BC
                0x10 => self.stop(),                        // STOP
                0x18 => self.jr(Uncond, Imm8),              // JR,r8
                0x20 => self.jr(NotZero, Imm8),             // JR NZ,r8
                0x21 => self.ld(HL, Imm16),                 // LD HL,d16
                0x23 => self.inc_u16(HL),                   // INC HL
                0x28 => self.jr(Zero, Imm8),                // JR Z,r8
                0x2a => self.ldi(A, Mem(HL), HL),           // LDI A,(HL)
                0x31 => self.ld(SP, Imm16),                 // LD SP,d16
                0x3e => self.ld(A, Imm8),                   // LD A,d8
                0x78 => self.ld(A, B),                      // LD A,B
                0x7c => self.ld(A, H),                      // LD A,H
                0x7d => self.ld(A, L),                      // LD A,L
                0xaf => self.xor(A),                        // XOR A
                0xb1 => self.or(C),                         // OR C
                0xc1 => self.pop(BC),                       // POP BC
                0xc3 => self.jp(Imm16),                     // JP a16
                0xc5 => self.push(BC),                      // PUSH BC
                0xc9 => self.ret(),                         // RET
                0xcb => self.execute_cb_instruction(),      // CB PREFIX
                0xcd => self.call(Imm16),                   // CALL nn
                0xe0 => self.ld(ZMem, A),                   // LDH (a8),A
                0xe1 => self.pop(HL),                       // POP HL
                0xe5 => self.push(HL),                      // PUSH HL
                0xe6 => self.and(Imm8),                     // AND d8
                0xea => self.ld(Mem(Imm16), A),             // LD (a16),A
                0xf0 => self.ld(A, ZMem),                   // LDH A,(a8)
                0xf1 => self.pop(AF),                       // POP AF
                0xf3 => self.di(),                          // DI
                0xf5 => self.push(AF),                      // PUSH AF
                0xf8 => self.ei(),                          // EI
                0xfa => self.ld(A, Mem(Imm16)),             // LD A,(a16)
                0xfe => self.cp(Imm8),                      // CP d8

                _ => {
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

        match opcode {

            0x7f => self.bit(7, A),       // BIT 7,A
            0x87 => self.res(0, A),       // RES 0,A

            _ => panic!("CB opcode not implemented: 0x{:x}", opcode),
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

    fn call<S: Src<u16>>(&mut self, src: S) -> Timing {
        let new_pc = src.read(self);
        let ret = self.reg.pc;
        self.push_u16(ret);
        self.reg.pc = new_pc;
        Timing::Default
    }

    fn ret(&mut self) -> Timing {
        let new_pc = self.pop_u16();
        self.reg.pc = new_pc;
        Timing::Default
    }

    fn ld<T, D: Dst<T>, S: Src<T>>(&mut self, dst: D, src: S) -> Timing {
        let value = src.read(self);
        dst.write(self, value);
        Timing::Default
    }

    fn ldi<T, D: Dst<T>, S: Src<T>>(&mut self, dst: D, src: S, inc: Reg16) -> Timing {
        let t = self.ld(dst, src);
        self.inc_u16(inc);
        t
    }

    fn jp<S: Src<u16>>(&mut self, src: S) -> Timing {
        let new_pc = src.read(self);
        self.reg.pc = new_pc;
        Timing::Default
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
        let value = src.read(self);
        let result = value & self.reg.a;
        self.reg.zero = result == 0;
        self.reg.subtract = false;
        self.reg.half_carry = true;
        self.reg.carry = false;
        self.reg.a = result;
        Timing::Default
    }

    fn bit<S: Src<u8>>(&mut self, bit: u8, src: S) -> Timing {
        let value = src.read(self) >> bit;
        self.reg.zero = (value & 0x01) == 0;
        self.reg.subtract = false;
        self.reg.half_carry = true;
        Timing::Default
    }

    fn res<T: Src<u8> + Dst<u8> + Copy>(&mut self, bit: u8, target: T) -> Timing {
        let value = target.read(self);
        let result = value & !(0x01 << bit);
        target.write(self, result);
        Timing::Default
    }

    fn xor<S: Src<u8>>(&mut self, src: S) -> Timing {
        let value = src.read(self);
        let result = self.reg.a ^ value;
        self.reg.zero = result == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = false;
        self.reg.a = result;
        Timing::Default
    }

    fn or<S: Src<u8>>(&mut self, src: S) -> Timing {
        let value = src.read(self);
        let result = self.reg.a | value;
        self.reg.zero = result == 0;
        self.reg.subtract = false;
        self.reg.half_carry = false;
        self.reg.carry = false;
        self.reg.a = result;
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

    fn inc_u16<L: Dst<u16> + Src<u16> + Copy>(&mut self, loc: L) -> Timing {
        // No condition bits are affected for 16 bit inc
        let value = loc.read(self);
        loc.write(self, value.wrapping_add(1));
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
