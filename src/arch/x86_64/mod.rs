//! Reserved x86_64 adapters for the FerrumOS kernel.
//!
//! These modules contain no register access or initialization code yet.
//! Future unsafe operations must document their preconditions and remain
//! behind small interfaces with explicit ownership and synchronization.

pub mod cpu;
pub mod interrupts;
pub mod memory;
