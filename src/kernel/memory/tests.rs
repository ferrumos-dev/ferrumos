use super::*;

const HEAP_CAPACITY: u64 = 100 * 1024;

/// T01 of the test matrix: a usage greater than the capacity is rejected
/// instead of producing a snapshot that would underflow.
#[test]
fn rejects_usage_above_capacity_without_overflow() {
    for (capacity, used) in [(0, 1), (16, 17), (u64::MAX - 1, u64::MAX)] {
        assert_eq!(MemorySnapshot::new(capacity, used), None);
    }
}

/// T02 of the test matrix: `(16, 6)` leaves 10 free and `(0, 0)` leaves 0.
#[test]
fn preserves_units_and_computes_free_bytes_at_boundaries() {
    for (capacity, used, free) in [
        (0, 0, 0),
        (16, 6, 10),
        (16, 16, 0),
        (u64::MAX, 0, u64::MAX),
        (u64::MAX, u64::MAX, 0),
    ] {
        let snapshot = MemorySnapshot::new(capacity, used).unwrap();
        assert_eq!(snapshot.heap_capacity_bytes(), capacity);
        assert_eq!(snapshot.heap_used_bytes(), used);
        assert_eq!(snapshot.heap_free_bytes(), free);
    }
}

#[test]
fn a_full_heap_reports_a_complete_usage_share() {
    let snapshot = MemorySnapshot::new(HEAP_CAPACITY, HEAP_CAPACITY).expect("coherent values");

    assert_eq!(snapshot.heap_free_bytes(), 0);
    assert_eq!(snapshot.usage_permille(), Some(1000));
}

#[test]
fn usage_share_is_unavailable_without_capacity() {
    let snapshot = MemorySnapshot::new(0, 0).expect("an empty heap is coherent");

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
