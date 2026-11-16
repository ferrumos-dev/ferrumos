//! Allocation-free line editing shared by console consumers.
//!
//! This layer buffers text; it does not parse or execute shell commands.

/// Fixed-capacity UTF-8 input line. Capacity is measured in bytes.
pub struct InputLine<const N: usize> {
    bytes: [u8; N],
    len: usize,
}

/// An input character was not accepted by the line buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineError {
    /// The complete UTF-8 character would exceed the buffer capacity.
    Full,
    /// Line delimiters and control characters are handled as input events.
    ControlCharacter,
}

impl<const N: usize> Default for InputLine<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> InputLine<N> {
    /// Create an empty input line without allocating.
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
        }
    }

    /// Append a printable character; failure leaves the line unchanged.
    pub fn push(&mut self, character: char) -> Result<(), LineError> {
        if character.is_control() {
            return Err(LineError::ControlCharacter);
        }
        let mut encoded = [0; 4];
        let text = character.encode_utf8(&mut encoded);
        if text.len() > N - self.len {
            return Err(LineError::Full);
        }
        self.bytes[self.len..self.len + text.len()].copy_from_slice(text.as_bytes());
        self.len += text.len();
        Ok(())
    }

    /// Remove the final Unicode scalar value, or return `None` for an empty line.
    pub fn pop(&mut self) -> Option<char> {
        let character = self.as_str().chars().next_back()?;
        self.len -= character.len_utf8();
        Some(character)
    }

    /// Borrow the current UTF-8 text.
    pub fn as_str(&self) -> &str {
        // All writes append a complete encoded char and pop removes a complete
        // char, so the active prefix is always valid UTF-8; retain checked access.
        core::str::from_utf8(&self.bytes[..self.len]).expect("input line UTF-8 invariant")
    }

    /// Empty the line while retaining its fixed storage.
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{InputLine, LineError};

    #[test]
    fn capacity_failure_preserves_full_utf8_characters() {
        let mut line = InputLine::<4>::new();
        assert_eq!(line.push('é'), Ok(()));
        assert_eq!(line.push('€'), Err(LineError::Full));
        assert_eq!(line.as_str(), "é");
        assert_eq!(line.push('à'), Ok(()));
        assert_eq!(line.push('a'), Err(LineError::Full));
        assert_eq!(line.pop(), Some('à'));
        assert_eq!(line.as_str(), "é");
        assert_eq!(line.pop(), Some('é'));
        assert_eq!(line.pop(), None);
    }

    #[test]
    fn controls_and_zero_capacity_never_mutate_the_line() {
        let mut line = InputLine::<0>::default();
        assert_eq!(line.push('a'), Err(LineError::Full));
        assert_eq!(line.push('\n'), Err(LineError::ControlCharacter));
        assert_eq!(line.as_str(), "");
        assert_eq!(line.pop(), None);
    }

    #[test]
    fn editing_and_clear_allow_reuse() {
        let mut line = InputLine::<4>::new();
        for c in "tesx".chars() {
            line.push(c).unwrap();
        }
        line.pop();
        line.push('t').unwrap();
        assert_eq!(line.as_str(), "test");
        line.clear();
        assert_eq!(line.as_str(), "");
        line.push('€').unwrap();
        assert_eq!(line.as_str(), "€");
    }
}
