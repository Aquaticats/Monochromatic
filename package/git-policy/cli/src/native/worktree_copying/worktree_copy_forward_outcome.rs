//! What:
//!  How one forwarded run ended,
//!  the words a stopped step is named with,
//!  and the repository directories every query of this route reads.
//! Why:
//!  Each value here is a fact the runs of this folder share,
//!  so every one of them is written once at the bottom of the route.
//!  The plain run below starts real Git and reports the same ending a settled run does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type ForwardedRun = { kind: 'ran'; code: number } | { kind: 'killed'; signal: number } | { kind: 'failed'; stderr: string; code: number };
//! ```

use crate::forwarding::{
    ChildOutcome,
    run_real_git,
};
use crate::hooks_off::StartOwner;
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  How one forwarded run ended.
/// Why:
///   Only a run that failed before or after Git needs its own text;
/// otherwise the status is Git's,
/// and a Git that
///  was ended by a signal is this process's to end by (`Killed`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ForwardedRun = { exitCode: number } | { signal: number } | { stderr: string; exitCode: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ForwardedRun {
    /// Git ran (and the copy,
    /// when one applied,
    /// finished);
    /// this is the exit status.
    Ran(i32),
    /// Git was ended by this signal,
    /// so this process ends by it too;
    /// a wrapper cannot always die of the same signal,
    /// but [`crate::forwarding::end_as_git_ended`] does where it can.
    Killed(i32),
    /// The run could not start or settle;
    /// the standard error text and exit status.
    Failed {
        /// Text for standard error.
        stderr: String,
        /// Exit status.
        code: i32,
    },
}

/// What:
///  The repository's common Git directory,
///  which owns the registrations a sweep reads and the record of a worktree this program made.
/// Why:
///   Both jobs write beside the registrations,
///  and a bare repository has a common directory too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.commonDir
/// ```
#[cfg(unix)]
pub(super) fn common_directory(identity: &WorktreeIdentity) -> Option<&Path> {
    // `match` on the repository shape: only a place outside a repository has none.
    match identity {
        WorktreeIdentity::MainWorktree {
            common_dir,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            common_dir,
            ..
        }
        | WorktreeIdentity::BareRepository {
            common_dir,
            ..
        } => return Some(common_dir.as_path()),
        WorktreeIdentity::OutsideWorktree => return None,
    }
}

/// What:
///  The directory the repository queries of the copy run in.
/// Why:
///   Every query names absolute paths,
/// so any directory of the repository works;
/// the worktree root exists whenever
///  the identity has a worktree,
/// and a bare repository's common directory is itself a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.effectiveCwd
/// ```
#[cfg(unix)]
pub(super) fn query_directory(identity: &WorktreeIdentity) -> PathBuf {
    // `match` on the repository shape: a worktree root, or the bare repository's directory.
    match identity {
        WorktreeIdentity::MainWorktree {
            worktree_root,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            worktree_root,
            ..
        } => return worktree_root.clone(),
        WorktreeIdentity::BareRepository {
            common_dir,
            ..
        } => return common_dir.clone(),
        WorktreeIdentity::OutsideWorktree => return PathBuf::new(),
    }
}

/// What:
///  The words of a step as one line for a message,
/// each quoted the way a shell reads it.
/// Why:
///   A stopped step is named by the command it ran,
/// so a person can run it again,
/// and a word holding
///  a space,
/// a `$` or a quote has to reach their shell as one word and no expansion.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// arguments.map((word) => shellQuote(word)).join(' ')
/// ```
pub(super) fn words_of(arguments: &[OsString]) -> String {
    let mut text: String = String::new();
    for (index, word) in arguments.iter().enumerate() {
        if index > 0 {
            text.push(' ');
        }
        text.push_str(crate::shell_quote::shell_quote(word.as_bytes()).as_str());
    }
    return text;
}

/// What:
///  The plain start of real Git,
/// with the report every run of this route gives.
/// Why:
///   It is the one place real Git is started for a run that copies nothing,
/// and `run_plain_of`'s two callers differ only in whose Git it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await nanoSpawn(gitPath, args, { env, stdio: 'inherit' });
/// ```
pub(super) fn run_plain_of(
    owner: StartOwner,
    real_git: &Path,
    arguments: &[OsString],
    overlay: &[(OsString, OsString)],
) -> ForwardedRun {
    // What: A typed network command whose remote ssh carries relays Git's error stream and names the next step after a refusal.
    // Why:  Every child this route starts reaches Git here, the fetch a merging `pull` runs for the wrapper's own work included,
    //       and a command ssh does not carry keeps the inherited streams it always had.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const relayed = relayWhenSshCarries(git, argv, env, owner); if (relayed?.value) return forwardedRun(relayed.value);
    // ```
    #[cfg(unix)]
    match crate::typed_network_command_relay::relay_when_ssh_carries(
        real_git, arguments, overlay, owner,
    ) {
        Some(Ok(ending)) => {
            return match ending {
                crate::typed_network_command_relay::TypedNetworkCommandEnding::Ran(code) => {
                    ForwardedRun::Ran(code)
                }
                crate::typed_network_command_relay::TypedNetworkCommandEnding::Signalled(
                    signal,
                ) => ForwardedRun::Killed(signal),
            };
        }
        Some(Err(error)) => {
            return ForwardedRun::Failed {
                stderr: format!(
                    "monochromatic-git-policy-cli: cannot start {}: {error}\n",
                    real_git.display()
                ),
                code: crate::action::ENGINE_FAILURE_EXIT_CODE,
            };
        }
        None => {}
    }
    // `match` on the start result: the status is Git's, the death by a signal is Git's own too, and an unstartable Git
    //  is an engine failure.
    match run_real_git(real_git, arguments, overlay, owner) {
        Ok(ChildOutcome::Exited(code)) => return ForwardedRun::Ran(code),
        // What: A Git that a signal ended stays a death by that signal.
        // Why:  This route keeps a process around Git, so the death comes back as a value; flattening it into 128 + the
        //       signal made the caller see a normal exit where native Git's caller sees `WIFSIGNALED`
        //       (`doc/planning/cli-git-entry-and-service-lifecycle-route-findings-on-main.md`, finding 3 of the round-2 review).
        Ok(ChildOutcome::Signaled(signal)) => return ForwardedRun::Killed(signal),
        Err(error) => {
            return ForwardedRun::Failed {
                stderr: format!(
                    "monochromatic-git-policy-cli: cannot start {}: {error}\n",
                    real_git.display()
                ),
                code: crate::action::ENGINE_FAILURE_EXIT_CODE,
            };
        }
    }
}
