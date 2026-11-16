use core::fmt;

use crate::arch::x86_64::io::{read_u8, write_u8};

const COM1: u16 = 0x3f8;
const POLL_LIMIT: usize = 100_000;

/// An error while preparing or writing to the boot serial port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialError {
    /// The UART did not return the expected loopback byte.
    Unavailable,
    /// The UART did not become writable within the bounded poll limit.
    Timeout,
}

/// Exclusive boot-time owner of COM1.
pub struct SerialPort {
    _private: (),
}

impl SerialPort {
    /// Initialize COM1 and verify it using the UART's internal loopback mode.
    ///
    /// # Safety
    /// Call at ring 0 with exclusive ownership of COM1. No interrupt handler,
    /// other CPU or device driver may access this UART concurrently. A previous
    /// owner must be abandoned before reinitializing the port on a fatal path.
    pub unsafe fn initialize() -> Result<Self, SerialError> {
        // SAFETY: the caller owns COM1. These are the 16550 UART configuration
        // registers; disable IRQ, set divisor 1 and 8N1, reset FIFOs, then test
        // internal loopback before enabling normal output.
        unsafe {
            // Clear a possibly inherited DLAB before addressing IER at COM1+1.
            write_u8(COM1 + 3, 0x03);
            write_u8(COM1 + 1, 0);
            write_u8(COM1 + 3, 0x80);
            write_u8(COM1, 1);
            write_u8(COM1 + 1, 0);
            write_u8(COM1 + 3, 0x03);
            write_u8(COM1 + 2, 0xc7);
            write_u8(COM1 + 4, 0x1e);
            write_u8(COM1, 0xae);
        }
        let mut received = None;
        for _ in 0..POLL_LIMIT {
            // SAFETY: COM1 is still exclusively owned and in loopback mode.
            if unsafe { read_u8(COM1 + 5) } & 1 != 0 {
                // SAFETY: the line status reported a byte in the receive buffer.
                received = Some(unsafe { read_u8(COM1) });
                break;
            }
            core::hint::spin_loop();
        }
        // SAFETY: restore normal mode even when the loopback check failed.
        unsafe { write_u8(COM1 + 4, 0x0b) };
        if received != Some(0xae) {
            return Err(SerialError::Unavailable);
        }
        Ok(Self { _private: () })
    }

    fn write_byte(&mut self, byte: u8) -> Result<(), SerialError> {
        for _ in 0..POLL_LIMIT {
            // SAFETY: only this owner accesses COM1; reading line status has no
            // effect on shared memory or the register configuration.
            if unsafe { read_u8(COM1 + 5) } & 0x20 != 0 {
                // SAFETY: the transmitter is ready and the port is owned by self.
                unsafe { write_u8(COM1, byte) };
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(SerialError::Timeout)
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.write_byte(b'\r').map_err(|_| fmt::Error)?;
            }
            self.write_byte(byte).map_err(|_| fmt::Error)?;
        }
        Ok(())
    }
}

/// Best-effort bounded serial output after a fatal error.
///
/// # Safety
/// Maskable interrupts must be disabled and normal execution abandoned, with
/// no concurrent CPU/UART access. This reinitializes the UART and creates a
/// temporary owner solely for the terminal panic path; never resume execution.
pub unsafe fn emergency_write(message: fmt::Arguments<'_>) {
    use fmt::Write;
    // SAFETY: the caller has abandoned the normal owner and guarantees exclusive
    // UART access for this terminal path. Initialization and writes are bounded.
    if let Ok(mut port) = unsafe { SerialPort::initialize() } {
        let _ = port.write_fmt(message);
    }
}
