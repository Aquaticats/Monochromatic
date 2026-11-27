//! What:
//!  Keeping the bytes a clean displaced,
//! and staging the checked-out lock beside them:
//! the copy that survives the clean under the worktree's Git directory,
//! and the file a step takes the checked-out bytes from when it puts them back.
//! Why:
//!   A comparison of the lock with the snapshot cannot say who wrote it,
//! so the bytes that stand there when the clean ends are kept whole before anything replaces them,
//! and the checked-out bytes are staged in the same directory,
//! outside the worktree,
//! so the checked-out tree never shows them.
//! Both are written exclusively and synced, so a power cut leaves one of the two files whole.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const keepTheBytes = (keptIn: string, lock: string, bytes: Uint8Array) => string;
//! const stageTheCheckout = (directory: string, lock: string, checkout: Uint8Array) => string;
//! ```

/// Import the names a copy of one lock is written under.
use crate::package_repair_cargo_lock_names::{
    LOCK_FILE,
    a_free_staging_name,
    copies_directory,
};
/// Import exclusive,
/// synced,
/// private files and directories.
use crate::private_storage::{
    create_private_directory_all,
    sync_directory,
    write_exclusive_synced,
};
/// Import the borrowed and owned filesystem paths a copy is made for.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  Put the bytes a clean displaced where they survive it,
/// and answer the path they are kept at.
/// Why:
///   The copy is outside the worktree,
///  so the checked-out tree stays clean,
///  and inside the repository's Git directory,
///  so a removal of the worktree that lost them does not take it with it.
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
pub(super) fn keep_the_bytes(
    kept_in: &Path,
    lock: &Path,
    bytes: &[u8],
) -> Result<PathBuf, String> {
    let directory: PathBuf = copies_directory(kept_in, lock);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!(
            "{} could not be made ({error})",
            directory.display()
        ));
    }
    // A lock displaced once already keeps its first copy and this one stands beside it: the first copy is the oldest statement of
    //  what a clean found, and replacing it would lose the bytes of the earlier repair.
    for step in 0.. {
        let name: String = if step == 0 {
            String::from(LOCK_FILE)
        } else {
            format!("{LOCK_FILE}.{step}")
        };
        let copy: PathBuf = directory.join(name);
        match write_exclusive_synced(copy.as_path(), bytes) {
            Ok(()) => {
                if let Err(error) = sync_directory(directory.as_path()) {
                    return Err(format!(
                        "{} could not be made durable ({error})",
                        directory.display()
                    ));
                }
                return Ok(copy);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("{} could not be written ({error})", copy.display())),
        }
    }
    return Err(format!(
        "no free name was found under {}",
        directory.display()
    ));
}

/// What:
///  Stage the checked-out bytes in the displaced directory,
///  where the exchange and the fallback take them from,
///  and answer the name they are at.
/// Why:
///   The staged file is outside the worktree,
///  so the checked-out tree never shows it,
///  and it is written exclusively and synced as every copy there is,
///  so a power cut leaves one of the two files whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = freeStagingName(dir, lock); writeExclusiveSynced(staged, checkout); return staged;
/// ```
pub(super) fn stage_the_checkout(
    directory: &Path,
    lock: &Path,
    checkout: &[u8],
) -> Result<PathBuf, String> {
    let copies: PathBuf = copies_directory(directory, lock);
    if let Err(error) = create_private_directory_all(copies.as_path()) {
        return Err(format!("{} could not be made ({error})", copies.display()));
    }
    let staged: PathBuf = a_free_staging_name(directory, lock);
    if let Err(error) = write_exclusive_synced(staged.as_path(), checkout) {
        return Err(format!(
            "{} could not be written ({error})",
            staged.display()
        ));
    }
    return Ok(staged);
}
