//! What:
//!  The look every command makes for a subvolume a removal renamed aside whose leaving record nothing kept:
//!  a directory of this program's leaving shape standing in a directory a removal of this repository renames into,
//!  proved by the mark this program sets on a subvolume before it moves it, or by a record of its delete,
//!  and never by its name.
//! Why:
//!  A removal renames its subvolume to `cli-git-deleting-<id>`
//!  and writes a record that outlives every queue record for the same place
//!  (`worktree_removal_leaving_that_outlives_its_record.rs`).
//!  That record is the whole of what names the directory once the queue record is settled,
//!  and a release before it wrote no such thing:
//!  the renames of 2026-11-10 onwards
//!  (`5214f4e553`) queued a delete whose first round looked before the rename and settled it as done,
//!  so a removal that was then killed,
//!  or whose own destroy the kernel refused,
//!  left the person's files under a name nothing read any more.
//!
//!  What a name proves:
//!  only that someone wrote sixteen hexadecimal digits equal to the subvolume's own kernel id.
//!  No accident reaches that:
//!  a directory entry is named before the subvolume under it has an id,
//!  and nothing guesses a 64-bit id,
//!  so by accident the shape and the id never meet.
//!  By intent they do,
//!  and a person can read a subvolume's id
//!  (`btrfs subvolume show`) and name their own directory after it,
//!  so a name is no proof of who renamed a directory,
//!  and this look never deletes on one.
//!
//!  What it requires instead is evidence this program left behind:
//!  - the mark it sets on a subvolume before it renames it,
//!    an extended attribute naming that subvolume's own id
//!    (`mark_the_leaving`).
//!    It travels with the subvolume through the rename,
//!    so it is carried by the directory itself and survives the loss of every record of the repository.
//!    A copy of the attribute onto another subvolume names a different id,
//!    so it never carries over.
//!  - or a record of that very delete, in any state the queue keeps:
//!    one a round still owes, or one settled `done` or `failed`,
//!    which a release that wrote no leaving record wrote before it renamed the same place
//!    (`a_settled_record_names`).
//!    Those records are why a leftover of an earlier release is still proved:
//!    the queue never forgot it,
//!    only the leaving did.
//!
//!  Where it looks:
//!  the directory each linked worktree of this repository stands in,
//!  and each directory a removal of it has written down before a rename,
//!  which is what keeps a directory readable once its last registration goes;
//!  nothing else.
//!  A removal renames its subvolume inside the directory that holds the worktree it removes,
//!  in one call on the parent it already holds open
//!  (`worktree_remove_hand_over.rs`, `hand_over_with`).
//!  The main checkout is no such directory:
//!  Git removes no main worktree,
//!  and a linked worktree placed beside the main checkout is registered itself,
//!  so its own directory is looked at through its own registration.
//!
//!  How it opens what it finds:
//!  a directory entry the look did not create is opened once,
//!  as a directory,
//!  without waiting for anything,
//!  below the held directory the entry was listed from,
//!  and every question after that is asked of that open descriptor:
//!  its inode says it is a subvolume's root,
//!  its own id says it is the subvolume the name holds,
//!  and the mark is read through the same descriptor.
//!  Nothing here asks the path a second time,
//!  so an entry replaced by a link, a device or a named pipe between two questions cannot make this command wait
//!  or act on what it did not check.
//!
//!  What it never does:
//!  it deletes nothing.
//!  A proved leftover is written into a leaving record of its own and queued for a round,
//!  which is where every handed-over delete goes
//!  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`),
//!  and a directory it cannot prove is named once with the command that removes it and left exactly as it is.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = queueTheLeavingsNoRecordNames({ commonDir, work, named });
//! if (found.queued) await wakeTheService(context, commonDir);
//! for (const line of found.said) stderr.write(line + '\n');
//! ```

use crate::diagnostic_log::debug;
use crate::fs_beneath::{handle_path, identity_of, open_directory_at_without_waiting, open_root};
use crate::fs_btrfs_ioctl::subvolume_id;
use crate::fs_kind::BTRFS_SUBVOLUME_ROOT_INODE;
use crate::plain_file::read_plain_file;
use crate::work_item::{DONE, FAILED, WorkItem, WorkKind, queued_ids, state_directory};
use crate::worktree_remove_hand_over::{DELETE_REFERENCE, removal_item};
use crate::worktree_removal_leaving_that_outlives_its_record::{RECORD_LIMIT, forget_a_place, record_a_leaving_found_by_a_scan, the_ids_of, the_mark_of};
use crate::worktree_removal_leavings_a_command_queues_again::{Leavings, wake_a_pending_delete};
use crate::worktree_removal_leavings_already_reported::{forget_the_report, report_once};
use crate::worktree_removal_takes_its_pending_deletions::is_a_leaving_name;
use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeavingsNoRecord";

/// The words every line of this module starts with,
/// so a person reading a terminal knows which program said it.
const SAID: &str = "monochromatic-git-policy-cli:";

/// The directory that holds every other directory,
/// whose parent is itself and names no place a removal renames into.
const THE_ROOT: &str = "/";

/// The number of hexadecimal digits the subvolume id in a leaving name takes,
/// which `is_a_leaving_name` has already checked.
const IDENTIFIER_DIGITS: usize = 16;

/// What:
///  Every directory a removal of this repository renames a subvolume into:
///  the directory each of its linked worktrees stands in.
/// Why:
///   A removal renames its subvolume inside the directory that holds the worktree,
///  in one call on the parent it already holds open,
///  so a leftover of one stands beside the worktrees of the same directory.
///  The registrations of the repository are what names those directories.
///  The main checkout is not one of them:
///  no removal of this program renames anything beside it,
///  because Git removes no main worktree,
///  and a linked worktree beside it is registered itself and names its own directory.
///  A registration is exactly what a removal takes away,
///  so the registrations alone lose a directory whose last worktree is removed,
///  which is the ordinary case:
///  each removal writes down the directory it renames in before it renames
///  (`record_the_place`),
///  and those records are read beside the registrations that stand now.
///  The checkout itself is never a place a removal renames into:
///  the files of a worktree are a person's.
///  A directory that cannot be listed leaves the list empty,
///  which costs the repair and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = theDirectoriesARemovalRenamesInside(commonDir); // distinct, never the root, never a checkout
/// ```
fn the_directories_a_removal_renames_inside(common_dir: &Path) -> Vec<PathBuf> {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let Ok(registered) = crate::worktree_remove_registry::registered_paths(common_dir, worktrees.as_path()) else {
        debug(TAG, format!("the worktrees of {} cannot be listed, so no directory a removal renames into is looked at", common_dir.display()).as_str());
        return Vec::new();
    };
    let mut places: Vec<PathBuf> = Vec::new();
    for listed in registered {
        // The main checkout is no linked worktree, so no removal of this program renames inside its directory.
        if listed.id.is_none() {
            continue;
        }
        let checkout: PathBuf = PathBuf::from(OsStr::from_bytes(listed.path.as_slice()));
        let Some(parent) = checkout.parent() else { continue };
        if parent.as_os_str().is_empty() || parent == Path::new(THE_ROOT) {
            continue;
        }
        if places.iter().any(|seen| { return seen == parent }) {
            continue;
        }
        places.push(parent.to_path_buf());
    }
    for recorded in crate::worktree_removal_leaving_that_outlives_its_record::the_recorded_places(common_dir) {
        if recorded == Path::new(THE_ROOT) || places.iter().any(|seen| { return seen == &recorded }) {
            continue;
        }
        places.push(recorded);
    }
    return places;
}

/// What:
///  The id in a leaving name,
///  or nothing when the name carries none.
/// Why:
///   `is_a_leaving_name` proves the shape before this reads it,
///  and sixteen hexadecimal digits are one `u64` whatever they say,
///  so this answers nothing only for a name the caller never proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const id = idInTheName(path); // BigInt('0x' + last 16 hex digits), or undefined
/// ```
fn id_in_the_name(path: &Path) -> Option<u64> {
    let name: &str = path.file_name()?.to_str()?;
    let identifier: &str = name.rsplit_once('-')?.1;
    if identifier.len() != IDENTIFIER_DIGITS {
        return None;
    }
    return u64::from_str_radix(identifier, 16).ok();
}

/// What:
///  The subvolume id of one open entry the name says is a subvolume's root,
///  or the sentence for the person saying why the entry is not the subvolume the name holds.
/// Why:
///   The inode of a subvolume's root is the kernel's own, and the id comes from the open descriptor,
///  so the two answers are about the directory this command opened and not about whatever the path holds later.
///  A directory that is no subvolume's root is not a directory this program renamed aside,
///  and one whose id the name does not hold is another program's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proved = theSubvolumeOfTheOpenEntry({ held, namedId, path }); // throws the reason it is not this program's
/// ```
fn the_subvolume_of_the_open_entry(held: &File, named_id: u64, path: &Path) -> Result<u64, String> {
    let identity = match identity_of(held) {
        Ok(found) => found,
        Err(error) => return Err(format!("its identity cannot be read ({error})")),
    };
    if identity.inode != BTRFS_SUBVOLUME_ROOT_INODE {
        return Err(String::from("what stands there is not the root of a btrfs subvolume"));
    }
    return match subvolume_id(held, path) {
        Ok(id) if id == named_id => Ok(id),
        Ok(id) => Err(format!("the subvolume standing there is {id} and the name says {named_id}, so another program wrote that name")),
        Err(error) => Err(format!("the subvolume standing there cannot be identified ({error})")),
    };
}

/// What:
///  Whether a queue record names the delete of exactly this place and this subvolume,
///  in the states a round still owes or a round settled.
/// Why:
///   This is the proof a release that wrote no leaving record left behind:
///  the delete of the renamed path was queued before the rename,
///  and the first round that looked too early settled it `done`,
///  which names the place and the subvolume to this day.
///  The record is read as the queue reads it,
///  and its own id must be the one this place and subvolume derive,
///  so a record of another place or another subvolume proves nothing here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = aSettledRecordNames(work, item); // done/<id> or failed/<id> holds this item
/// ```
fn a_settled_record_names(work: &Path, item: &WorkItem) -> bool {
    // Both names a release may have written the record under: a delete an earlier release settled stands under the older
    // one, and it is the same delete of the same place and subvolume.
    for state in [DONE, FAILED] {
        for name in the_ids_of(item) {
            let record: PathBuf = state_directory(work, state).join(name.as_str());
            let Ok(bytes) = read_plain_file(record.as_path(), RECORD_LIMIT) else { continue };
            let Ok(text) = std::str::from_utf8(bytes.as_slice()) else { continue };
            let Ok(settled) = WorkItem::decode(text) else { continue };
            if settled.kind != WorkKind::SubvolumeDelete || settled.branch_ref != DELETE_REFERENCE {
                continue;
            }
            if settled.oid != item.oid || settled.parent != item.parent {
                continue;
            }
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a round of the service owes this delete,
///  under either name a queue record of it may stand.
/// Why:
///  The queue outlives the release that wrote to it,
///  so a pending delete an earlier release queued stands under the older name,
///  and a look that asked the new name alone would neither know it owed nor wake it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owes = aRoundOwes(queuedIds, item); // either id is among the ids a round owes
/// ```
fn a_round_owes(owes: &HashSet<String>, item: &WorkItem) -> bool {
    return the_ids_of(item).iter().any(|id| { return owes.contains(id.as_str()) });
}

/// What:
///  Queue the delete of a proved leftover,
///  or ask for the round that already owes it,
///  and name the places this command may not act on,
///  once each.
/// Why:
///   A proved leftover is this program's own,
///  so it is recorded and queued exactly as a handed-over one is,
///  and the record gives it the same bound:
///  the attempts a delete may cost,
///  and then the line that names the directory and the command that removes it
///  (`worktree_removal_leavings_a_command_queues_again.rs`).
///   A delete a round already owes is what this command would queue,
///  so nothing is written for it and the round is told instead,
///  which is the wake a killed command never made.
///   A place that is not proved is said once and untouched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { queued: false, said: [] };
/// for (const place of places) for (const found of readdir(place)) queueOneFoundName({ commonDir, work, place, found, answer });
/// return answer;
/// ```
pub(crate) fn queue_the_leavings_no_record_names(common_dir: &Path, work: &Path, named: &HashSet<PathBuf>) -> Leavings {
    let mut answer: Leavings = Leavings { queued: false, asked: None, said: Vec::new() };
    // One listing of each state a round owes, asked only when a place turns out to need it.
    let mut owed: Option<HashSet<String>> = None;
    for place in the_directories_a_removal_renames_inside(common_dir) {
        let held_place: File = match open_root(place.as_path()) {
            Ok(opened) => opened,
            // A directory that is gone holds nothing of this program's, so what a removal wrote down about it is finished.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                forget_a_place(common_dir, place.as_path());
                continue;
            }
            Err(error) => {
                debug(TAG, format!("{} cannot be opened, so no leftover of a removal is looked for in it ({error})", place.display()).as_str());
                continue;
            }
        };
        // The listing is of the held descriptor's own name, so it is that directory and never a path that could be replaced.
        let listing = match std::fs::read_dir(handle_path(&held_place)) {
            Ok(listing) => listing,
            Err(error) => {
                debug(TAG, format!("{} cannot be listed, so no leftover of a removal is looked for in it ({error})", place.display()).as_str());
                continue;
            }
        };
        let mut saw_the_shape: bool = false;
        for found in listing.flatten() {
            let name: OsString = found.file_name();
            if is_a_leaving_name(place.join(name.as_os_str()).as_path()) {
                saw_the_shape = true;
            }
            queue_one_found_name(common_dir, work, place.as_path(), &held_place, named, &mut owed, name, &mut answer);
        }
        // What: The record of a directory goes with the last thing of this program's left to look for in it.
        // Why:  A record that outlived its need would be read by every command for ever, and a leaving record naming a path
        //       inside it is that path still to look at.
        if !saw_the_shape && !a_leaving_record_names_a_path_inside(named, place.as_path()) {
            forget_a_place(common_dir, place.as_path());
        }
    }
    return answer;
}

/// What:
///  Whether a leaving record names a path inside this directory,
///  which is a leftover still to look at even when nothing of the shape stands there now.
/// Why:
///  A record whose directory the kernel has yet to take keeps its place worth reading,
///  and dropping the place under it would cost the next command that look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = aLeavingRecordNamesAPathInside(named, place);
/// ```
fn a_leaving_record_names_a_path_inside(named: &HashSet<PathBuf>, place: &Path) -> bool {
    return named.iter().any(|path| { return path.parent() == Some(place) });
}

/// What:
///  Decide one directory entry of a place a removal renames into.
/// Why:
///   The answers need different steps:
///  a proved leftover is recorded and queued,
///  and everything else is said once and left exactly as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// queueOneFoundName({ commonDir, work, place, heldPlace, named, owed, name, answer });
/// ```
fn queue_one_found_name(
    common_dir: &Path,
    work: &Path,
    place: &Path,
    held_place: &File,
    named: &HashSet<PathBuf>,
    owed: &mut Option<HashSet<String>>,
    name: OsString,
    answer: &mut Leavings,
) {
    let path: PathBuf = place.join(name.as_os_str());
    // A name no removal of this program gives anything is the person's own, and a place a leaving record already names is
    // that record's to decide, which this command has just done or left alone.
    if !is_a_leaving_name(path.as_path()) || named.contains(&path) {
        return;
    }
    let Some(named_id) = id_in_the_name(path.as_path()) else { return };
    // One open, below the held directory, as a directory, and without waiting: an entry a concurrent command replaced with a
    // link, a device or a named pipe is refused here instead of being followed.
    let held: File = match open_directory_at_without_waiting(held_place, name.as_os_str()) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            say_unproved(common_dir, path.as_path(), format!("it cannot be opened as a directory ({error})"), answer);
            return;
        }
    };
    // What: The refusal that holds for every proof, before any of them can queue a delete.
    // Why:  A record names its paths in text, and a delete queued for a place that is not text names a directory with the
    //       replacement character in it, which the service finds empty and settles as done while the real subvolume stands.
    //       The proof does not matter: a marked subvolume under such a parent is the same lossy record.
    if place.to_str().is_none() {
        say_unproved(
            common_dir,
            path.as_path(),
            String::from("the directory that holds it is not named by text, so no record of its delete can name the path a delete would act on"),
            answer,
        );
        return;
    }
    let id: u64 = match the_subvolume_of_the_open_entry(&held, named_id, path.as_path()) {
        Ok(proved) => proved,
        Err(why) => {
            say_unproved(common_dir, path.as_path(), why, answer);
            return;
        }
    };
    if the_mark_of(&held) == Some(id) {
        debug(TAG, format!("{} carries the mark this program sets on a subvolume it renames aside", path.display()).as_str());
        queue_the_leaving_found(common_dir, work, place, path.as_path(), id, owed, answer);
        return;
    }
    let item: WorkItem = removal_item(common_dir, place, id);
    // Both answers end in one step: a round that owes the delete is told, and a delete only a settled record names is
    // queued again (`queue_the_leaving_found`).
    let proved: bool = {
        let owes: &HashSet<String> = owed.get_or_insert_with(|| { return queued_ids(work); });
        a_round_owes(owes, &item) || a_settled_record_names(work, &item)
    };
    if proved {
        queue_the_leaving_found(common_dir, work, place, path.as_path(), id, owed, answer);
        return;
    }
    say_unproved(
        common_dir,
        path.as_path(),
        String::from("neither the mark this program sets on a subvolume before it moves it nor a record of its delete names it, so a person or another program wrote that name"),
        answer,
    );
}

/// What:
///  Write the leaving a found directory needs and queue its delete,
///  or ask for the round that already owes it,
///  and say so when the record or the queue cannot be written.
/// Why:
///   The record is what names the directory once every queue record is settled,
///  and it is what bounds a delete the kernel keeps refusing,
///  so both outlive the command a person ran exactly as a hand-over's do.
///   A delete a round already owes is the work this command would ask for,
///  so nothing is queued again and the round is told instead,
///  within the attempts a record may cost, which is what stops every command from asking for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leaving = await recordALeavingFoundByAScan(commonDir, removalItem(commonDir, place, id));
/// if (owed.has(leaving.item.id())) return wakeAPendingDelete(commonDir, leaving, path);
/// await queueTheDeleteAgain(commonDir, leaving);
/// ```
fn queue_the_leaving_found(common_dir: &Path, work: &Path, place: &Path, path: &Path, id: u64, owed: &mut Option<HashSet<String>>, answer: &mut Leavings) {
    let item: WorkItem = removal_item(common_dir, place, id);
    let leaving = match record_a_leaving_found_by_a_scan(common_dir, &item) {
        Ok(found) => found,
        Err(reason) => {
            say_not_queued(common_dir, path, id, format!("the record of its delete cannot be written ({reason})"), answer);
            return;
        }
    };
    let owes: &HashSet<String> = owed.get_or_insert_with(|| { return queued_ids(work); });
    if a_round_owes(owes, &item) {
        wake_a_pending_delete(common_dir, &leaving, path, answer);
        return;
    }
    match crate::worktree_removal_leaving_that_outlives_its_record::queue_the_delete_again(common_dir, &leaving) {
        Ok(()) => {
            debug(TAG, format!("the subvolume {} no leaving record named is recorded and queued for a round's delete", path.display()).as_str());
            forget_the_report(common_dir, path);
            answer.queued = true;
        }
        Err(reason) => say_not_queued(common_dir, path, id, format!("the delete that would remove it cannot be queued ({reason})"), answer),
    }
}

/// What:
///  Say that one place is this program's own leftover and its delete could not be queued,
///  once,
///  with the command that removes it.
/// Why:
///   A proof that something is this program's own still leaves the person with the space until a round can take the
///  subvolume,
///  so the line names the directory,
///  what stood in the way of the queueing,
///  and the command that removes it by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportOnce(commonDir, path, `${SAID} ... is the subvolume ${id} this program renamed aside ... `btrfs subvolume delete ${path}` ...`, answer.said);
/// ```
fn say_not_queued(common_dir: &Path, path: &Path, id: u64, why: String, answer: &mut Leavings) {
    let line: String = format!(
        "{SAID} the subvolume {id} this program renamed aside stands at {named}, and {why}, so no round will delete it; `btrfs subvolume delete {named}` removes it by hand and frees the space it holds.",
        named = path.display()
    );
    report_once(common_dir, path, line, &mut answer.said);
}

/// What:
///  Say why one place is not proved this program's own,
///  once,
///  with the command that removes a subvolume of the person's there.
/// Why:
///   The line must let the person act without this program guessing:
///  it names the directory and what stood in the way of a proof,
///  and it is said once so a commit is not preceded by the same sentence for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reportOnce(commonDir, path, `${SAID} ... ${path} ... `btrfs subvolume delete ${path}` ...`, answer.said);
/// ```
fn say_unproved(common_dir: &Path, path: &Path, why: String, answer: &mut Leavings) {
    let line: String = format!(
        "{SAID} a directory named as this program names the subvolume a removed worktree is renamed to stands at {named}, and nothing proves it is a leftover of this program's: {why}. It was left exactly as it is; if it is a subvolume of yours, `btrfs subvolume delete {named}` removes it and frees the space it holds.",
        named = path.display()
    );
    report_once(common_dir, path, line, &mut answer.said);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_leavings_no_record_names_tests.rs"]
mod tests;
