use minifb::{Key, Scale, Window, WindowOptions};

pub struct Io {
    pub framebuffer: [u32; WIDTH * HEIGHT],
    pub window: Window,
    keys: [bool; 16],
}

impl Io {
    pub fn new() -> Io {
        Io {
            keys: [false; 16],
            framebuffer: [0; WIDTH * HEIGHT],
            window: Window::new(
                "Crusty-8 (Press ESC to exit)",
                WIDTH,
                HEIGHT,
                WindowOptions {
                    resize: false,
                    scale: Scale::X16,
                    ..WindowOptions::default()
                },
            )
            .unwrap_or_else(|e| {
                panic!("{}", e);
            }),
        }
    }

    pub fn setup(&mut self) {
        self.window.limit_update_rate(Some(std::time::Duration::from_micros(1660))); // 600FPS
    }

    pub fn cls(&mut self) {
        self.framebuffer = [0; 64 * 32];
        self.draw();
    }

    pub fn set_pixel(&mut self, x: usize, y: usize) -> bool {
        let index = y * WIDTH + x;

        if index >= 64 * 32 { return false};

        if self.framebuffer[index] == 0xFFFF_FFFF {
            self.framebuffer[index] = 0;
            true
        } else {
            self.framebuffer[index] = 0xFFFF_FFFF;
            false
        }
    }

    pub fn draw(&mut self) {
        self.window.update_with_buffer(&self.framebuffer, WIDTH, HEIGHT).unwrap();
    }

    pub fn is_key_pressed(&self, index: u8) -> bool {
        self.keys[index as usize]
    }

    pub fn set_keys(&mut self) {
        for (i, key) in KEY_MAP.iter().enumerate() {
            if self.window.is_key_down(*key) {
                self.keys[i] = true;
            } else {
                self.keys[i] = false;
            }
        }
    }
}

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
#[allow(dead_code)]
const KEY_MAP: [Key; 16] = [
    Key::Key1,
    Key::Key2,
    Key::Key3,
    Key::Key4,
    Key::Q,
    Key::W,
    Key::E,
    Key::R,
    Key::A,
    Key::S,
    Key::D,
    Key::F,
    Key::Y,
    Key::X,
    Key::C,
    Key::V,
];
