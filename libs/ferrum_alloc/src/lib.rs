//! Kernel heap allocators for FerrumOS.
//!
//! Three allocators share the [`Heap`] interface so that the same experiments
//! can be run against each of them, on the host (research/heap-allocator/bench)
//! and inside the kernel under QEMU (POC/ferrum_Poc_adaptative_sheduller):
//!
//! | Allocator              | Role                                                         |
//! |------------------------|--------------------------------------------------------------|
//! | [`BumpHeap`]           | baseline, identical to the allocator measured in October     |
//! | [`LinkedListHeap`]     | candidate A: address-ordered free list, first fit, coalescing |
//! | [`FixedBlockHeap`]     | candidate B: size classes 16..2048 B over a linked-list fallback |
//!
//! None of them take a lock: the kernel wraps the chosen one in a spin mutex.
//! See docs/objectif1/ for the protocol, the measurements and the decision.
#![cfg_attr(not(test), no_std)]

mod bump;
mod fixed_block;
mod linked_list;

pub use bump::BumpHeap;
pub use fixed_block::{FixedBlockHeap, BLOCK_SIZES};
pub use linked_list::LinkedListHeap;

use core::alloc::Layout;

/// Snapshot of a heap, comparable across allocators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeapStats {
    /// Size of the managed region in bytes.
    pub heap_size: usize,
    /// Bytes that cannot serve a new allocation (`heap_size - free`).
    /// For the bump allocator this is how far the pointer has advanced.
    pub used: usize,
    /// Bytes that can serve a new allocation of a suitable size.
    pub free: usize,
    /// Sum of the sizes requested by the allocations still alive.
    /// Unlike `used`, it does not include rounding, padding or lost memory.
    pub live_bytes: usize,
    /// Largest request (alignment 16) that can succeed right now.
    pub largest_free_block: usize,
    pub live_allocations: usize,
    pub total_allocations: u64,
    pub failed_allocations: u64,
    /// Frees rejected because they overlapped free memory (double free,
    /// wrong layout). Always 0 for the bump allocator, which cannot tell.
    pub rejected_frees: u64,
}

impl HeapStats {
    /// External fragmentation in per mille: `1 - largest_free_block / free`.
    /// 0 means all the free memory is in one piece.
    pub fn fragmentation_permille(&self) -> u32 {
        if self.free == 0 {
            return 0;
        }
        let largest = self.largest_free_block.min(self.free);
        (1000 - (largest as u128 * 1000 / self.free as u128)) as u32
    }
}

/// Common interface of the allocators. Not thread safe: callers serialize.
pub trait Heap {
    /// Short name used in measurement records.
    const NAME: &'static str;

    /// Hand the region `start..start + size` to the allocator.
    ///
    /// # Safety
    /// The region must be valid, writable, unused by anything else for as
    /// long as the allocator lives, and `init` must be called only once.
    /// `start` must be 16-byte aligned.
    unsafe fn init(&mut self, start: usize, size: usize);

    /// Returns a pointer to a block that fits `layout`, or null when the
    /// request cannot be served. Never panics.
    fn allocate(&mut self, layout: Layout) -> *mut u8;

    /// Gives a block back.
    ///
    /// # Safety
    /// `ptr` must come from `allocate` on this heap with the same `layout`
    /// and must not have been freed already.
    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout);

    fn stats(&self) -> HeapStats;
}

/// Rounds `addr` up to `align`, which must be a power of two.
/// Returns `None` on overflow instead of wrapping.
#[inline]
pub fn align_up(addr: usize, align: usize) -> Option<usize> {
    debug_assert!(align.is_power_of_two());
    Some(addr.checked_add(align - 1)? & !(align - 1))
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::alloc::{alloc, dealloc, Layout};

    /// A page-aligned region on the host, so that alignment behaves as in
    /// the kernel, where the heap starts at 0x4444_4444_0000.
    pub struct Region {
        pub start: usize,
        pub size: usize,
        layout: Layout,
    }

    impl Region {
        pub fn new(size: usize) -> Self {
            let layout = Layout::from_size_align(size, 4096).unwrap();
            let start = unsafe { alloc(layout) } as usize;
            assert_ne!(start, 0);
            Region {
                start,
                size,
                layout,
            }
        }
    }

    impl Drop for Region {
        fn drop(&mut self) {
            unsafe { dealloc(self.start as *mut u8, self.layout) }
        }
    }
}
