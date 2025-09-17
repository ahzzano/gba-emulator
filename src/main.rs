use raylib::prelude::*;
use std::{
    fmt::format, fs::{self, File}, thread::sleep, time::Duration
};

pub mod emulator;
pub mod utils;

use crate::emulator::cpu::CPU;

const GBA_RES_WIDTH: i32 = 240;
const GBA_RES_HEIGHT: i32 = 160;

const WIDTH: i32 = 1080;
const HEIGHT: i32 = 720;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filepath = &args[1];

    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("GBA Emulator")
        .build();

    let mut cpu = CPU::default();

    // let binary = fs::read(filepath).expect("File does not exist");
    let binary = File::open(filepath).expect("File does not exist");
    cpu.load_rom(binary);

    while !rl.window_should_close() {
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            break;
        }
        // if rl.is_key_pressed(KeyboardKey::KEY_F) {
        //     cpu.step();
        //     println!("{0:?}", cpu.regs);
        // }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text(&format!("{filepath}"), 12, 12, 20, Color::BLACK);

        for i in 0..16  {
            d.draw_text(&format!("r{i}: {:08x}", cpu.regs[i]), 12, 40 + 20 * i as i32, 20, Color::BLACK);
        }

        cpu.step();
        sleep(Duration::from_millis(200));
    }
}
