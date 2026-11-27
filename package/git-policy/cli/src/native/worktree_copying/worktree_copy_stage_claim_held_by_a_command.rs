//! What:
//!  The claim one command holds on a stage container's name:
//!  the three files that claim is,
//!  the marking of the add it covers as complete,
//!  the naming of the Git process that add started,
//!  the binding recorded once Git returned,
//!  and the release that removes them.
//! Why:
//!  A claim of a live owner is an add or a copy still running,
//!  and the reaper removes only a dead owner's,
//!  so the claim itself must say which files are its own and must be settled as the copying command would:
//!  this file is every step a command takes on a claim it wrote.
//!  Writing a claim,
//! and reading which claim names a stage,
//!  are in `worktree_copy_stage_claim.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const claim = StageClaim.adopted({ path, container, destinationRecord });
//! await claim.markComplete();
//! await claim.release();
//! ```

use crate::add_timing::{
    begin_step,
    end_step,
};
use crate::private_storage::sync_directory;
use crate::worktree_add_binding::Binding;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure_of,
};
use crate::worktree_copy_journal::{
    write_atomic,
    write_atomic_unsynced,
};
use crate::worktree_copy_journal_removal::remove_file_if_present;
use crate::worktree_copy_stage_made_object::remove_made;
use std::path::{
    Path,
    PathBuf,
};
/// The suffix of the marker that says the worktree add the claim covers is complete.
pub(crate) const COMPLETE_SUFFIX: &str = ".done";

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
    pub(super) path: PathBuf,
    /// The stage container the claim names;
    /// it does not exist until staging creates it.
    pub(super) container: PathBuf,
    /// The record of the worktree an interrupted add left without a checkout,
    /// when the claim covers one.
    pub(super) destination_record: Option<PathBuf>,
}

impl StageClaim {
    /// What:
    ///  A claim this process did not create,
    /// read from the files a dead owner left.
    /// Why:
    ///   Recovery finishes the copy the claim names,
    /// so it needs the claim's own three files to settle them as the copying command would;
    /// nothing here checks the container,
    /// because the caller proves it first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// StageClaim.adopt({ path, container, destinationRecord })
    /// ```
    pub fn adopted(
        path: &Path,
        container: &Path,
        destination_record: Option<&Path>,
    ) -> Option<StageClaim> {
        if path.file_name().is_none() || container.file_name().is_none() {
            return None;
        }
        return Some(StageClaim {
            path: path.to_path_buf(),
            container: container.to_path_buf(),
            destination_record: destination_record.map(Path::to_path_buf),
        });
    }

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
        let Some(stem) = self.path.file_stem() else {
            return String::new();
        };
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
        let marked: Result<(), WorktreeCopyError> = write_atomic(
            self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path(),
            b"complete",
            "completion marker",
        );
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
        let marked: Result<(), WorktreeCopyError> = write_atomic_unsynced(
            self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path(),
            b"complete",
            "completion marker",
        );
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
        // The same order as the durable release: the record of the object made, the record, the marker, then the claim.
        remove_made(self.path.as_path())?;
        if let Some(record) = self.destination_record.as_deref() {
            remove_file_if_present(record)?;
        }
        remove_file_if_present(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
        remove_file_if_present(self.path.as_path())?;
        return Ok(());
    }

    /// What:
    ///  Replaces the add's record with the binding to the directories Git made,
    /// published whole by a rename and flushed by nobody.
    /// Why:
    ///   Recovery deletes only objects whose identities the record names;
    /// until this write,
    /// nothing is known to be the add's.
    /// What it guards is a killed command,
    /// whose writes stay in the page cache,
    /// and native Git flushes nothing for the worktree add it serves;
    /// a flush of it is a commit of the whole btrfs filesystem on the add's waiting path.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.recordBinding(binding);
    /// ```
    /// What:
    ///  Names the Git process this add started in the add's own record,
    /// as soon as the command that started it has its number.
    /// Why:
    ///   The record of an add exists because a copy of the source's ignored state is owed,
    /// and it is written before Git
    ///  runs;
    /// a command killed while its Git is still making the destination leaves a record whose destination is
    ///  not there yet,
    /// and a later command that settles that record would take the worktree Git is about to
    ///  finish for one nothing made.
    /// The name is written only while the record is still planned:
    /// once the binding is
    ///  recorded,
    /// Git has returned and nothing of it runs.
    /// A name that cannot be written is logged and not fatal,
    /// because the add's
    ///  own work must not stop for its bookkeeping,
    /// and the settlement's bound covers such a record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// claim.rememberRunningChild(destination, childPid);
    /// ```
    pub fn remember_running_child(
        &self,
        destination: &Path,
        child_pid: u32,
    ) {
        let Some(record) = self.destination_record.as_deref() else {
            return;
        };
        let Ok(text) = std::fs::read_to_string(record) else {
            return;
        };
        if crate::worktree_add_binding::decode_record(text.as_str())
            != Some(crate::worktree_add_binding::Recorded::Planned(
                destination.to_path_buf(),
            ))
        {
            return;
        }
        let Some(child) = crate::worktree_copy_owner::copy_owner_of_process(child_pid) else {
            crate::diagnostic_log::debug("worktreeCopyStageClaim", format!("the Git of {} was started and its identity cannot be read; the record stays unnamed", destination.display()).as_str());
            return;
        };
        // The copy this add owes survives the rewriting: the name of its Git is written beside the source, not instead of it.
        if let Err(error) = write_atomic_unsynced(
            record,
            crate::worktree_add_binding::planned_record_with(
                destination,
                crate::worktree_add_binding::owed_copy_of(text.as_str()).as_ref(),
                Some(&child),
            )
            .as_bytes(),
            "destination record",
        ) {
            crate::diagnostic_log::debug(
                "worktreeCopyStageClaim",
                format!(
                    "the Git of {} could not be named in its record ({}); the record stays unnamed",
                    destination.display(),
                    error.0
                )
                .as_str(),
            );
        }
    }

    pub fn record_binding(
        &self,
        binding: &Binding,
    ) -> Result<(), WorktreeCopyError> {
        let Some(record) = self.destination_record.as_deref() else {
            return Ok(());
        };
        // The copy the add owes is a line of the record this write replaces, so it is read out and written again: a
        // record that lost it would name a worktree no later command could fill.
        let owed: Option<crate::worktree_add_binding::OwedCopy> =
            match std::fs::read_to_string(record) {
                Ok(text) => crate::worktree_add_binding::owed_copy_of(text.as_str()),
                Err(_unreadable) => None,
            };
        // Published whole by one rename and flushed by nobody: this write waited 2.7 to 3.4 seconds of a loaded add
        // (measured 2026-11-11), a kill keeps it in the page cache, and a record a power cut takes back only makes a later
        // command keep and report the objects it names, which is the safe direction.
        write_atomic_unsynced(
            record,
            crate::worktree_add_binding::registered_record_with(binding, owed.as_ref(), None)
                .as_bytes(),
            "destination record",
        )?;
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
    ///  Removes the claim and its records after marking the add complete without flushing either,
    /// for an add whose destination a subvolume was swapped into.
    /// Why:
    ///   A snapshot that served an add swapped a subvolume into place,
    /// and every flush after that swap in the same btrfs
    ///  transaction is a commit of the whole filesystem,
    /// which waits behind other writers' data;
    /// the marker is published whole
    ///  by a rename and the filesystem's own commit makes it durable.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.settleUnsynced();
    /// ```
    pub fn settle_unsynced(self) -> Result<(), WorktreeCopyError> {
        self.mark_complete_unsynced()?;
        return self.release_unsynced();
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
        // The record of the object the stage's creation made goes first, so a kill in between leaves a claim whose stage
        // a later command reports rather than removes; then the destination record goes before the marker, and the marker
        // before the claim.
        remove_made(self.path.as_path())?;
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
