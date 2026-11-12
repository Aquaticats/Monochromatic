//! What:
//!  What a removal does with the folder Git's own removal left behind,
//!  and with the record that names it.
//! Why:
//!  A worktree removal the wrapper leaves to Git ends in one of three ways:
//!  Git removes the folder and the registration,
//!  Git refuses because the worktree holds changes and nothing moves,
//!  or Git cannot empty the folder because something is writing into it while its walk runs,
//!  and it deletes the registration anyway
//! (`builtin/worktree.c`,
//! `remove_worktree`:
//!  "continue on even if ret is non-zero, there's no going back from here").
//!  The third way leaves a folder nothing registers,
//!  which is the one outcome a removal must never leave:
//!  the command that asked for the removal finishes it here,
//!  and a removal whose process was killed is finished by the next command of the repository
//!  (`worktree_removal_left_behind_record.rs`).
//!  What the folder held before Git ran is the removal's own and goes;
//!  an entry that appeared while the removal was running is somebody else's,
//!  so it stays, the folder stays with it,
//!  it is named on standard error,
//!  and the record keeps pointing at the folder until the removal can be finished.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const remembered = rememberBeforeGit(commonDir, target);   // only a registered worktree
//! const outcome = finishAfterGit(remembered);                // Git came back
//! for (const line of notice(outcome, target)) stderr.write(line);
//! const problems = finishLeftBehindFolders(commonDir);        // every later command
//! ```

use crate::diagnostic_log::{debug, warn};
use crate::fs_beneath::{handle_path, open_root, remove_at};
use crate::fs_btrfs_ioctl::{snapshot_destroy_by_id, subvolume_id};
use crate::fs_kind::is_subvolume_root;
use crate::fs_mount_options::read_mount_facts;
use crate::plain_file::read_plain_file;
use crate::subvolume_remove_walk::{empty_directory, find_boundary_below};
use crate::worktree_removal_left_behind_record::{LeftBehindRecord, RECORD_FOLDER, RECORD_LIMIT, entries_of, entry_kind, forget, owner_may_be_running, read_record, record_before_git, record_returned};
use crate::worktree_remove_registry::registered_worktree;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeRemovalLeftBehind";

/// The mount option that lets a regular user destroy a subvolume with the ioctl.
const REMOVAL_OPTION: &str = "user_subvol_rm_allowed";

/// The most names a notice spells before it counts the rest.
const SHOWN_NAMES: usize = 4;

/// A removal that recorded the folder it hands to Git,
/// and what it recorded.
pub struct Remembered {
    /// The canonical common Git directory of the repository the removal runs in.
    pub common_dir: PathBuf,
    /// The registration's name below `worktrees`,
    /// which is the record's name.
    pub id: OsString,
    /// The folder the removal names.
    pub target: PathBuf,
}

/// What:
///  How a removal of a folder that Git could not empty ended.
/// Why:
///   Three endings need three different words:
///  nothing was left (the folder went),
///  the removal never took the registration (Git refused, nothing changed),
///  or the folder stays with what appeared in it while the removal ran.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LeftBehind = { kind: 'nothing' } | { kind: 'removed' } | { kind: 'kept'; names: string[]; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LeftBehind {
    /// Nothing was left behind:
    /// the folder is gone,
    /// or the removal never took the registration.
    Nothing,
    /// The folder was removed here.
    Removed,
    /// The folder stays,
    /// holding what appeared while the removal ran.
    Kept {
        /// What the folder holds now,
        /// as a person reads the names.
        names: Vec<String>,
        /// Why it stays,
        /// in words a person reads without knowing this program.
        reason: String,
    },
}

/// What:
///  Record the folder this removal is about to hand to Git,
///  when the repository registers it.
/// Why:
///   Only a registered worktree can be left behind by Git's walk,
///  and the registration's name is the record's name,
///  which is what tells the next command whether the removal took it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const remembered = rememberBeforeGit(commonDir, target); // undefined when no registration names target
/// ```
pub fn remember_before_git(common_dir: &Path, target: &Path) -> Option<Remembered> {
    let registration = match registered_worktree(common_dir, target) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("{} is not recorded before Git runs ({reason})", target.display()).as_str());
            return None;
        }
    };
    if let Err(reason) = record_before_git(common_dir, registration.id.as_os_str(), target) {
        // A record that cannot be written costs the automatic finish and nothing else, so Git still removes it.
        warn(TAG, format!("the folder {} hands to Git is not recorded, so a walk that cannot empty it leaves a folder no command finishes: {reason}", target.display()).as_str());
        return None;
    }
    return Some(Remembered { common_dir: common_dir.to_path_buf(), id: registration.id, target: target.to_path_buf() });
}

/// What:
///  Finish what Git left after the removal returned,
///  and say whether anything stays.
/// Why:
///   Git returned, so its walk is over whatever the command does next,
///  and the folder it could not empty is the removal's to finish.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = finishAfterGit(remembered);
/// ```
pub fn finish_after_git(remembered: &Remembered) -> LeftBehind {
    record_returned(remembered.common_dir.as_path(), remembered.id.as_os_str());
    return finish_recorded(remembered.common_dir.as_path(), remembered.id.as_os_str());
}

/// What:
///  Finish one removal from its record,
///  or say what stays.
/// Why:
///   The command that made the removal and every later command ask the same question of the same record,
///  so the answer cannot differ between them.
///  A folder the repository still registers is a worktree Git refused to remove,
///  and nothing was left behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = finishRecorded(commonDir, id);
/// ```
pub fn finish_recorded(common_dir: &Path, id: &OsStr) -> LeftBehind {
    let path: PathBuf = record_path(common_dir, id);
    let Ok(bytes) = read_plain_file(path.as_path(), RECORD_LIMIT) else { return LeftBehind::Nothing };
    let record: LeftBehindRecord = match read_record(bytes.as_slice()) {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("the record {} cannot be used ({reason}), so the removal it names is not finished", path.display()).as_str());
            forget(common_dir, id);
            return LeftBehind::Nothing;
        }
    };
    // A registration that is still there is a worktree Git did not remove, so the folder is where it belongs.
    if common_dir.join("worktrees").join(id).symlink_metadata().is_ok() {
        forget(common_dir, id);
        return LeftBehind::Nothing;
    }
    return match entry_kind(record.target.as_path()) {
        Ok(None) => {
            forget(common_dir, id);
            LeftBehind::Nothing
        }
        Ok(Some(found)) if !found.is_dir() => {
            // The name holds a file or a link now, which no removal made, so nothing here is the removal's and the folder it
            // named is already gone.
            forget(common_dir, id);
            debug(TAG, format!("{} holds a file or a link now, so no removal left a folder there", record.target.display()).as_str());
            LeftBehind::Nothing
        }
        Ok(Some(_directory)) => finish_the_folder(common_dir, id, &record),
        Err(reason) => LeftBehind::Kept { names: Vec::new(), reason },
    };
}

/// What:
///  Remove the folder a removal asked to be gone,
///  leaving every entry the removal did not see.
/// Why:
///   An entry the record does not name appeared while Git was walking the folder,
///  so it is somebody's, and it stays with the folder around it:
///  the wrapper deletes only what the removal asked for.
///  Nothing is deleted from a folder that holds something this program may not delete,
///  a mount point or a subvolume of its own above all,
///  because the walk that empties a folder refuses those where it meets them and the removal stays unfinished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = finishTheFolder(record);
/// ```
fn finish_the_folder(common_dir: &Path, id: &OsStr, record: &LeftBehindRecord) -> LeftBehind {
    let target: &Path = record.target.as_path();
    let held: std::fs::File = match open_root(target) {
        Ok(opened) => opened,
        Err(error) => return LeftBehind::Kept { names: names_of(target), reason: format!("{} cannot be opened without following a link ({error})", target.display()) },
    };
    match find_boundary_below(&held, target) {
        Ok(None) => {}
        Ok(Some(found)) => {
            return LeftBehind::Kept { names: names_of(target), reason: format!("it holds a mount point or a subvolume of its own at {}, which nothing here deletes", found.path.display()) };
        }
        Err(error) => return LeftBehind::Kept { names: names_of(target), reason: format!("its tree cannot be walked for a mount point or a subvolume ({error})") },
    }
    // The entries are read through the descriptor, not through the path, so a folder replaced at that name between the two
    // calls is not the one this finish judges.
    let now: Vec<OsString> = match entries_of(handle_path(&held).as_path()) {
        Ok(found) => found,
        Err(reason) => return LeftBehind::Kept { names: Vec::new(), reason },
    };
    let strangers: Vec<OsString> = now.iter().filter(|name: &&OsString| { return !record.seen.contains(*name); }).cloned().collect();
    if !strangers.is_empty() {
        // What the removal asked for still goes, so the person keeps the smallest thing the removal never named.
        for name in &record.seen {
            if now.contains(name)
                && let Err(error) = remove_entry(&held, target, name)
            {
                return LeftBehind::Kept { names: names_of(target), reason: format!("{} cannot be removed ({error})", target.join(name).display()) };
            }
        }
        return LeftBehind::Kept { names: strangers.iter().map(|name: &OsString| { return name.to_string_lossy().into_owned(); }).collect(), reason: String::from("it appeared while the removal was running, and the removal never named it") };
    }
    return match destroy_the_folder(&held, target) {
        Ok(()) => {
            debug(TAG, format!("removed {}, which Git's own walk could not empty", target.display()).as_str());
            forget(common_dir, id);
            LeftBehind::Removed
        }
        Err(reason) => LeftBehind::Kept { names: names_of(target), reason },
    };
}

/// What:
///  Remove one entry of the folder,
///  recursively when it is a directory.
/// Why:
///   A name the removal saw is deleted with everything below it,
///  which is the set Git's own walk deletes for the removal the person asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeEntry(held, target, name); // throws the reason
/// ```
fn remove_entry(held: &std::fs::File, target: &Path, name: &OsStr) -> Result<(), String> {
    let shown: PathBuf = target.join(name);
    let metadata: std::fs::Metadata = match crate::fs_beneath::entry_metadata_at(held, name) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("{error}")),
    };
    if !metadata.is_dir() {
        return remove_at(held, name, false).map_err(|error| format!("{error}"));
    }
    let inside: std::fs::File = crate::fs_beneath::open_directory_widening(held, name).map_err(|error| format!("{error}"))?;
    empty_directory(&inside, shown.as_path()).map_err(|error| format!("{error}"))?;
    drop(inside);
    return remove_at(held, name, true).map_err(|error| format!("{error}"));
}

/// What:
///  Remove the folder itself,
///  by one subvolume destroy where the kernel and the mount allow it,
///  and by emptying it and removing its name otherwise.
/// Why:
///   A subvolume goes in one call whatever it holds,
///  which is what makes the removal finish for a folder something is still writing into;
///  a plain directory is emptied first,
///  and its name then goes only when nothing is left in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// destroyTheFolder(held, target); // throws the reason
/// ```
fn destroy_the_folder(held: &std::fs::File, target: &Path) -> Result<(), String> {
    let Some(parent) = target.parent() else { return Err(format!("{} has no parent directory", target.display())) };
    let Some(name) = target.file_name() else { return Err(format!("{} has no last component", target.display())) };
    let parent_held: std::fs::File = open_root(parent).map_err(|error| format!("{} cannot be opened without following a link ({error})", parent.display()))?;
    let destroyable: bool = is_subvolume_root(target).unwrap_or(false) && read_mount_facts(target).ok().flatten().is_some_and(|facts| { return facts.has_option(REMOVAL_OPTION); });
    if destroyable {
        let id: u64 = subvolume_id(held, target).map_err(|error| format!("the subvolume of {} cannot be identified ({error})", target.display()))?;
        return snapshot_destroy_by_id(&parent_held, id, target).map_err(|error| format!("the subvolume of {} cannot be destroyed ({error})", target.display()));
    }
    empty_directory(held, target).map_err(|error| format!("{error}"))?;
    return remove_at(&parent_held, name, true).map_err(|error| format!("{} cannot be removed ({error})", handle_path(&parent_held).join(name).display()));
}

/// What:
///  The names one folder holds now,
///  as a person reads them,
///  and nothing when it cannot be listed.
/// Why:
///   A notice that says what stays has to name it,
///  and a folder that cannot be listed is one the reader could not name either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = namesOf(folder); // ['a', 'b']
/// ```
fn names_of(folder: &Path) -> Vec<String> {
    return entries_of(folder).unwrap_or_default().iter().map(|name: &OsString| { return name.to_string_lossy().into_owned(); }).collect();
}

/// What:
///  One path as a notice spells it.
/// Why:
///   A name that is not text still has to reach the person,
///  and the reading of its bytes is what a person can look for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = shownName(path);
/// ```
fn shown_name(path: &Path) -> String {
    return path.to_string_lossy().into_owned();
}

/// What:
///  The path of one record,
///  which is its registration's name below the record folder.
/// Why:
///   The caller names a registration,
///  and the record is found from that name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = join(commonDir, RECORD_FOLDER, id);
/// ```
fn record_path(common_dir: &Path, id: &OsStr) -> PathBuf {
    return common_dir.join(RECORD_FOLDER).join(id);
}

/// What:
///  Finish every folder a removal left behind whose own command is over,
///  and name what could not be finished.
/// Why:
///   A removal killed between its record and the end of Git's walk leaves the folder and the record,
///  and the next command of the repository is what finishes it;
///  a removal still running is left alone,
///  and a record whose removal took the registration is dropped.
///  The records are one `read_dir` and absent when no removal was interrupted,
///  so an ordinary command pays one failed open.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const problems: string[] = finishLeftBehindFolders(commonDir);
/// ```
pub fn finish_left_behind_folders(common_dir: &Path) -> Vec<String> {
    let mut problems: Vec<String> = Vec::new();
    let Ok(listing) = std::fs::read_dir(common_dir.join(RECORD_FOLDER)) else { return problems };
    for entry in listing.flatten() {
        let id: OsString = entry.file_name();
        let record: PathBuf = entry.path();
        let Ok(bytes) = read_plain_file(record.as_path(), RECORD_LIMIT) else { continue };
        let read: LeftBehindRecord = match read_record(bytes.as_slice()) {
            Ok(found) => found,
            Err(reason) => {
                problems.push(format!(
                    "The record of an interrupted worktree removal {record:?} cannot be used ({reason}), so that removal is not finished automatically, and the folder it named is yours to remove: {reason}."
                ));
                forget(common_dir, id.as_os_str());
                continue;
            }
        };
        if !read.returned && owner_may_be_running(read.owner.as_ref()) {
            debug(TAG, format!("the record {} was written by a removal that may still be running, so it is left alone", record.display()).as_str());
            continue;
        }
        match finish_recorded(common_dir, id.as_os_str()) {
            LeftBehind::Nothing => {}
            LeftBehind::Removed => problems.push(finished_notice(read.target.as_path())),
            LeftBehind::Kept { names, reason } => problems.push(kept_notice(read.target.as_path(), names.as_slice(), reason.as_str())),
        }
    }
    return problems;
}

/// What:
///  The line for a removal this command finished after Git's walk failed.
/// Why:
///   The person asked for the worktree to go and it did,
///  but the reason it needed finishing is what tells them something was writing into it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = finishedNotice(target);
/// ```
pub fn finished_notice(target: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: Git could not empty {}, and Git deletes a worktree's registration whether or not its walk emptied the folder; the wrapper removed the worktree here, so anything that was still being written into it, and anything its walk could not take, went with it, which is the risk of git worktree remove.",
        shown_name(target)
    );
}

/// What:
///  The line for a folder that stays because what appeared in it is not the removal's.
/// Why:
///   Never discarding a person's data means such an entry stays with the folder around it,
///  and a folder nothing registers is only acceptable while something names it:
///  the record does, so the next command finishes the removal once the entry is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = keptNotice(target, names, reason);
/// ```
pub fn kept_notice(target: &Path, names: &[String], reason: &str) -> String {
    let listed: String = match names.is_empty() {
        true => String::from("everything it holds"),
        false => shown_names(names),
    };
    return format!(
        "monochromatic-git-policy-cli: Git could not empty {}, and Git deletes a worktree's registration whether or not its walk emptied the folder; {listed} stayed there because {reason}, and the wrapper recorded the folder, so the next command finishes the removal once it is gone (remove it with `rm -rf {}` when nothing in it is yours).",
        shown_name(target),
        shown_name(target)
    );
}

/// What:
///  A few names, and how many more there are.
/// Why:
///   A notice names what a person has to look at,
///  and a list longer than a few would bury the sentence it sits in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = names.slice(0, 4).map((n) => `\`${n}\``).concat(rest ? [`and ${rest} more`] : []).join(', ');
/// ```
fn shown_names(names: &[String]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for name in names.iter().take(SHOWN_NAMES) {
        parts.push(format!("`{name}`"));
    }
    let rest: usize = names.len().saturating_sub(SHOWN_NAMES);
    if rest > 0 {
        parts.push(format!("and {rest} more"));
    }
    return parts.join(", ");
}

/// Controls of this module stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_left_behind_folder_tests.rs"]
mod tests;
