//! What:
//!  Reading `Cargo.lock` before a `cargo clean` and putting it back as it was afterwards,
//! and keeping the bytes a clean displaced where they cannot be attributed to the clean.
//! Why:
//!  A clean may rewrite the lock file,
//! and a new worktree must stay as checked out;
//! the two steps need nothing from
//!  either clean,
//! so they live in a file of their own that both import:
//! `package_repair_cargo.rs`,
//! which cleans what the worktree already has,
//! and `package_repair_prewarm_job.rs`,
//! which cleans the copy the commit service fills a build directory from after the add returns.
//!   The clean rewrites the lock because the committed one no longer matches its manifest,
//! and it runs in a worktree another command may already be working in,
//! so the bytes standing there when it ends can be Cargo's own rewrite
//!  or a command or person's newer write,
//! and nothing at that moment can tell the two apart.
//! A comparison of the file with the snapshot cannot say who wrote it,
//!  so the bytes that stand there are kept whole under the worktree's Git directory
//!  before the checked-out ones are put back
//! (the owner's rule "never discard user data";
//! a review of `ee545033ad`, finding 3,
//!  found the restore writing over a newer edit without a copy of it).
//! A lock the clean left unchanged is not written at all,
//!  and a lock whose displaced bytes cannot be kept is left exactly as it was found,
//! because putting the checked-out bytes back would then be the only place those bytes existed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = readLock(root); await clean(); restoreLock({ root, before, keptIn: gitDir });
//! ```

/// The debug log of the repair.
use crate::diagnostic_log::debug;
/// Exclusive,
/// synced,
/// private files and directories.
use crate::private_storage::{create_private_directory_all, sync_directory, write_exclusive_synced};
/// A path written into a name as its own bytes.
use crate::path_bytes_in_records::hex_of_path;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairCargoLock";

/// The lock file beside a workspace's root manifest.
const LOCK_FILE: &str = "Cargo.lock";

/// The directory,
/// under the worktree's Git directory,
/// that holds the bytes a clean displaced.
pub const DISPLACED_DIRECTORY: &str = "monochromatic-git-policy-cli-displaced-locks";

/// The most bytes of one lock's hexadecimal path that go into one directory name.
/// Why:
///   A file system refuses a name longer than 255 bytes,
/// and a worktree path of ordinary depth is longer than that once it is written in hexadecimal,
///  so the text is split into names that each fit and that read back as one path when they are joined.
const NAME_BYTES: usize = 200;

/// What:
///  The bytes of the workspace's `Cargo.lock`,
/// or none when it has none.
/// Why:
///   `cargo clean --package` resolves the workspace,
/// and Cargo rewrites a lock that no longer matches its manifest;
/// the
///  new worktree must stay as checked out,
/// so the file is put back afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = readLock(root); // Uint8Array | undefined
/// ```
pub fn read_lock(root: &Path) -> Option<Vec<u8>> {
    return std::fs::read(root.join(LOCK_FILE)).ok();
}

/// What:
///  Put `Cargo.lock` back as `before` read it,
/// keeping the bytes that stood there when the clean ended,
/// and answer the line the person reads when anything was kept.
/// Why:
///   A write is made only when the content changed,
/// so an untouched lock keeps its file time.
///   The bytes that changed it are kept before they are displaced,
/// because the change may be Cargo's rewrite of a lock that no longer matches its manifest
///  or another command's newer write,
/// and the repair may not decide which by guessing:
/// a copy under the worktree's Git directory holds either,
/// and the checked-out lock comes back so the worktree is not left dirty by a clean
///  (`DISPLACED_DIRECTORY`).
/// A lock whose bytes cannot be kept is left exactly as it was found,
/// and the line says so,
/// because the checked-out bytes are in the branch and the bytes standing there are not.
/// A lock that cannot be put back is logged,
///  and left for the person:
/// it is the worktree's own file and the checkout shows the difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = restoreLock({ root, before, keptIn }); // string | undefined
/// ```
pub fn restore_lock(root: &Path, before: Option<&[u8]>, kept_in: Option<&Path>) -> Option<String> {
    let path: PathBuf = root.join(LOCK_FILE);
    let after: Option<Vec<u8>> = std::fs::read(&path).ok();
    if after.as_deref() == before {
        return None;
    }
    let kept: Option<PathBuf> = match (kept_in, after.as_deref()) {
        // Nothing stands there, so nothing is displaced and nothing has to be kept.
        (_no_place, None) => None,
        (None, Some(_bytes)) => {
            return Some(format!(
                "the {LOCK_FILE} of {} changed while the clean ran, and the Git directory to keep those bytes in was not named, so the file is \
                 left exactly as it is found and the worktree shows the difference",
                path.display()
            ));
        }
        (Some(directory), Some(bytes)) => match keep_the_bytes(directory, path.as_path(), bytes) {
            Ok(copy) => Some(copy),
            Err(reason) => {
                return Some(format!(
                    "the {LOCK_FILE} of {} changed while the clean ran, and the bytes that changed it could not be kept ({reason}), so the file \
                     is left exactly as it is found and the worktree shows the difference",
                    path.display()
                ));
            }
        },
    };
    put_back(path.as_path(), before);
    let copy: PathBuf = kept?;
    return Some(format!(
        "the {LOCK_FILE} of {} was not the checked-out file when the clean ended, so the bytes that stood there are kept at {}, and the file \
         is back as checked out",
        root.display(),
        copy.display()
    ));
}

/// What:
///  Write the checked-out bytes over what the clean left,
/// or remove the lock a clean created where the checkout had none.
/// Why:
///   The write is the step that displaces whatever stands there,
/// so the caller keeps those bytes first and this step runs only after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// before === undefined ? unlink(path) : writeFile(path, before);
/// ```
fn put_back(path: &Path, before: Option<&[u8]>) {
    let outcome: std::io::Result<()> = match before {
        Some(bytes) => std::fs::write(path, bytes),
        None => std::fs::remove_file(path),
    };
    match outcome {
        Ok(()) => debug(TAG, format!("{} was put back as checked out after the clean", path.display()).as_str()),
        Err(error) => debug(TAG, format!("{} changed during the clean and could not be put back ({error})", path.display()).as_str()),
    }
}

/// What:
///  Put the bytes a clean displaced where they survive it,
/// and answer the path they are kept at.
/// Why:
///   The copy is outside the worktree,
///  so the checked-out tree stays clean,
///  and inside its Git directory,
///  so it belongs to the worktree that lost them.
/// Its name is the lock's own path in hexadecimal
///  under a directory per path,
///  so two workspaces of one worktree never write over each other,
///  a second displacement of the same lock is a second name rather than a replacement,
///  and the path survives as bytes whatever the text encoding of the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dir = join(keptIn, DISPLACED_DIRECTORY, hexOf(lock)); mkdir(dir); const copy = nextFree(join(dir, 'Cargo.lock')); writeAndSync(copy, bytes); syncDirectory(dir); return copy;
/// ```
fn keep_the_bytes(kept_in: &Path, lock: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let directory: PathBuf = copies_directory(kept_in, lock);
    create_private_directory_all(directory.as_path()).map_err(|error| format!("{} could not be made ({error})", directory.display()))?;
    // A lock displaced once already keeps its first copy and this one stands beside it: the first copy is the oldest statement of
    //  what a clean found, and replacing it would lose the bytes of the earlier repair.
    for step in 0.. {
        let name: String = if step == 0 { String::from(LOCK_FILE) } else { format!("{LOCK_FILE}.{step}") };
        let copy: PathBuf = directory.join(name);
        match write_exclusive_synced(copy.as_path(), bytes) {
            Ok(()) => {
                sync_directory(directory.as_path()).map_err(|error| format!("{} could not be made durable ({error})", directory.display()))?;
                return Ok(copy);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("{} could not be written ({error})", copy.display())),
        }
    }
    return Err(format!("no free name was found under {}", directory.display()));
}

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
fn copies_directory(kept_in: &Path, lock: &Path) -> PathBuf {
    let hex: String = hex_of_path(lock);
    let mut directory: PathBuf = kept_in.join(DISPLACED_DIRECTORY);
    for piece in hex.as_bytes().chunks(NAME_BYTES) {
        directory = directory.join(String::from_utf8_lossy(piece).into_owned());
    }
    return directory;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_lock_tests.rs"]
mod tests;
