//! Reserved for task execution and CPU context ownership.
//!
//! The executor will manage task states and lifetimes, consume scheduling
//! decisions and request architecture-specific context switches. It must record
//! a context switch only after an actual switch, not after a selection or tick.
//!
//! No tasks, execution loop or context switching are implemented yet.
