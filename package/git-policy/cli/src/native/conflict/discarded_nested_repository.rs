//! What:
//!  Moving the nested Git repositories a clean would delete beside the audit log,
//!  before Git runs.
//! Why:
//!  A nested repository holds its own history and its own uncommitted files,
//!  and Git's clean with a second `-f` removes the whole directory,
//!  which no copy of the surrounding repository holds.
//!  The owner's answer for uncommitted work is "save,
//! then allow" (2026-11-02,
//!  `conflict_side_guard.rs`),
//! and a rename keeps every byte of the repository without copying it:
//!  the directory moves under `<common git dir>/monochromatic-git-policy-cli/discarded/<stamp>/`,
//!  the clean then runs as typed,
//!  and the audit line names the place.
//!  A move the system refuses is returned,
//!  and every repository already moved is put back before the caller stops the command,
//!  so a stopped save leaves the worktree as the command found it;
//!  a repository that cannot be put back is returned with the place it is,
//!  and the caller names it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copies = await setNestedRepositoriesAside({ commonDir, worktreeRoot, repositories, milliseconds });
//! ```

use crate::conflict_sides::{SavedCopy, SaveFailure, depth_deepest_first, fresh_saved_directory};
use crate::fs_beneath::plain_components;
use crate::private_storage::{create_private_directory_all, sync_directory};
/// `OsStr`/`OsString` are operating-system text of raw bytes.
use std::ffi::OsStr;
/// `OsStrExt` gives `OsStr::from_bytes`,
/// because a path is bytes and not text.
use std::os::unix::ffi::OsStrExt;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths,
///  and a set of them keeps every directory synced once.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What:
///  Move each repository below the place saved copies live,
///  keeping its path below the worktree,
///  and answer where each one went.
///  `&[Vec<u8>]` borrows the paths Git named,
/// relative to the worktree top level.
/// Why:
///   A rename is the whole save:
///  every byte,
///  every link and the repository's own `.git` travel without being read,
///  so a nested repository of any size costs one directory entry move,
///  and a kill leaves it whole in one of the two places.
///  The directories that received an entry are synced after the last move,
///  so the saved repository outlives the power loss that would otherwise lose the only copy.
///  The first refusal stops everything,
///  and every repository already moved is put back before the answer,
///  because the worktree must be exactly as the command found it,
///  and a repository left below the saved copies is one nobody was told about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function setNestedRepositoriesAside(commonDir, worktreeRoot, repositories, milliseconds): SavedCopy[]; // throws SaveStopped
/// ```
pub fn set_nested_repositories_aside(
    common_dir: &Path,
    worktree_root: &Path,
    repositories: &[Vec<u8>],
    milliseconds: i64,
) -> Result<Vec<SavedCopy>, SaveStopped> {
    let base: PathBuf = match fresh_saved_directory(common_dir, milliseconds) {
        Ok(created) => created,
        Err(error) => return Err(SaveStopped { failure: SaveFailure::io(common_dir.display().to_string(), error.to_string()), still_moved: Vec::<SavedCopy>::new() }),
    };
    let mut moved: Vec<Moved> = Vec::<Moved>::new();
    let mut written: BTreeSet<PathBuf> = BTreeSet::<PathBuf>::new();
    for path in repositories {
        let shown: String = String::from_utf8_lossy(path).into_owned();
        let relative: &Path = Path::new(OsStr::from_bytes(path));
        if let Err(error) = plain_components(relative) {
            return Err(stopped(SaveFailure::io(shown, error.to_string()), moved.as_slice()));
        }
        let source: PathBuf = worktree_root.join(relative);
        let destination: PathBuf = base.join(relative);
        let Some(parent) = destination.parent() else {
            return Err(stopped(
                SaveFailure::io(shown, String::from("it has no directory to move the repository into")),
                moved.as_slice(),
            ));
        };
        if let Err(error) = create_private_directory_all(parent) {
            return Err(stopped(SaveFailure::io(shown, error.to_string()), moved.as_slice()));
        }
        written.insert(parent.to_path_buf());
        if let Err(error) = std::fs::rename(source.as_path(), destination.as_path()) {
            return Err(stopped(SaveFailure::io(shown, describe_rename_failure(error)), moved.as_slice()));
        }
        moved.push(Moved {
            source,
            destination: destination.clone(),
            copy: SavedCopy { path: shown, saved_to: Some(destination) },
        });
    }
    // Deepest first, then the directories that hold the copies' root, so every moved entry is durable.
    let mut ordered: Vec<&PathBuf> = written.iter().collect();
    ordered.sort_by_key(depth_deepest_first);
    for each in ordered {
        if let Err(error) = sync_directory(each.as_path()) {
            return Err(stopped(SaveFailure::io(each.display().to_string(), error.to_string()), moved.as_slice()));
        }
    }
    let discarded: &Path = base.parent().unwrap_or(base.as_path());
    for each in [base.as_path(), discarded] {
        if let Err(error) = sync_directory(each) {
            return Err(stopped(SaveFailure::io(each.display().to_string(), error.to_string()), moved.as_slice()));
        }
    }
    let mut copies: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    for each in moved {
        copies.push(each.copy);
    }
    return Ok(copies);
}

/// What:
///  What stopped a save,
///  and every repository it had moved and could not put back.
/// Why:
///   A save that stops after moving something must either put it back or say where it is;
///  the caller builds the rejection from this,
///  and a repository named here is one the audit line names a place for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SaveStopped = { failure: SaveFailure; stillMoved: SavedCopy[] };
/// ```
#[derive(Debug)]
pub struct SaveStopped {
    /// What the system refused, in words a person can act on.
    pub failure: SaveFailure,
    /// The repositories a put-back could not return,
    ///  each with the place it is.
    pub still_moved: Vec<SavedCopy>,
}

/// What:
///  One repository this save moved,
///  with both ends of the move so the put-back addresses exactly the directories it moved.
/// Why:
///  The name a person reads is the path below the worktree,
///  which loses the bytes of a name that is not UTF-8;
///  renaming it back needs the exact source and destination.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Moved = { source: string; destination: string; copy: SavedCopy };
/// ```
struct Moved {
    /// Where the repository was below the worktree.
    source: PathBuf,
    /// Where this save put it.
    destination: PathBuf,
    /// What the notice and the audit line name.
    copy: SavedCopy,
}

/// What:
///  Stop the save with its refusal,
///  after putting back every repository it had already moved.
/// Why:
///  The refusal keeps the worktree as the command found it,
///  so the person can fix what the system refused and run the same command again;
///  a repository a put-back cannot return is the one exception, and it travels with its place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopped = (failure, moved) => ({ failure, stillMoved: putBack(moved) });
/// ```
fn stopped(failure: SaveFailure, moved: &[Moved]) -> SaveStopped {
    return SaveStopped { failure, still_moved: put_back(moved) };
}

/// What:
///  Move every repository of a stopped save back where the clean found it,
///  and answer with the ones that could not be returned.
/// Why:
///   The reverse of the moves undoes exactly what happened,
///  and the last move is undone first so a directory a later move made is empty when its parent returns.
///  The saved directories stay,
///  empty,
///  because the folder's own limits prune them and a rename back must not depend on removing them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putBack(moved: Moved[]): SavedCopy[] {
///   const still = [];
///   for (const each of [...moved].reverse()) {
///     try { renameSync(each.destination, each.source); } catch { still.push(each.copy); }
///   }
///   return still;
/// }
/// ```
fn put_back(moved: &[Moved]) -> Vec<SavedCopy> {
    let mut still: Vec<SavedCopy> = Vec::<SavedCopy>::new();
    for each in moved.iter().rev() {
        if std::fs::rename(each.destination.as_path(), each.source.as_path()).is_err() {
            still.push(each.copy.clone());
        }
    }
    return still;
}

/// What:
///  The system's refusal of one move in words a person can act on.
/// Why:
///   One refusal a person meets has a remedy of its own:
///  a repository on another file system cannot be renamed into the saved copies,
///  and the wrapper does not copy a whole repository.
///  Every other refusal keeps the system's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function describeRenameFailure(error: Error): string;
/// ```
fn describe_rename_failure(error: std::io::Error) -> String {
    return match error.raw_os_error() {
        Some(libc::EXDEV) => String::from(
            "it is on another file system than the place the wrapper keeps saved copies, and the wrapper does not copy a whole \
             repository",
        ),
        _ => error.to_string(),
    };
}

/// The move,
/// its refusals and the place it keeps.
#[cfg(test)]
#[path = "discarded_nested_repository_tests.rs"]
mod tests;
