//! What:
//!  The look that finds every subvolume and mount point below a tree without changing it:
//! every boundary it met,
//! and whether it could see the whole tree.
//! Why:
//!  A removal that keeps what it did not make needs the whole set before it deletes anything,
//! and a read-only look is what the person's removal,
//! the landed sweep and the commit service all decide with;
//! the pass that empties the tree around each boundary is `subvolume_remove_around.rs`,
//! and the boundary a refuse names is `subvolume_remove_walk_boundary_and_refusal.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = lookForBoundaries(top, shown); // { found, unreadable? }
//! ```

use super::subvolume_remove_walk::{
    A_MOUNT_POINT,
    ANOTHER_FILESYSTEM,
    Boundary,
    EMPTY_SUBVOLUME_PLACEHOLDER_INODE,
    list,
};
use crate::fs_beneath::{
    device_of,
    open_directory_at,
};
use std::ffi::OsString;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Every boundary below `top`,
///  and whether the look could see the whole tree.
/// Why:
///   A removal that keeps what it did not make needs the whole set before it deletes anything,
/// and a look that stopped somewhere still reports what it proved:
///  a boundary it found is a boundary whatever stands behind an unreadable directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BoundaryLook = { found: Boundary[]; unreadable?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BoundaryLook {
    /// The subvolumes and mount points the look met,
    /// in the order it met them.
    pub found: Vec<Boundary>,
    /// Why the look could not cover every directory below `top`,
    /// or nothing when it did.
    pub unreadable: Option<String>,
}

/// What:
///  The refused outcome of a look that can read nothing below `top` at all,
///  which no removal may read as "nothing is there".
/// Why:
///   A tree nothing can be read below may hold a mount point,
/// and Git's own removal would delete through one,
/// so the caller
///  must refuse rather than hand the tree over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = LookRefusal.unreadable('the list of untracked files cannot be asked of Git');
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LookRefusal {
    /// Why the look could not be made,
    /// in words a person reads.
    pub reason: String,
}

/// What:
///  Look for every subvolume and mount point below `top`,
///  deleting nothing,
///  widening nothing,
///  and asking each entry's handle before it is opened for reading.
/// Why:
///   The removal that goes around a volume it did not make must not change what it judges,
/// and a read-only look is what the person's removal,
///  the landed sweep and the commit service all
///  decide with.
/// The look is the one the emptying pass crosses:
/// one plain name at a time below a descriptor,
///  never following a link,
///  never crossing a mount,
///  and a directory on another device than the top is a nested subvolume.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const look = lookForBoundaries(top, shown); // { found, unreadable? }
/// ```
pub fn look_for_boundaries(
    top: &File,
    shown: &Path,
) -> BoundaryLook {
    use std::os::unix::fs::MetadataExt;
    let baseline: u64 = match device_of(top) {
        Ok(device) => device,
        Err(error) => {
            return BoundaryLook {
                found: Vec::new(),
                unreadable: Some(format!(
                    "{} cannot be asked which filesystem holds it ({error})",
                    shown.display()
                )),
            };
        }
    };
    let mut found: Vec<Boundary> = Vec::new();
    let pending: Vec<(OsString, bool)> = match list(top, false) {
        Ok(entries) => entries,
        Err(error) => {
            return BoundaryLook {
                found,
                unreadable: Some(format!("{} cannot be listed ({error})", shown.display())),
            };
        }
    };
    let mut stack: Vec<(File, PathBuf, Vec<(OsString, bool)>)> = vec![(
        match top.try_clone() {
            Ok(held) => held,
            Err(error) => {
                return BoundaryLook {
                    found,
                    unreadable: Some(format!("{} cannot be held ({error})", shown.display())),
                };
            }
        },
        shown.to_path_buf(),
        pending,
    )];
    while let Some((directory, path, mut entries)) = stack.pop() {
        while let Some((name, is_directory)) = entries.pop() {
            let below: PathBuf = path.join(&name);
            if !is_directory {
                continue;
            }
            // The directory is opened once, and what it is is asked of the descriptor that is then read: a name the kernel
            // resolves twice can lead to another object the second time, and a look that judged one object and read another
            // would miss a volume that took the name in between.
            let inner: File = match open_directory_at(&directory, name.as_os_str()) {
                Ok(inner) => inner,
                Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
                    found.push(Boundary {
                        path: below,
                        what: A_MOUNT_POINT,
                    });
                    continue;
                }
                Err(error) => {
                    return BoundaryLook {
                        found,
                        unreadable: Some(format!("{} cannot be read ({error})", below.display())),
                    };
                }
            };
            let facts: std::fs::Metadata = match inner.metadata() {
                Ok(facts) => facts,
                Err(error) => {
                    return BoundaryLook {
                        found,
                        unreadable: Some(format!(
                            "{} cannot be examined ({error})",
                            below.display()
                        )),
                    };
                }
            };
            if facts.dev() != baseline && facts.ino() != EMPTY_SUBVOLUME_PLACEHOLDER_INODE {
                found.push(Boundary {
                    path: below,
                    what: ANOTHER_FILESYSTEM,
                });
                continue;
            }
            let listed: Vec<(OsString, bool)> = match list(&inner, false) {
                Ok(listed) => listed,
                Err(error) => {
                    return BoundaryLook {
                        found,
                        unreadable: Some(format!("{} cannot be listed ({error})", below.display())),
                    };
                }
            };
            stack.push((inner, below, listed));
        }
    }
    return BoundaryLook {
        found,
        unreadable: None,
    };
}
