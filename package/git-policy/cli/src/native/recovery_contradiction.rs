//! What:
//!  The check that a dead transaction without a usable landing record did not move its branch.
//! Why:
//!  Recovery treats a transaction with no `landing-<n>.json` (or no `preparing.json`) as one that stopped before it
//!  landed, and removes it. The journal writes `preparing.json` first, `landing-<n>.json` before the branch moves and
//!  `ref-updated.json` after it, each synced before the next, so a directory that holds a later record without an
//!  earlier one, or a branch reflog entry of the transaction without a landing record, is a record set that
//!  contradicts itself. Removing it would leave the branch ahead of an index nobody reconciled and report success.
//!  Recovery cannot repair that state (the landing record held the post-index identity, its digest and the lock
//!  identity), so it fails closed, names the evidence and leaves the directory.
//!  An `index-installed` marker means the install finished, so nothing is left to reconcile and the directory goes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (!names.includes('index-installed') && movedBranchEvidence) throw new CommitTransactionRecoveryError(message);
//! ```

/// Debug diagnostics.
use super::diagnostic_log::debug;
/// The nonce-bearing reflog subject prefix.
use super::recovery_completion::nonce_subject_prefix;
/// The fail-closed recovery failure.
use super::recovery_error::RecoveryError;
/// Attempt evidence in a directory's names.
use super::recovery_evidence::{LANDING_RECORD_PREFIX, attempt_numbers};
/// The steps that finish a retained transaction by hand.
use super::recovery_manual_steps::finish_by_hand_steps;
/// The reflog nonce search.
use super::recovery_reflog::list_nonce_reflog_oids;
/// Running real Git.
use super::transaction_git::GitContext;
/// Record names.
use super::transaction_journal::{INDEX_INSTALLED_FILENAME, PREPARING_FILENAME, PreparingRecord, REF_UPDATED_FILENAME};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Fail closed when the records of a transaction without a usable landing record show that its branch moved.
///  Does nothing for a directory that shows no such evidence.
/// Why:
///   See the module comment.
///  Without `preparing.json` the target reference is unknown, so only the files of the directory count; with it,
///  the reflog of the target is searched for the transaction's own entry when no file shows the move, which also
///  finds a move whose `ref-updated.json` was lost together with the landing record. A reflog that cannot be read
///  is not evidence: the transaction is then treated as before, and a repository with reflogs disabled cannot be
///  checked this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await refuseUnreconciledLanding({ context, cwd, directory, names, preparing });
/// ```
pub fn refuse_unreconciled_landing(
    context: &GitContext,
    cwd: &Path,
    directory: &Path,
    names: &[String],
    preparing: Option<&PreparingRecord>,
) -> Result<(), RecoveryError> {
    if names.iter().any(|name| { return name == INDEX_INSTALLED_FILENAME; }) {
        return Ok(());
    }
    let landing_files: Vec<i64> = attempt_numbers(names, LANDING_RECORD_PREFIX);
    let marker_present: bool = names.iter().any(|name| { return name == REF_UPDATED_FILENAME; });
    let Some(record) = preparing else {
        if landing_files.is_empty() && !marker_present {
            return Ok(());
        }
        let present: String = present_records(landing_files.as_slice(), marker_present);
        return Err(RecoveryError(format!(
            "Transaction directory {} has {present} but no {PREPARING_FILENAME}. The journal writes \
             {PREPARING_FILENAME} first, so recovery cannot tell which branch the transaction moved or what the real \
             index should hold, and it changed nothing. Recovery retained the transaction at {}. {}",
            directory.display(),
            directory.display(),
            finish_by_hand_steps(directory, None)
        )));
    };
    if !landing_files.is_empty() {
        return Ok(());
    }
    if marker_present {
        return Err(RecoveryError(format!(
            "Transaction directory {} has {REF_UPDATED_FILENAME} but no {LANDING_RECORD_PREFIX}<n>.json, so the commit \
             of this transaction is already on {} but the record that names the post-index to install is missing. \
             Recovery changed neither the branch nor the real index {}. Recovery retained the transaction at {}. {}",
            directory.display(),
            record.target_ref,
            record.real_index_path,
            directory.display(),
            finish_by_hand_steps(directory, Some(record))
        )));
    }
    let found: Vec<String> = match list_nonce_reflog_oids(
        context,
        cwd,
        record.target_ref.as_str(),
        nonce_subject_prefix(record.transaction_id.as_str()).as_str(),
    ) {
        Ok(oids) => oids,
        Err(error) => {
            debug("refuseUnreconciledLanding", format!("reflog not usable as evidence: {error}").as_str());
            return Ok(());
        }
    };
    if found.is_empty() {
        return Ok(());
    }
    return Err(RecoveryError(format!(
        "The reflog of {} has an entry of transaction {} naming {} but its directory {} has no \
         {LANDING_RECORD_PREFIX}<n>.json and no {REF_UPDATED_FILENAME}, so the commit is already on the branch but the \
         record that names the post-index to install is missing. Recovery changed neither the branch nor the real \
         index {}. Recovery retained the transaction at {}. {}",
        record.target_ref,
        record.transaction_id,
        found.join(", "),
        directory.display(),
        record.real_index_path,
        directory.display(),
        finish_by_hand_steps(directory, Some(record))
    )));
}

/// What:
///  The journal records a directory holds, as the words a message uses.
/// Why:
///   The message names what was found, not only what is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// present(landingFiles, markerPresent) // 'landing-1.json and ref-updated.json'
/// ```
fn present_records(landing_files: &[i64], marker_present: bool) -> String {
    let mut names: Vec<String> = landing_files
        .iter()
        .map(|attempt| { return format!("{LANDING_RECORD_PREFIX}{attempt}.json"); })
        .collect();
    if marker_present {
        names.push(String::from(REF_UPDATED_FILENAME));
    }
    return names.join(" and ");
}

/// Unreconciled-landing controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_contradiction_tests.rs"]
mod tests;
