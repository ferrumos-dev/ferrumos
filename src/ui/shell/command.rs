//! Built-in commands and their execution.

use core::fmt;

use super::{CommandLine, Console, StatsProvider};

/// Version reported by the `version` command and the banner.
pub(super) const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A built-in shell command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Lists the available commands.
    Help,
    /// Prints the FerrumOS version.
    Version,
    /// Clears the screen.
    Clear,
    /// Prints memory and uptime statistics from the [`StatsProvider`].
    Stats,
}

/// Reason a parsed line does not designate a runnable command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError<'a> {
    /// No built-in command has this name.
    Unknown(&'a str),
    /// The command takes no argument but `argument` was given.
    UnexpectedArgument {
        /// The command that was called.
        command: Command,
        /// The first extra argument.
        argument: &'a str,
    },
}

impl fmt::Display for CommandError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(name) => {
                write!(f, "unknown command '{name}', type 'help' for the list")
            }
            Self::UnexpectedArgument { command, argument } => write!(
                f,
                "'{}' takes no argument (got '{argument}')",
                command.name()
            ),
        }
    }
}

impl Command {
    /// Every built-in command, in the order shown by `help`.
    pub const ALL: [Command; 4] = [Self::Help, Self::Version, Self::Clear, Self::Stats];

    /// Returns the name typed to run the command.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Help => "help",
            Self::Version => "version",
            Self::Clear => "clear",
            Self::Stats => "stats",
        }
    }

    /// Returns the one-line description shown by `help`.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Help => "list the available commands",
            Self::Version => "print the FerrumOS version",
            Self::Clear => "clear the screen",
            Self::Stats => "show memory usage and uptime",
        }
    }

    /// Finds the command named `name`. Names are case-sensitive.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|command| command.name() == name)
    }

    /// Maps a parsed line to a command and checks its arguments.
    ///
    /// No built-in command currently takes arguments.
    pub fn resolve<'a>(line: &CommandLine<'a>) -> Result<Self, CommandError<'a>> {
        let command = Self::from_name(line.name()).ok_or(CommandError::Unknown(line.name()))?;
        match line.args().first() {
            Some(&argument) => Err(CommandError::UnexpectedArgument { command, argument }),
            None => Ok(command),
        }
    }

    /// Runs the command, writing its output to `console`.
    ///
    /// `stats` is the provider used by [`Command::Stats`]; when it is `None`,
    /// the command reports that statistics are unavailable.
    pub fn run(self, console: &mut dyn Console, stats: Option<&dyn StatsProvider>) -> fmt::Result {
        match self {
            Self::Help => {
                writeln!(console, "Available commands:")?;
                for command in Self::ALL {
                    writeln!(console, "  {:<8} {}", command.name(), command.description())?;
                }
                Ok(())
            }
            Self::Version => writeln!(console, "FerrumOS {VERSION}"),
            Self::Clear => console.clear(),
            Self::Stats => match stats {
                Some(provider) => provider.write_stats(console),
                None => writeln!(console, "stats: statistics unavailable (no provider)"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse;
    use super::*;

    #[test]
    fn every_command_is_found_by_its_name() {
        for command in Command::ALL {
            assert_eq!(Command::from_name(command.name()), Some(command));
        }
    }

    #[test]
    fn names_are_case_sensitive() {
        assert_eq!(Command::from_name("HELP"), None);
    }

    #[test]
    fn resolve_rejects_unknown_command() {
        let line = parse("foo").unwrap().unwrap();
        assert_eq!(Command::resolve(&line), Err(CommandError::Unknown("foo")));
    }

    #[test]
    fn resolve_rejects_arguments() {
        let line = parse("version extra").unwrap().unwrap();
        assert_eq!(
            Command::resolve(&line),
            Err(CommandError::UnexpectedArgument {
                command: Command::Version,
                argument: "extra",
            })
        );
    }
}
