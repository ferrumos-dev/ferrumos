//! Kernel heap.
//!
//! The allocator itself lives in `libs/ferrum_alloc`, shared with the host
//! benchmark (research/heap-allocator/bench), so that the code measured on the
//! host is the code that runs here. One allocator is chosen at build time:
//!
//! | Cargo feature        | Allocator                                   |
//! |----------------------|---------------------------------------------|
//! | `heap-linked-list`   | free list with coalescing (default, decision of objective 1) |
//! | `heap-fixed-block`   | fixed-size blocks + reclaim (candidate B')  |
//! | `heap-bump`          | bump allocator, the October "before"        |
//!
//!   cargo test --no-default-features --features heap-bump
//!
//! See docs/greenlights/objecitf_obligatoire_1/tas_noyau/08_note_decision.md for the measurements behind the default.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr;
use ferrum_alloc::Heap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

pub use ferrum_alloc::HeapStats;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[cfg(any(
    all(feature = "heap-bump", feature = "heap-linked-list"),
    all(feature = "heap-bump", feature = "heap-fixed-block"),
    all(feature = "heap-linked-list", feature = "heap-fixed-block"),
))]
compile_error!("choose exactly one heap-* feature (use --no-default-features)");

#[cfg(feature = "heap-bump")]
pub type KernelHeap = ferrum_alloc::BumpHeap;
#[cfg(feature = "heap-linked-list")]
pub type KernelHeap = ferrum_alloc::LinkedListHeap;
#[cfg(feature = "heap-fixed-block")]
pub type KernelHeap = ferrum_alloc::FixedBlockHeap;

#[cfg(not(any(
    feature = "heap-bump",
    feature = "heap-linked-list",
    feature = "heap-fixed-block"
)))]
compile_error!("no heap-* feature selected");

/// Name of the allocator built into this kernel, printed by the tests.
pub const HEAP_NAME: &str = KernelHeap::NAME;

use spin::Mutex;

pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<'_, A> {
        self.inner.lock()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocError;

pub fn stats() -> HeapStats {
    ALLOCATOR.lock().stats()
}

/// Allocates from the kernel heap. `Err(AllocError)` when the heap cannot
/// serve `layout`; the heap is left unchanged.
pub fn try_alloc(layout: Layout) -> Result<ptr::NonNull<u8>, AllocError> {
    // SAFETY: `GlobalAlloc::alloc` only requires a non-zero size; every
    // `Heap` also accepts zero-size layouts (rounded up) and returns null on
    // failure, which is mapped to `AllocError`.
    let raw = unsafe { ALLOCATOR.alloc(layout) };
    ptr::NonNull::new(raw).ok_or(AllocError)
}

/// Gives back a block from `try_alloc`.
///
/// # Safety
/// `allocation` must come from `try_alloc` with the same `layout` and must
/// not have been freed already.
pub unsafe fn free(allocation: ptr::NonNull<u8>, layout: Layout) {
    // SAFETY: forwarded from the caller.
    unsafe { ALLOCATOR.dealloc(allocation.as_ptr(), layout) };
}

// SAFETY: the spin mutex serializes every access to the heap, which only
// hands out blocks of its own region that are not in use (`Heap` contract,
// tested in libs/ferrum_alloc and by the QEMU experiments). Allocation
// failure returns null, never panics.
unsafe impl GlobalAlloc for Locked<KernelHeap> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.lock().allocate(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `GlobalAlloc` callers guarantee `ptr` came from `alloc` on
        // this allocator with the same `layout`, as `Heap::deallocate` requires.
        unsafe { self.lock().deallocate(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Locked<KernelHeap> = Locked::new(KernelHeap::new());

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        // SAFETY: the heap range is used by nothing else and `frame` was just
        // allocated, so the new mapping aliases no existing memory.
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // SAFETY: `HEAP_START..HEAP_START + HEAP_SIZE` was just mapped writable,
    // is used by nothing else, is page-aligned (so 16-byte aligned), and
    // `init_heap` is called once, from `lib::init`.
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
