//! Baseline: the bump allocator measured on 17 and 19 October 2026.
//!
//! Same algorithm as POC/ferrum_Poc_adaptative_sheduller/src/allocator.rs at
//! commit c88bde4: the pointer only moves back when every allocation has been
//! freed. Kept so that the "before" measurement can be replayed at any time.

use crate::{align_up, Heap, HeapStats};
use core::alloc::Layout;
use core::ptr;

pub struct BumpHeap {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
    live_bytes: usize,
    total_allocations: u64,
    failed_allocations: u64,
}

impl BumpHeap {
    pub const fn new() -> Self {
        BumpHeap {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
            live_bytes: 0,
            total_allocations: 0,
            failed_allocations: 0,
        }
    }
}

impl Default for BumpHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap for BumpHeap {
    const NAME: &'static str = "bump";

    // No unsafe operation inside: `unsafe` comes from the `Heap` contract.
    unsafe fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.heap_end = start + size;
        self.next = start;
    }

    fn allocate(&mut self, layout: Layout) -> *mut u8 {
        let alloc_start = match align_up(self.next, layout.align()) {
            Some(start) => start,
            None => return ptr::null_mut(),
        };
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > self.heap_end {
            self.failed_allocations += 1;
            ptr::null_mut()
        } else {
            self.next = alloc_end;
            self.allocations += 1;
            self.live_bytes += layout.size();
            self.total_allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn deallocate(&mut self, _ptr: *mut u8, layout: Layout) {
        self.allocations = self.allocations.saturating_sub(1);
        self.live_bytes = self.live_bytes.saturating_sub(layout.size());
        if self.allocations == 0 {
            self.next = self.heap_start;
        }
    }

    fn stats(&self) -> HeapStats {
        let free = self.heap_end - self.next;
        HeapStats {
            heap_size: self.heap_end - self.heap_start,
            used: self.next - self.heap_start,
            free,
            live_bytes: self.live_bytes,
            largest_free_block: free,
            live_allocations: self.allocations,
            total_allocations: self.total_allocations,
            failed_allocations: self.failed_allocations,
            rejected_frees: 0,
        }
    }
}
