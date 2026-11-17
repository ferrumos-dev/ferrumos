//! Kernel reporter used while no measurement source is wired yet.

use super::{Counters, KernelReporter, KernelSnapshot};

/// Reports a kernel whose measurement sources are not connected yet.
///
/// The heap is reported as unavailable, and the counters hold the events this
/// kernel has actually recorded, which is none. The two are different claims:
/// an unavailable heap means nobody can measure it, while a counter at zero
/// means the event never happened.
///
/// This reporter lets the `stats` command run and tell the truth before the
/// hardware adapters exist. It is replaced, not extended, once the heap and
/// the timer are connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnwiredKernel;

impl KernelReporter for UnwiredKernel {
    fn kernel_snapshot(&self) -> KernelSnapshot {
        KernelSnapshot {
            memory: None,
            runnable_tasks: 0,
            counters: Counters::new(),
        }
    }
}

#[cfg(test)]
mod tests;
