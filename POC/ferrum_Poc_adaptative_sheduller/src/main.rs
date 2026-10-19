#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use ferrumos::{init, println, scheduler, shell, ui};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init(boot_info);

    println!("FerrumOS Kernel Starting...");
    println!("Heap initialized.");
    println!("Scheduler core initialized.");

    scheduler::spawn("desktop", scheduler::TaskType::Interactive, 1200);
    scheduler::spawn("taskmgr", scheduler::TaskType::Interactive, 900);
    scheduler::spawn("render", scheduler::TaskType::CpuBound, 1500);
    scheduler::spawn("disk", scheduler::TaskType::IoBound, 1000);
    scheduler::spawn("input", scheduler::TaskType::Interactive, 700);

    println!("Interrupts initialized.");

    println!("Starting graphic launcher...");
    ui::show_boot_screen();

    println!("Starting Shell...");
    shell::run();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    ferrumos::hlt_loop()
}
