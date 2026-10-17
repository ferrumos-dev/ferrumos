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
pub mod interrupts;
pub mod memory;
pub mod allocator;
pub mod scheduler;
pub mod shell;
pub mod ui;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    x86_64::instructions::interrupts::disable();

    println!("FerrumOS Kernel Starting...");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    
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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}

pub fn test_runner(tests: &[&dyn Fn()]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0xf4);
    unsafe {
        port.write(0x10u32);
    }
}
