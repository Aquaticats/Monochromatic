//! What:
//! Removing one entry of a checkout only when it is still the entry that was judged,
//! the identity a judgement keeps,
//! and the sentence for a file that could not be put back.
//! Why:
//! A decision about a file (is it mpm-shadow's launcher,
//! its record,
//! its link?)
//! reads a few hundred bytes of a record and up to five million of a program,
//! and a person can put a file of their own at that path while it runs.
//! A check followed by a remove deletes that file.
//! Moving the path aside is one atomic rename,
//! so whatever stood there is in hand and can be compared with the entry the proof read;
//! anything else is put back where it was.
//! A put-back can itself fail (the path can be taken by a directory in the instant between),
//! and then the file is kept at the hidden name it was moved to,
//! which is why the failure has to say that name and the command that puts the file back:
//! a kept file nobody can find is a lost file to its owner.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // removal.ts
//! export type Judged = { device: number; inode: number };
//! export type Removal = 'retired' | 'gone' | 'replaced';
//! export function removeIfStill(path: string, judged: Judged): Removal  // throws
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `shell_word` from `super::cargo_installs`.
use super::cargo_installs::shell_word;

/// What:
/// The entry at a path when it was judged:
/// the file it was, by the two numbers every Unix file carries.
/// Why:
/// Keeping the identity of what was judged lets the removal tell the judged entry from a later one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Judged {
    /// The device of the file system the entry is on.
    #[cfg(unix)]
    device: u64,
    /// The entry's inode on that file system.
    #[cfg(unix)]
    inode: u64,
}

/// What a removal did.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Removal {
    /// This call removed the entry that was judged.
    Retired,
    /// Nothing stood at the path any more.
    Gone,
    /// Something that was not judged stood there: it was put back and left alone.
    Replaced,
}

/// How many hidden names beside an entry are tried before a removal refuses to move it aside.
/// Delegated pick under the owner's rule that nothing waits without end and never discards a person's file;
/// each name is one earlier run's kept file, and a folder with every one of them taken is not one to guess in.
const TEMPORARY_NAME_ATTEMPTS: usize = 64;

/// The identity of an entry from its metadata.
#[cfg(unix)]
pub(crate) fn judged_of_metadata(metadata: &fs::Metadata) -> Judged {
    /// Imports `MetadataExt` from `std::os::unix::fs`.
    use std::os::unix::fs::MetadataExt;
    return Judged { device: metadata.dev(), inode: metadata.ino() };
}

/// Without inodes nothing tells one file at a path from another,
/// so the removal can only remove.
#[cfg(not(unix))]
pub(crate) fn judged_of_metadata(_metadata: &fs::Metadata) -> Judged {
    return Judged {};
}

/// The identity of the entry at `path`, without following a symbolic link.
/// The callers judge from the metadata they already hold, so this is the seam their tests judge by.
#[cfg(test)]
pub(crate) fn judged_of(path: &Path) -> Result<Judged, String> {
    return match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(judged_of_metadata(&metadata)),
        Err(error) => Err(format!("it cannot be inspected: {error}")),
    };
}

/// What:
/// A hidden name beside `path` that nothing stands at, for the one atomic step of a removal.
/// Why:
/// The passes never judge a hidden name,
/// so a run killed between moving an entry aside and removing it leaves nothing a later pass takes for a launcher,
/// a record or a link.
/// A name that is taken is never returned:
/// the file standing there is a person's, kept by an earlier run's failed put-back or left by a killed one,
/// and a rename onto it would overwrite it,
/// so a folder where every name of the bound is taken is an error that removes nothing.
fn temporary_name_beside(path: &Path) -> Result<PathBuf, String> {
    let name: String = path.file_name().map(|name| return name.to_string_lossy().to_string()).unwrap_or_default();
    for attempt in 0..TEMPORARY_NAME_ATTEMPTS {
        let candidate: PathBuf = path.with_file_name(format!(".{name}.{}.{attempt}.retired", std::process::id()));
        if fs::symlink_metadata(&candidate).is_err() {
            return Ok(candidate);
        }
    }
    return Err(format!(
        "every hidden temporary name beside it (from 0 to {}) is taken by a file this tool kept there, so nothing was moved aside \
         and nothing removed",
        TEMPORARY_NAME_ATTEMPTS - 1
    ));
}

/// What:
/// The sentence for a file that was moved aside and could not be put back:
/// what happened,
/// the path the file is at now,
/// and the command that puts it back.
/// Why:
/// The file is a person's,
/// kept but effectively lost while nothing names it,
/// so the one line that reports the failure has to be enough to recover it.
pub(crate) fn could_not_put_back(path: &Path, temporary: &Path, error: &std::io::Error) -> String {
    return format!(
        "something replaced it while it was judged, and what replaced it cannot be put back: {error}; it is kept at {} instead of {}, \
         and `mv -- {} {}` puts it back",
        temporary.display(),
        path.display(),
        shell_word(&temporary.to_string_lossy()),
        shell_word(&path.to_string_lossy())
    );
}

/// What:
/// The sentence for a file that was moved aside and cannot be put back because something newer took the path.
/// Why:
/// Putting it back would overwrite what is there now, and both files are kept only if one of them stays where it is:
/// the moved file is kept at the hidden name it was given,
/// and nothing removes it,
/// so the sentence has to say where it is and who can move it (the person it belongs to).
fn kept_aside(path: &Path, temporary: &Path) -> String {
    return format!(
        "something newer was placed at {} while what stood there was moved aside, so that file was not put back and nothing was \
         deleted: it is kept at {} and nothing removes it; `mv -- {} <where it belongs>` moves it",
        path.display(),
        temporary.display(),
        shell_word(&temporary.to_string_lossy())
    );
}

/// What:
/// Puts the moved entry back at `path`,
/// only if nothing stands at `path` now.
/// Why:
/// `rename` writes over whatever took the path in the window,
/// discarding it without notice (the reviewed defect).
/// A hard link is the atomic "make this name only if it is free": it fails with `EEXIST` instead,
/// and the moved file then stays at its hidden name for the person to move.
/// A directory cannot be hard-linked, and a directory is not an entry this tool removes,
/// so that one is renamed back when the path is free (the only check-then-act left, and it moves nothing over).
pub(crate) fn put_back(path: &Path, temporary: &Path) -> Result<(), String> {
    match fs::hard_link(temporary, path) {
        Ok(()) => {
            let _ = fs::remove_file(temporary);
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(kept_aside(path, temporary)),
        Err(_) => {}
    }
    if fs::symlink_metadata(path).is_ok() {
        return Err(kept_aside(path, temporary));
    }
    return match fs::rename(temporary, path) {
        Ok(()) => Ok(()),
        Err(error) => Err(could_not_put_back(path, temporary, &error)),
    };
}

/// What:
/// Removes `path` only when the entry standing there is still the one judged,
/// and puts back whatever else stands there.
/// Why:
/// Moving the entry aside is one atomic step,
/// so whatever stood there is in hand and can be compared with the judged entry before anything is deleted;
/// a file that arrived after the judgement goes back exactly where it was,
/// and a put-back that cannot happen names the name the file was kept under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeIfStill(path: string, judged: Judged): Removal  // throws
/// ```
pub(crate) fn remove_if_still(path: &Path, judged: Judged) -> Result<Removal, String> {
    let temporary: PathBuf = temporary_name_beside(path)?;
    match fs::rename(path, &temporary) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Removal::Gone),
        Err(error) => return Err(format!("it cannot be removed: {error}")),
    }
    let moved: Option<Judged> = fs::symlink_metadata(&temporary).ok().map(|metadata| return judged_of_metadata(&metadata));
    if moved != Some(judged) {
        return match put_back(path, &temporary) {
            Ok(()) => Ok(Removal::Replaced),
            Err(reason) => Err(reason),
        };
    }
    return match fs::remove_file(&temporary) {
        Ok(()) => Ok(Removal::Retired),
        Err(error) => Err(format!(
            "it was moved aside as the entry judged and then could not be removed: {error}; it is at {}",
            temporary.display()
        )),
    };
}
