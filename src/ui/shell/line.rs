//! Bounded line editor fed one character at a time.

/// Maximum number of bytes kept for one command line.
pub const MAX_LINE_LEN: usize = 128;

/// Result of feeding one character to the [`LineEditor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEvent {
    /// The character was added and should be echoed.
    Echo(char),
    /// The last character was removed and should be erased on screen.
    Erase,
    /// The line is complete and ready to be executed.
    Submit,
    /// Nothing changed: control character, non-ASCII character, backspace on
    /// an empty line or character refused because the line is full.
    Ignored,
}

/// Fixed-capacity buffer holding the line being typed.
///
/// Only printable ASCII is stored, so the content is always valid UTF-8 and
/// one byte equals one column on screen. Characters typed beyond
/// [`MAX_LINE_LEN`] are refused and the line is marked as overflowed until
/// [`LineEditor::clear`] is called.
pub struct LineEditor {
    buf: [u8; MAX_LINE_LEN],
    len: usize,
    overflowed: bool,
}

impl Default for LineEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineEditor {
    /// Creates an empty line.
    pub const fn new() -> Self {
        Self {
            buf: [0; MAX_LINE_LEN],
            len: 0,
            overflowed: false,
        }
    }

    /// Feeds one character and reports what the caller should display.
    ///
    /// `'\n'` and `'\r'` submit the line; `'\x08'` (backspace) and `'\x7f'`
    /// (delete) remove the last character.
    pub fn push(&mut self, c: char) -> LineEvent {
        match c {
            '\n' | '\r' => LineEvent::Submit,
            '\x08' | '\x7f' => {
                if self.len == 0 {
                    LineEvent::Ignored
                } else {
                    self.len -= 1;
                    LineEvent::Erase
                }
            }
            ' '..='~' => {
                if self.len == MAX_LINE_LEN {
                    self.overflowed = true;
                    LineEvent::Ignored
                } else {
                    // The match arm guarantees `c` is printable ASCII.
                    self.buf[self.len] = c as u8;
                    self.len += 1;
                    LineEvent::Echo(c)
                }
            }
            _ => LineEvent::Ignored,
        }
    }

    /// Returns the current content of the line.
    pub fn as_str(&self) -> &str {
        // Only printable ASCII bytes are ever stored.
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }

    /// Returns `true` if characters were refused because the line was full.
    pub fn overflowed(&self) -> bool {
        self.overflowed
    }

    /// Empties the line and resets the overflow flag.
    pub fn clear(&mut self) {
        self.len = 0;
        self.overflowed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printable_characters_are_stored_and_echoed() {
        let mut line = LineEditor::new();
        assert_eq!(line.push('h'), LineEvent::Echo('h'));
        assert_eq!(line.push('i'), LineEvent::Echo('i'));
        assert_eq!(line.as_str(), "hi");
    }

    #[test]
    fn newline_and_carriage_return_submit() {
        let mut line = LineEditor::new();
        assert_eq!(line.push('\n'), LineEvent::Submit);
        assert_eq!(line.push('\r'), LineEvent::Submit);
    }

    #[test]
    fn backspace_removes_last_character() {
        let mut line = LineEditor::new();
        line.push('a');
        line.push('b');
        assert_eq!(line.push('\x08'), LineEvent::Erase);
        assert_eq!(line.push('\x7f'), LineEvent::Erase);
        assert_eq!(line.as_str(), "");
        assert_eq!(line.push('\x08'), LineEvent::Ignored);
    }

    #[test]
    fn control_and_non_ascii_characters_are_ignored() {
        let mut line = LineEditor::new();
        assert_eq!(line.push('\t'), LineEvent::Ignored);
        assert_eq!(line.push('\x1b'), LineEvent::Ignored);
        assert_eq!(line.push('é'), LineEvent::Ignored);
        assert_eq!(line.as_str(), "");
    }

    #[test]
    fn full_line_refuses_characters_and_flags_overflow() {
        let mut line = LineEditor::new();
        for _ in 0..MAX_LINE_LEN {
            assert_eq!(line.push('x'), LineEvent::Echo('x'));
        }
        assert!(!line.overflowed());
        assert_eq!(line.push('y'), LineEvent::Ignored);
        assert!(line.overflowed());
        assert_eq!(line.as_str().len(), MAX_LINE_LEN);

        line.clear();
        assert!(!line.overflowed());
        assert_eq!(line.as_str(), "");
    }
}
