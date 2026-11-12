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
        self.report_with(&[]);
    }

    /// Same line, followed by extra `key=value` fields (parsed by
    /// scripts/kernel_tests/results.py like the standard ones).
    pub fn report_with(&self, extra: &[(&str, u64)]) {
        crate::serial_print!(
            "\n\tRECORD name={} iterations={} errors={} used_before={} used_after={} cycles={}",
            self.name,
            self.iterations,
            self.errors,
            self.used_before,
            self.used_after,
            self.cycles
        );
        for (key, value) in extra {
            crate::serial_print!(" {}={}", key, value);
        }
        crate::serial_println!();
    }
}
