#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;

mod desktop;
mod framebuffer;
mod input;
mod mouse;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = None;
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let framebuffer = match boot_info.framebuffer.as_mut() {
        Some(fb) => fb,
        None => loop {
            x86_64::instructions::hlt();
        },
    };

    framebuffer::init(framebuffer);
    let (w, h) = framebuffer::dimensions();
    mouse::init(w, h);
    desktop::run();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    framebuffer::clear(framebuffer::Color::new(120, 8, 18));
    framebuffer::draw_text(24, 24, "Kernel panic", framebuffer::Color::WHITE, 2);
    loop {
        x86_64::instructions::hlt();
    }
}
