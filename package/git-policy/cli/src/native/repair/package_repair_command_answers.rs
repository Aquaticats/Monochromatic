//! What:
//!  The answers one tool's run is asked for and hands back:
//! the command it is given,
//! the two bounds it runs under,
//! how it ended,
//! and what that run left for the person to read.
//! Why:
//!  The Cargo repair,
//! the pnpm repair and the toolchain repair each hand a run one command,
//! and each words the end it got,
//! so the four types live where both sides can name them
//! without naming how a run is made.
//! They are values:
//! no run,
//! no clock and no child is in this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ToolCommand = { arguments: string[]; directory: string; set: [string, string][] };
//! type ToolLimits = { window: number; ceiling: number };
//! type ToolEnd = { kind: 'succeeded' } | { kind: 'failed'; reason: string } | { kind: 'timed-out' } | { kind: 'ceiling'; afterMs: number };
//! type ToolRun = { end: ToolEnd; millis: number; stdout: Uint8Array; notes: string[] };
//! ```

/// Import the owned,
/// platform-native string a program argument and a variable are.
use std::ffi::OsString;
/// Import the owned filesystem path a run happens in.
use std::path::PathBuf;
/// Import the span of time a bound is written as.
use std::time::Duration;
/// What:
///  One program start:
/// its arguments,
/// the directory it runs in,
/// and the variables added to its environment.
/// Why:
///   The program path is found separately,
/// so a test can name a fixture and a person's search path is read once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolCommand = { arguments: string[]; directory: string; set: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCommand {
    /// The arguments after the program name.
    pub arguments: Vec<OsString>,
    /// The working directory.
    pub directory: PathBuf,
    /// Variables added to the inherited environment.
    pub set: Vec<(OsString, OsString)>,
}

/// What:
///  How long a started program may stay silent and how long it may run in all.
/// Why:
///   Two bounds travel together:
/// the quiet window ends a program that shows nothing,
/// and the ceiling ends one that keeps showing
///  something (owner,
/// 2026-11-04,
/// "Silence plus a ceiling (Recommended)").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolLimits = { window: number; ceiling: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolLimits {
    /// How long the program may show no sign of work.
    pub window: Duration,
    /// The longest the program runs,
    /// whatever it shows.
    pub ceiling: Duration,
}

/// What:
///  How a started program ended.
/// Why:
///   The repair words each case differently,
/// so they stay apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolEnd = { kind: 'succeeded' } | { kind: 'failed'; reason: string } | { kind: 'timed-out' } | { kind: 'ceiling'; afterMs: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolEnd {
    /// Exit status zero.
    Succeeded,
    /// The program did not start,
    /// or ended with a failure status;
    /// the text names the status and the tail of its output.
    Failed(String),
    /// The program showed no sign of work for its window,
    /// and was stopped.
    TimedOut,
    /// The program ran as long as the ceiling allows,
    /// whatever it showed,
    /// and was stopped with its whole process group.
    CeilingReached(Duration),
}

/// What:
///  The end of one run and how long it took.
/// Why:
///   The time is reported to the person and measured by the docs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolRun = { end: ToolEnd; millis: number; stdout: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolRun {
    /// How the program ended.
    pub end: ToolEnd,
    /// Wall time from start to end,
    /// in milliseconds.
    pub millis: u128,
    /// What the program wrote to its standard output;
    /// empty when it did not run to its end.
    pub stdout: Vec<u8>,
    /// What was left out of the output or could not be read,
    /// in words for the person.
    /// Output with a note is not the whole output:
    ///  a parser of `stdout` must not take it for a complete result.
    pub notes: Vec<String>,
}
