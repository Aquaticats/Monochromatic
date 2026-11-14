//! What:
//!  The look every command makes for a subvolume a removal renamed aside whose record nothing kept:
//!  a directory of this program's own leaving shape standing in a directory a removal renames into,
//!  queued for its delete only while its name and the kernel agree on which subvolume it is.
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
//!  left the person's files under a name no record named at all.
//!  A directory of that shape standing where a removal renames is therefore looked at,
//!  and it is proved this program's by two things nothing else answers:
//!  it is the root of a btrfs subvolume,
//!  and the id in its name is that subvolume's own id,
//!  which only this program writes (`deleting_name`).
//!  A name alone proves nothing, so an entry that is no subvolume,
//!  no directory,
//!  or a subvolume another id names is the person's and is never queued for a delete;
//!  it is named once with the command that removes it,
//!  and never touched.
//!  Nothing here deletes anything either:
//!  a proved leftover is written into a leaving record of its own and queued for a round,
//!  which is where every handed-over delete goes
//!  (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const found = queueTheLeavingsNoRecordNames({ commonDir, work, named });
//! if (found.queued) await wakeTheService(context, commonDir);
//! for (const line of found.said) stderr.write(line + '\n');
//! ```

use crate::diagnostic_log::debug;
use crate::fs_btrfs_ioctl::subvolume_id;
use crate::fs_kind::is_subvolume_root;
use crate::worktree_remove_hand_over::removal_item;
use crate::worktree_removal_leaving_that_outlives_its_record::record_a_leaving_found_by_a_scan;
use crate::worktree_removal_leavings_a_command_queues_again::Leavings;
use crate::worktree_removal_leavings_already_reported::{forget_the_report, report_once};
use crate::worktree_removal_takes_its_pending_deletions::is_a_leaving_name;
use std::collections::HashSet;
use std::ffi::OsStr;
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
///  Every directory this repository's removals rename a subvolume into:
///  the directory each of its worktrees stands in.
/// Why:
///   A removal renames its subvolume inside the directory that holds the worktree,
///  in one call on the parent it already holds open
///  (`worktree_remove_hand_over.rs`, `hand_over_with`),
///  so a leftover of one stands beside the worktrees of the same directory.
///  The repository's own registrations are what names those directories,
///  and a registration a removal of this program left behind names one too,
///  so a removal killed before its registration went is still looked at.
///  The checkout itself is dropped:
///  the files of a worktree are a person's and never a place this program renames into.
///  A directory that cannot be listed leaves the list empty,
///  which costs the repair and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = theDirectoriesRemovalsRenameInside(commonDir); // distinct, never the root, never the checkout
/// ```
fn the_directories_removals_rename_inside(common_dir: &Path) -> Vec<PathBuf> {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let Ok(registered) = crate::worktree_remove_registry::registered_paths(common_dir, worktrees.as_path()) else {
        debug(TAG, format!("the worktrees of {} cannot be listed, so no directory a removal renames into is looked at", common_dir.display()).as_str());
        return Vec::new();
    };
    let mut places: Vec<PathBuf> = Vec::new();
    for listed in registered {
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
///  Whether what stands at `path` is the subvolume the id in its name says,
///  or the sentence for the person saying why it is not proved.
/// Why:
///   The name is what this program wrote and the id is what the kernel made,
///  and comparing them is the one proof that costs no trust in either alone.
///  A directory that is no subvolume,
///  or a subvolume another id names,
///  is the person's or another program's and is never queued for a delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proved = theSubvolumeTheNameShows(path, namedId, fileType); // throws the reason it is not this program's
/// ```
fn the_subvolume_the_name_shows(path: &Path, named_id: u64, kind: std::fs::FileType) -> Result<u64, String> {
    if !kind.is_dir() {
        return Err(String::from("what stands there is not a directory"));
    }
    match is_subvolume_root(path) {
        Ok(true) => {}
        Ok(false) => return Err(String::from("what stands there is not the root of a btrfs subvolume")),
        Err(error) => return Err(format!("what stands there cannot be examined ({error})")),
    }
    let held: File = match File::open(path) {
        Ok(opened) => opened,
        Err(error) => return Err(format!("it cannot be opened ({error})")),
    };
    return match subvolume_id(&held, path) {
        Ok(id) if id == named_id => Ok(id),
        Ok(id) => Err(format!("the subvolume standing there is {id} and the name says {named_id}, so another program wrote that name")),
        Err(error) => Err(format!("the subvolume standing there cannot be identified ({error})")),
    };
}

/// What:
///  Queue the delete of a proved leftover and name the places this command may not act on,
///  once each.
/// Why:
///   A proved leftover is this program's own,
///  so it is recorded and queued exactly as a handed-over one is,
///  and the record gives it the same bound:
///  the attempts a delete may cost,
///  and then the line that names the directory and the command that removes it
///  (`worktree_removal_leavings_a_command_queues_again.rs`).
///   A place that is not proved is said once and untouched,
///  and the memory of the line goes with the state that earned it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = { queued: false, said: [] };
/// for (const found of readdir(place)) queueOneFoundName({ commonDir, work, named, found, answer });
/// return answer;
/// ```
pub(crate) fn queue_the_leavings_no_record_names(common_dir: &Path, work: &Path, named: &HashSet<PathBuf>) -> Leavings {
    let mut answer: Leavings = Leavings { queued: false, said: Vec::new() };
    for place in the_directories_removals_rename_inside(common_dir) {
        let listing = match std::fs::read_dir(place.as_path()) {
            Ok(listing) => listing,
            // A directory that cannot be listed names no leftover: a place this program cannot read is not one it acts on.
            Err(error) => {
                debug(TAG, format!("{} cannot be listed, so no leftover of a removal is looked for in it ({error})", place.display()).as_str());
                continue;
            }
        };
        for found in listing.flatten() {
            queue_one_found_name(common_dir, work, named, found, &mut answer);
        }
    }
    return answer;
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
/// queueOneFoundName({ commonDir, work, named, found, answer });
/// ```
fn queue_one_found_name(common_dir: &Path, work: &Path, named: &HashSet<PathBuf>, found: std::fs::DirEntry, answer: &mut Leavings) {
    let path: PathBuf = found.path();
    // A name no removal of this program gives anything is the person's own, and a place a record already names is that
    // record's to decide, which this command has just done or left alone.
    if !is_a_leaving_name(path.as_path()) || named.contains(&path) {
        return;
    }
    let Some(named_id) = id_in_the_name(path.as_path()) else { return };
    let kind: std::fs::FileType = match found.file_type() {
        Ok(kind) => kind,
        Err(error) => {
            say_unproved(common_dir, path.as_path(), format!("it cannot be examined ({error})"), answer);
            return;
        }
    };
    let id: u64 = match the_subvolume_the_name_shows(path.as_path(), named_id, kind) {
        Ok(proved) => proved,
        Err(why) => {
            say_unproved(common_dir, path.as_path(), why, answer);
            return;
        }
    };
    queue_the_leaving_found(common_dir, work, path.as_path(), id, answer);
}

/// What:
///  Write the leaving a found directory needs and queue its delete,
///  unless a round already owes that delete,
///  and say so when the record or the queue cannot be written.
/// Why:
///   The record is what bounds a delete the kernel keeps refusing,
///  and the queue record is what a round acts on,
///  so the two outlive the command a person ran exactly as a hand-over's do.
///   A queue record a round already owes is the work this command would ask for,
///  so nothing is queued again and the leaving still names the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leaving = await recordALeavingFoundByAScan(commonDir, removalItem(commonDir, place, id));
/// if (owed.has(leaving.item.id())) return;
/// await queueTheDeleteAgain(commonDir, leaving);
/// ```
fn queue_the_leaving_found(common_dir: &Path, work: &Path, path: &Path, id: u64, answer: &mut Leavings) {
    let place: &Path = match path.parent() {
        Some(parent) => parent,
        None => return,
    };
    let item = removal_item(common_dir, place, id);
    let leaving = match record_a_leaving_found_by_a_scan(common_dir, &item) {
        Ok(found) => found,
        Err(reason) => {
            say_unproved(
                common_dir,
                path,
                format!("it is the subvolume {id} this program renamed aside, and the record of its delete cannot be written ({reason})"),
                answer,
            );
            return;
        }
    };
    if crate::work_item::queued_ids(work).contains(item.id().as_str()) {
        debug(TAG, format!("a round of the service owes the delete of {}, so nothing here is queued", path.display()).as_str());
        return;
    }
    match crate::worktree_removal_leaving_that_outlives_its_record::queue_the_delete_again(common_dir, &leaving) {
        Ok(()) => {
            debug(TAG, format!("the subvolume {} no record named is recorded and queued for a round's delete", path.display()).as_str());
            forget_the_report(common_dir, path);
            answer.queued = true;
        }
        Err(reason) => say_unproved(
            common_dir,
            path,
            format!("it is the subvolume {id} this program renamed aside, and the delete that would remove it cannot be queued ({reason})"),
            answer,
        ),
    }
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
