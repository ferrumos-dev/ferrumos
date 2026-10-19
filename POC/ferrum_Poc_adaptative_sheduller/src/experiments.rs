use crate::allocator::{self, AllocError};
use crate::interrupts;
use crate::memory;
use crate::serial_println;
use crate::testing::{Record, Rng};
use crate::timing::Stopwatch;
use core::alloc::Layout;
use core::ptr::NonNull;
use x86_64::structures::paging::{
    FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB, Translate,
};
use x86_64::VirtAddr;

const MIN_SIZE: usize = 8;
const MAX_SIZE: usize = 512;

#[derive(Clone, Copy)]
struct Block {
    ptr: NonNull<u8>,
    layout: Layout,
    pattern: u8,
}

impl Block {
    fn start(&self) -> usize {
        self.ptr.as_ptr() as usize
    }

    fn end(&self) -> usize {
        self.start() + self.layout.size()
    }

    fn fill(&self) {
        unsafe {
            core::ptr::write_bytes(self.ptr.as_ptr(), self.pattern, self.layout.size());
        }
    }

    fn is_intact(&self) -> bool {
        let slice =
            unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.layout.size()) };
        slice.iter().all(|byte| *byte == self.pattern)
    }

    fn is_aligned(&self) -> bool {
        self.start() % self.layout.align() == 0
    }

    fn overlaps(&self, other: &Block) -> bool {
        self.start() < other.end() && other.start() < self.end()
    }
}

struct LiveSet<const N: usize> {
    slots: [Option<Block>; N],
    len: usize,
}

impl<const N: usize> LiveSet<N> {
    const fn new() -> Self {
        LiveSet {
            slots: [None; N],
            len: 0,
        }
    }

    fn capacity(&self) -> usize {
        N
    }

    fn insert(&mut self, block: Block) -> bool {
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(block);
                self.len += 1;
                return true;
            }
        }
        false
    }

    fn take_nth(&mut self, index: usize) -> Option<Block> {
        let mut seen = 0;
        for slot in self.slots.iter_mut() {
            if slot.is_some() {
                if seen == index {
                    self.len -= 1;
                    return slot.take();
                }
                seen += 1;
            }
        }
        None
    }

    fn overlaps_any(&self, candidate: &Block) -> bool {
        self.slots
            .iter()
            .flatten()
            .any(|block| block.overlaps(candidate))
    }

    fn corrupted(&self) -> u64 {
        self.slots
            .iter()
            .flatten()
            .filter(|block| !block.is_intact() || !block.is_aligned())
            .count() as u64
    }

    fn release(&mut self, errors: &mut u64) {
        while self.len > 0 {
            if let Some(block) = self.take_nth(0) {
                if !block.is_intact() || !block.is_aligned() {
                    *errors += 1;
                }
                unsafe { allocator::free(block.ptr, block.layout) };
            }
        }
    }
}

pub fn alloc_free_sequence() {
    const ITERATIONS: u64 = 10_000;
    const SLOTS: usize = 32;

    let mut rng = Rng::new(0x2026_1017);
    let mut live: LiveSet<SLOTS> = LiveSet::new();
    let mut errors = 0u64;
    let mut exhaustions = 0u64;
    let used_before = allocator::stats().used;

    let watch = Stopwatch::start();
    for iteration in 0..ITERATIONS {
        if live.len == live.capacity() || (live.len > 0 && rng.next_u64() % 3 == 0) {
            let victim = rng.next_in_range(0, live.len - 1);
            if let Some(block) = live.take_nth(victim) {
                if !block.is_intact() {
                    errors += 1;
                }
                if !block.is_aligned() {
                    errors += 1;
                }
                unsafe { allocator::free(block.ptr, block.layout) };
            }
            continue;
        }

        let size = rng.next_in_range(MIN_SIZE, MAX_SIZE);
        let align = rng.next_align();
        let layout = Layout::from_size_align(size, align).expect("invalid layout");

        let ptr = match allocator::try_alloc(layout) {
            Ok(ptr) => ptr,
            Err(AllocError) => {
                exhaustions += 1;
                live.release(&mut errors);
                match allocator::try_alloc(layout) {
                    Ok(ptr) => ptr,
                    Err(AllocError) => {
                        errors += 1;
                        continue;
                    }
                }
            }
        };

        let block = Block {
            ptr,
            layout,
            pattern: (iteration % 251) as u8,
        };

        if !block.is_aligned() {
            errors += 1;
        }
        if live.overlaps_any(&block) {
            errors += 1;
        }
        block.fill();

        if !live.insert(block) {
            unsafe { allocator::free(block.ptr, block.layout) };
        }
    }
    let cycles = watch.elapsed();

    live.release(&mut errors);
    let stats = allocator::stats();

    serial_println!(
        "\n\theap exhaustions forcing a full release: {}",
        exhaustions
    );

    Record {
        name: "exp1_alloc_free_sequence",
        iterations: ITERATIONS,
        errors,
        used_before,
        used_after: stats.used,
        cycles,
    }
    .report();

    assert_eq!(errors, 0);
}

pub fn long_lived_vs_churn() {
    const CHURN_SIZE: usize = 1024;
    const LONG_LIVED: usize = 4;
    const TARGET: u64 = 10_000;

    let mut live: LiveSet<8> = LiveSet::new();
    let mut errors = 0u64;
    let used_before = allocator::stats().used;

    for index in 0..LONG_LIVED {
        let layout = Layout::from_size_align(64, 8).expect("invalid layout");
        let ptr = allocator::try_alloc(layout).expect("long lived allocation failed");
        let block = Block {
            ptr,
            layout,
            pattern: 0xA0 + index as u8,
        };
        block.fill();
        live.insert(block);
    }

    let layout = Layout::from_size_align(CHURN_SIZE, 8).expect("invalid layout");
    let heap_size = allocator::stats().heap_size;
    let mut churn = 0u64;

    let watch = Stopwatch::start();
    while churn < TARGET {
        match allocator::try_alloc(layout) {
            Ok(ptr) => {
                unsafe {
                    core::ptr::write_bytes(ptr.as_ptr(), 0x5A, CHURN_SIZE);
                    allocator::free(ptr, layout);
                }
                churn += 1;
            }
            Err(AllocError) => break,
        }
    }
    let cycles = watch.elapsed();

    let stats = allocator::stats();
    errors += live.corrupted();
    live.release(&mut errors);

    serial_println!(
        "\n\tchurn of {} B survived {} iterations out of {} ({} B heap, {} long lived blocks)",
        CHURN_SIZE,
        churn,
        TARGET,
        heap_size,
        LONG_LIVED
    );
    serial_println!(
        "\tfreed memory is reusable: {}",
        if churn == TARGET { "yes" } else { "no" }
    );

    Record {
        name: "exp2_long_lived_vs_churn",
        iterations: churn,
        errors,
        used_before,
        used_after: stats.used,
        cycles,
    }
    .report();

    assert_eq!(errors, 0);
}

pub fn exhaustion_and_recovery() {
    const BLOCK_SIZE: usize = 1024;
    const SLOTS: usize = 256;
    const ROUNDS: usize = 3;

    let layout = Layout::from_size_align(BLOCK_SIZE, 8).expect("invalid layout");
    let mut errors = 0u64;
    let mut capacities = [0usize; ROUNDS];
    let mut failures_reported = 0u64;
    let used_before = allocator::stats().used;
    let failed_before = allocator::stats().failed_allocations;

    let watch = Stopwatch::start();
    for round in 0..ROUNDS {
        let mut live: LiveSet<SLOTS> = LiveSet::new();

        loop {
            match allocator::try_alloc(layout) {
                Ok(ptr) => {
                    let block = Block {
                        ptr,
                        layout,
                        pattern: (0x10 + round) as u8,
                    };
                    if !block.is_aligned() || live.overlaps_any(&block) {
                        errors += 1;
                    }
                    block.fill();
                    if !live.insert(block) {
                        unsafe { allocator::free(block.ptr, block.layout) };
                        break;
                    }
                }
                Err(AllocError) => {
                    failures_reported += 1;
                    break;
                }
            }
        }

        capacities[round] = live.len;
        errors += live.corrupted();

        let after_failure = allocator::try_alloc(Layout::from_size_align(8, 8).unwrap());
        if after_failure.is_ok() {
            if let Ok(ptr) = after_failure {
                unsafe { allocator::free(ptr, Layout::from_size_align(8, 8).unwrap()) };
            }
        }

        live.release(&mut errors);
    }
    let cycles = watch.elapsed();

    let stats = allocator::stats();

    serial_println!(
        "\n\tcapacity per round: {} / {} / {} blocks of {} B",
        capacities[0],
        capacities[1],
        capacities[2],
        BLOCK_SIZE
    );
    serial_println!(
        "\texplicit failures reported: {} (allocator counter went from {} to {})",
        failures_reported,
        failed_before,
        stats.failed_allocations
    );
    serial_println!(
        "\trecovery after a full release: {}",
        if capacities[1] == capacities[0] && capacities[2] == capacities[0] {
            "complete"
        } else {
            "partial"
        }
    );

    Record {
        name: "exp3_exhaustion_and_recovery",
        iterations: ROUNDS as u64,
        errors,
        used_before,
        used_after: stats.used,
        cycles,
    }
    .report();

    assert_eq!(errors, 0);
    assert!(failures_reported > 0);
    assert_eq!(capacities[1], capacities[0]);
    assert_eq!(capacities[2], capacities[0]);
}

pub fn page_cycle() {
    const CYCLES: u64 = 500;
    const BASE: u64 = 0x_5555_0000_0000;

    let mut errors = 0u64;
    let mut translation_errors = 0u64;
    let used_before = allocator::stats().used;

    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(BASE));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    let free_at_start = memory::frame_stats().free_frames;

    memory::with_memory(|mapper, frame_allocator| {
        let frame = frame_allocator.allocate_frame().expect("no frame available");
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("mapping failed")
                .flush();
        }
        let (frame, flush) = mapper.unmap(page).expect("unmapping failed");
        flush.flush();
        frame_allocator.deallocate_frame(frame);
    });

    let page_table_frames = free_at_start - memory::frame_stats().free_frames;
    let free_before_loop = memory::frame_stats().free_frames;

    let watch = Stopwatch::start();
    for iteration in 0..CYCLES {
        memory::with_memory(|mapper, frame_allocator| {
            let frame = frame_allocator.allocate_frame().expect("no frame available");
            let expected = frame.start_address();

            unsafe {
                mapper
                    .map_to(page, frame, flags, frame_allocator)
                    .expect("mapping failed")
                    .flush();
            }

            match mapper.translate_addr(page.start_address()) {
                Some(physical) if physical == expected => {}
                _ => translation_errors += 1,
            }

            let ptr: *mut u64 = page.start_address().as_mut_ptr();
            unsafe {
                ptr.write_volatile(0xC0FF_EE00 + iteration);
                if ptr.read_volatile() != 0xC0FF_EE00 + iteration {
                    errors += 1;
                }
            }

            let (frame, flush) = mapper.unmap(page).expect("unmapping failed");
            flush.flush();

            if mapper.translate_addr(page.start_address()).is_some() {
                translation_errors += 1;
            }

            frame_allocator.deallocate_frame(frame);
        });
    }
    let cycles = watch.elapsed();

    let free_after_loop = memory::frame_stats().free_frames;
    let lost = free_before_loop as i64 - free_after_loop as i64;
    errors += translation_errors;

    serial_println!(
        "\n\tpage table frames consumed by the first mapping: {}",
        page_table_frames
    );
    serial_println!(
        "\tframes free before the loop: {} after {} cycles: {} (lost: {})",
        free_before_loop,
        CYCLES,
        free_after_loop,
        lost
    );
    serial_println!("\ttranslation errors: {}", translation_errors);

    Record {
        name: "exp4_page_cycle",
        iterations: CYCLES,
        errors,
        used_before,
        used_after: allocator::stats().used,
        cycles,
    }
    .report();

    assert_eq!(translation_errors, 0);
    assert_eq!(errors, 0);
    assert_eq!(lost, 0);
}

pub fn under_interrupts() {
    const ITERATIONS: u64 = 10_000;
    const SLOTS: usize = 32;
    const PAGE_CYCLES: u64 = 200;
    const BASE: u64 = 0x_6666_0000_0000;

    let mut rng = Rng::new(0x2026_1019);
    let mut live: LiveSet<SLOTS> = LiveSet::new();
    let mut errors = 0u64;
    let mut allocation_failures = 0u64;
    let mut completed = 0u64;
    let used_before = allocator::stats().used;

    interrupts::unmask_timer();
    let ticks_before = interrupts::ticks();

    let watch = Stopwatch::start();
    for iteration in 0..ITERATIONS {
        if live.len == live.capacity() || (live.len > 0 && rng.next_u64() % 3 == 0) {
            let victim = rng.next_in_range(0, live.len - 1);
            if let Some(block) = live.take_nth(victim) {
                if !block.is_intact() || !block.is_aligned() {
                    errors += 1;
                }
                unsafe { allocator::free(block.ptr, block.layout) };
            }
            continue;
        }

        let size = rng.next_in_range(MIN_SIZE, MAX_SIZE);
        let align = rng.next_align();
        let layout = Layout::from_size_align(size, align).expect("invalid layout");

        let ptr = match allocator::try_alloc(layout) {
            Ok(ptr) => ptr,
            Err(AllocError) => {
                live.release(&mut errors);
                match allocator::try_alloc(layout) {
                    Ok(ptr) => ptr,
                    Err(AllocError) => {
                        allocation_failures += 1;
                        continue;
                    }
                }
            }
        };

        let block = Block {
            ptr,
            layout,
            pattern: (iteration % 251) as u8,
        };

        if !block.is_aligned() || live.overlaps_any(&block) {
            errors += 1;
        }
        block.fill();
        completed += 1;

        if !live.insert(block) {
            unsafe { allocator::free(block.ptr, block.layout) };
        }
    }

    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(BASE));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let free_before_pages = memory::frame_stats().free_frames;

    for iteration in 0..PAGE_CYCLES {
        memory::with_memory(|mapper, frame_allocator| {
            let frame = frame_allocator.allocate_frame().expect("no frame available");
            unsafe {
                mapper
                    .map_to(page, frame, flags, frame_allocator)
                    .expect("mapping failed")
                    .flush();
            }
            let ptr: *mut u64 = page.start_address().as_mut_ptr();
            unsafe {
                ptr.write_volatile(0xBEEF_0000 + iteration);
                if ptr.read_volatile() != 0xBEEF_0000 + iteration {
                    errors += 1;
                }
            }
            let (frame, flush) = mapper.unmap(page).expect("unmapping failed");
            flush.flush();
            frame_allocator.deallocate_frame(frame);
        });
    }
    let cycles = watch.elapsed();

    let ticks_after = interrupts::ticks();
    let free_after_pages = memory::frame_stats().free_frames;
    errors += live.corrupted();
    live.release(&mut errors);
    interrupts::mask_all();

    let elapsed_ticks = ticks_after - ticks_before;
    let scheduler_ticks = crate::scheduler::snapshot().0;

    serial_println!(
        "\n\ttimer interrupts during the run: {} (scheduler ticks: {})",
        elapsed_ticks,
        scheduler_ticks
    );
    serial_println!(
        "\tframes before the page loop: {} after: {}",
        free_before_pages,
        free_after_pages
    );
    serial_println!(
        "\tallocations completed: {} refused after exhaustion: {}",
        completed,
        allocation_failures
    );
    serial_println!(
        "\tdata corruption under interrupts: {}",
        if errors == 0 { "none" } else { "detected" }
    );
    serial_println!("\tmulti task replay: not possible, the scheduler has no context switch");

    Record {
        name: "exp5_under_interrupts",
        iterations: ITERATIONS + PAGE_CYCLES,
        errors,
        used_before,
        used_after: allocator::stats().used,
        cycles,
    }
    .report();

    assert!(elapsed_ticks > 0);
    assert_eq!(errors, 0);
}
