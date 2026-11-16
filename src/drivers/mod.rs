//! Hardware device drivers, grouped by device responsibility.
//!
//! Hardware details remain in this layer. Portable kernel and service code
//! will consume explicit device interfaces with defined ownership and
//! synchronization. These modules currently reserve the integration points.
//!
//! Network protocols belong in [`crate::services::network`], and filesystems
//! belong in [`crate::services::filesystem`], above the device drivers.

pub mod display;
pub mod input;
pub mod network;
pub mod serial;
pub mod storage;
