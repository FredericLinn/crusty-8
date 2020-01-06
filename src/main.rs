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
    fb: [u8; 64 * 32],
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
            fb: [0; 64 * 32],
            keys: [false; 16],
        }
    }

    fn reset(&mut self) {
        self.pc = 0x200;
        self.i = 0;
        self.memory = [0; 4096];
        self.v =  [0; 16];
        self.stack = [0; 16];
        self.sp =  0;
        self.dt =  0;
        self.st = 0;
        self.fb = [0; 64 * 32];
        self.keys = [false; 16];
    }

    fn execute_cycle(&mut self) {
        let opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[(self.pc + 1) as usize] as u16);
        self.process_instruction(opcode);
    }

    fn process_instruction(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let vx = self.v[x];
        let vy = self.v[y];
        let nnn = opcode & 0x0FFF;
        let nn = (opcode & 0x00FF) as u8;
        let nibbles = split_opcode(opcode);

        match nibbles {
            // 0NNN Execute machine language subroutine at address NNN
            //(0x0, _, _, _) => (),

            // 00E0 Clear the screen
            (0x0, 0x0, _, 0x0)=> (),
            
            // 00EE Return from a subroutine
            (0x0, 0x0, _, _) => (),
            
            // 1NNN Jump to address NNN
            (0x1, _, _, _) => (),
           
            // 2NNN Execute subroutine starting at address NNN
            (0x2, _, _, _) => (),

            // 3XNN Skip the following instruction if the value of register VX equals NN
            (0x3, _, _, _) => (),

            // 4XNN Skip the following instruction if the value of register VX is not equal to NN
            (0x4, _, _, _) => (),

            // 5XY0 Skip the following instruction if the value of register VX is equal to the value of register VY
            (0x5, _, _, 0x0) => (),

            // 6XNN Store number NN in register VX
            (0x6, _, _, _) => (),

            // 7XNN Add the value NN to register VX
            (0x7, _, _, _) => (),

            // 8XY0 Store the value of register VY in register VX
            (0x8, _, _, 0x0) => (),

            // 8XY1 Set VX to VX OR VY
            (0x8, _, _, 0x1) => (),

            // 8XY2 Set VX to VX AND VY
            (0x8, _, _, 0x2) => (),

            // 8XY3 Set VX to VX XOR VY
            (0x8, _, _, 0x3) => (),
           
            // 8XY4 Add the value of register VY to register VX
            // Set VF to 01 if a carry occurs
            // Set VF to 00 if a carry does not occur
            (0x8, _, _, 0x4) => (),

            // 8XY5 Subtract the value of register VY from register VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            (0x8, _, _, 0x5) => (),

            // 8XY6 Store the value of register VY shifted right one bit in register VX
            // Set register VF to the least significant bit prior to the shift
            (0x8, _, _, 0x6) => (),

            // 8XY7 Set register VX to the value of VY minus VX
            // Set VF to 00 if a borrow occurs
            // Set VF to 01 if a borrow does not occur
            (0x8, _, _, 0x7) => (),

            // 8XYE Store the value of register VY shifted left one bit in register VX
            // Set register VF to the most significant bit prior to the shift
            (0x8, _, _, 0xE) => (),

            // 9XY0 Skip the following instruction if the value of register VX is not equal to the value of register VY
            (0x9, _, _, 0x0) => (),

            // ANNN Store memory address NNN in register I
            (0xA, _, _, _) => (),

            // BNNN Jump to address NNN + V0
            (0xB, _, _, _) => (),

            // CXNN Set VX to a random number with a mask of NN
            (0xC, _, _, _) => (),

            // DXYN Draw a sprite at position VX, VY with N bytes of sprite data starting at the address stored in I
            // Set VF to 01 if any set pixels are changed to unset, and 00 otherwise
            (0xD, _, _, _) => (),

            // EX9E Skip the following instruction if the key corresponding to the hex value currently stored in register VX is pressed
            (0xE, _, _, 0xE) => (),

            // EXA1 Skip the following instruction if the key corresponding to the hex value currently stored in register VX is not pressed
            (0xE, _, 0xA, 0x1) => (),

            // FX07 Store the current value of the delay timer in register VX
            (0xF, _, 0x0, 0x7) => (),

            // FX0A Wait for a keypress and store the result in register VX
            (0xF, _, 0x0, 0xA) => (),

            // FX15 Set the delay timer to the value of register VX
            (0xF, _, 0x1, 0x5) => (),

            // FX18 Set the sound timer to the value of register VX
            (0xF, _, 0x1, 0x8) => (),

            // FX1E Add the value stored in register VX to register I
            (0xF, _, 0x1, 0xE) => (),

            // FX29 Set I to the memory address of the sprite data corresponding to the hexadecimal digit stored in register VX
            (0xF, _, 0x2, 0x9) => (),

            // FX33 Store the binary-coded decimal equivalent of the value stored in register VX at addresses I, I+1, and I+2
            (0xF, _, 0x3, 0x3) => (),

            // FX55 Store the values of registers V0 to VX inclusive in memory starting at address I
            // I is set to I + X + 1 after operation
            (0xF, _, 0x5, 0x5) => (),

            // FX65 Fill registers V0 to VX inclusive with the values stored in memory starting at address I
            // I is set to I + X + 1 after operation
            (0xF, _, 0x6, 0x5) => (),

            // Gotta catch 'em all!
            (_, _, _, _) => (),
        }

    }
}

fn main() {
    println!("Hello, world!");
    let mut cpu = Cpu::new();
    cpu.reset();
    println!("{:?}", cpu.pc);
}
