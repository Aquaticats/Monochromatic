//! What:
//!  Undoing a conversion that did not reach its swap:
//! the stage a conversion left beside a worktree whose registration is gone,
//! and an unfinished conversion whose journal still names it.
//! Why:
//!   The `--abandon` mode destroys only what this command recorded it made,
//! and a stage is a directory a person may have a shell or an editor in,
//! so the process table is read before each destroy and again immediately before it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = await abandonSeamed(platform, candidate, journal, say, betweenScans);
//! const leftover = await abandonLeftoverStageSeamed(platform, path, reason, say, afterScan);
//! ```

use crate::line_sink::LineSink;
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
use crate::subvolume_error::MigrateError;
use crate::subvolume_platform::Platform;
use crate::subvolume_remove::remove_tree;
use crate::worktree_convert_inspection::{
    Candidate,
    converting_path,
};
use crate::worktree_convert_journal::{
    Journal,
    Phase,
};
use crate::worktree_convert_leftover_stage_record::{
    forget_leftover_stage,
    leftover_stage_of,
};
use crate::worktree_convert_phases::{
    Job,
    finish,
    identity_of_path,
    stage_identity_of,
};
use crate::worktree_convert_verdict_and_options::Verdict;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  One process whose working directory or open file is inside the directory a conversion destroys,
///  as the line naming it:
///  its number and the program it runs,
///  the program shown so no byte of it can act on the line.
/// Why:
///   A program names itself to any bytes it likes (`/proc/<pid>/comm`),
///  and this line is how the person finds what must be closed before the directory goes:
///  an escape in the name would move the cursor or erase the line,
///  and a `)` would end the parentheses that say whose name it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = (pid, command) => `process ${pid} (${shownSoItCannotRewriteTheLine(command ?? 'unknown', TheWrappingAroundTheName.Parentheses)})`;
/// ```
pub(crate) fn a_process_inside_the_directory_line(
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
        holders.push(a_process_inside_the_directory_line(
            user.pid,
            user.command.as_deref(),
        ));
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
pub(crate) fn abandon_leftover_stage(
    platform: &dyn Platform,
    path: &Path,
    reason: &str,
    say: &mut dyn LineSink,
) -> Result<Option<Verdict>, MigrateError> {
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
pub(crate) fn abandon_leftover_stage_seamed(
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
        return Ok(Some(Verdict::Skipped(vec![format!(
            "{} cannot be examined, so it is left alone",
            converting.display()
        )])));
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
    say.say(
        format!(
            "  {reason}; what is left of that conversion is the copy at {}",
            converting.display()
        )
        .as_str(),
    );
    if platform.is_subvolume_root(converting.as_path())? {
        platform.destroy_subvolume(converting.as_path())?;
    } else {
        MigrateError::from_io_result(
            remove_tree(converting.as_path()),
            "remove the leftover stage",
            converting.as_path(),
        )?;
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
pub(crate) fn abandon(
    platform: &dyn Platform,
    candidate: &Candidate,
    journal: Journal,
    say: &mut dyn LineSink,
) -> Result<Verdict, MigrateError> {
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
pub(crate) fn abandon_seamed(
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
        return Ok(Verdict::Skipped(vec![format!(
            "the stage {} is kept because {}; stop that and run the command again",
            converting.display(),
            holders.join("; ")
        )]));
    }
    between_scans();
    if stage_identity_of(candidate.path.as_path())?.is_some() {
        // An empty stage this conversion created and never recorded is its own as well, so the abandon destroys it instead
        // of leaving the person a directory only a hand can remove.
        if crate::worktree_convert_unrecorded_stage::stage_is_this_conversions(
            &journal,
            candidate.path.as_path(),
        )? {
            if !platform.is_subvolume_root(converting.as_path())? {
                return Err(MigrateError::Journal(format!(
                    "{} is not a subvolume root, so it is not what this conversion created and is left alone",
                    converting.display()
                )));
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
            say.say(
                format!(
                    "  {} is not recorded as made by this conversion, so it was left alone",
                    converting.display()
                )
                .as_str(),
            );
        }
    }
    let job: Job<'_> = Job {
        candidate,
        converting,
        journal,
    };
    finish(&job)?;
    return Ok(Verdict::Abandoned);
}
