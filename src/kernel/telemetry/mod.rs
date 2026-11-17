//! Portable kernel snapshots and counters.
//!
//! [`Counters`] accumulates kernel events, [`KernelSnapshot`] gathers them
//! with the heap view of [`crate::kernel::memory`] and the number of runnable
//! tasks, and [`KernelReporter`] is the source that produces the snapshot.
//! [`Uptime`] turns a tick count into a duration for the views.
//!
//! A timer tick and an actual context switch are two distinct events, so they
//! are counted separately: a kernel that receives ticks without ever switching
//! task must show it. [`Counters::record_context_switch`] is called after a
//! commutation that really happened, never after a mere selection. Counters
//! saturate instead of wrapping, so a long running kernel never reports a
//! smaller count than before.
//!
//! These types hold no hardware state and take no lock. The implementer of
//! [`KernelReporter`] captures memory, runnable-task and counter data
//! coherently inside the synchronization that protects them, then hands the
//! plain value to consumers in [`crate::ui`], which read it after that
//! critical section.
//!
//! Signatures follow `contrats_noyau.md` and the `ferrumos-kernel-core`
//! reference in `foundation/`.

#![forbid(unsafe_code)]

use crate::kernel::memory::MemorySnapshot;

/// Microseconds in one millisecond.
const MICROSECONDS_PER_MILLISECOND: u64 = 1_000;

/// Milliseconds in one second.
const MILLISECONDS_PER_SECOND: u64 = 1_000;

/// Seconds in one minute.
const SECONDS_PER_MINUTE: u64 = 60;

/// Minutes in one hour.
const MINUTES_PER_HOUR: u64 = 60;

/// Saturating counters of kernel events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counters {
    timer_ticks: u64,
    context_switches: u64,
}

impl Counters {
    /// Builds counters that have not recorded any event yet.
    pub const fn new() -> Self {
        Self {
            timer_ticks: 0,
            context_switches: 0,
        }
    }

    /// Records one timer tick.
    ///
    /// Called from the timer path, which must stay bounded and non blocking.
    pub fn on_timer_tick(&mut self) {
        self.timer_ticks = self.timer_ticks.saturating_add(1);
    }

    /// Records one context switch that actually happened.
    ///
    /// Selecting a task is not switching to it. A selection that leaves the
    /// running task in place must not be recorded here.
    pub fn record_context_switch(&mut self) {
        self.context_switches = self.context_switches.saturating_add(1);
    }

    /// Timer ticks received since boot.
    pub const fn timer_ticks(&self) -> u64 {
        self.timer_ticks
    }

    /// Context switches performed since boot.
    pub const fn context_switches(&self) -> u64 {
        self.context_switches
    }
}

/// Time elapsed since boot, in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uptime {
    milliseconds: u64,
}

impl Uptime {
    /// Builds an uptime from a duration already expressed in milliseconds.
    pub const fn from_milliseconds(milliseconds: u64) -> Self {
        Self { milliseconds }
    }

    /// Converts a tick count into an uptime.
    ///
    /// Returns [`None`] when `tick_period_microseconds` is zero, because the
    /// tick period is then unknown and no duration can be derived. That
    /// absence is expected while the timer is not calibrated, and it stays
    /// distinct from a zero duration.
    pub const fn from_ticks(ticks: u64, tick_period_microseconds: u64) -> Option<Self> {
        if tick_period_microseconds == 0 {
            return None;
        }
        let microseconds = ticks.saturating_mul(tick_period_microseconds);
        Some(Self {
            milliseconds: microseconds / MICROSECONDS_PER_MILLISECOND,
        })
    }

    /// Whole milliseconds elapsed since boot.
    pub const fn milliseconds(&self) -> u64 {
        self.milliseconds
    }

    /// Whole seconds elapsed since boot.
    pub const fn seconds(&self) -> u64 {
        self.milliseconds / MILLISECONDS_PER_SECOND
    }

    /// Elapsed time split into readable units.
    pub const fn parts(&self) -> UptimeParts {
        let seconds = self.seconds();
        UptimeParts {
            hours: seconds / (SECONDS_PER_MINUTE * MINUTES_PER_HOUR),
            minutes: (seconds / SECONDS_PER_MINUTE) % MINUTES_PER_HOUR,
            seconds: seconds % SECONDS_PER_MINUTE,
            milliseconds: self.milliseconds % MILLISECONDS_PER_SECOND,
        }
    }
}

/// Elapsed time split into readable units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UptimeParts {
    /// Whole hours elapsed.
    pub hours: u64,
    /// Whole minutes within the current hour.
    pub minutes: u64,
    /// Whole seconds within the current minute.
    pub seconds: u64,
    /// Milliseconds within the current second.
    pub milliseconds: u64,
}

/// Coherent view of the kernel at a single instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelSnapshot {
    /// Heap view, or [`None`] when no coherent measurement is available.
    pub memory: Option<MemorySnapshot>,
    /// Tasks ready to run at the time of the snapshot.
    pub runnable_tasks: usize,
    /// Event counters at the time of the snapshot.
    pub counters: Counters,
}

/// Source of kernel snapshots.
///
/// The implementer owns the synchronization: it serializes the shared accesses
/// and captures memory, runnable tasks and counters in one coherent reading.
pub trait KernelReporter {
    /// Current snapshot of the kernel.
    ///
    /// The snapshot always exists; a measurement that could not be taken is
    /// carried as [`None`] rather than as zero.
    fn kernel_snapshot(&self) -> KernelSnapshot;
}

#[cfg(test)]
mod tests;
