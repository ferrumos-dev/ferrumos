use super::MemorySnapshot;

#[test]
fn rejects_usage_above_capacity_without_overflow() {
    for (capacity, used) in [(0, 1), (16, 17), (u64::MAX - 1, u64::MAX)] {
        assert_eq!(MemorySnapshot::new(capacity, used), None);
    }
}

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
