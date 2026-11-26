//! Event counters and copied observations for the shell and monitor.
//!
//! Timer ticks, cooperative task-function invocations and CPU context switches
//! describe separate events. Recording one never fabricates another. Counters
//! saturate at their integer limit, preserving monotonic observations.
//!
//! The provider captures a snapshot under the synchronization of the underlying
//! state. Consumers own the resulting copy and can format it after leaving the
//! critical section. No hardware access or allocator measurement occurs here.
//! On the integrated kernel, the provider copies the hardware clock's total into
//! the sample with [`Counters::with_timer_ticks`].
//!
//! [`Uptime`] holds an already converted duration and splits it into the units
//! a view prints. The conversion from delivered ticks belongs to
//! [`crate::kernel::interrupts::TickSnapshot`], which keeps the exact rational
//! timer configuration, so this module never derives time from a tick count.
//!
//! Signatures follow `contrats_noyau.md`.

use crate::kernel::interrupts::TimerTickSink;
use crate::kernel::memory::MemorySnapshot;

mod captured;

pub use captured::CapturedKernel;

/// Milliseconds in one second.
const MILLISECONDS_PER_SECOND: u64 = 1_000;

/// Seconds in one minute.
const SECONDS_PER_MINUTE: u64 = 60;

/// Minutes in one hour.
const MINUTES_PER_HOUR: u64 = 60;

/// Monotonic counts of independently observed kernel events.
///
/// These values are counts, not elapsed time or CPU utilization. Mutating a
/// counter requires exclusive access; shared-state synchronization belongs to
/// its owner. Once a counter reaches `u64::MAX`, later events keep that value.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counters {
    timer_ticks: u64,
    context_switches: u64,
    dispatches: u64,
}

impl Counters {
    /// Returns a copy with the total from an already captured timer reading.
    ///
    /// The reading replaces the previous tick count and preserves dispatches
    /// and context switches. Use a fresh reading for each kernel snapshot.
    #[must_use]
    pub const fn with_timer_ticks(self, timer_ticks: u64) -> Self {
        Self {
            timer_ticks,
            ..self
        }
    }

    /// Returns the number of timer events represented by this sample.
    pub const fn timer_ticks(&self) -> u64 {
        self.timer_ticks
    }

    /// Returns the number of actual CPU context switches recorded by the owner.
    pub const fn context_switches(&self) -> u64 {
        self.context_switches
    }

    /// Returns the number of cooperative task-function invocations recorded.
    pub const fn dispatches(&self) -> u64 {
        self.dispatches
    }

    /// Records a completed CPU context switch, saturating at `u64::MAX`.
    ///
    /// The context-switch mechanism calls this after an actual switch. A policy
    /// selecting an identifier or an executor invoking a function is separate.
    pub fn record_context_switch(&mut self) {
        self.context_switches = self.context_switches.saturating_add(1);
    }

    /// Records one cooperative task-function invocation, saturating at the limit.
    ///
    /// The dispatcher calls this once when invoking the selected task function.
    /// Selecting an identifier alone does not count as an invocation or a CPU
    /// context switch.
    pub fn record_dispatch(&mut self) {
        self.dispatches = self.dispatches.saturating_add(1);
    }
}

impl TimerTickSink for Counters {
    fn on_timer_tick(&mut self) {
        self.timer_ticks = self.timer_ticks.saturating_add(1);
    }
}

/// Elapsed time since boot, held in whole milliseconds for display.
///
/// This type carries a duration that a provider already converted, and splits
/// it into readable units. It performs no tick arithmetic of its own: a caller
/// holding the timer configuration obtains the milliseconds from
/// [`crate::kernel::interrupts::TickSnapshot::uptime_millis`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uptime {
    milliseconds: u64,
}

impl Uptime {
    /// Builds an uptime from a duration already expressed in milliseconds.
    pub const fn from_milliseconds(milliseconds: u64) -> Self {
        Self { milliseconds }
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

/// Copied kernel state captured by a provider under its synchronization.
///
/// The fields describe the same logical observation. A struct literal cannot
/// enforce that guarantee, so its provider owns coherent capture across the
/// allocator, ready set and counters. Copying this value preserves the sample
/// while the live state continues changing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelSnapshot {
    /// Sampled heap capacity and usage in bytes, or `None` when unavailable.
    pub memory: Option<MemorySnapshot>,
    /// Number of tasks in the provider's ready set at capture time.
    pub runnable_tasks: usize,
    /// Event counts captured with the ready set and memory sample.
    pub counters: Counters,
}

/// Supplies copied observations without exposing mutable kernel internals.
pub trait KernelReporter {
    /// Captures one coherent observation using the provider's synchronization.
    fn kernel_snapshot(&self) -> KernelSnapshot;
}

#[cfg(test)]
mod tests;
