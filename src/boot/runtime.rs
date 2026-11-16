use core::fmt::{self, Write};

use bootloader_api::BootInfo;

use crate::arch::x86_64::cpu;
use crate::drivers::console::InputLine;
use crate::drivers::display::TextConsole;
use crate::drivers::input::{InputEvent, KeyboardLayout, Ps2Keyboard};
use crate::drivers::serial::SerialPort;

struct Console<'a> {
    display: TextConsole<'a>,
    serial: SerialPort,
    serial_enabled: bool,
}

impl Write for Console<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.display.write_str(text)?;
        // A failed diagnostic port must not truncate formatted screen output.
        if self.serial_enabled && self.serial.write_str(text).is_err() {
            self.serial_enabled = false;
        }
        Ok(())
    }
}

impl Console<'_> {
    fn erase_character(&mut self) {
        self.display.write_char('\u{8}');
        if self.serial_enabled && self.serial.write_str("\u{8} \u{8}").is_err() {
            self.serial_enabled = false;
        }
    }
}

/// Initialize the boot console and poll keyboard input on the boot CPU.
///
/// Called once from the bootloader entry point with exclusive boot information.
/// The framebuffer, COM1 and PS/2 controller stay owned by this function. No
/// interrupt handler or other CPU may use these devices during this phase.
pub fn run(boot_info: &'static mut BootInfo) -> ! {
    cpu::disable_interrupts();
    // SAFETY: this is the first entry on the only active CPU. Interrupts are
    // disabled and no other driver owns COM1 for the lifetime of the boot loop.
    let mut serial = match unsafe { SerialPort::initialize() } {
        Ok(serial) => serial,
        Err(_) => cpu::halt_forever(),
    };
    let _ = serial.write_str("FERRUMOS_BOOT_OK\n");

    let Some(framebuffer) = boot_info.framebuffer.as_mut() else {
        let _ = serial.write_str("BOOT_FAILED: framebuffer unavailable\n");
        cpu::halt_forever();
    };
    let info = framebuffer.info();
    let display = match TextConsole::new(framebuffer.buffer_mut(), info) {
        Ok(display) => display,
        Err(error) => {
            let _ = writeln!(serial, "BOOT_FAILED: framebuffer {error:?}");
            cpu::halt_forever();
        }
    };
    let _ = serial.write_str("DISPLAY_READY\n");
    let _ = writeln!(serial, "FRAMEBUFFER {}x{}", info.width, info.height);
    let mut console = Console {
        display,
        serial,
        serial_enabled: true,
    };
    let _ = writeln!(console, "FerrumOS {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(
        console,
        "Console clavier AZERTY - Entree valide, Echap efface."
    );
    let _ = writeln!(console, "Demarrage, affichage et saisie disponibles.\n");

    // SAFETY: the boot CPU exclusively owns i8042; maskable interrupts remain
    // disabled and the polling driver disables keyboard/mouse IRQ generation.
    let mut keyboard = match unsafe { Ps2Keyboard::initialize(KeyboardLayout::Azerty) } {
        Ok(keyboard) => keyboard,
        Err(error) => {
            let _ = writeln!(console, "BOOT_FAILED: keyboard {error:?}");
            cpu::halt_forever();
        }
    };
    let _ = console.serial.write_str("KEYBOARD_READY\n");
    let _ = console.write_str("> ");
    let mut line = InputLine::<128>::new();
    let mut input_error_reported = false;
    loop {
        match keyboard.poll_event() {
            Ok(Some(InputEvent::Character(character))) => {
                if line.push(character).is_ok() {
                    let _ = console.write_char(character);
                }
            }
            Ok(Some(InputEvent::Backspace)) => {
                if line.pop().is_some() {
                    console.erase_character();
                }
            }
            Ok(Some(InputEvent::Enter)) => {
                let _ = console.write_str("\n");
                // A stable serial acknowledgement makes hardware-input checks
                // independent of terminal control characters and display pixels.
                let _ = writeln!(console.serial, "INPUT_LINE: {}", line.as_str());
                line.clear();
                let _ = console.write_str("> ");
            }
            Ok(Some(InputEvent::Escape)) => {
                while line.pop().is_some() {
                    console.erase_character();
                }
            }
            Ok(None) => {}
            Err(error) => {
                // Keep malformed input observable without flooding the console
                // or interpreting corrupted scancodes as user characters.
                if !input_error_reported {
                    let _ = writeln!(console.serial, "INPUT_WARNING: {error:?}");
                    input_error_reported = true;
                }
            }
        }
        core::hint::spin_loop();
    }
}
