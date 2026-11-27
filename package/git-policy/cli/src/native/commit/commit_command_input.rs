//! What:
//!  What one commit command is handed and how it ended:
//!  the facts the lifecycle already resolved,
//!  and the exit status the process returns.
//! Why:
//!  Everything the command wrote it wrote itself and in order,
//!  so only the exit status is left to the caller;
//!  a stop by a signal travels beside that status,
//!  because the command's record must be on disk before the process ends by the signal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CommitCommand = { context: GitContext; arguments: string[]; /* ... */ };
//! type CommitEnding = { code: number; signal?: number };
//! ```

/// The `git commit` options Git's own table read.
use crate::command_commit::CommitRegion;
/// The loaded configuration.
use crate::config_schema::CliGitConfig;
/// The transaction Git wrapper,
///  whose context a commit runs in.
use crate::transaction_git::GitContext;
/// The wrapper controls given on the command line.
use crate::wrapper_controls::Controls;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;

/// What:
///  The ending of one commit command.
/// Why:
///   Everything the command wrote it wrote itself and in order,
/// so only the exit status is left to the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = code;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitEnding {
    /// The process exit status,
    /// which a command that ends by a signal never returns a caller.
    pub code: i32,
    /// What:
    ///  The signal this command ends by,
    ///  when it was stopped by one.
    /// Why:
    ///  A stop by a signal is reported to the person before the process ends by that signal,
    ///  and the record of the commit is written between the two:
    ///  the command's own end used to raise the signal inside the report,
    ///  so a commit the person interrupted wrote no record at all,
    ///  and no record of 1,792 has ever said `client-signal` (`SPEC.md`,
    /// "The commit protocol").
    ///  The ending carries the signal so the raise happens after the record is on disk.
    pub signal: Option<i32>,
}

/// What:
///  Everything the commit command reads from the wrapped invocation.
/// Why:
///   The command needs the identity and the loaded configuration the lifecycle already resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { rawArgs, gitPath, runtimeResolution, ... }
/// ```
pub struct CommitCommand<'a> {
    /// Real Git,
    /// its overlay and the global options.
    pub context: &'a GitContext,
    /// The invocation's environment.
    pub environment: &'a [(OsString, OsString)],
    /// The arguments with the wrapper's controls removed.
    pub arguments: &'a [OsString],
    /// The index of `commit` within the arguments.
    pub subcommand_index: usize,
    /// The commit options Git's table read.
    pub region: &'a CommitRegion,
    /// The loaded configuration.
    pub config: &'a CliGitConfig,
    /// The wrapper controls given on the command line.
    pub controls: &'a Controls,
    /// The directory this command was started in,
    ///  read once by the executable's entry;
    ///  the commit's own options are relative to it.
    pub start_directory: crate::start_directory::StartDirectory,
}
