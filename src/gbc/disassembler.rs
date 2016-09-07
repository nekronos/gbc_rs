use super::cpu::Cpu;
use super::interconnect::Interconnect;
use super::opcode::OPCODE_NAME_LUT;
use super::opcode::CB_OPCODE_NAME_LUT;
use std::string::String;

pub fn disassemble(program_counter: u16, interconnect: &Interconnect) -> String {

    let mut pc = program_counter;
    let opcode = interconnect.read(pc);
    pc = pc + 1;

    let mut disasm_str = String::from({
        match opcode {
            0xcb => {
                let cb_opcode = interconnect.read(pc);
                pc = pc + 1;
                CB_OPCODE_NAME_LUT[cb_opcode as usize]
            }
            _ => OPCODE_NAME_LUT[opcode as usize],
        }
    });

    if disasm_str.contains("nn") {
        let low = interconnect.read(pc) as u16;
        pc = pc + 1;
        let high = interconnect.read(pc) as u16;

        let address = (high << 8) | low;
        let address = format!("{:04X}", address);

        disasm_str = disasm_str.replace("nn", &address)
    } else if disasm_str.contains("n") {
        let low = interconnect.read(pc);
        let address = format!("{:02X}", low);
        disasm_str = disasm_str.replace("n", &address)
    }

    format!("{:04X}\t\t{}", program_counter, disasm_str)
}
