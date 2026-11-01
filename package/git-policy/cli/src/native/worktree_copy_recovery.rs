//! What:
//!  Recovery of interrupted linked-worktree copies: the work of every abandoned claim is resumed, completed, or
//!  discarded with a notice; malformed state fails closed and stays; the pending check that forwarded commands run first.
//! Why:
//!  A copy interrupted by a crash must not leave a worktree half-filled forever, but recovery may only act on
//!  state no live copy holds, on journals whose destination is still a registration of this repository and on stages
//!  that are still private. Liveness comes from the claim file's open descriptor (`worktree_copy_claim.rs`), so a
//!  pass never waits for a running copy and never touches one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = await recoverWorktreeCopyTransactions(commonDir);
//! await recoverPendingWorktreeCopies(commonDir);
//! ```

use super::json_record::quote;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_claim::{
    ADOPTED_KEY_PREFIX, Claim, Examination, claims_exist, examine_claim, list_claims, publish_claim,
    remove_abandoned_temporaries,
};
use super::worktree_copy_stage::StagedSnapshot;
use super::worktree_copy_transaction::{Outcome, complete_journal, snapshot_from_journal};
use crate::worktree_copy_journal::{
    PendingJournal, Phase, Role, assert_private_path, journal_root, read_pending_journals, remove_pending_journal,
    remove_stage,
};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  Whether a journal's destination is still a linked worktree of the journal's repository.
/// Why:
///   A destination that was removed, moved, pruned or re-pointed can receive nothing, so its journal is discarded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestinationRegistration = 'registered' | 'unregistered';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Registration {
    /// The destination is registered under the common directory.
    Registered,
    /// The destination is not registered under the common directory.
    Unregistered,
}

/// What:
///  Whether a journal's private stage and payload still exist.
/// Why:
///   Installation reads the payload, so a missing payload ends recovery with a notice instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagePresence = 'stage-intact' | 'stage-missing';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StagePresence {
    /// The container and payload exist and are private.
    Intact,
    /// The container or payload is gone.
    Missing,
}

/// What:
///  One pass of recovery: how many transactions completed and one notice per transaction ended otherwise.
/// Why:
///   The caller prints the notices and the summary; the counts come from this one value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecoveryReport = { recovered: number; notices: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    /// Transactions that completed, including completed cleanups.
    pub recovered: usize,
    /// One line for each transaction that ended without completing.
    pub notices: Vec<String>,
}

/// What:
///  The raw registration check before missing paths are turned into `unregistered`.
/// Why:
///   A missing path on the way to the registration is an answer, not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readRegistration({ commonDir, destinationRoot }): Promise<DestinationRegistration>;
/// ```
fn read_registration(common_dir: &Path, destination: &Path) -> std::io::Result<Registration> {
    if std::fs::canonicalize(destination)? != destination {
        return Ok(Registration::Unregistered);
    }
    let pointer: String = std::fs::read_to_string(destination.join(".git"))?
        .trim_end()
        .to_string();
    let Some(rest) = pointer.strip_prefix("gitdir: ") else {
        return Ok(Registration::Unregistered);
    };
    let admin: PathBuf = std::fs::canonicalize(destination.join(rest))?;
    let expected: PathBuf = std::fs::canonicalize(common_dir.join("worktrees"))?;
    if admin.parent() == Some(expected.as_path()) {
        return Ok(Registration::Registered);
    }
    return Ok(Registration::Unregistered);
}

/// What:
///  Whether a destination is still a linked worktree of the common directory.
/// Why:
///   Recovery decides whether to finish or discard a copy from this answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function destinationRegistration({ commonDir, destinationRoot }): Promise<DestinationRegistration>;
/// ```
pub fn destination_registration(
    common_dir: &Path,
    destination: &Path,
) -> Result<Registration, WorktreeCopyError> {
    // `match` on the failure kind: a missing path means unregistered, anything else is a fault to report.
    match read_registration(common_dir, destination) {
        Ok(registration) => return Ok(registration),
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.raw_os_error() == Some(libc::ENOTDIR) =>
        {
            return Ok(Registration::Unregistered);
        }
        Err(error) => {
            let message: String = format!(
                "cli-git: could not read worktree registration of {}.",
                quote(destination.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    }
}

/// What:
///  Checks that a journal's stage container and payload are private directories, or that they are gone.
/// Why:
///   Installation copies from the payload, so a planted or replaced stage must never be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function validateJournalStage(record): Promise<StagePresence>;
/// ```
pub fn validate_stage(pending: &PendingJournal) -> Result<StagePresence, WorktreeCopyError> {
    let container: PathBuf = PathBuf::from(pending.journal.stage_container.as_str());
    let payload: PathBuf = PathBuf::from(pending.journal.stage_root.as_str());
    // `match` on the no-follow lookup: an absent path is reported as missing, and any other failure names the destination.
    for path in [&container, &payload] {
        match std::fs::symlink_metadata(path.as_path()) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(StagePresence::Missing);
            }
            Err(error) => {
                let message: String = format!(
                    "cli-git: could not validate worktree-copy recovery state for {}.",
                    quote(pending.journal.destination_root.as_str())
                );
                return Err(failure_with_cause(message.as_str(), Some(&error)));
            }
        }
        assert_private_path(path.as_path(), Role::PrivateStage)?;
    }
    return Ok(StagePresence::Intact);
}

/// What:
///  Brings one pending transaction to an end.
/// Why:
///   Every journal ends as completed, resumed-and-completed, or discarded with its notice; only unsafe state stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function recoverTransaction({ commonDir, journal }): Promise<RecoveryStep>;
/// ```
fn recover_transaction(
    common_dir: &Path,
    pending: PendingJournal,
) -> Result<(bool, Option<String>), WorktreeCopyError> {
    let destination: String = quote(pending.journal.destination_root.as_str());
    let registration: Registration = destination_registration(
        common_dir,
        Path::new(pending.journal.destination_root.as_str()),
    )?;
    let stage: StagePresence = validate_stage(&pending)?;
    if registration == Registration::Unregistered {
        remove_pending_journal(&pending)?;
        return Ok((
            false,
            Some(format!(
                "cli-git: discarded an interrupted ignored-state copy for {destination}, which is no longer a registered worktree."
            )),
        ));
    }
    if pending.journal.phase == Phase::Complete {
        remove_pending_journal(&pending)?;
        return Ok((true, None));
    }
    if stage == StagePresence::Missing {
        remove_pending_journal(&pending)?;
        return Ok((
            false,
            Some(format!(
                "cli-git: discarded an interrupted ignored-state copy into {destination} because its private stage is missing; ignored files there may be incomplete."
            )),
        ));
    }
    let journal_for_snapshot: PendingJournal = pending.clone();
    let snapshot = move |intended: &BTreeSet<String>| -> Result<StagedSnapshot, WorktreeCopyError> {
        return snapshot_from_journal(&journal_for_snapshot, intended);
    };
    // `match` on the outcome: a completed installation is recovered; a failed one becomes a notice.
    match complete_journal(pending, &snapshot)? {
        Outcome::Installed(_) => return Ok((true, None)),
        Outcome::Ended(failure) => {
            return Ok((
                false,
                Some(format!(
                    "cli-git: ended an interrupted ignored-state copy into {destination} without finishing it. {}",
                    failure.0
                )),
            ));
        }
    }
}

/// What:
///  Settles the claim this process now holds because its creator died: the journal it names is recovered, or a stage no
///  journal names is removed, and then the claim goes.
/// Why:
///   The claim names every durable thing its copy made, and it is removed last, so a crash anywhere in this settlement
///  leaves the claim for the next pass to settle again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = await settleClaim({ commonDir, claim });
/// ```
pub fn settle_claim(common_dir: &Path, claim: Claim) -> Result<RecoveryReport, WorktreeCopyError> {
    let container: PathBuf = claim.container();
    let mut report: RecoveryReport = RecoveryReport {
        recovered: 0,
        notices: Vec::new(),
    };
    let journaled: Vec<PendingJournal> = read_pending_journals(common_dir)?
        .into_iter()
        .filter(|pending| return Path::new(pending.journal.stage_container.as_str()) == container.as_path())
        .collect();
    let has_journal: bool = !journaled.is_empty();
    for pending in journaled {
        let (completed, notice) = recover_transaction(common_dir, pending)?;
        if completed {
            report.recovered += 1;
        }
        if let Some(line) = notice {
            report.notices.push(line);
        }
    }
    // `match` on the stage no journal names: absent was never staged, present was cut off while staging.
    if !has_journal {
        match std::fs::symlink_metadata(container.as_path()) {
            Ok(_) => {
                assert_private_path(container.as_path(), Role::PrivateStage)?;
                remove_stage(container.as_path())?;
                report.notices.push(format!(
                    "cli-git: removed the stage {} that an interrupted ignored-state copy left behind.",
                    quote(container.to_string_lossy().as_ref())
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(raw_failure(&error)),
        }
    }
    claim.release()?;
    return Ok(report);
}

/// What:
///  Adds one report to another.
/// Why:
///   A pass settles several claims and prints one summary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// merge(report, other)
/// ```
fn absorb(report: &mut RecoveryReport, other: RecoveryReport) {
    report.recovered += other.recovered;
    report.notices.extend(other.notices);
}

/// What:
///  Recovers everything no running copy holds, in claim order and then journal order.
/// Why:
///   A claim a process holds open is a copy in flight and is skipped without waiting; a claim nobody holds is settled;
///  a journal no claim names (an older release wrote those) is adopted with a claim of its own, then settled. A journal
///  that cannot be read or proven stops the pass, so its evidence is never acted on blindly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverWorktreeCopyTransactions(commonDir);
/// ```
pub fn recover_worktree_copy_transactions(
    common_dir: &Path,
) -> Result<RecoveryReport, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    // Journals are listed before claims: a journal exists only after its claim, so a journal seen here with no claim
    // below has no running copy.
    let journals: Vec<PendingJournal> = read_pending_journals(common_dir)?;
    remove_abandoned_temporaries(root.as_path())?;
    let mut report: RecoveryReport = RecoveryReport {
        recovered: 0,
        notices: Vec::new(),
    };
    let mut named: BTreeSet<PathBuf> = BTreeSet::new();
    for path in list_claims(root.as_path())? {
        // `match` on the claim: gone needs nothing, held is left to its copy, abandoned is settled by this pass.
        match examine_claim(path.as_path())? {
            Examination::Gone => {}
            Examination::Held { content, .. } => {
                named.insert(content.container());
            }
            Examination::Abandoned(claim) => {
                named.insert(claim.container());
                absorb(&mut report, settle_claim(common_dir, claim)?);
            }
        }
    }
    for pending in journals {
        let container: PathBuf = PathBuf::from(pending.journal.stage_container.as_str());
        if named.contains(&container) {
            continue;
        }
        let stem: String = pending
            .path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let key: String = format!("{ADOPTED_KEY_PREFIX}{stem}");
        // A claim already published under the adoption key belongs to another pass adopting the same journal.
        if let Some(claim) = publish_claim(
            root.as_path(),
            key.as_str(),
            Path::new(pending.journal.destination_root.as_str()),
            container.as_path(),
        )? {
            absorb(&mut report, settle_claim(common_dir, claim)?);
        }
    }
    return Ok(report);
}

/// What:
///  Writes each notice on its own line, then the summary line when any transaction recovered.
/// Why:
///   The user sees why each discarded copy was discarded, and how many resumed copies completed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportRecoveredWorktreeCopies(report);
/// ```
pub fn report_recovered(
    report: &RecoveryReport,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    for notice in &report.notices {
        writeln!(stderr, "{notice}").map_err(|error| return raw_failure(&error))?;
    }
    if report.recovered > 0 {
        let noun: &str = if report.recovered == 1 {
            "transaction"
        } else {
            "transactions"
        };
        writeln!(
            stderr,
            "cli-git: recovered ignored-state copies for {} worktree {noun}.",
            report.recovered
        )
        .map_err(|error| return raw_failure(&error))?;
    }
    return Ok(());
}

/// What:
///  The pending check forwarded commands run first: recovers abandoned copies only when something is pending.
/// Why:
///   A command that creates no worktrees must not wait behind a copy, but still must not pass a half-recovered
///  repository; the common case costs two directory reads and a running copy is skipped, never waited for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverPendingWorktreeCopies(commonDir);
/// ```
pub fn recover_pending_worktree_copies(
    common_dir: &Path,
    stderr: &mut dyn Write,
) -> Result<(), WorktreeCopyError> {
    if read_pending_journals(common_dir)?.is_empty() && !claims_exist(journal_root(common_dir).as_path())? {
        return Ok(());
    }
    return report_recovered(&recover_worktree_copy_transactions(common_dir)?, stderr);
}

/// Recovery controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_recovery_tests.rs"]
mod tests;
