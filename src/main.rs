mod chip8;
mod io;

use std::fs::File;
use structopt::StructOpt;
use chip8::Chip8;
use io::Io;

#[derive(StructOpt)]
#[structopt(about = "My supersweet Chip-8 interpreter.")]
struct Settings {
    #[structopt(short, long, parse(from_os_str))]
    /// Specifies a path to a chip-8 rom
    path: std::path::PathBuf,

    #[structopt(short, long, default_value = "1660")]
    /// Specifies the MAXIMUM refresh rate in microseconds, see --authentic-drawing
    update_rate: u64,

    /// Draws only when the actual instruction was executed
    #[structopt(short, long = "authentic")]
    authentic_drawing: bool,
}

fn main() {
    let args = Settings::from_args();
    run(args);
}

fn run (args: Settings) {
    let f = File::open(args.path).unwrap();

    let mut chip8 = Chip8::new_with_state();
    chip8.load_rom(&f);

    let mut io = Io::new();
    io.setup(args.update_rate);

    while io.window.is_open() && !io.window.is_key_down(minifb::Key::Escape) {

        if args.authentic_drawing {
            // Only draw when the actual drawing instruction was executed.
            if chip8.should_draw { io.draw(&chip8.framebuffer) } else { io.window.update() }
        } else {
            // Draw on every iteration.
            io.draw(&chip8.framebuffer);
        }

        // Set the keys after updating the window to get the new input.
        io.set_keys(&mut chip8.keys);
        chip8.tick();
    }
}
