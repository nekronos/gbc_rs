use super::interconnect::Interconnect;
use super::registers::{Registers, Reg8, Reg16};
use super::opcode::{CB_OPCODE_TIMES, OPCODE_TIMES, OPCODE_COND_TIMES};
use super::GameboyType;

use std::u8;
use std::u16;

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
struct ImmAddr16;

#[allow(dead_code)]
enum Cond {
    Uncond,
    Zero,
    Carry,
    NotZero,
    NotCarry,
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

impl Dst<u8> for ImmAddr16 {
    fn write(self, cpu: &mut Cpu, val: u8) {
        let addr = cpu.fetch_u16();
        cpu.write(addr, val)
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
                0x00 => Timing::Default,                     // NOP
                0x10 => self.stop(),                        // STOP
                0x20 => self.jr(NotZero, Imm8),             // JR NZ,r8
                0x28 => self.jr(Zero, Imm8),                // JR Z,r8
                0x31 => self.ld(SP, Imm16),                 // LD SP,d16
                0x3e => self.ld(A, Imm8),                   // LD A,d8
                0xaf => self.xor(A),                        // XOR A
                0xc3 => self.jp(Imm16),                     // JP a16
                0xc9 => self.ret(),                         // RET
                0xcb => self.execute_cb_instruction(),      // CB PREFIX
                0xcd => self.call(Imm16),                   // CALL nn
                0xe0 => self.ld(ZMem, A),                   // LDH (a8),A
                0xe6 => self.and(Imm8),                     // AND d8
                0xea => self.ld(ImmAddr16, A),              // LD (a16),A
                0xf0 => self.ld(A, ZMem),                   // LDH A,(a8)
                0xf3 => self.di(),                          // DI
                0xf8 => self.ei(),                          // EI
                0xfe => self.cp(Imm8),                      // CP d8

                _ => panic!("Opcode not implemented: 0x{:x}", opcode),
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

    fn jp<S: Src<u16>>(&mut self, src: S) -> Timing {
        let new_pc = src.read(self);
        self.reg.pc = new_pc;
        Timing::Default
    }

    fn jr<S: Src<u8>>(&mut self, cond: Cond, src: S) -> Timing {
        let offset = (src.read(self) as i8) as i16;

        use self::Cond::*;

        let jump = {
            match cond {
                Uncond => true,
                Zero => self.reg.zero,
                Carry => self.reg.carry,
                NotZero => !self.reg.zero,
                NotCarry => !self.reg.carry,
            }
        };

        if jump {
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

    fn cp<S: Src<u8>>(&mut self, src: S) -> Timing {
        let a = self.reg.a;
        let value = src.read(self);
        self.reg.subtract = true;
        self.reg.carry = a < value;
        self.reg.zero = a == value;
        self.reg.half_carry = (a.wrapping_sub(value) & 0xf) > (a & 0xf);
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
