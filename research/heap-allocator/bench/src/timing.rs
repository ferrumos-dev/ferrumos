//! Cost per operation, measured on the host.
//!
//! The same operation sequences are replayed without the correctness checks
//! (no fill, no overlap search) and every allocation and free is timed on its
//! own with `Instant`. Each measurement is repeated RUNS times on a fresh
//! heap; the median of each statistic over the runs is reported.
//! The clock overhead (~20-30 ns) is included in every figure and is the
//! same for every allocator.

use crate::heaps::{make, Kind};
use crate::scenarios::Rng;
use core::alloc::Layout;
use std::hint::black_box;
use std::time::Instant;

pub const RUNS: usize = 7;

#[derive(Debug, Clone, Copy, Default)]
pub struct Latency {
    pub ops: usize,
    pub p50_ns: u64,
    pub p99_ns: u64,
    pub max_ns: u64,
    pub mean_ns: u64,
}

fn summarize(mut samples: Vec<u64>) -> Latency {
    samples.sort_unstable();
    let n = samples.len();
    if n == 0 {
        return Latency::default();
    }
    Latency {
        ops: n,
        p50_ns: samples[n / 2],
        p99_ns: samples[(n * 99 / 100).min(n - 1)],
        max_ns: samples[n - 1],
        mean_ns: samples.iter().sum::<u64>() / n as u64,
    }
}

fn median_of(runs: &[Latency]) -> Latency {
    let pick = |f: fn(&Latency) -> u64| {
        let mut v: Vec<u64> = runs.iter().map(f).collect();
        v.sort_unstable();
        v[v.len() / 2]
    };
    Latency {
        ops: runs[0].ops,
        p50_ns: pick(|l| l.p50_ns),
        p99_ns: pick(|l| l.p99_ns),
        max_ns: pick(|l| l.max_ns),
        mean_ns: pick(|l| l.mean_ns),
    }
}

/// Workload of exp1 (seed 0x2026_1017), without the checks. A refused
/// request triggers a full release, as in exp1, so that the bump baseline
/// can run it too; full releases are not timed.
fn exp1_once(kind: Kind) -> Latency {
    let (_region, mut heap) = make(kind);
    let mut rng = Rng::new(0x2026_1017);
    let mut live: Vec<Option<(*mut u8, Layout)>> = vec![None; 32];
    let mut len = 0usize;
    let mut samples = Vec::with_capacity(20_000);

    for _ in 0..10_000u64 {
        if len == 32 || (len > 0 && rng.next_u64() % 3 == 0) {
            let victim = rng.next_in_range(0, len - 1);
            let slot = live.iter_mut().filter(|s| s.is_some()).nth(victim).unwrap();
            let (ptr, layout) = slot.take().unwrap();
            len -= 1;
            let t = Instant::now();
            unsafe { heap.free(ptr, layout) };
            samples.push(t.elapsed().as_nanos() as u64);
            continue;
        }
        let layout = Layout::from_size_align(rng.next_in_range(8, 512), rng.next_align()).unwrap();
        let t = Instant::now();
        let mut ptr = black_box(heap.alloc(layout));
        samples.push(t.elapsed().as_nanos() as u64);
        if ptr.is_null() {
            samples.pop();
            for slot in live.iter_mut() {
                if let Some((p, l)) = slot.take() {
                    unsafe { heap.free(p, l) };
                }
            }
            len = 0;
            ptr = heap.alloc(layout);
            if ptr.is_null() {
                continue;
            }
        }
        *live.iter_mut().find(|s| s.is_none()).unwrap() = Some((ptr, layout));
        len += 1;
    }
    summarize(samples)
}

/// Workload of long_run (mixed sizes up to 2 KiB, 48 live blocks, no full
/// release): this is where a list walk shows its cost.
fn long_run_once(kind: Kind) -> Latency {
    let (_region, mut heap) = make(kind);
    let mut rng = Rng::new(0x2026_1112);
    let mut live: Vec<Option<(*mut u8, Layout)>> = vec![None; 48];
    let mut len = 0usize;
    let mut samples = Vec::with_capacity(200_000);

    for _ in 0..100_000u64 {
        if len == 48 || (len > 0 && rng.next_u64() % 2 == 0) {
            let victim = rng.next_in_range(0, len - 1);
            let slot = live.iter_mut().filter(|s| s.is_some()).nth(victim).unwrap();
            let (ptr, layout) = slot.take().unwrap();
            len -= 1;
            let t = Instant::now();
            unsafe { heap.free(ptr, layout) };
            samples.push(t.elapsed().as_nanos() as u64);
            continue;
        }
        let size = if rng.next_u64() % 8 == 0 {
            rng.next_in_range(512, 2048)
        } else {
            rng.next_in_range(16, 256)
        };
        let layout = Layout::from_size_align(size, rng.next_align()).unwrap();
        let t = Instant::now();
        let ptr = black_box(heap.alloc(layout));
        samples.push(t.elapsed().as_nanos() as u64);
        if ptr.is_null() {
            continue;
        }
        *live.iter_mut().find(|s| s.is_none()).unwrap() = Some((ptr, layout));
        len += 1;
    }
    summarize(samples)
}

/// Worst single operation of the phase change: fill the heap with 64 B
/// blocks, free them all in a shuffled order (so that the free list is not
/// built in address order), then time the first 1 KiB allocation. For the
/// fixed-block allocator with reclaim, this is the call that drains every
/// class list into the fallback.
fn phase_change_once(kind: Kind) -> u64 {
    let (_region, mut heap) = make(kind);
    let small = Layout::from_size_align(64, 8).unwrap();
    let mut blocks = Vec::new();
    loop {
        let p = heap.alloc(small);
        if p.is_null() {
            break;
        }
        blocks.push(p);
    }
    let mut rng = Rng::new(0x2026_1112);
    for i in (1..blocks.len()).rev() {
        blocks.swap(i, rng.next_in_range(0, i));
    }
    for p in blocks {
        unsafe { heap.free(p, small) };
    }
    let large = Layout::from_size_align(1024, 8).unwrap();
    let t = Instant::now();
    let p = black_box(heap.alloc(large));
    let elapsed = t.elapsed().as_nanos() as u64;
    if p.is_null() {
        return 0;
    }
    elapsed
}

/// Median over RUNS of `phase_change_once`, 0 if the allocation is refused.
pub fn phase_change_first_large_ns(kind: Kind) -> u64 {
    let mut v: Vec<u64> = (0..RUNS).map(|_| phase_change_once(kind)).collect();
    v.sort_unstable();
    v[v.len() / 2]
}

pub fn exp1_latency(kind: Kind) -> Latency {
    let runs: Vec<Latency> = (0..RUNS).map(|_| exp1_once(kind)).collect();
    median_of(&runs)
}

pub fn long_run_latency(kind: Kind) -> Latency {
    let runs: Vec<Latency> = (0..RUNS).map(|_| long_run_once(kind)).collect();
    median_of(&runs)
}
