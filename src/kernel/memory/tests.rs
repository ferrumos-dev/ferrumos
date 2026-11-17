use super::*;

const HEAP_CAPACITY: u64 = 100 * 1024;

#[test]
fn a_snapshot_keeps_the_capacity_and_the_usage_it_was_given() {
    let snapshot = MemorySnapshot::new(HEAP_CAPACITY, 41_984).expect("coherent values");

    assert_eq!(snapshot.heap_capacity_bytes(), HEAP_CAPACITY);
    assert_eq!(snapshot.heap_used_bytes(), 41_984);
}

#[test]
fn free_bytes_complements_used_bytes() {
    let snapshot = MemorySnapshot::new(HEAP_CAPACITY, 41_984).expect("coherent values");

    assert_eq!(snapshot.heap_free_bytes(), HEAP_CAPACITY - 41_984);
    assert_eq!(
        snapshot.heap_free_bytes() + snapshot.heap_used_bytes(),
        snapshot.heap_capacity_bytes()
    );
}

/// T02 of the test matrix: `(16, 6)` gives 10 and `(0, 0)` gives 0.
#[test]
fn t02_free_space_is_the_difference_between_capacity_and_usage() {
    let measured = MemorySnapshot::new(16, 6).expect("coherent values");
    let empty = MemorySnapshot::new(0, 0).expect("an empty heap is coherent");

    assert_eq!(measured.heap_free_bytes(), 10);
    assert_eq!(empty.heap_free_bytes(), 0);
    assert_eq!(empty.heap_capacity_bytes(), 0);
}

#[test]
fn a_full_heap_leaves_nothing_free() {
    let snapshot = MemorySnapshot::new(HEAP_CAPACITY, HEAP_CAPACITY).expect("coherent values");

    assert_eq!(snapshot.heap_free_bytes(), 0);
    assert_eq!(snapshot.usage_permille(), Some(1000));
}

/// T01 of the test matrix: a usage of 17 for a capacity of 16 is rejected.
#[test]
fn t01_a_usage_greater_than_the_capacity_produces_no_snapshot() {
    assert_eq!(MemorySnapshot::new(16, 17), None);
    assert_eq!(MemorySnapshot::new(1024, 2048), None);
}

#[test]
fn usage_share_is_unavailable_without_capacity() {
    let snapshot = MemorySnapshot::new(0, 0).expect("coherent values");

    assert_eq!(snapshot.usage_permille(), None);
}

#[test]
fn usage_share_is_expressed_in_parts_per_thousand() {
    let snapshot = MemorySnapshot::new(1000, 410).expect("coherent values");

    assert_eq!(snapshot.usage_permille(), Some(410));
}

#[test]
fn usage_share_does_not_overflow_on_a_large_heap() {
    let snapshot = MemorySnapshot::new(u64::MAX, u64::MAX / 2).expect("coherent values");

    assert_eq!(snapshot.usage_permille(), Some(499));
}

#[test]
fn an_unavailable_heap_reports_no_snapshot() {
    struct UninitializedHeap;

    impl MemoryReporter for UninitializedHeap {
        fn memory_snapshot(&self) -> Option<MemorySnapshot> {
            None
        }
    }

    assert_eq!(UninitializedHeap.memory_snapshot(), None);
}

#[test]
fn a_reporter_returns_the_values_it_measured() {
    struct Heap {
        used_bytes: u64,
    }

    impl MemoryReporter for Heap {
        fn memory_snapshot(&self) -> Option<MemorySnapshot> {
            MemorySnapshot::new(HEAP_CAPACITY, self.used_bytes)
        }
    }

    let snapshot = Heap { used_bytes: 2048 }
        .memory_snapshot()
        .expect("the heap is initialized");

    assert_eq!(snapshot.heap_used_bytes(), 2048);
    assert_eq!(snapshot.heap_free_bytes(), HEAP_CAPACITY - 2048);
}

#[test]
fn an_incoherent_reading_reports_no_snapshot_rather_than_a_wrong_one() {
    struct TornHeap;

    impl MemoryReporter for TornHeap {
        fn memory_snapshot(&self) -> Option<MemorySnapshot> {
            MemorySnapshot::new(1024, 4096)
        }
    }

    assert_eq!(TornHeap.memory_snapshot(), None);
}
