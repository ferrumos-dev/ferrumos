//! Boot-CPU interrupt table, legacy PIC routing and periodic PIT timer.
//!
//! Initialization owns the IDT and both legacy controllers. Only IRQ0 is
//! unmasked. Its interrupt gate disables nesting, preserves the interrupted
//! registers and records one delivered tick before acknowledging the master
//! PIC. The handler never allocates, takes a lock or touches the boot console.
//!
//! Exception entries report a terminal diagnostic through the emergency UART
//! path. They use the current kernel stack. A dedicated double-fault stack,
//! userspace transitions and multiprocessor routing require additional setup.
//! The entry code supports the root target's default no-AVX configuration.

/// Nominal input frequency of the PC-compatible channel-0 PIT, in hertz.
pub const PIT_INPUT_HZ: u32 = 1_193_182;

/// Closest integral PIT divisor to a nominal 100 Hz periodic event rate.
pub const PIT_DIVISOR: u16 = 11_932;

#[cfg(target_os = "none")]
mod hardware;

#[cfg(target_os = "none")]
pub use hardware::{clock_snapshot, initialize_timer, tick_count, TimerInitError};
