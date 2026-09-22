use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::{print, println};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::instructions::port::Port;

const INPUT_CAPACITY: usize = 256;

struct InputQueue {
    data: [char; INPUT_CAPACITY],
    head: usize,
    len: usize,
}

impl InputQueue {
    const fn new() -> Self {
        Self {
            data: ['\0'; INPUT_CAPACITY],
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, c: char) {
        if self.len >= INPUT_CAPACITY {
            return;
        }
        let tail = (self.head + self.len) % INPUT_CAPACITY;
        self.data[tail] = c;
        self.len += 1;
    }

    fn pop(&mut self) -> Option<char> {
        if self.len == 0 {
            return None;
        }
        let c = self.data[self.head];
        self.head = (self.head + 1) % INPUT_CAPACITY;
        self.len -= 1;
        Some(c)
    }
}

lazy_static! {
    static ref INPUT_BUFFER: Mutex<InputQueue> = Mutex::new(InputQueue::new());
    static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Azerty,
            HandleControl::Ignore,
        ));
}

pub fn add_key(c: char) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut buffer = INPUT_BUFFER.lock();
        buffer.push(c);
    });
}

pub fn next_key() -> Option<char> {
    let mut queued = x86_64::instructions::interrupts::without_interrupts(|| {
        let mut buffer = INPUT_BUFFER.lock();
        buffer.pop()
    });

    if queued.is_none() {
        queued = poll_keyboard_char();
    }

    queued
}

pub fn run() -> ! {
    let mut command_buffer = String::new();
    
    print!("FerrumOS> ");
    
    loop {
        crate::scheduler::tick();

        if let Some(c) = next_key() {
            match c {
                '\r' | '\n' => {
                    println!("");
                    process_command(&command_buffer);
                    command_buffer.clear();
                    print!("FerrumOS> ");
                }
                '\x08' | '\x7f' => {
                    if !command_buffer.is_empty() {
                        command_buffer.pop();
                        crate::vga_buffer::backspace();
                    }
                }
                _ => {
                    command_buffer.push(c);
                    print!("{}", c);
                }
            }
        } else {
            core::hint::spin_loop();
        }
    }
}

fn poll_keyboard_char() -> Option<char> {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    let status = unsafe { status_port.read() };
    if status & 0x01 == 0 {
        return None;
    }

    let scancode = unsafe { data_port.read() };
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(decoded) = keyboard.process_keyevent(key_event) {
            return match decoded {
                DecodedKey::Unicode(ch) => Some(ch),
                DecodedKey::RawKey(_) => None,
            };
        }
    }

    None
}

fn process_command(command: &str) {
    let trimmed = command.trim();
    match trimmed {
        "help" => {
            println!("FerrumOS 0.1.0 - Adaptive MLFQ Scheduler PoC");
            println!("Available commands:");
            println!("  help                      - Show this help message");
            println!("  mem                       - Show memory information");
            println!("  stats                     - Show scheduler metrics");
            println!("  tasks                     - List all tasks and their queue level");
        }
        "mem" => {
            println!("Memory management enabled.");
            println!("Heap allocator: Simple Bump Allocator");
        }
        "stats" => {
            x86_64::instructions::interrupts::without_interrupts(|| {
                let sched = crate::scheduler::SCHEDULER.lock();
                let m = &sched.metrics;
                println!("--- Scheduler Metrics ---");
                println!("  Ticks total       : {}", m.total_ticks);
                println!("  Scheduled         : {}", m.total_scheduled);
                println!("  Demotions         : {}", m.total_demotions);
                println!("  Promotions        : {}", m.total_promotions);
                println!("  Aged (anti-starv) : {}", m.total_aged);
                println!("  Completed tasks   : {}", m.completed_tasks);
                println!("  Avg wait (ticks)  : {}", m.avg_wait());
                println!("  Max wait (ticks)  : {}", m.max_wait_ticks);
                let tp = m.throughput_x100();
                println!("  Throughput        : {}.{:02} tasks/100t", tp / 100, tp % 100);
                println!("  Active tasks      : {}", sched.task_count());
                println!("  Q0 (high)  len    : {}", sched.queue_len(0));
                println!("  Q1 (mid)   len    : {}", sched.queue_len(1));
                println!("  Q2 (low)   len    : {}", sched.queue_len(2));
                if let Some(ref t) = sched.running {
                    println!("  Running now       : [{}] {} ({})",
                        t.id, t.name_str(), t.task_type.name());
                } else {
                    println!("  Running now       : idle");
                }
            });
        }
        "tasks" => {
            x86_64::instructions::interrupts::without_interrupts(|| {
                let sched = crate::scheduler::SCHEDULER.lock();
                println!("--- Task List ---");
                println!(" ID  Name             Type         Q  Work  Wait");
                if let Some(ref t) = sched.running {
                    println!(" {:2}  {:<16} {}  *  {:4}  {:4}  <running>",
                        t.id, t.name_str(), t.task_type.name(), t.work_left, t.wait_ticks);
                }
                for level in 0..crate::scheduler::LEVELS {
                    for t in sched.queues[level].iter() {
                        println!(" {:2}  {:<16} {}  {}  {:4}  {:4}",
                            t.id, t.name_str(), t.task_type.name(),
                            level, t.work_left, t.wait_ticks);
                    }
                }
            });
        }
        "" => {}
        _ => {
            println!("Unknown command: {}", trimmed);
        }
    }
}
