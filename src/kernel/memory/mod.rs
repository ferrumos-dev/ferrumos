//! Portable memory reporting contracts.
//!
//! [`MemorySnapshot`] carries one coherent view of the kernel heap and
//! [`MemoryReporter`] is the source that produces it. Measurements use bytes.
//! An unavailable measurement stays distinct from zero: the reporter returns
//! [`None`] instead of a snapshot full of zeroes, and the constructor refuses
//! a usage greater than the capacity. A snapshot of `(0, 0)` is valid and
//! describes a heap that exists and holds nothing.
//!
//! These types hold no hardware state and take no lock. Page tables, physical
//! frames, heap allocation and synchronization belong to the hardware layer,
//! which captures capacity and usage inside its own critical section and hands
//! the value to consumers once that section is over.
//!
//! Signatures follow `contrats_noyau.md` and the `ferrumos-kernel-core`
//! reference in `foundation/`.

#![forbid(unsafe_code)]

/// Coherent view of the kernel heap at a single instant, in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemorySnapshot {
    capacity_bytes: u64,
    used_bytes: u64,
}

impl MemorySnapshot {
    /// Builds a snapshot from a capacity and a usage measured together.
    ///
    /// Returns [`None`] when `used` is greater than `capacity`, because such a
    /// pair describes an incoherent measurement. The caller then reports the
    /// heap as unavailable rather than publishing a wrong value.
    pub const fn new(capacity: u64, used: u64) -> Option<Self> {
        if used > capacity {
            return None;
        }
        Some(Self {
            capacity_bytes: capacity,
            used_bytes: used,
        })
    }

    /// Total size of the heap, in bytes.
    pub const fn heap_capacity_bytes(&self) -> u64 {
        self.capacity_bytes
    }

    /// Bytes currently held by the heap, in bytes.
    pub const fn heap_used_bytes(&self) -> u64 {
        self.used_bytes
    }

    /// Bytes still available in the heap, in bytes.
    ///
    /// The subtraction cannot underflow: [`MemorySnapshot::new`] rejects a
    /// usage greater than the capacity.
    pub const fn heap_free_bytes(&self) -> u64 {
        self.capacity_bytes - self.used_bytes
    }

    /// Share of the heap currently used, in parts per thousand.
    ///
    /// [`None`] when the capacity is zero, because no share is defined then.
    pub const fn usage_permille(&self) -> Option<u16> {
        if self.capacity_bytes == 0 {
            return None;
        }
        let used = self.used_bytes as u128;
        let capacity = self.capacity_bytes as u128;
        Some((used * 1000 / capacity) as u16)
    }
}

/// Source of heap measurements.
///
/// The implementer owns the synchronization: it reads capacity and usage
/// coherently under the protections of the allocator, then returns a plain
/// value that consumers read afterwards.
pub trait MemoryReporter {
    /// Current snapshot of the heap.
    ///
    /// Returns [`None`] while no coherent measurement is available, for
    /// instance before the heap is initialized. Consumers must render that
    /// absence as unavailable and never as zero.
    fn memory_snapshot(&self) -> Option<MemorySnapshot>;
}

#[cfg(test)]
mod tests;
