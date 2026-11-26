use super::*;

use crate::kernel::interrupts::TimerTickSink;
use crate::kernel::memory::MemorySnapshot;
use crate::kernel::telemetry::{Counters, KernelReporter, UnwiredKernel};
use crate::ui::shell::{Console, Shell};

const BUFFER_LEN: usize = 1024;
const HEAP_CAPACITY: u64 = 100 * 1024;

/// Input frequency of the legacy timer, as configured by the architecture layer.
const TIMER_INPUT_HZ: u32 = 1_193_182;

/// Divisor programmed on that timer, giving close to one tick every 10 ms.
const TIMER_DIVISOR: u16 = 11_932;

/// Timer configuration of a kernel whose timer is not programmed yet.
const UNCONFIGURED_TIMER: (u32, u16) = (0, 0);

struct MeasuredKernel {
    used_bytes: u64,
    runnable_tasks: usize,
    timer_ticks: u64,
}

impl KernelReporter for MeasuredKernel {
    fn kernel_snapshot(&self) -> KernelSnapshot {
        KernelSnapshot {
            memory: MemorySnapshot::new(HEAP_CAPACITY, self.used_bytes),
            runnable_tasks: self.runnable_tasks,
            counters: Counters::default().with_timer_ticks(self.timer_ticks),
        }
    }
}

struct TextBuffer {
    bytes: [u8; BUFFER_LEN],
    len: usize,
}

impl TextBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; BUFFER_LEN],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).expect("the report is valid utf8")
    }
}

impl Console for TextBuffer {
    fn clear(&mut self) -> fmt::Result {
        self.len = 0;
        Ok(())
    }
}

impl Write for TextBuffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len + text.len();
        if end > BUFFER_LEN {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn measured_snapshot() -> KernelSnapshot {
    let mut counters = Counters::default();
    counters.on_timer_tick();
    counters.record_context_switch();
    counters.record_dispatch();
    counters.record_dispatch();

    KernelSnapshot {
        memory: MemorySnapshot::new(HEAP_CAPACITY, 41_984),
        runnable_tasks: 4,
        counters,
    }
}

fn empty_snapshot() -> KernelSnapshot {
    KernelSnapshot {
        memory: None,
        runnable_tasks: 0,
        counters: Counters::default(),
    }
}

fn render(snapshot: &KernelSnapshot, uptime: Option<Uptime>, out: &mut TextBuffer) {
    render_stats(snapshot, uptime, out).expect("the buffer is large enough");
}

fn value_of<'a>(report: &'a str, label: &str) -> &'a str {
    report
        .lines()
        .find(|line| line.starts_with(label))
        .map(|line| line[LABEL_WIDTH..].trim_end())
        .expect("the report holds that label")
}

#[test]
fn a_measured_snapshot_reports_real_values() {
    let mut out = TextBuffer::new();

    render(
        &measured_snapshot(),
        Some(Uptime::from_milliseconds(3_723_456)),
        &mut out,
    );

    let report = out.as_str();
    assert_eq!(value_of(report, "uptime"), "01:02:03.456");
    assert_eq!(
        value_of(report, "heap "),
        "41984 B used of 102400 B (41.0 %)"
    );
    assert_eq!(value_of(report, "heap free"), "60416 B");
    assert_eq!(value_of(report, "runnable tasks"), "4");
    assert_eq!(value_of(report, "timer ticks"), "1");
    assert_eq!(value_of(report, "context switches"), "1");
    assert_eq!(value_of(report, "dispatches"), "2");
}

#[test]
fn an_empty_snapshot_reports_every_missing_source_as_unavailable() {
    let mut out = TextBuffer::new();

    render(&empty_snapshot(), None, &mut out);

    let report = out.as_str();
    assert_eq!(value_of(report, "uptime"), UNAVAILABLE);
    assert_eq!(value_of(report, "heap "), UNAVAILABLE);
    assert_eq!(value_of(report, "heap free"), UNAVAILABLE);
    assert_eq!(report.matches(UNAVAILABLE).count(), 3, "{report}");
}

#[test]
fn an_idle_counter_is_not_rendered_as_unavailable() {
    let mut out = TextBuffer::new();

    render(&empty_snapshot(), None, &mut out);

    let report = out.as_str();
    assert_eq!(value_of(report, "runnable tasks"), "0");
    assert_eq!(value_of(report, "timer ticks"), "0");
    assert_eq!(value_of(report, "context switches"), "0");
    assert_eq!(value_of(report, "dispatches"), "0");
}

#[test]
fn a_kernel_that_just_booted_shows_a_zero_uptime() {
    let mut out = TextBuffer::new();

    render(
        &empty_snapshot(),
        Some(Uptime::from_milliseconds(0)),
        &mut out,
    );

    assert_eq!(value_of(out.as_str(), "uptime"), "00:00:00.000");
}

#[test]
fn an_empty_heap_is_reported_as_measured() {
    let mut out = TextBuffer::new();
    let snapshot = KernelSnapshot {
        memory: MemorySnapshot::new(0, 0),
        runnable_tasks: 0,
        counters: Counters::default(),
    };

    render(&snapshot, None, &mut out);

    let report = out.as_str();
    assert_eq!(value_of(report, "heap "), "0 B used of 0 B");
    assert_eq!(value_of(report, "heap free"), "0 B");
}

#[test]
fn every_line_is_labelled_and_terminated() {
    let mut out = TextBuffer::new();

    render(
        &measured_snapshot(),
        Some(Uptime::from_milliseconds(0)),
        &mut out,
    );

    let report = out.as_str();
    assert!(report.ends_with('\n'), "{report}");
    for line in report.lines() {
        assert!(line.len() > LABEL_WIDTH, "{line}");
        assert!(!line.starts_with(' '), "{line}");
        assert!(!line[LABEL_WIDTH..].starts_with(' '), "{line}");
    }
    assert_eq!(report.lines().count(), 7, "{report}");
}

#[test]
fn the_command_reports_an_unwired_kernel_without_inventing_values() {
    let mut out = TextBuffer::new();
    let (input_hz, divisor) = UNCONFIGURED_TIMER;

    stats_command(&UnwiredKernel, input_hz, divisor, &mut out).expect("the buffer is large enough");

    let report = out.as_str();
    assert_eq!(value_of(report, "uptime"), UNAVAILABLE);
    assert_eq!(value_of(report, "heap "), UNAVAILABLE);
    assert_eq!(value_of(report, "runnable tasks"), "0");
    assert_eq!(value_of(report, "timer ticks"), "0");
}

#[test]
fn the_command_derives_the_uptime_from_the_ticks_it_read() {
    struct TickingKernel;

    impl KernelReporter for TickingKernel {
        fn kernel_snapshot(&self) -> KernelSnapshot {
            let mut counters = Counters::default();
            for _ in 0..100 {
                counters.on_timer_tick();
            }

            KernelSnapshot {
                memory: MemorySnapshot::new(HEAP_CAPACITY, 2048),
                runnable_tasks: 2,
                counters,
            }
        }
    }

    let mut out = TextBuffer::new();

    stats_command(&TickingKernel, TIMER_INPUT_HZ, TIMER_DIVISOR, &mut out)
        .expect("the buffer is large enough");

    let report = out.as_str();
    assert_eq!(value_of(report, "uptime"), "00:00:01.000");
    assert_eq!(value_of(report, "timer ticks"), "100");
    assert_eq!(value_of(report, "runnable tasks"), "2");
    assert_eq!(value_of(report, "heap free"), "100352 B");
}

#[test]
fn a_configured_timer_with_no_tick_yet_shows_a_zero_uptime() {
    let mut out = TextBuffer::new();

    stats_command(&UnwiredKernel, TIMER_INPUT_HZ, TIMER_DIVISOR, &mut out)
        .expect("the buffer is large enough");

    assert_eq!(value_of(out.as_str(), "uptime"), "00:00:00.000");
}

#[test]
fn a_destination_that_overflows_reports_the_error() {
    struct FullBuffer;

    impl Write for FullBuffer {
        fn write_str(&mut self, _text: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    assert!(render_stats(&measured_snapshot(), None, &mut FullBuffer).is_err());
}

#[test]
fn the_view_serves_the_shell_the_report_of_the_command() {
    let kernel = MeasuredKernel {
        used_bytes: 41_984,
        runnable_tasks: 4,
        timer_ticks: 100,
    };
    let view = StatsView::new(&kernel, TIMER_INPUT_HZ, TIMER_DIVISOR);
    let mut through_view = TextBuffer::new();
    let mut through_command = TextBuffer::new();

    view.write_stats(&mut through_view)
        .expect("the buffer is large enough");
    stats_command(&kernel, TIMER_INPUT_HZ, TIMER_DIVISOR, &mut through_command)
        .expect("the buffer is large enough");

    assert_eq!(through_view.as_str(), through_command.as_str());
    assert_eq!(value_of(through_view.as_str(), "uptime"), "00:00:01.000");
}

#[test]
fn the_shell_stats_command_prints_the_measured_report() {
    let kernel = MeasuredKernel {
        used_bytes: 2048,
        runnable_tasks: 2,
        timer_ticks: 100,
    };
    let view = StatsView::new(&kernel, TIMER_INPUT_HZ, TIMER_DIVISOR);
    let shell = Shell::with_stats(&view);
    let mut out = TextBuffer::new();

    shell
        .execute("stats", &mut out)
        .expect("the buffer is large enough");

    let report = out.as_str();
    assert_eq!(value_of(report, "uptime"), "00:00:01.000");
    assert_eq!(value_of(report, "heap free"), "100352 B");
    assert_eq!(value_of(report, "runnable tasks"), "2");
    assert_eq!(value_of(report, "timer ticks"), "100");
    assert_eq!(report.lines().count(), 7, "{report}");
}

#[test]
fn the_shell_reports_an_unwired_kernel_without_inventing_values() {
    let (input_hz, divisor) = UNCONFIGURED_TIMER;
    let view = StatsView::new(&UnwiredKernel, input_hz, divisor);
    let shell = Shell::with_stats(&view);
    let mut out = TextBuffer::new();

    shell
        .execute("stats", &mut out)
        .expect("the buffer is large enough");

    let report = out.as_str();
    assert_eq!(value_of(report, "uptime"), UNAVAILABLE);
    assert_eq!(value_of(report, "heap "), UNAVAILABLE);
    assert_eq!(value_of(report, "runnable tasks"), "0");
}
