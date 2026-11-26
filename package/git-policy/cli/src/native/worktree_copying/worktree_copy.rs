//! What:
//!  The copy's contract with whoever settles a forwarded invocation:
//!  the facts a settlement is handed,
//!  the steps it asks its Git runner for,
//!  the settlement it answers with,
//!  and the summary line and exit status a person reads from it.
//! Why:
//!  The integrator builds the invocation and supplies the runner,
//!  so both sides need these names without needing the code that acts on them.
//!  Git's status is the answer unless the copy failed:
//!  a copy-only failure is 2,
//!  and a Git that ended by a signal without a status is 1.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface GitRunner { run(step: GitStep): GitStatus }
//! type Settlement = { git: GitStatus; copyFailure?: WorktreeCopyError };
//! ```

use crate::forwarding::ChildStarted;
use crate::json_record::quote;
#[cfg(target_os = "linux")]
use crate::package_repair::RepairSwitch;
use crate::transaction_git::GitContext;
use crate::worktree_command_facts::AddPlan;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  How a real Git run ended.
/// Why:
///   Its numeric status is the exit status the wrapper keeps;
/// a signal has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedGitExecution = { failure?: SubprocessError };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GitStatus {
    /// The exit code,
    /// absent when Git ended by a signal.
    pub code: Option<i32>,
}

impl GitStatus {
    /// Whether Git exited with status zero.
    pub fn succeeded(&self) -> bool {
        return self.code == Some(0);
    }
}

/// What:
///  Everything the lifecycle needs to decide how one invocation is settled.
/// Why:
///   The decisions that depend on argv and global options are made by their owners and passed in as facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runGitWithWorktreeCopy({ args, gitPath, identity, command, environment })
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Invocation {
    /// The effective repository identity captured before real Git.
    pub identity: WorktreeIdentity,
    /// The working directory the repository queries run in.
    pub effective_cwd: PathBuf,
    /// Whether the command creates a worktree after alias resolution.
    pub creates: bool,
    /// The destination the creating command's line names,
    /// in the form Git records it;
    /// absent when the line has a form that is
    ///  not read,
    /// in which case no copy is made and the command says so.
    pub destination: Option<PathBuf>,
    /// Whether `--no-worktree-copy` appeared in flag position.
    pub opted_out: bool,
    /// Whether real Git only prints help or its version.
    pub short_circuit: bool,
    /// The real Git executable and the variables every Git child receives.
    pub context: GitContext,
    /// The destination of a `git worktree add` in a form the cheap-worktree ladder serves,
    /// when it is one.
    pub add_plan: Option<AddPlan>,
    /// Whether the person asked the add for `--quiet`,
    /// whether or not the ladder serves its form;
    /// an add the ladder does not serve still prints
    ///  no line a quiet add would suppress.
    pub quiet: bool,
    /// The person's `--quiet` and `--no-checkout`,
    /// read from the arguments,
    /// which decide what the package-manager repair of
    ///  a new worktree may print and whether it runs.
    #[cfg(target_os = "linux")]
    pub repair: RepairSwitch,
}

/// What:
///  The result of one settled invocation:
/// Git's status and the copy failure,
/// if any.
/// Why:
///   Both facts decide the exit status,
/// so they travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a resolved run, or a ForwardedGitWorktreeCopyError carrying both facts
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Settlement {
    /// How real Git ended.
    pub git: GitStatus,
    /// The ignored-state copy failure,
    /// when copying failed.
    pub copy_failure: Option<WorktreeCopyError>,
}

/// What:
///  Which Git run the settlement asks its runner to start.
/// Why:
///   The person's command is one run;
/// the cheap-worktree ladder adds `--no-checkout` to it and starts the commands that
///  complete the worktree itself,
/// each with the caller's streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GitStep = { kind: 'person' } | { kind: 'without-checkout'; insertAt: number } | { kind: 'other'; args: string[]; env: [string, string][] };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GitStep<'a> {
    /// The person's arguments,
    /// unchanged.
    Person,
    /// The person's arguments with `--no-checkout` inserted at this index.
    WithoutCheckout(usize),
    /// Another Git command with these arguments and these variables added.
    Other {
        /// The arguments,
        /// global options and subcommand included.
        arguments: &'a [std::ffi::OsString],
        /// The variables added to the run's environment.
        environment: &'a [(std::ffi::OsString, std::ffi::OsString)],
    },
}

/// What:
///  Runs one real Git step.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The ladder calls it once per Git step;
/// the state a step needs (the invocation,
/// the real Git,
/// the overlay,
/// the parallel-checkout
///  decision) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface GitRunner { run(step: GitStep): GitStatus } // throws WorktreeCopyError
/// ```
pub trait GitRunner {
    /// Run `step` and report how Git ended,
    /// naming the process number of the Git it started to `child_started` as soon as it has one,
    /// so the add's record can say which Git may still be at work if this command is killed.
    fn run(
        &mut self,
        step: &GitStep<'_>,
        child_started: &mut dyn ChildStarted,
    ) -> Result<GitStatus, WorktreeCopyError>;
}

/// What:
///  The one summary line a successful copy writes.
/// Why:
///   Users and agents read the count,
/// the source,
/// and the destination count from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderSummary({ copiedEntries, destinationCount, sourceRoot })
/// ```
pub fn summary_line(
    copied: usize,
    destinations: usize,
    source: Option<&Path>,
) -> String {
    let source_text: String = match source {
        None => String::from("bare repository with an empty source set"),
        Some(path) => quote(path.to_string_lossy().as_ref()),
    };
    let worktree_noun: &str = if destinations == 1 {
        "worktree"
    } else {
        "worktrees"
    };
    let entry_noun: &str = if copied == 1 { "entry" } else { "entries" };
    return format!(
        "monochromatic-git-policy-cli: copied {copied} ignored filesystem {entry_noun} from {source_text} into {destinations} new {worktree_noun}.\n"
    );
}

/// What:
///  The exit status of a settled invocation.
/// Why:
///   A copy-only failure is 2;
/// Git's own status is kept when Git failed;
/// a signal without a status is 1.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // copy failure 2, Git status kept, signal 1
/// ```
pub fn exit_code(settlement: &Settlement) -> i32 {
    let git_code: i32 = settlement.git.code.unwrap_or(1);
    // `match` on whether Git and the copy each succeeded.
    match (
        settlement.git.succeeded(),
        settlement.copy_failure.is_some(),
    ) {
        (true, false) => return 0,
        (false, false) => return git_code,
        (true, true) => return 2,
        (false, true) => return git_code,
    }
}

/// Lifecycle controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_lifecycle_tests.rs"]
mod tests;
