//! What:
//!  The boundary a removal may not cross and the words of its refusal:
//! the entry that stands in the way,
//! what stands there,
//! the sentence that names every way forward,
//! and the refusals of a directory the kernel would not remove because it is not empty.
//! Why:
//!  The person is the one who can move,
//! delete or unmount the entry,
//! so every refusal names the path,
//! what is there and the ways on;
//! these words live apart from the walk that meets the boundary (`subvolume_remove_walk.rs`)
//! and from the look that goes around a volume (`subvolume_remove_around.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! throw boundaryError(top, found); // names the entry, what it is and every way forward
//! ```

use crate::fs_beneath::handle_path;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};

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
pub(crate) const ANOTHER_FILESYSTEM: &str = "a nested subvolume or another filesystem";

/// The words for an entry this device cannot be crossed into.
pub(crate) const A_MOUNT_POINT: &str = "a mount point";

/// The ways forward for a boundary,
/// as the last sentence of a refusal.
/// Why the order:
///   Moving the entry out keeps every file it holds and is always a way on,
/// a mount point is kept whole by `umount`,
///  and a subvolume delete destroys everything inside the entry,
/// which the command that refuses here cannot see.
/// So the two commands that keep what stands there come first,
/// and the one that destroys is named for what it does.
fn ways_forward(
    top: &Path,
    found: &Path,
) -> String {
    return format!(
        "To go on: move {found} out of {top}, which keeps every file of it, and run the removal again; a mount point of it goes with `umount {found}`, which keeps its contents. `btrfs subvolume delete {found}` destroys everything inside that subvolume, so use it only for one you know holds nothing of yours; to keep it where it is, remove the other entries of {top} by hand.",
        found = found.display(),
        top = top.display()
    );
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
fn boundary_error(
    top: &Path,
    found: &Boundary,
) -> std::io::Error {
    return std::io::Error::new(
        std::io::ErrorKind::CrossesDevices,
        format!(
            "{found} is {what} inside {top}, so it holds data that is not part of {top}, and the removal stopped before it \
             changed anything below {top}. {ways}",
            found = found.path.display(),
            what = found.what,
            top = top.display(),
            ways = ways_forward(top, found.path.as_path())
        ),
    );
}

/// The error that names a boundary the removal met after it had listed and emptied the directory holding it.
pub(crate) fn boundary_met_after_the_listing(
    listed: &Path,
    top: &Path,
    found: &Boundary,
) -> std::io::Error {
    return std::io::Error::new(
        std::io::ErrorKind::CrossesDevices,
        format!(
            "{listed} cannot be removed although the entries it listed are already removed: {found} is {what}, and it \
             appeared in {listed} after that listing, so the removal stopped there and everything beside it is left as it \
             is. {ways}",
            listed = listed.display(),
            found = found.path.display(),
            what = found.what,
            ways = ways_forward(top, found.path.as_path())
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
/// type WalkRefusal = { kind: 'boundary'; boundary: Boundary } | { kind: 'failed'; error: Error };
/// ```
pub(crate) enum WalkRefusal {
    /// The tree holds a nested subvolume or a mount point,
    /// at this path.
    Boundary(Boundary),
    /// A step of the walk failed.
    Failed(std::io::Error),
}

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
pub(crate) fn what_is_there_now(
    directory: &File,
    listed: &Path,
    top: &Path,
) -> std::io::Error {
    let mut appeared: Vec<String> = Vec::new();
    match std::fs::read_dir(handle_path(directory)) {
        Ok(listing) => {
            for found in listing {
                match found {
                    Ok(entry) => appeared.push(entry.file_name().to_string_lossy().into_owned()),
                    Err(error) => {
                        return std::io::Error::new(
                            error.kind(),
                            format!(
                                "{listed} cannot be removed although the entries it listed are already removed, and what is in it now cannot be listed ({error})",
                                listed = listed.display()
                            ),
                        );
                    }
                }
            }
        }
        Err(error) => {
            return std::io::Error::new(
                error.kind(),
                format!(
                    "{listed} cannot be removed although the entries it listed are already removed, and it cannot be listed now ({error})",
                    listed = listed.display()
                ),
            );
        }
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

/// What:
///  A walk's refusal in the words the person reads.
/// Why:
///   Both passes refuse a boundary with the same sentence,
/// which says that nothing below the tree was changed
///  before the refusal came;
/// a refused step keeps the words it made.
pub(crate) fn refused_words(
    outcome: Result<(), WalkRefusal>,
    shown: &Path,
) -> std::io::Result<()> {
    return match outcome {
        Ok(()) => Ok(()),
        Err(WalkRefusal::Boundary(found)) => Err(boundary_error(shown, &found)),
        Err(WalkRefusal::Failed(error)) => Err(error),
    };
}
