//! What:
//!  The comparison the wrapper makes immediately before Git starts a command it saved for,
//!  and the copies it makes then.
//! Why:
//!  The save happens when the policy decides and Git discards later,
//!  so a file can be written in between and go with no copy.
//!  Each source is compared with what its copy proved it held,
//!  whatever changed is copied again beside the first copy,
//!  and a source that keeps changing through every round refuses the command instead of holding it:
//!  nothing may wait forever, and a refusal loses nothing.
//!  The window that remains is the instant between this comparison and Git's own reset,
//!  which no program outside Git can close, and `SPEC.md` and the notice say so in plain words.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! planTheLastChance(plan);
//! // immediately before Git starts:
//! const extra = saveAgainBeforeTheCommand(); // copied lines, or throws
//! ```

use crate::conflict_side_source::{SaveFailure, source_still_holds, still_nothing_there};
use crate::conflict_sides::{SaveAttempt, SavedCopy, SavedSource, save_into};
use std::path::{Path, PathBuf};
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

/// The folder the last comparison puts a source that changed since into.
const LAST_CHANCE_AGAIN_FOLDER: &str = "later";

/// What the last chance found:
///  nothing to copy,
///  or the copies it made and the sentences that name them.
type LastChance = Result<Option<String>, String>;

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
    let mut pending: std::sync::MutexGuard<'_, Option<SaveAgainPlan>> = PENDING.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); });
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
    let mut pending: std::sync::MutexGuard<'_, Option<SaveAgainPlan>> = PENDING.lock().unwrap_or_else(|poisoned| { return poisoned.into_inner(); });
    return pending.take();
}

/// The paths of `sources`, joined for a sentence
///  in the order the command listed them.
/// Why:
///   A refusal that named no path would leave the person guessing which file stopped the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function names(sources: SavedSource[]): string;
/// ```
fn names(sources: &[SavedSource]) -> String {
    let mut listed: Vec<String> = Vec::<String>::new();
    for source in sources {
        listed.push(String::from_utf8_lossy(source.path.as_slice()).into_owned());
    }
    return listed.join(", ");
}

/// What:
///  The sentence that names a second copy, which the person reads before Git runs.
/// Why:
///   A copy nobody is told about is a copy they will not find,
///  and this one was made after the first notice named the folder,
///  so it names the folder this copy is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function secondCopyNotice(changed: SavedSource[], copies: SavedCopy[], folder: string): string;
/// ```
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

/// What:
///  The sentences of the rounds that copied again, as one block,
///  or nothing when no round did.
/// Why:
///   The caller writes one answer,
///  and a command whose sources stood still owes the person no line at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function joinNotices(notices: string[]): string | undefined;
/// ```
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
        let Some(failure) = attempt.changed.first().cloned() else {
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
            // Nothing stood there when the copy was made,
            //  so a file that appeared since is work this command would discard.
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

/// The last comparison and its sentences.
#[cfg(test)]
#[path = "abort_save_again_tests.rs"]
mod tests;
