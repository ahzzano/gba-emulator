use std::mem::offset_of;

use crate::{emulator::bus::Bus, utils::bit_utils::BitUtils};

const REG_SP: usize = 13;
const REG_LR: usize = 14;
const REG_PC: usize = 15;

// FLAGS
const FLAG_SIGN: usize = 31;
const FLAG_ZERO: usize = 30;
const FLAG_CARRY: usize = 29;
const FLAG_OVERFLOW: usize = 28;

// CPU is the struct used to emulate the GBA's CPU.
//
// Main Ref: https://problemkaputt.de/gbatek.htm#arminstructionsummary
#[derive(Debug)]
pub struct CPU {
    regs: [u32; 16],
    cpsr: u32,
    spsr: [u32; 6],
    bus: Box<Bus>,
}

impl Default for CPU {
    fn default() -> Self {
        let mut to_ret = Self {
            regs: [0; 16],
            cpsr: 0x6000_001F,
            spsr: [0; 6],
            bus: Box::new(Bus::default()),
        };

        to_ret.regs[REG_SP] = 0x03007F00;
        to_ret.regs[REG_PC] = 0x08000000;

        to_ret
    }
}

impl CPU {
    pub fn step() {
        todo!()
    }

    pub fn run_instr(&mut self, instr: u32) {
        let instr_type = instr.get_bits(25, 27);

        match instr_type {
            0b000 | 0b001 => {
                self.exec_data_processing(instr);
            }
            0b101 => {
                // BRANCH
            }
            _ => unimplemented!(),
        }
    }

    fn exec_branch(&mut self, instr: u32) {
        let link = instr.at_bit(24);
        let imm24 = instr.get_bits(0, 23);

        let offset = (((imm24 as i32) << 8) >> 6) as u32;

        let current_pc = self.regs[REG_PC];
        let next_pc = current_pc + 8;

        if link == 1 {
           self.regs[REG_LR] = next_pc - 4;
        }

        self.regs[REG_PC] = next_pc.wrapping_add(offset);
    }

    fn exec_data_processing(&mut self, instr: u32) {
        let kind = instr.get_bits(25, 27);
        let rn = instr.get_bits(16, 19);
        let rd = instr.get_bits(12, 15);
        let opcode = instr.get_bits(21, 24);
        let s = instr.at_bit(20);
        println!("Data Processing Instr: {instr:0x}");
        println!("Opcode: {opcode}");

        let operand = if (kind & 0x1) == 1 {
            // immediate
            let imm8 = instr & 0xFF;
            println!("{imm8:08b}");
            let rot = (instr >> 8) & 0xF;

            imm8.rotate_right(rot * 2)
        } else {
            // operands
            let rs = instr & 0xF;
            let shift = (instr >> 4) & 0x7;

            self.regs[rs as usize]
        };

        let rn_value = self.regs[rn as usize];

        match opcode {
            0b0100 => self.regs[rd as usize] = rn_value + operand,
            0b1101 => self.regs[rd as usize] = operand,
            0b0010 => {
                let (value, flag) = rn_value.overflowing_sub(operand);
                self.regs[rd as usize] = value;
                if flag && s != 0 {
                    todo!("Implement SUBS, ADDS, and MOVS respectively");
                }
            },
            _ => {
                unimplemented!()
            }
        }

        self.regs[REG_PC] = self.regs[REG_PC].wrapping_add(4);
    }
}

#[cfg(test)]
mod test {
    use crate::emulator::cpu::{CPU, REG_SP};

    #[test]
    fn data_processing() {
        let mut cpu = CPU::default();

        // Add Instruction
        // cpu.run_instr(0x7F0080E2);
        cpu.run_instr(0xE2810020);
        assert_eq!(cpu.regs[0], 0x20);

        cpu.run_instr(0xE1A01000);
        assert_eq!(cpu.regs[1], 0x20);

        cpu.run_instr(0xE2414005);
        println!("{0:?}", cpu.regs);
        assert_eq!(cpu.regs[4], 0x20 - 0x05);
    }

    #[test]
    fn branch() {
        let mut cpu = CPU::default();

        cpu.run_instr(0xEA000001);
        println!("{0:?}", cpu.regs);
        assert_eq!(cpu.regs[REG_SP], 12);
    }


}
