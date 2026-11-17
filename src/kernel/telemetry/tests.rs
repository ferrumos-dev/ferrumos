use super::*;

const TICK_PERIOD_MICROSECONDS: u64 = 54_925;
const HEAP_CAPACITY: u64 = 100 * 1024;

#[test]
fn new_counters_have_recorded_nothing() {
    let counters = Counters::new();

    assert_eq!(counters.timer_ticks(), 0);
    assert_eq!(counters.context_switches(), 0);
    assert_eq!(counters, Counters::default());
}

/// T03 of the test matrix: two ticks and one declared context switch give 2
/// and 1.
#[test]
fn t03_ticks_and_context_switches_are_counted_separately() {
    let mut counters = Counters::new();

    counters.on_timer_tick();
    counters.on_timer_tick();
    counters.record_context_switch();

    assert_eq!(counters.timer_ticks(), 2);
    assert_eq!(counters.context_switches(), 1);
}

#[test]
fn a_context_switch_does_not_add_a_tick() {
    let mut counters = Counters::new();

    counters.record_context_switch();

    assert_eq!(counters.context_switches(), 1);
    assert_eq!(counters.timer_ticks(), 0);
}

/// T04 of the test matrix: a counter at `u64::MAX` keeps its value.
#[test]
fn t04_counters_saturate_instead_of_wrapping() {
    let mut counters = Counters {
        timer_ticks: u64::MAX,
        context_switches: u64::MAX,
    };

    counters.on_timer_tick();
    counters.record_context_switch();

    assert_eq!(counters.timer_ticks(), u64::MAX);
    assert_eq!(counters.context_switches(), u64::MAX);
}

#[test]
fn uptime_is_derived_from_the_tick_period() {
    let uptime = Uptime::from_ticks(100, TICK_PERIOD_MICROSECONDS).expect("the period is known");

    assert_eq!(uptime.milliseconds(), 5_492);
    assert_eq!(uptime.seconds(), 5);
}

#[test]
fn uptime_is_unavailable_without_a_tick_period() {
    assert_eq!(Uptime::from_ticks(100, 0), None);
}

#[test]
fn an_uncalibrated_timer_differs_from_a_kernel_that_just_booted() {
    let unknown = Uptime::from_ticks(100, 0);
    let just_booted = Uptime::from_ticks(0, TICK_PERIOD_MICROSECONDS);

    assert_eq!(unknown, None);
    assert_eq!(just_booted, Some(Uptime::from_milliseconds(0)));
    assert_ne!(unknown, just_booted);
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
fn uptime_does_not_overflow_on_a_very_long_run() {
    let uptime =
        Uptime::from_ticks(u64::MAX, TICK_PERIOD_MICROSECONDS).expect("the period is known");

    assert_eq!(
        uptime.milliseconds(),
        u64::MAX / MICROSECONDS_PER_MILLISECOND
    );
}

#[test]
fn a_snapshot_keeps_an_unavailable_heap_apart_from_an_empty_one() {
    let unavailable = KernelSnapshot {
        memory: None,
        runnable_tasks: 0,
        counters: Counters::new(),
    };
    let empty = KernelSnapshot {
        memory: MemorySnapshot::new(0, 0),
        runnable_tasks: 0,
        counters: Counters::new(),
    };

    assert_eq!(unavailable.memory, None);
    assert!(empty.memory.is_some());
    assert_ne!(unavailable, empty);
}

#[test]
fn a_reporter_returns_the_values_it_gathered() {
    struct Kernel;

    impl KernelReporter for Kernel {
        fn kernel_snapshot(&self) -> KernelSnapshot {
            let mut counters = Counters::new();
            counters.on_timer_tick();

            KernelSnapshot {
                memory: MemorySnapshot::new(HEAP_CAPACITY, 2048),
                runnable_tasks: 4,
                counters,
            }
        }
    }

    let snapshot = Kernel.kernel_snapshot();

    assert_eq!(snapshot.counters.timer_ticks(), 1);
    assert_eq!(snapshot.counters.context_switches(), 0);
    assert_eq!(snapshot.runnable_tasks, 4);
    assert_eq!(
        snapshot
            .memory
            .expect("the heap is measured")
            .heap_free_bytes(),
        HEAP_CAPACITY - 2048
    );
}
