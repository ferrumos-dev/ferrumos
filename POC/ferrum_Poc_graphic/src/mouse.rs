use spin::Mutex;
use x86_64::instructions::port::Port;

#[derive(Clone, Copy)]
pub struct MouseState {
    pub x: usize,
    pub y: usize,
    pub left_down: bool,
    pub left_clicked: bool,
}

impl MouseState {
    pub const fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            left_down: false,
            left_clicked: false,
        }
    }
}

struct MouseRuntime {
    packet: [u8; 3],
    index: usize,
    state: MouseState,
}

impl MouseRuntime {
    const fn new() -> Self {
        Self {
            packet: [0; 3],
            index: 0,
            state: MouseState::new(160, 100),
        }
    }

    fn handle_byte(&mut self, byte: u8) -> Option<MouseState> {
        self.packet[self.index] = byte;
        self.index += 1;

        if self.index < 3 {
            return None;
        }

        self.index = 0;

        let flags = self.packet[0];
        if flags & 0x08 == 0 {
            return None;
        }

        let dx = self.packet[1] as i8 as i16;
        let dy = self.packet[2] as i8 as i16;

        let next_x = self.state.x as i16 + dx;
        let next_y = self.state.y as i16 - dy;

        self.state.x = next_x.clamp(0, 319) as usize;
        self.state.y = next_y.clamp(0, 199) as usize;

        let left = flags & 0x01 != 0;
        self.state.left_clicked = left && !self.state.left_down;
        self.state.left_down = left;

        Some(self.state)
    }
}

static MOUSE: Mutex<MouseRuntime> = Mutex::new(MouseRuntime::new());

pub fn init() {
    let mut command_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    wait_write();
    unsafe { command_port.write(0xA8u8) };

    wait_write();
    unsafe { command_port.write(0x20u8) };
    wait_read();
    let status: u8 = unsafe { data_port.read() };
    let updated_status = (status | 0x02) & !0x20;

    wait_write();
    unsafe { command_port.write(0x60u8) };
    wait_write();
    unsafe { data_port.write(updated_status) };

    send_mouse_command(0xF6);
    let _ = read_mouse_ack();

    send_mouse_command(0xF4);
    let _ = read_mouse_ack();
}

pub fn poll() -> Option<MouseState> {
    let mut status_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    let status = unsafe { status_port.read() };
    if status & 0x01 == 0 {
        return None;
    }

    if status & 0x20 == 0 {
        return None;
    }

    let byte = unsafe { data_port.read() };
    let mut mouse = MOUSE.lock();
    mouse.handle_byte(byte)
}

pub fn current() -> MouseState {
    let mouse = MOUSE.lock();
    mouse.state
}

fn send_mouse_command(cmd: u8) {
    let mut command_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    wait_write();
    unsafe { command_port.write(0xD4u8) };
    wait_write();
    unsafe { data_port.write(cmd) };
}

fn read_mouse_ack() -> u8 {
    let mut data_port: Port<u8> = Port::new(0x60);
    wait_read();
    unsafe { data_port.read() }
}

fn wait_write() {
    let mut status_port: Port<u8> = Port::new(0x64);
    for _ in 0..100_000 {
        let status = unsafe { status_port.read() };
        if status & 0x02 == 0 {
            return;
        }
    }
}

fn wait_read() {
    let mut status_port: Port<u8> = Port::new(0x64);
    for _ in 0..100_000 {
        let status = unsafe { status_port.read() };
        if status & 0x01 != 0 {
            return;
        }
    }
}
