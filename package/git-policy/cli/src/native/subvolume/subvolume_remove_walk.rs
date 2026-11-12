//! What:
//!  Empty a directory that is held open,
//! one descriptor-relative step at a time,
//! refusing every nested subvolume and mount
//!  point before anything is deleted.
//! Why:
//!  `std::fs::remove_dir_all` crosses into a nested subvolume (it is a directory to the kernel) and into a bind mount,
//! and
//!  deletes what is there;
//! those hold data that is not part of the tree being removed.
//! Every directory is opened below the
//!  descriptor of its parent without following links or crossing a mount (`openat2`),
//! compared with the device of the top
//!  directory (a nested subvolume has its own),
//! and then emptied through its own descriptor,
//! so a path swapped for another
//!  directory or a link after the check cannot redirect any deletion.
//! A first pass visits the whole tree and deletes nothing;
//!  the second pass does the same checks again at each directory,
//! which catches a boundary that appeared in between.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await verifyBelow(top);  // throws on a nested subvolume or a mount point, deletes nothing
//! await emptyDirectory(top); // checks again at every directory, then unlinks everything below
//! ```

use crate::fs_beneath::{device_of, handle_path, open_directory_widening, remove_at, widen_directory};
use std::os::unix::fs::MetadataExt;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::{Path, PathBuf};

/// The inode number btrfs gives the empty directory that stands for a nested subvolume in a snapshot (`BTRFS_EMPTY_SUBVOL_DIR_OBJECTID`):
///  it reports a device of its own,
/// yet holds nothing and belongs to the snapshot being removed.
const EMPTY_SUBVOLUME_PLACEHOLDER_INODE: u64 = 2;

/// What:
///  One boundary below a directory that a removal may not cross,
/// and the words a report names it by.
/// Why:
///   The removal refuses before it changes anything,
/// and a caller that words its own refusal needs the path and what stands
///  there,
/// not a sentence it would have to read the path back out of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Boundary = { path: string; what: 'a nested subvolume or another filesystem' | 'a mount point' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Boundary {
    /// The entry that stands in the way.
    pub path: PathBuf,
    /// What it is,
    /// in the words every report of it uses.
    pub what: &'static str,
}

/// The words for a directory that holds another subvolume or another filesystem.
const ANOTHER_FILESYSTEM: &str = "a nested subvolume or another filesystem";

/// The words for an entry this device cannot be crossed into.
const A_MOUNT_POINT: &str = "a mount point";

/// Which of the two passes is running.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Pass {
    /// Visit every directory and delete nothing.
    Check,
    /// Delete every entry,
    /// directories last.
    Remove,
}

/// One directory on the work stack:
/// its handle,
/// its name in the parent,
/// its path for a message,
/// and the entries still to visit.
struct Frame {
    /// The directory's descriptor.
    directory: File,
    /// The name in the parent frame's directory;
    /// absent for the top.
    name: Option<OsString>,
    /// The directory's own path,
    /// which a message about an entry below it needs and which the stack's own borrow cannot give.
    path: PathBuf,
    /// Entries not visited yet,
    /// each with whether it is a directory (a link is not one).
    pending: Vec<(OsString, bool)>,
}

/// The entries of an open directory,
/// each with whether it is a directory;
/// only the directories when `directories_only`.
fn list(directory: &File, directories_only: bool) -> std::io::Result<Vec<(OsString, bool)>> {
    let mut entries: Vec<(OsString, bool)> = Vec::new();
    for found in std::fs::read_dir(handle_path(directory))? {
        let item: std::fs::DirEntry = found?;
        let is_directory: bool = item.file_type()?.is_dir();
        if is_directory || !directories_only {
            entries.push((item.file_name(), is_directory));
        }
    }
    return Ok(entries);
}

/// What:
///  Whether the caller named `path` as a folder it owns,
/// so the walk neither visits nor deletes it.
/// Why:
///   A caller that keeps its own state below the top removes it by itself,
/// and the answer is asked of every entry the walk
///  meets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = isTheCallersOwn(except, path); // boolean
/// ```
fn is_the_callers_own(except: &[&Path], path: &Path) -> bool {
    for owned in except {
        if *owned == path {
            return true;
        }
    }
    return false;
}

/// The path of the entry `name` inside the directory the frame holds,
/// for messages and for the question whether a caller owns it.
fn shown_path(frame: &Frame, name: &OsStr) -> PathBuf {
    return frame.path.join(name);
}

/// What:
///  The error that names a boundary:
/// the path,
/// what it is,
/// and the ways forward.
/// Why:
///   The person is the one who can move it,
/// delete it or unmount it,
/// and every way forward is named so that a
///  refusal is not a dead end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const error = boundaryError(top, found); // an Error naming both ways forward
/// ```
fn boundary_error(top: &Path, found: &Boundary) -> std::io::Error {
    return std::io::Error::new(
        std::io::ErrorKind::CrossesDevices,
        format!(
            "{found} is {what} inside {top}, so it holds data that is not part of {top}, and the removal stopped before it \
             changed anything below {top}. To go on: move {found} out of {top}, or delete it yourself (for a subvolume \
             `btrfs subvolume delete {found}`, for a mount `umount {found}`), then run the removal again; to keep it, \
             remove the other entries of {top} by hand.",
            found = found.path.display(),
            what = found.what,
            top = top.display()
        ),
    );
}

/// What:
///  Walk the tree below `top` and,
/// in the removal pass,
/// delete it (not `top` itself),
/// answering the first boundary that stops the walk.
/// Why:
///   Both passes stop at a boundary,
/// and what a caller does with it differs:
/// the removal words it as the
///  refusal of `top`,
/// and a caller that refuses before it starts needs the path itself.
///  A boundary stops the walk in either pass,
/// because nothing below it is part of the tree being removed.
/// A caller that owns folders below `top` names them as `except`:
/// they are neither visited nor deleted here,
/// because what stands in them is that caller's own to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopped = walk(top, shown, Pass.Remove, []); // a Boundary, or undefined when the whole tree went
/// ```
fn walk(top: &File, shown: &Path, pass: Pass, except: &[&Path]) -> std::io::Result<Option<Boundary>> {
    let baseline: u64 = device_of(top)?;
    // A directory the user does not own keeps its mode; the removal below then reports the operating system's refusal.
    let _top_widened = widen_directory(top);
    let mut stack: Vec<Frame> = vec![Frame { directory: top.try_clone()?, name: None, path: shown.to_path_buf(), pending: list(top, pass == Pass::Check)? }];
    while let Some(frame) = stack.last_mut() {
        let Some((name, is_directory)) = frame.pending.pop() else {
            let Some(done) = stack.pop() else { break };
            if pass == Pass::Remove
                && let (Some(name), Some(parent)) = (done.name.as_ref(), stack.last())
                && let Err(error) = remove_at(&parent.directory, name, true)
            {
                return Err(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", done.path.display())));
            }
            continue;
        };
        // The entry's own path decides two things: whether the caller owns it, and what a message names.
        let path: PathBuf = shown_path(frame, &name);
        if is_the_callers_own(except, path.as_path()) {
            continue;
        }
        if !is_directory {
            if let Err(error) = remove_at(&frame.directory, &name, false) {
                return Err(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", path.display())));
            }
            continue;
        }
        let opened: std::io::Result<File> = open_directory_widening(&frame.directory, &name);
        let child: File = match opened {
            Ok(child) => child,
            Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
                return Ok(Some(Boundary { path, what: A_MOUNT_POINT }));
            }
            Err(error) => {
                return Err(std::io::Error::new(error.kind(), format!("{} cannot be opened without following a link ({error})", path.display())));
            }
        };
        let child_facts: std::fs::Metadata = child.metadata()?;
        if child_facts.dev() != baseline && child_facts.ino() != EMPTY_SUBVOLUME_PLACEHOLDER_INODE {
            return Ok(Some(Boundary { path, what: ANOTHER_FILESYSTEM }));
        }
        let _child_widened = widen_directory(&child);
        let pending: Vec<(OsString, bool)> = list(&child, pass == Pass::Check)?;
        stack.push(Frame { directory: child, name: Some(name), path, pending });
    }
    return Ok(None);
}

/// What:
///  Visit every directory below `top` and fail on the first nested subvolume or mount point;
/// delete nothing.
///  Directories the owner cannot read get the owner's access first (a change of mode,
/// never of content).
/// Why:
///   The removal that follows is refused as a whole before it changes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await verifyBelow(top, shown);
/// ```
pub fn verify_below(top: &File, shown: &Path) -> std::io::Result<()> {
    return match walk(top, shown, Pass::Check, &[])? {
        None => Ok(()),
        Some(found) => Err(boundary_error(shown, &found)),
    };
}

/// What:
///  Delete everything below `top` (not `top` itself),
/// refusing a nested subvolume or mount point wherever one is met.
/// Why:
///   See the module description.
/// `shown` is the path that messages use for `top`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await emptyDirectory(top, shown);
/// ```
pub fn empty_directory(top: &File, shown: &Path) -> std::io::Result<()> {
    return match walk(top, shown, Pass::Remove, &[])? {
        None => Ok(()),
        Some(found) => Err(boundary_error(shown, &found)),
    };
}

/// What:
///  The first subvolume or mount point below `top`,
/// or nothing when the tree holds neither;
/// deletes nothing.
/// Why:
///   A caller that refuses before it starts needs the path it found,
/// not the sentence `verify_below` words
///  it as,
/// and it needs the same walk the emptying makes,
/// so the two cannot disagree about what is below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = findBoundaryBelow(top, shown); // Boundary | undefined
/// ```
pub fn find_boundary_below(top: &File, shown: &Path) -> std::io::Result<Option<Boundary>> {
    return walk(top, shown, Pass::Check, &[]);
}

/// What:
///  The same walk with the folders the caller owns left out of it,
/// or nothing when the rest of the tree holds neither a subvolume nor a mount point.
/// Why:
///   A caller that keeps its own state in folders below `top` and removes them by itself needs the proof of
///  everything else:
/// a volume a fill made there in the moments since that folder was cleared is the caller's own,
/// and refusing it
///  as somebody else's would stop a removal over state this program made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const found = findBoundaryBelowExcept(top, shown, [callingFolder]); // Boundary | undefined
/// ```
pub fn find_boundary_below_except(top: &File, shown: &Path, except: &[&Path]) -> std::io::Result<Option<Boundary>> {
    return walk(top, shown, Pass::Check, except);
}

/// Walk controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_remove_walk_tests.rs"]
mod tests;
