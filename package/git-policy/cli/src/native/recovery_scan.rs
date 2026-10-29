//! What:
//!  Startup recovery across every per-transaction directory of one worktree.
//! Why:
//!  Live owners are skipped;
//!  dead owners are recovered oldest first;
//!  dead transactions that
//!      entered the landing critical section are recovered only while holding the landing lock,
//!      so recovery never races a live lander (`src/policy-engine/commit-transaction-recovery-scan.ts`,
//!      `commit-landing-lock.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcomes = await recoverRegisteredTransactions({ root, gitPath, effectiveCwd });
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// Owner locks.
use super::live_lock::acquire_live_owner_lock;
use super::owner_lock::{OwnerLock, retire_lock_of_dead_owner};
/// The fail-closed recovery failure.
use super::recovery_error::{RecoveryError, io_failure};
/// Landing evidence.
use super::recovery_evidence::entered_landing;
/// Owner inspection and order.
use super::recovery_inspect::{
    InspectedEntry, OwnerEvidence, inspect_registry_entry, oldest_first,
};
/// Dead-transaction recovery.
use super::recovery_landing::{RecoveryAction, RecoveryPlace, recover_dead_transaction};
/// Isolated merges that died.
use super::merge_isolation_recovery::recover_interrupted_merges;
/// Owner liveness.
use super::transaction_owner::OwnerLiveness;
/// Registry listing and names.
use super::transaction_registry::{
    EntryKind, LANDING_LOCK_NAME, RESERVATION_LOCK_NAME, RegistryEntry, list_transaction_entries,
};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  What recovery did for one directory.
/// Why:
///   `CommitTransactionRecoveryOutcome`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CommitTransactionRecoveryOutcome = { directory: string; action: CommitTransactionRecoveryAction };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryOutcome {
    /// The directory examined.
    pub directory: PathBuf,
    /// What recovery did,
    ///  or why it left the directory.
    pub action: RecoveryAction,
}

/// What:
///  Recover or skip one inspected entry without a landing record,
///  or one whose landing the
///       caller serializes.
/// Why:
///   `recoverInspectedEntry`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverInspectedEntry({ inspected, gitPath, effectiveCwd }): Promise<CommitTransactionRecoveryAction>;
/// ```
pub fn recover_inspected_entry(
    place: &RecoveryPlace,
    inspected: &InspectedEntry,
) -> Result<RecoveryAction, RecoveryError> {
    let path: &Path = inspected.entry.path.as_path();
    match &inspected.owner {
        OwnerEvidence::Vanished => {
            debug(
                "recoverInspectedEntry",
                format!(
                    "transaction finished while recovery listed it: {}",
                    path.display()
                )
                .as_str(),
            );
            return Ok(RecoveryAction::Vanished);
        }
        OwnerEvidence::Unattributed => {
            debug(
                "recoverInspectedEntry",
                format!(
                    "staging candidate without a complete owner record left in place: {}",
                    path.display()
                )
                .as_str(),
            );
            return Ok(RecoveryAction::StagingUnattributed);
        }
        OwnerEvidence::Owned(record, OwnerLiveness::Alive) => {
            debug(
                "recoverInspectedEntry",
                format!(
                    "transaction owner {} is active; skipping {}",
                    record.owner_pid,
                    path.display()
                )
                .as_str(),
            );
            return Ok(RecoveryAction::OwnerActive);
        }
        OwnerEvidence::Owned(record, OwnerLiveness::Dead) => {
            if inspected.entry.kind == EntryKind::Staging {
                // An unpublished candidate never started capture, so it touched nothing outside
                // itself and remains for diagnosis.
                debug(
                    "recoverInspectedEntry",
                    format!(
                        "dead owner stopped before publishing; staging candidate kept: {}",
                        path.display()
                    )
                    .as_str(),
                );
                return Ok(RecoveryAction::StagingRetained);
            }
            return recover_dead_transaction_unless_vanished(
                place,
                path,
                inspected.entry.transaction_id.as_str(),
                record.owner_pid,
            );
        }
    }
}

/// What:
///  `recover_dead_transaction`, answering "vanished" when the failure came from a transaction directory that is gone.
/// Why:
///   Another recoverer of the same dead transaction may finish and remove it after this recoverer inspected it;
///  the incumbent treats a directory that disappeared between listing and reading as finished,
///  whereas a failure with the directory still present is real.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await recoverDeadTransaction(request); } catch (e) { if (!(await pathExists(directory))) return 'vanished'; throw e; }
/// ```
fn recover_dead_transaction_unless_vanished(
    place: &RecoveryPlace,
    directory: &Path,
    transaction_id: &str,
    owner_pid: i64,
) -> Result<RecoveryAction, RecoveryError> {
    match recover_dead_transaction(place, directory, transaction_id, owner_pid) {
        Ok(action) => return Ok(action),
        Err(error) => {
            if directory.exists() {
                return Err(error);
            }
            debug(
                "recoverInspectedEntry",
                format!("transaction finished while recovery recovered it: {}", directory.display()).as_str(),
            );
            return Ok(RecoveryAction::Vanished);
        }
    }
}

/// What:
///  Whether an inspected entry is a dead published transaction that entered the landing critical section:
///  it holds a landing record or the record of a real `index.lock` it created.
/// Why:
///   `needsLandingLock` keyed on the landing record alone, so a transaction killed holding the landing lock and
///       `index.lock` (an index-lock record, no landing record yet) was recovered without the landing lock.
///  Another recoverer under the lock could release that `index.lock` and a lander could create a new one at the
///       same path before this late release deleted it.
///  `entered_landing` is the predicate the landing pass already uses,
///  and the landing pass is documented as the only recovery that may release such a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function needsLandingLock(inspected: InspectedEntry): Promise<boolean>;
/// ```
fn needs_landing_lock(inspected: &InspectedEntry) -> Result<bool, RecoveryError> {
    if inspected.entry.kind != EntryKind::Transaction || inspected.dead_owner().is_none() {
        return Ok(false);
    }
    return entered_landing_unless_vanished(inspected.entry.path.as_path());
}

/// What:
///  `entered_landing`, answering "no" for a transaction directory that is gone.
/// Why:
///   Another recoverer of the same dead transaction may finish and remove it between this recoverer's registry listing
///       and this check; a transaction that no longer exists has nothing to recover,
///  whereas a directory that exists but cannot be listed stays an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { return await enteredLanding(directory); } catch (e) { if (e.code === 'ENOENT') return false; throw e; }
/// ```
fn entered_landing_unless_vanished(directory: &Path) -> Result<bool, RecoveryError> {
    match entered_landing(directory) {
        Ok(entered) => return Ok(entered),
        Err(error) => {
            if directory.exists() {
                return Err(error);
            }
            debug(
                "recoverRegisteredTransactions",
                format!("transaction finished while recovery listed it: {}", directory.display()).as_str(),
            );
            return Ok(false);
        }
    }
}

/// What:
///  Recover every dead published transaction that entered the landing critical section;
///  the
///       caller holds the landing lock.
/// Why:
///   `recoverDeadLandings`:
///  an owner killed while holding the landing lock and `index.lock`
///       has no landing record yet,
///  and its `index.lock` would block the next landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverDeadLandings({ root, gitPath, effectiveCwd }): Promise<readonly CommitTransactionRecoveryOutcome[]>;
/// ```
pub fn recover_dead_landings(
    place: &RecoveryPlace,
    root: &Path,
) -> Result<Vec<RecoveryOutcome>, RecoveryError> {
    let mut inspected: Vec<InspectedEntry> = Vec::new();
    for entry in list_transaction_entries(root)? {
        if entry.kind == EntryKind::Transaction {
            inspected.push(inspect_registry_entry(&entry)?);
        }
    }
    // `mut` allows collecting outcomes in recovery order.
    let mut outcomes: Vec<RecoveryOutcome> = Vec::new();
    for item in oldest_first(inspected) {
        let Some(record) = item.dead_owner() else {
            continue;
        };
        if !entered_landing_unless_vanished(item.entry.path.as_path())? {
            continue;
        }
        debug(
            "recoverDeadLandings",
            format!("recovering dead landing {}", item.entry.path.display()).as_str(),
        );
        let action: RecoveryAction = recover_dead_transaction_unless_vanished(
            place,
            item.entry.path.as_path(),
            item.entry.transaction_id.as_str(),
            record.owner_pid,
        )?;
        outcomes.push(RecoveryOutcome {
            directory: item.entry.path.clone(),
            action,
        });
    }
    // An isolated merge that died while it held the landing lock is settled under the same lock.
    if let Some(git_dir) = root.parent() {
        let merges = recover_interrupted_merges(place, git_dir)?;
        if !merges.is_empty() {
            debug("recoverDeadLandings", format!("isolated merge recovery outcomes: {merges:?}").as_str());
        }
    }
    return Ok(outcomes);
}

/// What:
///  Take the landing lock,
///  waiting while a live owner holds it.
/// Why:
///   Recovery of a landing must never race a live lander.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquireOwnerLock({ lockDirectory: join(root, LANDING_LOCK_NAME) })
/// ```
pub fn acquire_registry_lock(root: &Path, name: &str) -> Result<OwnerLock, RecoveryError> {
    let directory: PathBuf = root.join(name);
    match acquire_live_owner_lock(directory.as_path(), "a transaction registry lock") {
        Ok(lock) => return Ok(lock),
        Err(error) => return Err(RecoveryError(error.to_string())),
    }
}

/// What:
///  Whether an outcome already names a directory.
/// Why:
///   The landing pass may have recovered an entry the main pass would visit again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// outcomes.some((outcome) => outcome.directory === path)
/// ```
fn recovered_already(outcomes: &[RecoveryOutcome], path: &Path) -> bool {
    for outcome in outcomes {
        if outcome.directory == path {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether the landing pass owns an entry:
///  it was known to need the landing lock, or the pass has already recovered it.
/// Why:
///   An entry whose owner died after the first inspection is recovered by the landing pass although it did not need
///  the lock then, and one that vanished before the pass is not recovered at all;
///  in both cases the main pass must leave it alone instead of acting on its stale inspection.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// needed || outcomes.some((outcome) => outcome.directory === path)
/// ```
fn handled_by_the_landing_pass(needed: bool, outcomes: &[RecoveryOutcome], path: &Path) -> bool {
    return needed || recovered_already(outcomes, path);
}

/// What:
///  Recover every dead-owner transaction of one worktree registry and remove retired
///       leftovers;
///  a dead reservation holder's lock is retired too.
/// Why:
///   `recoverRegisteredTransactions`.
///  Unlike the incumbent,
///  an entry the landing pass already
///       recovered is not recovered a second time (the incumbent would fail reading its removed
///       directory;
///  recorded in `doc/handover/cli-git-native-transactions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverRegisteredTransactions({ root, gitPath, effectiveCwd }): Promise<readonly CommitTransactionRecoveryOutcome[]>;
/// ```
pub fn recover_registered_transactions(
    place: &RecoveryPlace,
    root: &Path,
) -> Result<Vec<RecoveryOutcome>, RecoveryError> {
    let entries: Vec<RegistryEntry> = list_transaction_entries(root)?;
    let mut inspected: Vec<InspectedEntry> = Vec::new();
    let mut retired: Vec<PathBuf> = Vec::new();
    for entry in &entries {
        if entry.kind == EntryKind::Retired {
            retired.push(entry.path.clone());
        } else {
            inspected.push(inspect_registry_entry(entry)?);
        }
    }
    let ordered: Vec<InspectedEntry> = oldest_first(inspected);
    let mut needs: Vec<bool> = Vec::with_capacity(ordered.len());
    for item in &ordered {
        needs.push(needs_landing_lock(item)?);
    }
    let mut outcomes: Vec<RecoveryOutcome> = Vec::new();
    if needs.contains(&true) {
        let lock: OwnerLock = acquire_registry_lock(root, LANDING_LOCK_NAME)?;
        let recovered: Result<Vec<RecoveryOutcome>, RecoveryError> =
            recover_dead_landings(place, root);
        let released: Result<(), super::owner_lock::OwnerLockError> = lock.release();
        outcomes.extend(recovered?);
        if let Err(error) = released {
            return Err(RecoveryError(error.to_string()));
        }
    }
    for (index, item) in ordered.iter().enumerate() {
        if handled_by_the_landing_pass(needs[index], outcomes.as_slice(), item.entry.path.as_path()) {
            continue;
        }
        let action: RecoveryAction = recover_inspected_entry(place, item)?;
        outcomes.push(RecoveryOutcome {
            directory: item.entry.path.clone(),
            action,
        });
    }
    for path in &retired {
        if let Err(error) = super::private_storage::remove_tree(path.as_path()) {
            return Err(io_failure(
                "removing the retired transaction",
                path.as_path(),
                &error,
            ));
        }
    }
    // A crashed reservation holder's lock is retired here as well as by the next requester.
    if let Err(error) = retire_lock_of_dead_owner(root.join(RESERVATION_LOCK_NAME).as_path()) {
        return Err(RecoveryError(error.to_string()));
    }
    for path in retired {
        outcomes.push(RecoveryOutcome {
            directory: path,
            action: RecoveryAction::RetiredRemoved,
        });
    }
    return Ok(outcomes);
}

/// Registry recovery controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_scan_tests.rs"]
mod tests;
