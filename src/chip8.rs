use rand::Rng;

use std::fs::File;
use std::io::prelude::*;

use crate::io::{Io, WIDTH, HEIGHT};

pub struct Chip8 {
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
    sp: usize,
    // delay timer
    dt: u8,
    // sound timer
    st: u8,
    // Input / Output
    pub io: Io,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            pc: 0,
            i: 0,
            memory: [0; 4096],
            v: [0; 16],
            stack: [0; 16],
            sp: 0,
            dt: 0,
            st: 0,
            io: Io::new(),
        }
    }

    pub fn reset(&mut self) {
        self.pc = 0x200;
        self.i = 0;
        self.init_memory();
        self.v = [0; 16];
        self.stack = [0; 16];
        self.sp = 0;
        self.dt = 0;
        self.st = 0;
        self.io.setup();
        self.io.cls();
    }

    pub fn load_rom(&mut self, f: &File) {
        for (i, byte) in f.bytes().enumerate() {
            let index = 0x200 + i;
            self.memory[index] = byte.unwrap();
        }
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

    fn init_memory(&mut self) {
        self.memory = [0; 4096];
        self.memory[0..(FONT.len())].copy_from_slice(&FONT);
    }

    pub fn tick(&mut self) {
        let opcode =
            (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);
        self.process_opcode(opcode);
    }

    fn process_opcode(&mut self, opcode: u16) {
        println!{"Processing: {:#02x}", opcode};
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

        if self.dt > 0 { self.dt -= 1 };
        if self.st > 0 { self.st -= 1 };

        let nibbles = Chip8::split_opcode(opcode);

        match nibbles {
            // 00E0 Clear the screen
            (0x0, 0x0, 0xE, 0x0) => self.io.cls(),

            // 00EE Return from a subroutine
            // pop old pc form stack
            (0x0, 0x0, 0xE, 0xE) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
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
            }

            // 3XNN Skip the following instruction if the value of register VX equals NN
            (0x3, _, _, _) => self.pc += if vx == nn { 2 } else { 0 },

            // 4XNN Skip the following instruction if the value of register VX is not equal to NN
            (0x4, _, _, _) => self.pc += if vx != nn { 2 } else { 0 },

            // 5XY0 Skip the following instruction if the value of register VX is equal to the value of register VY
            (0x5, _, _, 0x0) => self.pc += if vx == vy { 2 } else { 0 },

            // 6XNN Store number NN in register VX
            (0x6, _, _, _) => self.v[x] = nn,

            // 7XNN Add the value NN to register VX
            (0x7, _, _, _) => {
                let (val, _) = vx.overflowing_add(nn);
                self.v[x] = val;
            },

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
                let (sum, overflow) = vx.overflowing_add(vy);
                self.v[x] = sum;
                self.v[0xF] = if overflow { 1 } else { 0 };
            },

            // 8XY5 Subtract the value of register VY from register VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            (0x8, _, _, 0x5) => {
                let (sum, overflow) = vx.overflowing_sub(vy);
                self.v[x] = sum;
                self.v[0xF] = if overflow { 0 } else { 1 };
            },

            // 8XY6 Store the value of register VY shifted right one bit in register VX
            // Set register VF to the least significant bit prior to the shift
            (0x8, _, _, 0x6) => {
                let lsb = vx & 0x1;
                self.v[x] = vy >> 1;
                self.v[0xF] = lsb;
            }

            // 8XY7 Set register VX to the value of VY minus VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            (0x8, _, _, 0x7) => {
                let (val, overflow) = vy.overflowing_sub(vx);
                self.v[x] = val;
                self.v[0xF] = if overflow { 0 } else { 1 }
            }

            // 8XYE Store the value of register VY shifted left one bit in register VX
            // Set register VF to the most significant bit prior to the shift
            (0x8, _, _, 0xE) => {
                let msb = (vx & 0x80) >> 7;
                self.v[x] = vy << 1;
                self.v[0xF] = msb;
            }

            // 9XY0 Skip the following instruction if the value of register VX is not equal to the value of register VY
            (0x9, _, _, 0x0) => self.pc += if vx != vy { 2 } else { 0 },

            // ANNN Store memory address NNN in register I
            (0xA, _, _, _) => self.i = nnn,

            // BNNN Jump to address NNN + V0
            (0xB, _, _, _) => self.pc = nnn + self.v[0] as u16,

            // CXNN Set VX to a random number with a mask of NN
            (0xC, _, _, _) => {
                let mut rng = rand::thread_rng();
                let rn: u8 = rng.gen();
                self.v[x] = rn & nn;
            }

            // DXYN Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I
            // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            (0xD, _, _, _) => {
                self.v[0xF] = 0; // Never forget :(
                let range = (self.i as usize)..(self.i + n as u16) as usize;
                let sprite_data: &[u8] = &self.memory[range];

                for (i, current_byte) in sprite_data.iter().enumerate() {
                    for j in 0..8 {
                        let current_bit = current_byte >> (7 - j) & 0x01;
                        if current_bit != 0 {
                            let x = (vx as usize + j) % WIDTH;
                            let y = (vy as usize + i) % HEIGHT;

                            let index = y * WIDTH + x;

                            let on = self.io.framebuffer[index];

                            if on {
                                self.v[0xF] = 1;
                            }
                            self.io.framebuffer[index] = !on;
                        }
                    }
                }
            },

            // EX9E Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed
            (0xE, _, _, 0xE) => self.pc += if self.io.is_key_pressed(vx) { 2 } else { 0 },

            // EXA1 Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed
            (0xE, _, 0xA, 0x1) => self.pc += if !self.io.is_key_pressed(vx) { 2 } else { 0 },

            // FX07 Store the current value of the delay timer in register VX
            (0xF, _, 0x0, 0x7) => self.v[x] = self.dt,

            // FX0A Wait for a keypress and store the result in register VX
            (0xF, _, 0x0, 0xA) => {
                // https://github.com/ColinEberhardt/wasm-rust-chip8
                // I love the simplicity of just subtracting from the program counter.
                self.pc -= 2;
                for key in 0x0..=0xF {
                    if self.io.is_key_pressed(key) {
                        self.v[x] = key;
                        self.pc += 2;
                    }
                }
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
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.i as usize] = vx / 100;
                self.memory[self.i as usize + 1] = (vx / 10) % 10;
                self.memory[self.i as usize + 2] = vx % 10;
            }

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
            (0xF, _, 0x6, 0x5) => {
                for r in 0..=x {
                    self.v[r] = self.memory[(self.i + r as u16) as usize];
                }
                self.i += x as u16 + 1;
            }

            // Gotta catch 'em all!
            (_, _, _, _) => (),
        }
    }
}

#[allow(dead_code)]
const FONT: [u8; 16 * 5] = [
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reset() {
        let mut c = Chip8::new();
        c.reset();

        assert_eq!(0x200, c.pc);
        assert_eq!(4096, c.memory.len());
    }

    #[test]
    fn instruction_00e0() {
        let mut c = Chip8::new();
        c.reset();
        c.process_opcode(0x00E0);
        for pixel in c.io.framebuffer.iter() {
            assert_eq!(false, *pixel);
        }
    }

    #[test]
    fn instruction_00ee() {
        let mut c = Chip8::new();
        c.reset();

        assert_eq!(0x200, c.pc);
        // Execute subroutine
        c.process_opcode(0x2123);
        assert_eq!(1, c.sp);
        assert_eq!(0x123, c.pc);
        // Execute second subroutine
        c.process_opcode(0x2000);
        assert_eq!(2, c.sp);
        // Return from subroutine
        c.process_opcode(0x00EE);
        assert_eq!(1, c.sp);
        assert_eq!(0x125, c.pc);
        // Return from subroutine
        c.process_opcode(0x00EE);
        assert_eq!(0, c.sp);
        assert_eq!(0x202, c.pc)
    }

    #[test]
    fn instruction_1nnn() {
        let mut c = Chip8::new();
        c.reset();
        c.process_opcode(0x1123);
        assert_eq!(0x123, c.pc);
    }

    #[test]
    fn instruction_3xnn() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0x3] = 0x33;
        c.process_opcode(0x3333);
        assert_eq!(0x204, c.pc);
    }

    #[test]
    fn instruction_4xnn() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0x3] = 0x33;
        c.process_opcode(0x4333);
        assert_eq!(0x202, c.pc);
    }

    #[test]
    fn instruction_5xy0() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0x3] = 0x33;
        c.v[0x4] = 0x33;
        c.process_opcode(0x4330);
        assert_eq!(0x204, c.pc);
    }

    #[test]
    fn instruction_6xnn() {
        let mut c = Chip8::new();
        c.reset();
        c.process_opcode(0x6123);
        assert_eq!(0x23, c.v[1]);
    }

    #[test]
    fn instruction_7xnn() {
        let mut c = Chip8::new();
        c.reset();
        c.v[1] = 0x23;
        c.process_opcode(0x7123);
        assert_eq!(0x23 + 0x23, c.v[1]);
    }

    #[test]
    fn instruction_8xy0() {
        let mut c = Chip8::new();
        c.reset();
        c.v[1] = 0x23;
        assert_eq!(0x0, c.v[0]);
        c.process_opcode(0x8010);
        assert_eq!(0x23, c.v[0]);
    }

    #[test]
    fn instruction_8xy1() {
        let mut c = Chip8::new();
        c.reset();
        c.v[1] = 0x0F;
        c.v[0] = 0xF0;
        c.process_opcode(0x8011);
        assert_eq!(0xFF, c.v[0]);
    }

    #[test]
    fn instruction_8xy2() {
        let mut c = Chip8::new();
        c.reset();
        c.v[1] = 0x0F;
        c.v[0] = 0xF0;
        c.process_opcode(0x8012);
        assert_eq!(0x00, c.v[0]);
    }

    #[test]
    fn instruction_8xy3() {
        let mut c = Chip8::new();
        c.reset();
        c.v[1] = 0x0F;
        c.v[0] = 0xF0;
        c.process_opcode(0x8013);
        assert_eq!(0xFF, c.v[0]);
    }

    #[test]
    fn instruction_8xy4() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 0xFF;
        c.v[1] = 0x02;
        c.process_opcode(0x8014);
        assert_eq!(0x01, c.v[0]);
        assert_eq!(0x01, c.v[0xF]); // Carry
        c.process_opcode(0x8014);
        assert_eq!(0x03, c.v[0]);
        assert_eq!(0x00, c.v[0xF]); // Carry
    }

    #[test]
    fn instruction_8xy5() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 0x01;
        c.v[1] = 0x02;
        c.process_opcode(0x8015);
        assert_eq!(0xFF, c.v[0]);
        assert_eq!(0x00, c.v[0xF]); // Carry
        c.process_opcode(0x8015);
        assert_eq!(0xFD, c.v[0]);
        assert_eq!(0x01, c.v[0xF]); // No carry
    }

    #[test]
    fn instruction_8xy6() {
        let mut c = Chip8::new();
        c.reset();
        c.v[1] = 0xFF;
        c.process_opcode(0x8016);
        assert_eq!(127, c.v[0]);
        assert_eq!(255, c.v[1]);
        assert_eq!(0x00, c.v[0xF]);

        c.v[0] = 0x03;
        c.v[1] = 0x02;
        c.process_opcode(0x8016);
        assert_eq!(0x01, c.v[0]);
        assert_eq!(0x01, c.v[0xF]);
    }

    #[test]
    fn instruction_8xy7() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 0x01;
        c.v[1] = 0x00;

        c.process_opcode(0x8017);
        assert_eq!(0xFF, c.v[0]);
        assert_eq!(0x00, c.v[0xF]);

        c.v[0] = 0x01;
        c.v[1] = 0x02;
        c.process_opcode(0x8017);
        assert_eq!(0x01, c.v[0]);
        assert_eq!(0x01, c.v[0xF]);

    }

    #[test]
    fn instruction_8xye() {
        let mut c = Chip8::new();
        c.reset();

        c.v[0] = 255;
        c.v[1] = 64;
        c.process_opcode(0x801E);
        assert_eq!(128, c.v[0]);
        assert_eq!(0x01, c.v[0xF]);
    }

    #[test]
    fn instruction_9xy0() {
        let mut c = Chip8::new();
        c.reset();

        c.v[0] = 0xFF;
        c.v[1] = 0xFF;
        c.process_opcode(0x9010);
        assert_eq!(0x202, c.pc);

        c.v[1] = 0xFE;
        c.process_opcode(0x9010);
        assert_eq!(0x206, c.pc);
    }

    #[test]
    fn instruction_annn() {
        let mut c = Chip8::new();
        c.reset();

        assert_eq!(0x000, c.i);
        c.process_opcode(0xafff);
        assert_eq!(0xfff, c.i);
    }

    #[test]
    fn instruction_bnnn() {
        let mut c = Chip8::new();
        c.reset();

        c.v[0] = 0x0E;
        c.process_opcode(0xb000);
        assert_eq!(0x00E, c.pc);
    }

    #[test]
    fn instruction_cxyn() {
        let mut c = Chip8::new();
        c.reset();

        c.process_opcode(0xc000); // Should always produce zero.
        assert_eq!(0x00, c.v[0]);
        
        c.process_opcode(0xc0FF);
        let valid_range = 0..255;
        assert!(valid_range.contains(&c.v[0]));
    }
// DXYN 	Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I
// Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
    #[test]
    fn instruction_dxyn() {
        assert_eq!(1, 2);
    }

    #[test]
    fn instruction_ex9e() {
        let mut c = Chip8::new();
        c.reset();
        c.io.keys[0] = true;
        c.process_opcode(0xE09E);
        assert_eq!(0x204, c.pc); // Jump
        c.io.keys[0] = false;
        c.process_opcode(0xE09E);
        assert_eq!(0x206, c.pc); // No jump
    }

    #[test]
    fn instruction_exa1() {
        let mut c = Chip8::new();
        c.reset();
        c.io.keys[0] = true;
        c.process_opcode(0xE0A1);
        assert_eq!(0x202, c.pc); // No jump
        c.io.keys[0] = false;
        c.process_opcode(0xE0A1);
        assert_eq!(0x206, c.pc); // Jump
    }

    #[test]
    fn instruction_fx07() {
        let mut c = Chip8::new();
        c.reset();
        c.dt = 0xFF;
        c.process_opcode(0xF007);
        assert_eq!(0xFF - 1, c.v[0]); // Already ticked once
    }
// FX0A 	Wait for a keypress and store the result in register VX
    #[test]
    fn instruction_fx0a() {
        assert_eq!(1, 2);
    }
    #[test]
    fn instruction_fx15() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 0xFF;

        c.process_opcode(0xf015);

        assert_eq!(0xFF, c.dt);
    }

    #[test]
    fn instruction_fx18() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 0xFF;

        c.process_opcode(0xf018);

        assert_eq!(0xFF, c.st);
    }

    #[test]
    fn instruction_fx1e() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 15;
        c.i = 15;

        c.process_opcode(0xf01e);

        assert_eq!(30, c.i);
    }

    #[test]
    fn instruction_fx29() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 0xF;
        c.process_opcode(0xf029);

        assert_eq!(0xF * 5, c.i);
    }

    #[test]
    fn instruction_fx33() {
        let mut c = Chip8::new();
        c.reset();
        c.v[0] = 0xFF;
        c.process_opcode(0xf033);

        let index = c.i as usize;
        assert_eq!(2, c.memory[index]);
        assert_eq!(5, c.memory[index + 1]);
        assert_eq!(5, c.memory[index + 2]);
    }

    #[test]
    fn instruction_fx55() {
        let mut c = Chip8::new();
        c.reset();

        let base_address = 0x200;
        c.i = base_address;

        for i in 0..=0xF {
            c.v[i] = 0xEE;
        }

        c.process_opcode(0xfF55);

        for i in 0..=0xF {
            assert_eq!(0xEE, c.memory[(base_address + i) as usize]);
        }

        assert_eq!(base_address + 0xF + 1, c.i);
    }

    #[test]
    fn instruction_fx65() {
        let mut c = Chip8::new();
        c.reset();

        let base_address = 0x200;
        c.i = base_address;

        for i in 0..=0xF {
            c.memory[(base_address + i) as usize] = 0xFF;
        }

        c.process_opcode(0xfF65);

        for i in 0..=0xF {
            assert_eq!(0xFF, c.v[i]);
        }

        assert_eq!(base_address + 0xF + 1, c.i);
    }
}
