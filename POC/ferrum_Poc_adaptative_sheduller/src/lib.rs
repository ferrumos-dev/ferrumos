#![no_std]
#![cfg_attr(test, no_main)]
#![feature(abi_x86_interrupt)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::BootInfo;
use core::panic::PanicInfo;
use x86_64::VirtAddr;

pub mod allocator;
pub mod experiments;
pub mod interrupts;
pub mod memory;
pub mod scheduler;
pub mod serial;
pub mod shell;
pub mod testing;
pub mod timing;
pub mod ui;
pub mod vga_buffer;

pub fn init(boot_info: &'static BootInfo) {
    x86_64::instructions::interrupts::disable();
    serial::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    // SAFETY: the bootloader maps all physical memory at
    // `physical_memory_offset` (feature `map_physical_memory`), and this is
    // the only mapper: it is moved into `memory::install` below.
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    // SAFETY: the memory map comes from the bootloader, and this is the only
    // frame allocator, also moved into `memory::install`.
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    serial_println!("heap allocator: {}", allocator::HEAP_NAME);

    memory::install(mapper, frame_allocator);

    interrupts::init_idt();
    // SAFETY: interrupts are disabled (above) and the IDT is loaded, so the
    // PIC can be remapped to vectors 32..48 that have handlers. All lines
    // are masked until a test or the shell unmasks them.
    unsafe {
        let mut pics = interrupts::PICS.lock();
        pics.initialize();
        pics.write_masks(0b1111_1111, 0b1111_1111);
    }
    x86_64::instructions::interrupts::enable();
}

pub fn test_runner(tests: &[&dyn testing::Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    testing::exit_qemu(testing::QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("{}", info);
    testing::exit_qemu(testing::QemuExitCode::Failed);
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
bootloader::entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use x86_64::VirtAddr;

    #[test_case]
    fn trivial_assertion() {
        assert_eq!(1 + 1, 2);
    }

    #[test_case]
    fn heap_alloc_roundtrip() {
        let value = Box::new(0x_dead_beef_u64);
        assert_eq!(*value, 0x_dead_beef);
    }

    #[test_case]
    fn heap_stats_track_allocations() {
        let before = crate::allocator::stats();
        let value = Box::new([0u8; 512]);
        let during = crate::allocator::stats();
        core::hint::black_box(&value);

        assert!(during.used >= before.used + 512);
        assert_eq!(during.free, during.heap_size - during.used);
        assert_eq!(during.total_allocations, before.total_allocations + 1);
    }

    #[test_case]
    fn try_alloc_reports_exhaustion() {
        use core::alloc::Layout;
        let layout = Layout::from_size_align(crate::allocator::HEAP_SIZE * 2, 8).unwrap();
        assert_eq!(
            crate::allocator::try_alloc(layout),
            Err(crate::allocator::AllocError)
        );
        assert!(crate::allocator::stats().failed_allocations >= 1);
    }

    #[test_case]
    fn frame_stats_are_consistent() {
        let stats = crate::memory::frame_stats();
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
            crate::memory::with_memory(|mapper, frame_allocator| {
                let frame = frame_allocator
                    .allocate_frame()
                    .expect("no frame available");
                let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
                // SAFETY: `page` is in an address range used by nothing else
                // and `frame` was just allocated, so the mapping aliases no
                // existing memory.
                unsafe {
                    mapper
                        .map_to(page, frame, flags, frame_allocator)
                        .expect("mapping failed")
                        .flush();
                }

                let ptr: *mut u64 = page.start_address().as_mut_ptr();
                // SAFETY: the page was mapped writable just above and is
                // page-aligned, so `ptr` is valid and aligned for a `u64`.
                unsafe {
                    ptr.write_volatile(0x_cafe_babe);
                    assert_eq!(ptr.read_volatile(), 0x_cafe_babe);
                }

                let (frame, flush) = mapper.unmap(page).expect("unmapping failed");
                flush.flush();
                frame_allocator.deallocate_frame(frame);
            });
        };

        let before_first = crate::memory::frame_stats().free_frames;
        cycle();
        let page_table_frames = before_first - crate::memory::frame_stats().free_frames;
        crate::serial_println!("	page table frames kept after unmap: {}", page_table_frames);

        let before_second = crate::memory::frame_stats().free_frames;
        cycle();
        assert_eq!(crate::memory::frame_stats().free_frames, before_second);
    }

    #[test_case]
    fn cycle_counter_advances_and_measures() {
        let watch = crate::timing::Stopwatch::start();
        let mut sink = 0u64;
        for i in 0..10_000u64 {
            sink = sink.wrapping_add(i);
        }
        core::hint::black_box(sink);
        let elapsed = watch.elapsed();

        assert!(elapsed > 0);
        crate::serial_print!("{} cycles for 10 000 additions	", elapsed);
    }

    #[test_case]
    fn record_format_is_printable() {
        use crate::testing::Record;

        let before = crate::allocator::stats().used;
        let (value, cycles) = crate::timing::measure(|| Box::new([7u8; 64]));
        let after = crate::allocator::stats().used;
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

        crate::interrupts::unmask_timer();
        let start = crate::interrupts::ticks();
        while crate::interrupts::ticks() == start {
            core::hint::spin_loop();
        }
        let elapsed = crate::interrupts::ticks() - start;
        crate::interrupts::mask_all();

        assert!(elapsed > 0);
    }
}
