//! What:
//!  What an abort or a skip copies before it runs,
//!  and the comparison the wrapper makes immediately before Git starts.
//! Why:
//!  The owner's rule is "save,
//!  then allow",
//!  and the copy has to be of what Git is about to discard:
//!  the worktree files the listing named,
//!  and the content that stands only in the index,
//!  which no worktree file holds.
//!  A file can also change after it was copied and before Git starts,
//!  and that edit is discarded with no copy,
//!  so each saved source is compared once more at the last moment the wrapper controls,
//!  and whatever changed is copied again:
//!  a bounded number of rounds,
//!  so a file another program keeps writing cannot hold the command for good,
//!  after which the command is refused and the file is named.
//!  What remains after that comparison is a window no program outside Git can close,
//!  and `SPEC.md` and the notice say so in plain words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const saved = saveBeforeTheCommand({ root, commonDir, git, paths, stagedPaths, milliseconds });
//! planTheLastChance(saved.plan);
//! // later, immediately before Git starts:
//! const extra = saveAgainBeforeTheCommand(); // copied lines, or throws
//! ```

use crate::conflict_side_source::{FailureKind, SaveFailure, source_still_holds, still_nothing_there};
use crate::conflict_sides::{SaveAttempt, SavedCopy, SavedSource, fresh_saved_directory, save_into};
use crate::forwarding::git_command;
use crate::hooks_off::StartOwner;
use crate::private_storage::{create_private_directory_all, sync_directory};
use crate::transaction_git::{GitContext, input_file};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;

/// How many times one saved source that changed again may be copied again
///  before the command is refused.
/// Why:
///   One round catches a write that landed while the first copy was made,
///  a second catches one that landed while that copy was made,
///  and the count stops there:
///  a file another program writes in a loop must not hold the abort for good,
///  and a refusal loses nothing.
///  A round copies only the sources that changed,
///  so three of them cost what three reads of those files cost.
pub const SAVE_AGAIN_ROUNDS: u8 = 3;

/// The name of the folder the index's own content is copied into,
///  beside the worktree copies.
/// Why:
///   A person looking for the content of a staged file finds it at the path the file has,
///  and the folder says which stage it came from,
///  because a worktree copy of a path and the index's copy cannot share a name.
pub const STAGED_FOLDER: &str = "index-stage-0";

/// The folder a first save puts a source that changed while it was copied into.
const FIRST_SAVE_AGAIN_FOLDER: &str = "again";

/// The folder the last comparison puts a source that changed since into.
const LAST_CHANCE_AGAIN_FOLDER: &str = "later";

/// What the last chance found:
///  nothing to copy,
///  or the copies it made and the sentences that name them.
type LastChance = Result<Option<String>, String>;

/// What:
///  What a command saved before it runs, and what the last comparison needs.
/// Why:
///   The copies go into the notice and the audit record,
///  and the sources the wrapper must compare again travel with the command
///  until the instant before Git starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SavedBeforeCommand = { copies: SavedCopy[]; plan?: SaveAgainPlan };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SavedBeforeCommand {
    /// Every copy this command's saves made,
    ///  in the order they were made.
    pub copies: Vec<SavedCopy>,
    /// The comparison the wrapper owes immediately before Git starts,
    ///  or nothing when it copied no worktree file.
    pub plan: Option<SaveAgainPlan>,
}

/// What:
///  The sources a command saved, and where a further copy of one belongs.
/// Why:
///   A second copy is only warranted when the source no longer matches what was copied,
///  and the comparison needs the path as Git printed it,
///  what stood there then,
///  the worktree top it is read from,
///  and the folder this command's copies are in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SaveAgainPlan = { root: string; directory: string; sources: SavedSource[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveAgainPlan {
    /// The worktree top the sources are read from.
    pub root: PathBuf,
    /// The folder this command's copies are in.
    pub directory: PathBuf,
    /// Each source whose copy was made, with what stood there then.
    pub sources: Vec<SavedSource>,
}

/// What:
///  The plan one invocation's last comparison still owes,
///  which is at most one:
///  one command discards once.
/// Why:
///   The comparison has to happen where the wrapper starts the command the person typed,
///  and that start is several layers below the policy that made the copies;
///  handing the plan over in one place the wrapper owns keeps every layer between them
///  from having to carry it,
///  and taking it out means a start the system refused and the wrapper repeats compares once.
static PENDING: Mutex<Option<SaveAgainPlan>> = Mutex::new(None);

/// What:
///  Record the comparison this command's copies owe before Git starts.
/// Why:
///   The policy pass makes the copies and has no way to run code after them,
///  so the plan waits here for the start that hands the command to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planTheLastChance(plan: SaveAgainPlan): void;
/// ```
pub fn plan_the_last_chance(plan: SaveAgainPlan) {
    let mut pending: std::sync::MutexGuard<'_, Option<SaveAgainPlan>> =
        PENDING.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); });
    *pending = Some(plan);
}

/// What:
///  Take the pending plan, if there is one.
/// Why:
///   Only the first start of the person's command may compare and copy:
///  a command the wrapper starts again after a refused start reads the same copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = takeTheLastChance();
/// ```
fn take_the_last_chance() -> Option<SaveAgainPlan> {
    let mut pending: std::sync::MutexGuard<'_, Option<SaveAgainPlan>> =
        PENDING.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); });
    return pending.take();
}

/// What:
///  Forget a plan that no command will use,
///  which a control does before it checks that nothing is pending.
/// Why:
///   A plan is one process's own state,
///  and a run that ends without starting Git leaves it;
///  no shipped path needs to forget one,
///  and a control that does says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheLastChance();
/// ```
#[cfg(test)]
pub fn forget_the_last_chance() {
    let _taken: Option<SaveAgainPlan> = take_the_last_chance();
}

/// What:
///  Compare each saved source with what was copied,
///  copy again whatever changed,
///  and refuse when a source will not hold still.
/// Why:
///   This is the last moment the wrapper controls before Git discards the files,
///  so it is where an edit that landed after the first copy is caught.
///  A source that changed is copied again into a numbered folder beside the first copy,
///  so the earlier copy is never overwritten and nobody is told of a copy that replaced another;
///  the rounds are bounded,
///  and the refusal names the file,
///  says nothing ran,
///  and names the copies that were made.
///  A plan that is not there means nothing was copied, and nothing is done.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const extra = saveAgainBeforeTheCommand(); // copied lines, or throws
/// ```
pub fn save_again_before_the_command() -> LastChance {
    let Some(plan) = take_the_last_chance() else {
        return Ok(None);
    };
    let mut sources: Vec<SavedSource> = plan.sources;
    let mut notices: Vec<String> = Vec::<String>::new();
    let mut round: u8 = 0;
    loop {
        let changed: Vec<SavedSource> = changed_sources(plan.root.as_path(), sources.as_slice());
        if changed.is_empty() {
            return Ok(join_notices(notices.as_slice()));
        }
        if round >= SAVE_AGAIN_ROUNDS {
            return Err(still_changing_text(changed.as_slice(), plan.directory.as_path()));
        }
        round += 1;
        let again: PathBuf = plan.directory.join(format!("{LAST_CHANCE_AGAIN_FOLDER}-{round}"));
        let paths: Vec<Vec<u8>> = changed.iter().map(|source| { return source.path.clone(); }).collect();
        let attempt: SaveAttempt = match save_into(plan.root.as_path(), again.as_path(), paths.as_slice()) {
            Ok(saved) => saved,
            Err(failure) => return Err(refused_save_text(&failure, plan.directory.as_path(), changed.as_slice())),
        };
        let Some(failure) = attempt.changed else {
            notices.push(second_copy_notice(changed.as_slice(), attempt.copies.as_slice(), again.as_path()));
            sources = merge_sources(sources.as_slice(), attempt.sources.as_slice());
            continue;
        };
        return Err(refused_save_text(&failure, plan.directory.as_path(), changed.as_slice()));
    }
}

/// What:
///  The sources whose content is no longer what their copy holds.
/// Why:
///   A path the worktree lost,
///  gained or rewrote is one this command would discard
///  without the copy holding it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const changed = changedSources(root, sources);
/// ```
fn changed_sources(root: &Path, sources: &[SavedSource]) -> Vec<SavedSource> {
    let mut changed: Vec<SavedSource> = Vec::<SavedSource>::new();
    for source in sources {
        let held: bool = match source.stamp {
            Some(stamp) => source_still_holds(root, source.path.as_slice(), stamp),
            // Nothing stood there when the copy was made, so a file that appeared since is work this command would discard.
            None => still_nothing_there(root, source.path.as_slice()),
        };
        if !held {
            changed.push(source.clone());
        }
    }
    return changed;
}

/// What:
///  The sources of `before` with the stamps of the sources `saved` read again.
/// Why:
///   A round copies only what changed,
///  so the sources it did not touch keep the stamps the earlier rounds proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const merged = mergeSources({ before, saved });
/// ```
fn merge_sources(before: &[SavedSource], saved: &[SavedSource]) -> Vec<SavedSource> {
    let mut merged: Vec<SavedSource> = Vec::<SavedSource>::with_capacity(before.len());
    for source in before {
        let fresh: Option<&SavedSource> = saved.iter().find(|fresh| { return fresh.path == source.path; });
        merged.push(match fresh {
            Some(read) => read.clone(),
            None => source.clone(),
        });
    }
    return merged;
}

/// The paths of `sources`, joined for a sentence.
fn names(sources: &[SavedSource]) -> String {
    let mut listed: Vec<String> = Vec::<String>::new();
    for source in sources {
        listed.push(String::from_utf8_lossy(source.path.as_slice()).into_owned());
    }
    return listed.join(", ");
}

/// The sentence that names a second copy, which the person reads before Git runs.
fn second_copy_notice(changed: &[SavedSource], copies: &[SavedCopy], folder: &Path) -> String {
    let count: usize = copies.len();
    return format!(
        "monochromatic-git-policy-cli: {} changed again after the wrapper copied {}, so a second copy of {} was saved in {} before the command \
         starts.\n",
        names(changed),
        if count == 1 { "it" } else { "them" },
        if count == 1 { "it" } else { "each" },
        folder.display()
    );
}

/// The notices of the rounds that copied again, as one block.
fn join_notices(notices: &[String]) -> Option<String> {
    if notices.is_empty() {
        return None;
    }
    return Some(notices.concat());
}

/// What:
///  The text of a refused command whose source changed while a further copy was being made.
/// Why:
///   The person is told which file,
///  that nothing ran,
///  and where the copies already made are,
///  so nothing that was saved is lost from their view either.
fn refused_save_text(failure: &SaveFailure, directory: &Path, changed: &[SavedSource]) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} changed again while the wrapper was copying {} and could not be saved again ({}: {}), so the command \
         was refused and nothing was changed. The copies that were made stand in {}. Run the command again once nothing is writing to {}, or save \
         it yourself and pass `--no-enforce-worktree` directly after the subcommand.",
        names(changed),
        if changed.len() == 1 { "it" } else { "them" },
        failure.subject,
        failure.reason,
        directory.display(),
        names(changed)
    );
}

/// What:
///  The text of a refused command whose source kept changing through every round.
/// Why:
///   A file another program writes in a loop would otherwise hold the command for good;
///  the person is told which file,
///  that nothing ran,
///  where the copies are,
///  and what ends the loop.
fn still_changing_text(changed: &[SavedSource], directory: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} kept changing while the wrapper copied {}, through {SAVE_AGAIN_ROUNDS} rounds, so the command was \
         refused and nothing was changed. The copies that were made stand in {}. Run the command again once nothing is writing to {} (an editor, a \
         build or a log may be), or save it yourself and pass `--no-enforce-worktree` directly after the subcommand.",
        names(changed),
        if changed.len() == 1 { "it" } else { "them" },
        directory.display(),
        names(changed)
    );
}

/// What:
///  The refusal of a source that changed through every round of the first save.
/// Why:
///   The save step's own reason says the source changed while it was copied;
///  the person also needs to know that nothing was changed and where the copies are.
fn changed_source_failure(failure: &SaveFailure, directory: &Path) -> SaveFailure {
    return SaveFailure {
        subject: failure.subject.clone(),
        reason: format!(
            "{}; it changed through {SAVE_AGAIN_ROUNDS} rounds of copying, so nothing was changed and the copies that were made stand in {}",
            failure.reason,
            directory.display()
        ),
        kind: FailureKind::Changed,
    };
}

/// What:
///  Save the worktree files and the content the index holds, then say what the last comparison owes.
///  `&[Vec<u8>]` borrows the paths the listing named and the ones no worktree file holds.
/// Why:
///   The order is the owner's: everything the command would discard is on disk before it runs.
///  A source that changes while it is copied is copied again,
///  within the same bound the last comparison uses,
///  and one that never holds still is a refusal that names it.
///  The content of a path the worktree lost while the index kept it is copied from the index itself,
///  into a folder that names the stage.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const saved = saveBeforeTheCommand({ root, commonDir, git, paths, stagedPaths, milliseconds });
/// ```
pub fn save_before_the_command(
    root: &Path,
    common_dir: &Path,
    git: &GitContext,
    paths: &[Vec<u8>],
    staged_paths: &[Vec<u8>],
    milliseconds: i64,
) -> Result<SavedBeforeCommand, SaveFailure> {
    let directory: PathBuf = match fresh_saved_directory(common_dir, milliseconds) {
        Ok(made) => made,
        Err(error) => return Err(SaveFailure::io(common_dir.display().to_string(), error.to_string())),
    };
    let mut copies: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    let mut sources: Vec<SavedSource> = Vec::<SavedSource>::new();
    let mut remaining: Vec<Vec<u8>> = paths.to_vec();
    let mut round: u8 = 0;
    while !remaining.is_empty() {
        let into: PathBuf = if round == 0 {
            directory.clone()
        } else {
            directory.join(format!("{FIRST_SAVE_AGAIN_FOLDER}-{round}"))
        };
        let attempt: SaveAttempt = save_into(root, into.as_path(), remaining.as_slice())?;
        copies.extend(attempt.copies.iter().cloned());
        if let Some(failure) = attempt.changed {
            // The source that changed has neither a copy nor a stamp; it is asked for again until the bound,
            //  and every path the round had not reached yet is asked for again too.
            if round >= SAVE_AGAIN_ROUNDS {
                return Err(changed_source_failure(&failure, directory.as_path()));
            }
            round += 1;
            remaining = unsaved_paths(remaining.as_slice(), attempt.sources.as_slice());
            continue;
        }
        sources = attempt.sources;
        remaining = Vec::<Vec<u8>>::new();
    }
    let staged: Vec<SavedCopy> = save_staged_content(git, common_dir, directory.as_path(), staged_paths)?;
    copies.extend(staged);
    if sources.is_empty() {
        return Ok(SavedBeforeCommand { copies, plan: None });
    }
    return Ok(SavedBeforeCommand { copies, plan: Some(SaveAgainPlan { root: root.to_path_buf(), directory, sources }) });
}

/// What:
///  The paths one round did not save, in their own order:
///  the source that changed under it and every path after that one.
/// Why:
///   The round stops reading when a source changes,
///  so the paths after it have no copy yet and must be asked for again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remaining = unsavedPaths({ paths, savedSources });
/// ```
fn unsaved_paths(paths: &[Vec<u8>], saved: &[SavedSource]) -> Vec<Vec<u8>> {
    let mut remaining: Vec<Vec<u8>> = Vec::<Vec<u8>>::new();
    for path in paths {
        let copied: bool = saved.iter().any(|source| { return source.path == *path && source.stamp.is_some(); });
        if !copied {
            remaining.push(path.clone());
        }
    }
    return remaining;
}

/// What:
///  Copy the content the index holds at each path below `<directory>/index-stage-0`.
/// Why:
///   An abort resets the index as well as the worktree,
///  so content that stands only there would be discarded,
///  and no worktree file holds it to copy.
///  Git's own `checkout-index` writes it,
///  from the index the abort is about to reset,
///  so the copy is exactly what the index held rather than a second reading of it,
///  and a path Git writes nothing for is a refusal that names the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = saveStagedContent({ git, commonDir, directory, paths });
/// ```
pub fn save_staged_content(git: &GitContext, common_dir: &Path, directory: &Path, paths: &[Vec<u8>]) -> Result<Vec<SavedCopy>, SaveFailure> {
    if paths.is_empty() {
        return Ok(Vec::<SavedCopy>::new());
    }
    let staged_root: PathBuf = directory.join(STAGED_FOLDER);
    if let Err(error) = create_private_directory_all(staged_root.as_path()) {
        return Err(SaveFailure::io(staged_root.display().to_string(), error.to_string()));
    }
    let mut input: Vec<u8> = Vec::<u8>::new();
    for path in paths {
        input.extend_from_slice(path.as_slice());
        input.push(0);
    }
    let mut arguments: Vec<OsString> = git.global_prefix.clone();
    for token in ["--no-optional-locks", "checkout-index", "--stdin", "-z"] {
        arguments.push(OsString::from(token));
    }
    let mut prefix: OsString = OsString::from("--prefix=");
    prefix.push(staged_root.as_os_str());
    prefix.push("/");
    arguments.push(prefix);
    let mut command: Command = git_command(git.real_git.as_path(), arguments.as_slice(), git.overlay.as_slice(), StartOwner::Wrapper);
    let handle: std::fs::File = match input_file(input.as_slice(), Some(common_dir)) {
        Ok(file) => file,
        Err(error) => return Err(SaveFailure::io(common_dir.display().to_string(), error.to_string())),
    };
    command.stdin(Stdio::from(handle));
    command.stdout(Stdio::null());
    command.stderr(Stdio::piped());
    let output: std::process::Output = match command.output() {
        Ok(finished) => finished,
        Err(error) => return Err(SaveFailure::io(git.real_git.display().to_string(), error.to_string())),
    };
    if !output.status.success() {
        let detail: String = String::from_utf8_lossy(output.stderr.as_slice()).lines().next().unwrap_or_default().trim().to_string();
        return Err(SaveFailure::io(String::from_utf8_lossy(paths[0].as_slice()).into_owned(), detail));
    }
    let mut copies: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    for path in paths {
        let copy: PathBuf = staged_root.join(OsString::from_vec(path.clone()));
        let placed: std::fs::File = match std::fs::File::open(copy.as_path()) {
            Ok(opened) => opened,
            Err(error) => {
                return Err(SaveFailure::io(
                    String::from_utf8_lossy(path.as_slice()).into_owned(),
                    format!("Git wrote no copy of the content the index holds ({error})"),
                ));
            }
        };
        // The copy is durable before the command that resets the index runs.
        if let Err(error) = placed.sync_all() {
            return Err(SaveFailure::io(copy.display().to_string(), error.to_string()));
        }
        copies.push(SavedCopy {
            path: format!("{} ({STAGED_FOLDER})", String::from_utf8_lossy(path.as_slice())),
            saved_to: Some(copy),
        });
    }
    if let Err(error) = sync_directory(staged_root.as_path()) {
        return Err(SaveFailure::io(staged_root.display().to_string(), error.to_string()));
    }
    return Ok(copies);
}

/// The saves, the staged content and the last comparison.
#[cfg(test)]
#[path = "abort_save_tests.rs"]
mod tests;
