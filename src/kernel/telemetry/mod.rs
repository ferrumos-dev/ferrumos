//! Reserved for portable kernel snapshots and counters.
//!
//! The documented `Counters`, `KernelSnapshot` and `KernelReporter` will live
//! here. Timer ticks and actual context switches are distinct events; counters
//! must saturate and unavailable memory measurements must remain explicit.
//!
//! Snapshot assembly must capture memory, ready-task and counter data
//! coherently under the appropriate synchronization. Consumers in [`crate::ui`]
//! must read snapshots after the critical section rather than inspect hardware
//! or allocator internals. No counters or snapshots are implemented yet.
