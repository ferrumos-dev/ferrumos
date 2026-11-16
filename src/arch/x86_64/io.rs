//! Privileged x86 port I/O, isolated from portable policies.

use core::arch::asm;

/// Read one byte from an I/O port.
///
/// # Safety
/// The caller must execute at a privilege level allowing port I/O and own the
/// device/register being accessed. The read must be valid in its current state.
#[inline]
pub unsafe fn read_u8(port: u16) -> u8 {
    let value: u8;
    // SAFETY: the caller guarantees port access permissions and device ownership.
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags))
    };
    value
}

/// Write one byte to an I/O port.
///
/// # Safety
/// The caller must execute at a privilege level allowing port I/O and own the
/// device/register being accessed. The value must be valid for that register.
#[inline]
pub unsafe fn write_u8(port: u16, value: u8) {
    // SAFETY: the caller guarantees port permissions, ownership and register value.
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags))
    };
}
