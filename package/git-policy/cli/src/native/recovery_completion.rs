//! What:
//!  Deciding whether a dead transaction's landing attempt landed,
//!  and installing or
//!       recognizing its recorded post-index.
//! Why:
//!  Before `ref-updated.json` the only proof of a landing is the nonce entry in the target's
//!      reflog;
//!  missing or contradictory evidence fails closed.
//!  Artifacts are pinned under
//!      stable names by inode before use (`src/policy-engine/commit-transaction-recovery-completion.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (await commitLanded({ ... })) await completeIndex({ directory, preparing, landing });
//! ```

/// The fail-closed recovery failure.
use super::recovery_error::RecoveryError;
/// Exact reads,
///  owned links,
///  lock release and index installation.
use super::recovery_files::{
    create_owned_file_link, files_equal, install_recovered_index, read_recovery_file,
    recovery_path_exists, release_owned_lock, remove_file_if_present,
};
/// The reflog nonce search.
use super::recovery_reflog::list_nonce_reflog_oids;
/// The check of the snapshots' bytes.
use super::recovery_snapshot::{SnapshotRole, verify_snapshot};
/// Running real Git.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// Journal records and artifact names.
use super::transaction_journal::{
    FileIdentity, INDEX_INSTALLED_FILENAME, LandingRecord, PreparingRecord, REF_UPDATED_FILENAME,
    post_index_filename, pre_landing_index_filename,
};
/// The `ref-updated.json` parser.
use super::transaction_journal_parse::parse_ref_updated;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  What recovery did to the real index of a landed commit.
/// Why:
///   Reported per transaction,
///  as the incumbent's recovery action names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexCompletion = 'index-installed' | 'already-installed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexCompletion {
    /// The recorded post-index was installed now.
    Installed,
    /// The post-index was installed before the owner died.
    AlreadyInstalled,
}

/// What:
///  The nonce-bearing reflog subject prefix of a transaction.
/// Why:
///   The landing writes `commit (cli-git <nonce>): <subject>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `commit (cli-git ${transactionId}):`
/// ```
pub fn nonce_subject_prefix(transaction_id: &str) -> String {
    return format!("commit (cli-git {transaction_id}):");
}

/// What:
///  Decide whether a commit landing record's attempt landed.
///       `Ok(true)` means landed;
///  `Ok(false)` means the target never moved to it.
/// Why:
///   `commitLanded`:
///  `ref-updated.json` is exact proof;
///  otherwise one nonce entry naming the
///       recorded commit is proof,
///  several are contradictory,
///  one naming another commit is
///       contradictory,
///  and none while the commit is reachable anyway is contradictory too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitLanded({ gitPath, cwd, directory, preparing, landing }): Promise<boolean>;
/// ```
pub fn commit_landed(
    context: &GitContext,
    cwd: &Path,
    directory: &Path,
    preparing: &PreparingRecord,
    landing: &LandingRecord,
) -> Result<bool, RecoveryError> {
    let marker: PathBuf = directory.join(REF_UPDATED_FILENAME);
    let new_oid: &str = landing.new_oid.as_deref().unwrap_or("undefined");
    if recovery_path_exists(marker.as_path())? {
        let landed: String = parse_ref_updated(read_recovery_file(marker.as_path())?.as_slice())?;
        if landing.new_oid.as_deref() != Some(landed.as_str()) {
            return Err(RecoveryError(format!(
                "Landed marker names another commit than the landing record: {}",
                directory.display()
            )));
        }
        return Ok(true);
    }
    let oids: Vec<String> = list_nonce_reflog_oids(
        context,
        cwd,
        preparing.target_ref.as_str(),
        nonce_subject_prefix(preparing.transaction_id.as_str()).as_str(),
    )?;
    if oids.len() > 1 {
        return Err(RecoveryError(format!(
            "{} reflog names several commits for transaction {}; recovery retained at {}",
            preparing.target_ref,
            preparing.transaction_id,
            directory.display()
        )));
    }
    if let Some(found) = oids.first() {
        if found != new_oid {
            return Err(RecoveryError(format!(
                "{} reflog nonce names {found}, not the recorded {new_oid}; recovery retained at {}",
                preparing.target_ref,
                directory.display()
            )));
        }
        return Ok(true);
    }
    let reachable: bool = match run_git(
        context,
        &GitRequest::new(
            cwd,
            &[
                "merge-base",
                "--is-ancestor",
                new_oid,
                preparing.target_ref.as_str(),
            ],
        ),
    ) {
        Ok(output) => output.succeeded(),
        Err(_) => false,
    };
    if reachable {
        return Err(RecoveryError(format!(
            "{} contains the transaction commit {new_oid} but its reflog lacks the nonce entry; \
             recovery retained at {}",
            preparing.target_ref,
            directory.display()
        )));
    }
    return Ok(false);
}

/// What:
///  Pin a transaction artifact under `<name>.recovery` after proving its recorded identity.
/// Why:
///   `stabilizeArtifact`:
///  the stable name keeps the exact inode even if the original name
///       is replaced meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stabilizeArtifact({ directory, filename, identity }): Promise<string>;
/// ```
fn stabilize_artifact(
    directory: &Path,
    filename: &str,
    identity: &FileIdentity,
) -> Result<PathBuf, RecoveryError> {
    let stable: PathBuf = directory.join(format!("{filename}.recovery"));
    remove_file_if_present(stable.as_path())?;
    create_owned_file_link(
        directory.join(filename).as_path(),
        stable.as_path(),
        identity.device.as_str(),
        identity.inode.as_str(),
    )?;
    return Ok(stable);
}

/// What:
///  Install or recognize the recorded post-index after a landing.
/// Why:
///   `completeIndex`:
///  a present `index-installed` marker or a real index already equal to
///       the post-index means done (the owned lock is released);
///  otherwise the real index must
///       still equal the pre-landing snapshot,
///  or recovery fails closed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function completeIndex({ directory, preparing, landing }): Promise<'index-installed' | 'already-installed'>;
/// ```
pub fn complete_index(
    directory: &Path,
    preparing: &PreparingRecord,
    landing: &LandingRecord,
) -> Result<IndexCompletion, RecoveryError> {
    let pre_landing: PathBuf = stabilize_artifact(
        directory,
        pre_landing_index_filename(landing.attempt).as_str(),
        &landing.pre_landing_index,
    )?;
    let post: PathBuf = stabilize_artifact(
        directory,
        post_index_filename(landing.attempt).as_str(),
        &landing.post_index,
    )?;
    let real_index: &Path = Path::new(preparing.real_index_path.as_str());
    let lock_path: PathBuf = PathBuf::from(format!("{}.lock", preparing.real_index_path));
    if recovery_path_exists(directory.join(INDEX_INSTALLED_FILENAME).as_path())?
        || files_equal(post.as_path(), real_index)?
    {
        release_owned_lock(&landing.lock, lock_path.as_path())?;
        return Ok(IndexCompletion::AlreadyInstalled);
    }
    // Only the install path trusts the snapshots' bytes: a landing whose install already happened needs neither.
    verify_snapshot(
        directory,
        preparing,
        landing,
        pre_landing.as_path(),
        SnapshotRole::PreLanding,
        landing.pre_landing_digest,
    )?;
    verify_snapshot(directory, preparing, landing, post.as_path(), SnapshotRole::Post, landing.post_digest)?;
    if !files_equal(pre_landing.as_path(), real_index)? {
        return Err(RecoveryError(format!(
            "Real index no longer matches the pre-landing snapshot; recovery retained at {}",
            directory.display()
        )));
    }
    install_recovered_index(
        lock_path.as_path(),
        real_index,
        post.as_path(),
        &landing.lock,
    )?;
    return Ok(IndexCompletion::Installed);
}

/// Landed-or-not and index completion controls stay out of the release executable.
#[cfg(test)]
#[path = "recovery_completion_tests.rs"]
mod tests;
