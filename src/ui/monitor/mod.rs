//! Task and resource monitoring views.
//!
//! [`render_stats`] turns a [`KernelSnapshot`] into the text shown by the
//! `stats` command. The view reads a snapshot that was assembled elsewhere, so
//! it runs outside the synchronization protecting the measurements and never
//! inspects hardware registers or allocator internals.
//!
//! A measurement the kernel cannot take yet is rendered as [`UNAVAILABLE`] and
//! never as zero, so a reader can tell an idle counter from a missing source.
//!
//! [`StatsView`] serves that report to the shell through its
//! [`StatsProvider`] interface, so the shell holds no measurement source and
//! no timer configuration of its own.

#![forbid(unsafe_code)]

use core::fmt::{self, Write};

use crate::kernel::interrupts::TickSnapshot;
use crate::kernel::telemetry::{KernelReporter, KernelSnapshot, Uptime};
use crate::ui::shell::StatsProvider;

/// Text shown in place of a measurement the kernel cannot take yet.
pub const UNAVAILABLE: &str = "unavailable";

/// Width reserved for the label column, in characters.
///
/// Two characters wider than the longest label, so a value never touches it.
const LABEL_WIDTH: usize = 18;

/// Parts per thousand in one percent.
const PERMILLE_PER_PERCENT: u16 = 10;

/// Runs the `stats` command: reads the kernel once, then writes the report.
///
/// This is the entry point the shell calls. It takes one snapshot, converts the
/// tick count it contains into an elapsed time, and renders the result. Reading
/// the kernel once keeps every line of the report describing the same instant,
/// and converting the ticks carried by that snapshot keeps the uptime
/// consistent with the tick count printed next to it.
///
/// `timer_input_hz` and `timer_divisor` describe the timer that delivers those
/// ticks; the architecture layer exposes the configured pair. A zero in either
/// one means the timer is not configured yet, and the uptime is then reported
/// as unavailable rather than as zero.
///
/// # Errors
/// Returns the error reported by `out`, as [`render_stats`] does.
pub fn stats_command<W: Write + ?Sized>(
    reporter: &dyn KernelReporter,
    timer_input_hz: u32,
    timer_divisor: u16,
    out: &mut W,
) -> fmt::Result {
    let snapshot = reporter.kernel_snapshot();
    let uptime = TickSnapshot::new(
        snapshot.counters.timer_ticks(),
        timer_input_hz,
        timer_divisor,
    )
    .map(|clock| Uptime::from_milliseconds(clock.uptime_millis()));

    render_stats(&snapshot, uptime, out)
}

/// Writes the `stats` report of `snapshot` into `out`.
///
/// `uptime` is [`None`] while the timer configuration is unknown, which renders
/// the elapsed time as unavailable instead of as zero. Every line holds a label
/// and a value, and the report ends with a newline.
///
/// # Errors
/// Returns the error reported by `out`, which happens when the destination
/// cannot accept the whole report. The report is then incomplete, and the
/// caller decides whether to retry or to drop it.
pub fn render_stats<W: Write + ?Sized>(
    snapshot: &KernelSnapshot,
    uptime: Option<Uptime>,
    out: &mut W,
) -> fmt::Result {
    write_uptime(uptime, out)?;
    write_heap(snapshot, out)?;
    write_runnable_tasks(snapshot, out)?;
    write_counters(snapshot, out)
}

fn write_uptime<W: Write + ?Sized>(uptime: Option<Uptime>, out: &mut W) -> fmt::Result {
    write_label(out, "uptime")?;
    match uptime {
        Some(uptime) => {
            let parts = uptime.parts();
            writeln!(
                out,
                "{:02}:{:02}:{:02}.{:03}",
                parts.hours, parts.minutes, parts.seconds, parts.milliseconds
            )
        }
        None => writeln!(out, "{UNAVAILABLE}"),
    }
}

fn write_heap<W: Write + ?Sized>(snapshot: &KernelSnapshot, out: &mut W) -> fmt::Result {
    let Some(heap) = snapshot.memory else {
        write_label(out, "heap")?;
        writeln!(out, "{UNAVAILABLE}")?;
        write_label(out, "heap free")?;
        return writeln!(out, "{UNAVAILABLE}");
    };

    write_label(out, "heap")?;
    write!(
        out,
        "{} B used of {} B",
        heap.heap_used_bytes(),
        heap.heap_capacity_bytes()
    )?;
    match heap.usage_permille() {
        Some(permille) => writeln!(
            out,
            " ({}.{} %)",
            permille / PERMILLE_PER_PERCENT,
            permille % PERMILLE_PER_PERCENT
        )?,
        None => writeln!(out)?,
    }

    write_label(out, "heap free")?;
    writeln!(out, "{} B", heap.heap_free_bytes())
}

fn write_runnable_tasks<W: Write + ?Sized>(snapshot: &KernelSnapshot, out: &mut W) -> fmt::Result {
    write_label(out, "runnable tasks")?;
    writeln!(out, "{}", snapshot.runnable_tasks)
}

fn write_counters<W: Write + ?Sized>(snapshot: &KernelSnapshot, out: &mut W) -> fmt::Result {
    write_label(out, "timer ticks")?;
    writeln!(out, "{}", snapshot.counters.timer_ticks())?;

    write_label(out, "context switches")?;
    writeln!(out, "{}", snapshot.counters.context_switches())?;

    write_label(out, "dispatches")?;
    writeln!(out, "{}", snapshot.counters.dispatches())
}

fn write_label<W: Write + ?Sized>(out: &mut W, label: &str) -> fmt::Result {
    write!(out, "{label:<width$}", width = LABEL_WIDTH)
}

/// Serves the shell's `stats` command from one kernel reporter.
///
/// The shell owns no measurement source: it holds a [`StatsProvider`] and
/// calls it outside any critical section. This view carries the reporter to
/// read and the timer configuration that turns its tick count into an elapsed
/// time, so neither the shell nor the report depends on the architecture
/// exposing those values.
pub struct StatsView<'a> {
    reporter: &'a dyn KernelReporter,
    timer_input_hz: u32,
    timer_divisor: u16,
}

impl<'a> StatsView<'a> {
    /// Builds the view reading `reporter` through the configured timer.
    ///
    /// `timer_input_hz` and `timer_divisor` describe the timer delivering the
    /// ticks the reporter counts, as [`stats_command`] documents. A zero in
    /// either one reports the elapsed time as unavailable.
    pub const fn new(
        reporter: &'a dyn KernelReporter,
        timer_input_hz: u32,
        timer_divisor: u16,
    ) -> Self {
        Self {
            reporter,
            timer_input_hz,
            timer_divisor,
        }
    }
}

impl StatsProvider for StatsView<'_> {
    fn write_stats(&self, out: &mut dyn Write) -> fmt::Result {
        stats_command(self.reporter, self.timer_input_hz, self.timer_divisor, out)
    }
}

#[cfg(test)]
mod tests;
