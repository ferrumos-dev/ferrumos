//! Fixed-priority selection with first-in, first-out order within each priority.
//!
//! The executor registers runnable tasks, removes tasks that block or finish,
//! and registers a waking task again at the back of its priority. A voluntary
//! yield also moves a task to the back. Selection itself leaves the queue
//! unchanged, so a task keeps its place when a higher-priority task arrives.
//!
//! This policy can starve lower priorities when a higher-priority task remains
//! runnable. Task state, CPU context ownership and synchronization belong to
//! the executor. The scheduler stores identifiers and makes a decision only.

/// Identifier whose lifetime and uniqueness are managed by the executor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskId(
    /// Numeric identity of a task, independent of its position in the queue.
    pub u64,
);

/// Fixed scheduling priority, ordered from most urgent to least urgent.
///
/// The declaration order is intentional. Selection takes the smallest value,
/// making [`Priority::High`] win over [`Priority::Normal`] and [`Priority::Low`].
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Priority {
    /// Selected before every runnable task at another priority.
    High,
    /// Selected after high-priority tasks and before low-priority tasks.
    Normal,
    /// Selected when no runnable task has a higher priority.
    Low,
}

/// Reason a registration was rejected without changing the queue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegisterError {
    /// The identifier is already registered, including at another priority.
    Duplicate,
    /// The global capacity is exhausted.
    Full,
}

// A ready entry deliberately contains no stack, registers or task callback.
// Keeping execution out of this type makes policy tests independent of a CPU.
#[derive(Clone, Copy)]
struct ReadyTask {
    id: TaskId,
    priority: Priority,
}

/// Allocation-free ready queue with a global capacity of `N` tasks.
///
/// Tasks retain arrival order within their priority. Registration and a
/// voluntary yield append at the back, while selection is read-only. The
/// capacity covers all priorities together, allowing a task to yield even
/// when every slot is occupied.
///
/// Access requires exclusive borrowing for mutations. The integration layer
/// must still serialize callers if this value is shared with IRQs or CPUs.
pub struct PriorityScheduler<const N: usize> {
    // Dense-prefix invariant: 0..len contains Some(entry), len..N is None.
    // The prefix records arrival order across all priorities. Selecting the
    // first entry at the best priority therefore preserves FIFO ties without
    // a sequence counter, dynamically allocated queues or counter wraparound.
    ready: [Option<ReadyTask>; N],
    len: usize,
}

impl<const N: usize> Default for PriorityScheduler<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> PriorityScheduler<N> {
    /// Create an empty queue, including a valid queue with zero capacity.
    pub const fn new() -> Self {
        Self {
            ready: [None; N],
            len: 0,
        }
    }

    /// Append a runnable task at the back of its priority.
    ///
    /// A duplicate is reported before capacity exhaustion, even in a full
    /// queue. Both errors preserve all entries and their relative order.
    pub fn register(&mut self, id: TaskId, priority: Priority) -> Result<(), RegisterError> {
        // Rejecting duplicates across the entire prefix keeps one identity
        // from occupying several positions or being selected at two levels.
        if self.contains(id) {
            return Err(RegisterError::Duplicate);
        }
        // Check before indexing. This also handles N = 0 without a special
        // branch or an out-of-bounds access to the empty array.
        if self.len == N {
            return Err(RegisterError::Full);
        }
        self.ready[self.len] = Some(ReadyTask { id, priority });
        self.len += 1;
        Ok(())
    }

    /// Select the oldest task at the highest runnable priority.
    ///
    /// Return `None` when the queue is empty. Repeated calls preserve the same
    /// answer until registration, removal or a yield changes the queue.
    pub fn select_next(&self) -> Option<TaskId> {
        // min_by_key keeps the first entry when priorities tie. Scanning the
        // dense prefix thus chooses a priority and its FIFO head together.
        // No cursor advances here and no task or CPU context is executed.
        self.ready[..self.len]
            .iter()
            .flatten()
            .min_by_key(|task| task.priority)
            .map(|task| task.id)
    }

    /// Remove a blocked or finished task.
    ///
    /// Return `true` when an entry was removed, or `false` when the identifier
    /// was absent. Removing a task preserves every remaining task's order.
    pub fn remove(&mut self, id: TaskId) -> bool {
        let Some(index) = self.index_of(id) else {
            return false;
        };
        // Close the hole instead of reusing it for a future registration.
        // Reusing a hole would let a newly woken task jump ahead of older
        // peers. copy_within supports overlapping ranges, including an empty
        // suffix when the removed entry was already last.
        self.ready.copy_within(index + 1..self.len, index);
        self.len -= 1;
        self.ready[self.len] = None;
        true
    }

    /// Move a registered task to the back of its current priority.
    ///
    /// Return `true` when the task exists, even if it is already last or has
    /// no peer. Return `false` for an absent identifier. The executor calls
    /// this method when that task voluntarily yields. Its priority and the
    /// runnable-task count stay the same.
    pub fn yield_now(&mut self, id: TaskId) -> bool {
        let Some(index) = self.index_of(id) else {
            return false;
        };
        let task = self.ready[index];
        // Move the entry to the global tail. Selection compares priorities
        // first, so this affects FIFO position only among tasks at its level.
        // Moving in place needs no spare slot, including when len == N.
        self.ready.copy_within(index + 1..self.len, index);
        self.ready[self.len - 1] = task;
        true
    }

    /// Return the number of registered runnable tasks.
    pub fn runnable_count(&self) -> usize {
        self.len
    }

    /// Return whether an identifier is currently registered as runnable.
    pub fn contains(&self, id: TaskId) -> bool {
        self.index_of(id).is_some()
    }

    fn index_of(&self, id: TaskId) -> Option<usize> {
        self.ready[..self.len]
            .iter()
            .position(|slot| slot.is_some_and(|task| task.id == id))
    }
}

#[cfg(test)]
mod tests;
