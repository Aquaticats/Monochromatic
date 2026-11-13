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

use crate::fs_beneath::{device_of, handle_directory_at, handle_path, open_directory_at, open_directory_widening, remove_at, widen_directory, widen_through_a_handle};
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
pub(crate) const ANOTHER_FILESYSTEM: &str = "a nested subvolume or another filesystem";

/// The words for an entry this device cannot be crossed into.
pub(crate) const A_MOUNT_POINT: &str = "a mount point";

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

/// The ways forward for a boundary,
/// as the last sentence of a refusal.
/// Why the order:
///   Moving the entry out keeps every file it holds and is always a way on,
/// a mount point is kept whole by `umount`,
///  and a subvolume delete destroys everything inside the entry,
/// which the command that refuses here cannot see.
/// So the two commands that keep what stands there come first,
/// and the one that destroys is named for what it does.
fn ways_forward(top: &Path, found: &Path) -> String {
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
fn boundary_error(top: &Path, found: &Boundary) -> std::io::Error {
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
fn boundary_met_after_the_listing(listed: &Path, top: &Path, found: &Boundary) -> std::io::Error {
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
enum WalkRefusal {
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
    return match walk(directory, listed, Pass::Check, &[]) {
        Err(WalkRefusal::Boundary(found)) => boundary_met_after_the_listing(listed, top, &found),
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
/// const stopped = walk(top, shown, Pass.Remove, []); // a WalkRefusal, or nothing refused when the whole tree went
/// ```
fn walk(top: &File, shown: &Path, pass: Pass, except: &[&Path]) -> Result<(), WalkRefusal> {
    let baseline: u64 = device_of(top).map_err(WalkRefusal::Failed)?;
    // A directory the user does not own keeps its mode; the removal below then reports the operating system's refusal.
    let _top_widened = widen_directory(top);
    let mut stack: Vec<Frame> = vec![Frame { directory: top.try_clone().map_err(WalkRefusal::Failed)?, name: None, path: shown.to_path_buf(), pending: list(top, pass == Pass::Check).map_err(WalkRefusal::Failed)? }];
    while let Some(frame) = stack.last_mut() {
        let Some((name, is_directory)) = frame.pending.pop() else {
            let Some(done) = stack.pop() else { break };
            if pass == Pass::Remove
                && let (Some(name), Some(parent)) = (done.name.as_ref(), stack.last())
                && let Err(error) = remove_at(&parent.directory, name, true)
            {
                // A directory that was listed and emptied is not the directory the kernel was asked to remove when it is not
                //  empty there, so the look below says what stands there now and leaves it as it is.
                if refused_as_not_empty(&error) {
                    return Err(WalkRefusal::Failed(refused_because_not_empty(&done.directory, done.path.as_path(), shown)));
                }
                return Err(WalkRefusal::Failed(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", done.path.display()))));
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
                return Err(WalkRefusal::Failed(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", path.display()))));
            }
            continue;
        }
        let opened: std::io::Result<File> = open_directory_widening(&frame.directory, &name);
        let child: File = match opened {
            Ok(child) => child,
            Err(error) if error.raw_os_error() == Some(libc::EXDEV) => {
                return Err(WalkRefusal::Boundary(Boundary { path, what: A_MOUNT_POINT }));
            }
            Err(error) => {
                return Err(WalkRefusal::Failed(std::io::Error::new(error.kind(), format!("{} cannot be opened without following a link ({error})", path.display()))));
            }
        };
        let child_facts: std::fs::Metadata = child.metadata().map_err(WalkRefusal::Failed)?;
        if child_facts.dev() != baseline && child_facts.ino() != EMPTY_SUBVOLUME_PLACEHOLDER_INODE {
            return Err(WalkRefusal::Boundary(Boundary { path, what: ANOTHER_FILESYSTEM }));
        }
        let _child_widened = widen_directory(&child);
        let pending: Vec<(OsString, bool)> = list(&child, pass == Pass::Check).map_err(WalkRefusal::Failed)?;
        #[cfg(test)]
        let just_listed: PathBuf = path.clone();
        stack.push(Frame { directory: child, name: Some(name), path, pending });
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
        Err(WalkRefusal::Boundary(found)) => Err(boundary_error(shown, &found)),
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
    return refused_words(walk(top, shown, Pass::Check, &[]), shown);
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
    return refused_words(walk(top, shown, Pass::Remove, &[]), shown);
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
    return find_boundary_below_except(top, shown, &[]);
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
    return match walk(top, shown, Pass::Check, except) {
        Ok(()) => Ok(None),
        Err(WalkRefusal::Boundary(found)) => Ok(Some(found)),
        Err(WalkRefusal::Failed(error)) => Err(error),
    };
}

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
pub fn look_for_boundaries(top: &File, shown: &Path) -> BoundaryLook {
    let baseline: u64 = match device_of(top) {
        Ok(device) => device,
        Err(error) => return BoundaryLook { found: Vec::new(), unreadable: Some(format!("{} cannot be asked which filesystem holds it ({error})", shown.display())) },
    };
    let mut found: Vec<Boundary> = Vec::new();
    let pending: Vec<(OsString, bool)> = match list(top, false) {
        Ok(entries) => entries,
        Err(error) => return BoundaryLook { found, unreadable: Some(format!("{} cannot be listed ({error})", shown.display())) },
    };
    let mut stack: Vec<(File, PathBuf, Vec<(OsString, bool)>)> = vec![(match top.try_clone() { Ok(held) => held, Err(error) => return BoundaryLook { found, unreadable: Some(format!("{} cannot be held ({error})", shown.display())) } }, shown.to_path_buf(), pending)];
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
                    found.push(Boundary { path: below, what: A_MOUNT_POINT });
                    continue;
                }
                Err(error) => return BoundaryLook { found, unreadable: Some(format!("{} cannot be read ({error})", below.display())) },
            };
            let facts: std::fs::Metadata = match inner.metadata() {
                Ok(facts) => facts,
                Err(error) => return BoundaryLook { found, unreadable: Some(format!("{} cannot be examined ({error})", below.display())) },
            };
            if facts.dev() != baseline && facts.ino() != EMPTY_SUBVOLUME_PLACEHOLDER_INODE {
                found.push(Boundary { path: below, what: ANOTHER_FILESYSTEM });
                continue;
            }
            let listed: Vec<(OsString, bool)> = match list(&inner, false) {
                Ok(listed) => listed,
                Err(error) => return BoundaryLook { found, unreadable: Some(format!("{} cannot be listed ({error})", below.display())) },
            };
            stack.push((inner, below, listed));
        }
    }
    return BoundaryLook { found, unreadable: None };
}

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
fn give_back_the_mode(directory: &File, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    // A directory this user does not own keeps the mode the kernel left it: only an owner is refused the change, and the
    //  widening was refused too, so there is nothing to give back.
    let _given = std::fs::set_permissions(handle_path(directory), std::fs::Permissions::from_mode(mode));
}

/// Mark every frame that holds the directory at `index`,
/// and that directory,
/// as kept.
fn keep_this_and_every_frame_above(stack: &mut [AroundFrame], index: usize) {
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
fn take(directory: &File, name: &OsStr, folder: bool) -> std::io::Result<()> {
    return remove_at(directory, name, folder);
}

/// What:
///  Remove every entry below `top` except each subvolume or mount point the pass meets and the directories that lead
///  to one,
///  giving each kept directory back the mode it had.
/// Why:
///   This is the pass behind "remove around it".
/// A volume is never opened for reading, never renamed, never moved and never deleted here:
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
pub fn empty_directory_around(top: &File, shown: &Path) -> std::io::Result<()> {
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
                    return Err(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", done.path.display())));
                }
                // The directory was listed and emptied, and the kernel still finds something in it: a volume or an entry
                //  appeared in that moment. A volume keeps the directory, and the entries are removed as the listing says.
                match look_for_boundaries(&done.directory, done.path.as_path()) {
                    BoundaryLook { found, unreadable: None } if !found.is_empty() => {
                        let pending: Vec<(OsString, bool)> = list(&done.directory, false)?;
                        stack.push(AroundFrame { directory: done.directory, name: Some(own_name), path: done.path, pending, keep: true, mode: done.mode });
                        let again: usize = stack.len() - 1;
                        keep_this_and_every_frame_above(&mut stack, again);
                        continue;
                    }
                    BoundaryLook { found, unreadable: None } if found.is_empty() => {
                        return Err(what_is_there_now(&done.directory, done.path.as_path(), shown));
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
                return Err(std::io::Error::new(error.kind(), format!("{} cannot be removed ({error})", path.display())));
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
            Err(error) => return Err(std::io::Error::new(error.kind(), format!("{} cannot be opened without following a link ({error})", path.display()))),
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
            Err(error) => return Err(std::io::Error::new(error.kind(), format!("{} cannot be read ({error})", path.display()))),
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
        stack.push(AroundFrame { directory: child, name: Some(name), path, pending, keep: false, mode });
        #[cfg(test)]
        crate::subvolume_remove_listing_test_support::after_listing(just_listed.as_path());
    }
    return Ok(());
}

/// Walk controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_remove_walk_tests.rs"]
mod tests;
