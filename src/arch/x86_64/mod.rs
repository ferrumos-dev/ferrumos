//! x86_64 mechanisms and integration points for the FerrumOS kernel.
//!
//! Port I/O and CPU interrupt guards support the boot devices and heap lock.
//! Validated physical frames back the mapped kernel heap. IDT and PIC/PIT setup
//! deliver periodic timer observations; CPU context switching remains separate.
//! Unsafe operations document their preconditions and stay behind interfaces
//! with explicit ownership and synchronization.

pub mod cpu;
pub mod interrupts;
#[cfg(target_os = "none")]
pub mod io;
pub mod memory;
