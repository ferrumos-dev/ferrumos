//! Validated memory measurements for portable kernel consumers.
//!
//! The allocator adapter owns the measurements and their synchronization. This
//! module validates a sample before the shell or monitor sees it. All values
//! use bytes, and an unavailable measurement remains distinct from a valid zero.
//!
//! Capturing a sample does not allocate memory or change page mappings.

/// A coherent heap sample whose used bytes never exceed its capacity.
///
/// The private fields preserve the invariant after construction, so computing
/// free bytes cannot underflow. The provider must take both values from the
/// same allocator state, under its existing synchronization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemorySnapshot {
    heap_capacity_bytes: u64,
    heap_used_bytes: u64,
}

impl MemorySnapshot {
    /// Validates a heap sample measured in bytes.
    ///
    /// Returns `None` if `heap_used_bytes` exceeds `heap_capacity_bytes`.
    /// A capacity and usage of zero form a valid empty sample.
    pub const fn new(heap_capacity_bytes: u64, heap_used_bytes: u64) -> Option<Self> {
        if heap_used_bytes > heap_capacity_bytes {
            return None;
        }
        Some(Self {
            heap_capacity_bytes,
            heap_used_bytes,
        })
    }

    /// Returns the sampled heap capacity in bytes.
    pub const fn heap_capacity_bytes(self) -> u64 {
        self.heap_capacity_bytes
    }

    /// Returns the sampled heap usage in bytes.
    pub const fn heap_used_bytes(self) -> u64 {
        self.heap_used_bytes
    }

    /// Returns sampled free bytes, using the invariant checked by [`Self::new`].
    pub const fn heap_free_bytes(self) -> u64 {
        self.heap_capacity_bytes - self.heap_used_bytes
    }
}

/// Read-only boundary between an allocator adapter and its consumers.
///
/// Implementations own coherent sampling and preserve their allocator's locking
/// and interrupt invariants. An unavailable or inconsistent sample returns
/// `None`; a reported zero means the provider actually measured zero.
pub trait MemoryReporter {
    /// Captures one coherent heap sample, or reports that it is unavailable.
    fn memory_snapshot(&self) -> Option<MemorySnapshot>;
}

#[cfg(test)]
mod tests;
