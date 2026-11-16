//! FerrumOS: a modular operating system written in Rust.
//!
//! The source tree separates portable kernel policies, architecture-specific
//! mechanisms, drivers and higher-level services. Modules currently describe
//! their responsibilities; runtime behavior and a bootable entry point will
//! be introduced as the existing prototypes are integrated.

#![no_std]

pub mod arch;
pub mod boot;
pub mod drivers;
pub mod kernel;
pub mod programs;
pub mod services;
pub mod ui;
