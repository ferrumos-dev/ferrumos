use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use lazy_static::lazy_static;
use crate::{print, println};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::instructions::port::Port;

lazy_static! {
    static ref INPUT_BUFFER: Mutex<Vec<char>> = Mutex::new(Vec::new());
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
        ));
}

pub fn add_key(c: char) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let mut buffer = INPUT_BUFFER.lock();
        buffer.push(c);
    });
}

pub fn run() -> ! {
    let mut command_buffer = String::new();
    
    print!("FerrumOS> ");
    
    loop {
        let mut c = x86_64::instructions::interrupts::without_interrupts(|| {
            let mut buffer = INPUT_BUFFER.lock();
            if !buffer.is_empty() {
                Some(buffer.remove(0))
            } else {
                None
            }
        });

        if c.is_none() {
            c = poll_keyboard_char();
        }

        if let Some(c) = c {
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
                        print!("{}", c); 
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
            println!("FerrumOS 0.1.0");
            println!("Available commands:");
            println!("  help     - Show this help message");
            println!("  version  - Show OS version");
            println!("  mem      - Show memory information");
        }
        "version" => {
            println!("FerrumOS v0.1.0");
        }
        "mem" => {
            println!("Memory management enabled.");
            println!("Heap allocator: Simple Bump Allocator");
        }
        "" => {}
        _ => {
            println!("Unknown command: {}", trimmed);
        }
    }
}
