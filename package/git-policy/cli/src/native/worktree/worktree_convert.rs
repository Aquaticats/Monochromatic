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

use crate::line_sink::LineSink;
use crate::fs_beneath::ObjectId;
use crate::single_owner_claim::{ClaimOutcome, OwnerClaim, take_claim};
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_lock::is_ours;
use crate::subvolume_platform::Platform;
pub use crate::worktree_convert_inspection::{CONVERTING_SUFFIX, Candidate, Owned, Refusal, RefusalKind, converting_path, inspect, refusals};
use crate::worktree_convert_journal::{CLAIM_PREFIX, Journal, legacy_lock_is_held};
pub use crate::worktree_convert_journal::{JOURNAL_NAME, Phase};
use crate::worktree_convert_phases::{Job, Where, copy_phase, finish, hold_index_lock, index_phase, old_tree_is_at, removal_phase, stage_free_identity, stage_identity_of, swap_phase, verify_swap};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// What:
///  What a run did with one worktree.
/// Why:
///   The command prints one line per worktree and maps the verdicts to its exit status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = 'converted' | 'resumed' | 'would-convert' | 'already-subvolume' | 'abandoned' | { skipped: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The conversion ran from the start to the end.
    Converted,
    /// An unfinished conversion was completed.
    Resumed,
    /// A dry run found nothing in the way.
    WouldConvert,
    /// The path is a subvolume already.
    AlreadySubvolume,
    /// An unfinished conversion was undone before its swap.
    Abandoned,
    /// Something stands in the way;
    /// each reason is a sentence.
    Skipped(Vec<String>),
}

/// What:
///  What the command asks of one worktree.
/// Why:
///   The dry run and the abandon are modes of the same pass over the same facts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConvertOptions = { dryRun: boolean; abandon: boolean; commandLineAttributes: boolean };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConvertOptions {
    /// List what would happen and change nothing.
    pub dry_run: bool,
    /// Undo an unfinished conversion that has not reached its swap.
    pub abandon: bool,
    /// The invocation chose an attribute source on the command line (`--attr-source`,
    /// `-c attr.tree`),
    /// which the worktree
    ///  files read here do not model.
    pub command_line_attributes: bool,
}

/// A token unique to this run,
/// written to the index lock so that only this command removes it.
fn new_token() -> String {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_before_epoch) => 0,
    };
    return format!("monochromatic-git-policy-cli convert-worktree-to-subvolume pid={} at={nanos}", std::process::id());
}

/// Undo an unfinished conversion that has not reached its swap:
/// destroy the subvolume this command created (the one whose
///  identity the journal recorded;
/// anything else at the name is left where it is),
/// release the lock and drop the journal.
///  Past the swap there is nothing to undo,
/// and the command says to run it again.
fn abandon(platform: &dyn Platform, candidate: &Candidate, journal: Journal, say: &mut dyn LineSink) -> Result<Verdict, MigrateError> {
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if journal.phase >= Phase::Exchanged {
        say.say("  the swap already happened; run the command again to finish it (there is nothing to undo)");
        return Ok(Verdict::Skipped(vec![String::from("past the swap")]));
    }
    if let Some(found) = stage_identity_of(candidate.path.as_path())? {
        if journal.new_identity == Some(found) {
            if !platform.is_subvolume_root(converting.as_path())? {
                return Err(MigrateError::Journal(format!("{} is not a subvolume root, so it is not what this conversion created and is left alone", converting.display())));
            }
            platform.destroy_subvolume(converting.as_path())?;
        } else {
            say.say(format!("  {} is not recorded as made by this conversion, so it was left alone", converting.display()).as_str());
        }
    }
    let job: Job<'_> = Job { candidate, converting, journal };
    finish(&job)?;
    return Ok(Verdict::Abandoned);
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
pub fn convert_worktree(platform: &dyn Platform, path: &Path, options: ConvertOptions, interrupt: &Interrupt, say: &mut dyn LineSink) -> Result<Verdict, MigrateError> {
    // The platform's answer holds for the whole conversion, and a conversion on a platform that flushes turns the
    // flushes back on even inside a stand-in's extent, as a command's does.
    let _flushes: crate::subvolume_file_write_steps::WritesKeptInMemory =
        crate::subvolume_file_write_steps::WritesKeptInMemory::for_flush_policy(platform.flushes_to_disk());
    return convert_in_phases(platform, path, options, interrupt, say);
}

/// The phases of one conversion,
/// which `convert_worktree` wraps in the platform's answer about flushing.
fn convert_in_phases(platform: &dyn Platform, path: &Path, options: ConvertOptions, interrupt: &Interrupt, say: &mut dyn LineSink) -> Result<Verdict, MigrateError> {
    let candidate: Candidate = match inspect(path) {
        Ok(found) => found,
        Err(reason) => return Ok(Verdict::Skipped(vec![reason])),
    };
    // A dry run changes nothing, so it needs no claim; every other mode owns the conversion to the end. The claim is taken
    // before the journal is read, so two commands never read, write or resume one journal.
    let _claim: Option<OwnerClaim> = if options.dry_run {
        None
    } else {
        match take_claim(candidate.admin.as_path(), std::ffi::OsStr::new(CLAIM_PREFIX))? {
            ClaimOutcome::Held(held) => {
                if legacy_lock_is_held(candidate.admin.as_path())? {
                    return Ok(Verdict::Skipped(vec![format!(
                        "another conversion of {} is running (a command of release 0.5.20 or older holds {}), so this one did not start",
                        candidate.path.display(),
                        candidate.admin.join(crate::worktree_convert_journal::LOCK_NAME).display()
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
        if old_tree_is_at(found, candidate.path.as_path(), converting.as_path()) == Where::AtConverting {
            found.phase = Phase::Exchanged;
            if !options.dry_run {
                found.save(candidate.admin.as_path())?;
            }
        }
    }
    if options.abandon {
        return match journal {
            Some(found) => abandon(platform, &candidate, found, say),
            None => Ok(Verdict::Skipped(vec![String::from("no conversion is in progress here, so there is nothing to abandon")])),
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
                index_lock: is_ours(candidate.admin.join("index.lock").as_path(), found.token.as_str()),
                stage: found.new_identity.is_some() && stage_identity_of(candidate.path.as_path())? == found.new_identity,
            },
            None => Owned::default(),
        };
        let reasons: Vec<Refusal> = refusals(platform, &candidate, owned, options.command_line_attributes);
        if !reasons.is_empty() {
            let mut texts: Vec<String> = Vec::new();
            for reason in reasons {
                texts.push(reason.text);
            }
            return Ok(Verdict::Skipped(texts));
        }
    }
    if options.dry_run {
        return Ok(if resuming { Verdict::Resumed } else { Verdict::WouldConvert });
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
    let mut job: Job<'_> = Job { candidate: &candidate, converting, journal: started };
    if !resuming {
        // The journal first, made exclusively: the lock and the subvolume exist only after the journal names them.
        job.journal.create_new(candidate.admin.as_path())?;
    }
    if job.journal.phase < Phase::Exchanged {
        copy_phase(platform, &mut job, interrupt, say)?;
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
    return Ok(if resuming { Verdict::Resumed } else { Verdict::Converted });
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
