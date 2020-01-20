use minifb::{Key, Scale, Window, WindowOptions};

pub struct Io {
    pub window: Window,
    pub framebuffer:[bool; WIDTH * HEIGHT],
    pub keys: [bool; 16],
}

impl Io {
    pub fn new() -> Io {
        Io {
            keys: [false; 16],
            framebuffer: [false; WIDTH * HEIGHT],
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
        self.framebuffer = [false; WIDTH * HEIGHT];
    }

    pub fn draw(&mut self, buf: &[u32]) {
        self.window.update_with_buffer(buf, WIDTH, HEIGHT).unwrap();
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

pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

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
