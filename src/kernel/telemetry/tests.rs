use super::{Counters, KernelReporter, KernelSnapshot};
use crate::kernel::interrupts::TimerTickSink;
use crate::kernel::memory::MemorySnapshot;

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
