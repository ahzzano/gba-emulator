use crate::{emulator::bus::Bus, utils::bit_utils::BitUtils};

const REG_SP: usize = 13;
const REG_LR: usize = 14;
const REG_PC: usize = 15;

//
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
        let instr_type = (instr >> 25) & 0b111;

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

    fn exec_data_processing(&mut self, instr: u32) {
        let kind = (instr >> 25) & 0x7;
        let rn = (instr >> 16) & 0xF;
        let rd = (instr >> 12) & 0xF;
        let opcode = (instr >> 21) & 0xF;
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
            _ => {
                unimplemented!()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::emulator::cpu::CPU;

    #[test]
    fn data_processing() {
        let mut cpu = CPU::default();

        // Add Instruction
        // cpu.run_instr(0x7F0080E2);
        cpu.run_instr(0xE2810020);
        println!("{0:?}", cpu.regs);
        assert!(cpu.regs[0] == 0x20)
    }
}

