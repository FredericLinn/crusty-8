use std::fs::File;

mod chip8;
mod io;
use chip8::Chip8;

fn main() {
    let mut chip8 = Chip8::new();
    chip8.reset();

    // let f = File::open("./roms/roms/demos/Trip8 Demo (2008) [Revival Studios].ch8").unwrap();
    // let f = File::open("./roms/PONG2").unwrap();
    let f = File::open("./roms/BRIX").unwrap();
   
    chip8.load_rom(&f);
    let mut buf = vec![0u32; 64 * 32];

    while chip8.io.window.is_open() && !chip8.io.window.is_key_down(minifb::Key::Escape) {

        for (i, pixel) in buf.iter_mut().enumerate() {
            *pixel = if chip8.io.framebuffer[i] { 0xFF_FF_FF_FF } else { 0 };
        }

        chip8.io.draw(&buf);
        chip8.io.set_keys();
        chip8.tick();
    }
}
