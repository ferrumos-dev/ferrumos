use super::*;

use crate::kernel::memory::MemorySnapshot;
use crate::kernel::telemetry::Counters;

const BUFFER_LEN: usize = 512;
const HEAP_CAPACITY: u64 = 100 * 1024;

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
    let mut counters = Counters::new();
    counters.on_timer_tick();
    counters.record_context_switch();

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
        counters: Counters::new(),
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
        counters: Counters::new(),
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
    assert_eq!(report.lines().count(), 6, "{report}");
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
