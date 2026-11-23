//! Kernel boot integration for the BIOS/QEMU image.
//!
//! The bootloader supplies the stack, memory map and framebuffer. This layer
//! owns the first console, validates the physical map and initializes the heap.
//! After installing the IDT and PIC/PIT, it enables timer IRQ0. Cooperative
//! input polls PS/2 and edits UTF-8 lines between timer wakeups. Heap access
//! preserves the caller's interrupt state; command parsing belongs to the shell.
//!
//! No hardware initialization runs when the library is tested on the host.

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
mod runtime;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub use runtime::run;
