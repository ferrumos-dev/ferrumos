use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::instructions::port::Port;

pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
}

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Azerty, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Azerty,
            HandleControl::Ignore,
        ));
}

pub fn poll_key() -> Option<Key> {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    let status = unsafe { status_port.read() };
    if status & 0x01 == 0 {
        return None;
    }

    if status & 0x20 != 0 {
        return None;
    }

    let scancode = unsafe { data_port.read() };
    let mut keyboard = KEYBOARD.lock();

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(decoded) = keyboard.process_keyevent(key_event) {
            return match decoded {
                DecodedKey::Unicode('\n') | DecodedKey::Unicode('\r') => Some(Key::Enter),
                DecodedKey::Unicode('\t') => Some(Key::Tab),
                DecodedKey::Unicode('\x08') => Some(Key::Backspace),
                DecodedKey::Unicode(ch) => Some(Key::Char(ch)),
                DecodedKey::RawKey(pc_keyboard::KeyCode::Backspace) => Some(Key::Backspace),
                DecodedKey::RawKey(pc_keyboard::KeyCode::Return) => Some(Key::Enter),
                DecodedKey::RawKey(pc_keyboard::KeyCode::NumpadEnter) => Some(Key::Enter),
                DecodedKey::RawKey(pc_keyboard::KeyCode::Tab) => Some(Key::Tab),
                DecodedKey::RawKey(pc_keyboard::KeyCode::Escape) => Some(Key::Escape),
                _ => None,
            };
        }
    }

    None
}