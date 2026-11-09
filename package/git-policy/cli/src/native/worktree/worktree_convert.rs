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
use crate::process_use::{UseTarget, processes_using};
use crate::single_owner_claim::{ClaimOutcome, OwnerClaim, take_claim};
use crate::subvolume_error::{Interrupt, MigrateError};
use crate::subvolume_lock::is_ours;
use crate::subvolume_platform::Platform;
use crate::subvolume_remove::remove_tree;
pub use crate::worktree_convert_inspection::{CONVERTING_SUFFIX, Candidate, Owned, Refusal, RefusalKind, converting_path, inspect, refusals};
use crate::worktree_convert_journal::{CLAIM_PREFIX, Journal, legacy_lock_is_held};
pub use crate::worktree_convert_journal::{JOURNAL_NAME, Phase};
use crate::worktree_convert_leftover_stage_record::{LeftoverStage, forget_leftover_stage, leftover_stage_of, record_leftover_stage};
use crate::worktree_convert_phases::{Job, Where, copy_phase, finish, hold_index_lock, identity_of_path, index_phase, old_tree_is_at, removal_phase, stage_free_identity, stage_identity_of, swap_phase, verify_swap};
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

/// What:
///  The processes that may be working inside `directory`,
/// as sentences.
/// Why:
///   The abandon and the removal phase both destroy a directory a person may have a shell or an editor in,
/// and a process whose working directory goes away is a failure the person did not ask for.
/// A scan that cannot see everything counts as a holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holders: string[] = processesInside(directory);
/// ```
fn processes_inside(directory: &Path) -> Vec<String> {
    let scan = processes_using(&UseTarget::Inside(directory));
    let mut holders: Vec<String> = Vec::new();
    for user in &scan.users {
        holders.push(format!("process {} ({})", user.pid, user.command.as_deref().unwrap_or("unknown")));
    }
    if let Some(why) = scan.unavailable.as_ref() {
        holders.push(format!("the process table cannot be read ({why})"));
    }
    return holders;
}

/// What:
///  Destroy the stage of a conversion whose worktree is gone and whose journal went with its registration.
/// Why:
///   A conversion interrupted past its swap keeps the original tree at `<path>.cli-git-converting` on purpose,
/// and the journal
///  that alone records it lives in the registration,
/// so removing the worktree afterwards leaves a full copy beside where it
///  was that no other mode can name:
/// `inspect` refuses a path without a `.git` before any mode looks further.
/// The name is
///  one only this command makes,
/// so the person names it by naming the worktree,
/// and `--abandon` is their word for
///  discarding what is there:
/// a subvolume root is destroyed,
/// which is what a stop before the swap leaves,
/// and  any other directory is removed as a tree,
/// which is the original directory a stop after the swap leaves,
/// with the nested-subvolume and mount-point boundaries of every tree removal.
/// A process inside the stage keeps it,
/// as the removal phase does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verdict = await abandonLeftoverStage(platform, path, reason, say); // undefined when there is no stage
/// ```
fn abandon_leftover_stage(platform: &dyn Platform, path: &Path, reason: &str, say: &mut dyn LineSink) -> Result<Option<Verdict>, MigrateError> {
    return abandon_leftover_stage_seamed(platform, path, reason, say, &mut nothing_at_all);
}

/// What:
///  `abandon_leftover_stage`,
/// with a call after the process scan and before the last look at the identity,
/// where a control
///  renames the recorded stage aside and puts another directory at its name.
/// Why:
///   The identity check and the destroy are not one step,
/// and binding them needs a destroy that takes a descriptor,
/// which
///  the platform does not offer;
/// the last look is placed immediately before the destroy,
/// so the window is one check wide,
/// and
///  a test can only enter it through a seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abandonLeftoverStageSeamed(platform, path, reason, say, afterScan);
/// ```
fn abandon_leftover_stage_seamed(
    platform: &dyn Platform,
    path: &Path,
    reason: &str,
    say: &mut dyn LineSink,
    after_scan: &mut dyn FnMut(),
) -> Result<Option<Verdict>, MigrateError> {
    let converting: PathBuf = converting_path(path);
    match std::fs::symlink_metadata(&converting) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(MigrateError::io("examine", converting.as_path(), &error)),
        Ok(_) => {}
    }
    // The suffix proves nothing by itself: the record beside the worktree says which objects a conversion may have left
    // there, and anything else is somebody's own directory.
    let Some(recorded) = leftover_stage_of(path) else {
        return Ok(Some(Verdict::Skipped(vec![format!(
            "{} exists, and no record beside the worktree says this conversion made it, so it is left alone: look at it, and remove it yourself if it is a copy you do not want",
            converting.display()
        )])));
    };
    let Some(found) = identity_of_path(&converting) else {
        return Ok(Some(Verdict::Skipped(vec![format!("{} cannot be examined, so it is left alone", converting.display())])));
    };
    if found != recorded.original && recorded.created != Some(found) {
        return Ok(Some(Verdict::Skipped(vec![format!(
            "{} is not the stage this conversion recorded (it is {}, and the record names {} and {}), so it is left alone: look at it, and remove it yourself if it is a copy you do not want",
            converting.display(),
            found.encode(),
            recorded.original.encode(),
            match recorded.created {
                Some(created) => created.encode(),
                None => String::from("nothing else"),
            }
        )])));
    }
    let holders: Vec<String> = processes_inside(converting.as_path());
    if !holders.is_empty() {
        return Ok(Some(Verdict::Skipped(vec![format!(
            "the leftover stage {} is kept because {}; stop that and run the command again",
            converting.display(),
            holders.join("; ")
        )])));
    }
    after_scan();
    // The last look at which object the name holds, immediately before the destroy: a directory renamed in and put there by
    // another process is not one this conversion recorded, and it is kept.
    let now: Option<crate::fs_beneath::ObjectId> = identity_of_path(&converting);
    if now != Some(found) {
        return Ok(Some(Verdict::Skipped(vec![format!(
            "{} changed while the command was about to destroy it (it is {}, and the record names {}), so it is left alone: look at it, and remove it yourself if it is a copy you do not want",
            converting.display(),
            match now {
                Some(seen) => seen.encode(),
                None => String::from("a name that leads nowhere"),
            },
            found.encode()
        )])));
    }
    say.say(format!("  {reason}; what is left of that conversion is the copy at {}", converting.display()).as_str());
    if platform.is_subvolume_root(converting.as_path())? {
        platform.destroy_subvolume(converting.as_path())?;
    } else {
        MigrateError::from_io_result(remove_tree(converting.as_path()), "remove the leftover stage", converting.as_path())?;
    }
    say.say(format!("  destroyed the leftover stage {}", converting.display()).as_str());
    forget_leftover_stage(path);
    return Ok(Some(Verdict::Abandoned));
}

/// Undo an unfinished conversion that has not reached its swap:
/// destroy the subvolume this command created (the one whose
///  identity the journal recorded;
/// anything else at the name is left where it is),
/// release the lock and drop the journal.
///  Past the swap there is nothing to undo,
/// and the command says to run it again.
fn abandon(platform: &dyn Platform, candidate: &Candidate, journal: Journal, say: &mut dyn LineSink) -> Result<Verdict, MigrateError> {
    return abandon_seamed(platform, candidate, journal, say, &mut nothing_at_all);
}

/// A step that does nothing,
/// for the run between the two holder scans of an abandon.
fn nothing_at_all() {}

/// What:
///  `abandon`,
/// with a call between its two scans of the process table,
/// where a control lets a process enter the stage.
/// Why:
///   The scan and the destroy are not one step,
/// and a process that enters the stage in that window loses its working
///  directory.
/// The second scan is placed immediately before the destroy,
/// so the window is one check wide,
/// and a test can
///  only enter it through a seam.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abandonSeamed(platform, candidate, journal, say, betweenScans);
/// ```
fn abandon_seamed(
    platform: &dyn Platform,
    candidate: &Candidate,
    journal: Journal,
    say: &mut dyn LineSink,
    between_scans: &mut dyn FnMut(),
) -> Result<Verdict, MigrateError> {
    let converting: PathBuf = converting_path(candidate.path.as_path());
    if journal.phase >= Phase::Exchanged {
        say.say("  the swap already happened; run the command again to finish it (there is nothing to undo)");
        return Ok(Verdict::Skipped(vec![String::from("past the swap")]));
    }
    // The stage is a directory a person may have a shell or an editor in, and destroying it takes the working directory of
    // that process with it, so the same scan the removal phase runs comes first here too.
    let holders: Vec<String> = processes_inside(converting.as_path());
    if !holders.is_empty() {
        return Ok(Verdict::Skipped(vec![format!("the stage {} is kept because {}; stop that and run the command again", converting.display(), holders.join("; "))]));
    }
    between_scans();
    if let Some(found) = stage_identity_of(candidate.path.as_path())? {
        // An empty stage this conversion created and never recorded is its own as well, so the abandon destroys it instead
        // of leaving the person a directory only a hand can remove.
        let recorded: bool = journal.new_identity == Some(found);
        if recorded || crate::worktree_convert_phases::owns_unrecorded_stage(&journal, candidate.path.as_path())? {
            if !platform.is_subvolume_root(converting.as_path())? {
                return Err(MigrateError::Journal(format!("{} is not a subvolume root, so it is not what this conversion created and is left alone", converting.display())));
            }
            // The last look before the call, as the removal phase takes it: the window is one check wide and cannot be closed
            // without a lock the owner does not want.
            let arrived: Vec<String> = processes_inside(converting.as_path());
            if !arrived.is_empty() {
                return Ok(Verdict::Skipped(vec![format!(
                    "the stage {} is kept because {} entered it while the command was about to destroy it; stop that and run the command again",
                    converting.display(),
                    arrived.join("; ")
                )]));
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
        Err(reason) => {
            // A path without a registration can still own the stage of a conversion the removal cut off, and the abandon is
            // the one verb that may destroy it.
            if options.abandon && let Some(verdict) = abandon_leftover_stage(platform, path, reason.as_str(), say)? {
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
                stage: (found.new_identity.is_some() && stage_identity_of(candidate.path.as_path())? == found.new_identity)
                    || crate::worktree_convert_phases::owns_unrecorded_stage(found, candidate.path.as_path())?,
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
        // Both objects this conversion may leave at the converting name are recorded beside the worktree now, because the
        // journal that names them dies with the registration.
        record_leftover_stage(candidate.path.as_path(), &LeftoverStage { original: job.journal.old_identity, created: job.journal.new_identity });
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
