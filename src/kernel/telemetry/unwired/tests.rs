use super::*;

#[test]
fn an_unwired_kernel_cannot_measure_its_heap() {
    assert_eq!(UnwiredKernel.kernel_snapshot().memory, None);
}

#[test]
fn an_unwired_kernel_has_recorded_no_event() {
    let counters = UnwiredKernel.kernel_snapshot().counters;

    assert_eq!(counters.timer_ticks(), 0);
    assert_eq!(counters.context_switches(), 0);
}

#[test]
fn an_unwired_kernel_runs_no_task() {
    assert_eq!(UnwiredKernel.kernel_snapshot().runnable_tasks, 0);
}

#[test]
fn two_readings_of_an_unwired_kernel_agree() {
    assert_eq!(
        UnwiredKernel.kernel_snapshot(),
        UnwiredKernel.kernel_snapshot()
    );
}
