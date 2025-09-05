use raylib::prelude::*;

const GBA_RES_WIDTH: i32 = 240;
const GBA_RES_HEIGHT: i32 = 160;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(GBA_RES_WIDTH, GBA_RES_HEIGHT)
        .title("GBA Emulator")
        .build();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::WHITE);
        d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
}
