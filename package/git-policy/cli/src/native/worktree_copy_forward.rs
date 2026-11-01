//! What:
//!  Running a forwarded Git command through the linked-worktree ignored-state copy:
//!  which invocations the copy applies to, and the child-process run around which the copy is settled.
//! Why:
//!  A command that creates or moves worktrees must copy the ignored state into the worktrees real Git registered once
//!  it returns, and a command run from a linked worktree must first recover an interrupted copy;
//!  replacing the process with Git would drop both duties, so these invocations run Git as a child.
//!  The main checkout is a source only for the configured rules file, so it takes this path only when the
//!  command creates a worktree and the configuration names that file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runGitWithWorktreeCopy({ args, gitPath, identity, command, environment });
//! ```

use super::forwarding::{ChildOutcome, exit_code as child_exit_code, run_real_git};
use super::worktree_copy::AddedPath;
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
    /// The invocation's global options. The copy no longer asks for the identity again, so it does not read them.
    pub global_prefix: Vec<OsString>,
    /// The environment the invocation inherited. The copy reads none of it since the settlement lease was removed.
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
fn common_dir_of(identity: &WorktreeIdentity) -> Option<&Path> {
    // `match` on the repository shape: every shape but "outside" names its common directory.
    match identity {
        WorktreeIdentity::MainWorktree { common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { common_dir, .. }
        | WorktreeIdentity::BareRepository { common_dir, .. } => return Some(common_dir.as_path()),
        WorktreeIdentity::OutsideWorktree => return None,
    }
}

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

/// The long options of `git worktree add`, with the ones that take a value marked. Git also accepts each with a `no-` prefix.
const ADD_LONG_OPTIONS: [(&str, bool); 9] = [
    ("force", false),
    ("detach", false),
    ("checkout", false),
    ("lock", false),
    ("reason", true),
    ("quiet", false),
    ("guess-remote", false),
    ("track", false),
    ("relative-paths", false),
];

/// The options of `git worktree add` that Git's option parser takes by an unambiguous abbreviation, with `no-` forms.
fn long_option_takes_value(name: &str) -> Option<bool> {
    let mut candidates: Vec<(String, bool)> = Vec::new();
    for (option, takes_value) in ADD_LONG_OPTIONS {
        candidates.push((option.to_string(), takes_value));
        candidates.push((format!("no-{option}"), false));
    }
    candidates.push((String::from("orphan"), false));
    candidates.push((String::from("no-orphan"), false));
    if let Some((_, takes_value)) = candidates.iter().find(|(known, _)| return known == name) {
        return Some(*takes_value);
    }
    let abbreviated: Vec<&(String, bool)> = candidates.iter().filter(|(known, _)| return known.starts_with(name)).collect();
    return match abbreviated.as_slice() {
        [(_, takes_value)] => Some(*takes_value),
        _ => None,
    };
}

/// What:
///  Reads from the command line which worktree a plain `git worktree add` creates, the way Git's option parser reads it.
/// Why:
///   A worktree locked by `--lock` is the command's own to fill, and the command line is the only thing that names it.
///  Short options cluster (`-fb new-branch`), a long option may be abbreviated and may carry its value after `=`; a form
///  this reading does not know is `Unreadable`, which the copy reports instead of skipping the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// addedWorktree(args, cwd) // { kind: 'named', path } | { kind: 'not-add' } | { kind: 'unreadable' }
/// ```
pub fn added_worktree(arguments: &[OsString], cwd: &Path) -> AddedPath {
    const GLOBAL_WITH_VALUE: [&str; 6] = ["-c", "--git-dir", "--work-tree", "--namespace", "--super-prefix", "--config-env"];
    let mut directory: PathBuf = cwd.to_path_buf();
    let mut at: usize = 0;
    while let Some(argument) = arguments.get(at) {
        let Some(text) = argument.to_str() else { return AddedPath::Unreadable };
        if text == "-C" {
            let Some(value) = arguments.get(at + 1) else { return AddedPath::Unreadable };
            directory = directory.join(value);
            at += 2;
        } else if GLOBAL_WITH_VALUE.contains(&text) {
            at += 2;
        } else if text.starts_with('-') {
            at += 1;
        } else {
            break;
        }
    }
    let words: (Option<&str>, Option<&str>) =
        (arguments.get(at).and_then(|word| return word.to_str()), arguments.get(at + 1).and_then(|word| return word.to_str()));
    match words {
        (Some("worktree"), Some("add")) => {}
        (Some("worktree"), Some("move")) => return AddedPath::NotAdd,
        _ => return AddedPath::Unreadable,
    }
    at += 2;
    let mut options_end: bool = false;
    while let Some(argument) = arguments.get(at) {
        let Some(text) = argument.to_str() else { return AddedPath::Unreadable };
        at += 1;
        if options_end || !text.starts_with('-') || text == "-" {
            return AddedPath::Named(directory.join(argument));
        }
        if text == "--" {
            options_end = true;
        } else if let Some(long) = text.strip_prefix("--") {
            let (name, attached) = long.split_once('=').map_or((long, false), |(name, _)| return (name, true));
            match long_option_takes_value(name) {
                Some(true) if !attached => at += 1,
                Some(_) => {}
                None => return AddedPath::Unreadable,
            }
        } else {
            // A cluster of short options: `b` and `B` take the rest of the word as their value, or else the next word.
            let flags: &str = &text[1..];
            for (position, flag) in flags.char_indices() {
                match flag {
                    'b' | 'B' => {
                        if flags.len() == position + flag.len_utf8() {
                            at += 1;
                        }
                        break;
                    }
                    'f' | 'd' | 'q' => {}
                    _ => return AddedPath::Unreadable,
                }
            }
        }
    }
    return AddedPath::Unreadable;
}

#[cfg(unix)]
fn run_settled(
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
    facts: &CopyFacts,
) -> ForwardedRun {
    use std::io::Write;
    use super::transaction_git::GitContext;
    use super::worktree_copy::{CopyRequester, GitStatus, Invocation, exit_code, run_with_worktree_copy};
    use super::service_copy_client::ServiceCopyRequester;
    use super::worktree_copy_error::WorktreeCopyError;

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
        context,
        main_checkout_paths: facts.main_checkout_paths.clone(),
        added: std::env::current_dir().map_or(AddedPath::Unreadable, |cwd| return added_worktree(arguments, cwd.as_path())),
    };
    let mut requester: ServiceCopyRequester = ServiceCopyRequester::from_environment(real_git);
    let mut stderr = std::io::stderr();
    // The runner starts Git as a child; nothing it runs, hooks and filters included, waits on or is told about the copy.
    let mut run_git = || -> Result<GitStatus, WorktreeCopyError> {
        match run_real_git(real_git, arguments, overlay) {
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
    match run_with_worktree_copy(&invocation, &mut run_git, &mut requester, &mut stderr) {
        Ok(settlement) => {
            if let Some(failure) = &settlement.copy_failure {
                // A closed standard error leaves nowhere to report the failure, and the exit status still carries it.
                let _ = writeln!(stderr, "{}", failure.0);
                // Shown: the notice the service left in case this command was killed first is no longer needed.
                if let Some(common_dir) = common_dir_of(&facts.identity) {
                    requester.failures_shown(common_dir);
                }
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
