//! What:
//! Removing one entry of a checkout only when it is the entry that was judged,
//! the identity a judgement keeps,
//! and the sentence for a file that could not be put back.
//!
//! Why:
//! A name is not its entry.
//! Between the look that judged an entry and the step that acts on its name,
//! another command can move a file of its own onto that name,
//! and an unlink of the name then removes a file this call never judged
//! (two reviews reproduced that, one of them losing the other command's file, and one losing an edit in place whose
//! writer put the modification time back).
//! So every step below that removes or replaces something is one of two things:
//! an operation that fails when what it needs is not true,
//! or an act on a name only this process can name.
//!
//! The steps, each with what must be true of that very entry at that very instruction,
//! how the step knows it,
//! and what a kill after it leaves:
//!
//! 1. **Judge**, once, before anything moves (`judge`).
//!    What must be true: the entry is an ordinary file or a symbolic link,
//!    and what is kept is enough to tell it from anything that takes its place.
//!    How: `symlink_metadata`, which never waits,
//!    and one read of its bytes through an open that never waits (`super::ordinary_file`).
//!    What is kept is device and inode, size, modification time to the nanosecond,
//!    and a SHA-256 of the bytes.
//!    Size and time alone miss an edit in place whose writer put the modification time back;
//!    the content catches it.
//!    The change time is deliberately not kept: the move itself bumps it on the inode.
//!    A named pipe, a directory, a socket or a device is not judged,
//!    so this module removes none of them.
//!    *A kill leaves nothing: nothing has been named or changed.*
//!
//! 2. **Choose a hidden name** (`temporary_name_beside`).
//!    What must be true: nothing yet; this step creates nothing and so can replace nothing.
//!    *A kill leaves nothing.*
//!
//! 3. **Move the entry at the path to that name with one no-replace rename** (`move_without_replacing`).
//!    What must be true: the hidden name is free,
//!    and the act takes the entry that stands at the path at that instruction, whatever it is.
//!    How: `renameat2` with `RENAME_NOREPLACE` on Linux and `renamex_np` with `RENAME_EXCL` on macOS,
//!    which fails with `EEXIST` when something took the name in between and never writes over it,
//!    and with `ENOENT` when nothing stands at the path.
//!    The step does not decide which entry it took: it takes whatever stood there,
//!    and step 4 judges what it took,
//!    so the decision is made about the entry in hand and never about a name.
//!    A file system or platform without that operation leaves the entry where it is, with a sentence:
//!    an undeclared launcher left one install longer costs nothing,
//!    and a file removed that was not the one judged is a loss.
//!    *A kill leaves the entry under the hidden name only, and the visible path free;
//!    the repair pass (`repair_what_a_kill_left` in `stale_launchers.rs`) moves it back with one rename, in one pass.*
//!
//! 4. **Compare what the hidden name holds with what was judged.**
//!    What must be true: every field of the judgement, content included, is the one recorded in step 1.
//!    How: `judge` again on the entry in hand, and equality.
//!    A difference means the caller's lock now declares the name, or another command's entry was moved aside:
//!    the entry is put back with another no-replace rename, and kept when something newer took the path.
//!    This step runs after step 3, so it describes the entry this call holds and nobody else can name;
//!    a replacement another command put at the visible path after step 3 is not touched at all.
//!    *A kill inside this compare leaves it hidden; the repair pass moves it back.*
//!
//! 5. **Ask `may_remove`** (the caller's question, asked of the lock on disk at this instant).
//!    What must be true: the entry may go now. No keeps it, put back with the same no-replace rename.
//!    *A kill here leaves it hidden; the repair pass moves it back, which is the old state,
//!    and the next pass judges it again.*
//!
//! 6. **Remove the hidden name** (`drop_aside`).
//!    What must be true: that name is the one this process made in step 3, and it still holds the entry moved there.
//!    How: the name carries this process's number, no other run's repair pass touches a live maker's name,
//!    and nothing else writes a name this process chose,
//!    so this unlink is an act on a name only this process can name.
//!    It is never an unlink of a shared path that another command may have replaced.
//!    *A kill inside this removal leaves the hidden name;
//!    the repair pass moves it back and the next pass judges it again.*
//!
//! The put-back is the same one operation in the other direction (`put_back`):
//! a no-replace rename, which fails with `EEXIST` instead of writing over whatever took the path,
//! and then leaves the moved file hidden with the sentence that names it.
//! Nothing in these steps replaces a file:
//! not the free-name choice (it creates nothing),
//! not the move (it fails when the name is taken),
//! not the put-back (a no-replace move again),
//! and the hidden name a person's file is kept at is named in the sentence that reports it,
//! together with the command that moves it back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // removal.ts
//! export type Judged = { device: number; inode: number; size: number; modified: bigint; content: Uint8Array };
//! export type Removal = 'retired' | 'gone' | 'replaced' | 'kept';
//! export function removeIfStill(path: string, judged: Judged, mayRemove: () => boolean): Removal  // throws
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::shell_word;
/// Imports the one way this tool opens a file it did not just create.
use super::ordinary_file::{sha256_of_bytes, sha256_of_ordinary};

/// How many hidden names beside an entry are tried before a removal refuses to take it aside.
/// Delegated pick under the owner's rule that nothing waits without end and never discards a person's file;
/// each name is one earlier run's kept file, and a folder with every one of them taken is not one to guess in.
const TEMPORARY_NAME_ATTEMPTS: usize = 64;

/// What:
/// The entry at a path when it was judged:
/// the file it was, by everything that tells it from anything that takes its place.
/// Why:
/// Inode identity alone misses a truncate-and-write in the same inode;
/// the size and the modification time move when any byte does,
/// and the content catches the writer that puts the modification time back after an edit in place.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Judged {
    /// The device of the file system the entry is on.
    #[cfg(unix)]
    device: u64,
    /// The entry's inode on that file system.
    #[cfg(unix)]
    inode: u64,
    /// Its size in bytes.
    size: u64,
    /// When its bytes last changed, in nanoseconds since the epoch.
    /// The change time is deliberately not part of this: the move itself bumps it on the inode,
    /// so a compare that included it would take every entry for a changed one.
    modified: i128,
    /// SHA-256 of its bytes: an ordinary file's content, or a symbolic link's stated target.
    content: [u8; 32],
}

/// What a take-aside left.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Removal {
    /// This call removed the entry that was judged.
    Retired,
    /// Nothing stood at the path any more.
    Gone,
    /// Something that was not judged stood there: it was put back and left alone.
    Replaced,
    /// The entry was the one judged and was put back because the caller said to keep it.
    Kept,
}

/// What a take-aside gave us.
pub(crate) enum Taken {
    /// The entry is at this hidden name and is the one judged; it is also gone from its visible name.
    Aside(PathBuf),
    /// Nothing stood at the path.
    Gone,
    /// Something else stood there: it was put back.
    Replaced,
}

/// The nanoseconds since the epoch of a time, so two of them compare as one number.
fn nanoseconds(time: std::io::Result<std::time::SystemTime>) -> i128 {
    return match time {
        Ok(since) => match since.duration_since(std::time::UNIX_EPOCH) {
            Ok(after) => i128::try_from(after.as_nanos()).unwrap_or(i128::MAX),
            Err(before) => -i128::try_from(before.duration().as_nanos()).unwrap_or(i128::MAX),
        },
        Err(_) => 0,
    };
}

/// The identity of an entry from its metadata,
/// with the digest of the bytes that were read from it.
fn judged_of_metadata(metadata: &fs::Metadata, content: [u8; 32]) -> Judged {
    /// Imports `MetadataExt` from `std::os::unix::fs`.
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    #[cfg(unix)]
    let (device, inode) = (metadata.dev(), metadata.ino());
    #[cfg(not(unix))]
    let (device, inode) = (0, 0);
    return Judged {
        #[cfg(unix)]
        device,
        #[cfg(unix)]
        inode,
        size: metadata.len(),
        modified: nanoseconds(metadata.modified()),
        content,
    };
}

/// What:
/// The judgement of the entry at `path`: what it is and what its bytes are,
/// or the sentence naming what is wrong with it.
/// Why:
/// This is step 1 of the steps at the top, and the same call makes step 4's comparison,
/// so what a removal compares is always one thing: two judgements.
/// A symbolic link is judged by its stated target,
/// which is read without opening anything;
/// an ordinary file by its content,
/// through an open that never waits;
/// anything else is not judged, so this module never removes it.
///
/// @throws The sentence naming why the entry cannot be judged, which leaves it where it is.
pub(crate) fn judged_of(path: &Path) -> Result<Judged, String> {
    let metadata: fs::Metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return Err(format!("{} cannot be inspected: {error}", path.display())),
    };
    let content: [u8; 32] = if metadata.file_type().is_symlink() {
        let stated: PathBuf = match fs::read_link(path) {
            Ok(stated) => stated,
            Err(error) => return Err(format!("{} cannot be read as a link: {error}", path.display())),
        };
        sha256_of_bytes(stated.as_os_str().as_encoded_bytes())
    } else if metadata.is_file() {
        sha256_of_ordinary(path)?
    } else {
        return Err(super::ordinary_file::not_ordinary(path, &metadata));
    };
    return Ok(judged_of_metadata(&metadata, content));
}

/// What:
/// The judgement of the file an open descriptor names,
/// from the bytes that were read through that same descriptor.
/// Why:
/// A record is read through a descriptor this tool opened,
/// and the proof is about what that descriptor holds:
/// judging the path again would describe an entry that may already be another file.
///
/// @throws The sentence naming why the entry cannot be judged, which leaves it where it is.
pub(crate) fn judged_of_open(file: &fs::File, content: &[u8]) -> Result<Judged, String> {
    let metadata: fs::Metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => return Err(format!("the open record cannot be inspected: {error}")),
    };
    if !metadata.is_file() {
        return Err(super::ordinary_file::not_ordinary(Path::new("the open record"), &metadata));
    }
    return Ok(judged_of_metadata(&metadata, sha256_of_bytes(content)));
}

/// What:
/// A hidden name beside `path` for the one no-replace step of a removal,
/// or the reason every name of the bound is taken.
/// Why:
/// The name is only *chosen* here, never created:
/// the no-replace move that follows fails when something took the name in between,
/// and then the next name is tried,
/// so a person's file at a hidden name is never written over.
fn temporary_name_beside(path: &Path, taken: usize) -> Result<PathBuf, String> {
    let name: String = path.file_name().map(|name| return name.to_string_lossy().to_string()).unwrap_or_default();
    let attempt: usize = taken;
    if attempt >= TEMPORARY_NAME_ATTEMPTS {
        return Err(format!(
            "every hidden temporary name beside it (from 0 to {}) is taken by a file this tool kept there, so nothing was moved \
             aside and nothing removed",
            TEMPORARY_NAME_ATTEMPTS - 1
        ));
    }
    return Ok(path.with_file_name(format!(".{name}.{}.{attempt}.retired", std::process::id())));
}

/// What:
/// Whether the entry just judged is the one that was judged before.
/// Why:
/// This is step 4's whole decision, and every field of the judgement is in it:
/// device and inode say it is the same file,
/// size and modification time say it has not been rewritten,
/// and the content says it even when a writer put the modification time back.
/// The change time is not compared, because the move bumps it on the inode.
fn is_the_judged_entry(now: &Judged, judged: &Judged) -> bool {
    #[cfg(unix)]
    let same_identity: bool = now.device == judged.device && now.inode == judged.inode;
    #[cfg(not(unix))]
    let same_identity: bool = true;
    return same_identity && now.size == judged.size && now.modified == judged.modified && now.content == judged.content;
}

/// What:
/// Moves the entry at `from` to `to`, replacing nothing.
/// Why:
/// This is the one act of the removal that takes an entry off a shared name,
/// and it must never write over a name another command made.
/// `renameat2` with `RENAME_NOREPLACE` (`renamex_np` with `RENAME_EXCL` on macOS) is the one operation that moves whatever
/// stands at one name to another and fails when the other name is taken,
/// so between the two names there is no instant in which a third command's entry can be lost.
/// A platform or file system without it answers `Unsupported` or `InvalidInput`,
/// and the caller then leaves the entry where it is with a sentence.
fn move_without_replacing(from: &Path, to: &Path) -> std::io::Result<()> {
    // The one call that carries the flag, from `rustix`.
    #[cfg(any(target_os = "linux", target_vendor = "apple", target_os = "redox"))]
    {
        return rustix::fs::renameat_with(rustix::fs::CWD, from, rustix::fs::CWD, to, rustix::fs::RenameFlags::NOREPLACE)
            .map_err(std::io::Error::from);
    }
    #[cfg(not(any(target_os = "linux", target_vendor = "apple", target_os = "redox")))]
    {
        let _ = (from, to);
        return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "this platform has no rename that replaces nothing"));
    }
}

/// What:
/// Whether an error says that this platform or file system has no move that replaces nothing.
/// Why:
/// `EINVAL` and `ENOSYS` mean the flag is not implemented here,
/// and that is a different thing from a move that failed on one entry:
/// the first leaves every entry where it is, and the sentence has to say so.
fn no_such_operation(error: &std::io::Error) -> bool {
    return matches!(error.kind(), std::io::ErrorKind::Unsupported | std::io::ErrorKind::InvalidInput);
}

/// What:
/// Whether two names are names of one entry (the same device and inode).
/// Why:
/// A run killed between a hard link and the removal of the other name left one entry under two names,
/// as have older versions of this tool that moved entries aside with a link and an unlink;
/// the repair pass tells that state from a name something newer took by asking this.
#[cfg(unix)]
pub(crate) fn same_entry(first: &Path, second: &Path) -> bool {
    /// Imports `MetadataExt` from `std::os::unix::fs`.
    use std::os::unix::fs::MetadataExt;
    return match (fs::symlink_metadata(first), fs::symlink_metadata(second)) {
        (Ok(one), Ok(two)) => one.dev() == two.dev() && one.ino() == two.ino(),
        _ => false,
    };
}

/// Whether two names are names of one entry, on a platform that cannot say;
/// `false` keeps both, which is the side that removes nothing.
#[cfg(not(unix))]
pub(crate) fn same_entry(_first: &Path, _second: &Path) -> bool {
    return false;
}

/// What:
/// Takes the entry at `path` aside under a hidden name with one no-replace move,
/// and says whether the entry it took is the one that was judged.
/// Why:
/// The move is step 3 of the steps at the top,
/// and the comparison is step 4:
/// the move takes whatever stands at the path, so the decision is made about the entry in hand;
/// a judgement that differs in any field, content included, is put back rather than removed.
/// The hidden name this call chooses is the only name it writes,
/// and it writes it with an operation that fails instead of replacing.
pub(crate) fn take_aside(path: &Path, judged: Judged) -> Result<Taken, String> {
    // An entry this tool never removes (a directory, a named pipe, a socket, a device) is left before anything moves:
    // moving it aside and finding that out would put a person's directory at a hidden name if the visible name were
    // then taken. This look decides only whether to try, never whether a removal succeeded.
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() || metadata.file_type().is_symlink() => {}
        Ok(metadata) => return Err(super::ordinary_file::not_ordinary(path, &metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Taken::Gone),
        Err(error) => return Err(format!("{} cannot be inspected: {error}", path.display())),
    }
    let mut attempt: usize = 0;
    loop {
        let temporary: PathBuf = temporary_name_beside(path, attempt)?;
        #[cfg(test)]
        seam::before_the_move();
        match move_without_replacing(path, &temporary) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Taken::Gone),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // Something took the hidden name in between: nothing was replaced, and the next name is tried.
                attempt += 1;
                continue;
            }
            Err(error) if no_such_operation(&error) => {
                return Err(format!(
                    "this platform or file system has no way to move an entry to another name without replacing what stands \
                     there, so {} was left where it is and nothing was removed ({error})",
                    path.display()
                ));
            }
            Err(error) => {
                return Err(format!("it cannot be moved aside to a hidden name ({error}), so nothing was removed"));
            }
        }
        #[cfg(test)]
        seam::after_the_move();
        // The entry is at the hidden name this call made, and nobody else can name it.
        let now: Option<Judged> = judged_of(&temporary).ok();
        if !now.is_some_and(|current| return is_the_judged_entry(&current, &judged)) {
            return match put_back(path, &temporary) {
                Ok(()) => Ok(Taken::Replaced),
                Err(reason) => Err(reason),
            };
        }
        return Ok(Taken::Aside(temporary));
    }
}

/// What:
/// The sentence for a file that was moved aside and could not be put back because something newer took the path.
/// Why:
/// Putting it back would overwrite what is there now, and both files are kept only if one of them stays where it is:
/// the moved file is kept at the hidden name it was given,
/// and nothing removes it,
/// so the sentence has to say where it is and who can move it (the person it belongs to).
pub(crate) fn kept_aside(path: &Path, temporary: &Path) -> String {
    return format!(
        "something newer was placed at {} while what stood there was moved aside, so that file was not put back and nothing was \
         deleted: it is kept at {} and nothing removes it; `mv -- {} <where it belongs>` moves it",
        path.display(),
        temporary.display(),
        shell_word(&temporary.to_string_lossy())
    );
}

/// The sentence for a file that was moved aside and cannot be put back for another reason.
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
/// Puts the entry at `temporary` back at `path`,
/// with one no-replace move, and only if nothing stands at `path` now.
/// Why:
/// The move is the atomic "make this name only if it is free": it fails with `EEXIST` instead of writing over whatever
/// took the path, and the moved file then stays at its hidden name for the person to move.
/// A hard link and an unlink would leave an instant in which the entry has both names,
/// and a kill in that instant left a duplicate that no later pass could tell from a name something newer took.
/// A directory is moved back the same way; a file system that cannot move it leaves it and says where.
pub(crate) fn put_back(path: &Path, temporary: &Path) -> Result<(), String> {
    return match move_without_replacing(temporary, path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(kept_aside(path, temporary)),
        Err(error) if no_such_operation(&error) => Err(format!(
            "this platform or file system has no way to move it back without replacing what stands there, so it stays at {} and \
             nothing removes it ({error}); `mv -- {} <where it belongs>` moves it",
            temporary.display(),
            shell_word(&temporary.to_string_lossy())
        )),
        Err(error) => Err(could_not_put_back(path, temporary, &error)),
    };
}

/// Deletes the entry taken aside, and names where it is when it cannot be.
/// Why:
/// This is step 6: it acts on the hidden name this call made,
/// which carries this process's number and which no other run writes,
/// so the unlink removes exactly the entry moved aside and never a name another command took.
pub(crate) fn drop_aside(temporary: &Path) -> Result<(), String> {
    return match fs::remove_file(temporary) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!(
            "it was moved aside as the entry judged and then could not be removed: {error}; it is at {}",
            temporary.display()
        )),
    };
}

/// What:
/// Removes `path` only when the entry standing there is still the one judged and `may_remove` says it may go.
/// Why:
/// The whole removal is `take_aside` (steps 1 to 5) and then either a put-back or the removal of the hidden name,
/// with `may_remove` asked in between: that is the instant the lock on disk is read again,
/// and a lock that cannot be read then keeps the entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function removeIfStill(path: string, judged: Judged, mayRemove: () => boolean): Removal  // throws
/// ```
pub(crate) fn remove_if_still(path: &Path, judged: Judged, may_remove: &dyn Fn() -> bool) -> Result<Removal, String> {
    let temporary: PathBuf = match take_aside(path, judged)? {
        Taken::Aside(temporary) => temporary,
        Taken::Gone => return Ok(Removal::Gone),
        Taken::Replaced => return Ok(Removal::Replaced),
    };
    if !may_remove() {
        return match put_back(path, &temporary) {
            Ok(()) => Ok(Removal::Kept),
            Err(reason) => Err(reason),
        };
    }
    return match drop_aside(&temporary) {
        Ok(()) => Ok(Removal::Retired),
        Err(reason) => Err(reason),
    };
}

/// What:
/// The name a hidden `.⟨name⟩.⟨pid⟩.⟨attempt⟩.retired` entry was taken from,
/// or `None` for any other name.
/// Why:
/// The repair pass finds these after a kill and puts them back where they came from,
/// and it asks the number in the name whether the run that made it is gone.
pub(crate) fn retired_from(name: &str) -> Option<String> {
    let parts: Vec<&str> = name.split('.').collect();
    // ".entry.4242.0.retired" is ["", "entry", "4242", "0", "retired"].
    if parts.first() != Some(&"") || parts.last() != Some(&"retired") || parts.len() < 5 {
        return None;
    }
    let _attempt: u32 = parts[parts.len() - 2].parse().ok()?;
    let _pid: u32 = parts[parts.len() - 3].parse().ok()?;
    let middle: &[&str] = &parts[1..parts.len() - 3];
    if middle.is_empty() {
        return None;
    }
    return Some(middle.join("."));
}

/// The process number a hidden `.⟨name⟩.⟨pid⟩.⟨attempt⟩.retired` name carries.
pub(crate) fn maker_of_retired(name: &str) -> Option<u32> {
    let parts: Vec<&str> = name.split('.').collect();
    if parts.last() != Some(&"retired") || parts.len() < 5 {
        return None;
    }
    return parts[parts.len() - 3].parse().ok();
}

/// What:
/// The two seams a control uses to place a second actor's write at the steps of a removal,
/// compiled in tests only, so no shipped path can be moved by anything but the file system.
/// Why:
/// One control places a file at the hidden name before the move, which must not replace it;
/// another places a replacement at the visible path after the move, which must not be unlinked.
/// Each hook is taken when it runs, so a control that places its second actor once does so once.
#[cfg(test)]
pub(crate) mod seam {
    /// Imports `Mutex` from `std`.
    use std::sync::Mutex;

    /// The hook run between the name choice and the move, when a test set one.
    static BEFORE_THE_MOVE: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

    /// The hook run after the move and before the comparison, when a test set one.
    static AFTER_THE_MOVE: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

    /// Sets (or with `None` clears) the hook run between the name choice and the move.
    pub(crate) fn set_before_the_move(hook: Option<Box<dyn Fn() + Send>>) {
        *BEFORE_THE_MOVE.lock().expect("the seam is not poisoned") = hook;
    }

    /// Runs the hook before the move, when there is one.
    pub(crate) fn before_the_move() {
        if let Some(hook) = BEFORE_THE_MOVE.lock().expect("the seam is not poisoned").as_ref() {
            hook();
        }
    }

    /// Sets (or with `None` clears) the hook run after the move and before the comparison.
    pub(crate) fn set_after_the_move(hook: Option<Box<dyn Fn() + Send>>) {
        *AFTER_THE_MOVE.lock().expect("the seam is not poisoned") = hook;
    }

    /// Runs the hook after the move once, when there is one.
    pub(crate) fn after_the_move() {
        let waiting = AFTER_THE_MOVE.lock().expect("the seam is not poisoned").take();
        if let Some(hook) = waiting {
            hook();
        }
    }
}
