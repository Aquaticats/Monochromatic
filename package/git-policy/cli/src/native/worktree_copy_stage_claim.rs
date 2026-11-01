//! What:
//!  The durable claim on a stage container's name, written before the container exists,
//!  and the cleanup of containers whose copy was interrupted before its journal was written.
//! Why:
//!  Staging a worktree's ignored state takes minutes and gigabytes, and the journal is written only after it.
//!  A process killed in between (a signal, the out-of-memory killer, a power loss) leaves a stage nothing names.
//!  The claim names it first, so the next recovery pass, which runs only when no live copy holds the settlement lock,
//!  removes it with a notice instead of leaving it to fill the disk.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = await claimStage({ commonDir, destinationRoot });
//! await stageIgnoredSnapshot({ stageContainer: claim.container, ... });
//! await createWorktreeCopyJournal(...);
//! await claim.release();
//! const notices = await reapAbandonedStages(commonDir);
//! ```

use super::json_record::quote;
use super::private_storage::sync_directory;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_journal::{
    Role, TEMPORARY_SUFFIX, assert_private_path, ensure_journal_root, journal_root,
    read_pending_journals, remove_file_if_present, remove_stage, write_atomic,
};
use super::worktree_copy_selection::{STAGE_PREFIX, js_order};
use super::worktree_copy_stage::new_stage_container;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What:
///  The suffix of a stage claim file.
/// Why:
///   Journals end in `.json`; a different suffix keeps a claim from ever being read as a journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CLAIM_SUFFIX = '.stage';
/// ```
const CLAIM_SUFFIX: &str = ".stage";

/// What:
///  One recorded claim on a stage container's name.
/// Why:
///   Releasing it tells recovery that a journal now names the stage, or that the stage is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StageClaim = { container: string; path: string; release(): Promise<void> };
/// ```
#[derive(Debug)]
pub struct StageClaim {
    /// The claim file under the journal root.
    path: PathBuf,
    /// The stage container the claim names; it does not exist until staging creates it.
    container: PathBuf,
}

impl StageClaim {
    /// What:
    ///  The stage container this claim names.
    /// Why:
    ///   Staging creates exactly this directory, so the claim and the stage cannot disagree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// claim.container
    /// ```
    pub fn container(&self) -> &Path {
        return self.container.as_path();
    }

    /// What:
    ///  Removes the claim file and makes the removal durable.
    /// Why:
    ///   Once a journal names the stage, or the stage is removed, the claim must not outlive it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.release();
    /// ```
    pub fn release(self) -> Result<(), WorktreeCopyError> {
        remove_file_if_present(self.path.as_path())?;
        if let Some(parent) = self.path.parent() {
            sync_directory(parent).map_err(|error| return raw_failure(&error))?;
        }
        return Ok(());
    }
}

/// What:
///  Chooses the stage container for one destination and records the choice durably, creating no container.
/// Why:
///   The record exists before any staged byte does, so no interruption can leave a stage without a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = await claimStage({ commonDir, destinationRoot });
/// ```
pub fn claim_stage(
    common_dir: &Path,
    destination_root: &Path,
) -> Result<StageClaim, WorktreeCopyError> {
    let root: PathBuf = ensure_journal_root(common_dir)?;
    let container: PathBuf = new_stage_container(destination_root)?;
    let identifier: String = container
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .trim_start_matches(STAGE_PREFIX)
        .to_string();
    let path: PathBuf = root.join(format!("{identifier}{CLAIM_SUFFIX}"));
    write_atomic(
        path.as_path(),
        container.to_string_lossy().as_bytes(),
        "stage claim",
    )?;
    return Ok(StageClaim { path, container });
}

/// What:
///  The names of the stage claim files under a journal root, in journal-name order.
/// Why:
///   The pending check and the cleanup read the same set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(root)).filter((name) => name.endsWith('.stage')).sort(jsOrder)
/// ```
fn claim_names(root: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    // `match` treats an absent root as having no claims, and every other failure as fatal.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed
            .map_err(|error| return raw_failure(&error))?
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.ends_with(CLAIM_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(|left, right| return js_order(left, right));
    return Ok(names);
}

/// What:
///  Whether any stage claim is recorded for a repository.
/// Why:
///   A forwarded command takes the settlement lock for recovery only when something is pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readStageClaimNames(commonDir)).length > 0
/// ```
pub fn stage_claims_exist(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    return Ok(!claim_names(journal_root(common_dir).as_path())?.is_empty());
}

/// What:
///  The stage container one claim file names, refused unless it is a cli-git stage container name.
/// Why:
///   The container is deleted recursively, so a claim that names anything else is never acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const container = await readFile(claimPath, 'utf8'); assertStageContainerName(container);
/// ```
fn claimed_container(claim: &Path) -> Result<PathBuf, WorktreeCopyError> {
    assert_private_path(claim, Role::JournalFile)?;
    let text: String = std::fs::read_to_string(claim).map_err(|error| {
        let message: String = format!(
            "cli-git: could not read worktree-copy stage claim {}.",
            quote(claim.to_string_lossy().as_ref())
        );
        return failure_with_cause(message.as_str(), Some(&error));
    })?;
    let container: PathBuf = PathBuf::from(text);
    let named: bool = container.is_absolute()
        && container
            .file_name()
            .is_some_and(|name| return name.to_string_lossy().starts_with(STAGE_PREFIX));
    if !named {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy stage claim {} does not name a stage container; remove the file after checking it.",
                quote(claim.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    return Ok(container);
}

/// What:
///  Removes the temporaries an interrupted claim write left beside the claims.
/// Why:
///   A write cut off before its rename leaves a file nothing else names, and no container exists for it yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(root)) if (name.includes('.stage.') && name.endsWith('.tmp')) await rm(join(root, name), { force: true });
/// ```
fn remove_unfinished_claims(root: &Path) -> Result<(), WorktreeCopyError> {
    let infix: String = format!("{CLAIM_SUFFIX}.");
    // `match` treats an absent root as having nothing to remove.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    };
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if name.contains(infix.as_str()) && name.ends_with(TEMPORARY_SUFFIX) {
            remove_file_if_present(entry.path().as_path())?;
        }
    }
    return Ok(());
}

/// What:
///  Accepts a finished action, or one that found its directory absent, and reports every other failure.
/// Why:
///   A root that was never created holds nothing to make durable, while any other refusal means the durable state is
///  unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await action(); } catch (e) { if (e.code !== 'ENOENT') throw e; }
/// ```
fn absent_is_fine(outcome: std::io::Result<()>) -> Result<(), WorktreeCopyError> {
    // `match` on the failure kind: only absence is accepted.
    match outcome {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  Removes every stage container that a claim names and no journal does, then the claims, and says what it removed.
/// Why:
///   The caller holds the settlement lock, so no live copy is between claiming and journaling;
///  a claimed container no journal names was abandoned by a process that died.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await reapAbandonedStages(commonDir);
/// ```
pub fn reap_abandoned_stages(common_dir: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    let names: Vec<String> = claim_names(root.as_path())?;
    let mut notices: Vec<String> = Vec::new();
    if !names.is_empty() {
        let journaled: BTreeSet<PathBuf> = read_pending_journals(common_dir)?
            .into_iter()
            .map(|pending| return PathBuf::from(pending.journal.stage_container))
            .collect();
        for name in names {
            let claim: PathBuf = root.join(name);
            let container: PathBuf = claimed_container(claim.as_path())?;
            let present: bool = match std::fs::symlink_metadata(container.as_path()) {
                Ok(_) => true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => return Err(raw_failure(&error)),
            };
            if present && !journaled.contains(&container) {
                assert_private_path(container.as_path(), Role::PrivateStage)?;
                remove_stage(container.as_path())?;
                notices.push(format!(
                    "cli-git: removed the stage {} that an interrupted ignored-state copy left behind.",
                    quote(container.to_string_lossy().as_ref())
                ));
            }
            remove_file_if_present(claim.as_path())?;
        }
    }
    remove_unfinished_claims(root.as_path())?;
    absent_is_fine(sync_directory(root.as_path()))?;
    return Ok(notices);
}

/// Claim controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_claim_tests.rs"]
mod tests;
