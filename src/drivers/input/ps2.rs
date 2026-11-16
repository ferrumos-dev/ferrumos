//! Bounded polling of the PC i8042 controller.

use super::{InputEvent, KeyboardDecoder, KeyboardLayout};

const OUTPUT_FULL: u8 = 1;
const INPUT_FULL: u8 = 1 << 1;
const AUXILIARY: u8 = 1 << 5;
const TRANSMISSION_ERROR: u8 = (1 << 6) | (1 << 7);
const POLL_BUDGET: usize = 100_000;
const DRAIN_BUDGET: usize = 256;
const ACK_BUDGET: usize = 32;
const COMMAND_ATTEMPTS: usize = 3;

/// Failure while initializing or polling the PS/2 keyboard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ps2Error {
    /// The controller did not become ready within the polling budget.
    Timeout,
    /// The controller reported a parity or transmission error.
    Transmission,
    /// A keyboard command was not acknowledged after bounded retries.
    CommandRejected,
    /// A malformed scan-code sequence was discarded and the decoder reset.
    InvalidScancode,
}

trait ControllerIo {
    fn status(&mut self) -> u8;
    fn read_data(&mut self) -> u8;
    fn write_command(&mut self, value: u8);
    fn write_data(&mut self, value: u8);
}

struct Controller<I> {
    io: I,
    decoder: KeyboardDecoder,
}

impl<I: ControllerIo> Controller<I> {
    fn initialize(io: I, layout: KeyboardLayout) -> Result<Self, Ps2Error> {
        let mut controller = Self {
            io,
            decoder: KeyboardDecoder::new(layout),
        };
        controller.command(0xad)?; // Disable the keyboard port.
        controller.command(0xa7)?; // Disable the mouse port.
        controller.drain()?;
        controller.command(0x20)?;
        let mode = controller.read_keyboard_byte()?;
        controller.command(0x60)?;
        // IRQ1/IRQ12 off, both ports initially disabled, set-2 to set-1 translation on.
        controller.data((mode & !0x03) | 0x70)?;
        controller.command(0xae)?; // Enable only the keyboard port.
        controller.keyboard_command(0xf5)?; // Stop scanning while configuring.
        controller.drain()?;
        controller.keyboard_command(0xf0)?;
        controller.keyboard_command(0x02)?; // Keyboard emits set 2; controller translates.
        controller.keyboard_command(0xf4)?; // Start scanning.
        Ok(controller)
    }

    fn wait_to_write(&mut self) -> Result<(), Ps2Error> {
        for _ in 0..POLL_BUDGET {
            if self.io.status() & INPUT_FULL == 0 {
                return Ok(());
            }
            core::hint::spin_loop();
        }
        Err(Ps2Error::Timeout)
    }

    fn command(&mut self, value: u8) -> Result<(), Ps2Error> {
        self.wait_to_write()?;
        self.io.write_command(value);
        Ok(())
    }

    fn data(&mut self, value: u8) -> Result<(), Ps2Error> {
        self.wait_to_write()?;
        self.io.write_data(value);
        Ok(())
    }

    fn drain(&mut self) -> Result<(), Ps2Error> {
        for _ in 0..DRAIN_BUDGET {
            if self.io.status() & OUTPUT_FULL == 0 {
                return Ok(());
            }
            let _ = self.io.read_data();
        }
        Err(Ps2Error::Timeout)
    }

    fn read_keyboard_byte(&mut self) -> Result<u8, Ps2Error> {
        for _ in 0..POLL_BUDGET {
            let status = self.io.status();
            if status & OUTPUT_FULL == 0 {
                core::hint::spin_loop();
                continue;
            }
            let byte = self.io.read_data();
            if status & AUXILIARY != 0 {
                continue;
            }
            if status & TRANSMISSION_ERROR != 0 {
                return Err(Ps2Error::Transmission);
            }
            return Ok(byte);
        }
        Err(Ps2Error::Timeout)
    }

    fn keyboard_command(&mut self, value: u8) -> Result<(), Ps2Error> {
        for _ in 0..COMMAND_ATTEMPTS {
            self.data(value)?;
            for _ in 0..ACK_BUDGET {
                match self.read_keyboard_byte()? {
                    0xfa => return Ok(()),
                    0xfe => break, // Device requested retransmission of this byte.
                    0xfc | 0xfd => return Err(Ps2Error::CommandRejected),
                    _ => {} // A key already in flight before scanning was disabled.
                }
            }
        }
        Err(Ps2Error::CommandRejected)
    }

    fn poll_event(&mut self) -> Result<Option<InputEvent>, Ps2Error> {
        let status = self.io.status();
        if status & OUTPUT_FULL == 0 {
            return Ok(None);
        }
        let byte = self.io.read_data();
        if status & AUXILIARY != 0 {
            return Ok(None);
        }
        if status & TRANSMISSION_ERROR != 0 {
            self.decoder.reset();
            return Err(Ps2Error::Transmission);
        }
        self.decoder
            .feed(byte)
            .map_err(|_| Ps2Error::InvalidScancode)
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
mod hardware {
    use super::*;
    use crate::arch::x86_64::io::{read_u8, write_u8};
    use core::marker::PhantomData;

    const DATA_PORT: u16 = 0x60;
    const STATUS_COMMAND_PORT: u16 = 0x64;

    struct PortIo;

    impl ControllerIo for PortIo {
        fn status(&mut self) -> u8 {
            // SAFETY: PortIo is created only by initialize under exclusive i8042
            // ownership on a privileged x86_64 CPU with these fixed I/O ports.
            unsafe { read_u8(STATUS_COMMAND_PORT) }
        }

        fn read_data(&mut self) -> u8 {
            // SAFETY: Exclusive controller ownership comes from initialize;
            // callers check the output-full status before consuming the byte.
            unsafe { read_u8(DATA_PORT) }
        }

        fn write_command(&mut self, value: u8) {
            // SAFETY: Exclusive controller ownership comes from initialize;
            // callers wait until the input buffer is empty before this write.
            unsafe { write_u8(STATUS_COMMAND_PORT, value) }
        }

        fn write_data(&mut self, value: u8) {
            // SAFETY: Exclusive controller ownership comes from initialize;
            // callers wait until the input buffer is empty before this write.
            unsafe { write_u8(DATA_PORT, value) }
        }
    }

    /// Exclusive polling owner of the i8042 keyboard controller.
    ///
    /// The mouse port and keyboard/mouse IRQ delivery remain disabled. All
    /// waits are bounded by polling counts, not a calibrated wall-clock delay.
    /// Dropping this handle does not restore the firmware's controller state.
    pub struct Ps2Keyboard {
        controller: Controller<PortIo>,
        // Do not transfer the controller into another CPU/interrupt context.
        _not_send_sync: PhantomData<*mut ()>,
    }

    impl Ps2Keyboard {
        /// Configures set-1 translation and enables keyboard scanning.
        ///
        /// # Safety
        /// The caller runs at the privilege level required for x86 I/O and
        /// guarantees exclusive ownership of ports 0x60 and 0x64 for the entire
        /// handle lifetime. No firmware, other CPU, driver or IRQ handler may
        /// access the controller concurrently. Hardware interrupts must remain
        /// disabled until their routing and handlers are safely configured.
        pub unsafe fn initialize(layout: KeyboardLayout) -> Result<Self, Ps2Error> {
            Ok(Self {
                controller: Controller::initialize(PortIo, layout)?,
                _not_send_sync: PhantomData,
            })
        }

        /// Consumes at most one available byte without waiting for a key.
        ///
        /// Auxiliary-device bytes are discarded. Corrupt keyboard data resets
        /// decoder state and returns an error; callers may continue polling.
        pub fn poll_event(&mut self) -> Result<Option<InputEvent>, Ps2Error> {
            self.controller.poll_event()
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub use hardware::Ps2Keyboard;

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::{collections::VecDeque, vec::Vec};

    #[derive(Default)]
    struct FakeIo {
        received: VecDeque<(u8, u8)>,
        commands: Vec<u8>,
        written: Vec<u8>,
        config_write: bool,
        busy: bool,
        resend: usize,
    }

    impl ControllerIo for FakeIo {
        fn status(&mut self) -> u8 {
            if self.busy {
                INPUT_FULL
            } else {
                self.received.front().map_or(0, |(status, _)| *status)
            }
        }

        fn read_data(&mut self) -> u8 {
            self.received.pop_front().expect("status checked").1
        }

        fn write_command(&mut self, value: u8) {
            self.commands.push(value);
            match value {
                0x20 => self.received.push_back((OUTPUT_FULL, 0x07)),
                0x60 => self.config_write = true,
                _ => {}
            }
        }

        fn write_data(&mut self, value: u8) {
            self.written.push(value);
            if self.config_write {
                self.config_write = false;
            } else if self.resend > 0 {
                self.resend -= 1;
                self.received.push_back((OUTPUT_FULL, 0xfe));
            } else {
                self.received.push_back((OUTPUT_FULL, 0xfa));
            }
        }
    }

    #[test]
    fn initialization_disables_irqs_and_configures_translated_set_one() {
        let mut io = FakeIo::default();
        io.received.push_back((OUTPUT_FULL, 0x1e));
        let controller = Controller::initialize(io, KeyboardLayout::Us).unwrap();
        assert_eq!(controller.io.commands, [0xad, 0xa7, 0x20, 0x60, 0xae]);
        assert_eq!(controller.io.written, [0x74, 0xf5, 0xf0, 0x02, 0xf4]);
    }

    #[test]
    fn a_busy_controller_fails_instead_of_hanging() {
        let io = FakeIo {
            busy: true,
            ..FakeIo::default()
        };
        assert!(matches!(
            Controller::initialize(io, KeyboardLayout::Us),
            Err(Ps2Error::Timeout)
        ));
    }

    #[test]
    fn resend_is_retried_but_never_indefinitely() {
        let io = FakeIo {
            resend: 1,
            ..FakeIo::default()
        };
        let controller = Controller::initialize(io, KeyboardLayout::Us).unwrap();
        assert_eq!(controller.io.written, [0x74, 0xf5, 0xf5, 0xf0, 0x02, 0xf4]);
        let io = FakeIo {
            resend: COMMAND_ATTEMPTS,
            ..FakeIo::default()
        };
        assert!(matches!(
            Controller::initialize(io, KeyboardLayout::Us),
            Err(Ps2Error::CommandRejected)
        ));
    }

    #[test]
    fn mouse_data_is_discarded_and_corruption_does_not_leave_stuck_modifiers() {
        let mut controller = Controller::initialize(FakeIo::default(), KeyboardLayout::Us).unwrap();
        for packet in [
            (OUTPUT_FULL | AUXILIARY, 0x1e),
            (OUTPUT_FULL, 0x2a),
            (OUTPUT_FULL | TRANSMISSION_ERROR, 0xaa),
            (OUTPUT_FULL, 0x1e),
        ] {
            controller.io.received.push_back(packet);
        }
        assert_eq!(controller.poll_event(), Ok(None));
        assert_eq!(controller.poll_event(), Ok(None));
        assert_eq!(controller.poll_event(), Err(Ps2Error::Transmission));
        assert_eq!(
            controller.poll_event(),
            Ok(Some(InputEvent::Character('a')))
        );
        assert_eq!(controller.poll_event(), Ok(None));
    }
}
