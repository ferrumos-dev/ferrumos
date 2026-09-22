use crate::framebuffer::{self, Color};
use crate::input::{self, Key};
use crate::mouse;

const ICON_X: usize = 36;
const ICON_Y: usize = 40;
const ICON_W: usize = 110;
const ICON_H: usize = 122;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Desktop,
    Terminal,
    Search,
}

struct Terminal {
    lines: [[u8; 48]; 18],
    lens: [usize; 18],
    line_count: usize,
    input: [u8; 44],
    input_len: usize,
}

impl Terminal {
    const fn new() -> Self {
        Self {
            lines: [[0; 48]; 18],
            lens: [0; 18],
            line_count: 0,
            input: [0; 44],
            input_len: 0,
        }
    }

    fn seed(&mut self) {
        self.push_line(b"FerrumOS Terminal");
        self.push_line(b"type help for commands");
    }

    fn push_line(&mut self, text: &[u8]) {
        if self.line_count == self.lines.len() {
            for i in 1..self.lines.len() {
                self.lines[i - 1] = self.lines[i];
                self.lens[i - 1] = self.lens[i];
            }
            self.line_count = self.lines.len() - 1;
        }

        let idx = self.line_count;
        let len = text.len().min(self.lines[idx].len());
        for i in 0..len {
            self.lines[idx][i] = text[i];
        }
        for i in len..self.lines[idx].len() {
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
        if self.input_len < self.input.len() {
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

    fn equals(&self, value: &[u8]) -> bool {
        if self.input_len != value.len() {
            return false;
        }
        for (i, b) in value.iter().enumerate() {
            if self.input[i] != *b {
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
        let mut line = [0u8; 48];
        line[0] = b'>';
        line[1] = b' ';
        let cp = self.input_len.min(46);
        for i in 0..cp {
            line[2 + i] = self.input[i];
        }
        self.push_line(&line[..2 + cp]);

        if self.input_len == 0 {
            self.clear_input();
            return;
        }

        if self.equals(b"help") {
            self.push_line(b"help clear about mouse search");
        } else if self.equals(b"clear") {
            self.line_count = 0;
            self.push_line(b"terminal cleared");
        } else if self.equals(b"about") {
            self.push_line(b"Linear framebuffer desktop");
        } else if self.equals(b"mouse") {
            self.push_line(b"Mouse enabled: click UI");
        } else if self.equals(b"search") {
            self.push_line(b"Use bottom search bar");
        } else {
            self.push_line(b"Unknown command");
        }

        self.clear_input();
    }
}

pub fn run() -> ! {
    let mut terminal = Terminal::new();
    terminal.seed();

    let mut focus = Focus::Desktop;
    let mut terminal_open = false;
    let mut search = [0u8; 40];
    let mut search_len = 0usize;
    let mut frame: u64 = 0;
    let mut mouse_state = mouse::current();
    let mut last_drawn_mouse = mouse_state;
    let mut redraw_divider: u32 = 0;

    draw(
        &terminal,
        focus,
        terminal_open,
        &search,
        search_len,
        mouse_state,
        frame,
    );

    loop {
        frame = frame.wrapping_add(1);
        redraw_divider = redraw_divider.wrapping_add(1);
        let mut redraw = false;

        let (w, h) = framebuffer::dimensions();
        let task_h = (h as f32 * 0.13) as usize;
        let search_h = task_h.saturating_sub(16);
        let search_w = w.min(900) * 42 / 100;
        let search_x = (w.saturating_sub(search_w)) / 2;
        let search_y = h.saturating_sub(task_h) + ((task_h.saturating_sub(search_h)) / 2);

        let term_w = w.min(1100) * 68 / 100;
        let term_h = h.min(760) * 68 / 100;
        let term_x = (w.saturating_sub(term_w)) / 2;
        let term_y = (h.saturating_sub(task_h + term_h)) / 2;

        // Drain all pending mouse packets so clicks/releases don't get lost.
        while let Some(ms) = mouse::poll() {
            let moved = ms.x != mouse_state.x || ms.y != mouse_state.y;
            mouse_state = ms;
            if moved || ms.left_clicked {
                redraw = true;
            }

            if ms.left_clicked || ms.left_down {
                if in_rect(ms.x, ms.y, search_x, search_y, search_w, search_h) {
                    focus = Focus::Search;
                } else if in_rect(ms.x, ms.y, ICON_X, ICON_Y, ICON_W, ICON_H) {
                    terminal_open = true;
                    focus = Focus::Terminal;
                } else if in_rect(ms.x, ms.y, term_x, term_y, term_w, term_h) {
                    if terminal_open {
                        focus = Focus::Terminal;
                    } else {
                        focus = Focus::Desktop;
                    }
                } else {
                    focus = Focus::Desktop;
                }
            }
        }

        if let Some(key) = input::poll_key() {
            match key {
                Key::ArrowUp => {
                    mouse_state.y = mouse_state.y.saturating_sub(10);
                    redraw = true;
                }
                Key::ArrowDown => {
                    let (_, h) = framebuffer::dimensions();
                    mouse_state.y = (mouse_state.y + 10).min(h.saturating_sub(1));
                    redraw = true;
                }
                Key::ArrowLeft => {
                    mouse_state.x = mouse_state.x.saturating_sub(10);
                    redraw = true;
                }
                Key::ArrowRight => {
                    let (w, _) = framebuffer::dimensions();
                    mouse_state.x = (mouse_state.x + 10).min(w.saturating_sub(1));
                    redraw = true;
                }
                Key::Tab => {
                    focus = match (terminal_open, focus) {
                        (true, Focus::Terminal) => Focus::Search,
                        (true, Focus::Search) => Focus::Desktop,
                        (true, Focus::Desktop) => Focus::Terminal,
                        (false, Focus::Search) => Focus::Desktop,
                        (false, Focus::Desktop) => Focus::Search,
                        (false, Focus::Terminal) => Focus::Desktop,
                    };
                    redraw = true;
                }
                Key::Enter => {
                    if terminal_open && focus == Focus::Terminal {
                        terminal.submit();
                        redraw = true;
                    } else {
                        if !terminal_open {
                            terminal_open = true;
                            focus = Focus::Terminal;
                            continue;
                        }

                        // Logical click on current cursor position for keyboard-only usage.
                        if in_rect(mouse_state.x, mouse_state.y, search_x, search_y, search_w, search_h) {
                            focus = Focus::Search;
                        } else if in_rect(mouse_state.x, mouse_state.y, ICON_X, ICON_Y, ICON_W, ICON_H) {
                            terminal_open = true;
                            focus = Focus::Terminal;
                        } else if in_rect(mouse_state.x, mouse_state.y, term_x, term_y, term_w, term_h) {
                            if terminal_open {
                                focus = Focus::Terminal;
                            } else {
                                focus = Focus::Desktop;
                            }
                        } else {
                            focus = Focus::Desktop;
                        }
                        redraw = true;
                    }
                }
                Key::Backspace => {
                    if terminal_open && focus == Focus::Terminal {
                        terminal.backspace();
                    } else if search_len > 0 {
                        search_len -= 1;
                        search[search_len] = 0;
                    }
                    redraw = true;
                }
                Key::Escape => {
                    if terminal_open && focus == Focus::Terminal {
                        terminal_open = false;
                        focus = Focus::Desktop;
                    } else {
                        focus = Focus::Desktop;
                    }
                    redraw = true;
                }
                Key::Char(ch) => {
                    if !terminal_open && (ch == 't' || ch == 'T') {
                        terminal_open = true;
                        focus = Focus::Terminal;
                        continue;
                    }

                    if terminal_open && focus == Focus::Terminal {
                        terminal.add_char(ch);
                    } else if ch.is_ascii() {
                        let b = ch as u8;
                        if (0x20..=0x7e).contains(&b) && search_len < search.len() {
                            search[search_len] = b;
                            search_len += 1;
                        }
                    }
                    redraw = true;
                }
            }
        }

        let cursor_changed = mouse_state.x != last_drawn_mouse.x || mouse_state.y != last_drawn_mouse.y;
        if cursor_changed && redraw_divider % 2 == 0 {
            redraw = true;
        }

        if redraw {
            draw(
                &terminal,
                focus,
                terminal_open,
                &search,
                search_len,
                mouse_state,
                frame,
            );
            last_drawn_mouse = mouse_state;
        } else {
            core::hint::spin_loop();
        }
    }
}

fn draw(
    terminal: &Terminal,
    focus: Focus,
    terminal_open: bool,
    search: &[u8; 40],
    search_len: usize,
    mouse_state: mouse::MouseState,
    frame: u64,
) {
    let (w, h) = framebuffer::dimensions();
    if w == 0 || h == 0 {
        return;
    }

    draw_wallpaper(w, h);

    let task_h = (h as f32 * 0.13) as usize;
    let term_w = w.min(1100) * 68 / 100;
    let term_h = h.min(760) * 68 / 100;
    let term_x = (w.saturating_sub(term_w)) / 2;
    let term_y = (h.saturating_sub(task_h + term_h)) / 2;

    draw_terminal_icon(focus == Focus::Desktop, terminal_open);

    if terminal_open {
        draw_terminal(
            terminal,
            focus == Focus::Terminal,
            term_x,
            term_y,
            term_w,
            term_h,
            frame,
        );
    }
    draw_taskbar(focus, search, search_len, w, h, task_h);
    draw_keyboard_debug_overlay(w);

    framebuffer::draw_mouse_cursor(mouse_state.x, mouse_state.y);
}

fn draw_keyboard_debug_overlay(screen_w: usize) {
    let dbg = input::debug_snapshot();

    let panel_w = 262;
    let panel_h = 92;
    let panel_x = screen_w.saturating_sub(panel_w + 12);
    let panel_y = 20;

    framebuffer::draw_panel(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        Color::new(22, 30, 42),
        Color::new(90, 110, 150),
    );

    framebuffer::draw_text(
        panel_x + 10,
        panel_y + 10,
        "KB DEBUG",
        Color::new(210, 226, 255),
        1,
    );

    let mut raw_hex = [0u8; 2];
    let mut code_hex = [0u8; 2];
    to_hex2(dbg.raw_scancode, &mut raw_hex);
    to_hex2(dbg.code, &mut code_hex);

    framebuffer::draw_text(panel_x + 10, panel_y + 26, "raw:", Color::new(180, 195, 220), 1);
    framebuffer::draw_ascii(
        panel_x + 54,
        panel_y + 26,
        &raw_hex,
        2,
        Color::new(132, 255, 189),
        1,
    );

    framebuffer::draw_text(panel_x + 90, panel_y + 26, "code:", Color::new(180, 195, 220), 1);
    framebuffer::draw_ascii(
        panel_x + 142,
        panel_y + 26,
        &code_hex,
        2,
        Color::new(132, 255, 189),
        1,
    );

    framebuffer::draw_text(
        panel_x + 10,
        panel_y + 42,
        if dbg.extended { "ext:1" } else { "ext:0" },
        Color::new(180, 195, 220),
        1,
    );
    framebuffer::draw_text(
        panel_x + 90,
        panel_y + 42,
        if dbg.released { "rel:1" } else { "rel:0" },
        Color::new(180, 195, 220),
        1,
    );

    let label = match dbg.event_id {
        1 => "event: ENTER",
        2 => "event: BACKSPACE",
        3 => "event: TAB",
        4 => "event: ESC",
        5 => "event: UP",
        6 => "event: DOWN",
        7 => "event: LEFT",
        8 => "event: RIGHT",
        9 => "event: CHAR",
        _ => "event: NONE",
    };
    framebuffer::draw_text(panel_x + 10, panel_y + 58, label, Color::new(215, 226, 252), 1);

    if dbg.event_id == 9 && dbg.has_event {
        framebuffer::draw_text(panel_x + 10, panel_y + 74, "char:", Color::new(180, 195, 220), 1);
        let ch = [dbg.event_char];
        framebuffer::draw_ascii(panel_x + 58, panel_y + 74, &ch, 1, Color::new(255, 245, 136), 1);
    } else {
        framebuffer::draw_text(
            panel_x + 10,
            panel_y + 74,
            "Press key and watch scancode",
            Color::new(170, 182, 205),
            1,
        );
    }
}

fn to_hex2(v: u8, out: &mut [u8; 2]) {
    out[0] = hex_nibble((v >> 4) & 0x0f);
    out[1] = hex_nibble(v & 0x0f);
}

fn hex_nibble(v: u8) -> u8 {
    match v {
        0..=9 => b'0' + v,
        _ => b'A' + (v - 10),
    }
}

fn draw_wallpaper(w: usize, h: usize) {
    for y in 0..h {
        let t = y as u32 * 255 / h.max(1) as u32;
        let r = 18 + (t / 10) as u8;
        let g = 78 + (t / 5) as u8;
        let b = 138 + (t / 4) as u8;
        framebuffer::fill_rect(0, y, w, 1, Color::new(r, g, b));
    }

    framebuffer::fill_rect(w / 8, h / 6, w / 6, h / 3, Color::new(255, 255, 255));
    framebuffer::fill_rect(w / 8 + 2, h / 6 + 2, w / 6 - 4, h / 3 - 4, Color::new(40, 110, 180));
}

fn draw_terminal_icon(active: bool, opened: bool) {
    let x = ICON_X;
    let y = ICON_Y;
    let w = ICON_W;
    let h = ICON_H;

    let card = if active {
        Color::new(232, 238, 252)
    } else {
        Color::new(220, 228, 245)
    };

    framebuffer::draw_panel(x, y, w, h, card, Color::new(110, 126, 172));
    framebuffer::draw_panel(x + 16, y + 12, 76, 58, Color::new(20, 28, 38), Color::new(70, 84, 118));
    framebuffer::draw_text(x + 30, y + 32, ">_", Color::new(112, 242, 188), 2);
    framebuffer::draw_text(x + 20, y + 82, "Terminal", Color::new(30, 40, 62), 1);
    if opened {
        framebuffer::draw_text(x + 20, y + 98, "Open", Color::new(40, 110, 70), 1);
    } else {
        framebuffer::draw_text(x + 20, y + 98, "Click", Color::new(70, 80, 100), 1);
    }
}

fn draw_terminal(
    terminal: &Terminal,
    active: bool,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    frame: u64,
) {
    let border = if active {
        Color::new(30, 75, 180)
    } else {
        Color::new(110, 110, 120)
    };

    framebuffer::draw_panel(x, y, width, height, Color::new(244, 246, 252), border);
    framebuffer::fill_rect(x + 1, y + 1, width - 2, 34, Color::new(32, 42, 68));
    framebuffer::draw_text(x + 14, y + 11, "Terminal", Color::new(240, 245, 255), 2);

    let content_x = x + 16;
    let content_y = y + 46;
    let content_w = width.saturating_sub(32);
    let content_h = height.saturating_sub(64);

    framebuffer::draw_panel(content_x, content_y, content_w, content_h, Color::new(15, 22, 30), Color::new(45, 56, 76));

    for i in 0..terminal.line_count {
        let ty = content_y + 10 + i * 16;
        framebuffer::draw_ascii(content_x + 10, ty, &terminal.lines[i], terminal.lens[i], Color::new(225, 235, 245), 1);
    }

    let input_y = content_y + content_h.saturating_sub(20);
    framebuffer::draw_text(content_x + 10, input_y, ">", Color::new(100, 230, 180), 1);
    framebuffer::draw_ascii(content_x + 20, input_y, &terminal.input, terminal.input_len, Color::new(220, 230, 240), 1);

    if frame % 30 < 15 {
        let cx = content_x + 20 + terminal.input_len * 8;
        framebuffer::fill_rect(cx, input_y + 10, 7, 2, Color::new(100, 230, 180));
    }
}

fn draw_taskbar(focus: Focus, search: &[u8; 40], search_len: usize, w: usize, h: usize, task_h: usize) {
    let y = h.saturating_sub(task_h);

    framebuffer::fill_rect(0, y, w, task_h, Color::new(232, 236, 246));
    framebuffer::fill_rect(0, y, w, 1, Color::new(165, 175, 196));

    framebuffer::draw_panel(16, y + 8, 56, task_h.saturating_sub(16), Color::new(246, 248, 254), Color::new(168, 178, 200));
    framebuffer::draw_text(30, y + 18, "Start", Color::new(38, 50, 74), 1);

    let search_h = task_h.saturating_sub(16);
    let search_w = w.min(900) * 42 / 100;
    let search_x = (w.saturating_sub(search_w)) / 2;
    let search_y = y + ((task_h.saturating_sub(search_h)) / 2);

    let bg = if focus == Focus::Search {
        Color::new(255, 255, 255)
    } else {
        Color::new(246, 248, 253)
    };

    framebuffer::draw_panel(search_x, search_y, search_w, search_h, bg, Color::new(178, 188, 210));
    framebuffer::draw_text(search_x + 12, search_y + 10, "Search", Color::new(112, 120, 142), 1);
    framebuffer::draw_ascii(search_x + 92, search_y + 10, search, search_len, Color::new(40, 52, 78), 1);
}

fn in_rect(x: usize, y: usize, rx: usize, ry: usize, rw: usize, rh: usize) -> bool {
    x >= rx && x < rx.saturating_add(rw) && y >= ry && y < ry.saturating_add(rh)
}
