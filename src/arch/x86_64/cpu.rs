//! Boot-CPU interrupt flags, scoped critical sections and halt operations.
//!
//! These helpers control the current CPU's maskable interrupt flag and restore
//! its prior state around shared-resource access. Task selection remains in
//! the portable scheduler, while task state and context ownership belong to
//! the executor. Task stacks and context-switch mechanisms remain future work.
//!
//! The boot path enables maskable interrupts only after the IDT and timer are
//! installed. Critical sections preserve their caller's interrupt-enable flag.

/// Disable maskable interrupts on the current CPU before polling boot devices.
#[cfg(target_os = "none")]
pub fn disable_interrupts() {
    // SAFETY: this function is compiled only for the ring-0 bare-metal kernel.
    // Omitting nomem also makes this a compiler memory barrier, so protected
    // accesses cannot be moved across the critical-section boundary.
    unsafe { core::arch::asm!("cli", options(nostack)) };
}

/// Return whether the current CPU permits maskable interrupts.
#[cfg(target_os = "none")]
pub fn interrupts_enabled() -> bool {
    let flags: u64;
    // SAFETY: pushfq/pop preserve the stack and read this CPU's flags. No
    // external memory is touched and the saved value is consumed immediately.
    unsafe { core::arch::asm!("pushfq", "pop {}", out(reg) flags, options(preserves_flags)) };
    flags & (1 << 9) != 0
}

/// Enable maskable interrupts after their handlers and devices are configured.
///
/// The boot owner must install a valid IDT and configure interrupt routing
/// before calling this function. Other CPUs and NMI remain separate concerns.
#[cfg(target_os = "none")]
pub fn enable_interrupts() {
    // SAFETY: this function is available only in the ring-0 kernel. Device and
    // handler installation is the boot owner's system-wide ordering invariant.
    unsafe { core::arch::asm!("sti", options(nostack)) };
}

/// Run a closure with maskable interrupts disabled, preserving the prior flag.
///
/// Nested sections remain disabled until the outer section returns. The guard
/// restores the flag during Rust unwinding too. The closure must not manually
/// enable interrupts, transfer its resources to another CPU or rely on this
/// local mask as protection against another CPU or an NMI handler.
#[cfg(target_os = "none")]
pub fn without_interrupts<T>(operation: impl FnOnce() -> T) -> T {
    struct RestoreInterrupts(bool);
    impl Drop for RestoreInterrupts {
        fn drop(&mut self) {
            if self.0 {
                enable_interrupts();
            }
        }
    }
    let enabled = interrupts_enabled();
    disable_interrupts();
    let restore = RestoreInterrupts(enabled);
    let result = operation();
    drop(restore);
    result
}

/// Enable interrupts and sleep until the next interrupt on this CPU.
///
/// Keeping `sti` and `hlt` adjacent uses x86's interrupt shadow to avoid an
/// interrupt arriving in a gap before the CPU starts waiting. Call only after
/// installing the IDT and interrupt controllers, with no critical lock held.
#[cfg(target_os = "none")]
pub fn enable_and_halt() {
    // SAFETY: the boot owner installed the handlers and caller holds no lock
    // needed by them. The instructions change no stack or general registers.
    unsafe { core::arch::asm!("sti", "hlt", options(nostack)) };
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
