//! What:
//!  Reading `Cargo.lock` before a `cargo clean` and putting it back as it was afterwards,
//! and keeping the bytes a clean displaced where they cannot be attributed to the clean.
//! Why:
//!  A clean may rewrite the lock file,
//! and a new worktree must stay as checked out;
//! the two steps need nothing from either clean,
//! so they live in a file of their own that both import:
//! `package_repair_cargo.rs`,
//! which cleans what the worktree already has,
//! and `package_repair_prewarm_job.rs`,
//! which cleans the copy the commit service fills a build directory from after the add returns.
//! The add's clean runs while the worktree it fills was made by the same command and nobody has it yet,
//! which `SPEC.md` records as the reason its put-back stands
//! ("A committed `Cargo.lock` that disagrees with its manifest"),
//! so it puts the checked-out lock back over whatever the clean left (`restore_lock`).
//! The filling runs after the add has returned and the person may be working in the worktree,
//! so it accepts the checked-out bytes back only over bytes its own resolution wrote
//! (`restore_only_the_cleans_own`),
//! and the clean it runs is the one given `--locked` (`package_repair_prewarm_job.rs`).
//!   The clean rewrites the lock because the committed one no longer matches its manifest,
//! and the worktree it runs in may be one another command is already working in,
//! so the bytes standing there when it ends can be Cargo's own rewrite
//!  or a command or person's newer write,
//! and nothing at that moment can tell the two apart.
//! A comparison of the file with the snapshot cannot say who wrote it,
//!  so the bytes that stand there are kept whole under the repository's Git directory
//!  before the checked-out ones are put back
//! (the owner's rule "never discard user data";
//! a review of `ee545033ad`,
//! finding 3,
//!  found the restore writing over a newer edit without a copy of it).
//! A lock the clean left unchanged is not written at all,
//!  and a lock whose displaced bytes cannot be kept is left exactly as it was found,
//! because putting the checked-out bytes back would then be the only place those bytes existed.
//! The reading that decided all this is a moment of its own,
//! and a person may write the lock after it:
//! a write made over the file by an earlier version was in neither the copy nor the file afterwards.
//! So the checked-out bytes are staged in the displaced directory and exchanged with the lock in one step
//! (`renameat2(RENAME_EXCHANGE)`,
//! `rename_that_never_replaces.rs`),
//! which hands back the file that stood there at that instant:
//! the file the reading named goes,
//! and any other is put back where the person wrote it while the checked-out bytes stay where the line names them
//! (`replace_the_clean_s_own`).
//! A lock the clean created is taken away by a move and put back if it was written since,
//! and one the clean removed is written with `create_new`,
//! so no step here replaces a file nobody kept.
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
pub const LOCK_FILE: &str = "Cargo.lock";

/// The directory,
/// under the worktree's Git directory,
/// that holds the bytes a clean displaced.
pub const DISPLACED_DIRECTORY: &str = "monochromatic-git-policy-cli-displaced-locks";

/// The part of a staged file's name that says it is the checked-out lock on its way into the worktree,
/// and not one of the copies a read displaced.
const STAGING_SUFFIX: &str = "restoring";

/// The most bytes of one `Cargo.lock` this module reads.
/// Why:
///   The lock is the person's own file,
/// and a read whose size somebody else decides is not a read this module takes:
/// a file past this size is not a lock any step here has to judge,
/// and refusing it costs one workspace's repair rather than a wait,
/// or a memory spike from a file somebody else sized.
///   Where the number comes from:
/// measured on this host on 2026-11-13,
/// the thirty-five committed locks of this repository are 78,968 bytes for this package,
/// 173,511 for `package/ide/app`,
/// 201,284 for `package/music-player/desktop-app`,
/// and no other above 200,000,
/// so `1 MiB` is more than five times the largest lock this repository commits
/// and leaves room for a workspace whose dependency graph a person grows by hand.
/// A lock past it is refused (`ReadRefusal::TooLarge`),
/// never truncated,
/// and the refusal is two answers for the caller:
/// the add gives that workspace's clean up before anything is written,
/// and the filling gives the workspace up,
/// so nothing places a resolution over a file it could not read and nothing removes one.
pub const LOCK_READ_LIMIT: u64 = 1 << 20;

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
pub fn read_lock(root: &Path) -> Result<Option<Vec<u8>>, String> {
    return read_the_lock_at(root.join(LOCK_FILE).as_path());
}

/// What:
///  The bytes of a lock file, read the way the wrapper reads a regular file:
/// a regular file only,
/// opened without following a link and without waiting for a writer,
/// and stopped at `LOCK_READ_LIMIT` bytes.
/// Why:
///   A lock may be read at the filling's start,
/// while the person works in the worktree,
///  and after an exchange has moved whatever stood at its name into a staging file.
/// A plain read of a named pipe at that name never returns,
/// and no ceiling could expire on the thread
/// (`SPEC.md`,
/// "Every wait has a bound");
/// the marker beside it and the repository's exclude file are read the same way.
/// A file that is not a regular file is a person's,
/// so the caller is told what stands there and touches nothing of it:
/// nothing is placed over it and nothing of it is removed.
///
/// # Errors
///
/// A lock that is not a regular file,
/// one larger than `LOCK_READ_LIMIT`,
/// and one the operating system refused to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readTheLockAt = (path) => readRegularFile(path, LOCK_READ_LIMIT) // Ok(bytesOrNone) | Err(whatStandsThere)
/// ```
fn read_the_lock_at(path: &Path) -> Result<Option<Vec<u8>>, String> {
    return match crate::private_storage::read_regular_file(path, LOCK_READ_LIMIT) {
        Ok(bytes) => Ok(Some(bytes)),
        // Nothing stands at the name, which is a state every step here has an answer for.
        Err(crate::private_storage::ReadRefusal::Missing) => Ok(None),
        Err(refusal) => Err(format!("{} {}, so nothing of it was read", path.display(), refusal_words(&refusal))),
    };
}

/// What:
///  Why a lock file could not be read,
/// in the words a person reads.
/// Why:
///   What stands at that name decides what to do about it,
/// and a pipe, a link, a directory and a file past the limit are four different things to fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusalWords = (refusal) => refusal.kind === 'not-regular' ? 'is not a regular file (a link, a directory or a named pipe)' : ...;
/// ```
fn refusal_words(refusal: &crate::private_storage::ReadRefusal) -> String {
    use crate::private_storage::ReadRefusal;
    return match refusal {
        ReadRefusal::Missing => String::from("is not there"),
        ReadRefusal::NotRegular => String::from("is not a regular file (a link, a directory or a named pipe)"),
        ReadRefusal::TooLarge => format!("holds more than {LOCK_READ_LIMIT} bytes"),
        ReadRefusal::Io(error) => format!("could not be read ({error})"),
    };
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
    return put_the_checkout_back(root, before, kept_in, None);
}

/// What:
///  Put `Cargo.lock` back as `before` read it only when the bytes standing there are the ones the clean's own
/// resolution of the workspace wrote,
/// and leave whatever else stands there exactly as it is found.
/// Why:
///   This face is the commit service's,
/// and it runs after the add returned,
/// when the person may be working in the worktree:
/// a write standing there when the clean ends may be theirs,
/// and replacing it would be the repair discarding what a person wrote.
/// `cargo clean` rewrites a lock that no longer matches its manifest,
/// so its caller asks Cargo for that resolution before the clean runs and gives the clean `--locked`,
/// which keeps the clean itself from writing the lock at all;
/// the bytes that resolution wrote are Cargo's,
/// so a file equal to them is the clean's own and the checked-out bytes may go back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = restoreOnlyTheCleansOwn({ root, before, keptIn, theCleansOwn }); // string | undefined
/// ```
pub fn restore_only_the_cleans_own(root: &Path, before: Option<&[u8]>, kept_in: Option<&Path>, the_cleans_own: &[u8]) -> Option<String> {
    return put_the_checkout_back(root, before, kept_in, Some(the_cleans_own));
}

/// What:
///  `restore_lock`'s work,
/// with the write it may displace judged first when the caller knows which bytes its own clean wrote.
/// Why:
///   Both faces keep the bytes they displace and put the checked-out lock back;
/// only the service can tell a clean's own write from a person's,
///  and that answer is read before anything is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putTheCheckoutBack({ root, before, keptIn, onlyWhen }): string | undefined;
/// ```
fn put_the_checkout_back(root: &Path, before: Option<&[u8]>, kept_in: Option<&Path>, only_when: Option<&[u8]>) -> Option<String> {
    let path: PathBuf = root.join(LOCK_FILE);
    let after: Option<Vec<u8>> = match read_the_lock_at(path.as_path()) {
        Ok(bytes) => bytes,
        // Something that is not a regular file stands there, so nothing of it is read, written over or removed.
        Err(refusal) => {
            return Some(format!("{refusal}, so the file is left exactly as it is found; `git diff -- {LOCK_FILE}` in that worktree shows it"));
        }
    };
    if after.as_deref() == before {
        return None;
    }
    // A chained condition keeps the two readings of one write together: whose bytes stand there, and whether they are ours.
    if let Some(the_cleans_own) = only_when
        && after.as_deref() != Some(the_cleans_own)
    {
        return Some(format!(
            "the {LOCK_FILE} of {} was written while the clean ran and does not hold what that clean's own resolution wrote, so it is left exactly as \
             it is found; `git diff -- {LOCK_FILE}` in that worktree shows it",
            path.display()
        ));
    }
    let kept: Option<PathBuf> = match (kept_in, after.as_deref()) {
        // Nothing stands there, so nothing is displaced and nothing has to be kept.
        (_no_place, None) => None,
        (None, Some(_bytes)) => {
            return Some(format!(
                "the {LOCK_FILE} of {} changed while the clean ran, and no Git directory was named to keep those bytes in, so the file is left \
                 exactly as it is found; `git diff -- {LOCK_FILE}` in that worktree shows what changed, and the branch holds the bytes it was \
                 checked out with",
                path.display()
            ));
        }
        (Some(directory), Some(bytes)) => match keep_the_bytes(directory, path.as_path(), bytes) {
            Ok(copy) => Some(copy),
            Err(reason) => {
                return Some(format!(
                    "the {LOCK_FILE} of {} changed while the clean ran, and the bytes that changed it could not be kept ({reason}), so the file \
                     is left exactly as it is found; `git diff -- {LOCK_FILE}` in that worktree shows what changed, and the branch holds the \
                     bytes it was checked out with",
                    path.display()
                ));
            }
        },
    };
    let restored: Restored = replace_the_clean_s_own(path.as_path(), before, after.as_deref(), kept_in);
    return match (restored, kept) {
        (Restored::TheirWriteStays(Some(kept_lock)), _) => Some(format!(
            "the {LOCK_FILE} of {} was written again while the checked-out lock was being put back, so the file is left exactly as it is found; \
             the checked-out lock is kept at {}, and `git diff -- {LOCK_FILE}` in that worktree shows the difference",
            root.display(),
            kept_lock.display()
        )),
        (Restored::TheirWriteStays(None), _) => Some(format!(
            "the {LOCK_FILE} of {} was written again while the checked-out lock was being put back, so the file is left exactly as it is found; the \
             branch holds the bytes it was checked out with, and `git diff -- {LOCK_FILE}` in that worktree shows the difference",
            root.display()
        )),
        (Restored::LeftAsFound(reason), _) => {
            // The step names what it could not do and what became of every byte, so this line adds only where to look.
            debug(TAG, format!("{} was not put back as checked out ({reason})", path.display()).as_str());
            Some(format!(
                "the {LOCK_FILE} of {} was not put back as checked out ({reason}); `git diff -- {LOCK_FILE}` in that worktree shows the file as it stands",
                root.display()
            ))
        }
        (Restored::TheCheckoutIsBack, Some(copy)) => Some(format!(
            "the {LOCK_FILE} of {} was not the checked-out file when the clean ended, so the bytes that stood there are kept at {}, and the file \
             is back as checked out",
            root.display(),
            copy.display()
        )),
        (Restored::TheCheckoutIsBack, None) => None,
    };
}


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
fn keep_the_bytes(kept_in: &Path, lock: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    let directory: PathBuf = copies_directory(kept_in, lock);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("{} could not be made ({error})", directory.display()));
    }
    // A lock displaced once already keeps its first copy and this one stands beside it: the first copy is the oldest statement of
    //  what a clean found, and replacing it would lose the bytes of the earlier repair.
    for step in 0.. {
        let name: String = if step == 0 { String::from(LOCK_FILE) } else { format!("{LOCK_FILE}.{step}") };
        let copy: PathBuf = directory.join(name);
        match write_exclusive_synced(copy.as_path(), bytes) {
            Ok(()) => {
                if let Err(error) = sync_directory(directory.as_path()) {
                    return Err(format!("{} could not be made durable ({error})", directory.display()));
                }
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

/// What:
///  What placing a resolution's bytes at a workspace's lock path came to.
/// Why:
///   The placement is the one step of the repair that writes the person's file,
///  so its answer says either that the write found exactly the bytes the filling read at its start,
///  or why nothing was written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placed = { kind: 'placed' } | { kind: 'gave-way'; reason: string };
/// ```
#[derive(Debug)]
pub enum Placed {
    /// The resolution's bytes stand at the lock path,
    /// and the bytes the exchange displaced are kept at this path until the caller's restore has put them back.
    ///
    /// They are kept even when they look like the checkout's own:
    /// `before` is whatever stood at the lock when the filling began,
    /// and the filling runs after the add returned,
    /// so a person may have written their own lock by then,
    /// and this step cannot tell their bytes from the checkout's.
    /// The copy is therefore kept until the lock holds those bytes again,
    /// so a caller that never gets that far leaves them on disk.
    Placed {
        /// Where the displaced bytes are kept,
        /// or nothing when the lock had no file for the resolution to displace.
        displaced: Option<PathBuf>,
    },
    /// Nothing was written there,
    /// because the steps that would prove what they displaced cannot run,
    /// or because the bytes there are a person's.
    GaveWay(String),
}

/// What:
///  Put a resolution's bytes at the workspace's `Cargo.lock` without replacing anything a person wrote,
///  and answer what became of the file that stood there.
/// Why:
///   The resolution is made in a mirror of the workspace, so the person's file is untouched until this step;
/// this is the only write to it,
/// and it is an exchange (`RENAME_EXCHANGE`),
///  which puts the resolution's bytes there and hands back the file that stood there in one step:
/// whatever a person wrote at any moment before it is returned here and not truncated,
///  which is the one thing a write-by-write comparison cannot promise.
/// The file handed back is compared with the bytes the filling read before its clean began:
/// they are the same when nobody wrote,
/// and anything else is a person's and is put back by the second exchange,
///  so the repair gives way and their file is exactly as they left it.
/// A lock the checkout does not have is created with `create_new`,
/// so a file a person created in the meantime is never written over.
///  A filesystem that cannot exchange two names gives way here rather than fall back to a plain write:
/// a plain write would replace a write made a moment earlier,
/// and giving the workspace up costs only the prewarm of it.
///
/// # Errors
///
/// `Placed::GaveWay` names what could not be done and what became of every byte.
pub fn place_the_resolution(root: &Path, resolution: &[u8], before: Option<&[u8]>, kept_in: Option<&Path>) -> Placed {
    let path: PathBuf = root.join(LOCK_FILE);
    let Some(directory) = kept_in else {
        return Placed::GaveWay(format!(
            "the {LOCK_FILE} of {} would have to be resolved, and no Git directory was named to place the resolution from",
            path.display()
        ));
    };
    let staged: PathBuf = match stage_the_checkout(directory, path.as_path(), resolution) {
        Ok(staged) => staged,
        Err(reason) => return Placed::GaveWay(reason),
    };
    let Some(standing) = before else {
        // Nothing stood at the lock path when the filling began, so the resolution is created there and never written over a file
        //  a person made meanwhile.
        return match crate::rename_that_never_replaces::rename_noreplace(staged.as_path(), path.as_path()) {
            Ok(()) => Placed::Placed { displaced: None },
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let _kept = std::fs::remove_file(staged.as_path());
                Placed::GaveWay(format!(
                    "the {LOCK_FILE} of {} was created while the repair resolved it, so it is left exactly as it is found",
                    path.display()
                ))
            }
            Err(error) => {
                let _kept = std::fs::remove_file(staged.as_path());
                Placed::GaveWay(format!("the {LOCK_FILE} of {} could not be written ({error})", path.display()))
            }
        };
    };
    if let Err(error) = crate::rename_that_never_replaces::exchange(staged.as_path(), path.as_path()) {
        if error.kind() == std::io::ErrorKind::NotFound {
            // The file went away since the filling read it: the resolution is created where nothing stands, and a person's new
            //  file is never written over.
            return match crate::rename_that_never_replaces::rename_noreplace(staged.as_path(), path.as_path()) {
                Ok(()) => Placed::Placed { displaced: None },
                Err(refusal) => {
                    let _kept = std::fs::remove_file(staged.as_path());
                    Placed::GaveWay(format!(
                        "the {LOCK_FILE} of {} was removed while the repair resolved it and could not be written back ({refusal}), so nothing was \
                         replaced and the resolution is at {}",
                        path.display(),
                        staged.display()
                    ))
                }
            };
        }
        let _kept = std::fs::remove_file(staged.as_path());
        return Placed::GaveWay(format!(
            "the {LOCK_FILE} of {} could not be replaced without losing a write ({error}), so the repair gives way and the file is exactly as it is \
             found",
            path.display()
        ));
    }
    // The exchange hands back the file that stood there, judged here and not from a reading taken earlier.
    let handed_back: Option<Vec<u8>> = match read_the_lock_at(staged.as_path()) {
        Ok(bytes) => bytes,
        // Something that is not a regular file stood at the lock's name, and the exchange moved it here. It is a person's, and it is
        //  not read, not written over and not removed: it goes back where it stood in one step, and the resolution with it.
        Err(refusal) => {
            return match crate::rename_that_never_replaces::exchange(staged.as_path(), path.as_path()) {
                Ok(()) => {
                    // The person's thing is back at the lock; the resolution stands at the staged name again, and it is this step's own.
                    let _dropped = std::fs::remove_file(staged.as_path());
                    Placed::GaveWay(format!("{refusal}, so it was put back where it stood and nothing was placed over it"))
                }
                Err(error) => Placed::GaveWay(format!(
                    "{refusal} and putting it back failed ({error}), so it is kept at {} and the resolution is at the file; nothing of it was removed",
                    staged.display()
                )),
            };
        }
    };
    if handed_back.as_deref() == Some(standing) {
        // What came out is the bytes the filling read at its start, and this step cannot tell whether they are the checkout's or a
        //  write of the person's made since the add returned: the branch holds the checkout's and nobody holds theirs. The copy
        //  stays until the caller's restore has put those bytes back at the lock, and a kill before that leaves them on disk.
        return Placed::Placed { displaced: Some(staged) };
    }
    #[cfg(test)]
    BETWEEN_THE_READING_AND_THE_ROLLBACK.with(|hook| {
        if let Some(write) = hook.borrow().as_ref() {
            write();
        }
    });
    // A person wrote while the resolution was made, so their bytes go back where they wrote them.
    if let Err(error) = crate::rename_that_never_replaces::exchange(staged.as_path(), path.as_path()) {
        return Placed::GaveWay(format!(
            "the {LOCK_FILE} of {} was written while the repair resolved it, and putting those bytes back failed ({error}), so they are kept at {}",
            path.display(),
            staged.display()
        ));
    }
    // Whatever that exchange displaced is judged, and only this step's own bytes are ever removed: the resolution is the one thing
    //  written here, and anything else is a write that landed after the rollback began.
    let displaced: Option<Vec<u8>> = match read_the_lock_at(staged.as_path()) {
        Ok(bytes) => bytes,
        Err(refusal) => {
            return Placed::GaveWay(format!(
                "{refusal}, which stood at the lock's name when the person's write was put back, so it is kept at {} and the repair gives way",
                staged.display()
            ));
        }
    };
    if displaced.as_deref() == Some(resolution) {
        let _dropped = std::fs::remove_file(staged.as_path());
        return Placed::GaveWay(format!(
            "the {LOCK_FILE} of {} was written while the repair resolved it, so it is left exactly as it is found and the repair gives way",
            path.display()
        ));
    }
    // A newer write landed while the person's earlier one was being put back. The lock holds the earlier write and the newer
    //  bytes are kept where the exchange left them, under a name this line gives; nothing of theirs was removed.
    return Placed::GaveWay(format!(
        "the {LOCK_FILE} of {} was written twice while the repair resolved it, so the earlier write is back at the file and the newer bytes are \
         kept at {}; `git diff -- {LOCK_FILE}` in that worktree shows the difference",
        path.display(),
        staged.display()
    ));
}

/// What:
///  What putting the checked-out lock back did with the bytes that stood there.
/// Why:
///   The reading that judged those bytes is a moment of its own,
/// and a person's write may land after it:
///  the answer says whether the lock is the checkout's again,
///  or their file was left where they wrote it and the checked-out bytes were kept instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Restored = { kind: 'the-checkout-is-back' } | { kind: 'their-write-stays'; kept?: string } | { kind: 'left-as-found'; reason: string };
/// ```
enum Restored {
    /// The lock holds the checked-out bytes again,
    /// or no file stands there because the checkout has none.
    TheCheckoutIsBack,
    /// A write landed after the reading,
    /// so that file is left where it is;
    /// the checked-out bytes are kept at this path when there was a place to keep them.
    TheirWriteStays(Option<PathBuf>),
    /// The step could not run,
    /// so nothing was written and the lock is exactly as it was found.
    LeftAsFound(String),
}

/// What:
///  Put the checked-out bytes back over the file `after` read,
///  in one step that hands back whatever file stood there instead,
///  and leave a write that landed after that reading exactly where it is.
/// Why:
///   A window between a last look and a write cannot be closed from outside the writer,
///  but it can be made to lose nothing:
///  the checked-out bytes are staged in the displaced directory and exchanged with the lock by
///  `renameat2(RENAME_EXCHANGE)` (`rename_that_never_replaces.rs`),
///  which leaves the file that stood there under the staged name.
///  When that file holds the bytes the reading named it is the clean's own, and it goes
///  (a copy of it is kept already);
///  when it holds anything else a person wrote it after the reading,
///  so the exchange is undone and their file is back where they wrote it while the checked-out bytes are kept where the notice says.
///  A filesystem that cannot exchange two names reads the lock once more and renames over it at once,
///  and the one window that leaves is named where it is (`replace_after_a_second_look`);
///  a displaced directory on another filesystem than the worktree cannot exchange at all,
///  and there the checked-out bytes are written as they were before this step existed.
///  A lock the checkout does not have is taken away by a move and put back if it was written since,
///  and one the clean removed is written with `create_new`,
///  so no step here replaces bytes nobody kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// replaceTheCleansOwn(path, before, after, keptIn) // never replaces an unkept file
/// ```
fn replace_the_clean_s_own(path: &Path, before: Option<&[u8]>, after: Option<&[u8]>, kept_in: Option<&Path>) -> Restored {
    #[cfg(test)]
    BETWEEN_THE_READING_AND_THE_RESTORE.with(|hook| { if let Some(pause) = hook.borrow().as_ref() { pause(); } });
    return match (before, after, kept_in) {
        (Some(checkout), Some(read), Some(directory)) => exchange_the_file_read(path, checkout, read, directory),
        // The clean created a lock the checkout has not, so the file goes, and a write that landed since keeps its file.
        (None, Some(read), Some(directory)) => take_the_file_away(path, read, directory),
        // The clean removed the lock the checkout has, so it is written where nothing stands.
        (Some(checkout), None, _) => write_where_nothing_stands(path, checkout),
        // Nothing stood there and the checkout has none: nothing to put back.
        (None, None, _) | (Some(_), Some(_), None) | (None, Some(_), None) => Restored::TheCheckoutIsBack,
    };
}

/// What:
///  Replace the lock with the checked-out bytes and judge the file the exchange handed back.
/// Why:
///  The exchange is the step that says what it displaced,
///  which a comparison and a rename cannot,
///  and the bytes it hands back are the ones to judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = stageTheCheckout(dir, lock, checkout); await exchange(staged, lock); return judge(staged, read);
/// ```
fn exchange_the_file_read(path: &Path, checkout: &[u8], read: &[u8], directory: &Path) -> Restored {
    let staged: PathBuf = match stage_the_checkout(directory, path, checkout) {
        Ok(staged) => staged,
        Err(reason) => return Restored::LeftAsFound(reason),
    };
    if let Err(error) = crate::rename_that_never_replaces::exchange(staged.as_path(), path) {
        let _dropped = std::fs::remove_file(staged.as_path());
        // A displaced directory on another filesystem than the worktree cannot exchange a file with the lock (`EXDEV`),
        //  which is what a linked worktree whose Git directory the main checkout holds does:
        // no step taken from here can reach across, so this is the plain write such a setup allows,
        //  and its one window is the one no step from outside the writer closes there.
        if is_cross_device(&error) {
            return match std::fs::write(path, checkout) {
                Ok(()) => Restored::TheCheckoutIsBack,
                Err(refusal) => Restored::LeftAsFound(format!("{} could not be written ({refusal})", path.display())),
            };
        }
        if !crate::rename_that_never_replaces::rename_unsupported(&error) {
            return Restored::LeftAsFound(format!("{} could not be replaced ({error})", path.display()));
        }
        return replace_after_a_second_look(path, checkout, read, directory);
    }
    return judge_the_displaced(path, staged.as_path(), read);
}

/// Whether a refused exchange says the two names are on different filesystems.
/// Why:
///   `EXDEV` is the answer a linked worktree whose Git directory the main checkout holds gives,
/// and it is the one refusal the caller answers differently (`rename_that_never_replaces.rs`,
/// `rename_unsupported`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.code === 'EXDEV'
/// ```
fn is_cross_device(error: &std::io::Error) -> bool {
    #[cfg(unix)]
    {
        return error.raw_os_error() == Some(libc::EXDEV);
    }
    #[cfg(not(unix))]
    {
        let _ = error;
        return false;
    }
}

/// What:
///  The same judgement over a plain rename,
///  for a filesystem that cannot exchange two names.
/// Why:
///  Such a filesystem leaves one window:
/// the lock is read once more and the rename follows at once,
/// and a write that lands exactly between those two steps is replaced unseen,
/// which no step taken from outside the writer can prevent there.
/// Every other write, before the reading or after the rename, is judged as the exchange judges it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (read(lock) !== read) return theirWriteStays(staged); await rename(staged, lock); return theCheckoutIsBack;
/// ```
fn replace_after_a_second_look(path: &Path, checkout: &[u8], read: &[u8], directory: &Path) -> Restored {
    // The exchange's staging file went, so the bytes are staged again for this step.
    let staged: PathBuf = match stage_the_checkout(directory, path, checkout) {
        Ok(staged) => staged,
        Err(reason) => return Restored::LeftAsFound(reason),
    };
    let standing: Option<Vec<u8>> = match read_the_lock_at(path) {
        Ok(bytes) => bytes,
        // What stands there is not a regular file, so it is a person's and it stays where it is.
        Err(_refusal) => return Restored::TheirWriteStays(Some(staged)),
    };
    if standing.as_deref() != Some(read) {
        // The bytes that stand there are not the ones the reading named, so they are a person's and stay.
        return Restored::TheirWriteStays(Some(staged));
    }
    if let Err(error) = std::fs::rename(staged.as_path(), path) {
        let _dropped = std::fs::remove_file(staged.as_path());
        return Restored::LeftAsFound(format!("{} could not be replaced ({error})", path.display()));
    }
    return Restored::TheCheckoutIsBack;
}

/// What:
///  Judge the file an exchange handed back:
///  the one the reading named goes,
///  and any other is put back where it was written,
///  with the checked-out bytes kept where it stood.
/// Why:
///  The file an exchange hands back is exactly what the lock held at the one step that replaced it,
/// so this answer is about the bytes that were there and not about bytes read earlier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (read(displaced) === read) { unlink(displaced); return theCheckoutIsBack(); } await exchange(displaced, lock); return theirWriteStays(displaced);
/// ```
fn judge_the_displaced(path: &Path, displaced: &Path, read: &[u8]) -> Restored {
    let handed_back: Option<Vec<u8>> = match read_the_lock_at(displaced) {
        Ok(bytes) => bytes,
        // It is not a regular file, so it is somebody's and it is kept where the exchange left it.
        Err(_refusal) => return Restored::TheirWriteStays(Some(displaced.to_path_buf())),
    };
    if handed_back.as_deref() == Some(read) {
        let _dropped = std::fs::remove_file(displaced);
        return Restored::TheCheckoutIsBack;
    }
    if let Err(error) = crate::rename_that_never_replaces::exchange(displaced, path) {
        // The exchange that would have put the write back failed, so the checked-out bytes are in place and the write is kept.
        return Restored::LeftAsFound(format!(
            "the bytes written after the reading could not be put back at {} ({error}), so they are kept at {} and the checked-out lock is in its place",
            path.display(),
            displaced.display()
        ));
    }
    return Restored::TheirWriteStays(Some(displaced.to_path_buf()));
}

/// What:
///  Take away the lock a clean created where the checkout has none,
///  and put it back if it holds bytes written after the reading.
/// Why:
///  A move is one step, so the file that goes is the file that stood there when the step ran,
///  whatever a person wrote after the reading;
///  the bytes are judged from the moved file and not from an earlier reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = freeName(dir, lock); rename(lock, moved); return judgeTheMovedFile(lock, moved, read);
/// ```
fn take_the_file_away(path: &Path, read: &[u8], directory: &Path) -> Restored {
    let moving: PathBuf = a_free_staging_name(directory, path);
    if let Err(error) = std::fs::rename(path, moving.as_path()) {
        if error.kind() == std::io::ErrorKind::NotFound {
            return Restored::TheCheckoutIsBack;
        }
        return Restored::LeftAsFound(format!("{} could not be moved aside ({error})", path.display()));
    }
    let moved: Option<Vec<u8>> = match read_the_lock_at(moving.as_path()) {
        Ok(bytes) => bytes,
        Err(refusal) => {
            return Restored::LeftAsFound(format!("{refusal}, so it is kept where the move left it, at {}", moving.display()));
        }
    };
    if moved.as_deref() == Some(read) {
        // What the clean resolved is kept already (`keep_the_bytes`), so this name goes.
        let _dropped = std::fs::remove_file(moving.as_path());
        return Restored::TheCheckoutIsBack;
    }
    if let Err(error) = std::fs::rename(moving.as_path(), path) {
        // The file was moved away and could not be put back, so it is kept where the move left it.
        return Restored::LeftAsFound(format!(
            "the bytes written after the reading could not be put back at {} ({error}), so they are kept at {}",
            path.display(),
            moving.display()
        ));
    }
    return Restored::TheirWriteStays(None);
}

/// What:
///  Write the checked-out bytes where no file stands,
///  and leave a file that appeared after the reading where it is.
/// Why:
///  `create_new` is one step that never replaces a file,
/// so a lock a person created after the reading is not written over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openSync(lock, 'wx'); writeSync(file, checkout); closeSync(file);
/// ```
fn write_where_nothing_stands(path: &Path, checkout: &[u8]) -> Restored {
    let opened: std::io::Result<std::fs::File> = std::fs::OpenOptions::new().write(true).create_new(true).open(path);
    let mut file: std::fs::File = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Restored::TheirWriteStays(None),
        Err(error) => return Restored::LeftAsFound(format!("{} could not be written ({error})", path.display())),
    };
    if let Err(error) = std::io::Write::write_all(&mut file, checkout) {
        drop(file);
        let _dropped = std::fs::remove_file(path);
        return Restored::LeftAsFound(format!("{} could not be written ({error})", path.display()));
    }
    return Restored::TheCheckoutIsBack;
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
fn stage_the_checkout(directory: &Path, lock: &Path, checkout: &[u8]) -> Result<PathBuf, String> {
    let copies: PathBuf = copies_directory(directory, lock);
    if let Err(error) = create_private_directory_all(copies.as_path()) {
        return Err(format!("{} could not be made ({error})", copies.display()));
    }
    let staged: PathBuf = a_free_staging_name(directory, lock);
    if let Err(error) = write_exclusive_synced(staged.as_path(), checkout) {
        return Err(format!("{} could not be written ({error})", staged.display()));
    }
    return Ok(staged);
}

/// What:
///  A name in the copies directory that no file holds,
///  for bytes a step stages there before they take the lock's place.
/// Why:
///   The names `Cargo.lock`, `Cargo.lock.1`, ... belong to the copies a reading displaced,
/// and a step that staged its bytes under one of them would hide a copy a person is told about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let step = 0; ; step += 1) { const name = step === 0 ? 'Cargo.lock.restoring' : `Cargo.lock.restoring.${step}`; if (!exists(join(dir, name))) return join(dir, name); }
/// ```
fn a_free_staging_name(directory: &Path, lock: &Path) -> PathBuf {
    let copies: PathBuf = copies_directory(directory, lock);
    for step in 0.. {
        let name: String = if step == 0 { format!("{LOCK_FILE}.{STAGING_SUFFIX}") } else { format!("{LOCK_FILE}.{STAGING_SUFFIX}.{step}") };
        let candidate: PathBuf = copies.join(name);
        if std::fs::symlink_metadata(candidate.as_path()).is_err() {
            return candidate;
        }
    }
    return copies.join(format!("{LOCK_FILE}.{STAGING_SUFFIX}"));
}

#[cfg(test)]
thread_local! {
    /// A control may stop the restore between the reading that judged the lock and the step that puts the checked-out bytes back,
    /// which is where a person's write lands.
    /// Why:
    ///   The window cannot be reached from outside the writer on a wall clock,
    /// and a control that waited on one would pass whatever the code did.
    pub(crate) static BETWEEN_THE_READING_AND_THE_RESTORE: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };

    /// A control may write the lock between the reading of what the first exchange displaced and the rollback that
    /// puts a person's write back, which is the window a save of theirs can land in.
    /// Why:
    ///   The window cannot be reached from outside the writer on a wall clock,
    /// and a control that waited on one would pass whatever the code did.
    pub(crate) static BETWEEN_THE_READING_AND_THE_ROLLBACK: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_lock_tests.rs"]
mod tests;
