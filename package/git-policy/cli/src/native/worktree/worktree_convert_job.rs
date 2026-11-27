//! What:
//!  One conversion's job:
//! the candidate,
//! the converting sibling and the journal the phases share,
//! the stage container it makes sure of,
//! and Git's index lock that a phase holds and gives back.
//! Why:
//!  The phases share the same facts,
//! so they travel together,
//! and every step that says which object a conversion acts on does it through the job.
//! A directory at the converting name that the journal has no identity for is foreign,
//! whether someone made it or this command was killed between creating it and recording it:
//! it is never synced into,
//! exchanged or destroyed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! ensureStage(platform, job, interrupt); // throws when the name holds anything the journal did not record creating
//! holdIndexLock(job);
//! releaseIndexLock(job);
//! ```

use crate::fs_beneath::ObjectId;
use crate::subvolume_error::{
    Interrupt,
    MigrateError,
};
use crate::subvolume_lock::{
    acquire,
    is_ours,
};
use crate::subvolume_platform::Platform;
use crate::worktree_convert_inspection::Candidate;
use crate::worktree_convert_journal::{
    Journal,
    Phase,
};
use crate::worktree_convert_stage_identity::{
    held_parent,
    stage_identity_of,
};
use crate::worktree_convert_unrecorded_stage::owns_unrecorded_stage;
use std::path::PathBuf;

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
    pub fn reach(
        &mut self,
        phase: Phase,
    ) -> Result<(), MigrateError> {
        self.journal.phase = phase;
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
pub fn ensure_stage(
    platform: &dyn Platform,
    job: &mut Job<'_>,
    interrupt: &Interrupt,
) -> Result<(), MigrateError> {
    // An empty stage this conversion created and did not record is its own: the kill landed between the creation and the
    // line that names it, so the identity is taken now rather than leaving the worktree blocked by its own leftover.
    if owns_unrecorded_stage(&job.journal, job.candidate.path.as_path())? {
        let Some(made) = job.stage_identity()? else {
            return Err(MigrateError::Journal(format!(
                "{} was not there after it was created",
                job.converting.display()
            )));
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
        return Err(MigrateError::Journal(format!(
            "{} was not there after it was created",
            job.converting.display()
        )));
    };
    job.journal.new_identity = Some(made);
    return job.journal.save(job.candidate.admin.as_path());
}

/// Take Git's `index.lock` of the worktree with this conversion's token,
/// or accept the one an earlier run of it took.
pub fn hold_index_lock(job: &Job<'_>) -> Result<(), MigrateError> {
    return acquire(
        job.candidate.admin.join("index.lock").as_path(),
        job.journal.token.as_str(),
    );
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
        Err(error) => Err(MigrateError::io(
            "release the index lock",
            lock.as_path(),
            &error,
        )),
    };
}
