//! x86_64 mechanisms and integration points for the FerrumOS kernel.
//!
//! Port I/O and CPU interrupt masking support the polled boot console. Memory
//! management, interrupt handlers and context switching remain separate tasks.
//! Unsafe operations document their preconditions and stay behind interfaces
//! with explicit ownership and synchronization.

pub mod cpu;
pub mod interrupts;
#[cfg(target_os = "none")]
pub mod io;
pub mod memory;
