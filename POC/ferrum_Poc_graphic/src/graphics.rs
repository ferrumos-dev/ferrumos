use font8x8::UnicodeFonts;

pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 200;
const VGA_BUFFER: *mut u8 = 0xa0000 as *mut u8;

pub mod palette {
    pub const BLACK: u8 = 0x00;
    pub const WHITE: u8 = 0x3f;
    pub const LIGHT: u8 = 0x3a;
    pub const MID: u8 = 0x37;
    pub const DARK: u8 = 0x08;
    pub const TITLE: u8 = 0x01;
    pub const WINDOW: u8 = 0x3b;
    pub const SHADOW: u8 = 0x08;
    pub const ACCENT: u8 = 0x0e;
    pub const SUCCESS: u8 = 0x0a;
    pub const ERROR_BG: u8 = 0x04;
    pub const TERMINAL_BG: u8 = 0x00;
    pub const TERMINAL_FG: u8 = 0x3f;
}

pub fn clear_screen(color: u8) {
    for index in 0..(WIDTH * HEIGHT) {
        unsafe {
            VGA_BUFFER.add(index).write_volatile(color);
        }
    }
}

pub fn put_pixel(x: usize, y: usize, color: u8) {
    if x >= WIDTH || y >= HEIGHT {
        return;
    }
    unsafe {
        VGA_BUFFER.add(y * WIDTH + x).write_volatile(color);
    }
}

pub fn fill_rect(x: usize, y: usize, width: usize, height: usize, color: u8) {
    let x_end = (x + width).min(WIDTH);
    let y_end = (y + height).min(HEIGHT);
    for py in y..y_end {
        for px in x..x_end {
            put_pixel(px, py, color);
        }
    }
}

pub fn draw_rect(x: usize, y: usize, width: usize, height: usize, color: u8) {
    if width == 0 || height == 0 {
        return;
    }
    for px in x..(x + width).min(WIDTH) {
        put_pixel(px, y, color);
        if height > 1 {
            put_pixel(px, (y + height - 1).min(HEIGHT - 1), color);
        }
    }
    for py in y..(y + height).min(HEIGHT) {
        put_pixel(x, py, color);
        if width > 1 {
            put_pixel((x + width - 1).min(WIDTH - 1), py, color);
        }
    }
}

pub fn draw_shadow(x: usize, y: usize, width: usize, height: usize) {
    fill_rect(x + 3, y + height, width, 3, palette::SHADOW);
    fill_rect(x + width, y + 3, 3, height, palette::SHADOW);
}

pub fn draw_window(x: usize, y: usize, width: usize, height: usize, title: &str, active: bool) {
    draw_shadow(x, y, width, height);
    fill_rect(x, y, width, height, palette::WINDOW);
    draw_rect(x, y, width, height, palette::BLACK);
    fill_rect(x + 1, y + 1, width - 2, 12, if active { palette::TITLE } else { palette::MID });
    draw_text(x + 6, y + 3, title, palette::WHITE, 1);
    fill_rect(x + width - 26, y + 3, 6, 6, palette::LIGHT);
    fill_rect(x + width - 17, y + 3, 6, 6, palette::LIGHT);
    fill_rect(x + width - 8, y + 3, 6, 6, palette::ACCENT);
}

pub fn draw_text(x: usize, y: usize, text: &str, color: u8, scale: usize) {
    let mut cursor_x = x;
    for ch in text.chars() {
        draw_char(cursor_x, y, ch, color, scale);
        cursor_x += 8 * scale;
    }
}

pub fn draw_ascii_text(x: usize, y: usize, bytes: &[u8], len: usize, color: u8, scale: usize) {
    let mut cursor_x = x;
    let count = len.min(bytes.len());
    for byte in &bytes[..count] {
        let ch = if (0x20..=0x7e).contains(byte) {
            *byte as char
        } else {
            '?'
        };
        draw_char(cursor_x, y, ch, color, scale);
        cursor_x += 8 * scale;
    }
}

pub fn draw_char(x: usize, y: usize, ch: char, color: u8, scale: usize) {
    if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..8 {
                if bits & (1 << col) != 0 {
                    let px = x + col * scale;
                    let py = y + row * scale;
                    fill_rect(px, py, scale, scale, color);
                }
            }
        }
    }
}

pub fn draw_panel(x: usize, y: usize, width: usize, height: usize, bg: u8, border: u8) {
    if width < 4 || height < 4 {
        return;
    }

    fill_rect(x + 1, y, width - 2, height, bg);
    fill_rect(x, y + 1, width, height - 2, bg);
    draw_rect(x, y, width, height, border);

    // Soften corners for a less blocky look.
    put_pixel(x, y, bg);
    put_pixel(x + width - 1, y, bg);
    put_pixel(x, y + height - 1, bg);
    put_pixel(x + width - 1, y + height - 1, bg);
}

pub fn draw_mouse_cursor(x: usize, y: usize) {
    let points: &[(usize, usize)] = &[
        (0, 0),
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 4),
        (2, 5),
        (3, 6),
        (4, 7),
    ];

    for (dx, dy) in points {
        put_pixel(x + dx, y + dy, palette::WHITE);
    }
    for (dx, dy) in points {
        if *dx > 0 {
            put_pixel(x + dx - 1, y + dy, palette::BLACK);
        }
    }
}
