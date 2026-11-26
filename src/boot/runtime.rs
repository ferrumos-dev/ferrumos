use core::alloc::Layout;
use core::fmt::{self, Write};

use bootloader_api::BootInfo;

use crate::arch::x86_64::cpu;
use crate::arch::x86_64::memory::BootMemorySummary;
use crate::arch::x86_64::{interrupts, memory};
use crate::drivers::console::InputLine;
use crate::drivers::display::TextConsole;
use crate::drivers::input::{InputEvent, KeyboardLayout, Ps2Keyboard};
use crate::drivers::serial::SerialPort;
use crate::kernel::executor::{CooperativeExecutor, TaskStatus};
use crate::kernel::memory::MemoryReporter;
use crate::kernel::scheduler::{Priority, TaskId};
use crate::kernel::telemetry::CapturedKernel;
use crate::ui::monitor::StatsView;
use crate::ui::shell::StatsProvider;

// Task identifiers are local to this executor. Keeping device ownership in
// BootState lets the task borrow drivers without global mutable references.
const INPUT_TASK: TaskId = TaskId(1);
const DIAGNOSTICS_TASK: TaskId = TaskId(2);

struct Console<'a> {
    display: TextConsole<'a>,
    serial: SerialPort,
    serial_enabled: bool,
}

impl Write for Console<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.display.write_str(text)?;
        // A failed diagnostic port must not truncate formatted screen output.
        if self.serial_enabled && self.serial.write_str(text).is_err() {
            self.serial_enabled = false;
        }
        Ok(())
    }
}

impl Console<'_> {
    fn erase_character(&mut self) {
        self.display.write_char('\u{8}');
        if self.serial_enabled && self.serial.write_str("\u{8} \u{8}").is_err() {
            self.serial_enabled = false;
        }
    }
}

struct BootState<'a> {
    console: Console<'a>,
    keyboard: Ps2Keyboard,
    line: InputLine<128>,
    input_error_reported: bool,
    input_started: bool,
}

/// Poll one input byte, then give control back to the executor.
///
/// Blocking here means "not ready in the software queue". The boot loop wakes
/// this task before the next polling round. Only the timer IRQ is unmasked;
/// the PS/2 controller remains exclusively owned by the polling driver.
/// Consuming only one byte per call keeps this cooperative step bounded.
fn input_step(state: &mut BootState<'_>) -> TaskStatus {
    if !state.input_started {
        state.input_started = true;
        let _ = state.console.serial.write_str("INPUT_TASK_READY\n");
        // Initialization is a separate bounded step. It leaves any bytes
        // already queued by the keyboard untouched until diagnostics finishes.
        return TaskStatus::Blocked;
    }
    match state.keyboard.poll_event() {
        Ok(Some(InputEvent::Character(character))) => {
            if state.line.push(character).is_ok() {
                let _ = state.console.write_char(character);
            }
        }
        Ok(Some(InputEvent::Backspace)) => {
            if state.line.pop().is_some() {
                state.console.erase_character();
            }
        }
        Ok(Some(InputEvent::Enter)) => {
            let _ = state.console.write_str("\n");
            // The acknowledgement remains stable for the normal BIOS input
            // smoke test. This line is produced by a dispatched task function.
            let _ = writeln!(state.console.serial, "INPUT_LINE: {}", state.line.as_str());
            state.line.clear();
            let _ = state.console.write_str("> ");
        }
        Ok(Some(InputEvent::Escape)) => {
            while state.line.pop().is_some() {
                state.console.erase_character();
            }
        }
        Ok(None) => {}
        Err(error) => {
            if !state.input_error_reported {
                let _ = writeln!(state.console.serial, "INPUT_WARNING: {error:?}");
                state.input_error_reported = true;
            }
        }
    }
    TaskStatus::Blocked
}

/// Run once after the high-priority input task has returned its first step.
fn diagnostics_step(state: &mut BootState<'_>) -> TaskStatus {
    // Diagnostics is registered first at a lower priority. Reaching this check
    // with input_started set proves that selection changed real call order.
    assert!(
        state.input_started,
        "input must precede lower-priority diagnostics"
    );
    let _ = state.console.serial.write_str("KERNEL_PRIORITY_OK\n");
    let _ = state.console.write_str("> ");
    TaskStatus::Complete
}

/// Report the kernel statistics once, over the diagnostic port.
///
/// The capture reads the heap, the ready set, the execution counters and the
/// hardware clock while this function owns them, and the view formats that
/// single sample afterwards. The tick total and the frequency used to express
/// it come from the same clock reading, so the elapsed time matches the tick
/// count printed next to it.
///
/// The two markers delimit the report in the serial log, which lets the boot
/// test check that the measurements describe the real heap and a real tick
/// count instead of placeholder values.
fn report_stats(
    heap: &memory::KernelHeap,
    executor: &CooperativeExecutor<BootState<'_>, 2>,
    serial: &mut SerialPort,
) {
    let clock = interrupts::clock_snapshot();
    let counters = executor.counters().with_timer_ticks(clock.ticks());
    let kernel = CapturedKernel::capture(heap, executor.runnable_count(), counters);
    let view = StatsView::new(
        &kernel,
        clock.frequency_numerator_hz(),
        clock.frequency_divisor(),
    );
    let _ = serial.write_str(
        "STATS_BEGIN
",
    );
    let _ = view.write_stats(serial);
    let _ = serial.write_str(
        "STATS_END
",
    );
}

/// Exercise the real mapped heap before handing it to other kernel consumers.
///
/// Blocks own their exact layouts and free on drop. Holding two live blocks
/// checks that writes do not overlap. Reallocating after a drop checks reuse,
/// while the final snapshot verifies that all reserved bytes were recovered.
fn check_heap(heap: &memory::KernelHeap) -> Result<(), &'static str> {
    let initial = heap
        .memory_snapshot()
        .ok_or("heap has no initial snapshot")?;
    let layout = Layout::from_size_align(96, 64).expect("fixed nonzero heap test layout");
    let mut first = heap.allocate(layout).map_err(|_| "first heap allocation")?;
    let first_address = first.address();
    if !first_address.is_multiple_of(64) {
        return Err("heap allocation alignment");
    }
    first.as_mut_slice().fill(0xa5);
    let mut second = heap
        .allocate(Layout::new::<[u8; 128]>())
        .map_err(|_| "second heap allocation")?;
    second.as_mut_slice().fill(0x5a);
    if first.as_slice().iter().any(|&byte| byte != 0xa5) {
        return Err("live heap allocations overlap");
    }
    let used = heap
        .memory_snapshot()
        .ok_or("heap snapshot while allocated")?;
    if used.heap_used_bytes() <= initial.heap_used_bytes() {
        return Err("heap usage did not increase");
    }
    drop(first);
    let replacement = heap.allocate(layout).map_err(|_| "heap reuse allocation")?;
    if replacement.address() != first_address || second.as_slice().iter().any(|&byte| byte != 0x5a)
    {
        return Err("freed heap range was not reusable or damaged its neighbour");
    }
    drop(replacement);
    drop(second);
    // An oversized request must report exhaustion without consuming the free
    // list. This follows a different path from an allocation that succeeds.
    let too_large =
        Layout::from_size_align(memory::HEAP_SIZE * 2, 16).expect("bounded OOM test layout");
    if !matches!(
        heap.allocate(too_large),
        Err(memory::HeapError::OutOfMemory)
    ) {
        return Err("heap exhaustion contract");
    }
    if heap.memory_snapshot() != Some(initial) {
        return Err("heap bytes were not recovered after drop or failed allocation");
    }
    Ok(())
}

/// Initialize the console and execute cooperative tasks on the boot CPU.
///
/// Called once from the bootloader entry point with exclusive boot information.
/// The framebuffer, COM1 and PS/2 controller stay owned by this function. The
/// timer interrupt only records a tick and acknowledges its controller. Heap
/// access masks interrupts around its lock and provides copied observations.
pub fn run(boot_info: &'static mut BootInfo) -> ! {
    cpu::disable_interrupts();
    // SAFETY: this is the first entry on the only active CPU. Interrupts are
    // disabled and no other driver owns COM1 for the lifetime of the boot loop.
    let mut serial = match unsafe { SerialPort::initialize() } {
        Ok(serial) => serial,
        Err(_) => cpu::halt_forever(),
    };
    let _ = serial.write_str("FERRUMOS_BOOT_OK\n");
    let memory = match BootMemorySummary::from_regions(&boot_info.memory_regions) {
        Ok(memory) => memory,
        Err(error) => {
            let _ = writeln!(serial, "BOOT_FAILED: memory map {error:?}");
            cpu::halt_forever();
        }
    };
    let _ = serial.write_str("MEMORY_MAP_READY\n");
    let _ = writeln!(
        serial,
        "MEMORY_USABLE bytes={} frames={}",
        memory.usable_bytes(),
        memory.usable_frames()
    );
    let Some(physical_offset) = boot_info.physical_memory_offset.into_option() else {
        let _ = serial.write_str("BOOT_FAILED: physical memory mapping unavailable\n");
        cpu::halt_forever();
    };
    // SAFETY: this is the unique boot CPU, with IF clear. The bootloader
    // created the direct physical map requested in main.rs. No other mapper
    // or frame allocator exists, and BootInfo retains the memory descriptors.
    let heap = match unsafe { memory::initialize(physical_offset, &boot_info.memory_regions) } {
        Ok(heap) => heap,
        Err(error) => {
            let _ = writeln!(serial, "BOOT_FAILED: heap {error:?}");
            cpu::halt_forever();
        }
    };
    let _ = serial.write_str("HEAP_READY\n");
    if let Err(error) = check_heap(heap) {
        let _ = writeln!(serial, "BOOT_FAILED: {error}");
        cpu::halt_forever();
    }
    let _ = serial.write_str("HEAP_ALLOC_FREE_OK\n");
    if let Some(snapshot) = heap.memory_snapshot() {
        let _ = writeln!(
            serial,
            "HEAP_SAMPLE capacity={} used={} free={}",
            snapshot.heap_capacity_bytes(),
            snapshot.heap_used_bytes(),
            snapshot.heap_free_bytes()
        );
    }

    let Some(framebuffer) = boot_info.framebuffer.as_mut() else {
        let _ = serial.write_str("BOOT_FAILED: framebuffer unavailable\n");
        cpu::halt_forever();
    };
    let info = framebuffer.info();
    let display = match TextConsole::new(framebuffer.buffer_mut(), info) {
        Ok(display) => display,
        Err(error) => {
            let _ = writeln!(serial, "BOOT_FAILED: framebuffer {error:?}");
            cpu::halt_forever();
        }
    };
    let _ = serial.write_str("DISPLAY_READY\n");
    let _ = writeln!(serial, "FRAMEBUFFER {}x{}", info.width, info.height);
    let mut console = Console {
        display,
        serial,
        serial_enabled: true,
    };
    let _ = writeln!(console, "FerrumOS {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(
        console,
        "Console clavier AZERTY - Entree valide, Echap efface."
    );
    let _ = writeln!(console, "Demarrage, affichage et saisie disponibles.\n");

    // SAFETY: the boot CPU exclusively owns i8042. IF is still clear during
    // initialization. The polling driver disables keyboard/mouse IRQs, and
    // the PIC setup below unmasks only the separate timer line.
    let keyboard = match unsafe { Ps2Keyboard::initialize(KeyboardLayout::Azerty) } {
        Ok(keyboard) => keyboard,
        Err(error) => {
            let _ = writeln!(console, "BOOT_FAILED: keyboard {error:?}");
            cpu::halt_forever();
        }
    };
    let _ = console.serial.write_str("KEYBOARD_READY\n");
    let mut state = BootState {
        console,
        keyboard,
        line: InputLine::new(),
        input_error_reported: false,
        input_started: false,
    };
    let mut executor = CooperativeExecutor::<BootState<'_>, 2>::new();
    executor
        .register(DIAGNOSTICS_TASK, Priority::Normal, diagnostics_step)
        .expect("diagnostics fits the empty boot executor");
    executor
        .register(INPUT_TASK, Priority::High, input_step)
        .expect("input has a distinct identifier and the second reserved slot");
    // SAFETY: drivers are initialized, IF remains clear, and this is the only
    // caller installing the static IDT and owning the legacy PIC/PIT ports.
    if let Err(error) = unsafe { interrupts::initialize_timer() } {
        let _ = writeln!(state.console.serial, "BOOT_FAILED: timer {error:?}");
        cpu::halt_forever();
    }
    let _ = state.console.serial.write_str("TIMER_READY\n");
    cpu::enable_interrupts();
    // Repeat the allocator exercise with IF set. Its critical sections must
    // restore the caller's flag, otherwise the periodic timer would stop here.
    if let Err(error) = check_heap(heap) {
        let _ = writeln!(
            state.console.serial,
            "BOOT_FAILED: IRQ-enabled heap: {error}"
        );
        cpu::halt_forever();
    }
    if !cpu::interrupts_enabled() {
        let _ = state
            .console
            .serial
            .write_str("BOOT_FAILED: heap left interrupts disabled\n");
        cpu::halt_forever();
    }
    let _ = state.console.serial.write_str("HEAP_IRQ_SAFE_OK\n");
    let mut last_sample = 0;
    let mut timer_samples = 0;
    let mut stats_reported = false;
    loop {
        executor
            .wake(INPUT_TASK)
            .expect("the input task stays registered");
        // Bound each polling round even if another cooperative task is later
        // added and yields repeatedly. Wake sources run again on the next round.
        for _ in 0..executor.task_count() {
            executor.run_one(&mut state);
        }
        // Printing is kept in normal execution, outside the IRQ handler. Two
        // spaced samples let the QEMU test prove repeated delivery and EOI.
        let clock = interrupts::clock_snapshot();
        if timer_samples < 2 && clock.ticks().saturating_sub(last_sample) >= 5 {
            // Exercise the same copy a reporter supplies to stats. Both fields
            // come from this clock reading even if a new IRQ arrives now.
            let counters = executor.counters().with_timer_ticks(clock.ticks());
            let _ = writeln!(
                state.console.serial,
                "\nTIMER_SAMPLE ticks={} uptime_ms={}",
                counters.timer_ticks(),
                clock.uptime_millis()
            );
            last_sample = clock.ticks();
            timer_samples += 1;
        }
        if timer_samples == 2 && !stats_reported {
            stats_reported = true;
            report_stats(heap, &executor, &mut state.console.serial);
        }
        // STI and HLT are adjacent in the CPU helper. A pending timer wakes
        // the CPU instead of being lost between enabling interrupts and sleep.
        cpu::enable_and_halt();
    }
}
