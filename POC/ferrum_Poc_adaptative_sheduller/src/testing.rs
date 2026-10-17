use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
    loop {
        x86_64::instructions::hlt();
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        crate::serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        crate::serial_println!("[ok]");
    }
}

pub struct Rng {
    state: u64,
}

impl Rng {
    pub const fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_in_range(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_u64() as usize) % (max - min + 1)
    }

    pub fn next_align(&mut self) -> usize {
        1 << (self.next_u64() % 7)
    }
}

pub struct Record {
    pub name: &'static str,
    pub iterations: u64,
    pub errors: u64,
    pub used_before: usize,
    pub used_after: usize,
    pub cycles: u64,
}

impl Record {
    pub fn report(&self) {
        crate::serial_println!(
            "\n\tRECORD name={} iterations={} errors={} used_before={} used_after={} cycles={}",
            self.name,
            self.iterations,
            self.errors,
            self.used_before,
            self.used_after,
            self.cycles
        );
    }
}
