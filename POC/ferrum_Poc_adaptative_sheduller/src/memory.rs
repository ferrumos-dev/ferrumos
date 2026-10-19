use x86_64::structures::paging::{
    PageTable, OffsetPageTable, PhysFrame, Size4KiB, FrameAllocator
};
use x86_64::{PhysAddr, VirtAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use spin::Mutex;

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

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
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
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

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
