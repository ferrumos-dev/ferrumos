//! System interaction and presentation.
//!
//! The shell and monitoring components will consume kernel snapshots and
//! service interfaces. Display and input hardware belong in [`crate::drivers`].

pub mod monitor;
pub mod shell;
