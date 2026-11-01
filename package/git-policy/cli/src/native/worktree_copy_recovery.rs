//! What:
//!  Recovery of interrupted linked-worktree copies: every pending journal is resumed, completed, or discarded with a
//!  notice; malformed state fails closed and stays; and the finish of a copy, whose order makes a finished copy
//!  impossible to forget.
//! Why:
//!  A copy interrupted by a crash must not leave a worktree half-filled forever, but recovery may only act on
//!  journals whose destination is still the registration they were written for and whose stage is still private.
//!  The resident service is the only process that runs copies, so a journal on disk is the work of a copy that was
//!  killed, and no liveness has to be proven.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = await recoverWorktreeCopyTransactions(commonDir);
//! await recoverPendingWorktreeCopies(commonDir);
//! ```

use super::json_record::quote;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_registry::{copied_for, registration_token, write_copied_marker};
use super::worktree_copy_stage::StagedSnapshot;
use super::worktree_copy_transaction::{Outcome, install_recorded, snapshot_from_journal};
use crate::worktree_copy_journal::{
    Journal, PendingJournal, Phase, Role, assert_private_path, read_pending_journals, remove_pending_journal,
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
    return read_registration_named(common_dir, destination, None);
}

/// What:
///  The raw registration check that also requires the pointer to name one administrative directory.
/// Why:
///   A destination path is not a registration: the pointer file names the administrative directory it belongs to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readRegistrationNamed({ commonDir, destinationRoot, adminId }): Promise<DestinationRegistration>;
/// ```
fn read_registration_named(
    common_dir: &Path,
    destination: &Path,
    admin_id: Option<&str>,
) -> std::io::Result<Registration> {
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
    let named: bool = admin_id.is_none_or(|expected_name| {
        return admin.file_name().is_some_and(|name| return name.to_string_lossy() == expected_name);
    });
    if admin.parent() == Some(expected.as_path()) && named {
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
///  The points of a finished installation a test can stop at, as a kill would: the files are installed, and the
///  registration is marked.
/// Why:
///   The order of the last three steps (install, mark, remove the journal) is what makes a finished copy impossible to
///  forget, and only a stop at each point shows that recovery completes from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Step = 'installed' | 'marked';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Step {
    /// The entries are installed; nothing records that the copy is over.
    Installed,
    /// The registration's marker is durable; the journal and stage still exist.
    Marked,
}

/// What a finished installation came to.
#[derive(Debug)]
pub enum Finished {
    /// The installation ended, installed or failed, and its journal and stage are gone.
    Done(Outcome),
    /// A checkpoint function asked to stop, so the state stands as a kill at that point leaves it.
    Interrupted,
}

/// What:
///  Installs a journal's stage, marks the registration copied, and only then removes the journal and the stage.
/// Why:
///   The marker is durable before the journal goes, so a kill at any point leaves either a journal that recovery
///  resumes (and finds every path already installed) or a marker that tells recovery the copy is over. A failed
///  installation is not marked: it rolled back its own paths, and a later run may try again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const finished = await finishJournal({ commonDir, pending, snapshot, checkpoint });
/// ```
pub fn finish_journal(
    common_dir: &Path,
    pending: PendingJournal,
    snapshot: &dyn Fn(&BTreeSet<String>) -> Result<StagedSnapshot, WorktreeCopyError>,
    checkpoint: &mut dyn FnMut(Step) -> bool,
) -> Result<Finished, WorktreeCopyError> {
    let (latest, outcome) = install_recorded(pending, snapshot)?;
    if !checkpoint(Step::Installed) {
        return Ok(Finished::Interrupted);
    }
    if let (Outcome::Installed(_), Some(admin_id), Some(token)) =
        (&outcome, latest.journal.admin_id.as_deref(), latest.journal.registration.as_deref())
    {
        write_copied_marker(common_dir.join("worktrees").as_path(), admin_id, token)?;
    }
    if !checkpoint(Step::Marked) {
        return Ok(Finished::Interrupted);
    }
    remove_pending_journal(&latest)?;
    return Ok(Finished::Done(outcome));
}

/// What:
///  Whether a journal's destination is still the registration the journal was written for.
/// Why:
///   A journal that names its registration is judged by that registration's identity, not by the destination's path: a
///  worktree removed and added again at the same path or under the same name is another worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await journalRegistration({ commonDir, journal })
/// ```
fn journal_registration(common_dir: &Path, journal: &Journal) -> Result<Registration, WorktreeCopyError> {
    let destination: &Path = Path::new(journal.destination_root.as_str());
    let (Some(admin_id), Some(token)) = (journal.admin_id.as_deref(), journal.registration.as_deref()) else {
        return destination_registration(common_dir, destination);
    };
    return registration_of(common_dir, destination, admin_id, token);
}

/// What:
///  Whether a destination is the registration with this name and identity under the common directory.
/// Why:
///   Name, path and identity must all agree: the administrative directory still carries the token, the destination's
///  pointer file names that directory, and that directory is under this repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await registrationOf({ commonDir, destination, adminId, token })
/// ```
pub fn registration_of(
    common_dir: &Path,
    destination: &Path,
    admin_id: &str,
    token: &str,
) -> Result<Registration, WorktreeCopyError> {
    if registration_token(common_dir.join("worktrees").as_path(), admin_id)?.as_deref() != Some(token) {
        return Ok(Registration::Unregistered);
    }
    // `match` on the pointer: a missing path is an answer, anything else is a fault to report.
    let named: std::io::Result<Registration> = read_registration_named(common_dir, destination, Some(admin_id));
    match named {
        Ok(registration) => return Ok(registration),
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound
                || error.kind() == std::io::ErrorKind::NotADirectory =>
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
    let registration: Registration = journal_registration(common_dir, &pending.journal)?;
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
    // A journal whose registration is marked finished its copy: only the cleanup after the marker is left.
    let marked: bool = match (pending.journal.admin_id.as_deref(), pending.journal.registration.as_deref()) {
        (Some(admin_id), Some(token)) => copied_for(common_dir.join("worktrees").as_path(), admin_id, token)?,
        _ => false,
    };
    if pending.journal.phase == Phase::Complete || marked {
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
    match finish_journal(common_dir, pending, &snapshot, &mut |_step| return true)? {
        Finished::Done(Outcome::Installed(_)) | Finished::Interrupted => return Ok((true, None)),
        Finished::Done(Outcome::Ended(failure)) => {
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
///  Recovers every pending journal of one repository, in journal order.
/// Why:
///   The resident service is the only process that runs copies, so a journal on disk is never one a live copy is writing:
///  it is the work of a copy the service was killed in, and nothing has to be proven dead first. A journal that cannot
///  be read or proven stops the pass, so its evidence is never acted on blindly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverWorktreeCopyTransactions(commonDir);
/// ```
pub fn recover_worktree_copy_transactions(
    common_dir: &Path,
) -> Result<RecoveryReport, WorktreeCopyError> {
    let pending: Vec<PendingJournal> = read_pending_journals(common_dir)?;
    let mut recovered: usize = 0;
    let mut notices: Vec<String> = Vec::new();
    for journal in pending {
        let (completed, notice) = recover_transaction(common_dir, journal)?;
        if completed {
            recovered += 1;
        }
        if let Some(line) = notice {
            notices.push(line);
        }
    }
    return Ok(RecoveryReport { recovered, notices });
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

/// Recovery controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_recovery_tests.rs"]
mod tests;
