//! What:
//!  One entry a pull's branch move would remove,
//!  copied beside the Git directory before the failure is worded:
//!  a plain file with its permission bits,
//!  a symbolic link as a link with its own target,
//!  and the numbered folder every pull's copies go into.
//! Why:
//!  The owner's rule is "never discard user data",
//!  and the reset Git's recovery words offer deletes what Git does not track.
//!  The copy is the person's bytes as the worktree held them,
//!  so a file a person runs keeps its mode,
//!  and a link keeps its target's text rather than the bytes the link leads to.
//!  A folder is made with a create that fails when it exists,
//!  so two pulls never write one folder and nothing written here replaces anything already there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copy = freshKeptFolder(gitDir).join(name);
//! keepOne(join(worktreeRoot, name), copy); // a plain file, or a link made again
//! ```

use crate::kept_names_in_the_git_directories::data_folder_for_reading;
use crate::plain_file::open_plain_file;
use std::path::{Path, PathBuf};

/// The folder,
/// inside the Git directory's data folder,
/// that holds one numbered folder per
/// pull whose move could not run.
pub(super) const KEPT_FOLDER_NAME: &str = "pull-kept";

/// How many numbered folders a keep is willing to look at.
/// A number no attempt found is reported instead,
/// so a folder tree nobody cleans costs one message and never a wait.
const KEPT_FOLDER_ATTEMPTS: u32 = 1000;

/// What:
///  A fresh numbered folder under the Git directory's data folder that this pull's copies go into.
/// Why:
///   A numbered folder is made with a create that fails when it exists,
///  so two pulls never write one folder,
///  and nothing this command writes replaces anything already there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let number = 1; number <= KEPT_FOLDER_ATTEMPTS; number += 1) { try { mkdir(`${parent}/${number}`); return `${parent}/${number}`; } catch (e) { if (e.code !== 'EEXIST') throw e; } }
/// ```
pub(super) fn fresh_kept_folder(git_dir: &Path) -> std::io::Result<PathBuf> {
    let parent: PathBuf = data_folder_for_reading(git_dir).join(KEPT_FOLDER_NAME);
    std::fs::create_dir_all(&parent)?;
    for number in 1..=KEPT_FOLDER_ATTEMPTS {
        let candidate: PathBuf = parent.join(number.to_string());
        match std::fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "every numbered folder this command can make already exists"));
}

/// What:
///  Whether a directory holds no entry at all.
/// Why:
///   A directory standing where the new commit writes a file holds no bytes when it is empty,
///  so the message does not ask the person to move one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdirSync(path).length === 0
/// ```
pub(super) fn holds_nothing(directory: &Path) -> bool {
    return match std::fs::read_dir(directory) {
        Ok(mut listing) => listing.next().is_none(),
        Err(_unreadable) => false,
    };
}

/// What:
///  The folders that hold `in_the_copy`.
/// Why:
///   A copy keeps the path's own place under the folder,
///  so a path in a subdirectory needs its folders made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(dirname(target), { recursive: true });
/// ```
fn make_the_copy_folder(in_the_copy: &Path) -> Result<(), String> {
    let Some(parent): Option<&Path> = in_the_copy.parent() else { return Ok(()) };
    if let Err(error) = std::fs::create_dir_all(parent) {
        return Err(format!("the folder {} of its copy could not be made: {error}", parent.display()));
    }
    return Ok(());
}

/// What:
///  A copy of one plain file,
///  opened the way every plain file of a worktree is opened:
///  without blocking and without following a link.
/// Why:
///   A name swapped for a named pipe between the look and the open would otherwise wait for a
///  writer that may never come,
///  and the pull holds nothing while it copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFileSync(openPlain(source), target);
/// ```
fn copy_plain_file(from: &Path, to: &Path, metadata: &std::fs::Metadata) -> Result<(), String> {
    let mut source: std::fs::File = match open_plain_file(from) {
        Ok(opened) => opened,
        Err(error) => return Err(format!("the wrapper could not open {}: {error}", from.display())),
    };
    let mut destination: std::fs::File = match std::fs::File::create(to) {
        Ok(made) => made,
        Err(error) => return Err(format!("its copy at {} could not be made: {error}", to.display())),
    };
    if let Err(error) = std::io::copy(&mut source, &mut destination) {
        return Err(format!("its copy at {} could not be written: {error}", to.display()));
    }
    return copy_the_mode(metadata, to);
}

/// What:
///  Give the copy the permission bits the original holds.
/// Why:
///   An entry a person runs is its bytes and its mode,
///  so a copy that is not executable would not be the file they had.
#[cfg(unix)]
fn copy_the_mode(metadata: &std::fs::Metadata, to: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt as _;
    let mode: u32 = metadata.permissions().mode() & 0o777;
    if let Err(error) = std::fs::set_permissions(to, std::fs::Permissions::from_mode(mode)) {
        return Err(format!("the mode of its copy at {} could not be set: {error}", to.display()));
    }
    return Ok(());
}

/// The same on a platform whose permissions are not Unix bits:
///  the copy keeps what `std::fs::File::create` gave it.
#[cfg(not(unix))]
fn copy_the_mode(_metadata: &std::fs::Metadata, _to: &Path) -> Result<(), String> {
    return Ok(());
}

/// What:
///  Copy what stands at one path the move would overwrite into `in_the_copy`.
/// Why:
///   A plain file is copied byte for byte and a symbolic link is made again with its own target,
///  so the copy stands for the person's entry as the worktree held it;
///  anything else is answered with what stands there,
///  because a directory's contents and a named pipe are not one file this can keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const metadata = lstatSync(path); if (metadata.isDirectory()) throw 'it is a directory'; if (metadata.isSymbolicLink()) symlinkSync(readlinkSync(path), target); else copyPlainFile(path, target);
/// ```
pub(super) fn keep_one(in_the_worktree: &Path, in_the_copy: &Path) -> Result<(), String> {
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(in_the_worktree) {
        Ok(seen) => seen,
        Err(error) => return Err(format!("the wrapper could not read {}: {error}", in_the_worktree.display())),
    };
    let kind: std::fs::FileType = metadata.file_type();
    if kind.is_dir() {
        return Err(String::from("it is a directory, and the wrapper copies plain files and symbolic links"));
    }
    make_the_copy_folder(in_the_copy)?;
    if kind.is_symlink() {
        return copy_symbolic_link(in_the_worktree, in_the_copy);
    }
    if kind.is_file() {
        return copy_plain_file(in_the_worktree, in_the_copy, &metadata);
    }
    return Err(String::from("it is neither a plain file nor a symbolic link"));
}

/// What:
///  The link at `in_the_copy` that names what the link at `in_the_worktree` named.
/// Why:
///   A symbolic link's bytes are its target's text,
///  so the copy is made with that text and never by following it.
#[cfg(unix)]
fn copy_symbolic_link(in_the_worktree: &Path, in_the_copy: &Path) -> Result<(), String> {
    let target: PathBuf = match std::fs::read_link(in_the_worktree) {
        Ok(named) => named,
        Err(error) => return Err(format!("the wrapper could not read the link target of {}: {error}", in_the_worktree.display())),
    };
    if let Err(error) = std::os::unix::fs::symlink(target, in_the_copy) {
        return Err(format!("the link at {} could not be made: {error}", in_the_copy.display()));
    }
    return Ok(());
}

/// The same for a platform without Unix links,
///  which keeps the words and refuses the entry:
///  no gate builds this crate there.
#[cfg(not(unix))]
fn copy_symbolic_link(_in_the_worktree: &Path, _in_the_copy: &Path) -> Result<(), String> {
    return Err(String::from("it is a symbolic link, which this platform's copy cannot make"));
}
