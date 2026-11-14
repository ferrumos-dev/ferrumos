//! Candidate B: fixed-size blocks over a linked-list fallback.
//!
//! Requests up to 2 KiB are rounded to a size class (16, 32, ..., 2048 B).
//! A freed block goes to the head of its class list and is reused by the
//! next request of the same class in O(1). Larger requests, and classes
//! whose list is empty, go to a [`LinkedListHeap`] fallback.
//!
//! Design taken from Philipp Oppermann, "Writing an OS in Rust",
//! chapter "Allocator Designs" (fixed-size block allocator), with two changes
//! made for FerrumOS:
//! - classes start at 16 B, the fallback granularity, so that no byte is
//!   lost when a block moves between the fallback and a class list;
//! - when the fallback cannot serve a request, every block parked in the
//!   class lists is handed back to the fallback, which merges them, and the
//!   request is retried once. Without this, memory freed as small blocks can
//!   never serve a larger request ("phase change" scenario, see
//!   docs/greenlights/objecitf_obligatoire_1/tas_noyau/). The reclaim can be switched off to measure its effect.

use crate::linked_list::{LinkedListHeap, GRANULE};
use crate::{Heap, HeapStats};
use core::alloc::Layout;
use core::ptr;

/// Size classes. Each is also the alignment of its blocks.
pub const BLOCK_SIZES: [usize; 8] = [16, 32, 64, 128, 256, 512, 1024, 2048];

const _: () = assert!(BLOCK_SIZES[0] == GRANULE);

struct FreeBlock {
    next: *mut FreeBlock,
}

pub struct FixedBlockHeap {
    lists: [*mut FreeBlock; BLOCK_SIZES.len()],
    parked: [usize; BLOCK_SIZES.len()],
    fallback: LinkedListHeap,
    reclaim: bool,
    reclaims: u64,
    live_bytes: usize,
    live_allocations: usize,
    total_allocations: u64,
    failed_allocations: u64,
}

// Same reasoning as for LinkedListHeap: pointers stay inside the owned region.
unsafe impl Send for FixedBlockHeap {}

impl FixedBlockHeap {
    /// Fixed-size blocks with reclaim on exhaustion (the integrated version).
    pub const fn new() -> Self {
        Self::with_reclaim(true)
    }

    /// `reclaim = false` gives the textbook design, kept for comparison.
    pub const fn with_reclaim(reclaim: bool) -> Self {
        FixedBlockHeap {
            lists: [ptr::null_mut(); BLOCK_SIZES.len()],
            parked: [0; BLOCK_SIZES.len()],
            fallback: LinkedListHeap::new(),
            reclaim,
            reclaims: 0,
            live_bytes: 0,
            live_allocations: 0,
            total_allocations: 0,
            failed_allocations: 0,
        }
    }

    /// How many times the class lists were handed back to the fallback.
    pub fn reclaims(&self) -> u64 {
        self.reclaims
    }

    fn class_of(layout: Layout) -> Option<usize> {
        let needed = layout.size().max(layout.align());
        BLOCK_SIZES.iter().position(|&size| size >= needed)
    }

    fn parked_bytes(&self) -> usize {
        self.parked
            .iter()
            .zip(BLOCK_SIZES.iter())
            .map(|(count, size)| count * size)
            .sum()
    }

    /// Hands every parked block back to the fallback, which merges them.
    fn drain_lists(&mut self) {
        for (class, &size) in BLOCK_SIZES.iter().enumerate() {
            while let Some(block) = unsafe { self.lists[class].as_mut() } {
                self.lists[class] = block.next;
                self.parked[class] -= 1;
                let returned =
                    unsafe { self.fallback.deallocate_raw(block as *mut _ as usize, size) };
                debug_assert!(returned, "a parked block overlapped free memory");
            }
        }
        self.reclaims += 1;
    }

    fn allocate_inner(&mut self, layout: Layout) -> *mut u8 {
        match Self::class_of(layout) {
            Some(class) => {
                if let Some(block) = unsafe { self.lists[class].as_mut() } {
                    self.lists[class] = block.next;
                    self.parked[class] -= 1;
                    return block as *mut FreeBlock as *mut u8;
                }
                let size = BLOCK_SIZES[class];
                self.fallback.allocate_raw(size, size)
            }
            None => match LinkedListHeap::normalize(layout) {
                Some((size, align)) => self.fallback.allocate_raw(size, align),
                None => ptr::null_mut(),
            },
        }
    }
}

impl Default for FixedBlockHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap for FixedBlockHeap {
    const NAME: &'static str = "fixed_block";

    unsafe fn init(&mut self, start: usize, size: usize) {
        self.fallback.init(start, size);
    }

    fn allocate(&mut self, layout: Layout) -> *mut u8 {
        let mut block = self.allocate_inner(layout);
        if block.is_null() && self.reclaim && self.parked_bytes() > 0 {
            self.drain_lists();
            block = self.allocate_inner(layout);
        }

        if block.is_null() {
            self.failed_allocations += 1;
        } else {
            self.live_allocations += 1;
            self.live_bytes += layout.size();
            self.total_allocations += 1;
        }
        block
    }

    unsafe fn deallocate(&mut self, ptr: *mut u8, layout: Layout) {
        match Self::class_of(layout) {
            Some(class) => {
                let block = ptr as *mut FreeBlock;
                block.write(FreeBlock {
                    next: self.lists[class],
                });
                self.lists[class] = block;
                self.parked[class] += 1;
            }
            None => {
                if let Some((size, _)) = LinkedListHeap::normalize(layout) {
                    self.fallback.deallocate_raw(ptr as usize, size);
                }
            }
        }
        self.live_allocations = self.live_allocations.saturating_sub(1);
        self.live_bytes = self.live_bytes.saturating_sub(layout.size());
    }

    fn stats(&self) -> HeapStats {
        let heap_size = self.fallback.stats().heap_size;
        let free = self.fallback.free_bytes() + self.parked_bytes();
        let largest_parked = BLOCK_SIZES
            .iter()
            .zip(self.parked.iter())
            .filter(|(_, &count)| count > 0)
            .map(|(&size, _)| size)
            .max()
            .unwrap_or(0);
        HeapStats {
            heap_size,
            used: heap_size - free,
            free,
            live_bytes: self.live_bytes,
            largest_free_block: self.fallback.largest_free().max(largest_parked),
            live_allocations: self.live_allocations,
            total_allocations: self.total_allocations,
            failed_allocations: self.failed_allocations,
            // Class lists cannot detect a double free (no check, O(1) push).
            rejected_frees: self.fallback.stats().rejected_frees,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Region;

    fn heap(region: &Region, reclaim: bool) -> FixedBlockHeap {
        let mut heap = FixedBlockHeap::with_reclaim(reclaim);
        unsafe { heap.init(region.start, region.size) };
        heap
    }

    /// Fill with small blocks, free them all, then ask for 1 KiB blocks.
    fn phase_change(heap: &mut FixedBlockHeap) -> usize {
        let small = Layout::from_size_align(64, 8).unwrap();
        let mut blocks = Vec::new();
        loop {
            let p = heap.allocate(small);
            if p.is_null() {
                break;
            }
            blocks.push(p);
        }
        for p in blocks {
            unsafe { heap.deallocate(p, small) };
        }
        let large = Layout::from_size_align(1024, 8).unwrap();
        let mut count = 0;
        while !heap.allocate(large).is_null() {
            count += 1;
        }
        count
    }

    #[test]
    fn textbook_design_cannot_change_phase() {
        let region = Region::new(100 * 1024);
        assert_eq!(phase_change(&mut heap(&region, false)), 0);
    }

    #[test]
    fn reclaim_restores_full_capacity_after_phase_change() {
        let region = Region::new(100 * 1024);
        let mut heap = heap(&region, true);
        assert_eq!(phase_change(&mut heap), 100);
        assert_eq!(heap.reclaims(), 1);
    }

    #[test]
    fn same_class_reuse_and_stats() {
        let region = Region::new(100 * 1024);
        let mut heap = heap(&region, true);
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let first = heap.allocate(layout);
        unsafe { heap.deallocate(first, layout) };
        let second = heap.allocate(layout);
        assert_eq!(first, second, "freed block reused in O(1)");
        unsafe { heap.deallocate(second, layout) };
        let stats = heap.stats();
        assert_eq!(stats.used, 0);
        assert_eq!(stats.live_allocations, 0);
    }

    #[test]
    fn large_requests_use_the_fallback() {
        let region = Region::new(100 * 1024);
        let mut heap = heap(&region, true);
        let layout = Layout::from_size_align(8192, 4096).unwrap();
        let p = heap.allocate(layout);
        assert!(!p.is_null());
        assert_eq!(p as usize % 4096, 0);
        unsafe { heap.deallocate(p, layout) };
        assert_eq!(heap.stats().free, 100 * 1024);
    }
}
