use super::interconnect::Interconnect;
use super::registers::{Registers, Reg8, Reg16};
use super::opcode::{CB_OPCODE_TIMES, OPCODE_TIMES};
use super::GameboyType;

use std::u8;
use std::u16;

pub struct Cpu<'a> {
    regs: Registers,
    interconnect: &'a mut Interconnect,
    cycle_count: u64,
}

struct HiMem;
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

trait Src8 {
    fn read(self, cpu: &mut Cpu) -> u8;
}

trait Src16 {
    fn read(self, cpu: &mut Cpu) -> u16;
}

trait Dst8 {
    fn write(self, cpu: &mut Cpu, value: u8);
}

trait Dst16 {
    fn write(self, cpu: &mut Cpu, value: u16);
}

impl Dst8 for Reg8 {
    fn write(self, cpu: &mut Cpu, value: u8) {
        cpu.regs.write_u8(self, value)
    }
}

impl Dst8 for ImmAddr16 {
    fn write(self, cpu: &mut Cpu, value: u8) {
        let address = cpu.fetch_u16();
        cpu.interconnect.write(address, value)
    }
}

impl Dst16 for Reg16 {
    fn write(self, cpu: &mut Cpu, value: u16) {
        cpu.regs.write_u16(self, value)
    }
}

impl Src8 for Reg8 {
    fn read(self, cpu: &mut Cpu) -> u8 {
        cpu.regs.read_u8(self)
    }
}

impl Src8 for Imm8 {
    fn read(self, cpu: &mut Cpu) -> u8 {
        cpu.fetch_u8()
    }
}

impl Src16 for Reg16 {
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.regs.read_u16(self)
    }
}

impl Src16 for Imm16 {
    fn read(self, cpu: &mut Cpu) -> u16 {
        cpu.fetch_u16()
    }
}

impl Src8 for HiMem {
    fn read(self, cpu: &mut Cpu) -> u8 {
        let offset = cpu.fetch_u8() as u16;
        let address = 0xff00 + offset;
        cpu.interconnect.read(address)
    }
}

impl Dst8 for HiMem {
    fn write(self, cpu: &mut Cpu, value: u8) {
        let offset = cpu.fetch_u8() as u16;
        let address = 0xff00 + offset;
        cpu.interconnect.write(address, value)
    }
}

impl<'a> Cpu<'a> {
    pub fn new(gb_type: GameboyType, interconnect: &'a mut Interconnect) -> Cpu {
        Cpu {
            regs: Registers::new(gb_type),
            interconnect: interconnect,
            cycle_count: 0,
        }
    }

    pub fn execute_instruction(&mut self) {

        let pc = self.regs.pc;
        println!("{}",
                 super::disassembler::disassemble(pc, self.interconnect));

        let opcode = self.fetch_u8();

        match opcode {

            0x00 => {}                                  // NOP
            0x10 => self.stop(),                        // STOP
            0x20 => self.jr(Cond::NotZero, Imm8),       // JR NZ,r8
            0x28 => self.jr(Cond::Zero, Imm8),          // JR Z,r8
            0x31 => self.load_16(Reg16::SP, Imm16),     // LD SP,d16
            0x3e => self.load(Reg8::A, Imm8),           // LD A,d8
            0xaf => self.xor(Reg8::A),                  // XOR A
            0xc3 => self.jump(Imm16),                   // JP a16
            0xc9 => self.ret(),                         // RET
            0xcb => self.execute_cb_instruction(),      // CB PREFIX
            0xcd => self.call(Imm16),                   // CALL nn
            0xe0 => self.load(HiMem, Reg8::A),          // LDH (a8),A
            0xe6 => self.and(Imm8),                     // AND d8
            0xea => self.load(ImmAddr16, Reg8::A),      // LD (a16),A
            0xf0 => self.load(Reg8::A, HiMem),          // LDH A,(a8)
            0xf3 => self.di(),                          // DI
            0xfe => self.compare(Imm8),                 // CP d8

            _ => panic!("Opcode not implemented: 0x{:x}", opcode),
        }

        let elapsed_cycles = OPCODE_TIMES[opcode as usize];
        self.add_cycles(elapsed_cycles);

    }

    fn execute_cb_instruction(&mut self) {

        let opcode = self.fetch_u8();

        match opcode {

            0x7f => self.bit(7, Reg8::A),       // BIT 7,A
            0x87 => self.res(0, Reg8::A),       // RES 0,A

            _ => panic!("CB opcode not implemented: 0x{:x}", opcode),
        }

        let elapsed_cycles = CB_OPCODE_TIMES[opcode as usize];
        self.add_cycles(elapsed_cycles);

    }

    fn stop(&self) {
        // http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
        //
        // Instruction STOP has according to manuals opcode 10 00 and
        // thus is 2 bytes long. Anyhow it seems there is no reason for
        // it so some assemblers code it simply as one byte instruction 10
        //
    }

    fn call<S: Src16>(&mut self, src: S) {
        let new_pc = src.read(self);
        let ret = self.regs.pc;
        self.push_u16(ret);
        self.regs.pc = new_pc
    }

    fn ret(&mut self) {
        let new_pc = self.pop_u16();
        self.regs.pc = new_pc
    }

    fn load<D: Dst8, S: Src8>(&mut self, dst: D, src: S) {
        let value = src.read(self);
        dst.write(self, value)
    }

    fn load_16<D: Dst16, S: Src16>(&mut self, dst: D, src: S) {
        let value = src.read(self);
        dst.write(self, value)
    }

    fn jump<S: Src16>(&mut self, src: S) {
        let new_pc = src.read(self);
        self.regs.pc = new_pc
    }

    fn jr<S: Src8>(&mut self, cond: Cond, src: S) {
        let offset = (src.read(self) as i8) as i16;

        use self::Cond::*;

        let jump = {
            match cond {
                Uncond => true,
                Zero => self.regs.zero,
                Carry => self.regs.carry,
                NotZero => !self.regs.zero,
                NotCarry => !self.regs.carry,
            }
        };

        if jump {
            let pc = self.regs.pc as i16;
            let new_pc = (pc + offset) as u16;
            self.regs.pc = new_pc
        }
    }

    fn and<S: Src8>(&mut self, src: S) {
        let value = src.read(self);
        let result = value & self.regs.a;
        self.regs.zero = result == 0;
        self.regs.subtract = false;
        self.regs.half_carry = true;
        self.regs.carry = false;
        self.regs.a = result
    }

    fn bit<S: Src8>(&mut self, bit: u8, src: S) {
        let value = src.read(self) >> bit;
        self.regs.zero = (value & 0x01) == 0;
        self.regs.subtract = false;
        self.regs.half_carry = true;
    }

    fn res<T: Src8 + Dst8 + Copy>(&mut self, bit: u8, target: T) {
        let value = target.read(self);
        let result = value & !(0x01 << bit);
        target.write(self, result)
    }

    fn xor<S: Src8>(&mut self, src: S) {
        let value = src.read(self);
        let result = self.regs.a ^ value;
        self.regs.zero = result == 0;
        self.regs.subtract = false;
        self.regs.half_carry = false;
        self.regs.carry = false;
        self.regs.a = result
    }

    fn compare<S: Src8>(&mut self, src: S) {
        let a = self.regs.a;
        let value = src.read(self);
        self.regs.subtract = true;
        self.regs.carry = a < value;
        self.regs.zero = a == value;
        self.regs.half_carry = (a.wrapping_sub(value) & 0xf) > (a & 0xf);
    }

    fn di(&mut self) {
        // TODO: Disable Interrupt
    }

    fn fetch_u8(&mut self) -> u8 {
        let pc = self.regs.pc;
        let value = self.interconnect.read(pc);
        self.regs.pc = pc.wrapping_add(1);
        value
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch_u8() as u16;
        let high = self.fetch_u8() as u16;
        (high << 8) | low
    }

    fn push_u8(&mut self, value: u8) {
        let sp = self.regs.sp.wrapping_sub(1);
        self.interconnect.write(sp, value);
        self.regs.sp = sp
    }

    fn push_u16(&mut self, value: u16) {
        self.push_u8((value >> 8) as u8);
        self.push_u8(value as u8);
    }

    fn pop_u8(&mut self) -> u8 {
        let sp = self.regs.sp;
        let value = self.interconnect.read(sp);
        self.regs.sp = sp.wrapping_add(1);
        value
    }

    fn pop_u16(&mut self) -> u16 {
        let low = self.pop_u8() as u16;
        let high = self.pop_u8() as u16;
        (high << 8) | low
    }

    fn add_cycles(&mut self, cycles: u8) {
        let new_count = self.cycle_count + (cycles as u64);
        self.cycle_count = new_count
    }
}
