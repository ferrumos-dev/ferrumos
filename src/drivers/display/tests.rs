use super::*;
use core::fmt::Write;

fn info(width: usize, height: usize, stride: usize, format: PixelFormat) -> FrameBufferInfo {
    let bytes_per_pixel = if matches!(format, PixelFormat::U8) {
        1
    } else {
        4
    };
    FrameBufferInfo {
        byte_len: stride * height * bytes_per_pixel,
        width,
        height,
        pixel_format: format,
        bytes_per_pixel,
        stride,
    }
}

#[test]
fn colors_match_rgb_bgr_and_grayscale_formats() {
    for (format, expected) in [
        (PixelFormat::Rgb, BACKGROUND),
        (
            PixelFormat::Bgr,
            [BACKGROUND[2], BACKGROUND[1], BACKGROUND[0]],
        ),
    ] {
        let mut buffer = [0xaa; CELL_WIDTH * CELL_HEIGHT * 4];
        let frame = info(CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH, format);
        let _console = TextConsole::new(&mut buffer, frame).unwrap();
        assert_eq!(&buffer[..3], &expected);
        assert_eq!(buffer[3], 0xaa);
    }

    let mut buffer = [0xff; CELL_WIDTH * CELL_HEIGHT];
    let frame = info(CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH, PixelFormat::U8);
    let _console = TextConsole::new(&mut buffer, frame).unwrap();
    let [red, green, blue] = BACKGROUND;
    let intensity =
        ((u16::from(red) * 77 + u16::from(green) * 150 + u16::from(blue) * 29) >> 8) as u8;
    assert!(buffer.iter().all(|&byte| byte == intensity));
}

#[test]
fn drawing_and_scrolling_preserve_padding_reserved_bytes_and_canaries() {
    const WIDTH: usize = CELL_WIDTH * 2 + 1;
    const STRIDE: usize = WIDTH + 3;
    const HEIGHT: usize = CELL_HEIGHT * 2 + 1;
    const LENGTH: usize = STRIDE * HEIGHT * 4;
    let mut storage = [0xa5; LENGTH + 8];
    let frame = info(WIDTH, HEIGHT, STRIDE, PixelFormat::Bgr);
    {
        let mut console = TextConsole::new(&mut storage[4..], frame).unwrap();
        write!(console, "ABCDE\nFG\nHI\tJK\u{8}").unwrap();
    }
    assert_eq!(&storage[..4], &[0xa5; 4]);
    assert_eq!(&storage[LENGTH + 4..], &[0xa5; 4]);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            assert_eq!(storage[4 + (y * STRIDE + x) * 4 + 3], 0xa5);
        }
        let padding_start = 4 + (y * STRIDE + WIDTH) * 4;
        let padding_end = 4 + (y + 1) * STRIDE * 4;
        assert!(storage[padding_start..padding_end]
            .iter()
            .all(|&byte| byte == 0xa5));
    }
}

#[test]
fn scroll_moves_existing_glyphs_one_text_row() {
    const WIDTH: usize = CELL_WIDTH * 2;
    const HEIGHT: usize = CELL_HEIGHT * 2;
    let mut buffer = [0; WIDTH * HEIGHT];
    let frame = info(WIDTH, HEIGHT, WIDTH, PixelFormat::U8);
    let mut console = TextConsole::new(&mut buffer, frame).unwrap();
    console.write_str("A\nB").unwrap();
    let mut previous_bottom_row = [0; WIDTH * CELL_HEIGHT];
    previous_bottom_row.copy_from_slice(&console.buffer[WIDTH * CELL_HEIGHT..]);
    console.write_char('\n');
    assert_eq!(&console.buffer[..WIDTH * CELL_HEIGHT], &previous_bottom_row);
    let background = Encoding::Grayscale.color(BACKGROUND)[0];
    assert!(console.buffer[WIDTH * CELL_HEIGHT..]
        .iter()
        .all(|&byte| byte == background));
    assert_eq!((console.column, console.row), (0, 1));
}

#[test]
fn backspace_erases_a_cell_and_crosses_an_automatic_wrap() {
    const WIDTH: usize = CELL_WIDTH * 2;
    let mut buffer = [0; WIDTH * CELL_HEIGHT * 2];
    let frame = info(WIDTH, CELL_HEIGHT * 2, WIDTH, PixelFormat::U8);
    let mut console = TextConsole::new(&mut buffer, frame).unwrap();
    console.write_str("ABC\u{8}\u{8}").unwrap();
    assert_eq!((console.column, console.row), (1, 0));
    let background = Encoding::Grayscale.color(BACKGROUND)[0];
    for y in 0..CELL_HEIGHT {
        assert!(console.buffer[y * WIDTH + CELL_WIDTH..(y + 1) * WIDTH]
            .iter()
            .all(|&byte| byte == background));
    }
    assert!(console.buffer[WIDTH * CELL_HEIGHT..]
        .iter()
        .all(|&byte| byte == background));
}

#[test]
fn one_cell_console_wraps_lazily_and_backspace_at_origin_is_safe() {
    let mut buffer = [0; CELL_WIDTH * CELL_HEIGHT];
    let frame = info(CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH, PixelFormat::U8);
    let mut console = TextConsole::new(&mut buffer, frame).unwrap();
    console.write_char('\u{8}');
    console.write_char('A');
    assert_eq!((console.column, console.row), (1, 0));
    let mut before = [0; CELL_WIDTH * CELL_HEIGHT];
    before.copy_from_slice(console.buffer);
    console.write_char('B');
    assert_ne!(console.buffer, &before);
    assert_eq!((console.column, console.row), (1, 0));
    console.write_char('\u{8}');
    assert_eq!((console.column, console.row), (0, 0));
}

#[test]
fn tab_carriage_return_clear_and_unsupported_glyph_have_defined_behavior() {
    const WIDTH: usize = CELL_WIDTH * 8;
    let mut buffer = [0; WIDTH * CELL_HEIGHT * 2];
    let frame = info(WIDTH, CELL_HEIGHT * 2, WIDTH, PixelFormat::U8);
    let mut console = TextConsole::new(&mut buffer, frame).unwrap();
    console.write_str("a\tb").unwrap();
    assert_eq!(console.column, 5);
    console.write_char('\r');
    assert_eq!((console.column, console.row), (0, 0));
    console.write_char('🦀');
    let mut fallback = [0; WIDTH * CELL_HEIGHT];
    fallback.copy_from_slice(&console.buffer[..WIDTH * CELL_HEIGHT]);
    console.write_char('\r');
    console.write_char('?');
    assert_eq!(&console.buffer[..WIDTH * CELL_HEIGHT], &fallback);
    console.clear();
    assert_eq!((console.column, console.row), (0, 0));
    assert_eq!((console.columns(), console.rows()), (8, 2));
}

#[test]
fn french_accented_characters_have_their_own_glyph() {
    let mut buffer = [0; CELL_WIDTH * CELL_HEIGHT];
    let frame = info(CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH, PixelFormat::U8);
    let mut console = TextConsole::new(&mut buffer, frame).unwrap();
    console.write_char('?');
    let mut replacement = [0; CELL_WIDTH * CELL_HEIGHT];
    replacement.copy_from_slice(console.buffer);
    for character in ['é', 'è', 'à', 'ç', 'ù'] {
        console.write_char('\r');
        console.write_char(character);
        assert_ne!(console.buffer, &replacement);
    }
}

#[test]
fn malformed_framebuffers_are_rejected_without_writes() {
    let valid = info(CELL_WIDTH, CELL_HEIGHT, CELL_WIDTH, PixelFormat::Rgb);
    let cases = [
        (
            FrameBufferInfo { width: 0, ..valid },
            DisplayError::InvalidDimensions,
        ),
        (
            FrameBufferInfo { height: 1, ..valid },
            DisplayError::InvalidDimensions,
        ),
        (
            FrameBufferInfo { stride: 1, ..valid },
            DisplayError::InvalidStride,
        ),
        (
            FrameBufferInfo {
                bytes_per_pixel: 2,
                ..valid
            },
            DisplayError::InvalidBytesPerPixel,
        ),
        (
            FrameBufferInfo {
                byte_len: valid.byte_len - 1,
                ..valid
            },
            DisplayError::BufferTooSmall,
        ),
        (
            FrameBufferInfo {
                byte_len: valid.byte_len + 1,
                ..valid
            },
            DisplayError::BufferTooSmall,
        ),
        (
            FrameBufferInfo {
                stride: usize::MAX,
                ..valid
            },
            DisplayError::GeometryOverflow,
        ),
        (
            FrameBufferInfo {
                pixel_format: PixelFormat::Unknown {
                    red_position: 0,
                    green_position: 1,
                    blue_position: 2,
                },
                ..valid
            },
            DisplayError::UnsupportedPixelFormat,
        ),
    ];
    for (frame, expected) in cases {
        let mut buffer = [0xc7; CELL_WIDTH * CELL_HEIGHT * 4];
        let actual = match TextConsole::new(&mut buffer, frame) {
            Ok(_) => panic!("invalid framebuffer was accepted"),
            Err(error) => error,
        };
        assert_eq!(actual, expected);
        assert!(buffer.iter().all(|&byte| byte == 0xc7));
    }
}
