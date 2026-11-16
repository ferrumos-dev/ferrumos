//! Bounded, allocation-free COM1 output for boot diagnostics.
//!
//! Hardware access is available only in the x86_64 bare-metal build. A single
//! owner writes at 115200 baud, 8 data bits, no parity, one stop bit, with UART
//! interrupts disabled. Newlines are expanded to CRLF for serial terminals.

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
mod uart;
#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub use uart::{emergency_write, SerialError, SerialPort};
