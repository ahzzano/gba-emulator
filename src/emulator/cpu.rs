use std::{fs::File, io::Read, mem::offset_of};

use crate::{emulator::bus::Bus, utils::bit_utils::BitUtils};

const REG_SP: usize = 13;
const REG_LR: usize = 14;
const REG_PC: usize = 15;

// FLAGS
const FLAG_SIGN: usize = 31;
const FLAG_ZERO: usize = 30;
const FLAG_CARRY: usize = 29;
const FLAG_OVERFLOW: usize = 28;

// Memory Sizes
// Includes the On-Board work ram and On-Chip work ram
const RAM_SIZE: usize = 288_000;

// CPU is the struct used to emulate the GBA's CPU.
//
// Main Ref: https://problemkaputt.de/gbatek.htm#arminstructionsummary
#[derive(Debug)]
pub struct CPU {
    regs: [u32; 16],
    cpsr: u32,
    spsr: [u32; 6],
    bus: Box<Bus>,
    // The GamePak / Cartridge ROM
    rom: Vec<u8>,

    // the RAM of the GBA
    ram: [u8; RAM_SIZE],
}

impl Default for CPU {
    fn default() -> Self {
        let mut to_ret = Self {
            regs: [0; 16],
            cpsr: 0x6000_001F,
            spsr: [0; 6],
            bus: Box::new(Bus::default()),
            ram: [0; RAM_SIZE],
            rom: vec![0],
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

    pub fn load_rom(&mut self, mut file: File) {
        self.rom.clear();
        file.read_to_end(&mut self.rom).expect("Error reading file");
    }

    pub fn read_ram_u32(&self, addr: u32) -> u32 {
        u32::from_le_bytes([
            self.read_ram_u8(addr),
            self.read_ram_u8(addr + 1),
            self.read_ram_u8(addr + 2),
            self.read_ram_u8(addr + 3),
        ])
    }

    pub fn read_ram_u8(&self, addr: u32) -> u8 {
        match addr {
            0x02000000..=0x0203FFFF => self.ram[(addr - 0x02000000) as usize],
            0x03000000..=0x03007FFF => self.ram[(addr - 0x03000000) as usize],
            0x08000000..=0x0DFFFFFF => self.rom[(addr - 0x08000000) as usize],
            _ => 0,
        }
    }

    pub fn run_instr(&mut self, instr: u32) {
        let cond = instr.get_bits(28, 31);
        let instr_type = instr.get_bits(25, 27);
        if !self.can_exec(cond) {
            return;
        }

        match instr_type {
            0b000 | 0b001 => {
                self.exec_data_processing(instr);
            }
            0b101 => {
                // BRANCH
                self.exec_branch(instr);
            }
            _ => unimplemented!(),
        }
    }

    fn can_exec(&self, cond: u32) -> bool {
        match cond {
            // Z = 1
            0b0000 => self.cpsr.at_bit(FLAG_ZERO) == 1,
            0b0001 => self.cpsr.at_bit(FLAG_ZERO) == 0,
            0b0010 => self.cpsr.at_bit(FLAG_CARRY) == 1,
            0b0011 => self.cpsr.at_bit(FLAG_CARRY) == 0,
            0b0100 => self.cpsr.at_bit(FLAG_SIGN) == 1,
            0b0101 => self.cpsr.at_bit(FLAG_SIGN) == 0,
            0b0110 => self.cpsr.at_bit(FLAG_OVERFLOW) == 1,
            0b0111 => self.cpsr.at_bit(FLAG_OVERFLOW) == 0,
            0b1000 => self.cpsr.at_bit(FLAG_CARRY) == 1 && self.cpsr.at_bit(FLAG_ZERO) == 0,
            0b1001 => self.cpsr.at_bit(FLAG_CARRY) == 0 || self.cpsr.at_bit(FLAG_ZERO) == 1,
            0b1010 => self.cpsr.at_bit(FLAG_SIGN) == self.cpsr.at_bit(FLAG_OVERFLOW),
            0b1011 => self.cpsr.at_bit(FLAG_SIGN) != self.cpsr.at_bit(FLAG_OVERFLOW),
            0b1100 => {
                self.cpsr.at_bit(FLAG_ZERO) == 0
                    && (self.cpsr.at_bit(FLAG_OVERFLOW) == self.cpsr.at_bit(FLAG_SIGN))
            }
            0b1101 => {
                self.cpsr.at_bit(FLAG_ZERO) == 1
                    && (self.cpsr.at_bit(FLAG_OVERFLOW) != self.cpsr.at_bit(FLAG_SIGN))
            }
            0b1110 => true,
            0b1111 => false,
            _ => false,
        }
    }

    fn exec_branch(&mut self, instr: u32) {
        let link = instr.at_bit(24);
        let imm24 = instr.get_bits(0, 23);

        let offset = ((imm24 as i32) << 8) >> 6;

        let current_pc = self.regs[REG_PC];

        if link == 1 {
            self.regs[REG_LR] = current_pc.wrapping_add(4);
        }

        self.regs[REG_PC] = (current_pc as i32).wrapping_add(offset + 8) as u32;
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
            }
            _ => {
                unimplemented!()
            }
        }

        self.regs[REG_PC] = self.regs[REG_PC].wrapping_add(4);
    }
}

#[cfg(test)]
mod test {
    use crate::emulator::cpu::{CPU, REG_LR, REG_PC, REG_SP};

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

        let current_sp = cpu.regs[REG_PC];
        cpu.run_instr(0xEA000001);
        assert_eq!(cpu.regs[REG_PC], current_sp + 12);

        cpu.run_instr(0xEAFFFFFB);
        assert_eq!(cpu.regs[REG_PC], current_sp);

        cpu.run_instr(0xEB000001);
        assert_eq!(cpu.regs[REG_LR], current_sp + 4)
    }
}
