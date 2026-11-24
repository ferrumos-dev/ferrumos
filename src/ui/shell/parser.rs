//! Splitting of a command line into a command name and arguments.

use core::fmt;

/// Maximum number of arguments after the command name.
pub const MAX_ARGS: usize = 8;

/// A parsed command line borrowing from the input text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandLine<'a> {
    name: &'a str,
    args: [&'a str; MAX_ARGS],
    argc: usize,
}

impl<'a> CommandLine<'a> {
    /// Returns the command name (first word).
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the arguments following the command name.
    pub fn args(&self) -> &[&'a str] {
        &self.args[..self.argc]
    }
}

/// Reason a command line could not be parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    /// More than [`MAX_ARGS`] arguments were given.
    TooManyArguments,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyArguments => write!(f, "too many arguments (maximum {MAX_ARGS})"),
        }
    }
}

/// Splits `input` on ASCII whitespace.
///
/// Returns `Ok(None)` for an empty or blank line, which is not an error.
/// Quoting is not supported: every whitespace separates words.
pub fn parse(input: &str) -> Result<Option<CommandLine<'_>>, ParseError> {
    let mut words = input.split_ascii_whitespace();
    let Some(name) = words.next() else {
        return Ok(None);
    };
    let mut line = CommandLine {
        name,
        args: [""; MAX_ARGS],
        argc: 0,
    };
    for word in words {
        if line.argc == MAX_ARGS {
            return Err(ParseError::TooManyArguments);
        }
        line.args[line.argc] = word;
        line.argc += 1;
    }
    Ok(Some(line))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_lines_parse_to_none() {
        assert_eq!(parse(""), Ok(None));
        assert_eq!(parse("   \t "), Ok(None));
    }

    #[test]
    fn name_and_arguments_are_split_on_whitespace() {
        let line = parse("  stats   memory\tticks ").unwrap().unwrap();
        assert_eq!(line.name(), "stats");
        assert_eq!(line.args(), &["memory", "ticks"]);
    }

    #[test]
    fn command_without_arguments() {
        let line = parse("help").unwrap().unwrap();
        assert_eq!(line.name(), "help");
        assert!(line.args().is_empty());
    }

    #[test]
    fn maximum_argument_count_is_accepted() {
        let line = parse("x 1 2 3 4 5 6 7 8").unwrap().unwrap();
        assert_eq!(line.args().len(), MAX_ARGS);
    }

    #[test]
    fn too_many_arguments_is_an_error() {
        assert_eq!(
            parse("x 1 2 3 4 5 6 7 8 9"),
            Err(ParseError::TooManyArguments)
        );
    }
}
