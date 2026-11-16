use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{compiler_fence, Ordering};

pub fn cycles() -> u64 {
    compiler_fence(Ordering::SeqCst);
    // SAFETY: `rdtsc` is available on every x86_64 CPU and only reads the
    // time stamp counter.
    let value = unsafe { _rdtsc() };
    compiler_fence(Ordering::SeqCst);
    value
}

pub struct Stopwatch {
    start: u64,
}

impl Stopwatch {
    pub fn start() -> Self {
        Stopwatch { start: cycles() }
    }

    pub fn elapsed(&self) -> u64 {
        cycles().saturating_sub(self.start)
    }
}

pub fn measure<R>(f: impl FnOnce() -> R) -> (R, u64) {
    let watch = Stopwatch::start();
    let result = f();
    (result, watch.elapsed())
}
