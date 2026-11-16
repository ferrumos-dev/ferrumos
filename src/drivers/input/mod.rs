//! Typed keyboard input for the boot console and future shell.
//!
//! Scan-code decoding is independent of I/O and can be tested on the host.
//! The x86_64 adapter owns the i8042 controller and polls it with IRQ delivery
//! disabled; it can later be replaced by an interrupt-backed event producer.

use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1};

#[cfg(any(test, all(target_arch = "x86_64", target_os = "none")))]
mod ps2;

#[cfg(all(target_arch = "x86_64", target_os = "none"))]
pub use ps2::{Ps2Error, Ps2Keyboard};

/// Supported physical keyboard mappings.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum KeyboardLayout {
    /// Standard French AZERTY mapping, including accented characters and AltGr.
    #[default]
    Azerty,
    /// Standard US QWERTY mapping.
    Us,
}

impl KeyboardLayout {
    fn decoder(self) -> Keyboard<layouts::AnyLayout, ScancodeSet1> {
        let layout = match self {
            Self::Azerty => layouts::AnyLayout::Azerty(layouts::Azerty),
            Self::Us => layouts::AnyLayout::Us104Key(layouts::Us104Key),
        };
        Keyboard::new(
            ScancodeSet1::new(),
            layout,
            HandleControl::MapLettersToUnicode,
        )
    }
}

/// Input accepted by the console without exposing hardware scan codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputEvent {
    /// A printable Unicode character, or a tab from the physical Tab key.
    Character(char),
    /// The main or numeric-keypad Enter key.
    Enter,
    /// Erase the preceding character.
    Backspace,
    /// The physical Escape key.
    Escape,
}

/// A malformed or unsupported scan-code sequence was discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeError;

/// Allocation-free decoder for i8042-translated scan-code set 1.
///
/// Modifier and key-release events update internal state without generating
/// input. Function/navigation keys and control-letter shortcuts are ignored.
/// This layer does not implement compose/dead-key input or keyboard LED updates.
pub struct KeyboardDecoder {
    layout: KeyboardLayout,
    keyboard: Keyboard<layouts::AnyLayout, ScancodeSet1>,
}

impl KeyboardDecoder {
    /// Creates a decoder with all modifiers released and lock keys reset.
    pub fn new(layout: KeyboardLayout) -> Self {
        Self {
            layout,
            keyboard: layout.decoder(),
        }
    }

    /// Discards partial scan codes and resets modifiers after a lost byte.
    ///
    /// Caps Lock and Num Lock return to the decoder's initial state. This
    /// intentionally favors recovery over preserving potentially stale keys.
    pub fn reset(&mut self) {
        self.keyboard = self.layout.decoder();
    }

    /// Processes one translated scan-code byte.
    ///
    /// Returns `None` for an incomplete sequence, release, modifier or ignored
    /// key. On malformed input the decoder resets, so the next complete key
    /// sequence can be accepted without a stuck modifier or prefix.
    pub fn feed(&mut self, scancode: u8) -> Result<Option<InputEvent>, DecodeError> {
        let event = match self.keyboard.add_byte(scancode) {
            Ok(Some(event)) => event,
            Ok(None) => return Ok(None),
            Err(_) => {
                self.reset();
                return Err(DecodeError);
            }
        };
        let key = event.code;
        let Some(decoded) = self.keyboard.process_keyevent(event) else {
            return Ok(None);
        };
        let input = match key {
            KeyCode::Return | KeyCode::NumpadEnter => Some(InputEvent::Enter),
            KeyCode::Backspace => Some(InputEvent::Backspace),
            KeyCode::Escape => Some(InputEvent::Escape),
            KeyCode::Tab => Some(InputEvent::Character('\t')),
            _ => match decoded {
                DecodedKey::Unicode(character) if !character.is_control() => {
                    Some(InputEvent::Character(character))
                }
                _ => None,
            },
        };
        Ok(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn azerty_and_us_use_distinct_physical_mappings() {
        let mut french = KeyboardDecoder::new(KeyboardLayout::Azerty);
        let mut us = KeyboardDecoder::new(KeyboardLayout::Us);
        assert_eq!(french.feed(0x10), Ok(Some(InputEvent::Character('a'))));
        assert_eq!(us.feed(0x10), Ok(Some(InputEvent::Character('q'))));
        assert_eq!(french.feed(0x03), Ok(Some(InputEvent::Character('é'))));
    }

    #[test]
    fn shift_release_and_caps_lock_keep_correct_state() {
        let mut decoder = KeyboardDecoder::new(KeyboardLayout::Azerty);
        assert_eq!(decoder.feed(0x2a), Ok(None));
        assert_eq!(decoder.feed(0x10), Ok(Some(InputEvent::Character('A'))));
        assert_eq!(decoder.feed(0x90), Ok(None));
        assert_eq!(decoder.feed(0xaa), Ok(None));
        assert_eq!(decoder.feed(0x10), Ok(Some(InputEvent::Character('a'))));
        assert_eq!(decoder.feed(0x3a), Ok(None));
        assert_eq!(decoder.feed(0xba), Ok(None));
        assert_eq!(decoder.feed(0x10), Ok(Some(InputEvent::Character('A'))));
        assert_eq!(decoder.feed(0x2a), Ok(None));
        assert_eq!(decoder.feed(0x10), Ok(Some(InputEvent::Character('a'))));
    }

    #[test]
    fn editing_keys_are_typed_events_and_releases_are_silent() {
        let mut decoder = KeyboardDecoder::new(KeyboardLayout::Us);
        for (press, release, expected) in [
            (0x1c, 0x9c, InputEvent::Enter),
            (0x0e, 0x8e, InputEvent::Backspace),
            (0x01, 0x81, InputEvent::Escape),
            (0x0f, 0x8f, InputEvent::Character('\t')),
        ] {
            assert_eq!(decoder.feed(press), Ok(Some(expected)));
            assert_eq!(decoder.feed(release), Ok(None));
        }
        assert_eq!(decoder.feed(0xe0), Ok(None));
        assert_eq!(decoder.feed(0x1c), Ok(Some(InputEvent::Enter)));
    }

    #[test]
    fn navigation_and_control_shortcuts_do_not_generate_text() {
        let mut decoder = KeyboardDecoder::new(KeyboardLayout::Us);
        for byte in [0xe0, 0x48, 0xe0, 0xc8, 0x3b, 0xbb, 0x1d, 0x2e, 0xae, 0x9d] {
            assert_eq!(decoder.feed(byte), Ok(None));
        }
        assert_eq!(decoder.feed(0x2e), Ok(Some(InputEvent::Character('c'))));
    }

    #[test]
    fn altgr_works_without_emitting_the_modifier() {
        let mut decoder = KeyboardDecoder::new(KeyboardLayout::Azerty);
        assert_eq!(decoder.feed(0xe0), Ok(None));
        assert_eq!(decoder.feed(0x38), Ok(None));
        assert_eq!(decoder.feed(0x0b), Ok(Some(InputEvent::Character('@'))));
        assert_eq!(decoder.feed(0xe0), Ok(None));
        assert_eq!(decoder.feed(0xb8), Ok(None));
        assert_eq!(decoder.feed(0x0b), Ok(Some(InputEvent::Character('à'))));
    }

    #[test]
    fn malformed_prefix_recovers_without_a_stuck_shift() {
        let mut decoder = KeyboardDecoder::new(KeyboardLayout::Us);
        assert_eq!(decoder.feed(0x2a), Ok(None));
        assert_eq!(decoder.feed(0xe0), Ok(None));
        assert_eq!(decoder.feed(0x00), Err(DecodeError));
        assert_eq!(decoder.feed(0x1e), Ok(Some(InputEvent::Character('a'))));
    }
}
