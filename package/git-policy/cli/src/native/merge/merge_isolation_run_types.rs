//! What:
//!  The values an isolated merge passes between its steps:
//! the input,
//! the outcome,
//! the way a step gives up,
//! the settings read from Git,
//! the plan,
//! and the constants of the run.
//! Why:
//!  `merge_isolation.rs` runs the merge and stays under the file-size limit;
//! its steps and these values are shared by several files,
//! so the values live in a file that imports none of them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type IsolatedMerge = { kind: "forward" } | { kind: "done"; code: number; stdout: string; stderr: string };
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The person's own work in the paths the merge changes.
use crate::merge_isolation_carry_parts::Capture;
/// The resolved command.
use crate::forwarded_command::ResolvedCommand;
/// The command-line reading.
use crate::merge_options::MergeShape;
/// The repository shape.
use crate::worktree_identity::WorktreeIdentity;
/// Owned operating-system text.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::Path;

/// Module tag for diagnostics.
pub const TAG: &str = "isolatedMerge";

/// Files and directories of the Git directory that show another operation is in progress;
/// native Git reports those
/// itself,
/// so the command is forwarded.
pub const IN_PROGRESS_MARKERS: [&str; 6] = [
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "rebase-merge",
    "rebase-apply",
    "sequencer",
];

/// Everything one isolated merge reads from its invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IsolatedMergeInput = { realGit; overlay; identity; command; environment };
/// ```
pub struct IsolatedMergeInput<'a> {
    /// Real Git executable.
    pub real_git: &'a Path,
    /// Variables added to every Git child.
    pub overlay: &'a [(OsString, OsString)],
    /// The repository shape of the invocation.
    pub identity: &'a WorktreeIdentity,
    /// The merge command after alias resolution.
    pub command: &'a ResolvedCommand,
    /// The environment the invocation inherited.
    pub environment: &'a [(OsString, OsString)],
}

/// How an isolated merge attempt ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IsolatedMerge = { kind: 'forward'; notice?: string } | { kind: 'finished'; code: number; stdout: string; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IsolatedMerge {
    /// The command is not for the wrapper to complete:
    /// native Git runs it unchanged,
    /// after the notice if any.
    Forward {
        /// A line for standard error before Git runs.
        notice: Option<String>,
    },
    /// The wrapper completed or refused the merge.
    Finished {
        /// Exit status.
        code: i32,
        /// Text for standard output.
        stdout: String,
        /// Text for standard error.
        stderr: String,
    },
}

/// Why the attempt stopped before it finished.
pub enum Bail {
    /// Forward to native Git.
    Forward(Option<String>),
    /// Done,
    /// with this result.
    Done(IsolatedMerge),
    /// The checkout is as it was before the round and the merge is computed again,
    /// for this reason (the branch moved,
    /// or
    ///  another program took the index lock in the instant before Git did).
    Again(String),
}

/// How many times one merge is computed again on a branch that moved underneath it before the wrapper stops.
/// Each round
/// is a whole merge on the newest head;
/// a branch that moves faster than that for this many rounds in a row is not a
/// race but a loop,
/// and the merge is refused with the cause named,
/// changing nothing.
pub const MAX_ROUNDS: usize = 20;

/// A failed query is a reason to forward,
/// with the cause in the diagnostic log.
impl From<String> for Bail {
    fn from(reason: String) -> Self {
        debug(TAG, format!("forwarding to native Git: {reason}").as_str());
        return Bail::Forward(None);
    }
}

/// The merge-related settings the isolated path honors or defers on.
#[derive(Debug, Default)]
pub struct MergeConfig {
    /// `merge.ff`:
    /// `Some(false)` is `--no-ff`;
    /// `Some(true)` allows;
    /// absent is unset.
    pub fast_forward: Option<bool>,
    /// `merge.ff=only`.
    pub fast_forward_only: bool,
    /// `merge.stat`:
    /// false hides the diffstat.
    pub stat: bool,
    /// `commit.gpgSign`.
    pub sign_commits: bool,
    /// Any setting that changes the merge in a way this path does not reproduce.
    pub deferred: bool,
}

/// What the apply step needs to know about the merge it applies.
pub struct Plan<'a> {
    /// The command line.
    pub shape: &'a MergeShape,
    /// The settings read for the merge.
    pub config: &'a MergeConfig,
    /// The worktree's Git directory.
    pub git_dir: &'a Path,
    /// The worktree top level.
    pub root: &'a Path,
    /// The commit the branch holds before the merge.
    pub old_oid: &'a str,
    /// The merge commit.
    pub new_oid: &'a str,
    /// The person's own work in the paths the merge changes,
    /// kept outside the checkout before the first of them is written,
    /// or nothing when those paths held none of it.
    pub carried: Option<&'a Capture>,
}

/// The text a finding or failure puts on standard error,
/// as a finished merge.
pub fn refused(code: i32, stderr: String) -> Bail {
    return Bail::Done(IsolatedMerge::Finished { code, stdout: String::new(), stderr });
}
