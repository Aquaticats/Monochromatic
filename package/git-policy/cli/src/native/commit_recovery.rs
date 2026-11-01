//! What:
//!  Startup recovery of interrupted commit transactions for the invocation's worktree.
//! Why:
//!  Before configuration loads,
//!  and before read-only commands that already asked Git for the
//!      repository location,
//!  every dead owner's transaction is recovered or fails closed;
//!  live
//!      owners are skipped.
//!  The legacy single-journal directory is not recovered here:
//!  the
//!      command stops and says how to recover it with the previous executable (the owner's
//!      verdict,
//!  `doc/planning/cli-git-rust-open-decisions.md`,
//!  "Legacy single-journal directory").
//!      An empty registry costs one directory read and no Git process
//!      (`src/policy-engine/commit-transaction-recovery.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recoverCommitTransaction({ args, gitPath, identity });
//! ```

/// Landed-capture pruning.
use super::capture_records::prune_landed_captures;
/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Record writers.
use super::json_record::{ObjectWriter, quote};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// Existence probes.
use super::recovery_files::recovery_path_exists;
/// Action names and the recovery place.
use super::recovery_landing::{RecoveryAction, RecoveryPlace, action_name};
/// Registry recovery.
use super::recovery_scan::{RecoveryOutcome, acquire_registry_lock, recover_registered_transactions};
/// Isolated merges that died.
use super::merge_isolation_recovery::{MergeRecoveryOutcome, merge_intents_to_recover, recover_interrupted_merges};
/// The landing lock.
use super::owner_lock::OwnerLock;
/// Running real Git.
use super::transaction_git::GitContext;
/// Registry names.
use super::transaction_registry::{
    LANDING_LOCK_NAME, LEGACY_TRANSACTION_DIRECTORY_NAME, TRANSACTION_ROOT_NAME, ensure_transaction_root,
};
/// The repository shape the invocation selects.
use super::worktree_identity::WorktreeIdentity;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Why recovery stopped the command.
/// Why:
///   The legacy directory and a recovery failure are reported differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryStop = { kind: 'legacy'; directory: string } | { kind: 'failure'; error: CommitTransactionRecoveryError };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryStop {
    /// The legacy single-journal directory exists.
    Legacy(PathBuf),
    /// Recovery failed closed.
    Failure(RecoveryError),
}

/// What:
///  The message for the legacy single-journal directory.
/// Why:
///   The owner chose to stop with instructions instead of porting its recovery.
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
///  The outcomes as the incumbent's debug JSON.
/// Why:
///   `JSON.stringify(outcomes)` in the recovery debug line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(outcomes)
/// ```
pub fn outcomes_json(outcomes: &[RecoveryOutcome]) -> String {
    let mut items: Vec<String> = Vec::with_capacity(outcomes.len());
    for outcome in outcomes {
        let mut writer: ObjectWriter = ObjectWriter::new();
        writer
            .string("directory", outcome.directory.to_string_lossy().as_ref())
            .raw("action", quote(action_name(outcome.action)).as_str());
        items.push(writer.finish());
    }
    return format!("[{}]", items.join(","));
}

/// What:
///  Whether a registry root holds nothing:
///  absent or empty.
/// Why:
///   The common case needs no further reads and no Git process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdirOrEmpty(root)).length === 0
/// ```
fn registry_is_empty(root: &Path) -> Result<bool, RecoveryError> {
    let mut listing: std::fs::ReadDir = match std::fs::read_dir(root) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => {
            return Err(RecoveryError(format!(
                "Unsafe transaction registry: {}",
                root.display()
            )));
        }
        Err(error) => return Err(io_failure("listing", root, &error)),
    };
    match listing.next() {
        None => return Ok(true),
        Some(Ok(_)) => return Ok(false),
        Some(Err(error)) => return Err(io_failure("listing", root, &error)),
    }
}

/// What:
///  The Git directory whose registry this invocation recovers,
///  or nothing outside a worktree
///       and in a bare repository.
/// Why:
///   `resolveCommitTransactionTargets`:
///  only worktrees hold transactions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identity.kind === 'outside-worktree' || identity.kind === 'bare-repository' ? RECOVERY_TARGET_NOT_APPLICABLE : identity.gitDir
/// ```
pub fn recovery_directories(identity: &WorktreeIdentity) -> Option<(&Path, &Path)> {
    match identity {
        WorktreeIdentity::OutsideWorktree | WorktreeIdentity::BareRepository { .. } => return None,
        WorktreeIdentity::MainWorktree {
            git_dir,
            common_dir,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            git_dir,
            common_dir,
            ..
        } => return Some((git_dir.as_path(), common_dir.as_path())),
    }
}

/// What:
///  Recover every interrupted transaction of the invocation's worktree;
///  returns the
///       outcomes,
///  empty when there was nothing to do.
/// Why:
///   `recoverCommitTransaction`,
///  with the legacy directory stopping the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverCommitTransaction({ args, gitPath, identity }): Promise<readonly CommitTransactionRecoveryOutcome[]>;
/// ```
pub fn recover_commit_transactions(
    git: &GitContext,
    cwd: &Path,
    identity: &WorktreeIdentity,
) -> Result<Vec<RecoveryOutcome>, RecoveryStop> {
    let Some((git_dir, common_dir)) = recovery_directories(identity) else {
        return Ok(Vec::new());
    };
    let legacy: PathBuf = git_dir.join(LEGACY_TRANSACTION_DIRECTORY_NAME);
    match recovery_path_exists(legacy.as_path()) {
        Ok(true) => return Err(RecoveryStop::Legacy(legacy)),
        Ok(false) => {}
        Err(error) => return Err(RecoveryStop::Failure(error)),
    }
    let root: PathBuf = git_dir.join(TRANSACTION_ROOT_NAME);
    let place: RecoveryPlace = RecoveryPlace {
        git: git.clone(),
        cwd: cwd.to_path_buf(),
        common_dir: common_dir.to_path_buf(),
    };
    // An isolated merge whose process died is settled under the landing lock before anything else reads the
    // checkout; a merge that still runs is left alone.
    match merge_intents_to_recover(git_dir) {
        Ok(true) => {
            if let Err(error) = recover_merges_under_lock(&place, git_dir, root.as_path()) {
                return Err(RecoveryStop::Failure(error));
            }
        }
        Ok(false) => {}
        Err(error) => return Err(RecoveryStop::Failure(error)),
    }
    match registry_is_empty(root.as_path()) {
        Ok(true) => return Ok(Vec::new()),
        Ok(false) => {}
        Err(error) => return Err(RecoveryStop::Failure(error)),
    }
    let outcomes: Vec<RecoveryOutcome> =
        match recover_registered_transactions(&place, root.as_path()) {
            Ok(found) => found,
            Err(error) => return Err(RecoveryStop::Failure(error)),
        };
    if !outcomes.is_empty() {
        debug(
            "recoverCommitTransaction",
            format!(
                "transaction recovery outcomes: {}",
                outcomes_json(outcomes.as_slice())
            )
            .as_str(),
        );
    }
    if has_recovered(outcomes.as_slice()) {
        // A recovered transaction no longer pins landed-capture records.
        prune_landed_captures(git_dir, root.as_path());
    }
    return Ok(outcomes);
}

/// What:
///  Take the landing lock and settle the isolated merges whose owner died.
/// Why:
///   A merge holds the landing lock for its whole run, so under the lock every record left is a dead merge's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using lock = await acquireOwnerLock(landingLock); await recoverInterruptedMerges({ place, gitDir });
/// ```
fn recover_merges_under_lock(place: &RecoveryPlace, git_dir: &Path, root: &Path) -> Result<(), RecoveryError> {
    ensure_transaction_root(root)?;
    let lock: OwnerLock = acquire_registry_lock(root, LANDING_LOCK_NAME)?;
    let recovered: Result<Vec<MergeRecoveryOutcome>, RecoveryError> = recover_interrupted_merges(place, git_dir);
    let released: Result<(), super::owner_lock::OwnerLockError> = lock.release();
    let outcomes: Vec<MergeRecoveryOutcome> = recovered?;
    if !outcomes.is_empty() {
        debug("recoverCommitTransaction", format!("isolated merge recovery outcomes: {outcomes:?}").as_str());
    }
    if let Err(error) = released {
        return Err(RecoveryError(error.to_string()));
    }
    return Ok(());
}

/// What:
///  Whether any outcome did more than skip a live owner.
/// Why:
///   Pruning runs only after a recovery changed something.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcomes.some((outcome) => outcome.action !== 'owner-active')
/// ```
fn has_recovered(outcomes: &[RecoveryOutcome]) -> bool {
    for outcome in outcomes {
        if outcome.action != RecoveryAction::OwnerActive {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether outcomes leave a transaction another process may still be running.
/// Why:
///   Until index writers coordinate with live transactions,
///  a guarded command beside one is
///       refused instead of racing it.
///  A dead owner's kept staging candidate touched nothing and
///       does not count;
///  an unattributed one may still be written by a live owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcomes.some((outcome) => outcome.action === 'owner-active' || outcome.action === 'staging-unattributed')
/// ```
pub fn leaves_live_transactions(outcomes: &[RecoveryOutcome]) -> bool {
    for outcome in outcomes {
        if matches!(
            outcome.action,
            RecoveryAction::OwnerActive | RecoveryAction::StagingUnattributed
        ) {
            return true;
        }
    }
    return false;
}

/// Recovery entry controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_recovery_tests.rs"]
mod tests;
