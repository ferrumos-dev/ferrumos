//! Portable boundary for timer events delivered to kernel policy.
//!
//! The architecture adapter owns timer configuration, interrupt-table setup,
//! masks and controller acknowledgement. It delivers an event only after it
//! has established the handler's ownership and synchronization rules.
//!
//! A tick is an event count. Converting it to time requires the configured
//! timer frequency, and executing a task requires the separate executor.

/// Receives one timer tick without exposing hardware-specific types.
///
/// A sink called directly from an interrupt handler must perform bounded,
/// non-blocking work. Its caller owns serialization, ordering and reentrancy.
/// Calling this trait does not acknowledge the interrupt at its controller.
pub trait TimerTickSink {
    /// Handles one delivered timer event.
    fn on_timer_tick(&mut self);
}

/// One delivered-tick observation and the exact rational timer configuration.
///
/// Convert a single observation rather than loading the live tick counter for
/// each displayed field. Durations are derived from delivered events and the
/// timer's input clock, with fractional milliseconds rounded downward. Long
/// periods with interrupts disabled can lose events at the controller, so this
/// is a monotonic timer-derived uptime rather than a battery-backed wall clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TickSnapshot {
    ticks: u64,
    input_hz: u32,
    divisor: u16,
}

impl TickSnapshot {
    /// Construct a snapshot using a nonzero input frequency and divisor.
    ///
    /// Returns `None` for either zero parameter. The output frequency is the
    /// rational value `input_hz / divisor`, preserving a non-integer rate.
    pub const fn new(ticks: u64, input_hz: u32, divisor: u16) -> Option<Self> {
        if input_hz == 0 || divisor == 0 {
            return None;
        }
        Some(Self {
            ticks,
            input_hz,
            divisor,
        })
    }

    /// Number of delivered periodic timer interrupts at capture time.
    pub const fn ticks(self) -> u64 {
        self.ticks
    }

    /// Timer input frequency in hertz, before division by its programmed count.
    pub const fn frequency_numerator_hz(self) -> u32 {
        self.input_hz
    }

    /// Programmed count dividing the input frequency into periodic events.
    pub const fn frequency_divisor(self) -> u16 {
        self.divisor
    }

    /// Whole elapsed milliseconds represented by this observation.
    ///
    /// The intermediate calculation uses `u128` so a long-running `u64` tick
    /// count cannot overflow during multiplication. Unrepresentable durations
    /// saturate instead of wrapping back toward zero.
    pub fn uptime_millis(self) -> u64 {
        let millis = u128::from(self.ticks) * u128::from(self.divisor) * 1000;
        (millis / u128::from(self.input_hz)).min(u128::from(u64::MAX)) as u64
    }

    /// Whole elapsed seconds represented by the same captured tick count.
    pub fn uptime_seconds(self) -> u64 {
        // Calculate directly from ticks so saturation in milliseconds cannot
        // truncate the representable range of a seconds observation.
        let seconds = u128::from(self.ticks) * u128::from(self.divisor);
        (seconds / u128::from(self.input_hz)).min(u128::from(u64::MAX)) as u64
    }
}

#[cfg(test)]
mod tests;
