//! What:
//!  The journaled phases of one worktree conversion:
//! the claim on the stage,
//! the copy and its proof,
//! the swap,
//! the index
//!  refresh and the removal of the old tree.
//! Why:
//!  Each phase is recorded before the next begins,
//! so a rerun after a kill resumes at the phase the journal names and
//!  nothing is deleted that the journal does not prove this conversion made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyPhase(platform, job, interrupt, say);
//! await swapPhase(platform, job, interrupt);
//! ```

use crate::line_sink::LineSink;
use crate::fs_beneath::{ObjectId, identity_at, open_root};
use crate::plain_file::read_plain_file;
use crate::process_use::{UseTarget, processes_using};
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_index_files::{IndexLock, RefreshOptions, RefreshStatus, refresh_index_file_report, repository_hash_length};
use crate::subvolume_lock::{acquire, is_ours};
use crate::subvolume_pass_report::what_a_pass_could_not_carry;
use crate::subvolume_platform::Platform;
use crate::subvolume_sync::{SyncOptions, SyncStats, sync_tree};
use crate::subvolume_sync_pass_state_and_checks::SAMPLE_SIZE;
use crate::worktree_convert_inspection::{Candidate, GIT_FILE_LIMIT, converting_path};
use crate::worktree_convert_journal::{JOURNAL_NAME, Journal, Phase, io_step, is_recorded_object};
use crate::worktree_convert_remove::{Removal, remove_recorded_old_tree};
use std::path::{Path, PathBuf};

/// What:
///  Everything one conversion needs besides the platform.
/// Why:
///   The phases share the same facts,
/// so they travel together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Job = { candidate: Candidate; converting: string; journal: Journal };
/// ```
pub struct Job<'a> {
    /// The worktree.
    pub candidate: &'a Candidate,
    /// The sibling the copy is built at.
    pub converting: PathBuf,
    /// The journal,
    /// updated in step with the phases.
    pub journal: Journal,
}

impl Job<'_> {
    /// Record a phase durably.
    pub fn reach(&mut self, phase: Phase) -> Result<(), MigrateError> {
        self.journal.phase = phase;
        if phase == Phase::Synced {
            // The proving pass just finished, so from this instant on a file that changes is one the removal compares by
            // contents; on a resume before the swap the pass runs again and the instant moves with it.
            self.journal.synced_ms = Some(crate::worktree_convert_journal::now_milliseconds());
        }
        return self.journal.save(self.candidate.admin.as_path());
    }

    /// The directory that holds the worktree and its converting sibling,
    /// held open.
    pub fn parent(&self) -> Result<std::fs::File, MigrateError> {
        return held_parent(self.candidate.path.as_path());
    }

    /// The name of the converting sibling in the parent directory.
    pub fn converting_name(&self) -> &std::ffi::OsStr {
        return self.converting.file_name().unwrap_or_default();
    }

    /// The identity of the object at the converting name,
    /// if there is one.
    pub fn stage_identity(&self) -> Result<Option<ObjectId>, MigrateError> {
        return stage_identity_of(self.candidate.path.as_path());
    }
}

/// The identity of the object at `path`,
/// a link not followed,
/// or nothing when it cannot be examined.
pub fn identity_of_path(path: &Path) -> Option<ObjectId> {
    let parent: std::fs::File = held_parent(path).ok()?;
    return identity_at(&parent, path.file_name()?).ok();
}

/// The identity,
/// with the birth time,
/// of the directory at `path`.
pub fn stage_free_identity(path: &Path) -> Result<ObjectId, MigrateError> {
    let parent: std::fs::File = held_parent(path)?;
    return io_step(identity_at(&parent, path.file_name().unwrap_or_default()), "examine", path);
}

/// The directory that holds `path`,
/// held open so that names below it mean the same objects for as long as it is held.
pub fn held_parent(path: &Path) -> Result<std::fs::File, MigrateError> {
    let Some(parent) = path.parent() else {
        return Err(MigrateError::Journal(format!("{} has no parent directory", path.display())));
    };
    return io_step(open_root(parent), "hold the directory of the worktree", parent);
}

/// The identity of the object at the converting name of `path`,
/// or nothing when the name is free.
pub fn stage_identity_of(path: &Path) -> Result<Option<ObjectId>, MigrateError> {
    let converting: PathBuf = converting_path(path);
    let parent: std::fs::File = held_parent(path)?;
    return match identity_at(&parent, converting.file_name().unwrap_or_default()) {
        Ok(found) => Ok(Some(found)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(MigrateError::io("examine", converting.as_path(), &error)),
    };
}

/// What:
///  Whether the object at the converting name is the empty stage this conversion made but never recorded,
/// because it was killed between the creation and the line that names it.
/// Why:
///   A leftover of that kind is the conversion's own,
/// and leaving it there blocks the worktree with a refusal no command can
///  lift:
/// the journal says the phase that creates the stage,
/// holds no identity yet,
/// and the name holds
///  nothing,
/// which is the one state that window can leave.
/// Anything that holds even one entry may be somebody's copy and is never adopted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownsUnrecordedStage(journal, worktree): boolean; // creating, no identity, empty directory at the stage name
/// ```
pub fn owns_unrecorded_stage(journal: &Journal, worktree: &Path) -> Result<bool, MigrateError> {
    if journal.new_identity.is_some() || journal.phase != Phase::Creating {
        return Ok(false);
    }
    let converting: PathBuf = converting_path(worktree);
    return match std::fs::read_dir(converting.as_path()) {
        Ok(mut entries) => Ok(entries.next().is_none()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_unreadable) => Ok(false),
    };
}

/// What:
///  Make sure the converting subvolume exists and is the one this command made:
/// create it and record its identity,
/// or find
///  the recorded identity at the name.
/// Why:
///   A directory at the converting name that the journal has no identity for is foreign,
/// whether someone made it or this
///  command was killed between creating it and recording it;
/// it is never synced into,
/// exchanged or destroyed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureStage(platform, job); // throws when the name holds anything the journal did not record creating
/// ```
pub fn ensure_stage(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt) -> Result<(), MigrateError> {
    // An empty stage this conversion created and did not record is its own: the kill landed between the creation and the
    // line that names it, so the identity is taken now rather than leaving the worktree blocked by its own leftover.
    if owns_unrecorded_stage(&job.journal, job.candidate.path.as_path())? {
        let Some(made) = job.stage_identity()? else {
            return Err(MigrateError::Journal(format!("{} was not there after it was created", job.converting.display())));
        };
        job.journal.new_identity = Some(made);
        return job.journal.save(job.candidate.admin.as_path());
    }
    match (job.stage_identity()?, job.journal.new_identity) {
        (Some(found), Some(recorded)) if found == recorded => return Ok(()),
        (Some(_), _) => {
            return Err(MigrateError::Journal(format!(
                "{} exists but the journal does not record this conversion creating it, so it is left alone; remove it yourself once you know what it is, or run the command with --abandon",
                job.converting.display()
            )));
        }
        (None, Some(_)) => {
            return Err(MigrateError::Journal(format!(
                "{} was created by this conversion and is gone; run the command with --abandon and start again",
                job.converting.display()
            )));
        }
        (None, None) => {}
    }
    platform.create_subvolume(job.converting.as_path())?;
    interrupt.check("stage-unrecorded")?;
    let Some(made) = job.stage_identity()? else {
        return Err(MigrateError::Journal(format!("{} was not there after it was created", job.converting.display())));
    };
    job.journal.new_identity = Some(made);
    return job.journal.save(job.candidate.admin.as_path());
}

/// Whether a sync pass changed anything:
/// it copied,
/// deleted,
/// recreated or lost a file or link.
pub fn pass_changed(stats: &SyncStats) -> bool {
    return stats.files_copied > 0 || stats.deleted > 0 || stats.vanished > 0 || stats.symlinks > 0 || stats.fifos > 0;
}

/// What:
///  Fail with what a pass could not carry into the stage,
///  because a copy that leaves it out would lose it at the swap:
/// a file
///  this user cannot read,
/// or a socket or a device node the copy never recreates.
/// Why:
///   The comparison before the removal names an entry the installed tree lacks,
/// but by then the swap has happened,
///  `--abandon` refuses,
/// and the only way out is deleting an entry below a name only this command uses.
/// Refusing here stops the conversion while the stage is a copy,
/// so nothing has been swapped,
///  `--abandon` works,
/// and a person who stops whatever holds the socket and deletes it,
/// or unlocks the
///  file,
/// finishes by running the command again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// requireComplete(stats, path); // throws when the copy would lose an entry
/// ```
pub fn require_complete(stats: &SyncStats, path: &Path) -> Result<(), MigrateError> {
    if let Some(first) = stats.unreadable.first() {
        return Err(MigrateError::Journal(format!(
            "{} holds {} file(s) this user cannot read, the first being {}; they would be lost at the swap, so nothing was swapped",
            path.display(),
            stats.unreadable.len(),
            first.display()
        )));
    }
    if !stats.special_skipped.is_empty() {
        let mut named: Vec<String> = Vec::new();
        for entry in stats.special_skipped.iter().take(SAMPLE_SIZE) {
            named.push(entry.display().to_string());
        }
        return Err(MigrateError::Journal(format!(
            "{} holds {} socket(s) or device node(s) that this copy does not recreate ({}), so the conversion stops here and nothing was swapped: stop whatever holds each one (a socket is recreated by the program that made it), delete it from {}, and run the command again",
            path.display(),
            stats.special_skipped.len(),
            named.join(", "),
            path.display()
        )));
    }
    return Ok(());
}

/// Take Git's `index.lock` of the worktree with this conversion's token,
/// or accept the one an earlier run of it took.
pub fn hold_index_lock(job: &Job<'_>) -> Result<(), MigrateError> {
    return acquire(job.candidate.admin.join("index.lock").as_path(), job.journal.token.as_str());
}

/// Remove Git's `index.lock` of the worktree when it holds this conversion's token;
/// a lock that holds other text is a Git
///  command's and stays.
pub fn release_index_lock(job: &Job<'_>) -> Result<(), MigrateError> {
    let lock: PathBuf = job.candidate.admin.join("index.lock");
    if !is_ours(lock.as_path(), job.journal.token.as_str()) {
        return Ok(());
    }
    return match std::fs::remove_file(&lock) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(MigrateError::io("release the index lock", lock.as_path(), &error)),
    };
}

/// What:
///  The copy phase:
/// create the subvolume if needed,
/// copy with cloned data,
/// and prove the copy equals the original.
/// Why:
///   It runs again on every resume before the swap,
/// whatever the journal says:
/// `Synced` only records that the copy once
///  equalled the original,
/// and a writer may have acted since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyPhase(platform, job, interrupt, say);
/// ```
pub fn copy_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt, say: &mut dyn LineSink) -> Result<(), MigrateError> {
    if job.journal.phase < Phase::Creating {
        job.reach(Phase::Creating)?;
    }
    interrupt.check("journaled")?;
    ensure_stage(platform, job, interrupt)?;
    interrupt.check("created")?;
    let first: SyncStats = sync_tree(platform, job.candidate.path.as_path(), job.converting.as_path(), SyncOptions { delete: false, update_only: false, excluded: &[], created_since: None }, interrupt)?;
    say.say(format!("  copied {} entries ({} files, {} cloned)", first.entries, first.files_copied, first.cloned).as_str());
    for text in what_a_pass_could_not_carry("  copy", &first) {
        say.say(text.as_str());
    }
    require_complete(&first, job.candidate.path.as_path())?;
    // Git's `index.lock` is taken only now: the long first copy ran without it, and the lock covers the span in which a Git
    // writer could invalidate the proof, the swap or the index rewrite.
    hold_index_lock(job)?;
    // The proof: a second pass that must find nothing to copy or delete, so the copy equals the original at this instant.
    let second: SyncStats = sync_tree(platform, job.candidate.path.as_path(), job.converting.as_path(), SyncOptions { delete: true, update_only: false, excluded: &[], created_since: None }, interrupt)?;
    for text in what_a_pass_could_not_carry("  proving copy", &second) {
        say.say(text.as_str());
    }
    if let Err(error) = require_complete(&second, job.candidate.path.as_path()) {
        release_index_lock(job)?;
        return Err(error);
    }
    if pass_changed(&second) {
        // Nothing is swapped, so nothing needs the lock any more; Git commands may write the index again.
        release_index_lock(job)?;
        return Err(MigrateError::Journal(format!(
            "{} changed while it was copied ({} file(s) copied and {} removed by the proving pass); nothing was swapped, run the command again",
            job.candidate.path.display(),
            second.files_copied,
            second.deleted
        )));
    }
    interrupt.check("verified")?;
    job.reach(Phase::Synced)?;
    interrupt.check("synced")?;
    return Ok(());
}

/// What:
///  Whether the object at `path` is the old tree this conversion recorded.
/// Why:
///   The swap and the removal act only on the object the journal names,
/// wherever it is now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityOfPath(path) matches journal.oldIdentity
/// ```
pub fn is_old_tree(journal: &Journal, path: &Path) -> bool {
    return match identity_of_path(path) {
        Some(found) => is_recorded_object(&journal.old_identity, &found),
        None => false,
    };
}

/// What:
///  Which of the two directories holds the old tree:
/// the worktree path (the swap has not happened) or the converting name
///  (it has).
/// Why:
///   The swap is one atomic exchange and the journal is written after it,
/// so a kill between the two leaves the journal one phase
///  behind;
/// the identity the journal recorded for the old root says where the old tree is now,
/// so a rerun neither exchanges back
///  nor loses track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whereIsTheOldTree(journal, path, converting): 'at-path' | 'at-converting' | 'nowhere';
/// ```
pub fn old_tree_is_at(journal: &Journal, path: &Path, converting: &Path) -> Where {
    if is_old_tree(journal, path) {
        return Where::AtPath;
    }
    if is_old_tree(journal, converting) {
        return Where::AtConverting;
    }
    return Where::Nowhere;
}

/// Where the old tree is,
/// relative to the swap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Where {
    /// At the worktree path:
    /// the swap has not happened.
    AtPath,
    /// At the converting name:
    /// the swap has happened.
    AtConverting,
    /// At neither:
    /// something else replaced it.
    Nowhere,
}

/// The swap phase:
/// exchange the two directories (unless a killed run already did),
/// then record it.
pub fn swap_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt) -> Result<(), MigrateError> {
    match old_tree_is_at(&job.journal, job.candidate.path.as_path(), job.converting.as_path()) {
        Where::AtPath => {
            ensure_stage(platform, job, interrupt)?;
            let refused: Vec<String> = refusals_after_lock(platform, job.candidate);
            if !refused.is_empty() {
                return Err(MigrateError::Journal(format!("it can no longer be swapped: {}", refused.join("; "))));
            }
            platform.exchange(job.candidate.path.as_path(), job.converting.as_path())?;
        }
        Where::AtConverting => {}
        Where::Nowhere => {
            return Err(MigrateError::Journal(format!(
                "neither {} nor {} is the directory this conversion recorded, so nothing is swapped or removed",
                job.candidate.path.display(),
                job.converting.display()
            )));
        }
    }
    interrupt.check("exchanged")?;
    job.reach(Phase::Exchanged)?;
    return Ok(());
}

/// The reasons that still bind just before the swap:
/// a process that appeared,
/// a lock that was taken,
/// a file that changed.
pub fn refusals_after_lock(_platform: &dyn Platform, candidate: &Candidate) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    let scan = processes_using(&UseTarget::Inside(candidate.path.as_path()));
    for user in &scan.users {
        reasons.push(format!("process {} ({}) is using the worktree", user.pid, user.command.as_deref().unwrap_or("unknown")));
    }
    if let Some(why) = scan.unavailable.as_ref() {
        reasons.push(format!("the process table cannot be read ({why})"));
    }
    return reasons;
}

/// The checks after the swap:
/// the path is the subvolume this command created and its `.git` file holds the bytes it held before.
pub fn verify_swap(platform: &dyn Platform, job: &Job<'_>) -> Result<(), MigrateError> {
    let path: &Path = job.candidate.path.as_path();
    if !platform.is_subvolume_root(path)? {
        return Err(MigrateError::Journal(format!("{} is not a subvolume root after the swap; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    let parent: std::fs::File = job.parent()?;
    let installed: ObjectId = io_step(identity_at(&parent, path.file_name().unwrap_or_default()), "examine", path)?;
    if job.journal.new_identity != Some(installed) {
        return Err(MigrateError::Journal(format!("{} is not the subvolume this conversion created; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    let now: Vec<u8> = io_step(read_plain_file(&path.join(".git"), GIT_FILE_LIMIT), "read", path.join(".git").as_path())?;
    if now != job.journal.git_file {
        return Err(MigrateError::Journal(format!("the .git file of {} differs after the swap; the old tree is kept at {}", path.display(), job.converting.display())));
    }
    return Ok(());
}

/// The index phase:
/// rewrite the stat fields of the index to the files of the subvolume,
/// so the first status stays fast.
/// A refresh that did not install the new index stops the conversion with the old tree kept,
/// and the journal stays at the swap.
pub fn index_phase(job: &mut Job<'_>, interrupt: &Interrupt, say: &mut dyn LineSink) -> Result<(), MigrateError> {
    let references: Vec<PathBuf> = vec![job.converting.clone()];
    let report = refresh_index_file_report(
        job.candidate.admin.join("index").as_path(),
        job.candidate.path.as_path(),
        &RefreshOptions { lock: IndexLock::Held(job.journal.token.as_str().into()), references: references.as_slice(), hash_length: repository_hash_length(job.candidate.common.as_path()) },
    );
    say.say(format!("  index: {}", report.line).as_str());
    if report.status == RefreshStatus::Skipped {
        return Err(MigrateError::Journal(format!(
            "the index of {} was not refreshed ({}); the old tree is kept at {}; fix that and run the command again",
            job.candidate.path.display(),
            report.line,
            job.converting.display()
        )));
    }
    interrupt.check("refreshed")?;
    job.reach(Phase::Refreshed)?;
    // The rewrite is the last thing that needs the lock; the removal of the old tree can take long and writes no index.
    return release_index_lock(job);
}

/// What:
///  The removal phase:
/// keep the old tree when a process still holds it or when the installed tree does not hold all of it,
/// and
///  otherwise delete it,
/// but only when it is the very directory the journal recorded.
/// Why:
///   The old tree sits at the converting name from the exchange on,
/// so nothing opens it by a name that anything uses;
/// a process
///  that already held it open is found in the process table.
/// The comparison with the installed tree is read-only and copies
///  nothing,
/// so an edit made to the installed worktree is never overwritten;
/// a difference keeps the old tree and is reported,
///  and a person decides which version is wanted.
/// Anything that leaves the old tree in place is an incomplete conversion:
/// the
///  journal stays and the run says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removalPhase(platform, job, interrupt, say);
/// ```
pub fn removal_phase(platform: &dyn Platform, job: &mut Job<'_>, interrupt: &Interrupt) -> Result<(), MigrateError> {
    let parent: std::fs::File = job.parent()?;
    let scan = processes_using(&UseTarget::Inside(job.converting.as_path()));
    let mut holders: Vec<String> = Vec::new();
    for user in &scan.users {
        holders.push(format!("process {} ({})", user.pid, user.command.as_deref().unwrap_or("unknown")));
    }
    if let Some(why) = scan.unavailable.as_ref() {
        holders.push(format!("the process table cannot be read ({why})"));
    }
    if !holders.is_empty() {
        return Err(MigrateError::Journal(format!("the old tree at {} is kept because {}; stop that and run the command again", job.converting.display(), holders.join("; "))));
    }
    match remove_recorded_old_tree(platform, &parent, job.converting_name(), job.journal.old_identity, job.candidate.path.as_path(), job.journal.synced_ms)? {
        Removal::Removed | Removal::Absent => {}
        Removal::Kept(why) => {
            return Err(MigrateError::Journal(format!("the old tree was not removed: {why}; the journal is kept, so put things right and run the command again")));
        }
        Removal::Differs(why) => {
            return Err(MigrateError::Journal(format!(
                "the old tree at {} is kept, and nothing was copied from it: {why}; settle which version is wanted by hand, then run the command again to remove it",
                job.converting.display()
            )));
        }
    }
    interrupt.check("removed")?;
    job.reach(Phase::Removed)?;
    return Ok(());
}

/// Release the lock and drop the journal,
/// the last step of a finished conversion.
pub fn finish(job: &Job<'_>) -> Result<(), MigrateError> {
    release_index_lock(job)?;
    // Nothing is left at the converting name, so the record beside the worktree must not name anything either.
    crate::worktree_convert_leftover_stage_record::forget_leftover_stage(job.candidate.path.as_path());
    let journal: PathBuf = job.candidate.admin.join(JOURNAL_NAME);
    return io_step(std::fs::remove_file(&journal), "remove the conversion journal", journal.as_path());
}
