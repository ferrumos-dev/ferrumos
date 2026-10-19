#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use ferrumos::{experiments, serial_println, testing};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    ferrumos::init(boot_info);
    serial_println!("Running experiment 1 on a freshly booted kernel");
    experiments::alloc_free_sequence();
    testing::exit_qemu(testing::QemuExitCode::Success)
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ferrumos::test_panic_handler(info)
}
