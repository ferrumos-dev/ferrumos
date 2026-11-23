//! Portable kernel policies, execution state and reporting contracts.
//!
//! Scheduling decisions and resource ownership belong here. Raw hardware
//! access and CPU context-switch mechanisms belong in [`crate::arch`], and
//! device access belongs in [`crate::drivers`].
//!
//! These modules must remain free of unsafe operations and hardware-specific
//! types. Their behavioral tests run on the host; boot integration exercises
//! the same scheduler and executor in the bare-metal image.

#![forbid(unsafe_code)]

pub mod executor;
pub mod interrupts;
pub mod memory;
pub mod scheduler;
pub mod telemetry;
