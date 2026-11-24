//! Allocation-free cooperative execution of bounded kernel task steps.
//!
//! The scheduler chooses an identifier; this module invokes its registered
//! function on the boot CPU. Each function borrows an explicit context and
//! returns its next state. Functions share the caller's stack, so they must
//! finish each step promptly. Hardware preemption and separate CPU stacks need
//! an architecture-specific executor with additional ownership invariants.

use crate::kernel::scheduler::{Priority, PriorityScheduler, RegisterError, TaskId};
use crate::kernel::telemetry::Counters;

/// State requested by a task after one cooperative execution step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Remain ready, after the other tasks of the same priority.
    Yield,
    /// Stop running until the owner explicitly wakes this task.
    Blocked,
    /// Release the task's registration and ready-queue slot.
    Complete,
}

/// One step of actual task execution, suitable for diagnostics and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dispatch {
    /// Identifier of the function that was invoked.
    pub task: TaskId,
    /// State returned by the task function.
    pub status: TaskStatus,
}

/// Registration or wakeup failed without modifying the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskError {
    /// This identifier is already owned, including by a blocked task.
    Duplicate,
    /// All task slots are owned, including slots of blocked tasks.
    Full,
    /// No live task owns the identifier passed to `wake`.
    Unknown,
}

struct Task<C> {
    id: TaskId,
    priority: Priority,
    step: fn(&mut C) -> TaskStatus,
}

/// Fixed-capacity task registry and its strict-priority ready queue.
///
/// The owner holds exclusive access while dispatching or waking tasks. The
/// context contains shared resources, such as the boot console, so tasks cannot
/// accidentally construct a second owner of a hardware device. No callback
/// receives access to the executor itself during its execution.
pub struct CooperativeExecutor<C, const N: usize> {
    tasks: [Option<Task<C>>; N],
    ready: PriorityScheduler<N>,
    counters: Counters,
}

impl<C, const N: usize> Default for CooperativeExecutor<C, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, const N: usize> CooperativeExecutor<C, N> {
    /// Create an empty executor using only its inline task storage.
    pub fn new() -> Self {
        Self {
            tasks: core::array::from_fn(|_| None),
            ready: PriorityScheduler::new(),
            counters: Counters::default(),
        }
    }

    /// Register an initially ready function at a fixed priority.
    ///
    /// A rejected registration preserves both the registry and the ready queue.
    /// Blocked tasks keep their slots and identifiers until completion.
    pub fn register(
        &mut self,
        id: TaskId,
        priority: Priority,
        step: fn(&mut C) -> TaskStatus,
    ) -> Result<(), TaskError> {
        if self.tasks.iter().flatten().any(|task| task.id == id) {
            return Err(TaskError::Duplicate);
        }
        let slot = self
            .tasks
            .iter()
            .position(Option::is_none)
            .ok_or(TaskError::Full)?;

        // Reserve the ready entry first. There can be no fallible operation
        // after it succeeds: the registry slot was already found above.
        self.ready
            .register(id, priority)
            .map_err(|error| match error {
                RegisterError::Duplicate => TaskError::Duplicate,
                RegisterError::Full => TaskError::Full,
            })?;
        self.tasks[slot] = Some(Task { id, priority, step });
        Ok(())
    }

    /// Wake a blocked task, appending it after its ready peers.
    ///
    /// Returns `Ok(true)` for a newly ready task, `Ok(false)` for one already
    /// ready, or `Unknown` after completion. Repeated wakeups preserve FIFO order.
    pub fn wake(&mut self, id: TaskId) -> Result<bool, TaskError> {
        let task = self
            .tasks
            .iter()
            .flatten()
            .find(|task| task.id == id)
            .ok_or(TaskError::Unknown)?;
        if self.ready.contains(id) {
            return Ok(false);
        }
        self.ready
            .register(id, task.priority)
            .map_err(|error| match error {
                RegisterError::Duplicate => TaskError::Duplicate,
                RegisterError::Full => TaskError::Full,
            })?;
        Ok(true)
    }

    /// Invoke one ready function and apply the state it returns.
    ///
    /// Returns `None` when no task is runnable. Dispatching counts an actual
    /// function call; it does not count a CPU context switch or a timer tick.
    /// A task that fails to return can delay every other cooperative task.
    pub fn run_one(&mut self, context: &mut C) -> Option<Dispatch> {
        let id = self.ready.select_next()?;
        let slot = self
            .tasks
            .iter()
            .position(|task| task.as_ref().is_some_and(|task| task.id == id))
            .expect("ready identifiers belong to registered tasks");
        let step = self.tasks[slot]
            .as_ref()
            .expect("registered task slot")
            .step;
        self.counters.record_dispatch();
        let status = step(context);
        match status {
            TaskStatus::Yield => {
                self.ready.yield_now(id);
            }
            TaskStatus::Blocked => {
                // Keep the function and its priority, but stop selecting it
                // until the external owner has a reason to wake it.
                self.ready.remove(id);
            }
            TaskStatus::Complete => {
                self.ready.remove(id);
                self.tasks[slot] = None;
            }
        }
        Some(Dispatch { task: id, status })
    }

    /// Number of live task registrations, including blocked tasks.
    pub fn task_count(&self) -> usize {
        self.tasks.iter().flatten().count()
    }

    /// Number of tasks eligible for a call to `run_one`.
    pub fn runnable_count(&self) -> usize {
        self.ready.runnable_count()
    }

    /// Copy the executor's counters while the owner holds its shared borrow.
    pub fn counters(&self) -> Counters {
        self.counters
    }
}

#[cfg(test)]
mod tests;
