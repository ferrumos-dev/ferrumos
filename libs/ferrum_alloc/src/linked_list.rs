//! Candidate A: free list sorted by address, first fit, coalescing on free.
//!
//! Each free region ("hole") stores its size and the address of the next hole
//! in its own first 16 bytes, so the allocator needs no memory of its own and
//! allocated blocks carry no header.
//!
//! Invariant: every hole starts on a 16-byte boundary and its size is a
//! non-zero multiple of 16. Requests are rounded up to 16 bytes and to an
//! alignment of at least 16, so splitting a hole always leaves holes that
//! respect the invariant and no byte is ever lost to rounding.
//!
//! Costs: allocation and free walk the list, O(number of holes). Coalescing
//! keeps that number low; the measurements are in docs/objectif1/.

use crate::{align_up, Heap, HeapStats};
use core::alloc::Layout;
use core::mem;
use core::ptr;

/// Granularity of every block and hole.
pub(crate) const GRANULE: usize = 16;

#[repr(C)]
struct Hole {
    size: usize,
    next: *mut Hole,
}

const _: () = assert!(mem::size_of::<Hole>() <= GRANULE);

pub struct LinkedListHeap {
    /// Sentinel: `head.next` is the hole with the lowest address.
    head: Hole,
    heap_start: usize,
    heap_size: usize,
    free: usize,
    live_bytes: usize,
    live_allocations: usize,
    total_allocations: u64,
    failed_allocations: u64,
    rejected_frees: u64,
}

// The raw pointers only ever point into the region handed to `init`, which
// the caller guarantees is exclusively owned by this allocator.
unsafe impl Send for LinkedListHeap {}

impl LinkedListHeap {
    pub const fn new() -> Self {
        LinkedListHeap {
            head: Hole {
                size: 0,
                next: ptr::null_mut(),
            },
            heap_start: 0,
            heap_size: 0,
            free: 0,
            live_bytes: 0,
            live_allocations: 0,
            total_allocations: 0,
            failed_allocations: 0,
            rejected_frees: 0,
        }
    }

    /// Size and alignment actually reserved for `layout`.
    /// `None` if rounding overflows.
    pub(crate) fn normalize(layout: Layout) -> Option<(usize, usize)> {
        let size = align_up(layout.size().max(1), GRANULE)?;
        let align = layout.align().max(GRANULE);
        Some((size, align))
    }

    /// Number of holes, for tests and statistics.
    pub fn hole_count(&self) -> usize {
        let mut count = 0;
        let mut cursor = self.head.next;
        while !cursor.is_null() {
            count += 1;
            cursor = unsafe { (*cursor).next };
        }
        count
    }

    fn largest_hole(&self) -> usize {
        let mut largest = 0;
        let mut cursor = self.head.next;
        while !cursor.is_null() {
            unsafe {
                largest = largest.max((*cursor).size);
                cursor = (*cursor).next;
            }
        }
        largest
    }

    /// Reserves `size` bytes aligned on `align` (both already normalized).
    /// Used directly by the fixed-size block allocator.
    pub(crate) fn allocate_raw(&mut self, size: usize, align: usize) -> *mut u8 {
        let mut prev: *mut Hole = &mut self.head;
        unsafe {
            loop {
                let hole = (*prev).next;
                if hole.is_null() {
                    return ptr::null_mut();
                }

                let hole_start = hole as usize;
                let hole_end = hole_start + (*hole).size;
                let fits = align_up(hole_start, align)
                    .and_then(|start| Some((start, start.checked_add(size)?)))
                    .filter(|&(_, end)| end <= hole_end);

                if let Some((block_start, block_end)) = fits {
                    let front = block_start - hole_start;
                    let back = hole_end - block_end;
                    let after = (*hole).next;

                    // What follows the block: a new hole for the tail, or the next hole.
                    let following = if back > 0 {
                        let tail = block_end as *mut Hole;
                        tail.write(Hole {
                            size: back,
                            next: after,
                        });
                        tail
                    } else {
                        after
                    };

                    if front > 0 {
                        // The alignment padding stays a hole: nothing is lost.
                        (*hole).size = front;
                        (*hole).next = following;
                    } else {
                        (*prev).next = following;
                    }

                    self.free -= size;
                    return block_start as *mut u8;
                }

                prev = hole;
            }
        }
    }

    /// Returns `size` bytes at `addr` (already normalized) to the free list,
    /// merging with the neighbouring holes. Returns false, and changes
    /// nothing, if the block overlaps memory that is already free.
    pub(crate) unsafe fn deallocate_raw(&mut self, addr: usize, size: usize) -> bool {
        let heap_end = self.heap_start + self.heap_size;
        if addr < self.heap_start
            || !addr.is_multiple_of(GRANULE)
            || addr.checked_add(size).is_none_or(|end| end > heap_end)
        {
            return false;
        }

        let head: *mut Hole = &mut self.head;
        let mut prev = head;
        while !(*prev).next.is_null() && ((*prev).next as usize) < addr {
            prev = (*prev).next;
        }
        let next = (*prev).next;

        let prev_end = if prev == head {
            0
        } else {
            prev as usize + (*prev).size
        };
        if prev_end > addr || (!next.is_null() && addr + size > next as usize) {
            return false;
        }

        let block = addr as *mut Hole;
        block.write(Hole { size, next });

        if !next.is_null() && addr + size == next as usize {
            (*block).size += (*next).size;
            (*block).next = (*next).next;
        }

        if prev != head && prev_end == addr {
            (*prev).size += (*block).size;
            (*prev).next = (*block).next;
        } else {
            (*prev).next = block;
        }

        self.free += size;
        true
    }

    pub(crate) fn free_bytes(&self) -> usize {
        self.free
    }

    pub(crate) fn largest_free(&self) -> usize {
        self.largest_hole()
    }
}

impl Default for LinkedListHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heap for LinkedListHeap {
    const NAME: &'static str = "linked_list";

    unsafe fn init(&mut self, start: usize, size: usize) {
        let aligned_start = align_up(start, GRANULE).expect("heap start overflows");
        let usable = (start + size).saturating_sub(aligned_start) & !(GRANULE - 1);
        self.heap_start = aligned_start;
        self.heap_size = usable;
        self.free = 0;
        if usable >= GRANULE {
            self.head.next = ptr::null_mut();
            self.deallocate_raw(aligned_start, usable);
        }
    }

    fn allocate(&mut self, layout: Layout) -> *mut u8 {
        let block = match Self::normalize(layout) {
            Some((size, align)) => self.allocate_raw(size, align),
            None => ptr::null_mut(),
        };
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
        let accepted = match Self::normalize(layout) {
            Some((size, _)) => self.deallocate_raw(ptr as usize, size),
            None => false,
        };
        if accepted {
            self.live_allocations = self.live_allocations.saturating_sub(1);
            self.live_bytes = self.live_bytes.saturating_sub(layout.size());
        } else {
            self.rejected_frees += 1;
        }
    }

    fn stats(&self) -> HeapStats {
        HeapStats {
            heap_size: self.heap_size,
            used: self.heap_size - self.free,
            free: self.free,
            live_bytes: self.live_bytes,
            largest_free_block: self.largest_hole(),
            live_allocations: self.live_allocations,
            total_allocations: self.total_allocations,
            failed_allocations: self.failed_allocations,
            rejected_frees: self.rejected_frees,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::Region;

    fn heap(region: &Region) -> LinkedListHeap {
        let mut heap = LinkedListHeap::new();
        unsafe { heap.init(region.start, region.size) };
        heap
    }

    #[test]
    fn freed_memory_is_reused() {
        let region = Region::new(100 * 1024);
        let mut heap = heap(&region);
        let small = Layout::from_size_align(64, 8).unwrap();
        let keep: Vec<_> = (0..4).map(|_| heap.allocate(small)).collect();
        let churn = Layout::from_size_align(1024, 8).unwrap();
        for _ in 0..10_000 {
            let p = heap.allocate(churn);
            assert!(!p.is_null());
            unsafe { heap.deallocate(p, churn) };
        }
        for p in keep {
            unsafe { heap.deallocate(p, small) };
        }
        let stats = heap.stats();
        assert_eq!(stats.used, 0);
        assert_eq!(heap.hole_count(), 1, "everything merged back");
    }

    #[test]
    fn alignment_padding_is_not_lost() {
        let region = Region::new(8192);
        let mut heap = heap(&region);
        let a = Layout::from_size_align(16, 16).unwrap();
        let b = Layout::from_size_align(100, 1024).unwrap();
        let pa = heap.allocate(a);
        let pb = heap.allocate(b);
        assert_eq!(pb as usize % 1024, 0);
        unsafe {
            heap.deallocate(pa, a);
            heap.deallocate(pb, b);
        }
        assert_eq!(heap.stats().free, 8192);
        assert_eq!(heap.hole_count(), 1);
    }

    #[test]
    fn exhaustion_returns_null_and_counts() {
        let region = Region::new(4096);
        let mut heap = heap(&region);
        let big = Layout::from_size_align(8192, 8).unwrap();
        assert!(heap.allocate(big).is_null());
        assert_eq!(heap.stats().failed_allocations, 1);
    }

    #[test]
    fn double_free_is_rejected() {
        let region = Region::new(4096);
        let mut heap = heap(&region);
        let layout = Layout::from_size_align(64, 8).unwrap();
        let p = heap.allocate(layout);
        unsafe {
            heap.deallocate(p, layout);
            heap.deallocate(p, layout);
        }
        let stats = heap.stats();
        assert_eq!(stats.rejected_frees, 1);
        assert_eq!(stats.free, 4096);
    }

    #[test]
    fn capacity_without_headers() {
        let region = Region::new(100 * 1024);
        let mut heap = heap(&region);
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let mut count = 0;
        while !heap.allocate(layout).is_null() {
            count += 1;
        }
        assert_eq!(count, 100);
    }
}
