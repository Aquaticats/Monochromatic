//! What:
//!  The walks of a worktree's tracked files against the index's recorded stat data,
//! by metadata alone or by content for the
//!  files whose metadata cannot vouch for them.
//! Why:
//!  A file's name,
//! size and times are cheap to compare;
//! only a file with a racy or stale time is read,
//! and only then is
//!  its content judged against the committed bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reason = treeDiffersFromIndex(root, entries, indexSeconds); // undefined when every file agrees
//! ```

use crate::fs_beneath::{entry_metadata_at, open_root, plain_components};
use crate::fs_beneath_directory_chain::DirectoryChain;
use crate::repo_read_index::IndexEntry;
use crate::repo_read_layout::ObjectFormat;
use crate::subvolume_index::StatFields;
use crate::subvolume_index_files::fields_of;
use crate::worktree_clean_content::{ContentJudge, ConversionEnvironment, Place};
use crate::worktree_clean_stat_verdicts::{CleanRefusal, StatEntry, StatVerdict, changed_file, file_changed, metadata_verdict, stat_finding};
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

/// What:
///  The directory `parents` lead to below the walk's root,
/// from the chain that keeps the directories it has already opened.
/// Why:
///   The index's paths come in order,
/// so a chain that keeps the part it shares opens each directory once
///  instead of opening every directory above it again for each entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directory = chain.directory(parents); // the handle, or the refusal
/// ```
fn directory_of<'chain>(chain: &'chain mut DirectoryChain<'_>, parents: &[std::ffi::OsString]) -> Option<&'chain std::fs::File> {
    return chain.directory(parents).ok();
}

/// What:
///  The first reason a tracked file of `root` differs from its index entry,
/// or nothing when every file agrees.
/// Why:
///   After a snapshot the same comparison runs against the snapshot's own files,
/// so a source that changed while the
///  snapshot was taken is caught on the copy.
/// The files are reached below a held root with no link followed,
/// so a directory
///  replaced by a link is not read through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndex(root: string, entries: StatEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index(root: &Path, entries: &[StatEntry], index_seconds: u32) -> Option<String> {
    return match open_root(root) {
        Ok(held) => tree_differs_from_index_beneath(&held, entries, index_seconds),
        Err(error) => Some(format!("{} cannot be opened without following a link ({error})", root.display())),
    };
}

/// The first refusal of a tracked file of `root` against its index entry,
/// with its kind;
/// the files are reached below a held
///  root with no link followed.
pub fn tree_refusal(root: &Path, entries: &[StatEntry], index_seconds: u32) -> Option<CleanRefusal> {
    return match open_root(root) {
        Ok(held) => tree_refusal_beneath(&held, entries, index_seconds),
        Err(error) => Some(CleanRefusal::unreadable(format!("{} cannot be opened without following a link ({error})", root.display()))),
    };
}

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
pub fn tree_differs_from_index_by_content(target: &ContentTarget<'_>, entries: &[StatEntry], indexed: &[IndexEntry], index_seconds: u32) -> Option<String> {
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
pub fn tree_fields_by_content(target: &ContentTarget<'_>, entries: &[StatEntry], indexed: &[IndexEntry], index_seconds: u32) -> Result<Vec<StatFields>, CleanRefusal> {
    if entries.len() != indexed.len() {
        return Err(CleanRefusal::unreadable(String::from("the index entries and their stat data are not the same files")));
    }
    let held: std::fs::File = match open_root(target.root) {
        Ok(opened) => opened,
        Err(error) => return Err(CleanRefusal::unreadable(format!("{} cannot be opened without following a link ({error})", target.root.display()))),
    };
    let mut judge: ContentJudge<'_> = ContentJudge::new(&held, target.git_dir, target.common_dir, target.environment, target.format);
    let Ok(mut chain) = DirectoryChain::new(&held) else { return Err(CleanRefusal::unreadable(format!("{} cannot be examined", target.root.display()))) };
    let mut fields: Vec<StatFields> = Vec::with_capacity(entries.len());
    for (entry, committed) in entries.iter().zip(indexed) {
        if entry.path != committed.path {
            return Err(CleanRefusal::unreadable(String::from("the index entries and their stat data are not the same files")));
        }
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice()))) else { return Err(CleanRefusal::unreadable(format!("{shown} is not a plain path"))) };
        let Some((name, parents)) = names.split_last() else { return Err(CleanRefusal::unreadable(format!("{shown} is not a plain path"))) };
        let Some(directory) = directory_of(&mut chain, parents) else { return Err(file_changed(format!("{shown} is missing from the worktree or reached through a link"))) };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else { return Err(file_changed(format!("{shown} is missing from the worktree"))) };
        match metadata_verdict(entry, &metadata, index_seconds) {
            StatVerdict::Agrees => {}
            StatVerdict::Differs(reason) => return Err(file_changed(reason)),
            StatVerdict::Unproven(_reason) => {
                let place: Place<'_> = Place { directory, parents, name: name.as_os_str() };
                if let Err(reason) = judge.agrees(&place, entry, committed.id.as_slice(), index_seconds) {
                    return Err(file_changed(reason));
                }
            }
        }
        fields.push(fields_of(&metadata));
    }
    return Ok(fields);
}

/// What:
///  The first reason a tracked file below the held directory `root` differs from its index entry,
/// or nothing when every
///  file agrees;
/// each file is looked at below handles opened without following a link.
/// Why:
///   A snapshot is checked against the index it will carry;
/// a path walk through a link inside the snapshot would compare
///  a file outside it,
/// so every directory on the way must be a real one on the snapshot's device.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function treeDiffersFromIndexBeneath(root: Dir, entries: StatEntry[], indexSeconds: number): string | undefined;
/// ```
pub fn tree_differs_from_index_beneath(root: &std::fs::File, entries: &[StatEntry], index_seconds: u32) -> Option<String> {
    return match tree_refusal_beneath(root, entries, index_seconds) {
        Some(refusal) => Some(refusal.reason),
        None => None,
    };
}

/// The first refusal of a tracked file below the held directory `root` against its index entry,
/// with its kind.
fn tree_refusal_beneath(root: &std::fs::File, entries: &[StatEntry], index_seconds: u32) -> Option<CleanRefusal> {
    let Ok(mut chain) = DirectoryChain::new(root) else { return Some(CleanRefusal::unreadable(String::from("the worktree root cannot be examined"))) };
    for entry in entries {
        let shown: String = String::from_utf8_lossy(entry.path.as_slice()).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(entry.path.as_slice()))) else { return Some(CleanRefusal::unreadable(format!("{shown} is not a plain path"))) };
        let Some((name, parents)) = names.split_last() else { return Some(CleanRefusal::unreadable(format!("{shown} is not a plain path"))) };
        let Some(directory) = directory_of(&mut chain, parents) else { return changed_file(format!("{shown} is missing from the worktree or reached through a link")) };
        let Ok(metadata) = entry_metadata_at(directory, name.as_os_str()) else { return changed_file(format!("{shown} is missing from the worktree")) };
        if let Some(refusal) = stat_finding(entry, &metadata, index_seconds) {
            return Some(refusal);
        }
    }
    return None;
}

/// What:
///  A walk below a held directory that keeps the last directory it opened.
/// Why:
///   The paths of an index come in order,
/// so most of them share their parent with the one before;
/// one `openat2` chain per
///  directory instead of per file is what the walk costs,
/// and each file is still reached through handles that follow no link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cursor = new StatCursor(root); cursor.fields(path); // StatFields | undefined
/// ```
pub struct StatCursor<'a> {
    /// The chain of directories opened below the root,
    /// or the reason the root cannot be examined.
    chain: Result<DirectoryChain<'a>, std::io::Error>,
}

impl<'a> StatCursor<'a> {
    /// A cursor below `root` that has opened nothing.
    pub fn new(root: &'a std::fs::File) -> Self {
        return Self { chain: DirectoryChain::new(root) };
    }

    /// The metadata of the file `path` names below the root,
    /// or the reason it cannot be found through plain directories.
    fn metadata(&mut self, path: &[u8]) -> Result<std::fs::Metadata, String> {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let Ok(names) = plain_components(Path::new(OsStr::from_bytes(path))) else { return Err(format!("{shown} is not a plain path")) };
        let Some((name, parents)) = names.split_last() else { return Err(format!("{shown} is not a plain path")) };
        let Ok(chain) = &mut self.chain else { return Err(format!("{shown} is missing from the worktree: the worktree root cannot be examined")) };
        let Some(directory) = directory_of(chain, parents) else { return Err(format!("{shown} is missing from the worktree or reached through a link")) };
        return match entry_metadata_at(directory, name.as_os_str()) {
            Ok(found) => Ok(found),
            Err(_missing) => Err(format!("{shown} is missing from the worktree")),
        };
    }

    /// The stat fields of the tracked file `path` below the root,
    /// or nothing when it cannot be found through plain directories.
    pub fn fields(&mut self, path: &[u8]) -> Option<StatFields> {
        return match self.metadata(path) {
            Ok(found) => Some(fields_of(&found)),
            Err(_refused) => None,
        };
    }
}

/// What:
///  The stat fields of the tracked file `path` below the held directory `root`,
/// found through handles,
/// or nothing.
/// Why:
///   The index of a snapshot takes the stat fields of the snapshot's own files;
/// reading them by a path with several
///  components would follow a link substituted below the root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function statFieldsBeneath(root: Dir, path: Uint8Array): StatFields | undefined;
/// ```
pub fn stat_fields_beneath(root: &std::fs::File, path: &[u8]) -> Option<StatFields> {
    return StatCursor::new(root).fields(path);
}
