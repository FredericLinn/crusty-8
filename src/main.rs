use std::fs::File;

mod chip8;
mod io;
use chip8::Chip8;
use io::Io;

fn main() {
    // TODO: parse command line arguments: path_to_rom, screensettings

    // let path = "./roms/roms/demos/Trip8 Demo (2008) [Revival Studios].ch8";
    // let path = "./roms/roms/demos/Trip8 Demo (2008) [Revival Studios].ch8";
    // let path = "./roms/PONG2";
    let path = "./roms/BRIX";

    let f = File::open(path).unwrap();

    let mut chip8 = Chip8::new_with_state();
    chip8.load_rom(&f);
    let mut io = Io::new();
    io.setup();

    while io.window.is_open() && !io.window.is_key_down(minifb::Key::Escape) {
        // if chip8.should_draw { io.draw(&chip8.framebuffer) } else { io.window.update() }

        // Always draw with buffer, even though technically the draw instruction was not called.
        // Makes for a more fluent experience.
        io.draw(&chip8.framebuffer);

        // Set the keys after updating the window to get the new input.
        io.set_keys(&mut chip8.keys);
        chip8.tick();
    }
}
