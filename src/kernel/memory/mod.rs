//! Reserved for portable memory reporting contracts.
//!
//! The documented `MemorySnapshot` and `MemoryReporter` will live here.
//! Measurements use bytes; unavailable data must remain distinct from zero,
//! and a snapshot must reject usage greater than capacity.
//!
//! No snapshot type or reporter is implemented yet. Page tables, physical
//! frames, heap allocation and synchronization belong to the hardware layer.
