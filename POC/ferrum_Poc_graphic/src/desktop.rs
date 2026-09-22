use crate::graphics;
use crate::graphics::palette;
use crate::input::{self, Key};
use crate::mouse;

const TERM_X: usize = 56;
const TERM_Y: usize = 34;
const TERM_W: usize = 208;
const TERM_H: usize = 124;

const TERM_CONTENT_X: usize = TERM_X + 8;
const TERM_CONTENT_Y: usize = TERM_Y + 22;
const TERM_CONTENT_W: usize = TERM_W - 16;
const TERM_CONTENT_H: usize = TERM_H - 34;

const SEARCH_X: usize = 72;
const SEARCH_Y: usize = 182;
const SEARCH_W: usize = 176;
const SEARCH_H: usize = 14;

const TERM_MAX_LINES: usize = 8;
const TERM_MAX_COLS: usize = 22;
const TERM_INPUT_MAX: usize = 20;
const SEARCH_MAX: usize = 16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Terminal,
    Search,
}

struct Terminal {
    lines: [[u8; TERM_MAX_COLS]; TERM_MAX_LINES],
    lens: [usize; TERM_MAX_LINES],
    line_count: usize,
    input: [u8; TERM_INPUT_MAX],
    input_len: usize,
}

impl Terminal {
    const fn new() -> Self {
        Self {
            lines: [[0; TERM_MAX_COLS]; TERM_MAX_LINES],
            lens: [0; TERM_MAX_LINES],
            line_count: 0,
            input: [0; TERM_INPUT_MAX],
            input_len: 0,
        }
    }

    fn seed(&mut self) {
        self.push_line(b"FerrumOS modern shell");
        self.push_line(b"Type: help");
    }

    fn push_line(&mut self, text: &[u8]) {
        if self.line_count == TERM_MAX_LINES {
            for i in 1..TERM_MAX_LINES {
                self.lines[i - 1] = self.lines[i];
                self.lens[i - 1] = self.lens[i];
            }
            self.line_count = TERM_MAX_LINES - 1;
        }

        let idx = self.line_count;
        let len = text.len().min(TERM_MAX_COLS);
        for i in 0..len {
            self.lines[idx][i] = text[i];
        }
        for i in len..TERM_MAX_COLS {
            self.lines[idx][i] = 0;
        }
        self.lens[idx] = len;
        self.line_count += 1;
    }

    fn add_char(&mut self, ch: char) {
        if !ch.is_ascii() {
            return;
        }
        let b = ch as u8;
        if !(0x20..=0x7e).contains(&b) {
            return;
        }
        if self.input_len < TERM_INPUT_MAX {
            self.input[self.input_len] = b;
            self.input_len += 1;
        }
    }

    fn backspace(&mut self) {
        if self.input_len == 0 {
            return;
        }
        self.input_len -= 1;
        self.input[self.input_len] = 0;
    }

    fn input_equals(&self, value: &[u8]) -> bool {
        if self.input_len != value.len() {
            return false;
        }
        for (idx, b) in value.iter().enumerate() {
            if self.input[idx] != *b {
                return false;
            }
        }
        true
    }

    fn clear_input(&mut self) {
        for i in 0..self.input_len {
            self.input[i] = 0;
        }
        self.input_len = 0;
    }

    fn submit(&mut self) {
        let mut prompt = [0u8; TERM_MAX_COLS];
        let mut len = 0;
        prompt[len] = b'>';
        len += 1;
        prompt[len] = b' ';
        len += 1;
        let cp = self.input_len.min(TERM_MAX_COLS - len);
        for i in 0..cp {
            prompt[len + i] = self.input[i];
        }
        self.push_line(&prompt[..(len + cp)]);

        if self.input_len == 0 {
            self.clear_input();
            return;
        }

        if self.input_equals(b"help") {
            self.push_line(b"help clear about time");
        } else if self.input_equals(b"clear") {
            self.line_count = 0;
        } else if self.input_equals(b"about") {
            self.push_line(b"Windows-11-like PoC");
        } else if self.input_equals(b"time") {
            self.push_line(b"No RTC wired yet");
        } else {
            self.push_line(b"Command not found");
        }

        self.clear_input();
    }
}

pub fn run() -> ! {
    let mut terminal = Terminal::new();
    terminal.seed();

    let mut focus = Focus::Terminal;
    let mut search = [0u8; SEARCH_MAX];
    let mut search_len = 0usize;
    let mut frame: u32 = 0;

    mouse::init();
    let mut mstate = mouse::current();

    draw_desktop(&terminal, focus, &search, search_len, mstate, frame);

    loop {
        frame = frame.wrapping_add(1);
        let mut redraw = frame % 45 == 0;

        if let Some(ms) = mouse::poll() {
            mstate = ms;
            redraw = true;

            if ms.left_clicked {
                if in_rect(ms.x, ms.y, SEARCH_X, SEARCH_Y, SEARCH_W, SEARCH_H) {
                    focus = Focus::Search;
                } else if in_rect(ms.x, ms.y, TERM_X, TERM_Y, TERM_W, TERM_H) {
                    focus = Focus::Terminal;
                }
            }
        }

        if let Some(key) = input::poll_key() {
            match key {
                Key::Tab => {
                    focus = match focus {
                        Focus::Terminal => Focus::Search,
                        Focus::Search => Focus::Terminal,
                    };
                    redraw = true;
                }
                Key::Enter => {
                    if focus == Focus::Terminal {
                        terminal.submit();
                        redraw = true;
                    }
                }
                Key::Backspace => {
                    if focus == Focus::Terminal {
                        terminal.backspace();
                    } else if focus == Focus::Search && search_len > 0 {
                        search_len -= 1;
                        search[search_len] = 0;
                    }
                    redraw = true;
                }
                Key::Escape => {
                    focus = Focus::Terminal;
                    redraw = true;
                }
                Key::Char(ch) => {
                    if focus == Focus::Terminal {
                        terminal.add_char(ch);
                    } else if focus == Focus::Search {
                        let b = ch as u8;
                        if ch.is_ascii() && (0x20..=0x7e).contains(&b) && search_len < SEARCH_MAX {
                            search[search_len] = b;
                            search_len += 1;
                        }
                    }
                    redraw = true;
                }
            }
        }

        if redraw {
            draw_desktop(&terminal, focus, &search, search_len, mstate, frame);
        } else {
            core::hint::spin_loop();
        }
    }
}

pub fn draw_panic_screen() {
    graphics::draw_window(36, 52, 230, 72, "Kernel Panic", true);
    graphics::draw_text(56, 76, "FerrumOS Graphic PoC stopped.", palette::BLACK, 1);
    graphics::draw_text(56, 92, "Check the kernel state in debug mode.", palette::BLACK, 1);
}

fn draw_desktop(
    terminal: &Terminal,
    focus: Focus,
    search: &[u8; SEARCH_MAX],
    search_len: usize,
    mouse_state: mouse::MouseState,
    frame: u32,
) {
    draw_wallpaper(frame);
    draw_top_strip();
    draw_terminal_window(terminal, focus == Focus::Terminal, frame);
    draw_bottom_dock(focus, search, search_len, frame);
    graphics::draw_mouse_cursor(mouse_state.x, mouse_state.y);
}

fn draw_wallpaper(frame: u32) {
    let band = ((frame / 8) % 3) as usize;
    for y in 0..graphics::HEIGHT {
        let color = match ((y / 20) + band) % 3 {
            0 => 0x18,
            1 => 0x19,
            _ => 0x1a,
        };
        graphics::fill_rect(0, y, graphics::WIDTH, 1, color);
    }

    graphics::fill_rect(14, 22, 40, 40, 0x1f);
    graphics::fill_rect(250, 52, 52, 52, 0x1e);
}

fn draw_top_strip() {
    graphics::draw_panel(0, 0, graphics::WIDTH, 16, 0x3c, palette::BLACK);
    graphics::draw_text(8, 4, "FerrumOS Desktop", palette::BLACK, 1);
    graphics::draw_text(212, 4, "Modern Shell", palette::DARK, 1);
}

fn draw_terminal_window(terminal: &Terminal, active: bool, frame: u32) {
    graphics::draw_panel(TERM_X, TERM_Y, TERM_W, TERM_H, 0x38, palette::BLACK);
    graphics::fill_rect(TERM_X + 1, TERM_Y + 1, TERM_W - 2, 14, if active { 0x21 } else { 0x37 });
    graphics::draw_text(TERM_X + 8, TERM_Y + 4, "Terminal", palette::WHITE, 1);
    graphics::fill_rect(TERM_X + TERM_W - 16, TERM_Y + 4, 8, 8, 0x0c);

    graphics::draw_panel(
        TERM_CONTENT_X,
        TERM_CONTENT_Y,
        TERM_CONTENT_W,
        TERM_CONTENT_H,
        palette::TERMINAL_BG,
        0x20,
    );

    for i in 0..terminal.line_count {
        let y = TERM_CONTENT_Y + 6 + i * 10;
        graphics::draw_ascii_text(
            TERM_CONTENT_X + 4,
            y,
            &terminal.lines[i],
            terminal.lens[i],
            palette::TERMINAL_FG,
            1,
        );
    }

    let input_y = TERM_CONTENT_Y + TERM_CONTENT_H - 12;
    graphics::draw_text(TERM_CONTENT_X + 4, input_y, ">", palette::SUCCESS, 1);
    graphics::draw_ascii_text(
        TERM_CONTENT_X + 14,
        input_y,
        &terminal.input,
        terminal.input_len,
        palette::TERMINAL_FG,
        1,
    );

    if frame % 30 < 15 {
        let cursor_x = TERM_CONTENT_X + 14 + terminal.input_len * 8;
        graphics::fill_rect(cursor_x, input_y + 8, 6, 1, 0x2a);
    }
}

fn draw_bottom_dock(focus: Focus, search: &[u8; SEARCH_MAX], search_len: usize, frame: u32) {
    graphics::draw_panel(0, 178, graphics::WIDTH, 22, 0x39, palette::BLACK);

    let orb = if frame % 40 < 20 { 0x21 } else { 0x20 };
    graphics::fill_rect(10, 183, 12, 12, orb);
    graphics::draw_text(27, 186, "Start", palette::BLACK, 1);

    let active = focus == Focus::Search;
    let sbg = if active { 0x3d } else { 0x3b };
    graphics::draw_panel(SEARCH_X, SEARCH_Y, SEARCH_W, SEARCH_H, sbg, 0x35);
    graphics::draw_text(SEARCH_X + 6, SEARCH_Y + 4, "Search", 0x22, 1);

    if search_len > 0 {
        graphics::draw_ascii_text(SEARCH_X + 62, SEARCH_Y + 4, search, search_len, palette::BLACK, 1);
    }

    graphics::draw_text(254, 186, "EN", palette::BLACK, 1);
    graphics::draw_text(274, 186, "12:00", palette::BLACK, 1);
}

fn in_rect(x: usize, y: usize, rx: usize, ry: usize, rw: usize, rh: usize) -> bool {
    x >= rx && x < rx + rw && y >= ry && y < ry + rh
}
