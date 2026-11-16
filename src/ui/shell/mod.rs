//! Shell input parsing, command dispatch and textual feedback.
//!
//! The shell is portable: it never touches hardware, page tables or allocator
//! internals and it does not allocate. Its collaborators are reached through
//! two small interfaces:
//!
//! - [`Console`]: a text output that can also clear the screen. The display
//!   or serial driver implements it.
//! - [`StatsProvider`]: the source of the `stats` report. Telemetry implements
//!   it. Until a provider is attached, `stats` states that statistics are
//!   unavailable instead of failing.
//!
//! Input arrives one character at a time through [`Shell::handle_char`], so
//! the caller decides where characters come from (keyboard queue, serial port
//! or a test). The shell echoes accepted characters, edits the current line
//! and runs the command when the line is submitted.
//!
//! # Error handling
//!
//! Every user mistake produces a message and a new prompt; none of them panics
//! or leaves the shell in an inconsistent state:
//!
//! - an unknown command reports its name and suggests `help`;
//! - unexpected arguments are reported with the command name;
//! - a line longer than [`MAX_LINE_LEN`] bytes is truncated: extra characters
//!   are refused and the overflow is reported when the line is submitted;
//! - a line with more than [`MAX_ARGS`] arguments is rejected;
//! - control characters and non-ASCII characters are ignored.
//!
//! Errors returned by the console itself ([`core::fmt::Error`]) are propagated
//! to the caller, who owns the output device.
//!
//! # Example
//!
//! ```
//! use ferrumos::ui::shell::{Console, Shell};
//! use core::fmt::{self, Write};
//!
//! struct Screen;
//! impl Write for Screen {
//!     fn write_str(&mut self, _s: &str) -> fmt::Result { Ok(()) }
//! }
//! impl Console for Screen {
//!     fn clear(&mut self) -> fmt::Result { Ok(()) }
//! }
//!
//! let mut shell = Shell::new();
//! let mut screen = Screen;
//! shell.start(&mut screen)?;
//! for c in "version\n".chars() {
//!     shell.handle_char(c, &mut screen)?;
//! }
//! # Ok::<(), fmt::Error>(())
//! ```

mod command;
mod line;
mod parser;

pub use command::{Command, CommandError};
pub use line::{LineEditor, LineEvent, MAX_LINE_LEN};
pub use parser::{parse, CommandLine, ParseError, MAX_ARGS};

use core::fmt;

/// Prompt displayed before each command line.
pub const PROMPT: &str = "ferrum> ";

/// Text output used by the shell.
///
/// Implemented by the console driver. Writing goes through
/// [`core::fmt::Write`]; the extra methods cover terminal operations that are
/// not plain text.
pub trait Console: fmt::Write {
    /// Clears the screen and moves the cursor to the top-left corner.
    fn clear(&mut self) -> fmt::Result;

    /// Erases the character before the cursor.
    ///
    /// The default sequence (backspace, space, backspace) works on serial
    /// terminals; a display driver may override it.
    fn backspace(&mut self) -> fmt::Result {
        self.write_str("\x08 \x08")
    }
}

/// Source of the report printed by the `stats` command.
///
/// Implemented by telemetry. The provider writes a textual report and must
/// mark measurements that are not available yet rather than print zero. It is
/// called outside any critical section protecting the measurements.
pub trait StatsProvider {
    /// Writes the statistics report to `out`.
    fn write_stats(&self, out: &mut dyn fmt::Write) -> fmt::Result;
}

/// Interactive command shell.
///
/// Holds the line being edited and the optional statistics provider. It is
/// driven by [`Shell::start`] once, then [`Shell::handle_char`] for each input
/// character.
pub struct Shell<'a> {
    line: LineEditor,
    stats: Option<&'a dyn StatsProvider>,
}

impl Default for Shell<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Shell<'a> {
    /// Creates a shell without a statistics provider.
    pub const fn new() -> Self {
        Self {
            line: LineEditor::new(),
            stats: None,
        }
    }

    /// Creates a shell whose `stats` command reads from `provider`.
    pub const fn with_stats(provider: &'a dyn StatsProvider) -> Self {
        Self {
            line: LineEditor::new(),
            stats: Some(provider),
        }
    }

    /// Attaches or replaces the statistics provider.
    pub fn set_stats_provider(&mut self, provider: &'a dyn StatsProvider) {
        self.stats = Some(provider);
    }

    /// Prints the welcome banner and the first prompt.
    pub fn start(&mut self, console: &mut dyn Console) -> fmt::Result {
        writeln!(console, "FerrumOS {} shell", command::VERSION)?;
        writeln!(console, "Type 'help' to list the commands.")?;
        console.write_str(PROMPT)
    }

    /// Processes one input character.
    ///
    /// Accepted characters are echoed. On `'\n'` or `'\r'`, the line is
    /// parsed and executed, then a new prompt is printed.
    pub fn handle_char(&mut self, c: char, console: &mut dyn Console) -> fmt::Result {
        match self.line.push(c) {
            LineEvent::Echo(c) => console.write_char(c),
            LineEvent::Erase => console.backspace(),
            LineEvent::Ignored => Ok(()),
            LineEvent::Submit => {
                console.write_char('\n')?;
                if self.line.overflowed() {
                    writeln!(
                        console,
                        "error: line longer than {MAX_LINE_LEN} characters, ignored"
                    )?;
                } else {
                    self.execute(self.line.as_str(), console)?;
                }
                self.line.clear();
                console.write_str(PROMPT)
            }
        }
    }

    /// Parses and runs one command line, writing its output to `console`.
    ///
    /// An empty line does nothing. Errors are reported on the console; the
    /// returned error only comes from the console itself.
    pub fn execute(&self, input: &str, console: &mut dyn Console) -> fmt::Result {
        let line = match parse(input) {
            Ok(Some(line)) => line,
            Ok(None) => return Ok(()),
            Err(error) => return writeln!(console, "error: {error}"),
        };
        match Command::resolve(&line) {
            Ok(command) => command.run(console, self.stats),
            Err(error) => writeln!(console, "error: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::string::String;

    #[derive(Default)]
    struct TestConsole {
        out: String,
        clears: usize,
    }

    impl fmt::Write for TestConsole {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.out.push_str(s);
            Ok(())
        }
    }

    impl Console for TestConsole {
        fn clear(&mut self) -> fmt::Result {
            self.clears += 1;
            self.out.clear();
            Ok(())
        }
    }

    struct FakeStats;

    impl StatsProvider for FakeStats {
        fn write_stats(&self, out: &mut dyn fmt::Write) -> fmt::Result {
            writeln!(out, "uptime: 42 ticks")
        }
    }

    fn type_line(shell: &mut Shell<'_>, console: &mut TestConsole, text: &str) {
        for c in text.chars() {
            shell.handle_char(c, console).unwrap();
        }
    }

    #[test]
    fn start_prints_banner_and_prompt() {
        let mut console = TestConsole::default();
        Shell::new().start(&mut console).unwrap();
        assert!(console.out.contains("FerrumOS"));
        assert!(console.out.ends_with(PROMPT));
    }

    #[test]
    fn help_lists_every_command() {
        let mut console = TestConsole::default();
        Shell::new().execute("help", &mut console).unwrap();
        for command in Command::ALL {
            assert!(console.out.contains(command.name()));
        }
    }

    #[test]
    fn version_prints_crate_version() {
        let mut console = TestConsole::default();
        Shell::new().execute("version", &mut console).unwrap();
        assert!(console.out.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn clear_clears_the_console() {
        let mut console = TestConsole::default();
        Shell::new().execute("clear", &mut console).unwrap();
        assert_eq!(console.clears, 1);
    }

    #[test]
    fn unknown_command_reports_error_and_shell_continues() {
        let mut shell = Shell::new();
        let mut console = TestConsole::default();
        type_line(&mut shell, &mut console, "reboot-now\n");
        assert!(console.out.contains("unknown command 'reboot-now'"));
        assert!(console.out.ends_with(PROMPT));

        console.out.clear();
        type_line(&mut shell, &mut console, "version\n");
        assert!(console.out.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn unexpected_argument_is_reported() {
        let mut console = TestConsole::default();
        Shell::new().execute("clear now", &mut console).unwrap();
        assert!(console.out.contains("'clear' takes no argument"));
        assert_eq!(console.clears, 0);
    }

    #[test]
    fn stats_without_provider_reports_unavailable() {
        let mut console = TestConsole::default();
        Shell::new().execute("stats", &mut console).unwrap();
        assert!(console.out.contains("unavailable"));
    }

    #[test]
    fn stats_uses_provider() {
        let provider = FakeStats;
        let mut console = TestConsole::default();
        Shell::with_stats(&provider)
            .execute("stats", &mut console)
            .unwrap();
        assert!(console.out.contains("uptime: 42 ticks"));
    }

    #[test]
    fn empty_line_prints_only_a_new_prompt() {
        let mut shell = Shell::new();
        let mut console = TestConsole::default();
        type_line(&mut shell, &mut console, "   \n");
        assert_eq!(console.out, std::format!("   \n{PROMPT}"));
    }

    #[test]
    fn backspace_edits_the_line() {
        let mut shell = Shell::new();
        let mut console = TestConsole::default();
        type_line(&mut shell, &mut console, "versionx\x08\n");
        assert!(console.out.contains("\x08 \x08"));
        assert!(console.out.contains(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn overlong_line_is_rejected_without_running() {
        let mut shell = Shell::new();
        let mut console = TestConsole::default();
        for _ in 0..MAX_LINE_LEN + 10 {
            shell.handle_char('a', &mut console).unwrap();
        }
        shell.handle_char('\n', &mut console).unwrap();
        assert!(console.out.contains("line longer than"));
        assert!(!console.out.contains("unknown command"));

        console.out.clear();
        type_line(&mut shell, &mut console, "version\n");
        assert!(console.out.contains(env!("CARGO_PKG_VERSION")));
    }
}
