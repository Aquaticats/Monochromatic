//! What:
//!  The copy phase of one worktree conversion:
//! the converting subvolume is made and filled with cloned data,
//! the copy is proven equal to the original,
//! and a pass that could not carry an entry stops the conversion before anything is swapped.
//! Why:
//!  The phase runs again on every resume before the swap,
//! whatever the journal says:
//! `Synced` only records that the copy once equalled the original,
//! and a writer may have acted since.
//! What a pass could not carry would be lost at the swap,
//! so it is refused while the stage is a copy and nothing has been exchanged.
//!
//! The job and its stage container,
//! the swap and the removal of the old tree,
//! and the identity of the object at a name
//! are the sibling files `worktree_convert_job.rs`,
//! `worktree_convert_swap_and_removal.rs` and `worktree_convert_stage_identity.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyPhase(platform, job, interrupt, say);
//! ```

use crate::line_sink::LineSink;
use crate::subvolume_error::{
    Interrupt,
    MigrateError,
};
use crate::subvolume_pass_report::what_a_pass_could_not_carry;
use crate::subvolume_platform::Platform;
use crate::subvolume_sync::{
    SyncOptions,
    SyncStats,
    sync_tree,
};
use crate::subvolume_sync_pass_state_and_checks::SAMPLE_SIZE;
/// One conversion's job,
/// its stage container and its index lock,
/// re-exported so every caller of these phases names them here.
pub use crate::worktree_convert_job::{
    Job,
    ensure_stage,
    hold_index_lock,
    release_index_lock,
};
use crate::worktree_convert_journal::Phase;
/// The identity of the object at a name,
/// and the directory held open to ask it,
/// re-exported so every caller of these phases names them here.
pub use crate::worktree_convert_stage_identity::{
    held_parent,
    identity_of_path,
    stage_free_identity,
    stage_identity_of,
};
/// The swap,
/// the index refresh and the removal of the old tree,
/// and where the old tree stands,
/// re-exported so every caller of these phases names them here.
pub use crate::worktree_convert_swap_and_removal::{
    Where,
    finish,
    index_phase,
    old_tree_is_at,
    removal_phase,
    swap_phase,
    verify_swap,
};
use std::path::Path;

/// Whether a sync pass changed anything:
/// it copied,
/// deleted,
/// recreated or lost a file or link.
pub fn pass_changed(stats: &SyncStats) -> bool {
    return stats.files_copied > 0
        || stats.deleted > 0
        || stats.vanished > 0
        || stats.symlinks > 0
        || stats.fifos > 0;
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
pub fn require_complete(
    stats: &SyncStats,
    path: &Path,
) -> Result<(), MigrateError> {
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
pub fn copy_phase(
    platform: &dyn Platform,
    job: &mut Job<'_>,
    interrupt: &Interrupt,
    say: &mut dyn LineSink,
) -> Result<(), MigrateError> {
    if job.journal.phase < Phase::Creating {
        job.reach(Phase::Creating)?;
    }
    interrupt.check("journaled")?;
    ensure_stage(platform, job, interrupt)?;
    interrupt.check("created")?;
    let first: SyncStats = sync_tree(
        platform,
        job.candidate.path.as_path(),
        job.converting.as_path(),
        SyncOptions {
            delete: false,
            update_only: false,
            excluded: &[],
            created_since: None,
        },
        interrupt,
    )?;
    say.say(
        format!(
            "  copied {} entries ({} files, {} cloned)",
            first.entries, first.files_copied, first.cloned
        )
        .as_str(),
    );
    for text in what_a_pass_could_not_carry("  copy", &first) {
        say.say(text.as_str());
    }
    require_complete(&first, job.candidate.path.as_path())?;
    // Git's `index.lock` is taken only now: the long first copy ran without it, and the lock covers the span in which a Git
    // writer could invalidate the proof, the swap or the index rewrite.
    hold_index_lock(job)?;
    // The proof: a second pass that must find nothing to copy or delete, so the copy equals the original at this instant.
    // The instant the removal compares change times against is taken before the pass, not after it: an edit made while the
    // pass runs can land after the pass examined that entry, and it carries a change time before the pass ends.
    let proof_started_ms: i64 = crate::worktree_convert_journal::now_milliseconds();
    let second: SyncStats = sync_tree(
        platform,
        job.candidate.path.as_path(),
        job.converting.as_path(),
        SyncOptions {
            delete: true,
            update_only: false,
            excluded: &[],
            created_since: None,
        },
        interrupt,
    )?;
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
    job.journal.synced_ms = Some(proof_started_ms);
    job.reach(Phase::Synced)?;
    interrupt.check("synced")?;
    return Ok(());
}
