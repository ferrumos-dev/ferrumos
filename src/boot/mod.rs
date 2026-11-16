//! Kernel boot integration for the BIOS/QEMU image.
//!
//! The bootloader supplies the stack, memory map and framebuffer. This layer
//! owns the first console and polls the keyboard with maskable interrupts
//! disabled. Paging/heap management, timer IRQs and a command shell remain
//! separate integration tasks; the input loop only echoes editable lines.
//!
//! No hardware initialization runs when the library is tested on the host.

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
mod runtime;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub use runtime::run;
