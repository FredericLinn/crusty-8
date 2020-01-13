use minifb::{Key, Window, WindowOptions, Scale};
use rand::{thread_rng, Rng};

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Cpu {
    // program counter,
    pc: u16,
    // index register
    i: u16,
    // registers
    v: [u8; 16],
    // memory
    memory: [u8; 4096],
    // stack
    stack: [u16; 16],
    // stack pointer
    sp: u8,
    // delay timer
    dt: u8,
    // sound timer
    st: u8,
    // framebuffer
    display: Display,
    // keyboard
    keys: [bool; 16],
}

// Split the two bytes of an opcode into four nibbles.
fn split_opcode(opcode: u16) -> (u8, u8, u8, u8) {
    (
        ((opcode & 0xF000) >> 12) as u8,
        ((opcode & 0x0F00) >> 8) as u8,
        ((opcode & 0x00F0) >> 4) as u8,
        (opcode & 0x000F) as u8,
    )
}

#[allow(dead_code)]
impl Cpu {
    fn new() -> Cpu {
        Cpu {
            pc: 0,
            i: 0,
            memory: [0; 4096],
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
            display: Display::new(),
            keys: [false; 16],
        }
    }

    fn init_memory(&mut self) {
        self.memory[0..(FONT.len())].copy_from_slice(&FONT);
    }

    fn reset(&mut self) {
        self.pc = 0x200;
        self.i = 0;
        self.init_memory();
        self.v =  [0; 16];
        self.stack = [0; 16];
        self.sp =  0;
        self.dt =  0;
        self.st = 0;
        self.display.cls();
        self.keys = [false; 16];
    }

    fn execute_cycle(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        self.process_opcode(opcode);
    }

    fn process_opcode(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x];
        let vy = self.v[y];
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let n = (opcode & 0x000F) as u8;

        // Pre-emptively increment the program counter,
        // jump instructions will overwrite the value anyway.
        self.pc += 2;

        let nibbles = split_opcode(opcode);

        match nibbles {

            // 00E0 Clear the screen
            (0x0, 0x0, 0xE, 0x0) => self.display.cls(),

            // 00EE Return from a subroutine
            // pop old pc form stack
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp as usize];
                self.stack[self.sp as usize] = 0;
            }

            // 0NNN Execute machine language subroutine at address NNN (usually not needed)
            (0x0, _, _, _) => panic!("Instruction 0NNN not implemented."),

            // 1NNN Jump to address NNN
            (0x1, _, _, _) => self.pc = nnn,
           
            // 2NNN Execute subroutine starting at address NNN
            (0x2, _, _, _) => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            },

            // 3XNN Skip the following instruction if the value of register VX equals NN
            (0x3, _, _, _) => self.pc += if vx == nn { 2 } else { 0 },

            // 4XNN Skip the following instruction if the value of register VX is not equal to NN
            (0x4, _, _, _) => self.pc += if vx != nn { 2 } else { 0 },

            // 5XY0 Skip the following instruction if the value of register VX is equal to the value of register VY
            (0x5, _, _, 0x0) => self.pc += if vx == vy { 2 } else { 0 },

            // 6XNN Store number NN in register VX
            (0x6, _, _, _) => self.v[x] = nn,

            // 7XNN Add the value NN to register VX
            (0x7, _, _, _) => self.v[x] += nn,

            // 8XY0 Store the value of register VY in register VX
            (0x8, _, _, 0x0) => self.v[x] = vy,

            // 8XY1 Set VX to VX OR VY
            (0x8, _, _, 0x1) => self.v[x] = vx | vy,

            // 8XY2 Set VX to VX AND VY
            (0x8, _, _, 0x2) => self.v[x] = vx & vy,

            // 8XY3 Set VX to VX XOR VY
            (0x8, _, _, 0x3) => self.v[x] = vx ^ vy,
           
            // 8XY4 Add the value of register VY to register VX
            // Set VF to 01 if a carry occurs
            // Set VF to 00 if a carry does not occur
            (0x8, _, _, 0x4) => {
                let (_, overflow) = self.v[x].overflowing_add(vy);
                if overflow { self.v[x] = 1 } ;
            },

            // 8XY5 Subtract the value of register VY from register VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            (0x8, _, _, 0x5) => {
                let (_, overflow) = self.v[x].overflowing_sub(vy);
                if overflow { self.v[x] = 0 };
            }

            // 8XY6 Store the value of register VY shifted right one bit in register VX
            // Set register VF to the least significant bit prior to the shift
            (0x8, _, _, 0x6) => {
                let lsb = vy & 0x1;
                self.v[x] = vy >> 1;
                self.v[0xF] = lsb;
            },

            // 8XY7 Set register VX to the value of VY minus VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            (0x8, _, _, 0x7) => {
                let (val, overflow) = vy.overflowing_sub(vx);
                self.v[x] = val;
                if overflow { self.v[0xF] = 0};
            },

            // 8XYE Store the value of register VY shifted left one bit in register VX
            // Set register VF to the most significant bit prior to the shift
            (0x8, _, _, 0xE) => {
                let msb = vy & 0x80;
                self.v[x] = vy << 1;
                self.v[0xF] = msb;
            },

            // 9XY0 Skip the following instruction if the value of register VX is not equal to the value of register VY
            (0x9, _, _, 0x0) => self.pc += if vx != vy { 2 } else { 0 },

            // ANNN Store memory address NNN in register I
            (0xA, _, _, _) => self.i = nnn,

            // BNNN Jump to address NNN + V0
            (0xB, _, _, _) => self.pc = nnn + self.v[0] as u16,

            // CXNN Set VX to a random number with a mask of NN
            (0xC, _, _, _) => {

                let mut rng = thread_rng();
                let rn: u8 = rng.gen();
                self.v[x] = rn & nn;
            },

            // DXYN Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I
            // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            (0xD, _, _, _) => self.display.draw(),

            // EX9E Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed
            (0xE, _, _, 0xE) => self.pc += if self.keys[vx as usize] { 2 } else { 0 },

            // EXA1 Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed
            (0xE, _, 0xA, 0x1) => self.pc += if !self.keys[vx as usize] { 2 } else { 0 },

            // FX07 Store the current value of the delay timer in register VX
            (0xF, _, 0x0, 0x7) => self.v[x] = self.dt,

            // FX0A Wait for a keypress and store the result in register VX
            (0xF, _, 0x0, 0xA) => {
            },

            // FX15 Set the delay timer to the value of register VX
            (0xF, _, 0x1, 0x5) => self.dt = vx,

            // FX18 Set the sound timer to the value of register VX
            (0xF, _, 0x1, 0x8) => self.st = vx,

            // FX1E Add the value stored in register VX to register I
            (0xF, _, 0x1, 0xE) => self.i += vx as u16,

            // FX29 Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX
            (0xF, _, 0x2, 0x9) => self.i = vx as u16 * 5,

            // FX33 Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I+1, and I+2
            (0xF, _, 0x3, 0x3) => (),

            // FX55 Store the values of registers V0 to VX inclusive in memory starting at address I
            // I is set to I + X + 1 after operation
            (0xF, _, 0x5, 0x5) => {
                for r in 0..=x {
                    self.memory[(self.i + r as u16) as usize] = self.v[r] as u8;
                }
                self.i += x as u16 + 1;
            }

            // FX65 Fill registers V0 to VX inclusive with the values stored in memory starting at address I
            // I is set to I + X + 1 after operation
            (0xF, _, 0x6, 0x5) => (),

            // Gotta catch 'em all!
            (_, _, _, _) => (),
        }
    }

    fn is_pressed(&mut self, window: &Window, hex_key: u8) -> bool {
        match hex_key {
            0x0 => window.is_key_down(Key::Key1),
            0x1 => window.is_key_down(Key::Key2),
            0x2 => window.is_key_down(Key::Key3),
            0x3 => window.is_key_down(Key::Key4),
            0x4 => window.is_key_down(Key::Q),
            0x5 => window.is_key_down(Key::W),
            0x6 => window.is_key_down(Key::E),
            0x7 => window.is_key_down(Key::R),
            0x8 => window.is_key_down(Key::A),
            0x9 => window.is_key_down(Key::S),
            0xA => window.is_key_down(Key::D),
            0xB => window.is_key_down(Key::F),
            0xC => window.is_key_down(Key::Y), // Assume QWERTZ
            0xD => window.is_key_down(Key::X),
            0xE => window.is_key_down(Key::C),
            0xF => window.is_key_down(Key::V),
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
struct Display {
    framebuffer: [u32; 64 * 32],
}

#[allow(dead_code)]
impl Display {
   fn new() -> Display {
        Display {
            framebuffer: [0; 64 * 32],
        }
    }

    fn cls(&mut self) {
        self.framebuffer = [0; 64 * 32];
    }

    fn draw(&self) {
    }
}

#[allow(dead_code)]
const KEY_MAP: [Key; 16] =
    [
        Key::Key1, Key::Key2, Key::Key3, Key::Key4,
        Key::Q, Key::W, Key::E, Key::R,
        Key::A, Key::S, Key::D, Key::F,
        Key::Y, Key::X, Key::C, Key::V,
       
    ];

#[allow(dead_code)]
const FONT: [u8; 16 * 5] =
    [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

fn main() {
    let mut cpu = Cpu::new();
    cpu.reset();
    let width = 64;
    let height = 32;

    let mut window = Window::new(
        "Crusty-8 (Press ESC to exit)",
        width,
        height,
        WindowOptions {
            resize: false,
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {panic!("{}", e);});

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in cpu.display.framebuffer.iter_mut() {
            *i = 0xFFFF_FFFF; // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&cpu.display.framebuffer, width, height)
            .unwrap();
    }
}
