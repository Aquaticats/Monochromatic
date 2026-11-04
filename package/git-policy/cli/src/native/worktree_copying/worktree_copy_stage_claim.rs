//! What:
//!  The durable claim on a stage container's name, written before the container exists,
//!  and the cleanup of containers whose copy was interrupted before its journal was written.
//! Why:
//!  Staging a worktree's ignored state takes minutes and gigabytes, and the journal is written only after it.
//!  A process killed in between (a signal, the out-of-memory killer, a power loss) leaves a stage nothing names.
//!  The claim names it first, so the next recovery pass, which skips every claim whose owner still runs,
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

use crate::json_record::quote;
use crate::private_storage::sync_directory;
use crate::worktree_add_binding::{Binding, Recorded, decode_record, planned_record};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use crate::worktree_copy_owner::{CopyOwner, current_copy_owner, token_owner_is_dead};
use crate::worktree_copy_journal::{
    Role, TEMPORARY_SUFFIX, assert_private_path, ensure_journal_root, journal_root,
    read_pending_journals, remove_file_if_present, remove_stage, temporary_writer_token, write_atomic, write_atomic_unsynced,
};
use crate::worktree_copy_selection::{STAGE_PREFIX, js_order};
use crate::worktree_copy_stage::new_stage_container;
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
    /// The record of the worktree an interrupted add left without a checkout, when the claim covers one.
    destination_record: Option<PathBuf>,
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
    ///  Records durably that the worktree add this claim covers is complete.
    /// Why:
    ///   Recovery removes a registered worktree whose add has no such record, because the add never returned success; the
    ///  marker is written before the person is told of the result, so a worktree they were told about is never removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.markComplete();
    /// ```
    pub fn mark_complete(&self) -> Result<(), WorktreeCopyError> {
        crate::add_timing::step("markCompleteFile", || { return write_atomic(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path(), b"complete", "completion marker"); })?;
        if let Some(parent) = self.path.parent() {
            crate::add_timing::step("markCompleteDirectory", || { return sync_directory(parent); }).map_err(|error| return raw_failure(&error))?;
        }
        return Ok(());
    }

    /// What:
    ///  Records that the worktree add this claim covers is complete, published whole by a rename and flushed by nobody.
    /// Why:
    ///   Used after a subvolume was swapped into place: the swap makes the next flush in its btrfs transaction a commit of the
    ///  whole filesystem, which waited eighteen seconds behind other writers' data. The marker only silences the notice a
    ///  later command gives for an add it finds without one (recovery changes nothing), and the filesystem's own commit makes it
    ///  durable. It can never be durable before the swap: both belong to one transaction, or the marker to a later one.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.markCompleteUnsynced();
    /// ```
    pub fn mark_complete_unsynced(&self) -> Result<(), WorktreeCopyError> {
        return crate::add_timing::step("markCompleteFile", || { return write_atomic_unsynced(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path(), b"complete", "completion marker"); });
    }

    /// What:
    ///  Removes the claim and its records without flushing the removals.
    /// Why:
    ///   The same reason as [`StageClaim::mark_complete_unsynced`]. Records that return after a crash only make the next command
    ///  settle them: a marker present means no notice, and no record means nothing to say.
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
    ///   Recovery deletes only objects whose identities the record names; until this write, nothing is known to be the add's.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await claim.recordBinding(binding);
    /// ```
    pub fn record_binding(&self, binding: &Binding) -> Result<(), WorktreeCopyError> {
        let Some(record) = self.destination_record.as_deref() else { return Ok(()) };
        write_atomic(record, binding.encode().as_bytes(), "destination record")?;
        if let Some(parent) = self.path.parent() {
            sync_directory(parent).map_err(|error| return raw_failure(&error))?;
        }
        return Ok(());
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
        // The record goes before the marker: a kill in between leaves a marker with no record, never a record with no marker.
        if let Some(record) = self.destination_record.as_deref() {
            remove_file_if_present(record)?;
        }
        remove_file_if_present(self.path.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
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
    return claim_stage_for(common_dir, destination_root, &current_copy_owner()?);
}

/// The line of a claim's text that names its owner, after the line that names the container.
const OWNER_LINE_PREFIX: &str = "owner=";

/// What:
///  Claims a stage name for a named owner: the claim file holds the container path and, on a line of its own, the owner.
/// Why:
///   A claim of a live owner is an add or a copy still running; the reaper leaves it alone and removes only a dead owner's.
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
    let text: String = format!("{}\n{OWNER_LINE_PREFIX}{}\n", container.to_string_lossy(), owner.token());
    write_atomic(path.as_path(), text.as_bytes(), "stage claim")?;
    return Ok(StageClaim { path, container, destination_record: None });
}

const DESTINATION_SUFFIX: &str = ".add";

/// The suffix of the marker that says the worktree add the claim covers is complete.
const COMPLETE_SUFFIX: &str = ".done";

/// What:
///  Claims a stage name as `claim_stage` does and also records the destination of the `git worktree add` it serves.
/// Why:
///   Real Git registers the worktree before the stage is swapped in, so a process killed in between leaves a registered
///  worktree with no checkout; the next recovery names it to the person with the commands that finish or discard it.
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
///   The add's records stay whole while their owner runs and are settled once it is dead; a control names a dead owner to
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

/// What recovery decided about one add's record.
struct Settlement {
    /// What to tell the person, if anything.
    notice: Option<String>,
    /// Whether the record and the claim stay because the record could not be read or settled.
    keep: bool,
}

/// What:
///  Report an add that was interrupted before it was marked complete, and change nothing.
/// Why:
///   Recovery never deletes or unregisters a worktree: a path, a registration or a name can be changed by another process
///  between any check and any act, so nothing the wrapper recorded earlier is a right to remove something later. The report
///  names the worktree and the two commands that finish or remove it, as Git's own interrupted add leaves it. A record that
///  cannot be read, and a marker that cannot be looked up, are not "incomplete" and not "complete": the record stays and the
///  report says that the state is unknown.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { notice, keep } = await reportInterruptedAdd(record);
/// ```
fn settle_interrupted_add(record: &Path) -> Settlement {
    let marker: std::io::Result<bool> = match std::fs::symlink_metadata(record.with_extension(&COMPLETE_SUFFIX[1..])) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    };
    return settle_from(record, marker, &mut || { return std::fs::read_to_string(record); });
}

/// `settle_interrupted_add` with the two lookups as values, so that a lookup that fails other than by absence can be driven.
fn settle_from(record: &Path, marker: std::io::Result<bool>, read: &mut dyn FnMut() -> std::io::Result<String>) -> Settlement {
    let shown_record: String = quote(record.to_string_lossy().as_ref());
    match marker {
        Ok(true) => return Settlement { notice: None, keep: false },
        Ok(false) => {}
        Err(error) => {
            return Settlement {
                notice: Some(format!("cli-git: whether the `git worktree add` of {shown_record} completed cannot be checked ({error}); nothing was changed and the record stays.")),
                keep: true,
            };
        }
    }
    let text: String = match read() {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Settlement { notice: None, keep: false },
        Err(error) => {
            return Settlement {
                notice: Some(format!("cli-git: the record {shown_record} of an interrupted `git worktree add` cannot be read ({error}); nothing was changed and the record stays.")),
                keep: true,
            };
        }
    };
    return match decode_record(text.as_str()) {
        None => Settlement {
            notice: Some(format!("cli-git: the record {shown_record} of an interrupted `git worktree add` is unreadable; nothing was changed.")),
            keep: false,
        },
        Some(Recorded::Planned(destination)) => {
            let shown: String = quote(destination.to_string_lossy().as_ref());
            let notice: Option<String> = destination.join(".git").exists().then(|| {
                return format!("cli-git: an interrupted `git worktree add` may have left {shown}; nothing proves it is the add's, so it was not changed. `git worktree list` shows whether it is registered, and `git worktree remove --force {shown}` removes it.");
            });
            Settlement { notice, keep: false }
        }
        Some(Recorded::Registered(binding)) => {
            let shown: String = quote(binding.destination.to_string_lossy().as_ref());
            Settlement {
                notice: Some(format!("cli-git: an interrupted `git worktree add` left {shown} registered and possibly incomplete; it was not changed. `git -C {shown} reset --hard` completes its checkout, and `git worktree remove --force {shown}` removes it.")),
                keep: false,
            }
        }
    };
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
///   A forwarded command looks for recovery work only when something is pending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readStageClaimNames(commonDir)).length > 0
/// ```
pub fn stage_claims_exist(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    return Ok(!claim_names(journal_root(common_dir).as_path())?.is_empty());
}

/// What:
///  One claim file as read: the stage container it names, refused unless it is a cli-git stage container name, and the owner
///  token on its second line.
/// Why:
///   The container is deleted recursively, so a claim that names anything else is never acted on; the owner decides whether
///  the claim may be reaped at all. A claim that vanished while it was read was removed by its owner or by another command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [container, owner] = (await readFile(claimPath, 'utf8')).split('\n'); assertStageContainerName(container);
/// ```
fn read_claim(claim: &Path) -> Result<Option<(PathBuf, Option<String>)>, WorktreeCopyError> {
    // `match` on the lookup: a claim already gone is nobody's fault, and nothing is left to reap.
    match std::fs::symlink_metadata(claim) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    assert_private_path(claim, Role::JournalFile)?;
    // `match` on the read: the claim can vanish between the lookup and the read.
    let text: String = match std::fs::read_to_string(claim) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            let message: String = format!(
                "cli-git: could not read worktree-copy stage claim {}.",
                quote(claim.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    let mut lines = text.lines();
    let container: PathBuf = PathBuf::from(lines.next().unwrap_or_default());
    let owner: Option<String> = lines
        .find_map(|line: &str| { return line.strip_prefix(OWNER_LINE_PREFIX); })
        .map(str::to_string);
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
    return Ok(Some((container, owner)));
}

/// What:
///  Removes the temporaries that dead writers left in the journal root.
/// Why:
///   A write cut off before its rename leaves a file nothing else names. The writer is in the file's name, so the sweep
///  removes only what a dead process left and can never delete a temporary a live writer is about to rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(root)) if (name.endsWith('.tmp') && ownerIsDead(writerOf(name))) await rm(join(root, name), { force: true });
/// ```
fn remove_dead_temporaries(root: &Path) -> Result<(), WorktreeCopyError> {
    // `match` treats an absent root as having nothing to remove.
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(raw_failure(&error)),
    };
    for listed in listing {
        let entry: std::fs::DirEntry = listed.map_err(|error| return raw_failure(&error))?;
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !name.ends_with(TEMPORARY_SUFFIX) {
            continue;
        }
        // The name is `<item>.<writer>.<unique>.tmp`; a name of another shape has no writer and counts as a dead one's.
        if token_owner_is_dead(temporary_writer_token(name.as_str()), entry.path().as_path())? {
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
///  Removes every stage container that a dead owner's claim names and no journal does, then those claims, and says what it
///  removed.
/// Why:
///   A claim whose owner still runs belongs to an add or a copy between claiming and journaling, so it is left whole;
///  a claimed container no journal names, whose owner is dead, was abandoned by a process that died.
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
            let Some((container, owner)) = read_claim(claim.as_path())? else {
                continue;
            };
            // A live owner's claim is its running add or copy; a claim whose owner cannot be judged is kept, too.
            if !token_owner_is_dead(owner.as_deref(), claim.as_path())? {
                continue;
            }
            let present: bool = match std::fs::symlink_metadata(container.as_path()) {
                Ok(_) => true,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => return Err(raw_failure(&error)),
            };
            let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
            // The claim of a worktree add names a stage by its path alone, which proves nothing about what stands there now
            // (a snapshot, a directory another process put there, or nothing): such a stage is reported and never removed.
            // A record that cannot be looked up counts as one: when in doubt the stage is kept.
            let of_an_add: bool = !matches!(std::fs::symlink_metadata(record.as_path()), Err(ref error) if error.kind() == std::io::ErrorKind::NotFound);
            if present && !journaled.contains(&container) && of_an_add {
                notices.push(format!(
                    "cli-git: the stage {} was left by an interrupted `git worktree add` and was not changed; remove it yourself once nothing needs it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).",
                    quote(container.to_string_lossy().as_ref())
                ));
            } else if present && !journaled.contains(&container) {
                assert_private_path(container.as_path(), Role::PrivateStage)?;
                remove_stage(container.as_path())?;
                notices.push(format!(
                    "cli-git: removed the stage {} that an interrupted ignored-state copy left behind.",
                    quote(container.to_string_lossy().as_ref())
                ));
            }
            let settled: Settlement = settle_interrupted_add(record.as_path());
            if let Some(notice) = settled.notice {
                notices.push(notice);
            }
            if settled.keep {
                continue;
            }
            remove_file_if_present(record.as_path())?;
            remove_file_if_present(claim.with_extension(&COMPLETE_SUFFIX[1..]).as_path())?;
            remove_file_if_present(claim.as_path())?;
        }
    }
    remove_dead_temporaries(root.as_path())?;
    absent_is_fine(sync_directory(root.as_path()))?;
    return Ok(notices);
}

/// Claim controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_claim_tests.rs"]
mod tests;
