//! What:
//!  Runs a forwarded Git command under the index-writer coordination, finishes a settled forward, and writes to the
//!  standard streams, with the tip watch that hands a push over after a command that adds commits.
//! Why:
//!  `entry.rs` performs the action it planned; these are the steps of the coordinated and settled kinds, kept in a file
//!  that does not use `entry.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // runCoordinatedAction(forward): number; finishForwarded(run): number; writeStream(stream, text): void
//! ```

/// Import the sibling modules the decision combines.
use crate::action::ENGINE_FAILURE_EXIT_CODE;
#[cfg(unix)]
use crate::auto_push_trigger::{TipWatch, hand_over_if_ahead, try_watch_tip};
use crate::push_trigger_log::{Decision, Origin, Outcome, record_decision};
use crate::forwarding::replace_process_with_real_git;
use crate::index_writer_forward::{CoordinatedForward, CoordinatedRun, run_coordinated_forward};
use crate::forwarded_command::ResolvedCommand;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_copy_forward::{ForwardedRun, run_forwarded_git};
/// The trait that gives output streams `.write_all(..)`.
use std::io::Write;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::PathBuf;
/// What:
///  The exit code of a forwarded run, after printing the text of a run that failed.
/// Why:
///   A settled run keeps the wrapper alive around Git, so the exit code returns here instead of replacing the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.exitCode = outcome.exitCode;
/// ```
pub(crate) fn finish_forwarded(run: ForwardedRun) -> i32 {
    // `match` on the ending: a status, or a failure with its own text.
    match run {
        ForwardedRun::Ran(code) => return code,
        ForwardedRun::Failed { stderr, code } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            return code;
        }
    }
}

/// What:
///  Run a coordinated forward:
///  print its failure,
///  run Git under the landing lock,
///  or replace the process when no coordination applies after all.
/// Why:
///   The lock must outlive Git,
///  so only a run that takes no lock may replace the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await forwardToRealGit({ ... }); process.exitCode = outcome.exitCode;
/// ```
pub(crate) fn run_coordinated_action(forward: &CoordinatedForward) -> i32 {
    // A command that adds commits keeps the wrapper alive around Git, so the push can be handed over once it succeeded.
    let watch: Option<TipWatch> = if adds_commits(&forward.command) { watch_forward(forward) } else { None };
    let code: i32 = match run_coordinated_forward(forward) {
        CoordinatedRun::Ran(code) => code,
        CoordinatedRun::Failed { stderr, code } => {
            write_stream(&mut std::io::stderr(), stderr.as_str());
            code
        }
        CoordinatedRun::NotCoordinated if watch.is_some() => {
            finish_forwarded(run_forwarded_git(forward.real_git.as_path(), forward.arguments.as_slice(), forward.overlay.as_slice(), None))
        }
        CoordinatedRun::NotCoordinated => {
            let error: std::io::Error = replace_process_with_real_git(
                forward.real_git.as_path(),
                forward.arguments.as_slice(),
                forward.overlay.as_slice(),
            );
            write_stream(
                &mut std::io::stderr(),
                format!("cli-git: cannot start {}: {error}\n", forward.real_git.display()).as_str(),
            );
            ENGINE_FAILURE_EXIT_CODE
        }
    };
    if let Some(found) = watch.as_ref() {
        if code == 0 {
            hand_over_if_ahead(found, trigger_command(&forward.command).as_str(), &mut std::io::stderr());
        } else {
            // The watch holds the branch and its tip from before the command, which a reader needs to tell which merge failed.
            record_decision(&Decision {
                common_dir: Some(found.common_dir()),
                origin: Some(found.origin()),
                command: trigger_command(&forward.command).as_str(),
                outcome: Outcome::Declined,
                reason: "command-failed",
                detail: format!("Git exited with status {code}; {} was at {} before the command", found.branch_ref(), found.tip_before()).as_str(),
                branch: Some(found.branch_ref()),
            });
        }
    }
    return code;
}

/// Without file system readers for this platform, no push is handed over after a command that adds commits.
#[cfg(not(unix))]
struct TipWatch;

/// Without file system readers for this platform, no branch is watched.
#[cfg(not(unix))]
fn watch_forward(_forward: &CoordinatedForward) -> Option<TipWatch> {
    return None;
}

/// Without file system readers for this platform, nothing is handed over.
#[cfg(not(unix))]
fn hand_over_if_ahead(_watch: &TipWatch, _command: &str, _stderr: &mut dyn Write) {}

/// What:
///  Whether a resolved command adds commits to the branch that the owner wants pushed like a commit: a merge, a pull, a
///  cherry-pick, a revert or an applied patch.
/// Why:
///   A rebase is left out, because a rebased branch would need a forced push; the tip check then also leaves out the
///  rebase a pull runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function addsCommits(command: ResolvedGitCommand): boolean;
/// ```
fn adds_commits(command: &ResolvedCommand) -> bool {
    return command.is_builtin() && matches!(command.word(), b"merge" | b"pull" | b"cherry-pick" | b"revert" | b"am");
}

/// The command word a trigger record names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const word = command.word().toString();
/// ```
fn trigger_command(command: &ResolvedCommand) -> String {
    return String::from_utf8_lossy(command.word()).into_owned();
}

/// The branch watch of a coordinated forward, started in the worktree the command runs in.
/// A watch that cannot start leaves a record naming the step,
/// in the common Git directory the wrapper already resolved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function watchForward(forward: CoordinatedForward): TipWatch | undefined;
/// ```
#[cfg(unix)]
fn watch_forward(forward: &CoordinatedForward) -> Option<TipWatch> {
    let (root, common): (PathBuf, PathBuf) = match &forward.identity {
        WorktreeIdentity::MainWorktree { worktree_root, common_dir, .. } | WorktreeIdentity::LinkedWorktree { worktree_root, common_dir, .. } => {
            (worktree_root.clone(), common_dir.clone())
        }
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => return None,
    };
    return match try_watch_tip(root.as_path(), forward.real_git.as_path(), forward.overlay.as_slice(), forward.environment.as_slice()) {
        Ok(watch) => Some(watch),
        Err(declined) => {
            // The watch could not start, so no branch is known; the worktree is.
            let git_dir: PathBuf = match &forward.identity {
                WorktreeIdentity::MainWorktree { git_dir, .. } | WorktreeIdentity::LinkedWorktree { git_dir, .. } => git_dir.clone(),
                WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => common.clone(),
            };
            record_decision(&Decision {
                common_dir: Some(common.as_path()),
                origin: Some(Origin { worktree: root.as_path(), git_dir: git_dir.as_path() }),
                command: trigger_command(&forward.command).as_str(),
                outcome: Outcome::Declined,
                reason: declined.reason,
                detail: declined.detail.as_str(),
                branch: None,
            });
            None
        }
    };
}

/// What:
///  Write text to an output stream,
///  ignoring a closed stream.
///       `&mut dyn Write` lends "any writable stream" (`dyn` means the concrete type is
///       chosen at run time,
///  like a TS interface value).
/// Why:
///   The exit code already carries the result;
///  a reader that went away must not
///       turn a diagnostic into a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeStream(stream: Writable, text: string): void { try { stream.write(text); } catch {} }
/// ```
pub(crate) fn write_stream(stream: &mut dyn Write, text: &str) {
    // What: `let _ = ...` deliberately discards the write's `Result`.
    // Why:  There is nowhere left to report a failure to write a diagnostic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { stream.write(text); } catch { /* reader closed */ }
    // ```
    let _ = stream.write_all(text.as_bytes());
}
