//! What:
//!  Whether the record of an object a stage's creation made proves the folder standing at the container's name,
//! and the words a sweep says about it.
//! Why:
//!  A stage an add left can be a whole snapshot of the checkout,
//! so the removal rests on two proofs:
//! the record
//!  names the object still at the name,
//! and nothing in the folder was born after the instant that record holds,
//! except the entries the
//!  wrapper itself writes there later.
//! Everything else keeps the folder,
//! and the reason is the words the person reads,
//! so this
//!  module is the one place both the answer and its words are decided.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proved = await proofOf({ claim, container }); // 'proved' | 'gone' | { unproved: string }
//! writeln(proved === 'proved' ? removedStageWords(container) : keptStageWords(container, proved.unproved));
//! ```

use crate::json_record::quote;
use crate::worktree_copy_error::{WorktreeCopyError, raw_failure};
use crate::worktree_copy_stage_made_object::{is_the_recorded_object, made_record_beside_claim, read_made};
use std::path::Path;

/// What:
///  Whether the container can be examined at all,
/// and nothing when it is already gone.
/// Why:
///   Another command may remove the same stage at the same moment,
/// and a folder that is gone is nobody's failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const present = exists(container); // absence is an answer, every other refusal is a fault
/// ```
pub(crate) fn still_there(container: &Path) -> Result<bool, WorktreeCopyError> {
    // `match` on the lookup: absence is the answer false, every other failure is reported.
    match std::fs::symlink_metadata(container) {
        Ok(_) => return Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(raw_failure(&error)),
    }
}

/// What:
///  What the record of the object a stage's creation made says about what stands at the container's name.
/// Why:
///   Three answers are needed:
/// the record proves the object and the folder holds only what the record accounts for,
/// it proves nothing
///  and the folder is kept with the reason the person reads,
/// or the folder is already gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Provenance = 'proved' | 'gone' | { unproved: string };
/// ```
pub(crate) enum Provenance {
    /// The record proves this wrapper made the object there,
    /// and the folder holds nothing it does not account for.
    Proved,
    /// Nothing stands at the name any more.
    Gone,
    /// Nothing proves the folder is one the wrapper made and still holds what it made,
    /// in words the person reads.
    Unproved(String),
}

/// What:
///  The first entry of a stage the record cannot account for,
/// where the accounting is built.
/// Why:
///   Only the Linux build writes a record of a stage's object,
/// so only there can a folder be judged;
/// a build without it
///  keeps every such folder and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unaccounted = firstUnaccountedEntry(container, written);
/// ```
#[cfg(target_os = "linux")]
fn unaccounted_entry(container: &Path, written: std::time::SystemTime) -> Option<String> {
    return crate::worktree_copy_stage_entries_the_record_accounts_for::unaccounted_entry(container, written);
}

/// `unaccounted_entry` where the accounting is not built.
#[cfg(not(target_os = "linux"))]
fn unaccounted_entry(_container: &Path, _written: std::time::SystemTime) -> Option<String> {
    return Some(String::from("this build cannot examine what the folder holds, so nothing in it was removed"));
}

/// What:
///  Whether the record beside a claim proves the object at the container's name is the one the wrapper made,
/// and that
///  the folder holds only what that record accounts for.
/// Why:
///   This is the whole proof the reaping of an add's stage rests on:
/// a record this release writes,
/// naming this container,
/// whose
///  identity is still the one at the name,
/// and a folder holding no entry born after the recorded instant except the ones
///  the wrapper itself makes there.
/// Anything less keeps the folder,
/// and the reason is the words the person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function provenanceOf(claim: string, container: string): Provenance;
/// ```
pub(crate) fn provenance_of(claim: &Path, container: &Path) -> Result<Provenance, WorktreeCopyError> {
    if !still_there(container)? {
        return Ok(Provenance::Gone);
    }
    let Some(made) = read_made(claim) else {
        let stands: bool = std::fs::symlink_metadata(made_record_beside_claim(claim).as_path()).is_ok();
        // A file that stands there and does not read is damage, which is said differently from an absent record, so the
        // person knows whether to look at a file of their own or at nothing.
        return Ok(Provenance::Unproved(match stands {
            true => String::from("the record of the object its creation made does not read as this release writes it"),
            false => String::from("no record of the object its creation made stands beside its claim"),
        }));
    };
    if made.container.as_path() != container {
        return Ok(Provenance::Unproved(format!(
            "the record of the object its creation made names {} and not the folder the claim names",
            quote(made.container.to_string_lossy().as_ref())
        )));
    }
    if !is_the_recorded_object(container, &made) {
        return Ok(Provenance::Unproved(String::from("the object there now is not the one the record of its creation made")));
    }
    return match unaccounted_entry(container, made.written) {
        None => Ok(Provenance::Proved),
        Some(reason) => Ok(Provenance::Unproved(reason)),
    };
}

/// What:
///  The words for a stage an add left that the reaping keeps,
/// with the reason it was kept.
/// Why:
///   The person reads which folder stands,
/// why nothing in it was changed,
/// and how to remove it,
/// so a kept folder
///  never waits for something the wrapper will not do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the stage ${path} was left by an interrupted \`git worktree add\` and was not changed: ${reason}; remove it yourself once nothing needs it (...)`
/// ```
pub(crate) fn kept_stage_words(container: &Path, reason: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the stage {} was left by an interrupted `git worktree add` and was not changed: {reason}; remove it yourself once nothing needs it (`btrfs subvolume delete` for a snapshot, `rm -r` otherwise).",
        quote(container.to_string_lossy().as_ref())
    );
}

/// What:
///  The words for a stage an add left that the record proved and the reaping removed.
/// Why:
///   The person reads what went,
/// so a folder that held a copy of their ignored state does not disappear without a word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `removed the stage ${path} that an interrupted \`git worktree add\` left behind.`
/// ```
pub(crate) fn removed_stage_words(container: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: removed the stage {} that an interrupted `git worktree add` left behind.",
        quote(container.to_string_lossy().as_ref())
    );
}

/// What:
///  The words for a stage container a copy left that the reaping removed.
/// Why:
///   The same line the incumbent says,
/// so a copy's stage and an add's stage are told apart by what made them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `removed the stage ${path} that an interrupted ignored-state copy left behind.`
/// ```
pub(crate) fn removed_copy_stage_words(container: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: removed the stage {} that an interrupted ignored-state copy left behind.",
        quote(container.to_string_lossy().as_ref())
    );
}
