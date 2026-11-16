//! Reserved for x86_64 CPU state and context-switch mechanisms.
//!
//! Descriptor tables, task stacks and register save/restore operations belong
//! here. Task selection remains in the portable scheduler; task state and
//! context ownership remain in the executor.
//!
//! The boot path keeps maskable interrupts disabled until the IRQ layer is
//! integrated. It polls devices and uses `pause` between iterations.

/// Disable maskable interrupts on the current CPU before polling boot devices.
#[cfg(target_os = "none")]
pub fn disable_interrupts() {
    // SAFETY: this function is compiled only for the ring-0 bare-metal kernel.
    unsafe { core::arch::asm!("cli", options(nomem, nostack)) };
}

/// Stop the current CPU after an unrecoverable boot error or panic.
#[cfg(target_os = "none")]
pub fn halt_forever() -> ! {
    disable_interrupts();
    loop {
        // SAFETY: ring-0 execution with maskable interrupts disabled; no memory
        // or stack operands are touched. NMI wakeups re-enter the halt loop.
        unsafe { core::arch::asm!("hlt", options(nomem, nostack)) };
    }
}
