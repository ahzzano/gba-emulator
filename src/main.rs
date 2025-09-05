use raylib::prelude::*;

pub mod emulator;
use crate::emulator::cpu::CPU;

const GBA_RES_WIDTH: i32 = 240;
const GBA_RES_HEIGHT: i32 = 160;

const WIDTH: i32 = 1080;
const HEIGHT: i32 = 720;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("GBA Emulator")
        .build();

    let cpu = CPU::default();

    while !rl.window_should_close() {
        if rl.is_key_down(KeyboardKey::KEY_Q) {
            break;
        }

        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}
