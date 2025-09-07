use std::{fs::File, io::Read, mem::offset_of};

use crate::{
    emulator::bus::Bus,
    utils::bit_utils::{BitSetUtils, BitUtils},
};

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
const RAM_SIZE: usize = 294_912;

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
    pub fn step(&mut self) {
        let current_instr = self.read_ram_u32(self.regs[REG_PC]).to_be();
        println!("{current_instr:08x}");
        self.run_instr(current_instr);
    }

    pub fn load_rom(&mut self, mut file: File) {
        self.rom.clear();
        file.read_to_end(&mut self.rom).expect("Error reading file");
    }

    pub fn load_rom_bytes(&mut self, vec: Vec<u32>) {
        self.rom = vec
            .iter()
            .map(|v_u32| v_u32.to_le_bytes().to_vec())
            .flatten()
            .collect();
    }

    pub fn write_ram_u8(&mut self, addr: u32, value: u8) {
        let len1 = 0x0203FFFF - 0x02000000 + 1;
        match addr {
            0x02000000..=0x0203FFFF => self.ram[(addr - 0x02000000) as usize] = value,
            0x03000000..=0x03007FFF => self.ram[(addr - 0x03000000 + 262144) as usize] = value,
            0x08000000..=0x0DFFFFFF => self.rom[(addr - 0x08000000) as usize] = value,
            _ => (),
        }
    }

    pub fn read_ram_u32(&self, addr: u32) -> u32 {
        u32::from_le_bytes([
            self.read_ram_u8(addr),
            self.read_ram_u8(addr + 1),
            self.read_ram_u8(addr + 2),
            self.read_ram_u8(addr + 3),
        ])
    }

    pub fn write_ram_u32(&mut self, addr: u32, value: u32) {
        let bytes = value.to_le_bytes();
        for (index, value) in bytes.iter().enumerate() {
            self.write_ram_u8(addr + index as u32, *value);
        }
    }

    pub fn read_ram_u8(&self, addr: u32) -> u8 {
        match addr {
            0x02000000..=0x0203FFFF => self.ram[(addr - 0x02000000) as usize],
            0x03000000..=0x03007FFF => self.ram[(addr - 0x03000000 + 262144) as usize],
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
            0b0101 => self.regs[rd as usize] = rn_value + operand + self.cpsr.at_bit(FLAG_CARRY),
            0b0110 => {
                self.regs[rd as usize] = rn_value - operand + self.cpsr.at_bit(FLAG_CARRY) - 1
            }
            0b1010 => {
                // CMP Instruction
                let value = rn_value as i32 - operand as i32;
                self.cpsr = self.cpsr.set_bit(FLAG_SIGN, value < 0);
                self.cpsr = self.cpsr.set_bit(FLAG_ZERO, rn_value == operand);
                self.cpsr = self.cpsr.set_bit(FLAG_CARRY, rn_value >= operand);
                self.cpsr = self.cpsr.set_bit(
                    FLAG_OVERFLOW,
                    ((rn_value ^ operand) & (rn_value ^ value as u32)) >> 31 == 1,
                );
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
    use crate::{
        emulator::cpu::{CPU, FLAG_CARRY, FLAG_ZERO, REG_LR, REG_PC},
        utils::bit_utils::{BitSetUtils, BitUtils},
    };

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

    #[test]
    fn ram_reads() {
        let mut cpu = CPU::default();

        cpu.write_ram_u8(0x02000000, 0x69);
        cpu.write_ram_u8(0x02000001, 0x48);
        cpu.write_ram_u8(0x0203FFFF, 0x69);

        assert_eq!(cpu.read_ram_u8(0x02000000), 0x69);
        assert_eq!(cpu.read_ram_u8(0x02000001), 0x48);
        assert_eq!(cpu.read_ram_u8(0x0203FFFF), 0x69);

        cpu.write_ram_u8(0x03000000, 0x69);
        cpu.write_ram_u8(0x03000001, 0x48);
        cpu.write_ram_u8(0x03007FFF, 0x69);

        assert_eq!(cpu.read_ram_u8(0x03000000), 0x69);
        assert_eq!(cpu.read_ram_u8(0x03000001), 0x48);
        assert_eq!(cpu.read_ram_u8(0x03007FFF), 0x69);

        cpu.write_ram_u32(0x03000026, 0x69420);
        assert_eq!(cpu.read_ram_u32(0x03000026), 0x69420);
    }

    #[test]
    fn cmp_tests() {
        let mut cpu = CPU::default();

        cpu.regs[0] = 1;
        cpu.regs[1] = 1;

        cpu.run_instr(0xE1500001);
        assert_eq!(cpu.cpsr.at_bit(FLAG_ZERO), 1);

        cpu.regs[2] = 2;
        cpu.run_instr(0xE1510002);
        assert_eq!(cpu.cpsr.at_bit(FLAG_ZERO), 0);
    }

    #[test]
    fn carry_intructions() {
        let mut cpu = CPU::default();

        cpu.cpsr = cpu.cpsr.set_bit(FLAG_CARRY, true);
        cpu.regs[1] = 1;
        cpu.regs[2] = 1;

        cpu.run_instr(0xE0A11002);
        assert_eq!(cpu.regs[1], 3);

        cpu.regs[1] = 1;
        cpu.regs[2] = 1;
        cpu.cpsr = cpu.cpsr.set_bit(FLAG_CARRY, false);

        cpu.run_instr(0xE0A11002);
        println!("{0:?}", cpu.regs);
        assert_eq!(cpu.regs[1], 2);

        cpu.run_instr(0xE0C11002);
        assert_eq!(cpu.regs[1], 0);

        cpu.regs[1] = 1;
        cpu.regs[2] = 1;
        cpu.cpsr = cpu.cpsr.set_bit(FLAG_CARRY, true);
        assert_eq!(cpu.regs[1], 1);
    }

    #[test]
    fn fibonacci_iteration() {
        let mut cpu = CPU::default();

        // Load 1 into r1
        // Load 1 into r2
        // r1 = 2
        // r2 = 3
        // r1 = 5
        // MOV r1, r5
        cpu.load_rom_bytes(vec![
            0x0110A0E3, 0x0120A0E3, 0x021081E0, 0x012082E0, 0x021081E0, 0x0150A0E1,
        ]);

        println!("{0:?}", cpu.rom);

        for i in 0..6 {
            cpu.step();
        }
        println!("{0:?}", cpu.regs);
        println!("{0:x}", cpu.read_ram_u32(cpu.regs[REG_PC] - 4));

        assert_eq!(cpu.regs[5], 5);
    }
}
