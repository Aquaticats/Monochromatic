//! What:
//!  The end of a conversion outside the stage:
//! where the old tree stands relative to the swap,
//! the swap itself,
//! the checks before and after it,
//! the index refresh,
//! the removal of the old tree,
//! and the release of the lock and the journal.
//! Why:
//!  The old tree sits at the converting name from the exchange on,
//! so nothing opens it by a name that anything uses.
//! A process that already holds it open is found in the process table,
//! and another command's stage container is found by its claim,
//! because a command that writes into one holds no descriptor there between two entries.
//! Anything that leaves the old tree in place is an incomplete conversion:
//! the journal stays and the run says so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! swapPhase(platform, job, interrupt);
//! indexPhase(job, interrupt, say);
//! removalPhase(platform, job, interrupt, say);
//! finish(job);
//! ```

use crate::fs_beneath::{
    ObjectId,
    identity_at,
};
use crate::line_sink::LineSink;
use crate::plain_file::read_plain_file;
use crate::process_use::{
    UseTarget,
    processes_using,
};
/// A name another program chose,
/// shown so no byte of it can act on the line it lands in,
/// and the parentheses that line puts around it.
use crate::shown_so_it_cannot_rewrite_the_line::{
    TheWrappingAroundTheName,
    shown_so_it_cannot_rewrite_the_line,
};
use crate::subvolume_error::{
    Interrupt,
    MigrateError,
};
use crate::subvolume_index_files::{
    IndexLock,
    RefreshOptions,
    RefreshStatus,
    refresh_index_file_report,
    repository_hash_length,
};
use crate::subvolume_platform::Platform;
use crate::worktree_convert_filled_stages::{
    removal_refusal,
    swap_refusals,
};
use crate::worktree_convert_inspection::{
    Candidate,
    GIT_FILE_LIMIT,
};
use crate::worktree_convert_job::{
    Job,
    ensure_stage,
    release_index_lock,
};
use crate::worktree_convert_journal::{
    JOURNAL_NAME,
    Journal,
    Phase,
    io_step,
    is_recorded_object,
};
use crate::worktree_convert_remove::{
    Removal,
    remove_recorded_old_tree,
};
use crate::worktree_convert_stage_identity::identity_of_path;
use std::path::{
    Path,
    PathBuf,
};

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
pub fn is_old_tree(
    journal: &Journal,
    path: &Path,
) -> bool {
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
pub fn old_tree_is_at(
    journal: &Journal,
    path: &Path,
    converting: &Path,
) -> Where {
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
pub fn swap_phase(
    platform: &dyn Platform,
    job: &mut Job<'_>,
    interrupt: &Interrupt,
) -> Result<(), MigrateError> {
    match old_tree_is_at(
        &job.journal,
        job.candidate.path.as_path(),
        job.converting.as_path(),
    ) {
        Where::AtPath => {
            ensure_stage(platform, job, interrupt)?;
            let refused: Vec<String> = refusals_after_lock(platform, job.candidate);
            if !refused.is_empty() {
                return Err(MigrateError::Journal(format!(
                    "it can no longer be swapped: {}",
                    refused.join("; ")
                )));
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

/// What:
///  One process the scan saw inside the worktree, as the refusal that names it:
///  its number and the program it runs,
///  the program shown so no byte of it can act on the line.
/// Why:
///   The scan reports the name a program chose for itself (`/proc/<pid>/comm`),
///  and this refusal is how a person finds the program to close:
///  an escape in the name would move the cursor or erase the line,
///  and a `)` would end the parentheses that say whose name it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (pid, command) => `process ${pid} (${shownSoItCannotRewriteTheLine(command ?? 'unknown', TheWrappingAroundTheName.Parentheses)}) is using the worktree`;
/// ```
pub(crate) fn a_process_using_the_worktree_line(
    pid: i64,
    command: Option<&str>,
) -> String {
    return format!(
        "process {pid} ({}) is using the worktree",
        shown_so_it_cannot_rewrite_the_line(
            command.unwrap_or("unknown"),
            TheWrappingAroundTheName::Parentheses
        )
    );
}

/// The reasons that still bind just before the swap:
/// a process that appeared,
/// a lock that was taken,
/// a file that changed,
/// and another command's stage container standing inside the worktree.
pub fn refusals_after_lock(
    _platform: &dyn Platform,
    candidate: &Candidate,
) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    let scan = processes_using(&UseTarget::Inside(candidate.path.as_path()));
    for user in &scan.users {
        reasons.push(a_process_using_the_worktree_line(
            user.pid,
            user.command.as_deref(),
        ));
    }
    if let Some(why) = scan.unavailable.as_ref() {
        reasons.push(format!("the process table cannot be read ({why})"));
    }
    // A container another command is filling is that command's work, and the swap moves the whole tree it stands in, so the
    // command filling it would lose the place it writes to. The exchange has not happened yet, so refusing here leaves that
    // command's work where it is.
    reasons.extend(swap_refusals(
        candidate.common.as_path(),
        candidate.path.as_path(),
    ));
    return reasons;
}

/// The checks after the swap:
/// the path is the subvolume this command created and its `.git` file holds the bytes it held before.
pub fn verify_swap(
    platform: &dyn Platform,
    job: &Job<'_>,
) -> Result<(), MigrateError> {
    let path: &Path = job.candidate.path.as_path();
    if !platform.is_subvolume_root(path)? {
        return Err(MigrateError::Journal(format!(
            "{} is not a subvolume root after the swap; the old tree is kept at {}",
            path.display(),
            job.converting.display()
        )));
    }
    let parent: std::fs::File = job.parent()?;
    let installed: ObjectId = io_step(
        identity_at(&parent, path.file_name().unwrap_or_default()),
        "examine",
        path,
    )?;
    if job.journal.new_identity != Some(installed) {
        return Err(MigrateError::Journal(format!(
            "{} is not the subvolume this conversion created; the old tree is kept at {}",
            path.display(),
            job.converting.display()
        )));
    }
    let now: Vec<u8> = io_step(
        read_plain_file(&path.join(".git"), GIT_FILE_LIMIT),
        "read",
        path.join(".git").as_path(),
    )?;
    if now != job.journal.git_file {
        return Err(MigrateError::Journal(format!(
            "the .git file of {} differs after the swap; the old tree is kept at {}",
            path.display(),
            job.converting.display()
        )));
    }
    return Ok(());
}

/// The index phase:
/// rewrite the stat fields of the index to the files of the subvolume,
/// so the first status stays fast.
/// A refresh that did not install the new index stops the conversion with the old tree kept,
/// and the journal stays at the swap.
pub fn index_phase(
    job: &mut Job<'_>,
    interrupt: &Interrupt,
    say: &mut dyn LineSink,
) -> Result<(), MigrateError> {
    let references: Vec<PathBuf> = vec![job.converting.clone()];
    let report = refresh_index_file_report(
        job.candidate.admin.join("index").as_path(),
        job.candidate.path.as_path(),
        &RefreshOptions {
            lock: IndexLock::Held(job.journal.token.as_str().into()),
            references: references.as_slice(),
            hash_length: repository_hash_length(job.candidate.common.as_path()),
        },
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
///  One process the removal phase found inside the converted tree,
///  as the line naming it:
///  its number and the program it runs,
///  the program shown so no byte of it can act on the line.
/// Why:
///   A program names itself to any bytes it likes (`/proc/<pid>/comm`),
///  and this line is how the person finds what still stands in the tree:
///  an escape in the name would move the cursor or erase the line,
///  and a `)` would end the parentheses that say whose name it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (pid, command) => `process ${pid} (${shownSoItCannotRewriteTheLine(command ?? 'unknown', TheWrappingAroundTheName.Parentheses)})`;
/// ```
pub(crate) fn a_process_in_the_converted_tree_line(
    pid: i64,
    command: Option<&str>,
) -> String {
    return format!(
        "process {pid} ({})",
        shown_so_it_cannot_rewrite_the_line(
            command.unwrap_or("unknown"),
            TheWrappingAroundTheName::Parentheses
        )
    );
}

/// What:
///  The removal phase:
/// keep the old tree when a process still holds it,
/// when another command is still filling a stage container below it,
/// or when the installed tree
///  does not hold all of it,
/// and
///  otherwise delete it,
/// but only when it is the very directory the journal recorded.
/// Why:
///   The old tree sits at the converting name from the exchange on,
/// so nothing opens it by a name that anything uses;
/// a process
///  that already held it open is found in the process table,
/// and another command's stage container is found by its claim,
/// because a
///  command that writes into one holds no descriptor there between two entries.
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
pub fn removal_phase(
    platform: &dyn Platform,
    job: &mut Job<'_>,
    interrupt: &Interrupt,
) -> Result<(), MigrateError> {
    let parent: std::fs::File = job.parent()?;
    let scan = processes_using(&UseTarget::Inside(job.converting.as_path()));
    let mut holders: Vec<String> = Vec::new();
    for user in &scan.users {
        holders.push(a_process_in_the_converted_tree_line(
            user.pid,
            user.command.as_deref(),
        ));
    }
    if let Some(why) = scan.unavailable.as_ref() {
        holders.push(format!("the process table cannot be read ({why})"));
    }
    if !holders.is_empty() {
        return Err(MigrateError::Journal(format!(
            "the old tree at {} is kept because {}; stop that and run the command again",
            job.converting.display(),
            holders.join("; ")
        )));
    }
    // Another command's stage container below the old tree is that command's work even though no process holds the tree
    // open: its claim's owner still runs, so nothing here deletes the tree it stands in.
    removal_refusal(
        job.candidate.common.as_path(),
        job.candidate.path.as_path(),
        job.converting.as_path(),
    )?;
    match remove_recorded_old_tree(
        platform,
        &parent,
        job.converting_name(),
        job.journal.old_identity,
        job.candidate.path.as_path(),
        job.journal.synced_ms,
    )? {
        Removal::Removed | Removal::Absent => {}
        Removal::Kept(why) => {
            return Err(MigrateError::Journal(format!(
                "the old tree was not removed: {why}; the journal is kept, so put things right and run the command again"
            )));
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
    crate::worktree_convert_leftover_stage_record::forget_leftover_stage(
        job.candidate.path.as_path(),
    );
    let journal: PathBuf = job.candidate.admin.join(JOURNAL_NAME);
    return io_step(
        std::fs::remove_file(&journal),
        "remove the conversion journal",
        journal.as_path(),
    );
}
