//! Allocation-free text output on the bootloader's linear framebuffer.
//!
//! A console owns its mutable framebuffer borrow; there is no global writer or
//! implicit synchronization. The caller decides when to pass it to other code.
//! Color components are accessed through volatile operations so drawing and
//! scrolling remain visible to the display device. The bootloader establishes
//! the framebuffer mapping and its memory attributes.
//! Shell layout and command handling belong in [`crate::ui`].

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::fmt;
use font8x8::{UnicodeFonts, BASIC_FONTS, LATIN_FONTS};

/// Width of one text cell, in framebuffer pixels.
pub const CELL_WIDTH: usize = 8;
/// Height of one text cell, with every font row drawn twice for legibility.
pub const CELL_HEIGHT: usize = 16;

const FOREGROUND: [u8; 3] = [0xe6, 0xed, 0xf3];
const BACKGROUND: [u8; 3] = [0x0d, 0x11, 0x17];
const TAB_WIDTH: usize = 4;

/// A framebuffer description that cannot safely support this text console.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayError {
    /// The framebuffer cannot contain even one complete text cell.
    InvalidDimensions,
    /// The physical row stride is shorter than the visible row.
    InvalidStride,
    /// The pixel format is not RGB, BGR or eight-bit grayscale.
    UnsupportedPixelFormat,
    /// The pixel byte count does not match a supported representation.
    InvalidBytesPerPixel,
    /// The declared or supplied storage is shorter than the framebuffer.
    BufferTooSmall,
    /// Computing a byte offset would overflow the address-sized integer.
    GeometryOverflow,
}

impl fmt::Display for DisplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidDimensions => "framebuffer is smaller than one text cell",
            Self::InvalidStride => "framebuffer stride is shorter than its width",
            Self::UnsupportedPixelFormat => "unsupported framebuffer pixel format",
            Self::InvalidBytesPerPixel => "unsupported framebuffer pixel byte count",
            Self::BufferTooSmall => "framebuffer storage is too short",
            Self::GeometryOverflow => "framebuffer dimensions overflow byte offsets",
        })
    }
}

#[derive(Clone, Copy)]
enum Encoding {
    Rgb,
    Bgr,
    Grayscale,
}

impl Encoding {
    fn channels(self) -> usize {
        match self {
            Self::Rgb | Self::Bgr => 3,
            Self::Grayscale => 1,
        }
    }

    fn color(self, [red, green, blue]: [u8; 3]) -> [u8; 3] {
        match self {
            Self::Rgb => [red, green, blue],
            Self::Bgr => [blue, green, red],
            Self::Grayscale => {
                let intensity =
                    (u16::from(red) * 77 + u16::from(green) * 150 + u16::from(blue) * 29) >> 8;
                [intensity as u8, 0, 0]
            }
        }
    }
}

/// A borrowed framebuffer console with fixed-size cells and scrolling.
///
/// RGB/BGR use three color bytes per pixel, with an optional fourth byte left
/// untouched. Grayscale uses one byte. Row padding and storage beyond the
/// declared framebuffer length are never written. Basic Latin and Latin-1
/// characters use an embedded bitmap font; unsupported glyphs appear as `?`.
///
/// Ordinary character output wraps lazily: filling the last column does not
/// scroll until the next character or newline. Backspace erases the previous
/// cell in the physical text grid, including across a wrapped row; it is a
/// no-op at the top left. A line editor must enforce its own input boundary.
pub struct TextConsole<'a> {
    buffer: &'a mut [u8],
    info: FrameBufferInfo,
    encoding: Encoding,
    columns: usize,
    rows: usize,
    column: usize,
    row: usize,
}

impl<'a> TextConsole<'a> {
    /// Validates the framebuffer and clears its visible pixels.
    ///
    /// The constructor checks all dimensions before modifying the buffer.
    /// A failed construction leaves every supplied byte unchanged.
    pub fn new(buffer: &'a mut [u8], info: FrameBufferInfo) -> Result<Self, DisplayError> {
        if info.width < CELL_WIDTH || info.height < CELL_HEIGHT {
            return Err(DisplayError::InvalidDimensions);
        }
        if info.stride < info.width {
            return Err(DisplayError::InvalidStride);
        }
        let encoding = match info.pixel_format {
            PixelFormat::Rgb => Encoding::Rgb,
            PixelFormat::Bgr => Encoding::Bgr,
            PixelFormat::U8 => Encoding::Grayscale,
            _ => return Err(DisplayError::UnsupportedPixelFormat),
        };
        match encoding {
            Encoding::Rgb | Encoding::Bgr if !matches!(info.bytes_per_pixel, 3 | 4) => {
                return Err(DisplayError::InvalidBytesPerPixel);
            }
            Encoding::Grayscale if info.bytes_per_pixel != 1 => {
                return Err(DisplayError::InvalidBytesPerPixel);
            }
            _ => {}
        }
        let required_bytes = info
            .stride
            .checked_mul(info.height)
            .and_then(|pixels| pixels.checked_mul(info.bytes_per_pixel))
            .ok_or(DisplayError::GeometryOverflow)?;
        if required_bytes > info.byte_len || info.byte_len > buffer.len() {
            return Err(DisplayError::BufferTooSmall);
        }

        let mut console = Self {
            buffer: &mut buffer[..info.byte_len],
            info,
            encoding,
            columns: info.width / CELL_WIDTH,
            rows: info.height / CELL_HEIGHT,
            column: 0,
            row: 0,
        };
        console.clear();
        Ok(console)
    }

    /// Returns the number of complete text columns.
    pub const fn columns(&self) -> usize {
        self.columns
    }

    /// Returns the number of complete text rows.
    pub const fn rows(&self) -> usize {
        self.rows
    }

    /// Clears visible pixels and moves the text position to the top left.
    pub fn clear(&mut self) {
        self.fill_rows(0, self.info.height);
        self.column = 0;
        self.row = 0;
    }

    /// Writes a character, applying newline, carriage return, tab or backspace.
    ///
    /// Tabs insert spaces up to the next four-column stop. Other control
    /// characters are ignored. Unsupported printable glyphs render as `?`.
    pub fn write_char(&mut self, character: char) {
        match character {
            '\n' => self.new_line(),
            '\r' => self.column = 0,
            '\u{8}' => self.backspace(),
            '\t' => {
                let spaces = TAB_WIDTH - (self.column % TAB_WIDTH);
                for _ in 0..spaces {
                    self.write_char(' ');
                }
            }
            character if character.is_control() => {}
            character => {
                if self.column == self.columns {
                    self.new_line();
                }
                self.draw_cell(character, self.column, self.row);
                self.column += 1;
            }
        }
    }

    fn new_line(&mut self) {
        self.column = 0;
        if self.row + 1 < self.rows {
            self.row += 1;
        } else {
            self.scroll();
        }
    }

    fn backspace(&mut self) {
        if self.column > 0 {
            self.column -= 1;
        } else if self.row > 0 {
            self.row -= 1;
            self.column = self.columns - 1;
        } else {
            return;
        }
        self.draw_cell(' ', self.column, self.row);
    }

    fn draw_cell(&mut self, character: char, column: usize, row: usize) {
        // '?' is part of BASIC_FONTS; the final bitmap also makes the fallback
        // independent of that assumption without requiring an unwrap.
        let glyph = BASIC_FONTS
            .get(character)
            .or_else(|| LATIN_FONTS.get(character))
            .or_else(|| BASIC_FONTS.get('?'))
            .unwrap_or([0x7e, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7e, 0]);
        for (glyph_y, bits) in glyph.into_iter().enumerate() {
            for glyph_x in 0..CELL_WIDTH {
                let color = if bits & (1 << glyph_x) != 0 {
                    FOREGROUND
                } else {
                    BACKGROUND
                };
                let x = column * CELL_WIDTH + glyph_x;
                let y = row * CELL_HEIGHT + glyph_y * 2;
                self.write_pixel(x, y, color);
                self.write_pixel(x, y + 1, color);
            }
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, color: [u8; 3]) {
        let start = (y * self.info.stride + x) * self.info.bytes_per_pixel;
        let color = self.encoding.color(color);
        let count = self.encoding.channels();
        for (channel, value) in color[..count].iter().copied().enumerate() {
            self.write_framebuffer_byte(start + channel, value);
        }
    }

    fn read_framebuffer_byte(&self, index: usize) -> u8 {
        let byte = &self.buffer[index];
        // SAFETY: indexing bounds-checks the borrowed framebuffer before this
        // pointer is formed. A u8 needs no extra alignment and remains valid for
        // the entire read. Volatile preserves the device-visible memory access.
        unsafe { core::ptr::read_volatile(byte) }
    }

    fn write_framebuffer_byte(&mut self, index: usize, value: u8) {
        let byte = &mut self.buffer[index];
        // SAFETY: indexing bounds-checks the exclusive framebuffer borrow. The
        // pointer is valid, aligned for u8 and used only during this borrow.
        // Volatile prevents the compiler from eliding device-visible writes.
        unsafe { core::ptr::write_volatile(byte, value) };
    }

    fn fill_rows(&mut self, start_y: usize, end_y: usize) {
        for y in start_y..end_y {
            for x in 0..self.info.width {
                self.write_pixel(x, y, BACKGROUND);
            }
        }
    }

    fn scroll(&mut self) {
        let text_height = self.rows * CELL_HEIGHT;
        let channels = self.encoding.channels();
        // Copy only visible color components, preserving destination padding
        // and each pixel's reserved byte instead of copying complete rows.
        for y in 0..text_height - CELL_HEIGHT {
            for x in 0..self.info.width {
                let source = ((y + CELL_HEIGHT) * self.info.stride + x) * self.info.bytes_per_pixel;
                let destination = (y * self.info.stride + x) * self.info.bytes_per_pixel;
                for channel in 0..channels {
                    let value = self.read_framebuffer_byte(source + channel);
                    self.write_framebuffer_byte(destination + channel, value);
                }
            }
        }
        self.fill_rows(text_height - CELL_HEIGHT, text_height);
    }
}

impl fmt::Write for TextConsole<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for character in text.chars() {
            self.write_char(character);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
