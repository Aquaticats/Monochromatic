//! What:
//!  Convert one registered linked worktree that is an ordinary directory into a btrfs subvolume at the same path,
//!  resumably:
//! quiesce it,
//! copy it with cloned data into `<path>.cli-git-converting`,
//! prove the copy equals the original,
//!  swap the two with one atomic exchange,
//! refresh the index stat fields,
//! and remove the old tree only after the swap is
//!  verified.
//! Why:
//!  A snapshot of the main checkout shows a nested worktree that is a subvolume as an empty directory,
//! and one that is an
//!  ordinary directory as a full copy that must then be removed (0.5 s each).
//! Converting each ordinary worktree once makes
//!  every later snapshot skip it for nothing.
//! The registration is untouched:
//! the path and the bytes of the `.git` file do
//!  not change,
//! so the administrative directory's back-pointer stays valid.
//! A journal in the administrative directory
//!  records the phase,
//! so a rerun completes the conversion in place;
//! nothing is deleted that the journal does not prove
//!  this command created.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = await convertWorktree({ platform, path, options, interrupt, say });
//! ```

use crate::fs_beneath::ObjectId;
use crate::line_sink::LineSink;
use crate::single_owner_claim::{
    ClaimOutcome,
    OwnerClaim,
    take_claim,
};
use crate::subvolume_error::{
    Interrupt,
    MigrateError,
};
use crate::subvolume_lock::is_ours;
use crate::subvolume_platform::Platform;
use crate::worktree_convert_abandon::{
    abandon,
    abandon_leftover_stage,
};
#[cfg(test)]
use crate::worktree_convert_abandon::{
    abandon_leftover_stage_seamed,
    abandon_seamed,
};
pub use crate::worktree_convert_inspection::{
    CONVERTING_SUFFIX,
    Candidate,
    Owned,
    Refusal,
    RefusalKind,
    converting_path,
    inspect,
    refusals,
};
use crate::worktree_convert_journal::{
    CLAIM_PREFIX,
    Journal,
    legacy_lock_is_held,
};
pub use crate::worktree_convert_journal::{
    JOURNAL_NAME,
    Phase,
};
use crate::worktree_convert_leftover_stage_record::{
    LeftoverStage,
    record_leftover_stage,
};
use crate::worktree_convert_phases::{
    Job,
    Where,
    copy_phase,
    finish,
    hold_index_lock,
    index_phase,
    old_tree_is_at,
    removal_phase,
    stage_free_identity,
    swap_phase,
    verify_swap,
};
pub use crate::worktree_convert_verdict_and_options::{
    ConvertOptions,
    Verdict,
};
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};

/// A token unique to this run,
/// written to the index lock so that only this command removes it.
fn new_token() -> String {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_before_epoch) => 0,
    };
    return format!(
        "monochromatic-git-policy-cli convert-worktree-to-subvolume pid={} at={nanos}",
        std::process::id()
    );
}

/// What:
///  Convert,
/// resume,
/// abandon or preview one worktree.
/// Why:
///   One function holds the phase order,
/// so the controls exercise the real sequence;
/// the platform and the interrupt are
///  injected so the controls run it on any filesystem and kill it at every phase.
/// A real run holds the conversion lock from
///  before it reads the journal until it ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = await convertWorktree({ platform, path, options, interrupt, say });
/// ```
pub fn convert_worktree(
    platform: &dyn Platform,
    path: &Path,
    options: ConvertOptions,
    interrupt: &Interrupt,
    say: &mut dyn LineSink,
) -> Result<Verdict, MigrateError> {
    // The platform's answer holds for the whole conversion, and a conversion on a platform that flushes turns the
    // flushes back on even inside a stand-in's extent, as a command's does.
    let _flushes: crate::subvolume_file_write_steps::WritesKeptInMemory =
        crate::subvolume_file_write_steps::WritesKeptInMemory::for_flush_policy(
            platform.flushes_to_disk(),
        );
    return convert_in_phases(platform, path, options, interrupt, say);
}

/// The phases of one conversion,
/// which `convert_worktree` wraps in the platform's answer about flushing.
fn convert_in_phases(
    platform: &dyn Platform,
    path: &Path,
    options: ConvertOptions,
    interrupt: &Interrupt,
    say: &mut dyn LineSink,
) -> Result<Verdict, MigrateError> {
    let candidate: Candidate = match inspect(path) {
        Ok(found) => found,
        Err(reason) => {
            // A path without a registration can still own the stage of a conversion the removal cut off, and the abandon is
            // the one verb that may destroy it.
            if options.abandon
                && let Some(verdict) = abandon_leftover_stage(platform, path, reason.as_str(), say)?
            {
                return Ok(verdict);
            }
            return Ok(Verdict::Skipped(vec![reason]));
        }
    };
    // A dry run changes nothing, so it needs no claim; every other mode owns the conversion to the end. The claim is taken
    // before the journal is read, so two commands never read, write or resume one journal.
    let _claim: Option<OwnerClaim> = if options.dry_run {
        None
    } else {
        match take_claim(
            candidate.admin.as_path(),
            std::ffi::OsStr::new(CLAIM_PREFIX),
        )? {
            ClaimOutcome::Held(held) => {
                if legacy_lock_is_held(candidate.admin.as_path())? {
                    return Ok(Verdict::Skipped(vec![format!(
                        "another conversion of {} is running (a command of release 0.5.20 or older holds {}), so this one did not start",
                        candidate.path.display(),
                        candidate
                            .admin
                            .join(crate::worktree_convert_journal::LOCK_NAME)
                            .display()
                    )]));
                }
                Some(held)
            }
            ClaimOutcome::Busy(owner) => {
                return Ok(Verdict::Skipped(vec![format!(
                    "another conversion of {} is running (process {} owns it through {}), so this one did not start",
                    candidate.path.display(),
                    owner.pid,
                    owner.path.display()
                )]));
            }
        }
    };
    let mut journal: Option<Journal> = Journal::load(candidate.admin.as_path())?;
    // A journal belongs to the path the conversion began at; a worktree moved since then is refused before anything changes.
    if let Some(found) = journal.as_ref()
        && found.path.as_slice() != candidate.path.as_os_str().as_bytes()
    {
        return Err(MigrateError::Journal(format!(
            "an unfinished conversion began at {}, but the worktree is at {} now; it was moved while the conversion was unfinished, so nothing was changed; move it back and run the command again",
            String::from_utf8_lossy(found.path.as_slice()),
            candidate.path.display()
        )));
    }
    // A kill between the exchange and its journal line leaves the journal one phase behind; bring it up to date in memory in
    // every mode, and on disk when the run changes anything.
    if let Some(found) = journal.as_mut()
        && found.phase == Phase::Synced
    {
        let converting: PathBuf = converting_path(candidate.path.as_path());
        if old_tree_is_at(found, candidate.path.as_path(), converting.as_path())
            == Where::AtConverting
        {
            found.phase = Phase::Exchanged;
            if !options.dry_run {
                found.save(candidate.admin.as_path())?;
            }
        }
    }
    if options.abandon {
        return match journal {
            Some(found) => abandon(platform, &candidate, found, say),
            None => Ok(Verdict::Skipped(vec![String::from(
                "no conversion is in progress here, so there is nothing to abandon",
            )])),
        };
    }
    if journal.is_none() && platform.is_subvolume_root(candidate.path.as_path())? {
        return Ok(Verdict::AlreadySubvolume);
    }
    let resuming: bool = journal.is_some();
    // A conversion that has not reached its swap is checked again like a fresh one; past the swap the old path is gone.
    let must_be_quiet: bool = match journal.as_ref() {
        Some(found) => found.phase < Phase::Exchanged,
        None => true,
    };
    if must_be_quiet {
        // Only what this command owns is exempt: the index lock that holds its token, and the stage whose identity it recorded.
        let owned: Owned = match journal.as_ref() {
            Some(found) => Owned {
                index_lock: is_ours(
                    candidate.admin.join("index.lock").as_path(),
                    found.token.as_str(),
                ),
                stage: crate::worktree_convert_unrecorded_stage::stage_is_this_conversions(
                    found,
                    candidate.path.as_path(),
                )?,
            },
            None => Owned::default(),
        };
        let reasons: Vec<Refusal> =
            refusals(platform, &candidate, owned, options.command_line_attributes);
        if !reasons.is_empty() {
            let mut texts: Vec<String> = Vec::new();
            for reason in reasons {
                texts.push(reason.text);
            }
            return Ok(Verdict::Skipped(texts));
        }
    }
    if options.dry_run {
        return Ok(if resuming {
            Verdict::Resumed
        } else {
            Verdict::WouldConvert
        });
    }
    let old: ObjectId = stage_free_identity(candidate.path.as_path())?;
    let token: String = match journal.as_ref() {
        Some(found) => found.token.clone(),
        None => new_token(),
    };
    let started: Journal = journal.unwrap_or(Journal {
        phase: Phase::Started,
        token,
        path: candidate.path.as_os_str().as_bytes().to_vec(),
        old_identity: old,
        git_file: candidate.git_file.clone(),
        new_identity: None,
        synced_ms: None,
    });
    let converting: PathBuf = converting_path(candidate.path.as_path());
    let mut job: Job<'_> = Job {
        candidate: &candidate,
        converting,
        journal: started,
    };
    if !resuming {
        // The journal first, made exclusively: the lock and the subvolume exist only after the journal names them.
        job.journal.create_new(candidate.admin.as_path())?;
    }
    if job.journal.phase < Phase::Exchanged {
        copy_phase(platform, &mut job, interrupt, say)?;
        // Both objects this conversion may leave at the converting name are recorded beside the worktree now, because the
        // journal that names them dies with the registration.
        record_leftover_stage(
            candidate.path.as_path(),
            &LeftoverStage {
                original: job.journal.old_identity,
                created: job.journal.new_identity,
            },
        );
        swap_phase(platform, &mut job, interrupt)?;
    }
    verify_swap(platform, &job)?;
    if job.journal.phase < Phase::Refreshed {
        // A run that stopped after the swap still owes the index rewrite, which needs the lock; a release up to 0.5.20 left it
        // in place from the start of the conversion, and taking it again is accepted when it holds this token.
        hold_index_lock(&job)?;
        index_phase(&mut job, interrupt, say)?;
    }
    if job.journal.phase < Phase::Removed {
        removal_phase(platform, &mut job, interrupt)?;
    }
    finish(&job)?;
    return Ok(if resuming {
        Verdict::Resumed
    } else {
        Verdict::Converted
    });
}

/// Conversion controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_tests.rs"]
mod tests;

/// The cleanliness proof of the conversion by content stays out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_content_tests.rs"]
mod content_tests;

/// The controls for who owns a conversion and how long the index lock is held stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_ownership_tests.rs"]
mod ownership_tests;

/// The controls for what keeps a foreign or late change from being lost:
/// identities,
/// the lock,
/// the journal's files,
/// the pinned
///  removal and the refusals by kind.
#[cfg(test)]
#[path = "worktree_convert_safety_tests.rs"]
mod safety_tests;
