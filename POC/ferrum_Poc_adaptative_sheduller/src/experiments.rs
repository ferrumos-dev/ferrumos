use crate::allocator::{self, AllocError};
use crate::testing::{Record, Rng};
use crate::timing::Stopwatch;
use crate::serial_println;
use core::alloc::Layout;
use core::ptr::NonNull;

const LIVE_SLOTS: usize = 32;
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

struct LiveSet {
    slots: [Option<Block>; LIVE_SLOTS],
    len: usize,
}

impl LiveSet {
    const fn new() -> Self {
        LiveSet {
            slots: [None; LIVE_SLOTS],
            len: 0,
        }
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
}

fn release(set: &mut LiveSet, errors: &mut u64) {
    while set.len > 0 {
        if let Some(block) = set.take_nth(0) {
            if !block.is_intact() || !block.is_aligned() {
                *errors += 1;
            }
            unsafe { allocator::free(block.ptr, block.layout) };
        }
    }
}

/// Experiment 1: a fixed sequence of 10 000 allocations and deallocations with
/// varied sizes and alignments. Checks that live blocks never overlap and that
/// contents and alignments survive until the block is freed.
pub fn alloc_free_sequence() {
    const ITERATIONS: u64 = 10_000;

    let mut rng = Rng::new(0x2026_1017);
    let mut live = LiveSet::new();
    let mut errors = 0u64;
    let mut exhaustions = 0u64;
    let used_before = allocator::stats().used;

    let watch = Stopwatch::start();
    for iteration in 0..ITERATIONS {
        if live.len == LIVE_SLOTS || (live.len > 0 && rng.next_u64() % 3 == 0) {
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
                release(&mut live, &mut errors);
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

    release(&mut live, &mut errors);
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

/// Experiment 2: keep some allocations alive while others are created and
/// destroyed, to check whether freed memory becomes reusable.
pub fn long_lived_vs_churn() {
    const CHURN_SIZE: usize = 1024;
    const LONG_LIVED: usize = 4;
    const TARGET: u64 = 10_000;

    let mut live = LiveSet::new();
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
    for block in live.slots.iter().flatten() {
        if !block.is_intact() {
            errors += 1;
        }
    }
    release(&mut live, &mut errors);

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
