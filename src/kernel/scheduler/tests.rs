use super::{Priority, PriorityScheduler, RegisterError, TaskId};

#[test]
fn empty_and_zero_capacity_queues_have_no_runnable_task() {
    let mut empty = PriorityScheduler::<3>::new();
    assert_eq!(empty.select_next(), None);
    assert_eq!(empty.runnable_count(), 0);
    assert!(!empty.contains(TaskId(1)));
    assert!(!empty.remove(TaskId(1)));
    assert!(!empty.yield_now(TaskId(1)));

    let mut zero = PriorityScheduler::<0>::default();
    assert_eq!(
        zero.register(TaskId(1), Priority::High),
        Err(RegisterError::Full)
    );
    assert_eq!(zero.select_next(), None);
    assert_eq!(zero.runnable_count(), 0);
    assert!(!zero.remove(TaskId(1)));
    assert!(!zero.yield_now(TaskId(1)));
}

#[test]
fn highest_priority_wins_independently_of_registration_order() {
    let mut scheduler = PriorityScheduler::<3>::new();
    let low = TaskId(30);
    let normal = TaskId(20);
    let high = TaskId(10);
    assert_eq!(scheduler.register(low, Priority::Low), Ok(()));
    assert_eq!(scheduler.register(normal, Priority::Normal), Ok(()));
    assert_eq!(scheduler.register(high, Priority::High), Ok(()));
    assert_eq!(scheduler.select_next(), Some(high));
    assert!(scheduler.remove(high));
    assert_eq!(scheduler.select_next(), Some(normal));
    assert!(scheduler.remove(normal));
    assert_eq!(scheduler.select_next(), Some(low));
    assert_eq!(scheduler.runnable_count(), 1);
}

#[test]
fn selection_preserves_fifo_until_the_selected_task_yields() {
    let mut scheduler = PriorityScheduler::<3>::new();
    // Deliberately register a larger identifier first. FIFO follows arrival,
    // rather than comparing numeric task identities.
    let first = TaskId(9);
    let second = TaskId(2);
    let third = TaskId(1);
    for id in [first, second, third] {
        assert_eq!(scheduler.register(id, Priority::Normal), Ok(()));
    }
    assert_eq!(scheduler.select_next(), Some(first));
    assert_eq!(scheduler.select_next(), Some(first));
    assert!(scheduler.yield_now(first));
    assert_eq!(scheduler.select_next(), Some(second));
    assert!(scheduler.yield_now(second));
    assert_eq!(scheduler.select_next(), Some(third));
    assert!(scheduler.yield_now(third));
    assert_eq!(scheduler.select_next(), Some(first));
    assert_eq!(scheduler.runnable_count(), 3);
}

#[test]
fn higher_priority_arrival_preserves_the_previous_fifo_head() {
    let mut scheduler = PriorityScheduler::<3>::new();
    let first = TaskId(1);
    let second = TaskId(2);
    let urgent = TaskId(3);
    assert_eq!(scheduler.register(first, Priority::Normal), Ok(()));
    assert_eq!(scheduler.register(second, Priority::Normal), Ok(()));
    assert_eq!(scheduler.select_next(), Some(first));
    assert_eq!(scheduler.register(urgent, Priority::High), Ok(()));
    assert_eq!(scheduler.select_next(), Some(urgent));
    assert!(scheduler.remove(urgent));
    assert_eq!(scheduler.select_next(), Some(first));
    assert!(scheduler.yield_now(first));
    assert_eq!(scheduler.select_next(), Some(second));
}

#[test]
fn a_waking_task_rejoins_after_its_older_peers() {
    let mut scheduler = PriorityScheduler::<3>::new();
    let waking = TaskId(1);
    let older = TaskId(2);
    let newest = TaskId(3);
    assert_eq!(scheduler.register(waking, Priority::Normal), Ok(()));
    assert_eq!(scheduler.register(older, Priority::Normal), Ok(()));
    assert!(scheduler.remove(waking));
    assert!(!scheduler.contains(waking));
    assert_eq!(scheduler.register(newest, Priority::Normal), Ok(()));
    assert_eq!(scheduler.register(waking, Priority::Normal), Ok(()));
    for expected in [older, newest, waking] {
        assert_eq!(scheduler.select_next(), Some(expected));
        assert!(scheduler.remove(expected));
    }
    assert_eq!(scheduler.select_next(), None);
}

#[test]
fn rejected_registration_preserves_priority_count_and_fifo_order() {
    let mut scheduler = PriorityScheduler::<2>::new();
    let first = TaskId(1);
    let second = TaskId(2);
    assert_eq!(scheduler.register(first, Priority::Normal), Ok(()));
    assert_eq!(scheduler.register(second, Priority::Normal), Ok(()));
    // A duplicate remains a duplicate when the queue is full, and attempting
    // to re-register it at High must not silently change its priority.
    assert_eq!(
        scheduler.register(second, Priority::High),
        Err(RegisterError::Duplicate)
    );
    assert_eq!(
        scheduler.register(TaskId(3), Priority::High),
        Err(RegisterError::Full)
    );
    assert_eq!(scheduler.runnable_count(), 2);
    assert_eq!(scheduler.select_next(), Some(first));
    assert!(!scheduler.contains(TaskId(3)));
    assert!(scheduler.remove(first));
    assert_eq!(scheduler.select_next(), Some(second));
    assert_eq!(scheduler.register(TaskId(3), Priority::Normal), Ok(()));
    assert_eq!(scheduler.select_next(), Some(second));
}

#[test]
fn interior_removal_and_slot_reuse_keep_survivors_in_order() {
    let mut scheduler = PriorityScheduler::<3>::new();
    for number in 1..=3 {
        assert_eq!(scheduler.register(TaskId(number), Priority::Low), Ok(()));
    }
    assert!(scheduler.remove(TaskId(2)));
    assert!(!scheduler.remove(TaskId(2)));
    assert_eq!(scheduler.register(TaskId(4), Priority::Low), Ok(()));
    for expected in [1, 3, 4] {
        assert_eq!(scheduler.select_next(), Some(TaskId(expected)));
        assert!(scheduler.remove(TaskId(expected)));
    }
    assert_eq!(scheduler.runnable_count(), 0);
    assert_eq!(scheduler.register(TaskId(5), Priority::High), Ok(()));
    assert_eq!(scheduler.select_next(), Some(TaskId(5)));
}

#[test]
fn yield_of_absent_or_single_task_has_explicit_semantics() {
    let mut scheduler = PriorityScheduler::<1>::new();
    let only = TaskId(1);
    assert_eq!(scheduler.register(only, Priority::Normal), Ok(()));
    assert!(!scheduler.yield_now(TaskId(2)));
    assert!(scheduler.yield_now(only));
    assert_eq!(scheduler.select_next(), Some(only));
    assert_eq!(scheduler.runnable_count(), 1);
}

#[test]
fn yielding_high_priority_task_still_precedes_lower_priority() {
    let mut scheduler = PriorityScheduler::<2>::new();
    let high = TaskId(1);
    let low = TaskId(2);
    assert_eq!(scheduler.register(high, Priority::High), Ok(()));
    assert_eq!(scheduler.register(low, Priority::Low), Ok(()));
    for _ in 0..4 {
        assert_eq!(scheduler.select_next(), Some(high));
        assert!(scheduler.yield_now(high));
    }
    // Strict priority deliberately allows this starvation scenario. A yield
    // gives the turn to a peer at the same priority when one exists.
    assert!(scheduler.remove(high));
    assert_eq!(scheduler.select_next(), Some(low));
}
