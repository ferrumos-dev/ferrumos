use bootloader_api::info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use font8x8::UnicodeFonts;
use spin::Mutex;

#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const WHITE: Self = Self::new(255, 255, 255);
}

struct Renderer {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
}

impl Renderer {
    fn width(&self) -> usize {
        self.info.width
    }

    fn height(&self) -> usize {
        self.info.height
    }

    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }

        let idx = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        if idx + self.info.bytes_per_pixel > self.buffer.len() {
            return;
        }

        match self.info.pixel_format {
            PixelFormat::Rgb => {
                self.buffer[idx] = color.r;
                if self.info.bytes_per_pixel > 1 {
                    self.buffer[idx + 1] = color.g;
                }
                if self.info.bytes_per_pixel > 2 {
                    self.buffer[idx + 2] = color.b;
                }
            }
            PixelFormat::Bgr => {
                self.buffer[idx] = color.b;
                if self.info.bytes_per_pixel > 1 {
                    self.buffer[idx + 1] = color.g;
                }
                if self.info.bytes_per_pixel > 2 {
                    self.buffer[idx + 2] = color.r;
                }
            }
            PixelFormat::U8 => {
                let gray = ((color.r as u16 + color.g as u16 + color.b as u16) / 3) as u8;
                self.buffer[idx] = gray;
            }
            PixelFormat::Unknown { .. } => {
                self.buffer[idx] = color.r;
                if self.info.bytes_per_pixel > 1 {
                    self.buffer[idx + 1] = color.g;
                }
                if self.info.bytes_per_pixel > 2 {
                    self.buffer[idx + 2] = color.b;
                }
            }
            _ => {
                self.buffer[idx] = color.r;
                if self.info.bytes_per_pixel > 1 {
                    self.buffer[idx + 1] = color.g;
                }
                if self.info.bytes_per_pixel > 2 {
                    self.buffer[idx + 2] = color.b;
                }
            }
        }
    }
}

static RENDERER: Mutex<Option<Renderer>> = Mutex::new(None);

pub fn init(framebuffer: &'static mut FrameBuffer) {
    let info = framebuffer.info();
    let buffer = framebuffer.buffer_mut();
    let mut guard = RENDERER.lock();
    *guard = Some(Renderer { buffer, info });
}

pub fn dimensions() -> (usize, usize) {
    let guard = RENDERER.lock();
    if let Some(r) = guard.as_ref() {
        (r.width(), r.height())
    } else {
        (0, 0)
    }
}

pub fn clear(color: Color) {
    let mut guard = RENDERER.lock();
    let Some(r) = guard.as_mut() else { return; };

    for y in 0..r.height() {
        for x in 0..r.width() {
            r.set_pixel(x, y, color);
        }
    }
}

pub fn fill_rect(x: usize, y: usize, width: usize, height: usize, color: Color) {
    let mut guard = RENDERER.lock();
    let Some(r) = guard.as_mut() else { return; };

    let max_x = (x + width).min(r.width());
    let max_y = (y + height).min(r.height());

    for py in y..max_y {
        for px in x..max_x {
            r.set_pixel(px, py, color);
        }
    }
}

pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: Color) {
    if width < 2 || height < 2 {
        return;
    }

    fill_rect(x, y, width, 1, color);
    fill_rect(x, y + height - 1, width, 1, color);
    fill_rect(x, y, 1, height, color);
    fill_rect(x + width - 1, y, 1, height, color);
}

pub fn draw_panel(x: usize, y: usize, width: usize, height: usize, bg: Color, border: Color) {
    if width < 4 || height < 4 {
        return;
    }
    fill_rect(x + 1, y, width - 2, height, bg);
    fill_rect(x, y + 1, width, height - 2, bg);
    draw_rect(x, y, width, height, border);
}

pub fn draw_text(x: usize, y: usize, text: &str, color: Color, scale: usize) {
    let mut cursor_x = x;
    for ch in text.chars() {
        draw_char(cursor_x, y, ch, color, scale);
        cursor_x += 8 * scale;
    }
}

pub fn draw_ascii(x: usize, y: usize, bytes: &[u8], len: usize, color: Color, scale: usize) {
    let mut cursor_x = x;
    for b in &bytes[..len.min(bytes.len())] {
        let ch = if (0x20..=0x7e).contains(b) { *b as char } else { '?' };
        draw_char(cursor_x, y, ch, color, scale);
        cursor_x += 8 * scale;
    }
}

pub fn draw_char(x: usize, y: usize, ch: char, color: Color, scale: usize) {
    if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if bits & (1 << col) != 0 {
                    fill_rect(x + col * scale, y + row * scale, scale, scale, color);
                }
            }
        }
    }
}

pub fn draw_mouse_cursor(x: usize, y: usize) {
    let white = Color::new(255, 255, 255);
    let black = Color::new(0, 0, 0);
    let accent = Color::new(70, 160, 255);

    // Outer dark edge for contrast.
    for i in 0..20 {
        fill_rect(x, y + i, 1, 1, black);
    }
    for i in 0..11 {
        fill_rect(x + i, y + i + 2, 1, 1, black);
    }
    for i in 0..8 {
        fill_rect(x + 7, y + i + 10, 1, 1, black);
    }

    // Inner bright body.
    for i in 1..19 {
        fill_rect(x + 1, y + i, 1, 1, white);
    }
    for i in 1..10 {
        fill_rect(x + i, y + i + 2, 1, 1, white);
    }
    for i in 1..7 {
        fill_rect(x + 8, y + i + 10, 1, 1, white);
    }

    // Accent center pixel to make movement obvious.
    fill_rect(x + 3, y + 7, 2, 2, accent);
}
