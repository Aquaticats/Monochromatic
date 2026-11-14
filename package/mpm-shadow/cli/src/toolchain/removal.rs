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
/// A free hidden name beside `path`, for the one atomic step of a removal.
/// Why:
/// The passes never judge a hidden name,
/// so a run killed between moving an entry aside and removing it leaves nothing a later pass takes for a launcher,
/// a record or a link.
fn temporary_name_beside(path: &Path) -> PathBuf {
    let name: String = path.file_name().map(|name| return name.to_string_lossy().to_string()).unwrap_or_default();
    for attempt in 0..8 {
        let candidate: PathBuf = path.with_file_name(format!(".{name}.{}.{attempt}.retired", std::process::id()));
        if fs::symlink_metadata(&candidate).is_err() {
            return candidate;
        }
    }
    return path.with_file_name(format!(".{name}.{}.retired", std::process::id()));
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
    let temporary: PathBuf = temporary_name_beside(path);
    match fs::rename(path, &temporary) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Removal::Gone),
        Err(error) => return Err(format!("it cannot be removed: {error}")),
    }
    let moved: Option<Judged> = fs::symlink_metadata(&temporary).ok().map(|metadata| return judged_of_metadata(&metadata));
    if moved != Some(judged) {
        return match fs::rename(&temporary, path) {
            Ok(()) => Ok(Removal::Replaced),
            Err(error) => Err(could_not_put_back(path, &temporary, &error)),
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
