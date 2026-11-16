use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use spin::Mutex;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

pub static MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
pub static FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);

pub fn install(mapper: OffsetPageTable<'static>, frame_allocator: BootInfoFrameAllocator) {
    *MAPPER.lock() = Some(mapper);
    *FRAME_ALLOCATOR.lock() = Some(frame_allocator);
}

pub fn with_memory<R>(
    f: impl FnOnce(&mut OffsetPageTable<'static>, &mut BootInfoFrameAllocator) -> R,
) -> R {
    let mut mapper = MAPPER.lock();
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let mapper = mapper.as_mut().expect("memory subsystem not installed");
    let frame_allocator = frame_allocator
        .as_mut()
        .expect("memory subsystem not installed");
    f(mapper, frame_allocator)
}

pub fn frame_stats() -> FrameStats {
    FRAME_ALLOCATOR
        .lock()
        .as_ref()
        .expect("memory subsystem not installed")
        .stats()
}

/// Builds the mapper for the active page table.
///
/// # Safety
/// `physical_memory_offset` must map all physical memory at that offset
/// (bootloader `map_physical_memory`). The caller guarantees exclusive
/// mutable access to the active table while the mapper exists: call this
/// once at boot, then hand the mapper to `install`.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    // SAFETY: direct mapping and exclusivity are guaranteed by the caller.
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
    // SAFETY: the offset describes the physical mapping used by this table.
    unsafe { OffsetPageTable::new(level_4_table, physical_memory_offset) }
}

/// # Safety
/// Same as [`init`]; the returned reference must be the only one to the table.
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // SAFETY: CR3 holds the physical address of the active level 4 table and
    // the caller guarantees it is mapped at `physical_memory_offset` and not
    // referenced elsewhere.
    unsafe { &mut *page_table_ptr }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameStats {
    pub usable_frames: usize,
    pub allocated_frames: usize,
    pub free_frames: usize,
    pub reclaimed_frames: usize,
}

const RECLAIM_SLOTS: usize = 64;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
    reclaimed: [Option<PhysFrame>; RECLAIM_SLOTS],
    reclaimed_len: usize,
}

impl BootInfoFrameAllocator {
    /// # Safety
    /// Every region of `memory_map` marked `Usable` must really be unused:
    /// pass the memory map given by the bootloader, and create one allocator.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
            reclaimed: [None; RECLAIM_SLOTS],
            reclaimed_len: 0,
        }
    }

    pub fn stats(&self) -> FrameStats {
        let usable = self.usable_frames().count();
        FrameStats {
            usable_frames: usable,
            allocated_frames: self.next - self.reclaimed_len,
            free_frames: usable - self.next + self.reclaimed_len,
            reclaimed_frames: self.reclaimed_len,
        }
    }

    /// Gives a frame back for reuse. Returns false when the reclaim slots are
    /// full (the frame is then leaked, never reused).
    ///
    /// The frame must have been unmapped first: it is handed out again by
    /// `allocate_frame`. This is a precondition the type does not enforce;
    /// every caller unmaps the page just before (lib.rs, experiments.rs).
    pub fn deallocate_frame(&mut self, frame: PhysFrame) -> bool {
        for slot in self.reclaimed.iter_mut() {
            if slot.is_none() {
                *slot = Some(frame);
                self.reclaimed_len += 1;
                return true;
            }
        }
        false
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

// SAFETY: frames come from `Usable` regions of the bootloader map (see
// `init`) and each index is handed out once, as `next` only grows. Reclaimed
// frames are only those given back by `deallocate_frame` after unmapping.
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for slot in self.reclaimed.iter_mut().rev() {
            if slot.is_some() {
                self.reclaimed_len -= 1;
                return slot.take();
            }
        }
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
