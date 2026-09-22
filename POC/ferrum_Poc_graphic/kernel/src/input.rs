use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1, ScancodeSet2};
use spin::Mutex;
use x86_64::instructions::port::Port;

#[derive(Clone, Copy)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
}

#[derive(Clone, Copy)]
pub struct DebugKeyState {
    pub raw_scancode: u8,
    pub code: u8,
    pub extended: bool,
    pub released: bool,
    pub event_id: u8,
    pub event_char: u8,
    pub has_event: bool,
}

impl DebugKeyState {
    const fn new() -> Self {
        Self {
            raw_scancode: 0,
            code: 0,
            extended: false,
            released: false,
            event_id: 0,
            event_char: 0,
            has_event: false,
        }
    }
}

lazy_static! {
    static ref KEYBOARD_SET1: Mutex<Keyboard<layouts::Azerty, ScancodeSet1>> =
        Mutex::new(Keyboard::new(
            ScancodeSet1::new(),
            layouts::Azerty,
            HandleControl::Ignore,
        ));
    static ref KEYBOARD_SET2: Mutex<Keyboard<layouts::Azerty, ScancodeSet2>> =
        Mutex::new(Keyboard::new(
            ScancodeSet2::new(),
            layouts::Azerty,
            HandleControl::Ignore,
        ));
}

static DEBUG_STATE: Mutex<DebugKeyState> = Mutex::new(DebugKeyState::new());
static SERIAL_READY: Mutex<bool> = Mutex::new(false);
static SERIAL_ESC_STATE: Mutex<u8> = Mutex::new(0);

pub fn poll_key() -> Option<Key> {
    init_serial_once();

    if let Some(key) = poll_ps2_key() {
        return Some(key);
    }

    poll_serial_key()
}

fn poll_ps2_key() -> Option<Key> {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    let status = unsafe { status_port.read() };
    if status & 0x01 == 0 {
        return None;
    }

    // Ignore mouse bytes from the shared controller output buffer.
    if status & 0x20 != 0 {
        return None;
    }

    let scancode = unsafe { data_port.read() };

    {
        let mut dbg = DEBUG_STATE.lock();
        dbg.raw_scancode = scancode;
        dbg.has_event = false;
    }

    // Keep basic raw flags for overlay readability.
    {
        let mut dbg = DEBUG_STATE.lock();
        dbg.extended = scancode == 0xE0;
        dbg.released = scancode == 0xF0 || (scancode & 0x80 != 0);
        dbg.code = if scancode & 0x80 != 0 {
            scancode & 0x7f
        } else {
            scancode
        };
    }

    if let Some(key) = decode_with_set1(scancode) {
        update_debug_key(key);
        return Some(key);
    }

    if let Some(key) = decode_with_set2(scancode) {
        update_debug_key(key);
        return Some(key);
    }

    None
}

fn poll_serial_key() -> Option<Key> {
    let mut line_status: Port<u8> = Port::new(0x3fd);
    let ready = unsafe { line_status.read() } & 1 != 0;
    if !ready {
        return None;
    }

    let mut data_port: Port<u8> = Port::new(0x3f8);
    let byte = unsafe { data_port.read() };

    {
        let mut dbg = DEBUG_STATE.lock();
        dbg.raw_scancode = byte;
        dbg.code = byte;
        dbg.extended = false;
        dbg.released = false;
        dbg.has_event = false;
    }

    let mut esc = SERIAL_ESC_STATE.lock();
    let state = *esc;

    if state == 0 {
        if byte == 0x1b {
            *esc = 1;
            return None;
        }
        let key = match byte {
            b'\r' | b'\n' => Some(Key::Enter),
            0x08 | 0x7f => Some(Key::Backspace),
            b'\t' => Some(Key::Tab),
            _ if (0x20..=0x7e).contains(&byte) => Some(Key::Char(byte as char)),
            _ => None,
        };
        drop(esc);
        if let Some(k) = key {
            update_debug_key(k);
            return Some(k);
        }
        return None;
    }

    if state == 1 {
        if byte == b'[' {
            *esc = 2;
            return None;
        }
        *esc = 0;
        drop(esc);
        if byte == 0x1b {
            update_debug_key(Key::Escape);
            return Some(Key::Escape);
        }
        return None;
    }

    *esc = 0;
    drop(esc);

    let key = match byte {
        b'A' => Some(Key::ArrowUp),
        b'B' => Some(Key::ArrowDown),
        b'C' => Some(Key::ArrowRight),
        b'D' => Some(Key::ArrowLeft),
        _ => None,
    };

    if let Some(k) = key {
        update_debug_key(k);
        return Some(k);
    }

    None
}

fn init_serial_once() {
    let mut ready = SERIAL_READY.lock();
    if *ready {
        return;
    }

    let mut port_ier: Port<u8> = Port::new(0x3f9);
    let mut port_lcr: Port<u8> = Port::new(0x3fb);
    let mut port_data: Port<u8> = Port::new(0x3f8);
    let mut port_fcr: Port<u8> = Port::new(0x3fa);
    let mut port_mcr: Port<u8> = Port::new(0x3fc);

    unsafe {
        port_ier.write(0x00u8);
        port_lcr.write(0x80u8);
        port_data.write(0x03u8);
        port_ier.write(0x00u8);
        port_lcr.write(0x03u8);
        port_fcr.write(0xC7u8);
        port_mcr.write(0x0Bu8);
    }

    *ready = true;
}

pub fn debug_snapshot() -> DebugKeyState {
    *DEBUG_STATE.lock()
}

fn update_debug_key(key: Key) {
    let mut dbg = DEBUG_STATE.lock();
    dbg.has_event = true;
    match key {
        Key::Enter => {
            dbg.event_id = 1;
            dbg.event_char = 0;
        }
        Key::Backspace => {
            dbg.event_id = 2;
            dbg.event_char = 0;
        }
        Key::Tab => {
            dbg.event_id = 3;
            dbg.event_char = 0;
        }
        Key::Escape => {
            dbg.event_id = 4;
            dbg.event_char = 0;
        }
        Key::ArrowUp => {
            dbg.event_id = 5;
            dbg.event_char = 0;
        }
        Key::ArrowDown => {
            dbg.event_id = 6;
            dbg.event_char = 0;
        }
        Key::ArrowLeft => {
            dbg.event_id = 7;
            dbg.event_char = 0;
        }
        Key::ArrowRight => {
            dbg.event_id = 8;
            dbg.event_char = 0;
        }
        Key::Char(ch) => {
            dbg.event_id = 9;
            dbg.event_char = ch as u8;
        }
    }
}

fn decode_with_set1(scancode: u8) -> Option<Key> {
    let mut kb = KEYBOARD_SET1.lock();
    if let Ok(Some(event)) = kb.add_byte(scancode) {
        if let Some(decoded) = kb.process_keyevent(event) {
            return map_decoded_key(decoded);
        }
    }
    None
}

fn decode_with_set2(scancode: u8) -> Option<Key> {
    let mut kb = KEYBOARD_SET2.lock();
    if let Ok(Some(event)) = kb.add_byte(scancode) {
        if let Some(decoded) = kb.process_keyevent(event) {
            return map_decoded_key(decoded);
        }
    }
    None
}

fn map_decoded_key(decoded: DecodedKey) -> Option<Key> {
    match decoded {
        DecodedKey::Unicode('\n') | DecodedKey::Unicode('\r') => Some(Key::Enter),
        DecodedKey::Unicode('\t') => Some(Key::Tab),
        DecodedKey::Unicode('\x08') => Some(Key::Backspace),
        DecodedKey::Unicode(ch) => Some(Key::Char(ch)),
        DecodedKey::RawKey(pc_keyboard::KeyCode::Backspace) => Some(Key::Backspace),
        DecodedKey::RawKey(pc_keyboard::KeyCode::Return) => Some(Key::Enter),
        DecodedKey::RawKey(pc_keyboard::KeyCode::NumpadEnter) => Some(Key::Enter),
        DecodedKey::RawKey(pc_keyboard::KeyCode::Tab) => Some(Key::Tab),
        DecodedKey::RawKey(pc_keyboard::KeyCode::Escape) => Some(Key::Escape),
        DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowUp) => Some(Key::ArrowUp),
        DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowDown) => Some(Key::ArrowDown),
        DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowLeft) => Some(Key::ArrowLeft),
        DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowRight) => Some(Key::ArrowRight),
        _ => None,
    }
}
