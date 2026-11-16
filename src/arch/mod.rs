//! Architecture-specific kernel mechanisms.
//!
//! Portable scheduling and reporting rules remain in [`crate::kernel`].

#[cfg(target_arch = "x86_64")]
pub mod x86_64;
