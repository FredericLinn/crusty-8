use minifb::{Key, Scale, Window, WindowOptions};

pub struct Io {
    pub window: Window,
    pub framebuffer: [u32; WIDTH * HEIGHT],
    pub keys: [bool; 16],
}

impl Io {
    pub fn new() -> Io {
        Io {
            keys: [false; 16],
            framebuffer: [0x00_00_00_00; WIDTH * HEIGHT],
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

    pub fn draw(&mut self, display: &[bool]) {
        for (i, pixel) in self.framebuffer.iter_mut().enumerate() {
            *pixel = if display[i] { 0xFF_FF_FF_FF } else { 0x_00_00_00_00 };
        }

        self.window.update_with_buffer(&self.framebuffer, WIDTH, HEIGHT).unwrap();
    }

    pub fn set_keys(&mut self, keys: &mut [bool]) {
        for (i, key) in KEY_MAP.iter().enumerate() {
            if self.window.is_key_down(*key) {
                keys[i] = true;
            } else {
                keys[i] = false;
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
