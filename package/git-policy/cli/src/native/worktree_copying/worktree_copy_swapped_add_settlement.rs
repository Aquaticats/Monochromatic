//! What:
//!  The settlement of an add whose snapshot was swapped into the destination and whose command was killed
//!  before the directory Git made at the stage name was settled.
//! Why:
//!  The exchange is the wrapper's own step,
//! and both objects it moves are recorded before it runs.
//!  The record beside the claim names the directory Git made for the destination,
//!  and the record of the stage's creation names the snapshot.
//!  After the exchange the object at the stage name is the first of those,
//! and the object at the
//!  destination is the second,
//! so those two identities prove the step completed and nothing else does.
//! Without this settlement the stage name keeps a folder forever.
//!  The proof that removes an unswapped stage reads the snapshot's own identity there and finds Git's
//!  directory instead,
//! and the pass that finishes the copy an interrupted add owes reads a destination whose identity the
//!  exchange itself changed as another command's work,
//! so every command says a copy is still owed that
//!  no command makes.
//! Nothing of the person's is removed:
//! the folder holds Git's pointer and an empty directory,
//! what was added to it meanwhile moves into the worktree that is there,
//! and a folder holding anything else is named and kept.
//! The snapshot carries the source's whole ignored state,
//! so an add settled here owes no copy and no
//!  command may say that one never finished.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settlement = settleSwappedAdd(claim, container);
//! ```

use crate::fs_beneath::{open_directory_at, open_root, remove_at};
use crate::json_record::quote;
use crate::plain_file::read_plain_file;
use crate::shell_quote::shell_quote;
use crate::worktree_add_binding::{Binding, Recorded, decode_record, path_identity};
use crate::worktree_convert_inspection::GIT_FILE_LIMIT;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_stage_claim::{DESTINATION_SUFFIX, StageClaim};
use crate::worktree_copy_stage_reaping_proof_and_words::still_there;
use crate::worktree_copy_stage_made_object::{MadeObject, read_made};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// The file Git writes to make a directory a worktree of its own.
const POINTER: &str = ".git";

/// What:
///  What one dead claim says about an exchange of its add's snapshot that completed.
/// Why:
///   Both objects the exchange moves are recorded before it runs,
///   so their two identities are the whole proof.
/// The destination's own state only chooses what to say and where a stray entry belongs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CompletedSwap = { binding: Binding; made: MadeObject; displacedStands: boolean };
/// ```
pub(crate) struct CompletedSwap {
    /// The binding the add recorded:
    /// the destination directory Git made and the pointer it wrote.
    binding: Binding,
    /// The record of the object the stage's creation made:
    /// the snapshot.
    made: MadeObject,
    /// Whether the directory Git made still stands at the stage name,
    /// rather than having been removed by a command killed before it settled the claim.
    displaced_stands: bool,
}

/// What:
///  What settling one such add left for the person,
/// or why its records stay.
/// Why:
///   A settled add has no records and one line saying what came of the stage.
/// An add whose folder holds something else keeps its claim and records and says why,
/// so nothing stands unnamed and nothing a person put in the folder is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SwappedSettlement = 'not-an-exchange' | { settled: string[] } | { kept: string[] };
/// ```
pub(crate) enum SwappedSettlement {
    /// The claim is not one whose exchange of its own snapshot a kill left half settled.
    NotAnExchange,
    /// The add's records are settled,
    /// with these lines for the person.
    Settled(Vec<String>),
    /// The folder at the stage name holds something the add did not put there,
    /// so the claim and its records stay,
    /// with these lines for the person.
    Kept(Vec<String>),
}

/// What:
///  Whether a claim names an add whose own snapshot was swapped into the destination.
/// Why:
///   The copy an interrupted add owes is not owed at all when its snapshot carried the source's whole
///   ignored state.
/// So the pass that reads such claims asks this before it says a copy is still owed,
/// and before it
///   refuses a destination whose identity the exchange itself changed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exchanged = completedSwapOf(claim, container) !== undefined;
/// ```
pub(crate) fn is_a_completed_exchange(claim: &Path, container: &Path) -> Result<bool, WorktreeCopyError> {
    return Ok(completed_swap_of(claim, container)?.is_some());
}

/// What:
///  Settles the exchange a kill interrupted,
/// before the reaping's own proof is asked,
/// and answers whether
///  the claim is done with.
/// Why:
///   The proof that removes an unswapped stage reads the snapshot's own identity at the stage name,
/// and the
///   directory Git made never has it,
/// so the settlement has to come first.
/// It is one empty folder and a few identity checks,
/// never a copy,
/// so every command may do it wherever it
///   runs.
/// The claim is done with once it is settled or once the folder is kept,
/// with the lines the person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done = await settleBeforeTheStageProof(claim, container, notices);
/// ```
pub(crate) fn settle_before_the_stage_proof(
    claim: &Path,
    container: &Path,
    notices: &mut Vec<String>,
) -> Result<bool, WorktreeCopyError> {
    return match settle_swapped_add(claim, container)? {
        SwappedSettlement::NotAnExchange => Ok(false),
        SwappedSettlement::Settled(lines) | SwappedSettlement::Kept(lines) => {
            notices.extend(lines);
            return Ok(true);
        }
    };
}

/// What:
///  Settle the add one dead claim names,
/// when the exchange of its own snapshot completed.
/// Why:
///   A command killed after the exchange leaves Git's directory at the stage name and the snapshot at the
///   destination.
/// Settling it costs a few identity checks and one empty folder,
/// never a copy,
/// so any command may do it
///   wherever it runs.
/// The claim goes only once nothing stands at the stage name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settlement = settleSwappedAdd(claim, container);
/// ```
pub(crate) fn settle_swapped_add(claim: &Path, container: &Path) -> Result<SwappedSettlement, WorktreeCopyError> {
    let Some(swapped) = completed_swap_of(claim, container)? else { return Ok(SwappedSettlement::NotAnExchange) };
    let mut lines: Vec<String> = match clear_the_displaced_directory(&swapped)? {
        Displacement::Stays(lines) => return Ok(SwappedSettlement::Kept(lines)),
        Displacement::Cleared(lines) => lines,
    };
    // The snapshot carried the ignored state, so this add owes no copy and writes no completion marker a
    //  later command has to read: the claim is released without a flush, as the creating command releases
    //  the one its own snapshot served.
    let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
    if let Some(settle) = StageClaim::adopted(claim, container, Some(record.as_path())) {
        settle.settle_unsynced()?;
    }
    lines.push(settled_words(&swapped));
    return Ok(SwappedSettlement::Settled(lines));
}

/// What:
///  The exchange one dead claim proves,
/// or nothing when its records prove none.
/// Why:
///   The record beside the claim names the destination Git made and the object it was when the add bound
///   itself,
/// and the record of the stage's creation names the snapshot.
/// The object at the stage name after the exchange is the one the first record names,
/// and a command killed
///   after the exchange may already have removed it,
/// leaving the snapshot at the destination as the proof
///   that it was there.
/// Any other pair of objects proves nothing,
/// so the conservative answers of the reaping stand unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const swap = completedSwapOf(claim, container); // undefined unless the records prove it
/// ```
fn completed_swap_of(claim: &Path, container: &Path) -> Result<Option<CompletedSwap>, WorktreeCopyError> {
    let record: PathBuf = claim.with_extension(&DESTINATION_SUFFIX[1..]);
    let Ok(text) = std::fs::read_to_string(record.as_path()) else { return Ok(None) };
    let Some(Recorded::Registered(binding)) = decode_record(text.as_str()) else { return Ok(None) };
    let Some(made) = read_made(claim) else { return Ok(None) };
    if made.container.as_path() != container {
        return Ok(None);
    }
    let stands: bool = still_there(container)?;
    let at_the_destination: Option<crate::fs_beneath::ObjectId> = path_identity(binding.destination.as_path());
    // The object at the stage name is the one the binding recorded for the destination, which the exchange
    //  put there, or the stage name is clear and the snapshot it was exchanged with stands at the
    //  destination.
    let displaced_stands: bool = stands && path_identity(container) == Some(binding.destination_id);
    let settled_before: bool = !stands && at_the_destination == Some(made.object);
    // A stage name a command already cleared, a snapshot that stands nowhere, and a directory at the
    //  destination whose identity is neither the snapshot nor the directory Git made: the add's worktree
    //  was removed and made again, so its records describe nothing that stands and no copy has a worktree
    //  to fill. A destination still holding the directory Git made is excluded: that add never swapped its
    //  snapshot and the reaping's own answers stay its own.
    let destination_moved_on: bool = !stands && at_the_destination.is_some() && at_the_destination != Some(binding.destination_id) && at_the_destination != Some(made.object);
    if !displaced_stands && !settled_before && !destination_moved_on {
        return Ok(None);
    }
    return Ok(Some(CompletedSwap { binding, made, displaced_stands }));
}

/// What:
///  Whether the directory Git made left the stage name clear.
/// Why:
///   One of two answers moves the claim,
/// and both carry the lines the person reads;
/// anything the folder holds that the add did not put there keeps it whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Displacement = { cleared: string[] } | { stays: string[] };
/// ```
enum Displacement {
    /// Nothing stands at the stage name any more,
    /// with what settling it said.
    Cleared(Vec<String>),
    /// The folder still stands,
    /// with the lines saying what keeps it.
    Stays(Vec<String>),
}

/// What:
///  Clear the stage name of the directory Git made,
/// or say why it stays.
/// Why:
///   The destination is the worktree the folder's entries belong to when the snapshot is what stands
///   there,
/// so they are settled exactly as the creating command settles them.
/// When the destination is gone or is a different worktree,
/// only the pointer Git wrote for this add is
///   provably the add's,
/// so the folder goes only when it holds nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const displacement = clearTheDisplacedDirectory(swap);
/// ```
fn clear_the_displaced_directory(swapped: &CompletedSwap) -> Result<Displacement, WorktreeCopyError> {
    if !swapped.displaced_stands {
        return Ok(Displacement::Cleared(Vec::new()));
    }
    let container: &Path = swapped.made.container.as_path();
    let destination: &Path = swapped.binding.destination.as_path();
    if path_identity(destination) == Some(swapped.made.object) {
        let notes: Vec<String> = crate::worktree_snapshot::settle_displaced(container, destination, swapped.binding.destination_id, swapped.binding.pointer.as_slice());
        let mut lines: Vec<String> = Vec::with_capacity(notes.len());
        for note in notes {
            lines.push(format!("monochromatic-git-policy-cli: {note}"));
        }
        if still_there(container)? {
            return Ok(Displacement::Stays(lines));
        }
        return Ok(Displacement::Cleared(lines));
    }
    return remove_the_directory_git_made(swapped);
}

/// What:
///  Remove the directory Git made from the stage name when it holds nothing but the pointer this add
///  recorded,
/// or say what keeps it.
/// Why:
///   The destination is gone or is another worktree,
/// so the folder's entries belong to nobody this command
///   can name.
/// Only the pointer bytes are proven to be Git's by the binding,
/// and anything else may be a person's,
/// so a
///   folder holding it is named and kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleared = removeTheDirectoryGitMade(swap);
/// ```
fn remove_the_directory_git_made(swapped: &CompletedSwap) -> Result<Displacement, WorktreeCopyError> {
    let container: &Path = swapped.made.container.as_path();
    let pointer: PathBuf = container.join(POINTER);
    let held = match std::fs::read_dir(container) {
        Ok(listing) => listing,
        // A folder that another command removed while this one looked is clear.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Displacement::Cleared(Vec::new())),
        Err(_unreadable) => return Ok(Displacement::Stays(vec![stays_words(container, "it cannot be listed; it holds the directory Git made for this add")])),
    };
    for entry in held.flatten() {
        if entry.file_name() != OsStr::new(POINTER) {
            let named: String = quote(entry.file_name().to_string_lossy().as_ref());
            return Ok(Displacement::Stays(vec![stays_words(container, format!("it holds {named}, which this add did not put there").as_str())]));
        }
    }
    if std::fs::symlink_metadata(pointer.as_path()).is_ok() {
        // The plain-file reader follows no link and reads no more than a pointer may hold, so a directory,
        //  a pipe or a link at the name refuses here rather than being read or waited for.
        let Ok(bytes) = read_plain_file(pointer.as_path(), GIT_FILE_LIMIT) else {
            return Ok(Displacement::Stays(vec![stays_words(container, "the pointer in it cannot be read as the plain file Git writes")]));
        };
        if bytes != swapped.binding.pointer {
            return Ok(Displacement::Stays(vec![stays_words(container, "the pointer in it is not the one Git wrote for this add")]));
        }
    }
    let (Some(parent_path), Some(stage_name)) = (container.parent(), container.file_name()) else {
        return Ok(Displacement::Stays(vec![stays_words(container, "it has no parent to be removed from")]));
    };
    let Ok(parent) = open_root(parent_path) else {
        return Ok(Displacement::Stays(vec![stays_words(container, "the directory holding it cannot be opened")]));
    };
    if std::fs::symlink_metadata(pointer.as_path()).is_ok() {
        let Ok(directory) = open_directory_at(&parent, stage_name) else {
            return Ok(Displacement::Stays(vec![stays_words(container, "it cannot be opened")]));
        };
        if let Err(error) = remove_at(&directory, OsStr::new(POINTER), false) {
            return Ok(Displacement::Stays(vec![stays_words(container, format!("the pointer Git wrote cannot be removed ({error})").as_str())]));
        }
    }
    // `rmdir` removes nothing that holds anything, so an entry another process put there after the listing
    //  keeps the folder and is named by the next command.
    if let Err(error) = remove_at(&parent, stage_name, true) {
        return Ok(Displacement::Stays(vec![stays_words(container, format!("it is not empty ({error})").as_str())]));
    }
    return Ok(Displacement::Cleared(Vec::new()));
}

/// What:
///  The words for one add whose snapshot was swapped into place and whose records are settled.
/// Why:
///   The person reads what became of the stage,
/// and,
/// where the worktree is the one the add made,
/// that its
///   ignored state is whole,
/// so no later line promises a copy that this add never owed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const words = `the \`git worktree add\` of ${destination} was interrupted after its snapshot was swapped into place; ...`;
/// ```
fn settled_words(swapped: &CompletedSwap) -> String {
    let destination: String = quote(swapped.binding.destination.to_string_lossy().as_ref());
    if path_identity(swapped.binding.destination.as_path()) == Some(swapped.made.object) {
        return format!(
            "monochromatic-git-policy-cli: the `git worktree add` of {destination} was interrupted after its snapshot was swapped into place; the snapshot carried the source's ignored state, so nothing in that worktree is missing, and the add's records are settled."
        );
    }
    return format!(
        "monochromatic-git-policy-cli: the `git worktree add` of {destination} was interrupted after its snapshot was swapped into place, and the worktree at that path is no longer the one the add made; the directory Git made for it is gone and the add's records are settled."
    );
}

/// What:
///  The words for a folder at the stage name that holds something this add did not put there,
/// why it was
///  not changed,
/// and how to remove it.
/// Why:
///   A folder that stays must be named with what keeps it and how the person clears it,
/// so nothing waits on
///   the wrapper's next command to say the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the stage ${shown} holds the directory Git made for an interrupted \`git worktree add\` and was not changed: ${reason}; remove it yourself once nothing needs it (\`rm -r ${quoted}\`).`
/// ```
fn stays_words(container: &Path, reason: &str) -> String {
    let quoted: String = shell_quote(container.as_os_str().as_bytes());
    return format!(
        "monochromatic-git-policy-cli: the stage {} holds the directory Git made for an interrupted `git worktree add` and was not changed: {reason}; remove it yourself once nothing needs it (`rm -r {quoted}`).",
        quote(container.to_string_lossy().as_ref())
    );
}

/// Settlement of an exchanged add's records stays out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_swapped_add_settlement_tests.rs"]
mod tests;
