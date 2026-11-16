//! What:
//!  The startup step of a wrapped,
//! read-only or direct command:
//! settle an isolated merge whose process died,
//! stop on a
//!  commit journal of the previous executable's format,
//! and mention commit records an earlier release left behind.
//! Why:
//!  A commit is landed by the resident service,
//! which recovers its own journal,
//! so no commit transaction exists to
//!  recover.
//! What stays is state other code still writes (the intent record of an isolated merge) and state an earlier
//!  release wrote (the single-journal directory and the registry of commit transactions):
//! the first is settled before
//!  anything reads the checkout,
//! the others are named to the person,
//! because nothing here reads them and an interrupted
//!  commit of that release is for the person to look at.
//! An empty registry costs one directory read and no Git process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleInterruptedMerges({ identity }); // throws the stop
//! ```

use crate::diagnostic_log::debug;
use crate::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
use crate::merge_isolation_kept_copies::any_kept_copy;
use crate::merge_isolation_recovery::{RecoveryPlace, merge_intents_to_recover, recover_interrupted_merges};
/// Whether a branch move's record exists at all, which is the one directory read a command with none pays.
use crate::auto_push_branch_move_record::branch_moves_exist;
/// The command that finishes the checkout a stopped move left, and what it answers about this one.
use crate::auto_push_branch_move_repairs::finish_branch_moves;
/// The Git context a push's own starts read.
use crate::auto_push_single_flight::PushGit;
use crate::random_id::is_canonical_uuid;
use crate::repository_link_gate::{LinkGate, link_gate};
use crate::child_environment::environment_home;
use crate::recovery_error::RecoveryError;
use crate::recovery_files::recovery_path_exists;
use crate::transaction_git::GitContext;
use crate::worktree_identity::WorktreeIdentity;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The directory an earlier release kept its commit transactions in,
/// inside the worktree's Git directory.
pub const REGISTRY_DIRECTORY_NAME: &str = "cli-git-transactions";

/// The directory the previous executable kept one commit journal in,
/// inside the worktree's Git directory.
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
    /// The command stops;
    /// this is its standard error.
    Stop(String),
    /// The command stops because the repository it names is not reachable (the link to the 860 drive dangles,
    /// or is not the recorded one);
    /// this is its standard error,
    /// and the status is Git's fatal status,
    /// where `Stop` ends with the engine failure status.
    RepositoryUnreachable(String),
}

/// What:
///  The message for a commit journal of the previous executable's format.
/// Why:
///   The owner chose to stop with instructions instead of porting that recovery
///  (`doc/planning/cli-git-rust-open-decisions.md`,
/// "Legacy single-journal directory").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${directory} holds a commit journal in a format this executable does not recover. ...`
/// ```
pub fn legacy_notice(directory: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} holds a commit journal in a format this executable does not recover. \
         Nothing was changed. Run any git command once with the previous monochromatic-git-policy-cli executable to \
         recover it, then run this command again.\n",
        directory.display()
    );
}

/// What:
///  The path as one shell word that a copied command line reads back as exactly that path.
/// Why:
///   The note below gives a command to paste,
/// and a Git directory can sit under a name with spaces,
/// quotes or bytes that are
///  not UTF-8;
/// a word cut by one of them would remove a different path.
/// Text goes in single quotes,
/// where only the quote
///  itself needs the `'\''` spelling;
/// bytes that are not UTF-8 go in the `$'...'` form that bash,
/// zsh and ksh read,
/// with
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
///  The note about commit records an earlier release left in a registry directory,
/// or nothing when it holds no transaction.
/// Why:
///   A transaction directory there is a commit that release did not finish.
/// This release does not read it or use the
///  directory for anything,
/// so the person is told where it is,
/// how to see whether the commit was made,
/// and the exact command
///  that removes the directory;
/// lock directories and staging names carry nothing to look at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(registry)).some(isTransactionId) ? note(registry) : undefined
/// ```
pub fn registry_note(registry: &Path) -> Option<String> {
    let entries: std::fs::ReadDir = std::fs::read_dir(registry).ok()?;
    // The first leftover record decides it; an entry that cannot be read is skipped by `flatten`.
    let mut left: bool = false;
    for entry in entries.flatten() {
        if is_canonical_uuid(entry.file_name().to_string_lossy().as_ref()) {
            left = true;
            break;
        }
    }
    if !left {
        return None;
    }
    return Some(format!(
        "monochromatic-git-policy-cli: {} holds commit records an earlier release left unfinished. This release no longer uses that directory and \
         does not read it. Run `git log -1` and `git status` to see whether that commit was made. Once you have, remove the \
         directory with: rm --recursive --force -- {}\n",
        registry.display(),
        shell_word(registry)
    ));
}

/// What:
///  The Git directory of the worktree an identity names,
/// or nothing outside a worktree and in a bare repository.
/// Why:
///   Only worktrees hold merge records and commit records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind === 'outside-worktree' || identity.kind === 'bare-repository' ? undefined : [identity.gitDir, identity.commonDir]
/// ```
fn worktree_directories(identity: &WorktreeIdentity) -> Option<(&Path, &Path, &Path)> {
    match identity {
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => return None,
        WorktreeIdentity::MainWorktree { git_dir, common_dir, worktree_root }
        | WorktreeIdentity::LinkedWorktree { git_dir, common_dir, worktree_root } => {
            return Some((git_dir.as_path(), common_dir.as_path(), worktree_root.as_path()));
        }
    }
}

/// What:
///  Settle the isolated merges of the worktree whose process died.
/// Why:
///   A merge killed between Git's checkout update and the branch move leaves the checkout half done,
/// and the next command
///  would read it as another session's staged work.
/// A record of a merge that still runs is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (await mergeIntentsToRecover(gitDir)) await recoverInterruptedMerges({ place, gitDir, stderr });
/// ```
fn settle_merges(
    context: &GitContext,
    git_dir: &Path,
    common_dir: &Path,
    worktree_root: &Path,
    environment: &[(OsString, OsString)],
    out: &mut dyn Write,
) -> Result<(), RecoveryError> {
    // A copy a merge kept beside a path is decided here too: it is the one thing a worktree with no merge record still has to
    //  look at, and a copy no later merge touches would otherwise be a file no command ever ends.
    // A repository that moved no branch without its checkout pays one failed directory read for this too.
    let moves_to_finish: bool = branch_moves_exist(common_dir);
    if !merge_intents_to_recover(git_dir)? && !any_kept_copy(git_dir) && !moves_to_finish {
        return Ok(());
    }
    // What: finish the branch moves a stopped process left, in the worktree this command runs in.
    // Why: a stop after a branch's reference moved and before its checkout was repaired leaves a checkout whose next
    //      commit records a reversal of the published merge, and the record is what the next command finishes
    //      (a review of commit `36cae084ae`, its finding 3).
    if moves_to_finish {
        // What: the sweep's own Git starts run in the worktree the identity names, never in the process's directory.
        // Why: the repair reads and writes that worktree's index and files, and a command started elsewhere writes
        // another repository's.
        // What: the sweep's Git runs with the caller's environment plus the overlay, which is what every other
        // command's Git here gets.
        // Why: the overlay alone names no home, so the sweep's own queries could not make the scratch file they need
        //      and every command stopped before running
        //      (a review of commit `986a82bbab`, its finding 4).
        let git: PushGit = PushGit {
            real_git: context.real_git.clone(),
            cwd: worktree_root.to_path_buf(),
            environment: crate::child_environment::environment_with_overlay(environment, context.overlay.as_slice()),
            lfs: false,
        };
        // What: a command is cleared only when no path of a move this checkout would commit over still holds
        // what the branch moved from.
        // Why: the sweep's success used to mean only that it had run, and a commit admitted over a stale
        // checkout records the tree the record exists to prevent; the gate then cleared a commit after
        // naming exactly the paths that would be recorded
        //  (a review of commit `777c491325`, its finding 3, and commit `4ca7aebcdb`, its finding 2).
        // The sweep brings every path that still holds the moved-from tree forward by itself, so an answer
        // that names such a path means the write could not be made: another program holds the index lock, or
        // another command is repairing it. That command waits for the other one, bounded, and then stops the
        // command with the paths named rather than clearing it over them.
        let mut waited: std::time::Duration = std::time::Duration::ZERO;
        loop {
            match finish_branch_moves(&git, common_dir, out) {
                Ok(crate::auto_push_branch_move_repairs::Settled::Yes) => break,
                Ok(crate::auto_push_branch_move_repairs::Settled::OldTipLeft { branch, paths }) => {
                    return Err(RecoveryError(format!(
                        "this checkout still holds {branch}'s files from before its move on {}, and this command could not write the branch's \
                         own files over them, so it stops rather than let a commit record them. The record of that move is kept. Nothing was \
                         written: run `git status` there and decide what those files should hold, then run the command again.",
                        crate::auto_push_dirty_paths::listed(&paths)
                    )));
                }
                Ok(crate::auto_push_branch_move_repairs::Settled::Unknown { why }) => {
                    return Err(RecoveryError(format!(
                        "the checkout this command runs in could not be checked against a branch move ({why}), so the command stops rather \
                         than let a commit record files of unknown age. The record of that move is kept."
                    )));
                }
                Ok(crate::auto_push_branch_move_repairs::Settled::AnotherCommandAtWork)
                    if waited >= crate::auto_push_branch_move_repairs::REPAIR_WAIT =>
                {
                    return Err(RecoveryError(format!(
                        "another command has been finishing this checkout for {waited:?}, so this command stops rather than let a commit \
                         record files of unknown age. The record of that move is kept, and a command run once that repair has ended \
                         finishes it."
                    )));
                }
                Ok(crate::auto_push_branch_move_repairs::Settled::AnotherCommandAtWork) => {
                    std::thread::sleep(crate::auto_push_branch_move_repairs::REPAIR_PAUSE);
                    waited += crate::auto_push_branch_move_repairs::REPAIR_PAUSE;
                }
                Err(error) => return Err(RecoveryError(error)),
            }
        }
    }
    let place: RecoveryPlace = RecoveryPlace {
        git: context.clone(),
        cwd: PathBuf::from("."),
        common_dir: common_dir.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
    };
    // The notices of what recovery did go to the command's standard error, so a person who ran a read-only command
    //  learns that it finished a merge (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`, finding 1).
    let outcomes = recover_interrupted_merges(&place, git_dir, out)?;
    if !outcomes.is_empty() {
        debug("startupGate", format!("isolated merge recovery outcomes: {outcomes:?}").as_str());
    }
    return Ok(());
}

/// What:
///  The startup step for the invocation's worktree,
/// and how the command may go on.
/// Why:
///   Shared by wrapped commands,
/// read-only commands and direct commands.
/// The home the link gate reads is the one this invocation's environment names,
/// never this process's,
/// so the entry gate and this step judge the same command by the same identity file.
/// A recovery failure stops the command with exit 2,
///  as the incumbent's `content-unavailable` engine failure does;
/// the note about an earlier release's records goes to
///  standard error and does not stop it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverForCommand({ gitPath, identity, environment }): Promise<Gate>;
/// ```
pub fn recover_for_command(context: &GitContext, identity: &WorktreeIdentity, environment: &[(OsString, OsString)]) -> Gate {
    let home: Option<PathBuf> = environment_home(environment);
    return recover_for_command_in_environment(context, identity, home.as_deref(), environment);
}

/// What:
///  The startup step with the home directory and the caller's environment named.
/// Why:
///   The sweep's own Git starts need the caller's ordinary environment as much as any other command's:
/// with the overlay alone there is no `HOME` and no absolute `XDG_STATE_HOME`, so the query that reads the
/// branch's tip cannot make its own scratch file, and every ordinary command stopped with
///  "the tip of refs/heads/topic could not be read" while the record it was meant to finish sat there
///  (a review of commit `986a82bbab`, its finding 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recoverForCommandInEnvironment({ context, identity, home, environment })
/// ```
pub fn recover_for_command_in_environment(
    context: &GitContext,
    identity: &WorktreeIdentity,
    home: Option<&Path>,
    environment: &[(OsString, OsString)],
) -> Gate {
    return recover_for_command_with_home(context, identity, home, environment);
}

/// What:
///  The startup step with the home directory that holds the repository link identity file named.
/// Why:
///  The link gate runs first and costs one failed open when the file is absent;
/// tests give a disposable home.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recoverForCommand({ context, identity, home })
/// ```
pub fn recover_for_command_with_home(
    context: &GitContext,
    identity: &WorktreeIdentity,
    home: Option<&Path>,
    environment: &[(OsString, OsString)],
) -> Gate {
    let Some((git_dir, common_dir, worktree_root)) = worktree_directories(identity) else {
        return Gate::Clear;
    };
    // The paths the layout resolved (a worktree `.git` file may name the repository through the link while the command runs elsewhere).
    let resolved: [PathBuf; 2] = [git_dir.to_path_buf(), common_dir.to_path_buf()];
    if let LinkGate::Stop(stderr) = link_gate(home, &resolved) {
        return Gate::RepositoryUnreachable(stderr);
    }
    let legacy: PathBuf = git_dir.join(LEGACY_JOURNAL_DIRECTORY_NAME);
    match recovery_path_exists(legacy.as_path()) {
        Ok(true) => return Gate::Stop(legacy_notice(legacy.as_path())),
        Ok(false) => {}
        Err(error) => return stopped(&error),
    }
    if let Err(error) = settle_merges(context, git_dir, common_dir, worktree_root, environment, &mut std::io::stderr()) {
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
///   The code stays `content-unavailable` (state that could not be read),
/// but the generic hint for that code tells the person
///  to fetch history,
/// which does not apply:
/// the message itself names the file or directory and the ways out.
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
