use super::*;

const HEAP_CAPACITY: u64 = 100 * 1024;

/// T03 of the test matrix: ticks, dispatches and context switches are counted
/// separately, and recording one never fabricates another.
#[test]
fn records_each_event_kind_independently() {
    let mut counters = Counters::default();
    assert_eq!(counters.timer_ticks(), 0);
    assert_eq!(counters.dispatches(), 0);
    assert_eq!(counters.context_switches(), 0);

    counters.on_timer_tick();
    counters.on_timer_tick();
    assert_eq!(counters.timer_ticks(), 2);
    assert_eq!(counters.dispatches(), 0);
    assert_eq!(counters.context_switches(), 0);

    counters.record_dispatch();
    assert_eq!(counters.timer_ticks(), 2);
    assert_eq!(counters.dispatches(), 1);
    assert_eq!(counters.context_switches(), 0);

    counters.record_context_switch();
    assert_eq!(counters.timer_ticks(), 2);
    assert_eq!(counters.dispatches(), 1);
    assert_eq!(counters.context_switches(), 1);
}

/// T04 of the test matrix: a counter at its integer limit keeps its value
/// instead of wrapping back toward zero.
#[test]
fn counters_reach_the_limit_and_saturate_independently() {
    let mut counters = Counters {
        timer_ticks: u64::MAX - 1,
        context_switches: u64::MAX - 1,
        dispatches: u64::MAX - 1,
    };

    for _ in 0..3 {
        counters.on_timer_tick();
    }
    assert_eq!(counters.timer_ticks(), u64::MAX);
    assert_eq!(counters.dispatches(), u64::MAX - 1);
    assert_eq!(counters.context_switches(), u64::MAX - 1);

    for _ in 0..3 {
        counters.record_dispatch();
    }
    assert_eq!(counters.timer_ticks(), u64::MAX);
    assert_eq!(counters.dispatches(), u64::MAX);
    assert_eq!(counters.context_switches(), u64::MAX - 1);

    for _ in 0..3 {
        counters.record_context_switch();
    }
    assert_eq!(counters.timer_ticks(), u64::MAX);
    assert_eq!(counters.dispatches(), u64::MAX);
    assert_eq!(counters.context_switches(), u64::MAX);
}

struct Reporter {
    memory: Option<MemorySnapshot>,
    runnable_tasks: usize,
    counters: Counters,
}

impl KernelReporter for Reporter {
    fn kernel_snapshot(&self) -> KernelSnapshot {
        KernelSnapshot {
            memory: self.memory,
            runnable_tasks: self.runnable_tasks,
            counters: self.counters,
        }
    }
}

#[test]
fn snapshot_retains_its_sample_after_provider_state_changes() {
    let mut reporter = Reporter {
        memory: MemorySnapshot::new(16, 6),
        runnable_tasks: 2,
        counters: Counters::default(),
    };
    reporter.counters.record_dispatch();
    let before = reporter.kernel_snapshot();

    reporter.memory = None;
    reporter.runnable_tasks = 0;
    reporter.counters.on_timer_tick();
    let after = reporter.kernel_snapshot();

    assert_eq!(before.memory.map(MemorySnapshot::heap_free_bytes), Some(10));
    assert_eq!(before.runnable_tasks, 2);
    assert_eq!(before.counters.timer_ticks(), 0);
    assert_eq!(before.counters.dispatches(), 1);
    assert_eq!(after.memory, None);
    assert_eq!(after.runnable_tasks, 0);
    assert_eq!(after.counters.timer_ticks(), 1);
    assert_eq!(after.counters.dispatches(), 1);
}

#[test]
fn a_snapshot_keeps_an_unavailable_heap_apart_from_an_empty_one() {
    let unavailable = KernelSnapshot {
        memory: None,
        runnable_tasks: 0,
        counters: Counters::default(),
    };
    let empty = KernelSnapshot {
        memory: MemorySnapshot::new(0, 0),
        runnable_tasks: 0,
        counters: Counters::default(),
    };

    assert_eq!(unavailable.memory, None);
    assert!(empty.memory.is_some());
    assert_ne!(unavailable, empty);
}

#[test]
fn a_reporter_returns_the_heap_it_measured() {
    let reporter = Reporter {
        memory: MemorySnapshot::new(HEAP_CAPACITY, 2048),
        runnable_tasks: 4,
        counters: Counters::default(),
    };

    let snapshot = reporter.kernel_snapshot();

    assert_eq!(snapshot.runnable_tasks, 4);
    assert_eq!(
        snapshot
            .memory
            .expect("the heap is measured")
            .heap_free_bytes(),
        HEAP_CAPACITY - 2048
    );
}

#[test]
fn uptime_keeps_the_duration_it_was_given() {
    let uptime = Uptime::from_milliseconds(5_492);

    assert_eq!(uptime.milliseconds(), 5_492);
    assert_eq!(uptime.seconds(), 5);
}

#[test]
fn uptime_splits_into_hours_minutes_seconds_and_milliseconds() {
    let parts = Uptime::from_milliseconds(3_723_456).parts();

    assert_eq!(parts.hours, 1);
    assert_eq!(parts.minutes, 2);
    assert_eq!(parts.seconds, 3);
    assert_eq!(parts.milliseconds, 456);
}

#[test]
fn a_kernel_that_just_booted_has_a_zero_uptime_rather_than_no_uptime() {
    let just_booted = Uptime::from_milliseconds(0);
    let parts = just_booted.parts();

    assert_eq!(just_booted.milliseconds(), 0);
    assert_eq!(parts.hours, 0);
    assert_eq!(parts.minutes, 0);
    assert_eq!(parts.seconds, 0);
    assert_eq!(parts.milliseconds, 0);
}

#[test]
fn uptime_splits_a_duration_at_the_integer_limit() {
    let parts = Uptime::from_milliseconds(u64::MAX).parts();

    assert_eq!(parts.milliseconds, u64::MAX % 1_000);
    assert_eq!(parts.seconds, (u64::MAX / 1_000) % 60);
    assert_eq!(parts.minutes, (u64::MAX / 60_000) % 60);
    assert_eq!(parts.hours, u64::MAX / 3_600_000);
}
