//! What:
//!  The index locks the migration holds: one `index.lock` file for the main index and one for the index of each linked
//!  worktree, each carrying a token that names this migration.
//! Why:
//!  Git writes an index by creating `index.lock` exclusively, writing it and renaming it over `index`. A migration that
//!  swaps the checkout directory under such a writer, or copies a half-written lock into the new tree, damages the
//!  index. So the migration creates the lock files itself, in the old tree before its last copy and in the new tree
//!  before the exchange, and keeps them until the indexes are rewritten. The token makes a lock the migration's own:
//!  a lock file that exists when the migration tries to create one is accepted only if it holds this migration's token,
//!  so another Git command's lock is never mistaken for it, never exempted from the refusals, and never removed.
//!  The linked worktrees' administrative directories live inside `.git`, so their locks and indexes travel with the
//!  main checkout whether or not the worktree directory is inside it.
//!
//! Why these locks stay:
//!  They are Git's own protocol for writing an index, not a lock of this wrapper's invention: Git creates `index.lock`
//!  exclusively and treats its presence as "an index writer is active", so no design that leaves the name alone can keep
//!  Git out of an index while the checkout directory is exchanged. Each lock covers one index (never the repository), is
//!  created with `O_EXCL` and never waited for (a foreign lock stops the migration with the file named), and is held from
//!  before the last copy to the end of the index rewrite, which is the span in which a Git writer could be lost in the
//!  exchange. Which command may migrate a checkout at all is not decided here: `single_owner_claim.rs` decides that
//!  without a lock. Owner rule: concurrency is lock-free first; a lock that must stay covers the smallest unit, with its
//!  reason beside it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const lock of lockPaths(tree)) await acquire(lock, token); ... for (const lock of lockPaths(tree)) await release(lock, token);
//! ```

use crate::subvolume_error::MigrateError;
use crate::subvolume_journal::Journal;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The name of every Git index lock.
const LOCK_NAME: &str = "index.lock";

/// What:
///  The text written into every lock file of this migration.
/// Why:
///   The journal's start time is unique to the migration and survives a kill, so a rerun recognises the locks of the
///  run it resumes and nobody else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const token = `cli-git migrate-to-subvolume ${journal.started}\n`;
/// ```
pub fn token_of(journal: &Journal) -> String {
    return format!("cli-git migrate-to-subvolume {}\n", journal.started());
}

/// What:
///  The lock file paths of a tree: the main index's and one per administrative directory of a linked worktree, sorted.
/// Why:
///   Every index that a Git command could rewrite while the migration runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockPaths(tree: string): string[];
/// ```
pub fn lock_paths(tree: &Path) -> Vec<PathBuf> {
    let git_directory: PathBuf = tree.join(".git");
    let mut found: Vec<PathBuf> = vec![git_directory.join(LOCK_NAME)];
    if let Ok(names) = std::fs::read_dir(git_directory.join("worktrees")) {
        for name in names.flatten() {
            if name.file_type().is_ok_and(|kind: std::fs::FileType| { return kind.is_dir(); }) {
                found.push(name.path().join(LOCK_NAME));
            }
        }
    }
    found.sort();
    return found;
}

/// What:
///  The lock file paths of a tree relative to it.
/// Why:
///   The tree passes leave these paths alone, in both trees: they are never copied and never deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativeLockPaths(tree: string): string[];
/// ```
pub fn relative_lock_paths(tree: &Path) -> Vec<PathBuf> {
    return lock_paths(tree)
        .into_iter()
        .filter_map(|path: PathBuf| { return path.strip_prefix(tree).ok().map(Path::to_path_buf); })
        .collect();
}

/// Whether the lock file holds the token: it is this migration's.
pub fn is_ours(lock: &Path, token: &str) -> bool {
    return std::fs::read(lock).is_ok_and(|content: Vec<u8>| { return content == token.as_bytes(); });
}

/// What:
///  Create the lock file with the token, or accept one that already holds the token.
/// Why:
///   Another Git command's lock means that command is writing an index now; the migration stops and says which file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await acquire(lockPath, token);
/// ```
pub fn acquire(lock: &Path, token: &str) -> Result<(), MigrateError> {
    match std::fs::OpenOptions::new().write(true).create_new(true).open(lock) {
        Ok(mut file) => {
            let written: std::io::Result<()> = file.write_all(token.as_bytes()).and_then(|()| { return file.sync_all(); });
            return written.map_err(|error: std::io::Error| { return MigrateError::io("write the lock", lock, &error); });
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            if is_ours(lock, token) {
                return Ok(());
            }
            return Err(MigrateError::Journal(format!(
                "{} exists and is not this migration's lock: a Git command is writing that index. Wait for it to finish, or remove the file if no Git command is running, then run the command again",
                lock.display()
            )));
        }
        Err(error) => return Err(MigrateError::io("create the lock", lock, &error)),
    }
}

/// What:
///  Remove every lock file of the tree that holds the token; report how many were removed.
/// Why:
///   Only the migration's own locks are released; a lock that holds other text belongs to a Git command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const released = await releaseAll(tree, token);
/// ```
pub fn release_all(tree: &Path, token: &str) -> Result<usize, MigrateError> {
    let mut released: usize = 0;
    for lock in lock_paths(tree) {
        if !is_ours(lock.as_path(), token) {
            continue;
        }
        match std::fs::remove_file(&lock) {
            Ok(()) => released += 1,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(MigrateError::io("remove", lock.as_path(), &error)),
        }
    }
    return Ok(released);
}

/// Lock controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_lock_tests.rs"]
mod tests;
