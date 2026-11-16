//! What:
//!  Which entries of one tree are not proved to stand, byte for byte, in another.
//! Why:
//!  The old tree, or the subvolume a rollback deletes, is deleted only when every byte it holds is in the tree that is
//!  kept.
//!  A modification time is evidence a person can set with `touch`, `cp -p`, `tar -x` or `rsync -a`, so it is never that
//!  proof; the bytes are read and compared.
//!  One proof serves every way a name reached the kept tree:
//!  a copy of a regular file,
//!  a name linked to an earlier copy,
//!  a file that equaled its copy when the pass listed it,
//!  and an entry a judgement put back,
//!  because none of them is asked how it got there.
//!  A directory is proved by its children,
//!  a symbolic link by the name it holds,
//!  a named pipe by its kind,
//!  and a socket or a device node holds no bytes and is proved by nothing.
//!  An entry this program writes for itself while it builds a tree,
//!  the partial copy that a kill leaves incomplete,
//!  holds no bytes of a person's and is not proved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unproved = entriesNotIn(oldTree, checkout, accepted); // relative paths whose bytes are not in the checkout
//! ```

use crate::plain_file::open_plain_file;
use crate::subvolume_error::MigrateError;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};

/// How many bytes of two files are compared at a time.
const CHUNK: usize = 64 * 1024;

/// What:
///  The relative paths of the entries below `source` whose bytes are not proved to stand in `destination`,
///  except the paths in `accepted`.
///  `accepted` holds what a person has already said may go:
///  the paths of the record beside the checkout when the deletion takes `--discard-left-out`.
/// Why:
///   See the module description.
///  The answer is the paths themselves,
///  so the refusal can name each of them and the person can look at exactly those,
///  and nothing is deleted while one of them stands unproved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unproved = entriesNotIn(oldTree, checkout, accepted);
/// ```
pub(crate) fn entries_not_in(source: &Path, destination: &Path, accepted: &BTreeSet<PathBuf>) -> Result<Vec<PathBuf>, MigrateError> {
    let mut unproved: Vec<PathBuf> = Vec::new();
    let mut pending: Vec<PathBuf> = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let source_directory: PathBuf = source.join(&relative);
        let listing: std::fs::ReadDir = match std::fs::read_dir(source_directory.as_path()) {
            Ok(found) => found,
            // A directory this user cannot list is not proved: its children cannot be compared, so it is named.
            Err(_unreadable) => {
                unproved.push(relative);
                continue;
            }
        };
        for item in listing.flatten() {
            let child: PathBuf = relative.join(item.file_name());
            if accepted.contains(&child) {
                continue;
            }
            // The partial copy this program writes while it builds a tree holds no bytes of a person's.
            if item.file_name() == crate::subvolume_sync_pass_state_and_checks::PARTIAL_NAME {
                continue;
            }
            let Ok(kind) = item.file_type() else {
                unproved.push(child);
                continue;
            };
            // What the migration itself rewrites and what this program wrote for itself is no person's bytes to prove:
            //  the Git index, whose stat fields the migration rewrites on purpose,
            //  and this program's own bookkeeping under the Git directory, which every pass writes and sweeps.
            if is_this_programs_own(source, child.as_path(), kind) {
                continue;
            }
            if kind.is_dir() {
                pending.push(child.clone());
                if !destination.join(&child).is_dir() {
                    unproved.push(child);
                }
                continue;
            }
            if proved_to_stand(source, destination, child.as_path(), kind) {
                continue;
            }
            unproved.push(child);
        }
    }
    unproved.sort();
    return Ok(unproved);
}

/// What:
///  Whether the entry at `relative` in the tree at `root` is one this program writes while it builds a tree,
///  which holds no bytes of a person's and is not proved.
/// Why:
///  The Git index's stat fields are rewritten by the migration on purpose,
///  the locks of a Git command that is writing are Git's own,
///  and the partial copy and the bookkeeping are this program's.
///  One function answers it for the proof and for the record of a pass that never replaces,
///  so the two cannot disagree about what is the person's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ours = (relative, kind) => name(relative) === PARTIAL_NAME || (kind !== 'dir' && (isIndex(relative) || isLock(relative))) || isBookkeeping(relative);
/// ```
pub(crate) fn is_this_programs_own(root: &Path, relative: &Path, kind: std::fs::FileType) -> bool {
    if relative.file_name() == Some(std::ffi::OsStr::new(crate::subvolume_sync_pass_state_and_checks::PARTIAL_NAME)) {
        return true;
    }
    let below_a_git_directory: bool = relative.components().any(|part| return part.as_os_str() == std::ffi::OsStr::new(".git"));
    if !kind.is_dir() && below_a_git_directory {
        let name: &std::ffi::OsStr = relative.file_name().unwrap_or_default();
        if name == "index" || name.as_encoded_bytes().ends_with(b".lock") {
            return true;
        }
    }
    return match std::fs::symlink_metadata(root.join(relative)) {
        Ok(listed) => crate::subvolume_wrapper_bookkeeping::own_bookkeeping_of(root, relative, &listed).is_some(),
        Err(_gone) => false,
    };
}

/// What:
///  Whether the entry at `relative` stands in the destination with the same bytes,
///  or is the same kind of entry where nothing is compared.
/// Why:
///   One comparison per kind,
///  and every failure of a read is a failure of the proof:
///  nothing is taken for proved because it could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const proved = kind === 'dir' ? isDir(dest) : kind === 'file' ? sameBytes(src, dest) : kind === 'symlink' ? sameTarget(src, dest) : kind === 'fifo' ? isFifo(dest) : true;
/// ```
fn proved_to_stand(source: &Path, destination: &Path, relative: &Path, kind: std::fs::FileType) -> bool {
    use std::os::unix::fs::FileTypeExt;
    let other: PathBuf = destination.join(relative);
    let Ok(found) = std::fs::symlink_metadata(other.as_path()) else { return false };
    if kind.is_file() {
        if !found.is_file() || found.len() != std::fs::symlink_metadata(source.join(relative)).map(|original| return original.len()).unwrap_or(u64::MAX) {
            return false;
        }
        return same_bytes(source.join(relative).as_path(), other.as_path());
    }
    if kind.is_symlink() {
        return found.file_type().is_symlink() && std::fs::read_link(source.join(relative)).ok() == std::fs::read_link(&other).ok();
    }
    if kind.is_fifo() {
        return found.file_type().is_fifo();
    }
    // A socket or a device node holds no bytes, and the pass that copies the tree names it as not recreated.
    return true;
}

/// What:
///  Whether two files hold the same bytes,
///  read through the plain-file open so that a name turned into a pipe is refused instead of waited on.
/// Why:
///   A file that cannot be opened on either side is not proved,
///  and a size that differs is answered without reading anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (a, b) => { const x = openPlain(a), y = openPlain(b); for (;;) { const m = read(x, CHUNK), n = read(y, CHUNK); if (!m.equals(n)) return false; if (m.length === 0) return true; } };
/// ```
fn same_bytes(first: &Path, second: &Path) -> bool {
    let Ok(mut left) = open_plain_file(first) else { return false };
    let Ok(mut right) = open_plain_file(second) else { return false };
    let mut a: Vec<u8> = vec![0; CHUNK];
    let mut b: Vec<u8> = vec![0; CHUNK];
    loop {
        let read_left: usize = match Read::read(&mut left, a.as_mut_slice()) {
            Ok(read) => read,
            Err(_unreadable) => return false,
        };
        let read_right: usize = match Read::read(&mut right, b.as_mut_slice()) {
            Ok(read) => read,
            Err(_unreadable) => return false,
        };
        if read_left != read_right || a.get(..read_left) != b.get(..read_right) {
            return false;
        }
        if read_left == 0 {
            return true;
        }
    }
}

/// Controls of the proof stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_tree_proof_tests.rs"]
mod tests;
