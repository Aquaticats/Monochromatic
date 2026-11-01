//! What:
//!  Running a forwarded Git command through the linked-worktree ignored-state copy:
//!  which invocations the copy applies to, and the child-process run that carries the settlement lease.
//! Why:
//!  A command that creates or moves worktrees must stay under the settlement lock while real Git runs and the
//!  ignored state is copied, and a command run from a linked worktree must first recover an interrupted copy;
//!  replacing the process with Git would drop both duties, so these invocations run Git as a child.
//!  The main checkout is a source only for the configured rules file, so it takes this path only when the
//!  command creates a worktree and the configuration names that file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use super::forwarding::{ChildOutcome, exit_code as child_exit_code, run_real_git};
use super::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  Everything the copy needs to know about one forwarded invocation, decided before Git starts.
/// Why:
///   The plan decides that the copy applies; the run performs it with real Git and the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyForward = { identity: GitWorktreeIdentity; command: ResolvedGitCommand; mainCheckoutPaths: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyFacts {
    /// The repository shape the invocation selects.
    pub identity: WorktreeIdentity,
    /// Whether the command, after alias resolution, creates or moves worktrees.
    pub creates_or_moves: bool,
    /// Repository paths a main-checkout source hands over (the configured rules file).
    pub main_checkout_paths: Vec<String>,
    /// The invocation's global options, replayed before the repository queries that ask for the identity again.
    pub global_prefix: Vec<OsString>,
    /// The environment the invocation inherited, which may carry an outer settlement's lease.
    pub environment: Vec<(OsString, OsString)>,
}

/// What:
///  Whether the copy applies to an invocation.
/// Why:
///   Linked worktrees and bare repositories always take the lifecycle (recovery, lease check, creation);
///  the main checkout only when it creates a worktree and has paths to hand over, so its other commands
///  keep their plain forwarding and cost no filesystem reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind !== 'outside-worktree' && (identity.kind !== 'main-worktree' || (createsOrMoves && paths.length > 0))
/// ```
pub fn copy_applies(
    identity: &WorktreeIdentity,
    creates_or_moves: bool,
    main_checkout_paths: &[String],
) -> bool {
    // `match` on the repository shape; `||` keeps the main checkout's two conditions together.
    match identity {
        WorktreeIdentity::OutsideWorktree => return false,
        WorktreeIdentity::MainWorktree { .. } => {
            return creates_or_moves && !main_checkout_paths.is_empty();
        }
        WorktreeIdentity::LinkedWorktree { .. } | WorktreeIdentity::BareRepository { .. } => {
            return true;
        }
    }
}

/// What:
///  How one forwarded run ended.
/// Why:
///   Only a run that failed before or after Git needs its own text; otherwise the status is Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedRun = { exitCode: number } | { stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForwardedRun {
    /// Git ran (and the copy, when one applied, finished); this is the exit status.
    Ran(i32),
    /// The run could not start or settle; the standard error text and exit status.
    Failed {
        /// Text for standard error.
        stderr: String,
        /// Exit status.
        code: i32,
    },
}

/// What:
///  The directory the repository queries of the copy run in.
/// Why:
///   Every query names absolute paths, so any directory of the repository works; the worktree root exists whenever
///  the identity has a worktree, and a bare repository's common directory is itself a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.effectiveCwd
/// ```
#[cfg(unix)]
fn query_directory(identity: &WorktreeIdentity) -> PathBuf {
    // `match` on the repository shape: a worktree root, or the bare repository's directory.
    match identity {
        WorktreeIdentity::MainWorktree { worktree_root, .. }
        | WorktreeIdentity::LinkedWorktree { worktree_root, .. } => return worktree_root.clone(),
        WorktreeIdentity::BareRepository { common_dir, .. } => return common_dir.clone(),
        WorktreeIdentity::OutsideWorktree => return PathBuf::new(),
    }
}

/// What:
///  Runs real Git as a child with the given variables and reports how it ended.
/// Why:
///   One start for the plain and the settled runs, so both report an unstartable Git the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nanoSpawn(gitPath, args, { env, stdio: 'inherit' });
/// ```
fn run_plain(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> ForwardedRun {
    // `match` on the start result: the status is Git's, and an unstartable Git is an engine failure.
    match run_real_git(real_git, arguments, overlay) {
        Ok(outcome) => return ForwardedRun::Ran(child_exit_code(outcome)),
        Err(error) => {
            return ForwardedRun::Failed {
                stderr: format!("cli-git: cannot start {}: {error}\n", real_git.display()),
                code: super::action::ENGINE_FAILURE_EXIT_CODE,
            };
        }
    }
}

/// What:
///  Runs the forwarded Git command, settling the ignored-state copy around it when one applies.
/// Why:
///   Git's status is the answer unless the copy failed: a copy failure after a successful Git is status 2,
///  after a failed Git it keeps Git's status, and a failure of the lifecycle itself is status 2, each with its message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
pub fn run_forwarded_git(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    copy: Option<&CopyFacts>,
) -> ForwardedRun {
    #[cfg(unix)]
    {
        if let Some(facts) = copy {
            return run_settled(real_git, arguments, overlay, facts);
        }
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = copy;
    }
    return run_plain(real_git, arguments, overlay);
}

/// What:
///  The settled run: the lifecycle with real Git as its runner and the process's standard error as its stream.
/// Why:
///   The runner receives the lease token to put into the child's environment while the settlement lock is held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
/// ```
#[cfg(unix)]
fn run_settled(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    facts: &CopyFacts,
) -> ForwardedRun {
    use super::child_environment::environment_value;
    use std::io::Write;
    use super::repository_facts::{RepositoryFacts, git_facts};
    use super::transaction_git::GitContext;
    use super::worktree_copy::{GitStatus, Invocation, exit_code, run_with_worktree_copy};
    use super::worktree_copy_error::WorktreeCopyError;
    use super::worktree_copy_lock::WORKTREE_COPY_LEASE_ENV;

    let context: GitContext = GitContext {
        real_git: real_git.to_path_buf(),
        overlay: overlay.to_vec(),
        // Every copy query names absolute paths and leaves the caller's global options out.
        global_prefix: Vec::new(),
        // The copy queries send no input, so no scratch directory is needed.
        common_dir: None,
    };
    let invocation: Invocation = Invocation {
        identity: facts.identity.clone(),
        effective_cwd: query_directory(&facts.identity),
        creates_or_moves: facts.creates_or_moves,
        opted_out: false,
        short_circuit: false,
        inherited_token: environment_value(facts.environment.as_slice(), WORKTREE_COPY_LEASE_ENV)
            .map(|token| return token.to_string_lossy().into_owned()),
        context,
        main_checkout_paths: facts.main_checkout_paths.clone(),
    };
    let mut stderr = std::io::stderr();
    // The runner starts Git as a child with the lease token, so nested wrappers see the lock they are inside.
    let mut run_git = |token: Option<&str>| -> Result<GitStatus, WorktreeCopyError> {
        let mut child_overlay: Vec<(OsString, OsString)> = overlay.to_vec();
        if let Some(value) = token {
            child_overlay.push((OsString::from(WORKTREE_COPY_LEASE_ENV), OsString::from(value)));
        }
        match run_real_git(real_git, arguments, child_overlay.as_slice()) {
            Ok(ChildOutcome::Exited(code)) => return Ok(GitStatus { code: Some(code) }),
            Ok(ChildOutcome::Signaled(_)) => return Ok(GitStatus { code: None }),
            Err(error) => {
                return Err(WorktreeCopyError(format!(
                    "cli-git: cannot start {}: {error}",
                    real_git.display()
                )));
            }
        }
    };
    // The identity is asked again once the lock is held, as the incumbent re-observes its repository.
    let mut identity_after_lock = || -> Result<WorktreeIdentity, WorktreeCopyError> {
        let mut facts_again = git_facts(real_git, facts.global_prefix.as_slice(), overlay);
        match facts_again.location() {
            Ok(location) => return Ok(location.identity),
            Err(message) => return Err(WorktreeCopyError(format!("cli-git: {message}"))),
        }
    };
    match run_with_worktree_copy(&invocation, &mut run_git, &mut identity_after_lock, &mut stderr) {
        Ok(settlement) => {
            if let Some(failure) = &settlement.copy_failure {
                // A closed standard error leaves nowhere to report the failure, and the exit status still carries it.
                let _ = writeln!(stderr, "{}", failure.0);
            }
            return ForwardedRun::Ran(exit_code(&settlement));
        }
        Err(failure) => {
            return ForwardedRun::Failed {
                stderr: format!("{}\n", failure.0),
                code: super::action::ENGINE_FAILURE_EXIT_CODE,
            };
        }
    }
}

/// Forwarding-run controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_forward_tests.rs"]
mod tests;
