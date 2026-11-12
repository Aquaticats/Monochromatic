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
use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};

/// The inode number btrfs gives the empty directory that stands for a nested subvolume in a snapshot (`BTRFS_EMPTY_SUBVOL_DIR_OBJECTID`):
///  it reports a device of its own,
/// yet holds nothing and belongs to the snapshot being removed.
const EMPTY_SUBVOLUME_PLACEHOLDER_INODE: u64 = 2;

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
/// and the entries still to visit.
struct Frame {
    /// The directory's descriptor.
    directory: File,
    /// The name in the parent frame's directory;
    /// absent for the top.
    name: Option<OsString>,
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

/// The path of the directory on top of the stack,
/// for messages.
fn shown_path(top: &Path, stack: &[Frame], name: &OsString) -> PathBuf {
    let mut path: PathBuf = top.to_path_buf();
    for frame in stack {
        if let Some(part) = frame.name.as_ref() {
            path.push(part);
        }
    }
    path.push(name);
    return path;
}

/// The error that names a boundary:
/// the path,
/// what it is,
/// and the ways forward.
fn boundary_error(top: &Path, found: &Path, what: &str) -> std::io::Error {
    return std::io::Error::new(
        std::io::ErrorKind::CrossesDevices,
        format!(
            "{found} is {what} inside {top}, so it holds data that is not part of {top}, and the removal stopped before it \
             changed anything below {top}. To go on: move {found} out of {top}, or delete it yourself (for a subvolume \
             `btrfs subvolume delete {found}`, for a mount `umount {found}`), then run the removal again; to keep it, \
             remove the other entries of {top} by hand.",
            found = found.display(),
            what = what,
            top = top.display()
        ),
    );
}

/// Walk the tree below `top` and,
/// in the removal pass,
/// delete it (not `top` itself).
fn walk(top: &File, shown: &Path, pass: Pass) -> std::io::Result<()> {
    let baseline: u64 = device_of(top)?;
    // A directory the user does not own keeps its mode; the removal below then reports the operating system's refusal.
    let _top_widened = widen_directory(top);
    let mut stack: Vec<Frame> = vec![Frame { directory: top.try_clone()?, name: None, pending: list(top, pass == Pass::Check)? }];
    while let Some(frame) = stack.last_mut() {
        let Some((name, is_directory)) = frame.pending.pop() else {
            let Some(done) = stack.pop() else { break };
            if pass == Pass::Remove
                && let (Some(name), Some(parent)) = (done.name.as_ref(), stack.last())
                && let Err(error) = remove_at(&parent.directory, name, true)
            {
                return Err(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", shown_path(shown, &stack, name).display())));
            }
            continue;
        };
        if !is_directory {
            if let Err(error) = remove_at(&frame.directory, &name, false) {
                return Err(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", shown_path(shown, &stack, &name).display())));
            }
            continue;
        }
        let opened: std::io::Result<File> = open_directory_widening(&frame.directory, &name);
        let child: File = match opened {
            Ok(child) => child,
            Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
                return Err(boundary_error(shown, shown_path(shown, &stack, &name).as_path(), "a mount point"));
            }
            Err(error) => {
                return Err(std::io::Error::new(error.kind(), format!("{} cannot be opened without following a link ({error})", shown_path(shown, &stack, &name).display())));
            }
        };
        let child_facts: std::fs::Metadata = child.metadata()?;
        if child_facts.dev() != baseline && child_facts.ino() != EMPTY_SUBVOLUME_PLACEHOLDER_INODE {
            return Err(boundary_error(shown, shown_path(shown, &stack, &name).as_path(), "a nested subvolume or another filesystem"));
        }
        let _child_widened = widen_directory(&child);
        let pending: Vec<(OsString, bool)> = list(&child, pass == Pass::Check)?;
        #[cfg(test)]
        let listed: PathBuf = shown_path(shown, &stack, &name);
        stack.push(Frame { directory: child, name: Some(name), pending });
        #[cfg(test)]
        {
            if pass == Pass::Remove {
                crate::subvolume_remove_listing_test_support::after_listing(listed.as_path());
            }
        }
    }
    return Ok(());
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
    return walk(top, shown, Pass::Check);
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
    return walk(top, shown, Pass::Remove);
}

/// Walk controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_remove_walk_tests.rs"]
mod tests;
