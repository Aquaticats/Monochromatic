//! What:
//!  The look that finds every subvolume and mount point below a tree without changing it,
//!  and the pass that empties a worktree around each one it did not make.
//! Why:
//!  A removal that keeps what it did not make needs both halves of one subject,
//! and they are the two steps of it:
//! the look decides,
//! and the pass deletes everything else while giving back what it kept.
//!  A removal whose tree holds a volume the kernel cannot destroy through,
//! or a mount point Git's own removal would delete
//!  through,
//! goes around it instead:
//! the volume and the directories that lead to it stay exactly as they stand,
//! and every
//!  other entry of the worktree goes.
//!  The two halves live here,
//! apart from the walk that refuses a boundary before it changes anything,
//! because the
//!  decisions differ and the steps they check are the same ones
//! (`subvolume_remove_walk.rs` holds that walk).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const look = lookForBoundaries(top, shown);   // every volume, nothing changed
//! await emptyDirectoryAround(top, shown);       // everything else removed, the volumes kept
//! ```
//!
//! The look that finds every boundary below a tree is `subvolume_remove_around_look.rs`.

pub use super::subvolume_remove_around_look::{
    BoundaryLook,
    LookRefusal,
    look_for_boundaries,
};
use super::subvolume_remove_walk::{
    EMPTY_SUBVOLUME_PLACEHOLDER_INODE,
    list,
    refused_as_not_empty,
    what_is_there_now,
};
use crate::fs_beneath::{
    device_of,
    handle_directory_at,
    handle_path,
    open_directory_at,
    remove_at,
    widen_directory,
    widen_through_a_handle,
};
use std::ffi::{
    OsStr,
    OsString,
};
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};

/// One directory on the work stack of the pass that removes around what it did not make.
struct AroundFrame {
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
    /// Whether a volume below this directory keeps it where it stands.
    keep: bool,
    /// The mode the directory had before the pass gave the owner access,
    /// which it gets back when the directory stays.
    mode: u32,
}

/// The mode a held directory has,
/// as the kernel reports it.
fn mode_of(directory: &File) -> std::io::Result<u32> {
    use std::os::unix::fs::MetadataExt;
    return Ok(directory.metadata()?.mode() & 0o7777);
}

/// Give a directory the mode it had before the pass widened it.
/// Why:
///  A directory that stays is not the pass's to change,
///  and a mode that was given for the pass is given back.
fn give_back_the_mode(
    directory: &File,
    mode: u32,
) {
    use std::os::unix::fs::PermissionsExt;
    // A directory this user does not own keeps the mode the kernel left it: only an owner is refused the change, and the
    //  widening was refused too, so there is nothing to give back.
    let _given = std::fs::set_permissions(
        handle_path(directory),
        std::fs::Permissions::from_mode(mode),
    );
}

/// Mark every frame that holds the directory at `index`,
/// and that directory,
/// as kept.
fn keep_this_and_every_frame_above(
    stack: &mut [AroundFrame],
    index: usize,
) {
    if let Some(frame) = stack.get_mut(index) {
        frame.keep = true;
    }
    for frame in stack.iter_mut().take(index) {
        frame.keep = true;
    }
}

/// What:
///  Remove one entry of a directory the pass that goes around a volume is emptying.
/// Why:
///   Every entry that pass removes leaves the tree here,
/// so a step that records what a removal deletes
///  has one place to join it:
/// the log of the ignored files and nested repositories a removal takes
///  (`removal-refuses-a-worktree-a-process-works-in-and-logs-each-ignored-file-it-deletes`).
/// Keeping the calls in one
///  place is what makes that one place,
/// rather than a second pass that reads what is already gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// take(directory, name, folder); // unlink, or rmdir for a folder
/// ```
fn take(
    directory: &File,
    name: &OsStr,
    folder: bool,
) -> std::io::Result<()> {
    return remove_at(directory, name, folder);
}

/// What:
///  Remove every entry below `top` except each subvolume or mount point the pass meets and the directories that lead
///  to one,
///  giving each kept directory back the mode it had.
/// Why:
///   This is the pass behind "remove around it".
/// A volume is never opened for reading,
/// never renamed,
/// never moved and never deleted here:
/// its handle is asked what it is and the pass goes on,
///  so what it holds is left exactly as it stands.
/// A directory that holds one in its subtree is kept,
/// and its other entries are removed as any other entry of the
///  worktree is.
/// A directory the kernel refuses as not empty after the pass emptied it is looked at again through the descriptor the
///  pass holds:
/// a volume that appeared there is kept with the directory,
/// and anything else that appeared is refused in the
///  words the removal already uses,
/// so nothing is deleted that was not listed.
/// A failure stops the pass where it is;
/// what was removed before it stays removed,
/// and the caller reports that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await emptyAround(top, shown); // throws the reason it stopped
/// ```
pub fn empty_directory_around(
    top: &File,
    shown: &Path,
) -> std::io::Result<()> {
    use std::os::unix::fs::MetadataExt;
    let baseline: u64 = device_of(top)?;
    let top_mode: u32 = mode_of(top)?;
    let _top_opened_to_its_owner = widen_directory(top);
    let mut stack: Vec<AroundFrame> = vec![AroundFrame {
        directory: top.try_clone()?,
        name: None,
        path: shown.to_path_buf(),
        pending: list(top, false)?,
        keep: false,
        mode: top_mode,
    }];
    while !stack.is_empty() {
        let index: usize = stack.len() - 1;
        let next: Option<(OsString, bool)> = stack[index].pending.pop();
        let Some((name, is_directory)) = next else {
            let done: AroundFrame = stack.pop().expect("a frame that was there");
            let Some(own_name) = done.name else {
                if done.keep {
                    give_back_the_mode(&done.directory, done.mode);
                }
                continue;
            };
            if done.keep {
                give_back_the_mode(&done.directory, done.mode);
                continue;
            }
            let parent: &AroundFrame = stack.last().expect("a frame below the top");
            if let Err(error) = take(&parent.directory, own_name.as_os_str(), true) {
                if !refused_as_not_empty(&error) {
                    return Err(std::io::Error::new(
                        error.kind(),
                        format!("{} cannot be removed ({error})", done.path.display()),
                    ));
                }
                // The directory was listed and emptied, and the kernel still finds something in it: a volume or an entry
                //  appeared in that moment. A volume keeps the directory, and the entries are removed as the listing says.
                match look_for_boundaries(&done.directory, done.path.as_path()) {
                    BoundaryLook {
                        found,
                        unreadable: None,
                    } if !found.is_empty() => {
                        let pending: Vec<(OsString, bool)> = list(&done.directory, false)?;
                        stack.push(AroundFrame {
                            directory: done.directory,
                            name: Some(own_name),
                            path: done.path,
                            pending,
                            keep: true,
                            mode: done.mode,
                        });
                        let again: usize = stack.len() - 1;
                        keep_this_and_every_frame_above(&mut stack, again);
                        continue;
                    }
                    BoundaryLook {
                        found,
                        unreadable: None,
                    } if found.is_empty() => {
                        return Err(what_is_there_now(
                            &done.directory,
                            done.path.as_path(),
                            shown,
                        ));
                    }
                    _unread => {
                        return Err(std::io::Error::new(
                            error.kind(),
                            format!(
                                "{} cannot be removed ({error}), and what stands in it now cannot be examined",
                                done.path.display()
                            ),
                        ));
                    }
                }
            }
            continue;
        };
        let path: PathBuf = stack[index].path.join(&name);
        if !is_directory {
            if let Err(error) = take(&stack[index].directory, name.as_os_str(), false) {
                return Err(std::io::Error::new(
                    error.kind(),
                    format!("{} cannot be removed ({error})", path.display()),
                ));
            }
            continue;
        }
        // What the entry is comes before anything else: a volume is left where it stands, and its own mode is never widened
        //  to look inside it.
        let handle: File = match handle_directory_at(&stack[index].directory, name.as_os_str()) {
            Ok(handle) => handle,
            Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
                keep_this_and_every_frame_above(&mut stack, index);
                continue;
            }
            Err(error) => {
                return Err(std::io::Error::new(
                    error.kind(),
                    format!(
                        "{} cannot be opened without following a link ({error})",
                        path.display()
                    ),
                ));
            }
        };
        let facts: std::fs::Metadata = handle.metadata()?;
        if facts.dev() != baseline && facts.ino() != EMPTY_SUBVOLUME_PLACEHOLDER_INODE {
            keep_this_and_every_frame_above(&mut stack, index);
            continue;
        }
        let mode: u32 = facts.mode() & 0o7777;
        // The owner's access is given through the handle that was judged, never through the name: a volume that takes the
        // name in the moment between cannot be the object whose mode changes.
        let _opened_to_its_owner = widen_through_a_handle(&handle);
        #[cfg(test)]
        crate::subvolume_remove_listing_test_support::after_check(path.as_path());
        // The directory is opened for reading, and the name is proved to lead to the object that was checked: the kernel
        // resolves the name a second time here, so what it returns is compared with the handle before anything of it is
        // read, widened or removed.
        let child: File = match open_directory_at(&stack[index].directory, name.as_os_str()) {
            Ok(child) => child,
            Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
                keep_this_and_every_frame_above(&mut stack, index);
                continue;
            }
            Err(error) => {
                return Err(std::io::Error::new(
                    error.kind(),
                    format!("{} cannot be read ({error})", path.display()),
                ));
            }
        };
        let read_facts: std::fs::Metadata = child.metadata()?;
        if read_facts.dev() != facts.dev() || read_facts.ino() != facts.ino() {
            // Another object stands at that name now, which is nobody's to empty here: it and the directories that lead to
            // it stay where they stand.
            keep_this_and_every_frame_above(&mut stack, index);
            continue;
        }
        let _widened = widen_directory(&child);
        let pending: Vec<(OsString, bool)> = list(&child, false)?;
        #[cfg(test)]
        let just_listed: PathBuf = path.clone();
        stack.push(AroundFrame {
            directory: child,
            name: Some(name),
            path,
            pending,
            keep: false,
            mode,
        });
        #[cfg(test)]
        crate::subvolume_remove_listing_test_support::after_listing(just_listed.as_path());
    }
    return Ok(());
}

/// The look's and the pass's controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_remove_around_tests.rs"]
mod tests;
