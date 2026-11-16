//! Uniform wrapper around every allocator under test.

use core::alloc::Layout;
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use ferrum_alloc::{BumpHeap, FixedBlockHeap, Heap, HeapStats, LinkedListHeap};
use std::alloc::{alloc, dealloc};

/// Same size as the kernel heap (POC/ferrum_Poc_adaptative_sheduller/src/allocator.rs).
pub const HEAP_SIZE: usize = 100 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bump,
    LinkedList,
    FixedBlockTextbook,
    FixedBlock,
    Tlsf,
}

impl Kind {
    pub const ALL: [Kind; 5] = [
        Kind::Bump,
        Kind::LinkedList,
        Kind::FixedBlockTextbook,
        Kind::FixedBlock,
        Kind::Tlsf,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Bump => "bump (baseline)",
            Kind::LinkedList => "A linked_list",
            Kind::FixedBlockTextbook => "B fixed_block (no reclaim)",
            Kind::FixedBlock => "B' fixed_block + reclaim",
            Kind::Tlsf => "X tlsf (rlsf 0.2.3, exploration)",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Kind::Bump => "bump",
            Kind::LinkedList => "linked_list",
            Kind::FixedBlockTextbook => "fixed_block_textbook",
            Kind::FixedBlock => "fixed_block_reclaim",
            Kind::Tlsf => "tlsf",
        }
    }
}

/// A 4 KiB-aligned region, like the kernel heap at 0x4444_4444_0000.
pub struct Region {
    start: *mut u8,
    layout: Layout,
}

impl Region {
    pub fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 4096).unwrap();
        let start = unsafe { alloc(layout) };
        assert!(!start.is_null());
        Region { start, layout }
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        unsafe { dealloc(self.start, self.layout) }
    }
}

pub trait BenchHeap {
    fn alloc(&mut self, layout: Layout) -> *mut u8;
    /// # Safety
    /// `ptr` comes from `alloc` with the same layout.
    unsafe fn free(&mut self, ptr: *mut u8, layout: Layout);
    /// `None` when the allocator does not expose statistics (TLSF).
    fn stats(&self) -> Option<HeapStats>;
}

struct Lib<H: Heap>(H);

impl<H: Heap> BenchHeap for Lib<H> {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        self.0.allocate(layout)
    }
    unsafe fn free(&mut self, ptr: *mut u8, layout: Layout) {
        self.0.deallocate(ptr, layout)
    }
    fn stats(&self) -> Option<HeapStats> {
        Some(self.0.stats())
    }
}

type RlsfTlsf = rlsf::Tlsf<'static, u16, u16, 12, 16>;

struct Tlsf(RlsfTlsf);

impl BenchHeap for Tlsf {
    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        self.0
            .allocate(layout)
            .map_or(core::ptr::null_mut(), NonNull::as_ptr)
    }
    unsafe fn free(&mut self, ptr: *mut u8, layout: Layout) {
        self.0
            .deallocate(NonNull::new_unchecked(ptr), layout.align())
    }
    fn stats(&self) -> Option<HeapStats> {
        None
    }
}

/// A fresh allocator over a fresh region: every scenario starts from a
/// newly initialised heap, as every experiment boots its own kernel image.
pub fn make(kind: Kind) -> (Region, Box<dyn BenchHeap>) {
    let region = Region::new(HEAP_SIZE);
    let start = region.start as usize;
    let heap: Box<dyn BenchHeap> = unsafe {
        match kind {
            Kind::Bump => {
                let mut h = BumpHeap::new();
                h.init(start, HEAP_SIZE);
                Box::new(Lib(h))
            }
            Kind::LinkedList => {
                let mut h = LinkedListHeap::new();
                h.init(start, HEAP_SIZE);
                Box::new(Lib(h))
            }
            Kind::FixedBlockTextbook => {
                let mut h = FixedBlockHeap::with_reclaim(false);
                h.init(start, HEAP_SIZE);
                Box::new(Lib(h))
            }
            Kind::FixedBlock => {
                let mut h = FixedBlockHeap::with_reclaim(true);
                h.init(start, HEAP_SIZE);
                Box::new(Lib(h))
            }
            Kind::Tlsf => {
                let mut t = RlsfTlsf::new();
                let block =
                    NonNull::slice_from_raw_parts(NonNull::new(region.start).unwrap(), HEAP_SIZE);
                let inserted: Option<NonZeroUsize> = t.insert_free_block_ptr(block);
                assert!(inserted.is_some());
                Box::new(Tlsf(t))
            }
        }
    };
    (region, heap)
}
