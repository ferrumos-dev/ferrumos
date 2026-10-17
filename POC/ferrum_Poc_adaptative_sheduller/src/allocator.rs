use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use core::ptr;
use core::alloc::{GlobalAlloc, Layout};

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

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
pub struct HeapStats {
    pub heap_size: usize,
    pub used: usize,
    pub free: usize,
    pub live_allocations: usize,
    pub total_allocations: u64,
    pub failed_allocations: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocError;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
    total_allocations: u64,
    failed_allocations: u64,
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
            total_allocations: 0,
            failed_allocations: 0,
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }

    pub fn stats(&self) -> HeapStats {
        HeapStats {
            heap_size: self.heap_end - self.heap_start,
            used: self.next - self.heap_start,
            free: self.heap_end - self.next,
            live_allocations: self.allocations,
            total_allocations: self.total_allocations,
            failed_allocations: self.failed_allocations,
        }
    }
}

pub fn stats() -> HeapStats {
    ALLOCATOR.lock().stats()
}

pub fn try_alloc(layout: Layout) -> Result<ptr::NonNull<u8>, AllocError> {
    let raw = unsafe { ALLOCATOR.alloc(layout) };
    match ptr::NonNull::new(raw) {
        Some(nonnull) => Ok(nonnull),
        None => Err(AllocError),
    }
}

pub unsafe fn free(allocation: ptr::NonNull<u8>, layout: Layout) {
    ALLOCATOR.dealloc(allocation.as_ptr(), layout);
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock();

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            bump.failed_allocations += 1;
            ptr::null_mut()
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            bump.total_allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock();

        bump.allocations = bump.allocations.saturating_sub(1);
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[global_allocator]
static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());

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
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}
