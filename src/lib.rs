//! FerrumOS: a modular operating system written in Rust.
//!
//! The source tree separates portable kernel policies, architecture-specific
//! mechanisms, drivers and higher-level services. The boot path initializes
//! a framebuffer console, serial diagnostics and polled PS/2 input. A portable
//! priority scheduler and cooperative executor dispatch bounded kernel tasks.
//! Memory observations and event counters have explicit ownership contracts.

#![no_std]

pub mod arch;
pub mod boot;
pub mod drivers;
pub mod kernel;
pub mod programs;
pub mod services;
pub mod ui;
