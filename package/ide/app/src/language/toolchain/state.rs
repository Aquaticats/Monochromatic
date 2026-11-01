//! The outcome of asking the repository toolchain about a project,
//!  and what it means for the lookup and for the reader.

/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The tool directories the toolchain provides for a project.
/// Why:
///  The directories are searched for servers and placed on the servers' `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Provided = { directories: string[]; warnings: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provided {
    /// Existing absolute directories,
    /// in the toolchain's order.
    pub directories: Vec<PathBuf>,
    /// What the lookup printed while it succeeded,
    ///  such as a warning about the machine state.
    pub warnings: String,
}

/// What:
///  Why the lookup could not serve the project,
///  with its own words.
/// Why:
///  The reader is told the message and the remedy,
///  and the servers fall back to the plain `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failed = { message: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failed {
    /// The lookup's message,
    /// or what the application observed.
    pub message: String,
}

/// Facts about a failure.
impl Failed {
    /// What:
    ///  Whether the message is bubblewrap's own,
    ///  printed before the lookup ran.
    /// Why:
    ///  Bubblewrap starts every message with `bwrap:`,
    ///  and the lookup's own messages start with the quoted command,
    ///  so the first word tells whose message it is.
    ///  The reader is then not sent to the toolchain for a fault it never saw.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const stoppedBySandbox = (failed: Failed) => failed.message.trimStart().startsWith('bwrap:');
    /// ```
    pub fn stopped_by_sandbox(&self) -> bool {
        return self.message.trim_start().starts_with("bwrap:");
    }
}

/// What:
///  The outcome of asking the toolchain about a project.
///  An `enum` with data is a tagged union.
/// Why:
///  Each outcome changes the lookup differently,
///  and only a failure is reported to the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolchainState = { kind: 'disabled' } | { kind: 'unused'; reason: string }
///                     | { kind: 'provided'; provided: Provided } | { kind: 'failed'; failed: Failed };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolchainState {
    /// The setup never runs the lookup.
    Disabled,
    /// The toolchain does not apply:
    /// no toolchain plan for the project,
    /// or no sandbox.
    ///  The plain `PATH` lookup stays as it was.
    Unused(
        /// Why,
        ///  for the log.
        String,
    ),
    /// The project's toolchain was read.
    Provided(
        /// The directories the toolchain provides.
        Provided,
    ),
    /// The project has a toolchain the lookup could not use.
    Failed(
        /// The lookup's message.
        Failed,
    ),
}

/// Facts about a state.
impl ToolchainState {
    /// Whether a toolchain applies to the project,
    ///  working or not.
    ///  Then a leftover shim of a retired tool manager is never taken for a server.
    pub fn serves_project(&self) -> bool {
        return matches!(self, Self::Provided(_) | Self::Failed(_));
    }

    /// The tool directories,
    ///  empty unless the toolchain provided some.
    pub fn directories(&self) -> &[PathBuf] {
        return match self {
            Self::Provided(provided) => &provided.directories,
            _ => &[],
        };
    }

    /// What the lookup printed while it succeeded.
    pub fn warnings(&self) -> &str {
        return match self {
            Self::Provided(provided) => &provided.warnings,
            _ => "",
        };
    }

    /// What:
    ///  The paragraphs of the reader's notice about a failure,
    ///  or none when the lookup worked or did not apply.
    /// Why:
    ///  The notice must say what happened,
    /// quote the lookup,
    /// and say what the user does:
    ///  the application runs the lookup and never repairs the toolchain,
    ///  so the remedy names the command the user runs.
    ///  The remedy stands before the lookup's own words:
    ///  the display keeps only the first dozen lines,
    ///  and a failed run prints several long lines.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// advisory(root: string): string[]
    /// ```
    pub fn advisory(&self, root: &Path) -> Vec<String> {
        let Self::Failed(failed) = self else {
            return Vec::new();
        };
        if failed.stopped_by_sandbox() {
            return vec![
                format!(
                    "The sandbox that runs the toolchain lookup for {} stopped before the lookup started, so language servers are looked up on PATH only. The project's toolchain is not the cause.",
                    root.display()
                ),
                format!(
                    "The message does not say why. The IDE asks again when a file is displayed again, at least {} seconds after the last try. If it stays: check that `bwrap --version` runs in a terminal, that the IDE's private folder (`$XDG_CACHE_HOME/monochromatic-ide/language`, or `~/.cache/monochromatic-ide/language`) exists, has free space and can be written, and close any other IDE window on the same project.",
                    super::RETRY_AFTER.as_secs()
                ),
                format!("bubblewrap said: {}", failed.message),
            ];
        }
        return vec![
            format!(
                "The toolchain lookup for {} could not be used, so language servers are looked up on PATH only.",
                root.display()
            ),
            format!(
                "Fix what the lookup reports (run `{}` at the worktree root in a terminal to see it yourself; a worktree whose toolchain is not installed needs `mise run toolchain:install` first), then display a file again.",
                super::COMMAND_LINE
            ),
            format!("the toolchain lookup said: {}", failed.message),
        ];
    }
}

/// The notice text and the lookup facts of every state.
#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
