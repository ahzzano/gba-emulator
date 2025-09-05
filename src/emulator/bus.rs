const VRAM_SIZE: usize = 24_576;

// Bus is used to handle other hardware that GBA may use
// Currently implemented:
//   VRAM
#[derive(Debug)]
pub struct Bus {
    video_ram: [u32; VRAM_SIZE],
}

impl Default for Bus {
    fn default() -> Self {
        Self {
            video_ram: [0; VRAM_SIZE],
        }
    }
}
