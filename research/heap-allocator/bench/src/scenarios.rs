//! The scenarios of the protocol (docs/greenlights/objecitf_obligatoire_1/tas_noyau/06_prototype_decision.md).
//!
//! exp1, exp2 and exp3 are line-by-line ports of
//! POC/ferrum_Poc_adaptative_sheduller/src/experiments.rs (same seed, same
//! sizes, same checks), so that their deterministic results can be compared
//! with the QEMU runs. `phase_change` and `long_run` are new: they cover the
//! limits listed in docs/greenlights/objecitf_obligatoire_1/tas_noyau/experiences/compte_rendu_experiences_1_2.md §5.

use crate::heaps::BenchHeap;
use core::alloc::Layout;

/// xorshift64, identical to `testing::Rng` in the kernel.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn next_in_range(&mut self, min: usize, max: usize) -> usize {
        min + (self.next_u64() as usize) % (max - min + 1)
    }
    pub fn next_align(&mut self) -> usize {
        1 << (self.next_u64() % 7)
    }
}

#[derive(Clone, Copy)]
struct Block {
    ptr: *mut u8,
    layout: Layout,
    pattern: u8,
}

impl Block {
    fn start(&self) -> usize {
        self.ptr as usize
    }
    fn end(&self) -> usize {
        self.start() + self.layout.size()
    }
    fn fill(&self) {
        unsafe { core::ptr::write_bytes(self.ptr, self.pattern, self.layout.size()) }
    }
    fn is_intact(&self) -> bool {
        let bytes = unsafe { core::slice::from_raw_parts(self.ptr, self.layout.size()) };
        bytes.iter().all(|b| *b == self.pattern)
    }
    fn is_aligned(&self) -> bool {
        self.start() % self.layout.align() == 0
    }
    fn overlaps(&self, other: &Block) -> bool {
        self.start() < other.end() && other.start() < self.end()
    }
}

/// Same behaviour as `LiveSet<N>` in the kernel (slot order matters for
/// `take_nth`, hence the Option slots rather than a Vec).
struct LiveSet {
    slots: Vec<Option<Block>>,
    len: usize,
}

impl LiveSet {
    fn new(n: usize) -> Self {
        LiveSet {
            slots: vec![None; n],
            len: 0,
        }
    }
    fn capacity(&self) -> usize {
        self.slots.len()
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
        self.slots.iter().flatten().any(|b| b.overlaps(candidate))
    }
    fn corrupted(&self) -> u64 {
        self.slots
            .iter()
            .flatten()
            .filter(|b| !b.is_intact() || !b.is_aligned())
            .count() as u64
    }
    fn release(&mut self, heap: &mut dyn BenchHeap, errors: &mut u64) {
        while self.len > 0 {
            if let Some(block) = self.take_nth(0) {
                if !block.is_intact() || !block.is_aligned() {
                    *errors += 1;
                }
                unsafe { heap.free(block.ptr, block.layout) };
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Outcome {
    pub iterations: u64,
    pub errors: u64,
    /// exp1: full releases forced by an exhaustion. long_run: refused requests.
    pub exhaustions: u64,
    /// exp3: capacity of each round. phase_change: [small blocks, 1 KiB blocks].
    pub capacities: Vec<usize>,
    pub used_after: Option<usize>,
    pub live_bytes_after: Option<usize>,
    pub fragmentation_permille: Option<u32>,
}

fn used(heap: &dyn BenchHeap) -> Option<usize> {
    heap.stats().map(|s| s.used)
}

/// Experiment 1: 10 000 operations, sizes 8..512 B, alignments 1..64 B, at
/// most 32 live blocks; a full release when the heap is exhausted.
pub fn exp1(heap: &mut dyn BenchHeap, seed: u64) -> Outcome {
    const ITERATIONS: u64 = 10_000;
    let mut rng = Rng::new(seed);
    let mut live = LiveSet::new(32);
    let mut errors = 0;
    let mut exhaustions = 0;

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
                unsafe { heap.free(block.ptr, block.layout) };
            }
            continue;
        }

        let size = rng.next_in_range(8, 512);
        let align = rng.next_align();
        let layout = Layout::from_size_align(size, align).unwrap();

        let mut ptr = heap.alloc(layout);
        if ptr.is_null() {
            exhaustions += 1;
            live.release(heap, &mut errors);
            ptr = heap.alloc(layout);
            if ptr.is_null() {
                errors += 1;
                continue;
            }
        }

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
            unsafe { heap.free(block.ptr, block.layout) };
        }
    }

    live.release(heap, &mut errors);
    Outcome {
        iterations: ITERATIONS,
        errors,
        exhaustions,
        used_after: used(heap),
        ..Outcome::default()
    }
}

/// Experiment 2: 4 long-lived 64 B blocks, then allocate and free 1 KiB
/// until 10 000 iterations or the first refusal.
pub fn exp2(heap: &mut dyn BenchHeap) -> Outcome {
    const TARGET: u64 = 10_000;
    let mut live = LiveSet::new(8);
    let mut errors = 0;

    for index in 0..4u8 {
        let layout = Layout::from_size_align(64, 8).unwrap();
        let ptr = heap.alloc(layout);
        assert!(!ptr.is_null(), "long lived allocation failed");
        let block = Block {
            ptr,
            layout,
            pattern: 0xA0 + index,
        };
        block.fill();
        live.insert(block);
    }

    let layout = Layout::from_size_align(1024, 8).unwrap();
    let mut churn = 0;
    while churn < TARGET {
        let ptr = heap.alloc(layout);
        if ptr.is_null() {
            break;
        }
        unsafe {
            core::ptr::write_bytes(ptr, 0x5A, 1024);
            heap.free(ptr, layout);
        }
        churn += 1;
    }

    let stats = heap.stats();
    errors += live.corrupted();
    live.release(heap, &mut errors);
    Outcome {
        iterations: churn,
        errors,
        used_after: stats.map(|s| s.used),
        live_bytes_after: stats.map(|s| s.live_bytes),
        ..Outcome::default()
    }
}

/// Experiment 3: fill with 1 KiB blocks until refusal, free all, three rounds.
pub fn exp3(heap: &mut dyn BenchHeap) -> Outcome {
    let layout = Layout::from_size_align(1024, 8).unwrap();
    let mut errors = 0;
    let mut capacities = Vec::new();

    for round in 0..3u8 {
        let mut live = LiveSet::new(256);
        loop {
            let ptr = heap.alloc(layout);
            if ptr.is_null() {
                break;
            }
            let block = Block {
                ptr,
                layout,
                pattern: 0x10 + round,
            };
            if !block.is_aligned() || live.overlaps_any(&block) {
                errors += 1;
            }
            block.fill();
            if !live.insert(block) {
                unsafe { heap.free(block.ptr, block.layout) };
                break;
            }
        }
        capacities.push(live.len);
        errors += live.corrupted();
        live.release(heap, &mut errors);
    }

    Outcome {
        iterations: 3,
        errors,
        capacities,
        used_after: used(heap),
        ..Outcome::default()
    }
}

/// New scenario: the workload changes size. Fill the heap with 64 B blocks,
/// free them all, then fill it with 1 KiB blocks. A heap that really
/// reuses freed memory gets the same 1 KiB capacity as on a fresh heap.
pub fn phase_change(heap: &mut dyn BenchHeap) -> Outcome {
    let mut errors = 0;
    let small = Layout::from_size_align(64, 8).unwrap();
    let mut live = LiveSet::new(4096);
    loop {
        let ptr = heap.alloc(small);
        if ptr.is_null() {
            break;
        }
        let block = Block {
            ptr,
            layout: small,
            pattern: 0x64,
        };
        block.fill();
        if !live.insert(block) {
            unsafe { heap.free(ptr, small) };
            break;
        }
    }
    let small_capacity = live.len;
    live.release(heap, &mut errors);

    let large = Layout::from_size_align(1024, 8).unwrap();
    let mut live = LiveSet::new(256);
    loop {
        let ptr = heap.alloc(large);
        if ptr.is_null() {
            break;
        }
        let block = Block {
            ptr,
            layout: large,
            pattern: 0x1C,
        };
        if live.overlaps_any(&block) {
            errors += 1;
        }
        block.fill();
        if !live.insert(block) {
            unsafe { heap.free(ptr, large) };
            break;
        }
    }
    let large_capacity = live.len;
    errors += live.corrupted();
    live.release(heap, &mut errors);

    Outcome {
        iterations: 2,
        errors,
        capacities: vec![small_capacity, large_capacity],
        used_after: used(heap),
        ..Outcome::default()
    }
}

/// New scenario: a long mixed workload with no artificial full release.
/// 100 000 operations, sizes 16..2048 B, alignments 1..64 B, up to 48 live
/// blocks (at most 96 KiB requested). A refused request is counted and the
/// workload goes on, as a kernel would have to.
pub fn long_run(heap: &mut dyn BenchHeap, seed: u64) -> Outcome {
    const ITERATIONS: u64 = 100_000;
    let mut rng = Rng::new(seed);
    let mut live = LiveSet::new(48);
    let mut errors = 0;
    let mut refused = 0;
    let mut worst_fragmentation = 0;

    for iteration in 0..ITERATIONS {
        if live.len == live.capacity() || (live.len > 0 && rng.next_u64() % 2 == 0) {
            let victim = rng.next_in_range(0, live.len - 1);
            if let Some(block) = live.take_nth(victim) {
                if !block.is_intact() || !block.is_aligned() {
                    errors += 1;
                }
                unsafe { heap.free(block.ptr, block.layout) };
            }
            continue;
        }
        // Mostly small requests, sometimes large ones, as in a kernel.
        let size = if rng.next_u64() % 8 == 0 {
            rng.next_in_range(512, 2048)
        } else {
            rng.next_in_range(16, 256)
        };
        let layout = Layout::from_size_align(size, rng.next_align()).unwrap();
        let ptr = heap.alloc(layout);
        if ptr.is_null() {
            refused += 1;
            continue;
        }
        let block = Block {
            ptr,
            layout,
            pattern: (iteration % 251) as u8,
        };
        if !block.is_aligned() || live.overlaps_any(&block) {
            errors += 1;
        }
        block.fill();
        live.insert(block);
        if let Some(stats) = heap.stats() {
            worst_fragmentation = worst_fragmentation.max(stats.fragmentation_permille());
        }
    }

    let stats = heap.stats();
    errors += live.corrupted();
    live.release(heap, &mut errors);
    Outcome {
        iterations: ITERATIONS,
        errors,
        exhaustions: refused,
        used_after: used(heap),
        live_bytes_after: stats.map(|s| s.live_bytes),
        fragmentation_permille: stats.map(|_| worst_fragmentation),
        ..Outcome::default()
    }
}
