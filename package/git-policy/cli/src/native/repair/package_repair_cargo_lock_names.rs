//! What:
//!  The names under a worktree's Git directory that hold the bytes a clean displaced:
//! the lock file's own name,
//! the directory those copies live in,
//! the suffix of a file staged on its way back into the worktree,
//! and how one lock's path is split into directory names the file system accepts.
//! Why:
//!   The copies must survive the worktree and stand apart from each other,
//! so one lock's path decides its own directory,
//! written as bytes so the name survives whatever the text encoding,
//! and every step that writes or looks for a copy reads these names and no other rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const LOCK_FILE = 'Cargo.lock';
//! const DISPLACED_DIRECTORY = 'monochromatic-git-policy-cli-displaced-locks';
//! const STAGING_SUFFIX = 'restoring';
//! const copiesDirectory = (keptIn: string, lock: string) => string;
//! const aFreeStagingName = (directory: string, lock: string) => string;
//! ```

/// Import a path written into a name as its own bytes.
use crate::path_bytes_in_records::hex_of_path;
/// Import the borrowed and owned filesystem paths a name is made for.
use std::path::{
    Path,
    PathBuf,
};
/// The lock file beside a workspace's root manifest.
pub const LOCK_FILE: &str = "Cargo.lock";

/// The directory,
/// under the worktree's Git directory,
/// that holds the bytes a clean displaced.
pub const DISPLACED_DIRECTORY: &str = "monochromatic-git-policy-cli-displaced-locks";

/// The part of a staged file's name that says it is the checked-out lock on its way into the worktree,
/// and not one of the copies a read displaced.
pub(super) const STAGING_SUFFIX: &str = "restoring";

/// The most bytes of one lock's hexadecimal path that go into one directory name.
/// Why:
///   A file system refuses a name longer than 255 bytes,
/// and a worktree path of ordinary depth is longer than that once it is written in hexadecimal,
///  so the text is split into names that each fit and that read back as one path when they are joined.
pub(super) const NAME_BYTES: usize = 200;

/// What:
///  The directory that holds the copies of one lock's displaced bytes.
/// Why:
///   The lock's own path decides the directory,
/// so two workspaces of one worktree never write over each other,
///  and the path survives as bytes whatever the text encoding of a name,
///  split into names the file system accepts because its hexadecimal form is longer than one name may be.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dir = chunk(hexOf(lock), 200).reduce((path, piece) => join(path, piece), join(keptIn, DISPLACED_DIRECTORY));
/// ```
pub(super) fn copies_directory(
    kept_in: &Path,
    lock: &Path,
) -> PathBuf {
    let hex: String = hex_of_path(lock);
    let mut directory: PathBuf = kept_in.join(DISPLACED_DIRECTORY);
    for piece in hex.as_bytes().chunks(NAME_BYTES) {
        directory = directory.join(String::from_utf8_lossy(piece).into_owned());
    }
    return directory;
}

/// What:
///  A name in the copies directory that no file holds,
///  for bytes a step stages there before they take the lock's place.
/// Why:
///   The names `Cargo.lock`,
/// `Cargo.lock.1`,
/// ... belong to the copies a reading displaced,
/// and a step that staged its bytes under one of them would hide a copy a person is told about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let step = 0; ; step += 1) { const name = step === 0 ? 'Cargo.lock.restoring' : `Cargo.lock.restoring.${step}`; if (!exists(join(dir, name))) return join(dir, name); }
/// ```
pub(super) fn a_free_staging_name(
    directory: &Path,
    lock: &Path,
) -> PathBuf {
    let copies: PathBuf = copies_directory(directory, lock);
    for step in 0.. {
        let name: String = if step == 0 {
            format!("{LOCK_FILE}.{STAGING_SUFFIX}")
        } else {
            format!("{LOCK_FILE}.{STAGING_SUFFIX}.{step}")
        };
        let candidate: PathBuf = copies.join(name);
        if std::fs::symlink_metadata(candidate.as_path()).is_err() {
            return candidate;
        }
    }
    return copies.join(format!("{LOCK_FILE}.{STAGING_SUFFIX}"));
}
