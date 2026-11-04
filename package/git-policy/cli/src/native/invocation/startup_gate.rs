//! What:
//!  The startup step of a wrapped, read-only or direct command: settle an isolated merge whose process died, stop on a
//!  commit journal of the previous executable's format, and mention commit records an earlier release left behind.
//! Why:
//!  A commit is landed by the resident service, which recovers its own journal, so no commit transaction exists to
//!  recover. What stays is state other code still writes (the intent record of an isolated merge) and state an earlier
//!  release wrote (the single-journal directory and the registry of commit transactions): the first is settled before
//!  anything reads the checkout, the others are named to the person, because nothing here reads them and an interrupted
//!  commit of that release is for the person to look at. An empty registry costs one directory read and no Git process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleInterruptedMerges({ identity }); // throws the stop
//! ```

use crate::diagnostic_log::debug;
use crate::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
use crate::merge_isolation_recovery::{RecoveryPlace, merge_intents_to_recover, recover_interrupted_merges};
use crate::random_id::is_canonical_uuid;
use crate::recovery_error::RecoveryError;
use crate::recovery_files::recovery_path_exists;
use crate::transaction_git::GitContext;
use crate::worktree_identity::WorktreeIdentity;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The directory an earlier release kept its commit transactions in, inside the worktree's Git directory.
pub const REGISTRY_DIRECTORY_NAME: &str = "cli-git-transactions";

/// The directory the previous executable kept one commit journal in, inside the worktree's Git directory.
pub const LEGACY_JOURNAL_DIRECTORY_NAME: &str = "cli-git-transaction";

/// What:
///  How a command may go on after the startup step.
/// Why:
///   Callers continue or stop from this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Gate = { kind: 'clear' } | { kind: 'stop'; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Gate {
    /// The command goes on.
    Clear,
    /// The command stops; this is its standard error.
    Stop(String),
}

/// What:
///  The message for a commit journal of the previous executable's format.
/// Why:
///   The owner chose to stop with instructions instead of porting that recovery
///  (`doc/planning/cli-git-rust-open-decisions.md`, "Legacy single-journal directory").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git: ${directory} holds a commit journal in a format this executable does not recover. ...`
/// ```
pub fn legacy_notice(directory: &Path) -> String {
    return format!(
        "cli-git: {} holds a commit journal in a format this executable does not recover. \
         Nothing was changed. Run any git command once with the previous cli-git executable to \
         recover it, then run this command again.\n",
        directory.display()
    );
}

/// What:
///  The path as one shell word that a copied command line reads back as exactly that path.
/// Why:
///   The note below gives a command to paste, and a Git directory can sit under a name with spaces, quotes or bytes that are
///  not UTF-8; a word cut by one of them would remove a different path. Text goes in single quotes, where only the quote
///  itself needs the `'\''` spelling; bytes that are not UTF-8 go in the `$'...'` form that bash, zsh and ksh read, with
///  octal escapes for every byte outside plain printable ASCII.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `'${path.replaceAll("'", "'\\''")}'`
/// ```
pub fn shell_word(path: &Path) -> String {
    if let Some(text) = path.to_str() {
        return format!("'{}'", text.replace('\'', "'\\''"));
    }
    #[cfg(unix)]
    {
        /// The Unix view of a path as raw bytes.
        use std::os::unix::ffi::OsStrExt;
        let mut word: String = String::from("$'");
        for byte in path.as_os_str().as_bytes() {
            match *byte {
                b'\\' => word.push_str("\\\\"),
                b'\'' => word.push_str("\\'"),
                0x20..=0x7e => word.push(char::from(*byte)),
                other => word.push_str(format!("\\{other:03o}").as_str()),
            }
        }
        word.push('\'');
        return word;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"));
    }
}

/// What:
///  The note about commit records an earlier release left in a registry directory, or nothing when it holds no transaction.
/// Why:
///   A transaction directory there is a commit that release did not finish. This release does not read it or use the
///  directory for anything, so the person is told where it is, how to see whether the commit was made, and the exact command
///  that removes the directory; lock directories and staging names carry nothing to look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(registry)).some(isTransactionId) ? note(registry) : undefined
/// ```
pub fn registry_note(registry: &Path) -> Option<String> {
    let entries: std::fs::ReadDir = std::fs::read_dir(registry).ok()?;
    let left: bool = entries.flatten().any(|entry| { return is_canonical_uuid(entry.file_name().to_string_lossy().as_ref()); });
    if !left {
        return None;
    }
    return Some(format!(
        "cli-git: {} holds commit records an earlier release left unfinished. This release no longer uses that directory and \
         does not read it. Run `git log -1` and `git status` to see whether that commit was made. Once you have, remove the \
         directory with: rm --recursive --force -- {}\n",
        registry.display(),
        shell_word(registry)
    ));
}

/// What:
///  The Git directory of the worktree an identity names, or nothing outside a worktree and in a bare repository.
/// Why:
///   Only worktrees hold merge records and commit records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind === 'outside-worktree' || identity.kind === 'bare-repository' ? undefined : [identity.gitDir, identity.commonDir]
/// ```
fn worktree_directories(identity: &WorktreeIdentity) -> Option<(&Path, &Path)> {
    match identity {
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => return None,
        WorktreeIdentity::MainWorktree { git_dir, common_dir, .. } | WorktreeIdentity::LinkedWorktree { git_dir, common_dir, .. } => {
            return Some((git_dir.as_path(), common_dir.as_path()));
        }
    }
}

/// What:
///  Settle the isolated merges of the worktree whose process died.
/// Why:
///   A merge killed between Git's checkout update and the branch move leaves the checkout half done, and the next command
///  would read it as another session's staged work. A record of a merge that still runs is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (await mergeIntentsToRecover(gitDir)) await recoverInterruptedMerges({ place, gitDir });
/// ```
fn settle_merges(context: &GitContext, git_dir: &Path, common_dir: &Path) -> Result<(), RecoveryError> {
    if !merge_intents_to_recover(git_dir)? {
        return Ok(());
    }
    let place: RecoveryPlace = RecoveryPlace {
        git: context.clone(),
        cwd: PathBuf::from("."),
        common_dir: common_dir.to_path_buf(),
    };
    let outcomes = recover_interrupted_merges(&place, git_dir)?;
    if !outcomes.is_empty() {
        debug("startupGate", format!("isolated merge recovery outcomes: {outcomes:?}").as_str());
    }
    return Ok(());
}

/// What:
///  The startup step for the invocation's worktree, and how the command may go on.
/// Why:
///   Shared by wrapped commands, read-only commands and direct commands. A recovery failure stops the command with exit 2,
///  as the incumbent's `content-unavailable` engine failure does; the note about an earlier release's records goes to
///  standard error and does not stop it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverForCommand({ gitPath, identity }): Promise<Gate>;
/// ```
pub fn recover_for_command(context: &GitContext, identity: &WorktreeIdentity) -> Gate {
    let Some((git_dir, common_dir)) = worktree_directories(identity) else {
        return Gate::Clear;
    };
    let legacy: PathBuf = git_dir.join(LEGACY_JOURNAL_DIRECTORY_NAME);
    match recovery_path_exists(legacy.as_path()) {
        Ok(true) => return Gate::Stop(legacy_notice(legacy.as_path())),
        Ok(false) => {}
        Err(error) => return stopped(&error),
    }
    if let Err(error) = settle_merges(context, git_dir, common_dir) {
        return stopped(&error);
    }
    if let Some(note) = registry_note(git_dir.join(REGISTRY_DIRECTORY_NAME).as_path()) {
        let _noted = std::io::stderr().write_all(note.as_bytes());
    }
    return Gate::Clear;
}

/// What:
///  The stop for a failed read or recovery.
/// Why:
///   The code stays `content-unavailable` (state that could not be read), but the generic hint for that code tells the person
///  to fetch history, which does not apply: the message itself names the file or directory and the ways out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renderEngineFailure({ code: 'content-unavailable', message, hint: undefined })
/// ```
fn stopped(error: &RecoveryError) -> Gate {
    return Gate::Stop(render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.0.as_str(), None));
}

/// Startup step controls stay out of the release executable.
#[cfg(test)]
#[path = "startup_gate_tests.rs"]
mod tests;
