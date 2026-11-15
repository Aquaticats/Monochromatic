//! What:
//! Removing one entry of a checkout only when it is still the entry that was judged,
//! the identity a judgement keeps,
//! and the sentence for a file that could not be put back.
//! The steps, each and what it proves and what a kill after it leaves:
//!
//! 1. **Judge.** `judged_of_metadata` keeps what the entry is: its device and inode, its size, its modification time and
//!    its change time, to the nanosecond. An edit in place keeps the inode and changes the size and both times, so the
//!    compare below sees it.
//! 2. **Find a free hidden name, replacing nothing.** `temporary_name_beside` tries names in order and *creates* nothing;
//!    the step after it is the one that must not replace, and it fails when the name was taken in between.
//! 3. **Link the entry to that name.** `fs::hard_link` is the no-replace creation every platform this tool supports has:
//!    Linux and macOS have `link(2)` (and a no-replace rename, `renameat2` with `RENAME_NOREPLACE` and `renamex_np` with
//!    `RENAME_EXCL`, which the standard library does not expose), Windows has `CreateHardLinkW`, which also fails when the
//!    name exists. A filesystem without hard links answers `EPERM` and the entry is left, with a sentence. A symlink is
//!    linked as a symlink (`linkat` does not follow it).
//!    A kill after this leaves the entry under both names: nothing is lost, and the visible name is still the old state.
//! 4. **Unlink the visible name.** The entry is now only under the hidden name.
//!    A kill after this leaves the entry hidden and the visible path free: the repair pass (`repair_what_a_kill_left` in
//!    `stale_launchers.rs`) puts it back at its own name, which is the old state.
//! 5. **Compare what the hidden name holds with what was judged.** Different bytes, a different size or a later time mean
//!    something changed after the judgement, and the entry is put back (never over something newer) and kept.
//!    A kill inside this compare leaves it hidden, and the repair pass puts it back.
//! 6. **Ask `may_remove`** (the caller's question, asked of the lock on disk at this instant). No keeps it, put back.
//!    A kill here or after step 5 leaves it hidden, and the repair pass puts it back.
//! 7. **Remove the hidden name.** The entry is gone.
//!    A kill inside this removal leaves the hidden name, which the repair pass puts back (the old state) and the next
//!    pass judges again.
//!
//! Nothing in these steps replaces a file: not the free-name choice (it creates nothing), not the link (it fails when
//! the name exists), not the put-back (a hard link again), and the hidden name a person's file is kept at is named in the
//! sentence that reports a put-back that could not happen.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // removal.ts
//! export type Judged = { device: number; inode: number; size: number; modified: bigint; changed: bigint };
//! export type Removal = 'retired' | 'gone' | 'replaced' | 'kept';
//! export function removeIfStill(path: string, judged: Judged, mayRemove: () => boolean): Removal  // throws
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `shell_word` from `super::cargo_installs`.
use super::cargo_installs::shell_word;

/// How many hidden names beside an entry are tried before a removal refuses to take it aside.
/// Delegated pick under the owner's rule that nothing waits without end and never discards a person's file;
/// each name is one earlier run's kept file, and a folder with every one of them taken is not one to guess in.
const TEMPORARY_NAME_ATTEMPTS: usize = 64;

/// What:
/// The entry at a path when it was judged:
/// the file it was, by everything an edit in place would change.
/// Why:
/// Inode identity alone misses a truncate-and-write in the same inode;
/// the size and the two timestamps move when any byte does,
/// so a file a person rewrote after the judgement is not taken for the judged one.
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
    /// The change time is deliberately not part of this: creating the hard link of the move-aside bumps it on the inode
    /// itself, so a compare that included it would take every entry for a changed one.
    modified: i128,
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

/// The identity of an entry from its metadata.
pub(crate) fn judged_of_metadata(metadata: &fs::Metadata) -> Judged {
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
    };
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
/// A hidden name beside `path` for the one no-replace step of a removal,
/// or the reason every name of the bound is taken.
/// Why:
/// The name is only *chosen* here, never created:
/// the hard link that follows fails when something took the name in between,
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
/// Takes the entry at `path` aside under a hidden name, replacing nothing,
/// and says whether it is the entry that was judged.
/// Why:
/// The link is the no-replace creation (step 3 of the steps at the top),
/// the unlink is step 4,
/// and the compare is step 5:
/// a different size or time means the file was rewritten after the judgement,
/// and that one is put back rather than removed.
pub(crate) fn take_aside(path: &Path, judged: Judged) -> Result<Taken, String> {
    let mut attempt: usize = 0;
    loop {
        let temporary: PathBuf = temporary_name_beside(path, attempt)?;
        #[cfg(test)]
        seam::before_the_link();
        match fs::hard_link(path, &temporary) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Taken::Gone),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                // Something took the hidden name in between: nothing was replaced, and the next name is tried.
                attempt += 1;
                continue;
            }
            Err(error) => {
                return Err(format!(
                    "it cannot be taken aside under a hidden name ({error}); a file system without hard links keeps it where it is \
                     rather than guessing"
                ));
            }
        }
        // The entry now has two names; the visible one goes, so the hidden one holds it alone.
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let _ = fs::remove_file(&temporary);
                return Ok(Taken::Gone);
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                return Err(format!("it cannot be moved aside: {error}"));
            }
        }
        let moved: Option<Judged> = fs::symlink_metadata(&temporary).ok().map(|metadata| return judged_of_metadata(&metadata));
        if moved != Some(judged) {
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
/// only if nothing stands at `path` now.
/// Why:
/// A hard link is the atomic "make this name only if it is free": it fails with `EEXIST` instead of writing over whatever
/// took the path, and the moved file then stays at its hidden name for the person to move.
/// A directory cannot be hard-linked; it is not an entry this tool removes, so it is left where it is and said.
pub(crate) fn put_back(path: &Path, temporary: &Path) -> Result<(), String> {
    match fs::hard_link(temporary, path) {
        Ok(()) => {
            let _ = fs::remove_file(temporary);
            return Ok(());
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(kept_aside(path, temporary)),
        Err(error) => return Err(could_not_put_back(path, temporary, &error)),
    }
}

/// Deletes the entry taken aside, and names where it is when it cannot be.
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

/// The seam a control uses to place a second actor's write between the name choice and the link that must not replace it.
/// Compiled in tests only, so no shipped path can be moved by anything but the file system.
#[cfg(test)]
pub(crate) mod seam {
    /// Imports `Mutex` from `std`.
    use std::sync::Mutex;

    /// The hook, when a test set one.
    static BEFORE_THE_LINK: Mutex<Option<Box<dyn Fn() + Send>>> = Mutex::new(None);

    /// Sets (or with `None` clears) the hook run between the name choice and the hard link.
    pub(crate) fn set_before_the_link(hook: Option<Box<dyn Fn() + Send>>) {
        *BEFORE_THE_LINK.lock().expect("the seam is not poisoned") = hook;
    }

    /// Runs the hook, when there is one.
    pub(crate) fn before_the_link() {
        if let Some(hook) = BEFORE_THE_LINK.lock().expect("the seam is not poisoned").as_ref() {
            hook();
        }
    }
}
