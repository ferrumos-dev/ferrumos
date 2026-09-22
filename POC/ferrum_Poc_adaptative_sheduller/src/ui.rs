use alloc::string::String;
use core::fmt::Write;

use crate::scheduler;
use crate::shell;
use crate::vga_buffer::{self, Color, BUFFER_HEIGHT, BUFFER_WIDTH};

const DESKTOP_BG: Color = Color::Cyan;
const TITLE_BG: Color = Color::Blue;
const WINDOW_BG: Color = Color::LightGray;
const WINDOW_BORDER: Color = Color::Black;
const WINDOW_TITLE_BG: Color = Color::White;
const WINDOW_TITLE_FG: Color = Color::Black;

const TERMINAL_ICON_ROW: usize = 6;
const TERMINAL_ICON_COL: usize = 4;
const TASKMGR_TOP: usize = 4;
const TASKMGR_LEFT: usize = 20;
const TASKMGR_WIDTH: usize = 56;
const TASKMGR_HEIGHT: usize = 17;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Terminal,
    TaskManager,
}

pub fn show_boot_screen() {
    let mut focus = Focus::Terminal;
    let mut frame: u32 = 0;

    draw_desktop(focus);

    loop {
        scheduler::tick();
        frame = frame.wrapping_add(1);

        if frame % 12 == 0 {
            draw_task_manager_content(focus);
        }

        if let Some(key) = shell::next_key() {
            match key {
                '\t' | 'h' | 'H' => {
                    focus = match focus {
                        Focus::Terminal => Focus::TaskManager,
                        Focus::TaskManager => Focus::Terminal,
                    };
                    draw_desktop(focus);
                }
                '\r' | '\n' => {
                    if focus == Focus::Terminal {
                        vga_buffer::clear_screen(Color::Black);
                        vga_buffer::reset_terminal_theme();
                        return;
                    }
                    draw_desktop(focus);
                }
                't' | 'T' => {
                    vga_buffer::clear_screen(Color::Black);
                    vga_buffer::reset_terminal_theme();
                    return;
                }
                'r' | 'R' => draw_desktop(focus),
                _ => {}
            }
        } else {
            core::hint::spin_loop();
        }
    }
}

fn draw_desktop(focus: Focus) {
    vga_buffer::clear_screen(DESKTOP_BG);

    draw_top_bar();
    draw_desktop_icons(focus);
    draw_task_manager_window(focus);
    draw_bottom_hint(focus);
}

fn draw_top_bar() {
    fill_row(0, 0, BUFFER_WIDTH, ' ', Color::White, TITLE_BG);
    vga_buffer::draw_text(0, 2, "FerrumOS Desktop", Color::White, TITLE_BG);
    vga_buffer::draw_text(0, 27, "File  View  System", Color::LightGray, TITLE_BG);
    vga_buffer::draw_text(0, 63, "Task Manager", Color::Yellow, TITLE_BG);

    fill_row(1, 0, BUFFER_WIDTH, ' ', Color::Black, Color::LightGray);
    vga_buffer::draw_text(1, 2, "Windows-1 style shell launcher", Color::Black, Color::LightGray);
    vga_buffer::draw_text(1, 53, "Press T for terminal", Color::Black, Color::LightGray);
}

fn draw_desktop_icons(focus: Focus) {
    let selected = focus == Focus::Terminal;
    let label_bg = if selected { Color::White } else { DESKTOP_BG };
    let label_fg = if selected { Color::Black } else { Color::White };

    vga_buffer::draw_text(TERMINAL_ICON_ROW, TERMINAL_ICON_COL, "  ______  ", Color::White, DESKTOP_BG);
    vga_buffer::draw_text(TERMINAL_ICON_ROW + 1, TERMINAL_ICON_COL, " | [] []| ", Color::White, DESKTOP_BG);
    vga_buffer::draw_text(TERMINAL_ICON_ROW + 2, TERMINAL_ICON_COL, " |  __  | ", Color::White, DESKTOP_BG);
    vga_buffer::draw_text(TERMINAL_ICON_ROW + 3, TERMINAL_ICON_COL, " | |  | | ", Color::White, DESKTOP_BG);
    vga_buffer::draw_text(TERMINAL_ICON_ROW + 4, TERMINAL_ICON_COL, " |_|__|_| ", Color::White, DESKTOP_BG);
    vga_buffer::draw_text(TERMINAL_ICON_ROW + 6, TERMINAL_ICON_COL + 1, "Terminal", label_fg, label_bg);

    vga_buffer::draw_text(TERMINAL_ICON_ROW + 9, TERMINAL_ICON_COL, "FerrumOS PoC", Color::LightGray, DESKTOP_BG);
    vga_buffer::draw_text(TERMINAL_ICON_ROW + 10, TERMINAL_ICON_COL, "Enter opens shell", Color::LightGray, DESKTOP_BG);
}

fn draw_task_manager_window(focus: Focus) {
    draw_window(
        TASKMGR_TOP,
        TASKMGR_LEFT,
        TASKMGR_WIDTH,
        TASKMGR_HEIGHT,
        "Task Manager",
        focus == Focus::TaskManager,
    );
    draw_task_manager_content(focus);
}

fn draw_task_manager_content(focus: Focus) {
    let content_left = TASKMGR_LEFT + 2;
    let content_top = TASKMGR_TOP + 3;
    let content_width = TASKMGR_WIDTH - 4;

    for row in content_top..(TASKMGR_TOP + TASKMGR_HEIGHT - 1) {
        fill_row(row, content_left, content_width, ' ', Color::Black, WINDOW_BG);
    }

    let (total_ticks, active_tasks, running_name, q0, q1, q2, completed, avg_wait, tp_x100) =
        scheduler::snapshot();

    draw_window_line(content_top, "Live scheduler overview");
    draw_kv_line(content_top + 2, "Total ticks", total_ticks);
    draw_kv_line(content_top + 3, "Active tasks", active_tasks as u32);
    draw_kv_line(content_top + 4, "Completed", completed);
    draw_kv_line(content_top + 5, "Queue high", q0 as u32);
    draw_kv_line(content_top + 6, "Queue mid", q1 as u32);
    draw_kv_line(content_top + 7, "Queue low", q2 as u32);
    draw_kv_line(content_top + 8, "Avg wait", avg_wait);

    let mut throughput = String::new();
    let _ = write!(&mut throughput, "{}.{:02} tasks/100t", tp_x100 / 100, tp_x100 % 100);
    vga_buffer::draw_text(content_top + 10, content_left, "Throughput", Color::Black, WINDOW_BG);
    vga_buffer::draw_text(content_top + 10, content_left + 18, &throughput, Color::Blue, WINDOW_BG);

    vga_buffer::draw_text(content_top + 12, content_left, "Running", Color::Black, WINDOW_BG);
    vga_buffer::draw_text(content_top + 12, content_left + 18, &running_name, Color::Red, WINDOW_BG);

    let footer_bg = if focus == Focus::TaskManager { Color::Blue } else { WINDOW_BG };
    let footer_fg = if focus == Focus::TaskManager { Color::White } else { Color::Black };
    fill_row(TASKMGR_TOP + TASKMGR_HEIGHT - 2, content_left, content_width, ' ', footer_fg, footer_bg);
    vga_buffer::draw_text(
        TASKMGR_TOP + TASKMGR_HEIGHT - 2,
        content_left + 1,
        "Tab switches focus  R refreshes  Enter on Terminal opens shell",
        footer_fg,
        footer_bg,
    );
}

fn draw_bottom_hint(focus: Focus) {
    fill_row(BUFFER_HEIGHT - 2, 0, BUFFER_WIDTH, ' ', Color::Black, Color::LightGray);
    fill_row(BUFFER_HEIGHT - 1, 0, BUFFER_WIDTH, ' ', Color::White, Color::Blue);

    let status = match focus {
        Focus::Terminal => "Focus: Terminal",
        Focus::TaskManager => "Focus: Task Manager",
    };
    vga_buffer::draw_text(BUFFER_HEIGHT - 2, 2, status, Color::Black, Color::LightGray);
    vga_buffer::draw_text(BUFFER_HEIGHT - 2, 28, "Windows 1 inspired desktop mockup", Color::Black, Color::LightGray);
    vga_buffer::draw_text(BUFFER_HEIGHT - 1, 2, "Keys: Tab switch focus | Enter open terminal | T terminal | R redraw", Color::White, Color::Blue);
}

fn draw_window(top: usize, left: usize, width: usize, height: usize, title: &str, active: bool) {
    let title_bg = if active { Color::Blue } else { WINDOW_TITLE_BG };
    let title_fg = if active { Color::White } else { WINDOW_TITLE_FG };

    for row in top..(top + height) {
        fill_row(row, left, width, ' ', WINDOW_BORDER, WINDOW_BG);
    }

    fill_row(top, left, width, '#', WINDOW_BORDER, WINDOW_BORDER);
    fill_row(top + height - 1, left, width, '#', WINDOW_BORDER, WINDOW_BORDER);
    fill_row(top + 1, left, width, ' ', title_fg, title_bg);

    for row in (top + 1)..(top + height - 1) {
        vga_buffer::draw_text(row, left, "#", WINDOW_BORDER, WINDOW_BORDER);
        vga_buffer::draw_text(row, left + width - 1, "#", WINDOW_BORDER, WINDOW_BORDER);
    }

    vga_buffer::draw_text(top + 1, left + 2, title, title_fg, title_bg);
    vga_buffer::draw_text(top + 1, left + width - 8, "[_][X]", title_fg, title_bg);
}

fn draw_window_line(row: usize, text: &str) {
    vga_buffer::draw_text(row, TASKMGR_LEFT + 2, text, Color::Black, WINDOW_BG);
}

fn draw_kv_line(row: usize, label: &str, value: u32) {
    let mut line = String::new();
    let _ = write!(&mut line, "{}", value);
    vga_buffer::draw_text(row, TASKMGR_LEFT + 2, label, Color::Black, WINDOW_BG);
    vga_buffer::draw_text(row, TASKMGR_LEFT + 20, &line, Color::Blue, WINDOW_BG);
}

fn fill_row(row: usize, col: usize, width: usize, fill: char, foreground: Color, background: Color) {
    let mut line = String::new();
    for _ in 0..width {
        line.push(fill);
    }
    vga_buffer::draw_text(row, col, &line, foreground, background);
}