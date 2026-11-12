//! What:
//!  The decision one wrapper invocation ends in.
//! Why:
//!  Forwarding replaces the process and cannot run inside a unit test;
//!  returning
//!      the decision as data lets every path be checked directly.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // type Action = { kind: 'forward'; ... } | { kind: 'exit'; code: number; stdout: string; stderr: string };
//! ```

/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   Environment additions are passed to Git without decoding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // [string, string][] environment pairs, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path of raw OS bytes.
use crate::index_writer_forward::CoordinatedForward;
use crate::worktree_copy_forward::CopyFacts;
use std::path::PathBuf;

/// What:
///  Exit code for a usage,
///  configuration or wrapper failure that kept Git from running.
///       `i32` is a signed 32-bit integer,
///  the type of process exit codes.
/// Why:
///   The wrapper's contract reserves 2 for these,
///  distinct from 1 for policy findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ENGINE_FAILURE_EXIT_CODE = 2;
/// ```
pub const ENGINE_FAILURE_EXIT_CODE: i32 = 2;

/// What:
///  Exit code for a command a policy refused.
/// Why:
///   The wrapper's contract reserves 1 for a policy finding and 2 for a wrapper failure;
///  a command stopped at the last moment because its copy could not be made is a finding about that command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REJECTED_EXIT_CODE = 1;
/// ```
pub const REJECTED_EXIT_CODE: i32 = 1;

/// What:
///  What a plain push that may be integrated needs to run and,
/// when it fails on a divergence,
/// to merge.
/// Why:
///   The run keeps the caller's streams like the settled one does,
/// so the integration step starts from Git's status
///  and the same arguments,
/// environment and (inside a linked worktree) settlement as an unstacked push would have had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PushForward = { realGit: string; args: string[]; overlay: [string, string][]; copy?: CopyFacts; subcommandIndex: number; environment: [string, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PushForward {
    /// The selected real Git executable.
    pub real_git: PathBuf,
    /// The arguments Git receives,
    /// global options included.
    pub arguments: Vec<OsString>,
    /// Variables added to the inherited environment.
    pub overlay: Vec<(OsString, OsString)>,
    /// The linked-worktree settlement the run keeps when one applies.
    pub copy: Option<Box<CopyFacts>>,
    /// Index of `push` in `arguments`.
    pub subcommand_index: usize,
    /// The invocation's environment,
    /// which the integration step starts Git with.
    pub environment: Vec<(OsString, OsString)>,
    /// Whether this push must upload LFS objects first,
    /// by a `git-lfs push` the parent runs before the push.
    ///  Only the push itself is preceded by the upload;
    /// the integration step's dry-run queries are not.
    pub lfs: bool,
    /// The configuration entries the upload plan reads,
    /// from the snapshot taken when the invocation was planned.
    pub lfs_settings: Option<Vec<(String, String)>>,
    /// Whether a push that fails on a divergence is integrated and pushed again.
    ///  A push that only needs the LFS upload explained keeps its own status.
    pub stack: bool,
}

/// What:
///  What the executable must do for one invocation.
///       `Vec<OsString>` is an owned argument list;
///  `Vec<(OsString, OsString)>` is an
///       owned list of name/value pairs;
///  `String` is owned UTF-8 text.
/// Why:
///   The executable performs exactly one of these:
///  become Git,
///  run Git and then add text,
///  or print and exit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Action = { kind: 'forward'; realGit: string; args: string[]; overlay: [string, string][]; stderr: string }
///   | { kind: 'exit'; code: number; stdout: string; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Become real Git.
    Forward {
        /// The selected real Git executable.
        real_git: PathBuf,
        /// The arguments Git receives:
        ///  no wrapper control,
        ///  fixed transforms applied.
        arguments: Vec<OsString>,
        /// Variables added to the inherited environment.
        overlay: Vec<(OsString, OsString)>,
        /// Complete warning event lines,
        ///  written to standard error before Git starts.
        stderr: String,
    },
    /// Forward an index writer under its landing lock:
    ///  coordinate,
    ///  run Git as a child,
    ///  release.
    ForwardCoordinated {
        /// What the coordinated run needs.
        forward: Box<CoordinatedForward>,
        /// Warning events written to standard error before the coordination starts.
        stderr: String,
    },
    /// Forward a command that the linked-worktree copy settles:
    ///  run Git as a child,
    ///  recover a copy a dead process left first,
    ///  copy the ignored state into the worktree the command created.
    ForwardSettled {
        /// The selected real Git executable.
        real_git: PathBuf,
        /// The arguments Git receives.
        arguments: Vec<OsString>,
        /// Variables added to the inherited environment.
        overlay: Vec<(OsString, OsString)>,
        /// What the copy needs to know about the invocation.
        copy: Box<CopyFacts>,
        /// Warning events written to standard error before Git starts.
        stderr: String,
    },
    /// Forward a plain push as a child,
    /// so a push that lost a race to another session can be integrated:
    ///  run Git with the caller's streams,
    /// then merge in what landed on the remote and publish the merge.
    ForwardPush {
        /// What the run and the integration need.
        forward: Box<PushForward>,
        /// Warning events written to standard error before Git starts.
        stderr: String,
    },
    /// Run Git as a child for a top-level `--version`,
    /// `-v`,
    /// `--help` or `-h`,
    ///  and print the announcement after Git's own answer when Git exited with status 0.
    ForwardAnnounced {
        /// The selected real Git executable.
        real_git: PathBuf,
        /// The arguments Git receives.
        arguments: Vec<OsString>,
        /// Variables added to the inherited environment.
        overlay: Vec<(OsString, OsString)>,
        /// Complete text for standard output after Git's answer,
        ///  already line-terminated.
        announcement: String,
        /// Warning events written to standard error before Git starts.
        stderr: String,
    },
    /// Stop without running the caller's command.
    Exit {
        /// Process exit code.
        code: i32,
        /// Complete text for standard output,
        ///  already line-terminated.
        stdout: String,
        /// Complete text for standard error,
        ///  already line-terminated.
        stderr: String,
    },
    /// Stop without running the caller's command,
    /// with bytes on standard output and standard error.
    ///  `Vec<u8>` is a report exactly as it is written.
    /// Why:
    ///   A report that names pathnames cannot spell them as text:
    /// a pathname is a byte string,
    /// and its text
    ///  spelling with replacement characters names a different file.
    /// The doctor's list of traditional hooks and the explanation of the Git LFS shims it found
    ///  are such reports,
    /// and the second one goes to standard error beside the first.
    ExitBytes {
        /// Process exit code.
        code: i32,
        /// Complete report for standard output,
        ///  already line-terminated.
        stdout: Vec<u8>,
        /// Complete report for standard error,
        ///  already line-terminated.
        stderr: Vec<u8>,
    },
}

/// What:
///  Put a notice in front of the standard-error text an action already carries.
///       `Option<&str>` is the notice or nothing (`string | undefined`).
/// Why:
///   The notice that the real Git came from the unit's record and not from `PATH` is owed to the person
///  before anything else the invocation says,
///  whichever way the invocation ends;
///  every variant carries its standard-error text,
///  so the one prefix covers them all
///  and no start of Git has to know about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withLeadingNotice(action: Action, notice?: string): Action { return notice ? { ...action, stderr: notice + action.stderr } : action; }
/// ```
pub fn with_leading_notice(action: Action, notice: Option<&str>) -> Action {
    // `let Some(x) = ... else { return ...; }` returns the action unchanged when there is no notice.
    let Some(lead) = notice else {
        return action;
    };
    // `match` picks the variant and binds the fields it carries;
    //  `stderr: text` binds the field under a new name, `..` leaves the other fields as they are.
    match action {
        Action::Forward { real_git, arguments, overlay, stderr } => {
            return Action::Forward { real_git, arguments, overlay, stderr: format!("{lead}{stderr}") };
        }
        Action::ForwardCoordinated { forward, stderr } => {
            return Action::ForwardCoordinated { forward, stderr: format!("{lead}{stderr}") };
        }
        Action::ForwardSettled { real_git, arguments, overlay, copy, stderr } => {
            return Action::ForwardSettled { real_git, arguments, overlay, copy, stderr: format!("{lead}{stderr}") };
        }
        Action::ForwardPush { forward, stderr } => {
            return Action::ForwardPush { forward, stderr: format!("{lead}{stderr}") };
        }
        Action::ForwardAnnounced { real_git, arguments, overlay, announcement, stderr } => {
            return Action::ForwardAnnounced {
                real_git,
                arguments,
                overlay,
                announcement,
                stderr: format!("{lead}{stderr}"),
            };
        }
        Action::Exit { code, stdout, stderr } => {
            return Action::Exit { code, stdout, stderr: format!("{lead}{stderr}") };
        }
        Action::ExitBytes { code, stdout, stderr } => {
            // The explanation's own bytes lead the report: a notice is text, and what follows it is not.
            let mut explained: Vec<u8> = lead.as_bytes().to_vec();
            explained.extend_from_slice(stderr.as_slice());
            return Action::ExitBytes { code, stdout, stderr: explained };
        }
    }
}

/// What:
///  Build the stop action for a wrapper failure described in prose.
///       `&str` borrows the message;
///  the action owns its formatted copy.
/// Why:
///   These failures (no real Git,
///  a forwarding loop,
///  an unreadable repository)
///       print one prefixed,
///  line-terminated diagnostic on standard error and exit 2.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failure(message: string): Action;
/// ```
pub fn failure(message: &str) -> Action {
    return Action::Exit {
        code: ENGINE_FAILURE_EXIT_CODE,
        // `String::new()` is empty owned text: nothing goes to standard output.
        stdout: String::new(),
        // `format!` builds the owned, line-terminated text.
        stderr: format!("monochromatic-git-policy-cli: {message}\n"),
    };
}
