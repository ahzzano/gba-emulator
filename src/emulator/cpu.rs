use crate::emulator::bus::Bus;

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
        return Self {
            regs: [0; 16],
            cpsr: 0,
            spsr: [0; 6],
            bus: Box::new(Bus::default()),
        };
    }
}

impl CPU {}
