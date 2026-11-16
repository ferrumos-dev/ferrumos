//! Reserved for task identifiers and scheduling policies.
//!
//! The documented `TaskId` and reference `RoundRobin` policy will live here.
//! Future contracts must specify empty queues, duplicate identifiers, capacity
//! errors and task removal before adding priority policies.
//!
//! No scheduler is implemented yet. Selecting an identifier does not execute
//! a task: task lifetimes and CPU context switches belong to the executor.
