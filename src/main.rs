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
}

fn main() {
    println!("Hello, world!");
    let mut cpu = Cpu::new();
    cpu.reset();
}
