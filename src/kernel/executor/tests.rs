use super::{CooperativeExecutor, TaskError, TaskStatus};
use crate::kernel::scheduler::{Priority, TaskId};

#[derive(Default)]
struct Trace {
    values: [u8; 8],
    len: usize,
}

impl Trace {
    fn push(&mut self, value: u8) {
        self.values[self.len] = value;
        self.len += 1;
    }
}

fn high(trace: &mut Trace) -> TaskStatus {
    trace.push(1);
    TaskStatus::Complete
}

fn low(trace: &mut Trace) -> TaskStatus {
    trace.push(2);
    TaskStatus::Complete
}

fn block(trace: &mut Trace) -> TaskStatus {
    trace.push(3);
    TaskStatus::Blocked
}

fn yield_task(trace: &mut Trace) -> TaskStatus {
    trace.push(4);
    TaskStatus::Yield
}

#[test]
fn invokes_real_callbacks_by_priority_and_releases_completed_slots() {
    let mut executor = CooperativeExecutor::<Trace, 2>::new();
    let mut trace = Trace::default();
    executor.register(TaskId(2), Priority::Low, low).unwrap();
    executor.register(TaskId(1), Priority::High, high).unwrap();
    assert_eq!(executor.run_one(&mut trace).unwrap().task, TaskId(1));
    assert_eq!(executor.run_one(&mut trace).unwrap().task, TaskId(2));
    assert_eq!(executor.run_one(&mut trace), None);
    assert_eq!(&trace.values[..trace.len], &[1, 2]);
    assert_eq!(executor.task_count(), 0);
    executor
        .register(TaskId(1), Priority::Normal, high)
        .unwrap();
    assert_eq!(
        executor.run_one(&mut trace).unwrap().status,
        TaskStatus::Complete
    );
    assert_eq!(executor.counters().dispatches(), 3);
    assert_eq!(executor.counters().context_switches(), 0);
    assert_eq!(executor.counters().timer_ticks(), 0);
    assert_eq!(executor.wake(TaskId(1)), Err(TaskError::Unknown));
}

#[test]
fn blocked_task_keeps_its_registration_and_wakes_behind_ready_peers() {
    let mut executor = CooperativeExecutor::<Trace, 2>::new();
    let mut trace = Trace::default();
    executor
        .register(TaskId(1), Priority::Normal, block)
        .unwrap();
    executor.register(TaskId(2), Priority::Normal, low).unwrap();
    assert_eq!(
        executor.run_one(&mut trace).unwrap().status,
        TaskStatus::Blocked
    );
    assert_eq!(executor.task_count(), 2);
    assert_eq!(executor.runnable_count(), 1);
    assert_eq!(
        executor.register(TaskId(1), Priority::High, high),
        Err(TaskError::Duplicate)
    );
    assert_eq!(
        executor.register(TaskId(3), Priority::High, high),
        Err(TaskError::Full)
    );
    assert_eq!(executor.wake(TaskId(1)), Ok(true));
    assert_eq!(executor.wake(TaskId(1)), Ok(false));
    assert_eq!(executor.run_one(&mut trace).unwrap().task, TaskId(2));
    assert_eq!(executor.run_one(&mut trace).unwrap().task, TaskId(1));
    assert_eq!(executor.run_one(&mut trace), None);
    assert_eq!(&trace.values[..trace.len], &[3, 2, 3]);
}

#[test]
fn yielding_moves_a_task_behind_its_peers_but_keeps_priority() {
    let mut executor = CooperativeExecutor::<Trace, 3>::new();
    let mut trace = Trace::default();
    executor
        .register(TaskId(1), Priority::High, yield_task)
        .unwrap();
    executor.register(TaskId(2), Priority::High, high).unwrap();
    executor.register(TaskId(3), Priority::Low, low).unwrap();
    for id in [1, 2, 1, 1] {
        assert_eq!(executor.run_one(&mut trace).unwrap().task, TaskId(id));
    }
    assert_eq!(&trace.values[..trace.len], &[4, 1, 4, 4]);
    assert_eq!(executor.runnable_count(), 2);
}

#[test]
fn zero_capacity_rejects_registration_and_idle_dispatch_does_not_count() {
    let mut executor = CooperativeExecutor::<Trace, 0>::default();
    let mut trace = Trace::default();
    assert_eq!(
        executor.register(TaskId(1), Priority::High, high),
        Err(TaskError::Full)
    );
    assert_eq!(executor.wake(TaskId(1)), Err(TaskError::Unknown));
    assert_eq!(executor.run_one(&mut trace), None);
    assert_eq!(executor.task_count(), 0);
    assert_eq!(executor.runnable_count(), 0);
    assert_eq!(executor.counters().dispatches(), 0);
    assert_eq!(trace.len, 0);
}
