# Crusty-8

![A black and white maze drawn by the interpreter](maze-david-winter.png "Maze Demo by David Winter")

Crusty-8 is a... you guessed it! It's a [Chip-8](https://en.wikipedia.org/wiki/Chip-8) interpreter.
It can handle 35 different instructions, contains a whopping 4Kb of RAM (that's 4096 bytes!)
and outputs to a monochrome 64 x 32 pixel display.

But what sets this implementation apart from all the others? Well, I made it!
Even though this is a simple project, I'm still happy to have finished it.
It's a really manageable entry point for emulation and was a fitting choice for
my first contact with Rust.

If you have any suggestions or remarks, please let me know.

## Build instructions

In order to build the interpreter, a recent Rust toolchain has to be installed.
An easy way is to use [rustup](https://rustup.rs/). Afterwards it's just a matter
of navigating into the repository folder and entering:
```bash
$ cargo build --release
```

After the build process the final binary is located at *./target/release/*.

Run it with at least the path to a rom in order to get started:
```bash
$ ./crusty-8 --path <path-to-rom>
```

## Available options

Apart from the path, one can also adjust the refresh rate and set a drawing mode.
By default the contents of the framebuffer get drawn to the screen after every cycle,
which can be changed by adding the *-a* or *--authentic* flag. This makes sure
that the window only gets refreshed after the actual draw instruction (0xDXYN),
which makes for a slightly choppier experience.

## Acknowledgments

I mainly used Matthew Mikolay's [Mastering Chip-8](http://mattmik.com/files/chip8/mastering/chip8.html) for the implementation,
with the occasional look at the [Wikipedia page](https://en.wikipedia.org/wiki/Chip-8).

Being completely new to Rust, there were also multiple occasions were I took
inspiration from Colin Eberhardt's [wasm-chip8-project](https://github.com/ColinEberhardt/wasm-rust-chip8).
I wanted to do as much as possible on my own, but some things I just couldn't unsee,
like his handling of the 0xFX0A-instruction.

And finally the roms included in this repository were collected
by Martijn Wenting from [Revival Studios](http://www.revival-studios.com/).

I'm grateful that these resources exist, so thank you :).
