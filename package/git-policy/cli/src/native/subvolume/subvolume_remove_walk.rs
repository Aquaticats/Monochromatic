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
//! A directory the kernel then refuses as not empty is looked at again through the descriptor still held:
//! a boundary that appeared after that directory was listed is named there,
//! with the ways forward,
//! and the entries that appeared are named as what the removal deletes nothing of.
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

/// The ways forward for a boundary,
/// as the last sentence of a refusal.
fn ways_forward(top: &Path, found: &Path) -> String {
    return format!(
        "To go on: move {found} out of {top}, or delete it yourself (for a subvolume `btrfs subvolume delete {found}`, for a mount `umount {found}`), then run the removal again; to keep it, remove the other entries of {top} by hand.",
        found = found.display(),
        top = top.display()
    );
}

/// The error that names a boundary the walk met before it changed anything:
/// the path,
/// what it is,
/// and the ways forward.
fn boundary_error(top: &Path, found: &Path, what: &str) -> std::io::Error {
    return std::io::Error::new(
        std::io::ErrorKind::CrossesDevices,
        format!(
            "{found} is {what} inside {top}, so it holds data that is not part of {top}, and the removal stopped before it \
             changed anything below {top}. {ways}",
            found = found.display(),
            what = what,
            top = top.display(),
            ways = ways_forward(top, found)
        ),
    );
}

/// The error that names a boundary the removal met after it had listed and emptied the directory holding it.
fn boundary_met_after_the_listing(listed: &Path, top: &Path, found: &Path, what: &str) -> std::io::Error {
    return std::io::Error::new(
        std::io::ErrorKind::CrossesDevices,
        format!(
            "{listed} cannot be removed although the entries it listed are already removed: {found} is {what}, and it \
             appeared in {listed} after that listing, so the removal stopped there and everything beside it is left as it \
             is. {ways}",
            listed = listed.display(),
            found = found.display(),
            what = what,
            ways = ways_forward(top, found)
        ),
    );
}

/// What:
///  What the walk refused at.
/// Why:
///   The two words a boundary is said in belong to the caller,
/// not to the walk:
/// a check that has changed nothing
///  says so,
/// and the re-examination of a directory the removal already emptied says that instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WalkRefusal = { kind: 'boundary'; found: string; what: string } | { kind: 'failed'; error: Error };
/// ```
enum WalkRefusal {
    /// The tree holds a nested subvolume or a mount point,
    /// at this path.
    Boundary {
        /// The path of the nested subvolume or mount point.
        found: PathBuf,
        /// What it is,
        /// as a refusal names it.
        what: &'static str,
    },
    /// A step of the walk failed.
    Failed(std::io::Error),
}

/// What a walk calls a nested subvolume,
/// or another filesystem,
/// below the tree it removes.
const A_NESTED_SUBVOLUME_OR_ANOTHER_FILESYSTEM: &str = "a nested subvolume or another filesystem";

/// What a walk calls a mount point below the tree it removes.
const A_MOUNT_POINT: &str = "a mount point";

/// What:
///  Whether removing a directory was refused because the directory is not empty.
/// Why:
///   `rmdir` answers `ENOTEMPTY`,
/// and other kernels answer `EEXIST`,
/// so both are the same refusal of a directory
///  something else still holds;
/// the look below the directory the removal still holds is what answers it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notEmpty = (error) => error.code === 'ENOTEMPTY' || error.code === 'EEXIST';
/// ```
pub(crate) fn refused_as_not_empty(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(libc::ENOTEMPTY | libc::EEXIST));
}

/// What:
///  The refusal of a directory the kernel would not remove because it is not empty,
///  although the removal listed that directory and removed every entry of the listing.
/// Why:
///   A directory is listed once,
/// so a nested subvolume,
/// a mount point or an entry that appears after that listing is
///  not among what the removal removes;
/// the kernel then refuses the emptied directory as not empty,
/// and its words alone
///  tell the person nothing about what stands there or what to do about it.
/// The directory is looked at again
///  through the descriptor the removal still holds:
/// a boundary below it is named with the ways forward,
/// and the
///  entries that appeared are named as what the removal deletes nothing of.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = refusedBecauseNotEmpty(dir, listed, top); // names the boundary, the entries that appeared, or the name that changed
/// ```
pub(crate) fn refused_because_not_empty(directory: &File, listed: &Path, top: &Path) -> std::io::Error {
    return match walk(directory, listed, Pass::Check) {
        Err(WalkRefusal::Boundary { found, what }) => boundary_met_after_the_listing(listed, top, found.as_path(), what),
        Err(WalkRefusal::Failed(error)) => std::io::Error::new(error.kind(), format!("{listed} cannot be removed although the entries it listed are already removed, and what is in it now cannot be examined ({error})", listed = listed.display())),
        Ok(()) => what_is_there_now(directory, listed, top),
    };
}

/// What:
///  The refusal of a directory that holds no boundary and could not be removed as not empty.
/// Why:
///   The directory the removal emptied is held open while the name in the held parent leads to something else,
///  so the entries of the held directory are what the person needs:
/// entries that appeared are named,
///  and a held directory that holds nothing says that the directory the kernel looked at was not empty,
///  which is all the evidence the removal has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = whatIsThereNow(dir, listed, top); // the entries that appeared, or the name that no longer leads to it
/// ```
fn what_is_there_now(directory: &File, listed: &Path, top: &Path) -> std::io::Error {
    let mut appeared: Vec<String> = Vec::new();
    match std::fs::read_dir(handle_path(directory)) {
        Ok(listing) => {
            for found in listing {
                match found {
                    Ok(entry) => appeared.push(entry.file_name().to_string_lossy().into_owned()),
                    Err(error) => return std::io::Error::new(error.kind(), format!("{listed} cannot be removed although the entries it listed are already removed, and what is in it now cannot be listed ({error})", listed = listed.display())),
                }
            }
        }
        Err(error) => return std::io::Error::new(error.kind(), format!("{listed} cannot be removed although the entries it listed are already removed, and it cannot be listed now ({error})", listed = listed.display())),
    }
    if !appeared.is_empty() {
        return std::io::Error::new(
            std::io::ErrorKind::DirectoryNotEmpty,
            format!(
                "{listed} cannot be removed although the entries it listed are already removed: {appeared} appeared in it after \
                 that listing, and the removal deletes only what it listed, so they are left as they are and nothing else of \
                 {top} is removed.",
                listed = listed.display(),
                appeared = appeared.join(", "),
                top = top.display()
            ),
        );
    }
    return std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!(
            "{listed} cannot be removed although the entries it listed are already removed: it holds nothing now, so the \
             name in {top} led to a directory that was not empty when the removal asked for it, and whatever is there now is \
             left as it is.",
            listed = listed.display(),
            top = top.display()
        ),
    );
}

/// Walk the tree below `top` and,
/// in the removal pass,
/// delete it (not `top` itself).
fn walk(top: &File, shown: &Path, pass: Pass) -> Result<(), WalkRefusal> {
    let baseline: u64 = device_of(top).map_err(WalkRefusal::Failed)?;
    // A directory the user does not own keeps its mode; the removal below then reports the operating system's refusal.
    let _top_widened = widen_directory(top);
    let mut stack: Vec<Frame> = vec![Frame { directory: top.try_clone().map_err(WalkRefusal::Failed)?, name: None, pending: list(top, pass == Pass::Check).map_err(WalkRefusal::Failed)? }];
    while let Some(frame) = stack.last_mut() {
        let Some((name, is_directory)) = frame.pending.pop() else {
            let Some(done) = stack.pop() else { break };
            if pass == Pass::Remove
                && let (Some(name), Some(parent)) = (done.name.as_ref(), stack.last())
                && let Err(error) = remove_at(&parent.directory, name, true)
            {
                let listed: PathBuf = shown_path(shown, &stack, name);
                // A directory that was listed and emptied is not the directory the kernel was asked to remove when it is not
                //  empty there, so the look below says what stands there now and leaves it as it is.
                if refused_as_not_empty(&error) {
                    return Err(WalkRefusal::Failed(refused_because_not_empty(&done.directory, listed.as_path(), shown)));
                }
                return Err(WalkRefusal::Failed(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", listed.display()))));
            }
            continue;
        };
        if !is_directory {
            if let Err(error) = remove_at(&frame.directory, &name, false) {
                return Err(WalkRefusal::Failed(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", shown_path(shown, &stack, &name).display()))));
            }
            continue;
        }
        let opened: std::io::Result<File> = open_directory_widening(&frame.directory, &name);
        let child: File = match opened {
            Ok(child) => child,
            Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
                return Err(WalkRefusal::Boundary { found: shown_path(shown, &stack, &name), what: A_MOUNT_POINT });
            }
            Err(error) => {
                return Err(WalkRefusal::Failed(std::io::Error::new(error.kind(), format!("{} cannot be opened without following a link ({error})", shown_path(shown, &stack, &name).display()))));
            }
        };
        let child_facts: std::fs::Metadata = child.metadata().map_err(WalkRefusal::Failed)?;
        if child_facts.dev() != baseline && child_facts.ino() != EMPTY_SUBVOLUME_PLACEHOLDER_INODE {
            return Err(WalkRefusal::Boundary { found: shown_path(shown, &stack, &name), what: A_NESTED_SUBVOLUME_OR_ANOTHER_FILESYSTEM });
        }
        let _child_widened = widen_directory(&child);
        let pending: Vec<(OsString, bool)> = list(&child, pass == Pass::Check).map_err(WalkRefusal::Failed)?;
        #[cfg(test)]
        let just_listed: PathBuf = shown_path(shown, &stack, &name);
        stack.push(Frame { directory: child, name: Some(name), pending });
        #[cfg(test)]
        {
            if pass == Pass::Remove {
                crate::subvolume_remove_listing_test_support::after_listing(just_listed.as_path());
            }
        }
    }
    return Ok(());
}

/// What:
///  A walk's refusal in the words the person reads.
/// Why:
///   Both passes refuse a boundary with the same sentence,
/// which says that nothing below the tree was changed
///  before the refusal came;
/// a refused step keeps the words it made.
fn refused_words(outcome: Result<(), WalkRefusal>, shown: &Path) -> std::io::Result<()> {
    return match outcome {
        Ok(()) => Ok(()),
        Err(WalkRefusal::Boundary { found, what }) => Err(boundary_error(shown, found.as_path(), what)),
        Err(WalkRefusal::Failed(error)) => Err(error),
    };
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
    return refused_words(walk(top, shown, Pass::Check), shown);
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
    return refused_words(walk(top, shown, Pass::Remove), shown);
}

/// Walk controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_remove_walk_tests.rs"]
mod tests;
