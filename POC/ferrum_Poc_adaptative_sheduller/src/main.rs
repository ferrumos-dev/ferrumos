#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr};

pub mod vga_buffer;
pub mod serial;
pub mod testing;
pub mod timing;
#[cfg(test)]
pub mod experiments;
pub mod interrupts;
pub mod memory;
pub mod allocator;
pub mod scheduler;
pub mod shell;
pub mod ui;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    serial::init();

    println!("FerrumOS Kernel Starting...");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    memory::install(mapper, frame_allocator);
    
    println!("Heap initialized.");

    println!("Scheduler core initialized.");

    scheduler::spawn("desktop", scheduler::TaskType::Interactive, 1200);
    scheduler::spawn("taskmgr", scheduler::TaskType::Interactive, 900);
    scheduler::spawn("render", scheduler::TaskType::CpuBound, 1500);
    scheduler::spawn("disk", scheduler::TaskType::IoBound, 1000);
    scheduler::spawn("input", scheduler::TaskType::Interactive, 700);

    interrupts::init_idt();
    unsafe {
        let mut pics = interrupts::PICS.lock();
        pics.initialize();
        pics.write_masks(0b1111_1111, 0b1111_1111);
    }
    x86_64::instructions::interrupts::enable();

    println!("Interrupts initialized.");

    #[cfg(test)]
    test_main();

    println!("Starting graphic launcher...");
    ui::show_boot_screen();

    println!("Starting Shell...");
    shell::run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("{}", info);
    testing::exit_qemu(testing::QemuExitCode::Failed);
}

pub fn test_runner(tests: &[&dyn testing::Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    testing::exit_qemu(testing::QemuExitCode::Success);
}

#[test_case]
fn exp1_alloc_free_sequence() {
    experiments::alloc_free_sequence();
}

#[test_case]
fn exp2_long_lived_vs_churn() {
    experiments::long_lived_vs_churn();
}

#[test_case]
fn exp3_exhaustion_and_recovery() {
    experiments::exhaustion_and_recovery();
}

#[test_case]
fn exp4_page_cycle() {
    experiments::page_cycle();
}

#[test_case]
fn exp5_under_interrupts() {
    experiments::under_interrupts();
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1 + 1, 2);
}

#[test_case]
fn heap_alloc_roundtrip() {
    use alloc::boxed::Box;
    let value = Box::new(0x_dead_beef_u64);
    assert_eq!(*value, 0x_dead_beef);
}

#[test_case]
fn heap_stats_track_allocations() {
    use alloc::boxed::Box;
    let before = allocator::stats();
    let value = Box::new([0u8; 512]);
    let during = allocator::stats();
    core::hint::black_box(&value);

    assert!(during.used >= before.used + 512);
    assert_eq!(during.free, during.heap_size - during.used);
    assert_eq!(during.total_allocations, before.total_allocations + 1);
}

#[test_case]
fn try_alloc_reports_exhaustion() {
    use core::alloc::Layout;
    let layout = Layout::from_size_align(allocator::HEAP_SIZE * 2, 8).unwrap();
    assert_eq!(allocator::try_alloc(layout), Err(allocator::AllocError));
    assert!(allocator::stats().failed_allocations >= 1);
}

#[test_case]
fn frame_stats_are_consistent() {
    let stats = memory::frame_stats();
    assert!(stats.usable_frames > 0);
    assert_eq!(
        stats.free_frames + stats.allocated_frames,
        stats.usable_frames
    );
}

#[test_case]
fn page_map_write_unmap_returns_frames() {
    use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};

    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0x_5555_0000_0000));

    let cycle = || {
        memory::with_memory(|mapper, frame_allocator| {
            let frame = frame_allocator.allocate_frame().expect("no frame available");
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            unsafe {
                mapper
                    .map_to(page, frame, flags, frame_allocator)
                    .expect("mapping failed")
                    .flush();
            }

            let ptr: *mut u64 = page.start_address().as_mut_ptr();
            unsafe {
                ptr.write_volatile(0x_cafe_babe);
                assert_eq!(ptr.read_volatile(), 0x_cafe_babe);
            }

            let (frame, flush) = mapper.unmap(page).expect("unmapping failed");
            flush.flush();
            frame_allocator.deallocate_frame(frame);
        });
    };

    let before_first = memory::frame_stats().free_frames;
    cycle();
    let page_table_frames = before_first - memory::frame_stats().free_frames;
    serial_println!(
        "	page table frames kept after unmap: {}",
        page_table_frames
    );

    let before_second = memory::frame_stats().free_frames;
    cycle();
    assert_eq!(memory::frame_stats().free_frames, before_second);
}

#[test_case]
fn cycle_counter_advances_and_measures() {
    let watch = timing::Stopwatch::start();
    let mut sink = 0u64;
    for i in 0..10_000u64 {
        sink = sink.wrapping_add(i);
    }
    core::hint::black_box(sink);
    let elapsed = watch.elapsed();

    assert!(elapsed > 0);
    serial_print!("{} cycles for 10 000 additions	", elapsed);
}

#[test_case]
fn record_format_is_printable() {
    use testing::Record;
    use alloc::boxed::Box;

    let before = allocator::stats().used;
    let (value, cycles) = timing::measure(|| Box::new([7u8; 64]));
    let after = allocator::stats().used;
    core::hint::black_box(&value);

    Record {
        name: "box_alloc_64B",
        iterations: 1,
        errors: 0,
        used_before: before,
        used_after: after,
        cycles,
    }
    .report();
}

#[test_case]
fn timer_ticks_advance_with_interrupts_enabled() {
    assert!(x86_64::instructions::interrupts::are_enabled());

    interrupts::unmask_timer();
    let start = interrupts::ticks();
    while interrupts::ticks() == start {
        core::hint::spin_loop();
    }
    let elapsed = interrupts::ticks() - start;
    interrupts::mask_all();

    assert!(elapsed > 0);
}
