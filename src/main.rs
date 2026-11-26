//! Bare-metal entry point for the integrated FerrumOS image.

#![no_std]
#![no_main]

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!(
    "Build the kernel with --target x86_64-unknown-none; use scripts/boot.py to run it."
);

use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};

static BOOT_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.kernel_stack_size = 128 * 1024;
    // The paging adapter translates physical table-frame addresses through
    // this bootloader-owned direct map. It may only allocate frames marked
    // Usable and must keep exclusive mutable ownership of the active mapper.
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_entry, config = &BOOT_CONFIG);

fn kernel_entry(boot_info: &'static mut BootInfo) -> ! {
    ferrumos::boot::run(boot_info)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    ferrumos::arch::x86_64::cpu::disable_interrupts();
    // SAFETY: the single boot CPU abandons all normal execution with interrupts
    // disabled. The terminal writer reinitializes COM1 using bounded polling,
    // including when an early failure precedes normal UART initialization.
    unsafe {
        ferrumos::drivers::serial::emergency_write(format_args!("\nFERRUMOS_PANIC: {info}\n"))
    };
    ferrumos::arch::x86_64::cpu::halt_forever()
}
