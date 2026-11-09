//! What:
//!  The durable claim on a stage container's name,
//! written before the container exists,
//!  and the cleanup of containers whose copy was interrupted before its journal was written.
//! Why:
//!  Staging a worktree's ignored state takes minutes and gigabytes,
//! and the journal is written only after it.
//!  A process killed in between (a signal,
//! the out-of-memory killer,
//! a power loss) leaves a stage nothing names.
//!  The claim names it first,
//! so the next recovery pass,
//! which skips every claim whose owner still runs,
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

use crate::add_timing::{begin_step, end_step};
use crate::path_bytes_in_records::{hex_of_path, text_in_one_line};
use crate::private_storage::sync_directory;
use crate::worktree_add_binding::{Binding, planned_record};
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure, raw_failure_of};
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner};
use crate::worktree_copy_journal::{ensure_journal_root, journal_root, write_atomic, write_atomic_unsynced};
use crate::worktree_copy_journal_removal::remove_file_if_present;
use crate::worktree_copy_selection::{STAGE_PREFIX, by_js_order};
use crate::worktree_copy_stage::new_stage_container;
use std::path::{Path, PathBuf};

/// What:
///  The suffix of a stage claim file.
/// Why:
///   Journals end in `.json`;
/// a different suffix keeps a claim from ever being read as a journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CLAIM_SUFFIX = '.stage';
/// ```
const CLAIM_SUFFIX: &str = ".stage";

/// What:
///  One recorded claim on a stage container's name.
/// Why:
///   Releasing it tells recovery that a journal now names the stage,
/// or that the stage is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StageClaim = { container: string; path: string; release(): Promise<void> };
/// ```
#[derive(Debug)]
pub struct StageClaim {
    /// The claim file under the journal root.
    path: PathBuf,
    /// The stage container the claim names;
    /// it does not exist until staging creates it.
    container: PathBuf,
    /// The record of the worktree an interrupted add left without a checkout,
    /// when the claim covers one.
    destination_record: Option<PathBuf>,
}

impl StageClaim {
    /// What:
    ///  The random name this claim was made under,
    /// which no other command of this process shares.
    /// Why:
    ///   The registration one command created is told apart from another command's
    /// by this name,
    /// because two commands
    ///  of one process have one process token between them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// claim.identifier(); // the file stem, e.g. '0247e85a-...'
    /// ```
    pub fn identifier(&self) -> String {
        let Some(stem) = self.path.file_stem() else { return String::new() };
        return stem.to_string_lossy().into_owned();
    }

    /// What:
    ///  The stage container this claim names.
    /// Why:
    ///   Staging creates exactly this directory,
    /// so the claim and the stage cannot disagree.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// claim.container
    /// ```
    pub fn container(&self) -> &Path {
        return self.container.as_path();
    }

    /// What:
    ///  Records durably that the worktree add this claim covers is complete.
    /// Why:
    ///   Recovery removes a registered worktree whose add has no such record,
    /// because the add never returned success;
    /// the
    ///  marker is written before the person is told of the result,
    /// so a worktree they were told about is never removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.markComplete();
    /// ```
    pub fn mark_complete(&self) -> Result<(), WorktreeCopyError> {
        let file_entered: bool = begin_step("markCompleteFile");
        let marked: Result<(), WorktreeCopyError> = write_atomic(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path(), b"complete", "completion marker");
        end_step(file_entered);
        marked?;
        if let Some(parent) = self.path.parent() {
            let directory_entered: bool = begin_step("markCompleteDirectory");
            let synced: std::io::Result<()> = sync_directory(parent);
            end_step(directory_entered);
            synced.map_err(raw_failure_of)?;
        }
        return Ok(());
    }

    /// What:
    ///  Records that the worktree add this claim covers is complete,
    /// published whole by a rename and flushed by nobody.
    /// Why:
    ///   Used after a subvolume was swapped into place:
    /// the swap makes the next flush in its btrfs transaction a commit of the
    ///  whole filesystem,
    /// which waited eighteen seconds behind other writers' data.
    /// The marker only silences the notice a
    ///  later command gives for an add it finds without one (recovery changes nothing),
    /// and the filesystem's own commit makes it
    ///  durable.
    /// It can never be durable before the swap:
    /// both belong to one transaction,
    /// or the marker to a later one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.markCompleteUnsynced();
    /// ```
    pub fn mark_complete_unsynced(&self) -> Result<(), WorktreeCopyError> {
        let file_entered: bool = begin_step("markCompleteFile");
        let marked: Result<(), WorktreeCopyError> = write_atomic_unsynced(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path(), b"complete", "completion marker");
        end_step(file_entered);
        return marked;
    }

    /// What:
    ///  Removes the claim and its records without flushing the removals.
    /// Why:
    ///   The same reason as [`StageClaim::mark_complete_unsynced`].
    /// Records that return after a crash only make the next command
    ///  settle them:
    /// a marker present means no notice,
    /// and no record means nothing to say.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.releaseUnsynced();
    /// ```
    pub fn release_unsynced(self) -> Result<(), WorktreeCopyError> {
        // The same order as the durable release: the record before the marker, the marker before the claim.
        if let Some(record) = self.destination_record.as_deref() {
            remove_file_if_present(record)?;
        }
        remove_file_if_present(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
        remove_file_if_present(self.path.as_path())?;
        return Ok(());
    }

    /// What:
    ///  Replaces the add's record with the binding to the directories Git made.
    /// Why:
    ///   Recovery deletes only objects whose identities the record names;
    /// until this write,
    /// nothing is known to be the add's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.recordBinding(binding);
    /// ```
    pub fn record_binding(&self, binding: &Binding) -> Result<(), WorktreeCopyError> {
        let Some(record) = self.destination_record.as_deref() else { return Ok(()) };
        write_atomic(record, binding.encode().as_bytes(), "destination record")?;
        if let Some(parent) = self.path.parent() {
            sync_directory(parent).map_err(raw_failure_of)?;
        }
        return Ok(());
    }

    /// What:
    ///  Removes the claim and its records,
    /// telling recovery first that the add the claim covers is complete.
    /// Why:
    ///   A command that ends its copy without filling the destination still made a whole worktree,
    /// so a kill between the two removals must not leave a record that reads as an interrupted add.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.settle();
    /// ```
    pub fn settle(self) -> Result<(), WorktreeCopyError> {
        self.mark_complete()?;
        return self.release();
    }

    /// What:
    ///  Removes the claim file and makes the removal durable.
    /// Why:
    ///   Once a journal names the stage,
    /// or the stage is removed,
    /// the claim must not outlive it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.release();
    /// ```
    pub fn release(self) -> Result<(), WorktreeCopyError> {
        // The record goes before the marker: a kill in between leaves a marker with no record, never a record with no marker.
        if let Some(record) = self.destination_record.as_deref() {
            remove_file_if_present(record)?;
        }
        remove_file_if_present(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
        remove_file_if_present(self.path.as_path())?;
        if let Some(parent) = self.path.parent() {
            sync_directory(parent).map_err(raw_failure_of)?;
        }
        return Ok(());
    }
}

/// What:
///  Chooses the stage container for one destination and records the choice durably,
/// creating no container.
/// Why:
///   The record exists before any staged byte does,
/// so no interruption can leave a stage without a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = await claimStage({ commonDir, destinationRoot });
/// ```
pub fn claim_stage(
    common_dir: &Path,
    destination_root: &Path,
) -> Result<StageClaim, WorktreeCopyError> {
    return claim_stage_for(common_dir, destination_root, &current_copy_owner()?);
}

/// The line of a claim's text that names its owner,
/// after the line that names the container.
pub(crate) const OWNER_LINE_PREFIX: &str = "owner=";

/// What:
///  The line of a claim's text that holds the container path's bytes.
/// Why:
///   The first line stays the path itself when the path is text and fits one line,
/// because the release installed before this line reads it there;
/// a path that is text cannot spell another one on that line,
/// so both readers name the same directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONTAINER_HEX_PREFIX = 'containerHex=';
/// ```
pub(crate) const CONTAINER_HEX_PREFIX: &str = "containerHex=";

/// What:
///  The text of a claim:
/// the container's text on the first line when it can hold it,
/// its bytes on a line of its own,
/// and the owner last.
/// Why:
///   The bytes line is the whole path whatever bytes it holds,
/// and a container whose text needs replacement characters or a line break leaves the first line out,
/// so a reader that knows only the first line refuses the record instead of naming the replacement spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${textInOneLine(container) ?? ''}\n${CONTAINER_HEX_PREFIX}${hexOfPath(container)}\n${OWNER_LINE_PREFIX}${owner.token()}\n`
/// ```
fn claim_text(container: &Path, owner: &CopyOwner) -> String {
    let mut text: String = String::new();
    if let Some(line) = text_in_one_line(container) {
        text.push_str(line);
        text.push('\n');
    }
    text.push_str(format!("{CONTAINER_HEX_PREFIX}{}", hex_of_path(container)).as_str());
    text.push('\n');
    text.push_str(format!("{OWNER_LINE_PREFIX}{}", owner.token()).as_str());
    text.push('\n');
    return text;
}

/// What:
///  Claims a stage name for a named owner:
/// the claim file holds the container path and,
/// on a line of its own,
/// the owner.
/// Why:
///   A claim of a live owner is an add or a copy still running;
/// the reaper leaves it alone and removes only a dead owner's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await claimStageFor({ commonDir, destinationRoot, owner });
/// ```
pub fn claim_stage_for(
    common_dir: &Path,
    destination_root: &Path,
    owner: &CopyOwner,
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
    let text: String = claim_text(container.as_path(), owner);
    write_atomic(path.as_path(), text.as_bytes(), "stage claim")?;
    return Ok(StageClaim { path, container, destination_record: None });
}

pub(crate) const DESTINATION_SUFFIX: &str = ".add";

/// What:
///  Claims the stage name of a creating command and records the destination it is about to make,
/// or claims nothing when that command will not copy into it.
/// Why:
///   The record comes before real Git runs,
/// so a command killed after Git returned still leaves the destination and the stage name it chose,
/// and the next command names the worktree to the person.
/// Without the record,
/// a worktree registered by a killed add is in every later command's
/// before-set and is never filled and never reported.
/// A command that copies nothing (a bare repository,
/// a main checkout with nothing to hand over,
/// a line whose destination is not read) records nothing,
/// so no record is left that nothing will settle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = copies && destination ? await claimStageForAdd({ commonDir, destinationRoot }) : undefined;
/// ```
pub fn claim_stage_for_creation(common_dir: &Path, destination: Option<&Path>, copies: bool) -> Result<Option<StageClaim>, WorktreeCopyError> {
    let (true, Some(destination_root)) = (copies, destination) else { return Ok(None) };
    return claim_stage_for_add(common_dir, destination_root).map(Some);
}

/// The suffix of the marker that says the worktree add the claim covers is complete.
pub(crate) const COMPLETE_SUFFIX: &str = ".done";

/// What:
///  Claims a stage name as `claim_stage` does and also records the destination of the `git worktree add` it serves.
/// Why:
///   Real Git registers the worktree before the stage is swapped in,
/// so a process killed in between leaves a registered
///  worktree with no checkout;
/// the next recovery names it to the person with the commands that finish or discard it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = await claimStageForAdd({ commonDir, destinationRoot });
/// ```
pub fn claim_stage_for_add(common_dir: &Path, destination_root: &Path) -> Result<StageClaim, WorktreeCopyError> {
    return claim_stage_for_add_as(common_dir, destination_root, &current_copy_owner()?);
}

/// What:
///  Claims a stage name for an add on behalf of a named owner.
/// Why:
///   The add's records stay whole while their owner runs and are settled once it is dead;
/// a control names a dead owner to
///  stand for an add that was killed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = await claimStageForAddAs({ commonDir, destinationRoot, owner });
/// ```
pub fn claim_stage_for_add_as(common_dir: &Path, destination_root: &Path, owner: &CopyOwner) -> Result<StageClaim, WorktreeCopyError> {
    let mut claim: StageClaim = claim_stage_for(common_dir, destination_root, owner)?;
    let record: PathBuf = claim.path.with_extension(&DESTINATION_SUFFIX[1..]);
    write_atomic(record.as_path(), planned_record(destination_root).as_bytes(), "destination record")?;
    claim.destination_record = Some(record);
    return Ok(claim);
}

/// What:
///  The names of the stage claim files under a journal root,
/// in journal-name order.
/// Why:
///   The pending check and the cleanup read the same set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(root)).filter((name) => name.endsWith('.stage')).sort(jsOrder)
/// ```
pub(crate) fn claim_names(root: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    // `match` treats an absent root as having no claims, and every other failure as fatal.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed
            .map_err(raw_failure_of)?
            .file_name()
            .to_string_lossy()
            .into_owned();
        if name.ends_with(CLAIM_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(by_js_order);
    return Ok(names);
}

/// What:
///  Whether any stage claim is recorded for a repository.
/// Why:
///   A forwarded command looks for recovery work only when something is pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readStageClaimNames(commonDir)).length > 0
/// ```
pub fn stage_claims_exist(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    return Ok(!claim_names(journal_root(common_dir).as_path())?.is_empty());
}

/// Claim controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_claim_tests.rs"]
mod tests;
