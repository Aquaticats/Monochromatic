//! The outcome of asking mise about a project,
//!  and what it means for the lookup and for the reader.

/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The tool directories mise provides for a project.
/// Why:
///  The directories are searched for servers and placed on the servers' `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Provided = { directories: string[]; configs: string[]; warnings: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Provided {
    /// Existing absolute directories,
    /// in mise's order.
    pub directories: Vec<PathBuf>,
    /// The project's own mise configuration files.
    pub configs: Vec<PathBuf>,
    /// What mise printed while it succeeded,
    ///  such as `missing: taplo@0.9.0` for a listed tool that is not installed.
    pub warnings: String,
}

/// What:
///  Why mise could not serve the project,
///  with mise's own words.
/// Why:
///  The reader is told the message and the remedy,
///  and the servers fall back to the plain `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Failed = { message: string; untrusted?: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failed {
    /// Mise's message,
    /// or what the application observed.
    pub message: String,
    /// The configuration file mise refused as not trusted,
    /// when that is the cause.
    pub untrusted: Option<String>,
}

/// Facts about a failure.
impl Failed {
    /// What:
    ///  Whether the message is bubblewrap's own,
    ///  printed before mise ran.
    /// Why:
    ///  Bubblewrap starts every message with `bwrap:`,
    ///  and mise starts its own with `mise`,
    ///  so the first word tells whose message it is.
    ///  The reader is then not sent to `mise doctor` for a fault mise never saw.
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
///  The outcome of asking mise about a project.
///  An `enum` with data is a tagged union.
/// Why:
///  Each outcome changes the lookup differently,
///  and only a failure is reported to the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MiseState = { kind: 'disabled' } | { kind: 'unused'; reason: string }
///                | { kind: 'provided'; provided: Provided } | { kind: 'failed'; failed: Failed };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MiseState {
    /// The setup never runs mise.
    Disabled,
    /// Mise does not apply:
    /// no mise program,
    /// no configuration for the project,
    /// or no sandbox.
    ///  The plain `PATH` lookup stays as it was.
    Unused(
        /// Why,
        ///  for the log.
        String,
    ),
    /// The project's mise configuration was read.
    Provided(
        /// The directories and configuration files mise provided.
        Provided,
    ),
    /// The project has a mise configuration that mise could not use.
    Failed(
        /// Mise's message,
        ///  and the file it refused when there is one.
        Failed,
    ),
}

/// Facts about a state.
impl MiseState {
    /// Whether a mise configuration applies to the project,
    ///  working or not.
    ///  Then a mise shim is never taken for a server.
    pub fn serves_project(&self) -> bool {
        return matches!(self, Self::Provided(_) | Self::Failed(_));
    }

    /// The tool directories,
    ///  empty unless mise provided some.
    pub fn directories(&self) -> &[PathBuf] {
        return match self {
            Self::Provided(provided) => &provided.directories,
            _ => &[],
        };
    }

    /// What mise printed while it succeeded.
    pub fn warnings(&self) -> &str {
        return match self {
            Self::Provided(provided) => &provided.warnings,
            _ => "",
        };
    }

    /// What:
    ///  The paragraphs of the reader's notice about a failure,
    ///  or none when mise worked or did not apply.
    /// Why:
    ///  The notice must name the configuration,
    /// quote mise,
    /// and say what the user does:
    ///  the application never trusts a configuration,
    /// because trusting lets the project's
    ///  `[env]` commands run.
    ///  The remedy stands before mise's own words:
    ///  the display keeps only the first dozen lines,
    ///  and mise's message is several long lines.
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
                    "The sandbox that runs mise for {} stopped before mise started, so language servers are looked up on PATH only. The project's mise configuration is not the cause.",
                    root.display()
                ),
                format!(
                    "The message does not say why. The IDE asks again when a file is displayed again, at least {} seconds after the last try. If it stays: check that `bwrap --version` runs in a terminal, that the IDE's private folder (`$XDG_CACHE_HOME/monochromatic-ide/language`, or `~/.cache/monochromatic-ide/language`) exists, has free space and can be written, and close any other IDE window on the same project.",
                    super::RETRY_AFTER.as_secs()
                ),
                format!("bubblewrap said: {}", failed.message),
            ];
        }
        let remedy = match &failed.untrusted {
            Some(file) => format!(
                "If you trust that mise configuration, run `mise trust` on {file} in a terminal yourself (the IDE never does), then display a file again."
            ),
            None => format!(
                "Fix what mise names (`mise doctor` in {} shows more), then display a file again.",
                root.display()
            ),
        };
        return vec![
            format!(
                "The mise configuration for {} could not be used, so language servers are looked up on PATH only.",
                root.display()
            ),
            remedy,
            format!("mise said: {}", failed.message),
        ];
    }
}

/// The notice text and the lookup facts of every state.
#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
