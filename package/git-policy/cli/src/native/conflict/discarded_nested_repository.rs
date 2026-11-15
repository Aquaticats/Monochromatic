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

use crate::conflict_sides::{SAVED_DIRECTORY, SavedCopy, SaveFailure, depth_deepest_first, fresh_saved_directory};
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
    return set_nested_repositories_aside_waited_with(
        common_dir,
        worktree_root,
        repositories,
        milliseconds,
        &move_a_repository_within_a_ceiling,
    );
}

/// What:
///  One move of a repository into the saved copies, as the save waits for it:
///  the two places, and either the rename's own answer or nothing when the ceiling passed first.
///  `Option<std::io::Result<()>>` is "the answer, or none".
/// Why:
///  The move runs on a thread the save waits for only until the ceiling,
///  and a control cannot hold a real rename past that ceiling;
///  this is the seam it replaces,
///  so the control proves what the save does with a rename that gave no answer and then went
///  through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MoveAside = (from: string, to: string) => io.Result | undefined;
/// ```
pub type MoveAside = dyn Fn(&Path, &Path) -> Option<std::io::Result<()>>;

/// What:
///  The production move: a rename on a thread the save waits for only until the worktree lookup
///  ceiling.
/// Why:
///  A rename on a mount whose server stopped answering waits in the kernel with no end.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moveWithinACeiling = (from, to) => workWithin(WORKTREE_LOOKUP_CEILING, () => renameSync(from, to));
/// ```
fn move_a_repository_within_a_ceiling(from: &Path, to: &Path) -> Option<std::io::Result<()>> {
    let source: PathBuf = from.to_path_buf();
    let destination: PathBuf = to.to_path_buf();
    return crate::work_awaited_within_a_ceiling::work_within(crate::wait_ceilings::WORKTREE_LOOKUP_CEILING, move || {
        return std::fs::rename(source.as_path(), destination.as_path());
    });
}

/// What:
///  The same save with the move given, so a control can make one give no answer and then go through.
///  `&MoveAside` is the move to use for every repository.
/// Why:
///  The seventh review's finding is about a rename that was given up on and then completed:
///  the save must keep that move, the rollback must find the late success, and the rejection must say
///  the repository may be at either place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// setNestedRepositoriesAsideWaitedWith(commonDir, root, repositories, now, aMoveThatAnswersNothingAndThenMoves)
/// ```
pub(crate) fn set_nested_repositories_aside_waited_with(
    common_dir: &Path,
    worktree_root: &Path,
    repositories: &[Vec<u8>],
    milliseconds: i64,
    move_aside: &MoveAside,
) -> Result<Vec<SavedCopy>, SaveStopped> {
    // Creating the folder of saved copies is a storage step of its own, and a mount whose server stopped answering
    //  makes a `mkdir` wait in the kernel with no end, so it runs on a thread this save waits for only to the ceiling.
    let common: PathBuf = common_dir.to_path_buf();
    let base: PathBuf = match crate::work_awaited_within_a_ceiling::work_within(crate::wait_ceilings::WORKTREE_LOOKUP_CEILING, move || {
        return fresh_saved_directory(common.as_path(), milliseconds);
    }) {
        Some(Ok(created)) => created,
        Some(Err(error)) => {
            return Err(SaveStopped { failure: SaveFailure::io(common_dir.display().to_string(), error.to_string()), still_moved: Vec::<SavedCopy>::new(), unanswered: Vec::<SavedCopy>::new() })
        }
        // No answer is not a failed creation: the folder may exist now, so the save stops and names the place.
        None => {
            return Err(SaveStopped {
                failure: SaveFailure::io(
                    common_dir.join(SAVED_DIRECTORY).display().to_string(),
                    format!(
                        "creating the place the copies go in gave no answer within {} seconds",
                        crate::wait_ceilings::WORKTREE_LOOKUP_CEILING.as_secs()
                    ),
                ),
                still_moved: Vec::<SavedCopy>::new(),
                unanswered: Vec::<SavedCopy>::new(),
            })
        }
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
        // The directory a copy goes in is made on a thread this save waits for only to the ceiling, for the same reason
        //  the creation of the folder above is: a `mkdir` on a mount that stopped answering never returns.
        let make: PathBuf = parent.to_path_buf();
        match crate::work_awaited_within_a_ceiling::work_within(crate::wait_ceilings::WORKTREE_LOOKUP_CEILING, move || {
            return create_private_directory_all(make.as_path());
        }) {
            Some(Ok(())) => {}
            Some(Err(error)) => return Err(stopped(SaveFailure::io(shown, error.to_string()), moved.as_slice())),
            None => {
                return Err(stopped(
                    SaveFailure::io(
                        shown,
                        format!(
                            "the directory it is moved into gave no answer within {} seconds",
                            crate::wait_ceilings::WORKTREE_LOOKUP_CEILING.as_secs()
                        ),
                    ),
                    moved.as_slice(),
                ))
            }
        }
        written.insert(parent.to_path_buf());
        // A rename or a sync on a mount whose server stopped answering waits in the kernel with no end,
        //  so each runs on a thread this process waits for only until the read ceiling.
        let attempted: Moved = Moved {
            source: source.clone(),
            destination: destination.clone(),
            copy: SavedCopy { path: shown.clone(), saved_to: Some(destination.clone()) },
        };
        let Some(moved_here) = move_aside(source.as_path(), destination.as_path()) else {
            // No answer is not a failure: the rename may have gone through after the wait gave up on it, so the move
            //  is kept and the put-back returns a late success. The rejection says the move did not answer and where
            //  the repository may be.
            let mut maybe_moved: Vec<Moved> = moved.clone();
            maybe_moved.push(attempted);
            return Err(SaveStopped {
                failure: SaveFailure::io(
                    shown,
                    format!(
                        "moving it into the saved copies gave no answer within {} seconds, so the wrapper cannot tell whether it moved: it is at its own place in the worktree, or below the saved copies",
                        crate::wait_ceilings::WORKTREE_LOOKUP_CEILING.as_secs()
                    ),
                ),
                still_moved: put_back(maybe_moved.as_slice()),
                unanswered: vec![maybe_moved.last().expect("the move just added").copy.clone()],
            });
        };
        if let Err(error) = moved_here {
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
        if let Err(error) = synced_within_a_ceiling(each.as_path()) {
            return Err(stopped(SaveFailure::io(each.display().to_string(), error.to_string()), moved.as_slice()));
        }
    }
    let discarded: &Path = base.parent().unwrap_or(base.as_path());
    for each in [base.as_path(), discarded] {
        if let Err(error) = synced_within_a_ceiling(each) {
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
    /// What the system refused,
    /// in words a person can act on.
    pub failure: SaveFailure,
    /// The repositories a put-back could not return,
    ///  each with the place it is.
    pub still_moved: Vec<SavedCopy>,
    /// The repositories whose move gave no answer within its ceiling,
    ///  each with the place it may be:
    ///  a rename that was given up on is not known to have failed.
    pub unanswered: Vec<SavedCopy>,
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
#[derive(Clone)]
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
///  a repository a put-back cannot return is the one exception,
/// and it travels with its place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stopped = (failure, moved) => ({ failure, stillMoved: putBack(moved) });
/// ```
fn stopped(failure: SaveFailure, moved: &[Moved]) -> SaveStopped {
    return SaveStopped {
        failure,
        still_moved: put_back(moved),
        unanswered: Vec::<SavedCopy>::new(),
    };
}

/// What:
///  What came of putting one saved repository back.
///  An `enum` is a closed set of named alternatives.
/// Why:
///  Three answers mean three different things for the person:
///  the repository is back where it was,
///  it never left (a destination that is not there is a rename that never happened),
///  or the return gave no answer and it may be at either place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Back = 'returned' | 'never-moved' | 'not-answered';
/// ```
pub enum Back {
    /// The repository is where the worktree had it.
    Returned,
    /// Nothing stood below the saved copies, so the move never happened.
    NeverMoved,
    /// The return gave no answer within its ceiling.
    NotAnswered,
}

/// What:
///  One saved repository returned to the place the worktree had it,
///  for no longer than the worktree lookup ceiling.
///  `&Path` borrows the place the worktree had it and the place the copy is.
/// Why:
///  A rename on a mount whose server stopped answering waits in the kernel with no end,
///  so every return runs on a thread this process waits for only until the ceiling.
///  A destination that is not there is a move that never happened rather than a failure,
///  which is what a rename given up on and then completed elsewhere leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveBackWithinACeiling('/w/nested', '/r/.git/.../discarded/<stamp>/nested'): Back;
/// ```
pub fn move_back_within_a_ceiling(source: &Path, destination: &Path) -> Back {
    let to: PathBuf = source.to_path_buf();
    let from: PathBuf = destination.to_path_buf();
    let returned: Option<std::io::Result<()>> =
        crate::work_awaited_within_a_ceiling::work_within(crate::wait_ceilings::WORKTREE_LOOKUP_CEILING, move || {
            return std::fs::rename(from.as_path(), to.as_path());
        });
    return match returned {
        Some(Ok(())) => Back::Returned,
        Some(Err(error)) if error.kind() == std::io::ErrorKind::NotFound => Back::NeverMoved,
        Some(Err(_)) | None => Back::NotAnswered,
    };
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
        // A move that gave no answer may still have gone through, so the put-back tries every move it kept.
        //  A destination that is not there is a move that never happened: the repository is where it was, and
        //  nothing is kept below the saved copies for it.
        if matches!(move_back_within_a_ceiling(each.source.as_path(), each.destination.as_path()), Back::NotAnswered) {
            still.push(each.copy.clone());
        }
    }
    return still;
}

/// What:
///  One directory's entries flushed to the disk, for no longer than the read ceiling.
/// Why:
///  `sync_directory` opens the directory and syncs it, and a mount whose server stopped answering
///  waits in the kernel with no end;
///  a save that cannot prove the copies durable must stop and name the directory rather than hold
///  the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const synced = workWithin(WORKTREE_LOOKUP_CEILING, () => syncDirectory(directory)) ?? new Error('no answer');
/// ```
fn synced_within_a_ceiling(directory: &Path) -> std::io::Result<()> {
    let named: PathBuf = directory.to_path_buf();
    let Some(synced) = crate::work_awaited_within_a_ceiling::work_within(crate::wait_ceilings::WORKTREE_LOOKUP_CEILING, move || {
        return sync_directory(named.as_path());
    }) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!(
                "flushing it to the disk did not answer within {} seconds",
                crate::wait_ceilings::WORKTREE_LOOKUP_CEILING.as_secs()
            ),
        ));
    };
    return synced;
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
