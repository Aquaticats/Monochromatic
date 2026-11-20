//! What:
//!  The path of one frozen index entry and the places a clone run holds:
//!  the names a tracked path gives,
//!  the times and identity a clone must find again,
//!  the entries a run created,
//!  and the destination directories one entry is made in.
//! Why:
//!  The clone rung walks the index's paths in order and makes each entry below a held handle of its directory,
//!  whatever its name became by then,
//!  so the walking and the holding are one module apart from what is written into the entries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const names = entryNames(path); // throws a reason
//! const directories = new Map<string[], File>();
//! ```

use crate::fs_beneath::{make_directory_at, open_directory_at, plain_components};
use crate::fs_beneath_walk_chain::open_directory_beneath;
use crate::worktree_path_failure_message::subject_failure;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

/// What:
///  The refusal for an index path that is not a plain path inside the worktree.
/// Why:
///   A path with a link,
/// a `..` or a `.git` component could reach outside the tracked files.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `the index path ${path} is not a plain path`
/// ```
fn index_path_refusal(path: &[u8]) -> String {
    return format!("the index path {} is not a plain path", String::from_utf8_lossy(path));
}

/// What:
///  Whether any component of a path is `.git`.
/// Why:
///   The clones never write into a Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.includes('.git')
/// ```
fn names_dot_git(names: &[OsString]) -> bool {
    for name in names {
        if name == ".git" {
            return true;
        }
    }
    return false;
}

/// What:
///  The names of a tracked entry's path below a root,
/// refused unless every component is a plain name and none is `.git`.
/// Why:
///   The index is the source's own,
/// but a path that climbs out of the destination or enters `.git` is never created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryNames(path: Uint8Array): string[]; // throws
/// ```
pub(crate) fn entry_names(path: &[u8]) -> Result<Vec<OsString>, String> {
    let names: Vec<OsString> = match plain_components(Path::new(OsStr::from_bytes(path))) {
        Ok(found) => found,
        Err(_not_plain) => return Err(index_path_refusal(path)),
    };
    return if names.is_empty() || names_dot_git(names.as_slice()) { Err(index_path_refusal(path)) } else { Ok(names) };
}

/// What:
///  The access and modification times of metadata,
/// to the nanosecond.
/// Why:
///   The index entry keeps the source's modification time,
/// and Git trusts a file whose size and time equal it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const times = [atime, mtime];
/// ```
pub(crate) fn times_of(from: &std::fs::Metadata) -> [libc::timespec; 2] {
    return [
        libc::timespec { tv_sec: from.atime() as libc::time_t, tv_nsec: from.atime_nsec() as libc::c_long },
        libc::timespec { tv_sec: from.mtime() as libc::time_t, tv_nsec: from.mtime_nsec() as libc::c_long },
    ];
}

/// What:
///  The identity a clone must find again after it:
/// device,
/// inode,
/// size,
/// and both times.
/// Why:
///   A source that is written while it is cloned changes its size or times;
/// a swapped inode changes the inode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Identity = [dev, ino, size, mtime, ctime];
/// ```
pub(crate) fn identity_of_metadata(from: &std::fs::Metadata) -> [i64; 8] {
    return [from.dev() as i64, from.ino() as i64, from.size() as i64, from.mtime(), from.mtime_nsec(), from.ctime(), from.ctime_nsec(), i64::from(from.mode())];
}

/// What:
///  An entry this run created,
/// named by its components below the destination.
/// Why:
///   Nothing is removed on a failure:
/// a name cannot be checked and unlinked in one step,
/// and the entry may by then be another
///  process's,
/// so the run keeps what it made and reports it.
/// The list is what the report names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Created = { names: string[]; directory: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Created {
    /// The components below the destination.
    pub names: Vec<OsString>,
    /// Whether the entry is a directory.
    pub directory: bool,
}

/// The directories this entry's path was walked through,
/// by their components below the destination (the empty list is the destination),
/// each
///  held open so a later file is placed through the directory it was entered by,
/// whatever its name became.
/// Only the levels of the entry being cloned are held;
/// the index is sorted by path,
/// so no directory the walk left is entered again.
pub(crate) type Directories = HashMap<Vec<OsString>, File>;

/// What:
///  Create the directories of `names[..len - 1]` below the destination and return a handle on the last one,
/// recording the ones made
///  and holding the directories on that entry's path.
/// Why:
///   Each component is opened without following a link or crossing a mount,
/// so the file goes into a directory this run
///  made or found,
/// on the destination's device.
/// The handles of the directories on the path are kept and reused,
/// so an entry pays a lookup for each level
///  above it and no duplicated descriptor,
/// and the handles the walk left are dropped,
/// so a run holds one
///  descriptor per level and never one per directory of the tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parent = ensureParents(directories, names, created);
/// ```
pub(crate) fn ensure_parents<'a>(directories: &'a mut Directories, names: &[OsString], created: &mut Vec<Created>) -> Result<&'a File, String> {
    let parent: &[OsString] = &names[..names.len() - 1];
    if !directories.contains_key(parent) {
        let mut key: Vec<OsString> = Vec::new();
        for index in 0..parent.len() {
            key.push(parent[index].clone());
            if directories.contains_key(key.as_slice()) {
                continue;
            }
            let held: &File = match directories.get(key[..index].as_ref()) {
                Some(found) => found,
                None => return Err(String::from("the destination is not held")),
            };
            let name: &OsString = &parent[index];
            let made: bool = subject_failure(make_directory_at(held, name.as_os_str(), 0o777), &name.to_string_lossy(), "cannot be created")?;
            let next: File = subject_failure(open_directory_at(held, name.as_os_str()), &name.to_string_lossy(), "cannot be entered")?;
            if crate::fs_beneath::device_of(&next).ok() != crate::fs_beneath::device_of(held).ok() {
                return Err(format!("{} is on another device", name.to_string_lossy()));
            }
            if made {
                created.push(Created { names: key.clone(), directory: true });
            }
            directories.insert(key.clone(), next);
        }
        // The index is sorted by path, so a directory this entry left is never needed again;
        // dropping it keeps one descriptor per level of one entry, whichever engine holds the map.
        directories.retain(|held: &Vec<OsString>, _file: &mut File| { return parent.starts_with(held.as_slice()); });
    }
    return match directories.get(parent) {
        Some(found) => Ok(found),
        None => Err(String::from("the destination is not held")),
    };
}

/// What:
///  The source directory holding one entry,
/// opened below the held root or reused from the entry before it.
/// Why:
///   Every source file is opened below a handle of the source root without following a link;
/// consecutive entries of the
///  sorted index share their directory,
/// so the walk is made once per directory and not once per file,
/// and a directory
///  renamed away during the run is read through the handle taken for it,
/// which the rung's own controls state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sourceDirectory = (held, root, names) => held?.[0].join('#') === names.join('#') ? held[1] : (held = [names, openDirectoryBeneath(root, names)])[1];
/// ```
pub(crate) fn source_directory<'a>(held: &'a mut Option<(Vec<OsString>, File)>, root: &File, names: &[OsString], shown: &[u8]) -> Result<&'a File, String> {
    let shared: bool = matches!(held.as_ref(), Some((key, _file)) if key.as_slice() == names);
    if !shared {
        let opened: File = match open_directory_beneath(root, names) {
            Ok(found) => found,
            Err(error) => return Err(format!("the source directory of {} cannot be opened ({error})", String::from_utf8_lossy(shown))),
        };
        *held = Some((names.to_vec(), opened));
    }
    return match held.as_ref() {
        Some((_key, found)) => Ok(found),
        None => Err(String::from("the source directory was not opened")),
    };
}
