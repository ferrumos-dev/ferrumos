//! Hardware device drivers, grouped by device responsibility.
//!
//! Hardware details remain in this layer. Portable kernel and service code
//! will consume explicit device interfaces with defined ownership and
//! synchronization. Serial output, framebuffer text and PS/2 input support the
//! boot console; storage and network drivers remain integration points.
//!
//! Network protocols belong in [`crate::services::network`], and filesystems
//! belong in [`crate::services::filesystem`], above the device drivers.

pub mod console;
pub mod display;
pub mod input;
pub mod network;
pub mod serial;
pub mod storage;
