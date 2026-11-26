use super::*;

const HEAP_CAPACITY: u64 = 100 * 1024;

struct Heap {
    used_bytes: Option<u64>,
}

impl MemoryReporter for Heap {
    fn memory_snapshot(&self) -> Option<MemorySnapshot> {
        MemorySnapshot::new(HEAP_CAPACITY, self.used_bytes?)
    }
}

fn counters() -> Counters {
    let mut counters = Counters::default();
    counters.record_dispatch();
    counters.record_context_switch();
    counters
}

#[test]
fn a_capture_reports_the_heap_its_source_measured() {
    let heap = Heap {
        used_bytes: Some(2048),
    };

    let snapshot = CapturedKernel::capture(&heap, 3, counters()).kernel_snapshot();

    let memory = snapshot.memory.expect("the heap is initialized");
    assert_eq!(memory.heap_capacity_bytes(), HEAP_CAPACITY);
    assert_eq!(memory.heap_used_bytes(), 2048);
    assert_eq!(snapshot.runnable_tasks, 3);
    assert_eq!(snapshot.counters.dispatches(), 1);
    assert_eq!(snapshot.counters.context_switches(), 1);
}

#[test]
fn an_unmeasurable_heap_is_reported_as_unavailable_rather_than_empty() {
    let heap = Heap { used_bytes: None };

    let snapshot = CapturedKernel::capture(&heap, 0, Counters::default()).kernel_snapshot();

    assert_eq!(snapshot.memory, None);
    assert_eq!(snapshot.runnable_tasks, 0);
}

#[test]
fn a_capture_keeps_its_sample_while_the_sources_keep_changing() {
    let mut heap = Heap {
        used_bytes: Some(2048),
    };
    let captured = CapturedKernel::capture(&heap, 3, counters());

    heap.used_bytes = Some(8192);
    let later = CapturedKernel::capture(&heap, 1, Counters::default());

    assert_eq!(
        captured
            .kernel_snapshot()
            .memory
            .expect("the first reading measured the heap")
            .heap_used_bytes(),
        2048
    );
    assert_eq!(captured.kernel_snapshot().runnable_tasks, 3);
    assert_ne!(captured, later);
}

#[test]
fn an_imported_tick_total_reaches_the_snapshot_with_the_execution_counts() {
    let heap = Heap {
        used_bytes: Some(0),
    };

    let snapshot =
        CapturedKernel::capture(&heap, 2, counters().with_timer_ticks(417)).kernel_snapshot();

    assert_eq!(snapshot.counters.timer_ticks(), 417);
    assert_eq!(snapshot.counters.dispatches(), 1);
    assert_eq!(snapshot.counters.context_switches(), 1);
}

#[test]
fn two_reads_of_one_capture_agree() {
    let heap = Heap {
        used_bytes: Some(4096),
    };
    let captured = CapturedKernel::capture(&heap, 2, counters());

    assert_eq!(captured.kernel_snapshot(), captured.kernel_snapshot());
}

#[test]
fn a_heap_reading_taken_earlier_can_be_passed_directly() {
    let memory = MemorySnapshot::new(HEAP_CAPACITY, 1024);

    let snapshot = CapturedKernel::from_memory(memory, 1, Counters::default()).kernel_snapshot();

    assert_eq!(snapshot.memory, memory);
    assert_eq!(snapshot.runnable_tasks, 1);
}
