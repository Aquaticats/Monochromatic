//! What:
//!  One entry of a folder that holds worktrees,
//! while a snapshot is swept for the stand-ins btrfs shows for the nested subvolumes it did not carry:
//! the names a held folder lists,
//! whether an entry is such a stand-in,
//! whether it and the folder holding it are empty,
//! how many entries a trash's manifest already names,
//! and the two moments a control may change the snapshot at.
//! Why:
//!  The inode number and the directory type are the whole test for a stand-in,
//! and an entry that cannot be examined or read is no stand-in and stays.
//! A folder is taken only when every entry of it is an empty stand-in,
//! because every other entry may be someone's data.
//! The seams are here because both the check before a fold and the fold itself ask these questions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const names = listNames(held);
//! const empty = isAnEmptyStandIn(held, name);
//! const seams: FoldSeams = { beforeTake: () => {}, beforePutBack: () => {} };
//! ```

use crate::fs_beneath::{
    entry_metadata_at,
    handle_path,
    open_directory_at,
};
use crate::worktree_trash::TrashManifest;
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::os::unix::fs::MetadataExt;

/// What:
///  The inode number btrfs gives a stand-in,
/// read from the module that owns the number.
/// Why:
///   The leave-out step needs the same number to leave a stand-in alone,
/// and one constant in
///  two places would drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STAND_IN_INODE = BTRFS_STAND_IN_INODE;
/// ```
pub(crate) const BTRFS_STAND_IN_INODE: u64 = crate::fs_kind::BTRFS_STAND_IN_INODE;

/// What:
///  The names a held folder lists.
/// Why:
///   The placeholders are found by looking at every entry of a folder that holds worktrees,
/// and an entry that cannot be read
///  keeps the rung from finishing quietly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = (await readdir(handlePath(dir))).map((e) => e.name);
/// ```
pub(crate) fn listed_names(directory: &File) -> std::io::Result<Vec<OsString>> {
    let mut names: Vec<OsString> = Vec::new();
    for entry in std::fs::read_dir(handle_path(directory))? {
        names.push(entry?.file_name());
    }
    return Ok(names);
}

/// What:
///  Whether the entry `name` below `directory` is the stand-in btrfs shows for a nested subvolume a snapshot did not
///  carry.
/// Why:
///   The inode number and the directory type are the whole test;
/// an entry that cannot be examined is no stand-in and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (found) => found.isDirectory() && found.ino === STAND_IN_INODE
/// ```
pub(crate) fn is_stand_in(
    directory: &File,
    name: &OsStr,
) -> bool {
    return match entry_metadata_at(directory, name) {
        Ok(found) => found.is_dir() && found.ino() == BTRFS_STAND_IN_INODE,
        Err(_unreadable) => false,
    };
}

/// What:
///  Whether the entry `name` below `directory` is a stand-in that holds nothing.
/// Why:
///   The inode number,
/// the directory type and an empty listing are the whole test for what may go into the trash;
/// an
///  entry that cannot be opened or read is no stand-in and stays,
/// so an entry that somehow holds anything is never taken with
///  the folder it sits in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnEmptyStandIn = (dir, name) => isStandIn(dir, name) && readdirSync(handlePath(open(dir, name))).length === 0;
/// ```
pub(crate) fn is_an_empty_stand_in(
    directory: &File,
    name: &OsStr,
) -> bool {
    if !is_stand_in(directory, name) {
        return false;
    }
    return match open_directory_at(directory, name) {
        Ok(held) => listed_names(&held).is_ok_and(holds_nothing),
        Err(_unreadable) => false,
    };
}

/// What:
///  Whether a listing answered with no names at all.
/// Why:
///   A stand-in of a nested subvolume holds nothing,
/// so an empty listing is what makes it a folder that may go into
///  the trash.
pub(crate) fn holds_nothing(listed: Vec<OsString>) -> bool {
    return listed.is_empty();
}

/// What:
///  How many entries a manifest already names.
/// Why:
///   The folder taken here becomes the next entry of the same trash,
/// and its name must not collide with one the
///  leave-out moved there.
pub(crate) fn entries_of(manifest: &TrashManifest) -> usize {
    return manifest.entries.len();
}

/// What:
///  Whether every entry of the held folder `folder` is an empty stand-in and there is at least one.
/// Why:
///   The same question the check before a fold asks,
/// asked of the copy that arrived in the trash;
/// a folder that cannot
///  be listed proves nothing and is never deleted.
pub(crate) fn holds_only_empty_stand_ins(folder: &File) -> bool {
    let listed: Vec<OsString> = match listed_names(folder) {
        Ok(found) => found,
        Err(_unreadable) => return false,
    };
    if listed.is_empty() {
        return false;
    }
    for entry in &listed {
        if !is_an_empty_stand_in(folder, entry.as_os_str()) {
            return false;
        }
    }
    return true;
}

/// What:
///  The moments at which a control may change the snapshot while a folder is taken whole.
/// Why:
///   A folder goes into the trash behind one rename and is checked before it,
/// so only a moment between the check and
///  the rename lets a control show what a directory made in that window comes to;
/// and a folder that is no longer only
///  stand-ins goes back where it was,
/// so only a moment between the rename and that move lets a control show what
///  comes of a name somebody took meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FoldSeams = { beforeTake: () => void; beforePutBack: () => void };
/// ```
pub(crate) struct FoldSeams<'a> {
    /// Called after every entry of a folder was checked and before the folder goes into the trash.
    pub(crate) before_take: &'a mut dyn FnMut(),
    /// Called after the copy in the trash proved to hold more than stand-ins and before the folder goes back.
    pub(crate) before_put_back: &'a mut dyn FnMut(),
}

/// What:
///  Do nothing;
/// the seams of the fold run this unless a control puts work there.
/// Why:
///   The real fold has nothing to do between its steps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doNothing = () => {};
/// ```
pub(crate) fn do_nothing() {}
