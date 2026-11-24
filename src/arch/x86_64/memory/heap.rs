//! Locked ownership around the project's reusable linked-list allocator.

use core::alloc::Layout;
use core::ptr::NonNull;
use ferrum_alloc::{Heap, HeapStats, LinkedListHeap};
use spin::Mutex;

#[cfg(any(test, target_os = "none"))]
use super::MemoryInitError;
use crate::kernel::memory::{MemoryReporter, MemorySnapshot};

#[derive(Clone, Copy, Eq, PartialEq)]
enum State {
    #[cfg(any(test, target_os = "none"))]
    Uninitialized,
    Ready,
    #[cfg(any(test, target_os = "none"))]
    Failed,
}

struct HeapState {
    allocator: LinkedListHeap,
    state: State,
}

/// A mapped heap whose metadata and observations share the same lock.
///
/// The architecture adapter initializes the heap once. Each allocation owns a
/// distinct block until its handle is dropped. Interrupt masking surrounds each
/// lock on bare metal, preventing a same-CPU IRQ from spinning on an interrupted
/// owner. Mapping setup still requires the documented single-CPU boot contract.
pub struct KernelHeap {
    inner: Mutex<HeapState>,
}

/// Failure to allocate an owned block from the initialized kernel heap.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HeapError {
    /// Initialization has not completed, or a failed attempt poisoned the heap.
    NotInitialized,
    /// No suitably sized and aligned free region can satisfy the request.
    OutOfMemory,
}

impl KernelHeap {
    #[cfg(any(test, target_os = "none"))]
    pub(super) const fn new() -> Self {
        Self {
            inner: Mutex::new(HeapState {
                allocator: LinkedListHeap::new(),
                state: State::Uninitialized,
            }),
        }
    }

    fn with_state<R>(&self, action: impl FnOnce(&mut HeapState) -> R) -> R {
        #[cfg(target_os = "none")]
        {
            crate::arch::x86_64::cpu::without_interrupts(|| action(&mut self.inner.lock()))
        }
        #[cfg(not(target_os = "none"))]
        {
            action(&mut self.inner.lock())
        }
    }

    /// Claims a new region only after the mapping closure completed.
    ///
    /// A failed attempt remains poisoned because it may own physical frames
    /// and partial mappings. Retrying with a fresh frame cursor could assign
    /// those frames twice. Ready heaps likewise retain their live allocations.
    ///
    /// # Safety
    /// On closure success, `start..start + size` must be writable, 16-byte
    /// aligned and exclusively owned for the lifetime of this heap.
    #[cfg(any(test, target_os = "none"))]
    pub(super) unsafe fn initialize_region(
        &self,
        start: usize,
        size: usize,
        map_region: impl FnOnce() -> Result<(), MemoryInitError>,
    ) -> Result<(), MemoryInitError> {
        self.with_state(|inner| {
            match inner.state {
                State::Ready => return Err(MemoryInitError::AlreadyInitialized),
                State::Failed => return Err(MemoryInitError::PreviousInitializationFailed),
                State::Uninitialized => {}
            }
            inner.state = State::Failed;
            map_region()?;
            // SAFETY: the caller guarantees exclusive writable region ownership
            // after map_region succeeds, and the state guard enforces one init.
            unsafe { inner.allocator.init(start, size) };
            inner.state = State::Ready;
            Ok(())
        })
    }

    /// Allocates a zero-initialized owned block, preserving the requested layout.
    ///
    /// Metadata rounding and alignment padding are included in heap usage.
    /// A zero-length request can still reserve allocator bookkeeping capacity.
    /// Exhaustion returns an error without invalidating any live allocation.
    pub fn allocate(&self, layout: Layout) -> Result<HeapBlock<'_>, HeapError> {
        let pointer = self.with_state(|inner| {
            if inner.state != State::Ready {
                return Err(HeapError::NotInitialized);
            }
            NonNull::new(inner.allocator.allocate(layout)).ok_or(HeapError::OutOfMemory)
        })?;
        // SAFETY: the allocator returned an exclusive writable region covering
        // layout.size() bytes. Initialize it before exposing any safe byte slice.
        unsafe { pointer.as_ptr().write_bytes(0, layout.size()) };
        Ok(HeapBlock {
            heap: self,
            pointer,
            layout,
        })
    }

    /// Captures actual allocator statistics under the allocator's lock.
    ///
    /// Returns `None` until initialization succeeds. `used` includes allocator
    /// rounding, while `live_bytes` is the sum of requested allocation sizes.
    pub fn stats(&self) -> Option<HeapStats> {
        self.with_state(|inner| (inner.state == State::Ready).then(|| inner.allocator.stats()))
    }
}

impl MemoryReporter for KernelHeap {
    fn memory_snapshot(&self) -> Option<MemorySnapshot> {
        let stats = self.stats()?;
        MemorySnapshot::new(stats.heap_size as u64, stats.used as u64)
    }
}

/// Exclusive ownership of initialized bytes allocated from a [`KernelHeap`].
///
/// The original pointer and layout remain private, so dropping the handle frees
/// exactly this allocation once. Borrowing the heap keeps it alive until free.
/// A mutable slice requires a mutable borrow of the handle.
pub struct HeapBlock<'heap> {
    heap: &'heap KernelHeap,
    pointer: NonNull<u8>,
    layout: Layout,
}

impl HeapBlock<'_> {
    /// Returns the requested number of initialized bytes in this allocation.
    pub fn len(&self) -> usize {
        self.layout.size()
    }

    /// Returns whether the requested byte slice is empty.
    pub fn is_empty(&self) -> bool {
        self.layout.size() == 0
    }

    /// Returns the address for alignment and reuse diagnostics.
    pub fn address(&self) -> usize {
        self.pointer.as_ptr() as usize
    }

    /// Borrows this block's initialized bytes without exposing allocator metadata.
    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: allocation initialized these bytes and this handle owns them
        // until Drop. Its shared borrow prevents an overlapping mutable borrow.
        unsafe { core::slice::from_raw_parts(self.pointer.as_ptr(), self.layout.size()) }
    }

    /// Exclusively borrows the requested bytes until the returned slice expires.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: the unique handle owns the region and &mut self excludes all
        // other accesses through this handle for the lifetime of the slice.
        unsafe { core::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.layout.size()) }
    }
}

impl Drop for HeapBlock<'_> {
    fn drop(&mut self) {
        self.heap.with_state(|inner| {
            // SAFETY: private handle construction preserves the exact pointer
            // and Layout from allocate, and Drop runs once after all borrows end.
            unsafe {
                inner
                    .allocator
                    .deallocate(self.pointer.as_ptr(), self.layout)
            };
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[repr(align(4096))]
    struct Region([u8; 4096]);

    #[test]
    fn owned_blocks_zero_initialize_reuse_and_restore_real_stats() {
        let mut region = Region([0xa5; 4096]);
        let heap = KernelHeap::new();
        // SAFETY: region is aligned, exclusively borrowed for this test, and
        // remains alive until heap and every allocation have been dropped.
        unsafe {
            heap.initialize_region(region.0.as_mut_ptr() as usize, region.0.len(), || Ok(()))
        }
        .unwrap();
        let layout = Layout::from_size_align(64, 64).unwrap();
        let mut block = heap.allocate(layout).unwrap();
        let address = block.address();
        assert_eq!(address % 64, 0);
        assert_eq!(block.as_slice(), &[0; 64]);
        block.as_mut_slice().fill(0x5a);
        assert_eq!(block.as_slice(), &[0x5a; 64]);
        assert_eq!(heap.stats().unwrap().live_bytes, 64);
        drop(block);
        let reused = heap.allocate(layout).unwrap();
        assert_eq!(reused.address(), address);
        assert_eq!(reused.as_slice(), &[0; 64]);
        drop(reused);
        let snapshot = heap.memory_snapshot().unwrap();
        assert_eq!(snapshot.heap_capacity_bytes(), 4096);
        assert_eq!(snapshot.heap_used_bytes(), 0);
        assert_eq!(snapshot.heap_free_bytes(), 4096);
    }

    #[test]
    fn exhaustion_preserves_live_bytes_and_reinitialization_is_rejected() {
        let mut region = Region([0; 4096]);
        let heap = KernelHeap::new();
        // SAFETY: region is aligned, exclusive and outlives all heap handles.
        unsafe {
            heap.initialize_region(region.0.as_mut_ptr() as usize, region.0.len(), || Ok(()))
        }
        .unwrap();
        let block = heap
            .allocate(Layout::from_size_align(4096, 16).unwrap())
            .unwrap();
        assert!(matches!(
            heap.allocate(Layout::from_size_align(16, 16).unwrap()),
            Err(HeapError::OutOfMemory)
        ));
        assert_eq!(heap.stats().unwrap().live_allocations, 1);
        // SAFETY: the same valid region is offered, but a second initialization
        // must return an error before touching it or invalidating the live block.
        let second = unsafe { heap.initialize_region(block.address(), 4096, || Ok(())) };
        assert_eq!(second, Err(MemoryInitError::AlreadyInitialized));
        assert_eq!(block.as_slice(), &[0; 4096]);
        drop(block);
        assert_eq!(heap.stats().unwrap().used, 0);
    }

    #[test]
    fn simultaneous_allocations_own_disjoint_initialized_regions() {
        let mut region = Region([0xcc; 4096]);
        let heap = KernelHeap::new();
        // SAFETY: this aligned region is exclusive and remains alive until all
        // handles and the heap are dropped at the end of this test.
        unsafe {
            heap.initialize_region(region.0.as_mut_ptr() as usize, region.0.len(), || Ok(()))
        }
        .unwrap();
        let layout = Layout::from_size_align(128, 16).unwrap();
        let mut first = heap.allocate(layout).unwrap();
        let mut second = heap.allocate(layout).unwrap();
        assert!(first.address() + first.len() <= second.address());
        first.as_mut_slice().fill(0x11);
        second.as_mut_slice().fill(0x22);
        assert_eq!(first.as_slice(), &[0x11; 128]);
        assert_eq!(second.as_slice(), &[0x22; 128]);
        assert_eq!(heap.stats().unwrap().live_allocations, 2);
        drop(first);
        assert_eq!(second.as_slice(), &[0x22; 128]);
        assert_eq!(heap.stats().unwrap().live_allocations, 1);
        drop(second);
        assert_eq!(heap.stats().unwrap().free, 4096);
    }

    #[test]
    fn failed_mapping_publishes_no_heap_and_cannot_reassign_frames_on_retry() {
        let heap = KernelHeap::new();
        // SAFETY: the closure returns an error, so no supplied address is used.
        let result =
            unsafe { heap.initialize_region(0, 0, || Err(MemoryInitError::FrameAllocationFailed)) };
        assert_eq!(result, Err(MemoryInitError::FrameAllocationFailed));
        assert_eq!(heap.memory_snapshot(), None);
        assert!(matches!(
            heap.allocate(Layout::from_size_align(16, 16).unwrap()),
            Err(HeapError::NotInitialized)
        ));
        // SAFETY: poison rejects this attempt before inspecting its region.
        let result = unsafe { heap.initialize_region(0, 0, || panic!("must not retry mapping")) };
        assert_eq!(result, Err(MemoryInitError::PreviousInitializationFailed));
    }
}
