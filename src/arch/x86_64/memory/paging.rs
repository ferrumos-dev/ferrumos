//! Establish exclusive writable heap pages through the bootloader's direct map.

use bootloader_api::info::{MemoryRegion, MemoryRegionKind};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::{MapToError, TranslateResult};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
    Translate,
};
use x86_64::{PhysAddr, VirtAddr};

use super::frames::FrameCursor;
use super::{KernelHeap, MemoryInitError, FRAME_BYTES, HEAP_SIZE, HEAP_START};

static KERNEL_HEAP: KernelHeap = KernelHeap::new();

/// Returns the initialized shared heap for read-only reporters and allocations.
///
/// Returns `None` before initialization or after a failed attempt. The allocator
/// remains initialized for the kernel lifetime, so this reference stays valid.
pub fn heap() -> Option<&'static KernelHeap> {
    KERNEL_HEAP.stats().map(|_| &KERNEL_HEAP)
}

struct BootFrames<'a>(FrameCursor<'a>);

// SAFETY: initialize validates disjoint usable descriptors and exclusively owns
// every frame they describe. FrameCursor emits each whole frame at most once,
// and a failed attempt permanently prevents another cursor from reusing them.
unsafe impl FrameAllocator<Size4KiB> for BootFrames<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let address = self.0.next()?;
        PhysFrame::from_start_address(PhysAddr::new(address)).ok()
    }
}

/// Maps and initializes the 1 MiB kernel heap once on the boot CPU.
///
/// Only whole frames marked `Usable` are handed to data pages or new page
/// tables. A preflight rejects existing mappings across the chosen heap range.
/// Failure leaves the heap unavailable and permanently blocks another mapping
/// attempt. Partial mappings and their frames remain reserved until reboot.
///
/// # Safety
/// The caller must run at ring 0 on the only active CPU, own the active page
/// tables, and provide the bootloader's direct physical-memory offset. That map
/// must cover page tables and usable frames for the lifetime of the kernel.
/// `Usable` descriptors must represent exclusive, still-unused physical frames.
/// No other mapper, allocator, IRQ or CPU may claim those frames during setup.
pub unsafe fn initialize(
    physical_memory_offset: u64,
    regions: &[MemoryRegion],
) -> Result<&'static KernelHeap, MemoryInitError> {
    let frames = FrameCursor::new(regions).map_err(MemoryInitError::InvalidMemoryMap)?;
    let offset = VirtAddr::try_new(physical_memory_offset)
        .map_err(|_| MemoryInitError::InvalidPhysicalOffset)?;
    if !offset.is_aligned(FRAME_BYTES) {
        return Err(MemoryInitError::InvalidPhysicalOffset);
    }
    for region in regions {
        if region.kind == MemoryRegionKind::Usable && region.start < region.end {
            PhysAddr::try_new(region.end - 1)
                .map_err(|_| MemoryInitError::InvalidPhysicalAddress)?;
            direct_address(physical_memory_offset, region.end - 1)?;
        }
    }
    // SAFETY: the caller guarantees exclusive page-table and frame ownership.
    // initialize_region masks interrupts and serializes the single init attempt;
    // map_heap establishes every page before allocator metadata is written.
    unsafe {
        KERNEL_HEAP.initialize_region(HEAP_START as usize, HEAP_SIZE, || {
            map_heap(offset, BootFrames(frames))
        })?
    };
    Ok(&KERNEL_HEAP)
}

fn direct_address(offset: u64, physical: u64) -> Result<VirtAddr, MemoryInitError> {
    let address = offset
        .checked_add(physical)
        .ok_or(MemoryInitError::InvalidPhysicalOffset)?;
    VirtAddr::try_new(address).map_err(|_| MemoryInitError::InvalidPhysicalOffset)
}

/// Called only within initialize_region's single-CPU, IRQ-disabled transaction.
fn map_heap(offset: VirtAddr, mut frames: BootFrames<'_>) -> Result<(), MemoryInitError> {
    let (root_frame, _) = Cr3::read();
    let root_address = direct_address(offset.as_u64(), root_frame.start_address().as_u64())?;
    // SAFETY: initialize's caller guarantees that the direct map covers the live
    // root table and no alias accesses it while this temporary mapper exists.
    let root = unsafe { &mut *root_address.as_mut_ptr::<PageTable>() };
    // SAFETY: the supplied active table is exclusive and offset maps every
    // physical table frame, including the new frames allocated by this mapper.
    let mut mapper = unsafe { OffsetPageTable::new(root, offset) };
    let first = Page::<Size4KiB>::containing_address(VirtAddr::new(HEAP_START));
    let last =
        Page::<Size4KiB>::containing_address(VirtAddr::new(HEAP_START + HEAP_SIZE as u64 - 1));
    for page in Page::range_inclusive(first, last) {
        match mapper.translate(page.start_address()) {
            TranslateResult::NotMapped => {}
            TranslateResult::Mapped { .. } => return Err(MemoryInitError::HeapRangeInUse),
            TranslateResult::InvalidFrameAddress(_) => {
                return Err(MemoryInitError::InvalidPageTable)
            }
        }
    }
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_EXECUTE;
    for page in Page::range_inclusive(first, last) {
        let frame = frames
            .allocate_frame()
            .ok_or(MemoryInitError::FrameAllocationFailed)?;
        // SAFETY: preflight established unmapped pages; each fresh frame is
        // usable and exclusive. The same ownership contract covers table frames.
        let flush =
            unsafe { mapper.map_to(page, frame, flags, &mut frames) }.map_err(
                |error| match error {
                    MapToError::FrameAllocationFailed => MemoryInitError::FrameAllocationFailed,
                    MapToError::PageAlreadyMapped(_) => MemoryInitError::HeapRangeInUse,
                    MapToError::ParentEntryHugePage => MemoryInitError::MappingFailed,
                },
            )?;
        flush.flush();
    }
    Ok(())
}
