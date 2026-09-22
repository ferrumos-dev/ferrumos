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

struct Runtime {
    packet: [u8; 3],
    index: usize,
    max_x: usize,
    max_y: usize,
    state: MouseState,
}

impl Runtime {
    const fn new() -> Self {
        Self {
            packet: [0; 3],
            index: 0,
            max_x: 319,
            max_y: 199,
            state: MouseState::new(160, 100),
        }
    }

    fn clamp_position(&mut self) {
        self.state.x = self.state.x.min(self.max_x);
        self.state.y = self.state.y.min(self.max_y);
    }

    fn handle_byte(&mut self, byte: u8) -> Option<MouseState> {
        // Packet synchronization: first byte must always have bit 3 set.
        if self.index == 0 && (byte & 0x08) == 0 {
            return None;
        }

        self.packet[self.index] = byte;
        self.index += 1;

        if self.index < 3 {
            return None;
        }

        self.index = 0;

        let flags = self.packet[0];

        let dx = self.packet[1] as i8 as i16;
        let dy = self.packet[2] as i8 as i16;

        let next_x = self.state.x as i16 + dx;
        let next_y = self.state.y as i16 - dy;

        self.state.x = next_x.clamp(0, self.max_x as i16) as usize;
        self.state.y = next_y.clamp(0, self.max_y as i16) as usize;

        let left = flags & 0x01 != 0;
        self.state.left_clicked = left && !self.state.left_down;
        self.state.left_down = left;

        Some(self.state)
    }
}

static RUNTIME: Mutex<Runtime> = Mutex::new(Runtime::new());

pub fn init(width: usize, height: usize) {
    {
        let mut rt = RUNTIME.lock();
        rt.max_x = width.saturating_sub(1);
        rt.max_y = height.saturating_sub(1);
        rt.state.x = width / 2;
        rt.state.y = height / 2;
        rt.clamp_position();
    }

    let mut command_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    wait_write();
    unsafe { command_port.write(0xA8u8) };

    wait_write();
    unsafe { command_port.write(0x20u8) };
    wait_read();
    let status: u8 = unsafe { data_port.read() };
    let updated = (status | 0x02) & !0x20;

    wait_write();
    unsafe { command_port.write(0x60u8) };
    wait_write();
    unsafe { data_port.write(updated) };

    send_mouse(0xF6);
    let _ = read_ack();

    send_mouse(0xF4);
    let _ = read_ack();
}

pub fn current() -> MouseState {
    RUNTIME.lock().state
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
    let mut rt = RUNTIME.lock();
    rt.handle_byte(byte)
}

fn send_mouse(cmd: u8) {
    let mut command_port: Port<u8> = Port::new(0x64);
    let mut data_port: Port<u8> = Port::new(0x60);

    wait_write();
    unsafe { command_port.write(0xD4u8) };
    wait_write();
    unsafe { data_port.write(cmd) };
}

fn read_ack() -> u8 {
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
