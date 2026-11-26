//! Kernel reporter holding one observation of the live measurement sources.

use crate::kernel::memory::MemoryReporter;

use super::{Counters, KernelReporter, KernelSnapshot};

/// One coherent observation of the kernel, taken from its real sources.
///
/// [`CapturedKernel::capture`] reads the sources once; the value then answers
/// every later read with that same sample. Holding the sample rather than the
/// sources keeps the view out of the synchronization protecting them, and keeps
/// the lines of a report describing a single instant even though the live state
/// keeps changing.
///
/// This reporter replaces [`super::UnwiredKernel`] wherever the heap, the ready
/// set and the counters exist. An unmeasurable heap is still reported as
/// unavailable, so a missing source stays distinguishable from an empty heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapturedKernel {
    snapshot: KernelSnapshot,
}

impl CapturedKernel {
    /// Captures the heap, the ready set and the counters as one observation.
    ///
    /// The caller owns the capture: it reads these sources while it holds their
    /// synchronization, and the heap reading is taken here under the
    /// allocator's own protection. `runnable_tasks` is the number of tasks
    /// eligible to run at that instant, not the number of registrations.
    ///
    /// `counters` carries the execution counts. On a kernel whose timer
    /// interrupts are counted by hardware, the caller imports that total with
    /// [`Counters::with_timer_ticks`] from the same clock reading it uses to
    /// render the elapsed time.
    pub fn capture(memory: &dyn MemoryReporter, runnable_tasks: usize, counters: Counters) -> Self {
        Self {
            snapshot: KernelSnapshot {
                memory: memory.memory_snapshot(),
                runnable_tasks,
                counters,
            },
        }
    }
}

impl KernelReporter for CapturedKernel {
    fn kernel_snapshot(&self) -> KernelSnapshot {
        self.snapshot
    }
}

#[cfg(test)]
mod tests;
