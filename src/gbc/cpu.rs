use super::interconnect::Interconnect;
use super::registers::Registers;
use super::registers::Reg8;
use super::registers::Reg16;
use super::Model;

use std::u8;
use std::u16;


pub struct Cpu<'a> {
    regs: Registers,
    interconnect: &'a Interconnect,
}

trait Src8 {
    fn read(&self, cpu: &mut Cpu) -> u8;
}

trait Dst8 {
    fn write(&self, cpu: &mut Cpu, value: u8);
}

trait Src16 {
    fn read(&self, cpu: &mut Cpu) -> u16;
}

trait Dst16 {
    fn write(&self, cpu: &mut Cpu, value: u16);
}

impl Src8 for Reg8 {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.regs.read_u8(self.clone())
    }
}

impl Dst8 for Reg8 {
    fn write(&self, cpu: &mut Cpu, value: u8) {
        cpu.regs.write_u8(self.clone(), value)
    }
}

impl Src16 for Reg16 {
    fn read(&self, cpu: &mut Cpu) -> u16 {
        cpu.regs.read_u16(self.clone())
    }
}

impl Dst16 for Reg16 {
    fn write(&self, cpu: &mut Cpu, value: u16) {
        cpu.regs.write_u16(self.clone(), value)
    }
}

struct Immediate8;

impl Src8 for Immediate8 {
    fn read(&self, cpu: &mut Cpu) -> u8 {
        cpu.fetch_u8()
    }
}

struct Immediate16;

impl Src16 for Immediate16 {
    fn read(&self, cpu: &mut Cpu) -> u16 {
        cpu.fetch_u16()
    }
}

impl<'a> Cpu<'a> {
    pub fn new(interconnect: &'a Interconnect) -> Cpu {
        Cpu {
            regs: Registers::new(),
            interconnect: interconnect,
        }
    }

    pub fn execute_instruction(&mut self) {

        println!("0x{:x}", self.regs.pc);

        let opcode = self.fetch_u8();

        match opcode {
            // NOP
            0x00 => {}

            // JP a16
            0xc3 => self.jump(Immediate16),

            _ => panic!("Opcode not implemented: 0x{:x}", opcode),
        }

    }

    fn load<D: Dst8, S: Src8>(&mut self, dst: D, src: S) {
        let value = src.read(self);
        dst.write(self, value)
    }

    fn jump<S: Src16>(&mut self, src: S) {
        let new_pc = src.read(self);
        self.regs.pc = new_pc
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
}
