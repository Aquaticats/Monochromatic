//! What:
//!  Putting the checked-out lock back without replacing a file nobody kept:
//! the exchange that hands back whatever file stood there,
//! the judgement of that file,
//! the second look a file system that cannot exchange takes,
//! and the three steps that take a file away or write it where nothing stands.
//! Why:
//!   A window between a last look and a write cannot be closed from outside the writer,
//! but it can be made to lose nothing:
//! the checked-out bytes are exchanged with the lock in one step,
//! which leaves the file that stood there under the staged name,
//! and a write that landed after the reading is put back where it was written.
//! Every answer says what became of every byte,
//!  so the caller words one line from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Restored = { kind: 'the-checkout-is-back' } | { kind: 'their-write-stays'; kept?: string } | { kind: 'left-as-found'; reason: string };
//! const replaceTheCleansOwn = (path, before, after, keptIn) => Restored; // never replaces an unkept file
//! ```

/// Import the staged copy the checked-out bytes are taken from.
use crate::package_repair_cargo_lock_keeping_the_bytes::stage_the_checkout;
/// Import the name a file taken aside is moved to.
use crate::package_repair_cargo_lock_names::a_free_staging_name;
/// Import the borrowed and owned filesystem paths a step names.
use std::path::{
    Path,
    PathBuf,
};
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
pub(super) enum Restored {
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
///  When that file holds the bytes the reading named it is the clean's own,
///  and it goes
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
pub(super) fn replace_the_clean_s_own(
    path: &Path,
    before: Option<&[u8]>,
    after: Option<&[u8]>,
    kept_in: Option<&Path>,
) -> Restored {
    #[cfg(test)]
    BETWEEN_THE_READING_AND_THE_RESTORE.with(|hook| {
        if let Some(pause) = hook.borrow().as_ref() {
            pause();
        }
    });
    return match (before, after, kept_in) {
        (Some(checkout), Some(read), Some(directory)) => {
            exchange_the_file_read(path, checkout, read, directory)
        }
        // The clean created a lock the checkout has not, so the file goes, and a write that landed since keeps its file.
        (None, Some(read), Some(directory)) => take_the_file_away(path, read, directory),
        // The clean removed the lock the checkout has, so it is written where nothing stands.
        (Some(checkout), None, _) => write_where_nothing_stands(path, checkout),
        // Nothing stood there and the checkout has none: nothing to put back.
        (None, None, _) | (Some(_), Some(_), None) | (None, Some(_), None) => {
            Restored::TheCheckoutIsBack
        }
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
fn exchange_the_file_read(
    path: &Path,
    checkout: &[u8],
    read: &[u8],
    directory: &Path,
) -> Restored {
    let staged: PathBuf = match stage_the_checkout(directory, path, checkout) {
        Ok(staged) => staged,
        Err(reason) => return Restored::LeftAsFound(reason),
    };
    if let Err(error) = the_exchange(staged.as_path(), path) {
        let _dropped = std::fs::remove_file(staged.as_path());
        // A displaced directory on another filesystem than the worktree cannot exchange a file with the lock (`EXDEV`),
        //  which is what a linked worktree whose Git directory the main checkout holds does:
        // no step taken from here can reach across, so this is the plain write such a setup allows,
        //  and its one window is the one no step from outside the writer closes there.
        if is_cross_device(&error) {
            return match std::fs::write(path, checkout) {
                Ok(()) => Restored::TheCheckoutIsBack,
                Err(refusal) => Restored::LeftAsFound(format!(
                    "{} could not be written ({refusal})",
                    path.display()
                )),
            };
        }
        if !crate::rename_that_never_replaces::rename_unsupported(&error) {
            return Restored::LeftAsFound(format!(
                "{} could not be replaced ({error})",
                path.display()
            ));
        }
        return replace_after_a_second_look(path, checkout, read, directory);
    }
    return judge_the_displaced(path, staged.as_path(), read);
}

/// What:
///  The exchange that puts the checked-out bytes back,
///  or the refusal a control stands in for.
/// Why:
///   The one pair this branch serves is a clone's root on the clone's dataset
///  with its Git directory on the checkout's,
///  and an unprivileged control cannot build two filesystems inside one fixture
///  root.
///  The guest arm proves the kernel's own `EXDEV` across the two datasets;
///  the seam is how a host control reaches the branch at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exchange(staged, lock) // or the refusal a control planted
/// ```
fn the_exchange(
    from: &Path,
    to: &Path,
) -> std::io::Result<()> {
    #[cfg(test)]
    {
        if THE_NEXT_EXCHANGE_FINDS_ANOTHER_FILESYSTEM.with(|flag| return flag.replace(false)) {
            return Err(std::io::Error::from_raw_os_error(libc::EXDEV));
        }
    }
    return crate::rename_that_never_replaces::exchange(from, to);
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
/// Every other write,
///  before the reading or after the rename,
///  is judged as the exchange judges it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (read(lock) !== read) return theirWriteStays(staged); await rename(staged, lock); return theCheckoutIsBack;
/// ```
fn replace_after_a_second_look(
    path: &Path,
    checkout: &[u8],
    read: &[u8],
    directory: &Path,
) -> Restored {
    // The exchange's staging file went, so the bytes are staged again for this step.
    let staged: PathBuf = match stage_the_checkout(directory, path, checkout) {
        Ok(staged) => staged,
        Err(reason) => return Restored::LeftAsFound(reason),
    };
    let standing: Option<Vec<u8>> = std::fs::read(path).ok();
    if standing.as_deref() != Some(read) {
        // The bytes that stand there are not the ones the reading named, so they are a person's and stay.
        return Restored::TheirWriteStays(Some(staged));
    }
    if let Err(error) = std::fs::rename(staged.as_path(), path) {
        let _dropped = std::fs::remove_file(staged.as_path());
        return Restored::LeftAsFound(format!(
            "{} could not be replaced ({error})",
            path.display()
        ));
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
fn judge_the_displaced(
    path: &Path,
    displaced: &Path,
    read: &[u8],
) -> Restored {
    let handed_back: Option<Vec<u8>> = std::fs::read(displaced).ok();
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
///  A move is one step,
///  so the file that goes is the file that stood there when the step ran,
///  whatever a person wrote after the reading;
///  the bytes are judged from the moved file and not from an earlier reading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = freeName(dir, lock); rename(lock, moved); return judgeTheMovedFile(lock, moved, read);
/// ```
fn take_the_file_away(
    path: &Path,
    read: &[u8],
    directory: &Path,
) -> Restored {
    let moving: PathBuf = a_free_staging_name(directory, path);
    if let Err(error) = std::fs::rename(path, moving.as_path()) {
        if error.kind() == std::io::ErrorKind::NotFound {
            return Restored::TheCheckoutIsBack;
        }
        return Restored::LeftAsFound(format!(
            "{} could not be moved aside ({error})",
            path.display()
        ));
    }
    let moved: Option<Vec<u8>> = std::fs::read(moving.as_path()).ok();
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
fn write_where_nothing_stands(
    path: &Path,
    checkout: &[u8],
) -> Restored {
    let opened: std::io::Result<std::fs::File> = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path);
    let mut file: std::fs::File = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Restored::TheirWriteStays(None);
        }
        Err(error) => {
            return Restored::LeftAsFound(format!(
                "{} could not be written ({error})",
                path.display()
            ));
        }
    };
    if let Err(error) = std::io::Write::write_all(&mut file, checkout) {
        drop(file);
        let _dropped = std::fs::remove_file(path);
        return Restored::LeftAsFound(format!("{} could not be written ({error})", path.display()));
    }
    return Restored::TheCheckoutIsBack;
}

#[cfg(test)]
thread_local! {
    /// A control may stop the restore between the reading that judged the lock and the step that puts the checked-out bytes back,
    /// which is where a person's write lands.
    /// Why:
    ///   The window cannot be reached from outside the writer on a wall clock,
    /// and a control that waited on one would pass whatever the code did.
    pub(crate) static BETWEEN_THE_READING_AND_THE_RESTORE: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
    /// A control may make the next exchange answer the kernel's `EXDEV`,
    /// so the branch for a displaced directory on another filesystem is reached
    /// in a fixture root that holds one filesystem.
    /// Why:
    ///   No unprivileged control can build the pair the branch serves,
    ///   and a branch nothing reaches is a branch nothing proves.
    pub(crate) static THE_NEXT_EXCHANGE_FINDS_ANOTHER_FILESYSTEM: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}
