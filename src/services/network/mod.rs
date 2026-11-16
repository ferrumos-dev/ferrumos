//! Network protocols and system-facing network services.
//!
//! This layer will consume frames through [`crate::drivers::network`].
//! Device ownership and interrupt handling stay in the hardware layers.
