//! What:
//!  Recovery of one dead-owner transaction from its schema-version-2 records.
//! Why:
//!  Without a landing record the real index and refs were never touched:
//!  recovery releases
//!      the `index.lock` the transaction provably created and removes its `.keep` files,
//!  shadow and
//!      directory.
//!  With a landing record (the caller holds the landing lock) it decides from
//!      `ref-updated.json` or the reflog nonce whether the commit landed,
//!  then discards the
//!      attempt or completes it:
//!  index,
//!  conclusion cleanup,
//!  added-path copies.
//!  Missing evidence
//!      fails closed and keeps the directory (`src/policy-engine/commit-transaction-recovery-landing.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recoverDeadTransaction({ directory, transactionId, ownerPid, gitPath, effectiveCwd });
//! ```

/// Landed-capture recording.
use super::capture_records::record_landed_capture_or_warn;
/// Native conclusion cleanup.
use super::conclusion_cleanup::reproduce_conclusion_cleanup;
/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Transaction keeps.
use super::pack_keep::remove_transaction_keeps;
/// The refusal to discard a transaction whose branch moved without a usable landing record.
use super::recovery_contradiction::refuse_unreconciled_landing;
/// The landed-or-not decision and index completion.
use super::recovery_completion::{IndexCompletion, commit_landed, complete_index};
/// The fail-closed recovery failure.
use super::recovery_error::RecoveryError;
/// Attempt evidence.
use super::recovery_evidence::{
    LANDING_RECORD_PREFIX, RECORD_SUFFIX, attempt_numbers, directory_names, release_recorded_locks,
    remove_dead_pid_file,
};
/// Exact reads and existence probes.
use super::recovery_files::{path_present, read_recovery_file, recovery_path_exists};
/// Shadow location and removal.
use super::shadow_git::{remove_shadow_repository, shadow_repository_path};
/// Running real Git.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// Journal records.
use super::transaction_journal::{
    Base, LandingOperation, LandingRecord, PREPARING_FILENAME, PreparingRecord,
};
/// Journal parsers.
use super::transaction_journal_parse::{parse_landing, parse_preparing};
/// Directory removal.
use super::transaction_registry::remove_transaction_directory;
/// Added-path worktree completion.
use super::worktree_completion::install_added_worktree_files;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Prepared-journal filename written only by unreleased builds in registry directories.
pub const UNRELEASED_JOURNAL_FILENAME: &str = "journal.json";

/// What:
///  What recovery did for one transaction directory.
/// Why:
///   `CommitTransactionRecoveryAction`,
///  reported in debug output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitTransactionRecoveryAction = 'commit-not-created' | 'normalization-installed' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    /// The commit never landed;
    ///  the attempt was discarded.
    CommitNotCreated,
    /// An interrupted normalization's index was completed.
    NormalizationInstalled,
    /// A landed commit's post-index was installed now.
    IndexInstalled,
    /// A landed commit's post-index was already installed.
    AlreadyInstalled,
    /// The owner still runs;
    ///  nothing was touched.
    OwnerActive,
    /// The directory disappeared while recovery listed it.
    Vanished,
    /// A staging candidate without a complete owner record was left in place.
    StagingUnattributed,
    /// A dead owner's unpublished candidate was kept for diagnosis.
    StagingRetained,
    /// A retired directory whose removal was interrupted was removed.
    RetiredRemoved,
}

/// What:
///  The incumbent's name of an action.
/// Why:
///   Debug output lists outcomes by these names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// action // 'commit-not-created' | ...
/// ```
pub fn action_name(action: RecoveryAction) -> &'static str {
    match action {
        RecoveryAction::CommitNotCreated => return "commit-not-created",
        RecoveryAction::NormalizationInstalled => return "normalization-installed",
        RecoveryAction::IndexInstalled => return "index-installed",
        RecoveryAction::AlreadyInstalled => return "already-installed",
        RecoveryAction::OwnerActive => return "owner-active",
        RecoveryAction::Vanished => return "vanished",
        RecoveryAction::StagingUnattributed => return "staging-unattributed",
        RecoveryAction::StagingRetained => return "staging-retained",
        RecoveryAction::RetiredRemoved => return "retired-removed",
    }
}

/// What:
///  Where recovery runs Git and finds the shared Git directory.
/// Why:
///   Every recovery step asks Git in the invocation's repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryPlace = { gitPath: string; effectiveCwd: string; commonDir: string };
/// ```
#[derive(Clone, Debug)]
pub struct RecoveryPlace {
    /// Real Git and its environment.
    pub git: GitContext,
    /// Working directory of every Git command.
    pub cwd: PathBuf,
    /// The repository's common Git directory.
    pub common_dir: PathBuf,
}

/// What:
///  Remove everything a dead transaction left:
///  `.keep` files,
///  the shadow,
///  the directory.
/// Why:
///   `discardTransaction`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await discardTransaction({ directory, transactionId, objectDirectory, shadowPath });
/// ```
fn discard_transaction(
    directory: &Path,
    transaction_id: &str,
    preparing: &PreparingRecord,
) -> Result<(), RecoveryError> {
    remove_transaction_keeps(
        Path::new(preparing.object_directory.as_str()),
        transaction_id,
    )?;
    remove_shadow_repository(Path::new(preparing.shadow_path.as_str()))?;
    return remove_transaction_directory(directory);
}

/// What:
///  The invocation capture record,
///  or nothing when the owner died before writing it.
/// Why:
///   `readPreparing`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readPreparing(directory: string): Promise<PreparingRecord | typeof PREPARING_ABSENT>;
/// ```
fn read_preparing(directory: &Path) -> Result<Option<PreparingRecord>, RecoveryError> {
    let path: PathBuf = directory.join(PREPARING_FILENAME);
    if !recovery_path_exists(path.as_path())? {
        return Ok(None);
    }
    return Ok(Some(parse_preparing(
        read_recovery_file(path.as_path())?.as_slice(),
    )?));
}

/// What:
///  Whether the target still resolves to the base an interrupted normalization recorded.
/// Why:
///   A normalization never moves the target;
///  one that moved belongs to someone else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const current = await resolveRefCommit({ gitPath, cwd, ref: preparing.targetRef });
/// ```
fn target_unmoved(place: &RecoveryPlace, preparing: &PreparingRecord, expected: &Base) -> bool {
    let revision: String = format!("{}^{{commit}}", preparing.target_ref);
    let current: Base = match run_git(
        &place.git,
        &GitRequest::new(
            place.cwd.as_path(),
            &["rev-parse", "--verify", "--quiet", revision.as_str()],
        ),
    ) {
        Ok(output) if output.succeeded() => Base::Commit(
            String::from_utf8_lossy(output.stdout.as_slice())
                .trim()
                .to_string(),
        ),
        _ => Base::Unborn,
    };
    return current == *expected;
}

/// What:
///  Recover one dead-owner transaction;
///  a caller recovering one with a landing record must
///       hold the landing lock.
/// Why:
///   `recoverDeadTransaction`,
///  step for step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverDeadTransaction({ directory, transactionId, ownerPid, gitPath, effectiveCwd }): Promise<CommitTransactionRecoveryAction>;
/// ```
pub fn recover_dead_transaction(
    place: &RecoveryPlace,
    directory: &Path,
    transaction_id: &str,
    owner_pid: i64,
) -> Result<RecoveryAction, RecoveryError> {
    let names: Vec<String> = directory_names(directory)?;
    if names.contains(&String::from(UNRELEASED_JOURNAL_FILENAME)) {
        return Err(RecoveryError(format!(
            "Transaction directory uses an unreleased journal format; inspect it before removing it: {}",
            directory.display()
        )));
    }
    let preparing_record: Option<PreparingRecord> = read_preparing(directory)?;
    refuse_unreconciled_landing(&place.git, place.cwd.as_path(), directory, names.as_slice(), preparing_record.as_ref())?;
    let Some(preparing) = preparing_record else {
        debug(
            "recoverDeadTransaction",
            format!("dead owner stopped before capture: {}", directory.display()).as_str(),
        );
        remove_shadow_repository(
            shadow_repository_path(place.common_dir.as_path(), transaction_id).as_path(),
        )?;
        remove_transaction_directory(directory)?;
        return Ok(RecoveryAction::CommitNotCreated);
    };
    let real_index: &Path = Path::new(preparing.real_index_path.as_str());
    let Some(latest) = attempt_numbers(names.as_slice(), LANDING_RECORD_PREFIX)
        .last()
        .copied()
    else {
        release_recorded_locks(directory, names.as_slice(), real_index, owner_pid)?;
        discard_transaction(directory, transaction_id, &preparing)?;
        return Ok(RecoveryAction::CommitNotCreated);
    };
    let landing: LandingRecord = parse_landing(
        read_recovery_file(
            directory
                .join(format!("{LANDING_RECORD_PREFIX}{latest}{RECORD_SUFFIX}"))
                .as_path(),
        )?
        .as_slice(),
    )?;
    if landing.operation == LandingOperation::Commit
        && !commit_landed(
            &place.git,
            place.cwd.as_path(),
            directory,
            &preparing,
            &landing,
        )?
    {
        debug(
            "recoverDeadTransaction",
            format!(
                "landing attempt {latest} never advanced {}: {}",
                preparing.target_ref,
                directory.display()
            )
            .as_str(),
        );
        release_recorded_locks(directory, names.as_slice(), real_index, owner_pid)?;
        discard_transaction(directory, transaction_id, &preparing)?;
        return Ok(RecoveryAction::CommitNotCreated);
    }
    if landing.operation == LandingOperation::NormalizeOnly
        && !target_unmoved(place, &preparing, &landing.expected_old)
    {
        return Err(RecoveryError(format!(
            "{} moved after an interrupted normalization; recovery retained at {}",
            preparing.target_ref,
            directory.display()
        )));
    }
    remove_transaction_keeps(
        Path::new(preparing.object_directory.as_str()),
        transaction_id,
    )?;
    if landing.operation == LandingOperation::Commit
        && let Some(landed) = &landing.new_oid
    {
        // Before any other landing takes the landing lock, so no replay moves over this commit
        // without its record.
        record_landed_capture_or_warn(
            Path::new(preparing.git_dir.as_str()),
            directory,
            transaction_id,
            landed.as_str(),
        );
    }
    let completion: IndexCompletion = complete_index(directory, &preparing, &landing)?;
    remove_dead_pid_file(real_index, owner_pid)?;
    let shadow: &Path = Path::new(preparing.shadow_path.as_str());
    if landing.operation == LandingOperation::Commit && path_present(shadow)? {
        reproduce_conclusion_cleanup(
            &place.git,
            place.cwd.as_path(),
            Path::new(preparing.git_dir.as_str()),
            shadow,
            directory,
            preparing.ref_format,
        )?;
    }
    // Completion precedes shadow removal, so a missing shadow means an earlier run already
    // finished the copies; the shadow store still holds originals that never landed.
    if path_present(shadow)? {
        let mut records: Vec<super::transaction_journal::AddedPath> = landing.added_paths.clone();
        records.extend(landing.selected_worktree_paths.iter().cloned());
        install_added_worktree_files(
            &place.git,
            place.cwd.as_path(),
            Path::new(preparing.repository_root.as_str()),
            records.as_slice(),
            Some(shadow.join("objects").as_path()),
        )?;
    }
    remove_shadow_repository(shadow)?;
    remove_transaction_directory(directory)?;
    if landing.operation == LandingOperation::NormalizeOnly {
        return Ok(RecoveryAction::NormalizationInstalled);
    }
    match completion {
        IndexCompletion::Installed => return Ok(RecoveryAction::IndexInstalled),
        IndexCompletion::AlreadyInstalled => return Ok(RecoveryAction::AlreadyInstalled),
    }
}

/// Dead-transaction recovery controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_landing_tests.rs"]
pub(crate) mod tests;
