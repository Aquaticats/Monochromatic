//! What:
//!  The walks that judge a tracked file by its content where the index's stat data cannot vouch for it:
//!  the one that returns the stat fields an index carries,
//!  and the one that names the paths a restore must write.
//! Why:
//!  Git hashes a racily clean entry and an entry whose stat data is stale,
//!  and these walks do the same in process,
//!  so a clean worktree that was copied,
//!  cloned or written in the same second as its index is not refused.
//!  The snapshot rung installs the index it proved,
//!  so the walk that judges the files also returns the stat fields that index carries,
//!  and the conversion reads its proof through the same walk.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_clean_tree_walk_held_paths.rs` holds the directory chain and the stat read,
//!  and `worktree_clean_tree_walk.rs` holds the walks that judge by metadata alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fields = treeFieldsByContent(target, entries, indexed, indexSeconds); // or { refused }
//! const paths = pathsWhoseContentDiffers(target, entries, indexed, indexSeconds);
//! ```

use crate::fs_beneath::{
    entry_metadata_at,
    open_root,
    plain_components,
};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::repo_read_index::IndexEntry;
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::{
    fields_of,
    matches_stored_exactly,
};
use crate::worktree_clean_content::{
    ContentJudge,
    ConversionEnvironment,
    Place,
};
use crate::worktree_clean_stat_verdicts::{
    CleanRefusal,
    StatEntry,
    StatVerdict,
    file_changed,
    metadata_verdict,
};
use crate::worktree_clean_tree_walk_held_paths::directory_of;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What the content decision needs besides the entries:
/// the worktree,
/// its Git directories,
/// the environment and the format.
pub struct ContentTarget<'a> {
    /// The worktree root.
    pub root: &'a Path,
    /// The worktree's Git directory.
    pub git_dir: &'a Path,
    /// The shared Git directory.
    pub common_dir: &'a Path,
    /// The configuration and attribute locations.
    pub environment: &'a ConversionEnvironment,
    /// The object format of the repository.
    pub format: ObjectFormat,
}

/// What:
///  The first reason a tracked file of the worktree differs from its index entry,
/// deciding by content where the stat data
///  cannot vouch for the file.
/// Why:
///   Git hashes a racily clean entry and an entry whose stat data is stale;
/// this does the same in process,
/// so a clean
///  worktree that was copied,
/// cloned or written in the same second as its index is not refused.
/// Every file is reached below a
///  held root with no link followed,
/// so a directory replaced by a link is refused instead of being read through.
/// A file that
///  really differs,
/// or one a content conversion could change,
/// is refused with the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndexByContent(target: ContentTarget, entries: StatEntry[], indexed: IndexEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index_by_content(
    target: &ContentTarget<'_>,
    entries: &[StatEntry],
    indexed: &[IndexEntry],
    index_seconds: u32,
) -> Option<String> {
    return match tree_fields_by_content(target, entries, indexed, index_seconds) {
        Ok(_fields) => None,
        Err(refusal) => Some(refusal.reason),
    };
}

/// What:
///  The first refusal of a tracked file of the worktree against its index entry,
/// settling by content the entries the
///  stat data cannot vouch for,
/// and the stat fields of every file,
/// in one walk.
/// Why:
///   The snapshot rung installs the index it proved,
/// so the walk that judges the files also returns the stat fields that index carries,
///  and a file whose stat data is stale or racy is hashed instead of refused;
/// the conversion reads its proof through the same
///  walk.
/// A file that really differs,
/// or one a content conversion could change,
/// is refused with the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeFieldsByContent(target: ContentTarget, entries: StatEntry[], indexed: IndexEntry[], indexSeconds: number): StatFields[] | { refused };
/// ```
pub fn tree_fields_by_content(
    target: &ContentTarget<'_>,
    entries: &[StatEntry],
    indexed: &[IndexEntry],
    index_seconds: u32,
) -> Result<Vec<StatFields>, CleanRefusal> {
    if entries.len() != indexed.len() {
        return Err(CleanRefusal::unreadable(String::from(
            "the index entries and their stat data are not the same files",
        )));
    }
    let held: std::fs::File = match open_root(target.root) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(CleanRefusal::unreadable(format!(
                "{} cannot be opened without following a link ({error})",
                target.root.display()
            )));
        }
    };
    let mut judge: ContentJudge<'_> = ContentJudge::new(
        &held,
        target.git_dir,
        target.common_dir,
        target.environment,
        target.format,
    );
    let Ok(mut chain) = DirectoryChain::new(&held) else {
        return Err(CleanRefusal::unreadable(format!(
            "{} cannot be examined",
            target.root.display()
        )));
    };
    let mut fields: Vec<StatFields> = Vec::with_capacity(entries.len());
    for (entry, committed) in entries.iter().zip(indexed) {
        if entry.path != committed.path {
            return Err(CleanRefusal::unreadable(String::from(
                "the index entries and their stat data are not the same files",
            )));
        }
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice())))
        else {
            return Err(CleanRefusal::unreadable(format!(
                "{shown} is not a plain path"
            )));
        };
        let Some((name, parents)) = names.split_last() else {
            return Err(CleanRefusal::unreadable(format!(
                "{shown} is not a plain path"
            )));
        };
        let Some(directory) = directory_of(&mut chain, parents) else {
            return Err(file_changed(format!(
                "{shown} is missing from the worktree or reached through a link"
            )));
        };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else {
            return Err(file_changed(format!(
                "{shown} is missing from the worktree"
            )));
        };
        match metadata_verdict(entry, &metadata, index_seconds) {
            StatVerdict::Agrees => {}
            StatVerdict::Differs(reason) => return Err(file_changed(reason)),
            StatVerdict::Unproven(_reason) => {
                let place: Place<'_> = Place {
                    directory,
                    parents,
                    name: name.as_os_str(),
                };
                if let Err(reason) =
                    judge.agrees(&place, entry, committed.id.as_slice(), index_seconds)
                {
                    return Err(file_changed(reason));
                }
            }
        }
        fields.push(fields_of(&metadata));
    }
    return Ok(fields);
}

/// What:
///  The tracked paths whose files are not the blobs their entries name,
/// and those whose files could not be read to
///  decide,
/// for a restore that must write the commit's bytes where Git's own stat check cannot see the difference.
/// Why:
///   A restore makes a snapshot of a checkout with changes the commit's tree with
/// `git read-tree --reset -u`,
/// and Git
///  rewrites only a path it can see changed:
/// an edit of the same size made in the same second as the entry it was
///  recorded against keeps every field that check compares,
/// so the snapshot's bytes would stay and the proof after the
///  reset would trust the same fields.
/// The restore removes these paths before the reset,
/// which then writes the
///  commit's bytes there.
/// A file whose bytes could not be compared at all is named the same way:
/// removing it costs a
///  write of the commit's content and never publishes bytes the entry does not name.
/// A file whose recorded stat data is its own to the nanosecond is not read at all,
/// even when Git calls
///  the entry racy:
/// such a file is the one instant the index was written at,
/// so its bytes are the entry's,
/// and a
///  checkout whose index is as new as its files costs nothing here.
/// An entry whose recorded nanosecond count is zero is not such data unless the file's own count is
///  zero too:
/// zero is what a Git built without nanosecond support wrote for a time it could not express,
/// and it
///  matches any present count,
/// so that entry's file is read like any other unproven one.
/// A file the worktree does not hold,
/// or a path whose
///  directories a link replaced,
/// is left to the reset:
/// Git writes what is missing and refuses to write through a
///  link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paths = pathsWhoseContentDiffers(target, entries, indexed, indexSeconds); // throws the reason the walk could not run
/// ```
pub fn paths_whose_content_differs(
    target: &ContentTarget<'_>,
    entries: &[StatEntry],
    indexed: &[IndexEntry],
    index_seconds: u32,
) -> Result<Vec<Vec<u8>>, CleanRefusal> {
    if entries.len() != indexed.len() {
        return Err(CleanRefusal::unreadable(String::from(
            "the index entries and their stat data are not the same files",
        )));
    }
    let held: std::fs::File = match open_root(target.root) {
        Ok(opened) => opened,
        Err(error) => {
            return Err(CleanRefusal::unreadable(format!(
                "{} cannot be opened without following a link ({error})",
                target.root.display()
            )));
        }
    };
    let mut judge: ContentJudge<'_> = ContentJudge::new(
        &held,
        target.git_dir,
        target.common_dir,
        target.environment,
        target.format,
    );
    let Ok(mut chain) = DirectoryChain::new(&held) else {
        return Err(CleanRefusal::unreadable(format!(
            "{} cannot be examined",
            target.root.display()
        )));
    };
    let mut differing: Vec<Vec<u8>> = Vec::new();
    for (entry, committed) in entries.iter().zip(indexed) {
        if entry.path != committed.path {
            return Err(CleanRefusal::unreadable(String::from(
                "the index entries and their stat data are not the same files",
            )));
        }
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice())))
        else {
            return Err(CleanRefusal::unreadable(format!(
                "{shown} is not a plain path"
            )));
        };
        let Some((name, parents)) = names.split_last() else {
            return Err(CleanRefusal::unreadable(format!(
                "{shown} is not a plain path"
            )));
        };
        let Some(directory) = directory_of(&mut chain, parents) else {
            continue;
        };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else {
            continue;
        };
        // Git's own check cannot tell an edit from a file whose size, times and inode it recorded, so the recorded
        //  stat data is compared the strict way here: a file that is not the one the entry recorded is read, and one
        //  that is its own to the nanosecond is believed whatever Git calls a racy entry. A recorded count of zero
        //  proves nothing about the bytes unless the file's count is zero too, because zero stands for any present
        //  count, so such an entry's file is read as well.
        if matches_stored_exactly(&fields_of(&metadata), &entry.stored) {
            continue;
        }
        let place: Place<'_> = Place {
            directory,
            parents,
            name: name.as_os_str(),
        };
        if judge
            .agrees(&place, entry, committed.id.as_slice(), index_seconds)
            .is_err()
        {
            differing.push(entry.path.clone());
        }
    }
    return Ok(differing);
}
