//! What:
//!  The removal of a stale index lock by one reclaimer at a time:
//! the reclaimer opens the lock it will judge,
//! reads the marker
//!  from that open file,
//! takes its place in the claim chain beside the lock,
//! checks that the name still refers to that file,
//! and
//!  only then unlinks the name.
//! Why:
//!  A removal by name removes whatever the name holds by then.
//! Between a look at the marker and the removal,
//! the stale lock may
//!  be gone and a live lock in its place;
//! that happens when a second reclaimer (another instance serving the same physical
//!  index,
//! an alternate index shared by two worktrees,
//! a service on another machine over a shared file system) removed the stale
//!  lock and placed its own.
//! The earlier cure took the file aside by a rename and put it back by a hard link,
//! which left the
//!  live lock off its name for a while,
//! in which Git could take the name and strand it.
//! Here no file is moved:
//! the claim chain
//!  (`service_index_claim.rs`) lets one reclaimer of that file go on,
//! and a reclaimer that opened it before it was removed finds
//!  the name no longer refers to it.
//! The chain is used on every file system,
//! because a reclaimer on the `flock` path and one on
//!  the claim path would not see each other.
//! A live landing cannot place a lock while the name is occupied.
//! What stays open is a
//!  person's `rm index.lock` between the check and the unlink,
//! because no system call unlinks a name only if it still names a
//!  given file (`doc/planning/cli-git-one-reclaimer-per-index.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! reclaimIf(platform, lock, isStale) // open; judge; takeClaim ?? busy; sameFile(lock, fd) ? unlink(lock) : changed
//! ```

use crate::service_index_lock::lock_path_of;
use crate::service_index_claim::{Claim, ClaimHold, take_claim_carrying};
use crate::service_index_marker::{LOCK_MARKER, MARKER_VERSION, Marked, holder_is_elsewhere, holder_is_gone, marker_of_file};
use crate::service_index_placement::{Platform, SYSTEM};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

/// What a reclaimer made of the file at a lock's name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reclaimed {
    /// No lock is left in the way:
    /// the stale file was removed,
    /// or was already gone.
    Removed,
    /// The name no longer refers to the file that was judged stale:
    /// the placement is worth another try.
    Changed,
    /// Another reclaimer holds a claim on the stale file and is removing it:
    /// the lock is not this reclaimer's to remove.
    Busy,
    /// The file is not a stale lock of the service (a live holder's,
    /// Git's,
    /// or not readable):
    /// it is left where it is.
    Kept,
    /// The file is stale and cannot be removed;
    /// the log says why.
    Stuck,
    /// The file is stale,
    /// but no claim file could be placed beside it (the file system refuses the placement):
    /// the file is left
    ///  where it is,
    /// and the person is told how to remove it.
    /// It carries the marker read from the very file that was judged,
    /// so
    ///  the notice never describes a file that replaced it.
    Unguarded(Marked),
    /// The file (the lock,
    /// or a claim in the way of its removal) was written by a service on another machine,
    /// or on one whose
    ///  machine cannot be compared with this one:
    /// nothing proves its maker gone,
    /// so it is left where it is and the person is
    ///  told how to remove it.
    /// It carries the marker read from the file.
    Foreign { file: PathBuf, marked: Marked },
    /// A claim in the way cannot be judged (its maker is in another PID namespace,
    /// or the claim cannot be read):
    /// it is not removed,
    ///  not passed over and not a passing contention,
    /// and the person is told which file it is.
    /// It carries the marker when one
    ///  could be read.
    Unjudged { file: PathBuf, marked: Option<Marked> },
}

/// What:
///  Which marks `reclaim_if` takes for stale.
/// Why:
///   A mark is stale when its holder is gone;
/// the removal of a killed service's lock adds that the mark is the one of its own batch.
///  Each kind is a value,
/// so no closure has to carry the batch number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StaleMark = { kind: 'holder-gone' } | { kind: 'of-batch'; seq: number };
/// ```
#[derive(Clone, Copy, Debug)]
pub enum StaleMark {
    /// Any mark whose holder is gone.
    HolderIsGone,
    /// A mark of the batch with this number whose holder is gone.
    OfBatchWhoseHolderIsGone(u64),
}

impl StaleMark {
    /// Whether `marked` is stale under this kind.
    fn judges_stale(self, marked: &Marked) -> bool {
        return match self {
            StaleMark::HolderIsGone => holder_is_gone(marked.holder.as_ref()),
            StaleMark::OfBatchWhoseHolderIsGone(seq) => marked.seq == seq && holder_is_gone(marked.holder.as_ref()),
        };
    }
}

/// What:
///  Removes the index lock of `index` left by a service that was killed,
/// when it still carries the service's marker for `seq`
///  and its holder is gone.
/// Why:
///   Only the marker proves the lock is ours;
/// a foreign lock,
/// one a Git holds,
/// or one whose holder still runs is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeStaleLock(index, seq)
/// ```
pub fn remove_stale(index: &Path, seq: u64) {
    let lock: PathBuf = lock_path_of(index);
    let _outcome: Reclaimed = reclaim_if(&SYSTEM, lock.as_path(), StaleMark::OfBatchWhoseHolderIsGone(seq));
}

/// What:
///  Removes the file at `lock` if `is_stale` says the marker in it is a stale one,
/// while no other reclaimer holds the file.
/// Why:
///   One reclaimer per file at a time is what keeps a live lock another instance placed after the marker was read from being
///  removed.
/// The answer says what the caller does next:
/// place again after `Removed` and `Changed`,
/// wait as for any held lock
///  after `Busy` and `Kept`,
/// and give up after `Stuck`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = reclaimIf(platform, lock, isStale) // 'removed' | 'changed' | 'busy' | 'kept' | 'stuck'
/// ```
pub fn reclaim_if(platform: &Platform, lock: &Path, is_stale: StaleMark) -> Reclaimed {
    let mut abandoned: Vec<(PathBuf, String)> = crate::service_index_claim::abandoned_here();
    let answer: Reclaimed = reclaim_if_carrying(platform, lock, is_stale, &mut abandoned);
    crate::service_index_claim::keep_abandoned(abandoned);
    return answer;
}

/// What:
///  `reclaim_if` with the landing's abandoned claims carried rather than read from its state.
/// Why:
///  A reclaim of a job of the blocking pool cannot read the landing's state,
/// and a claim whose placement is not known must still be removed by that same landing's next attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = reclaimIfCarrying(platform, lock, isStale, abandoned)
/// ```
pub(crate) fn reclaim_if_carrying(platform: &Platform, lock: &Path, is_stale: StaleMark, abandoned: &mut Vec<(PathBuf, String)>) -> Reclaimed {
    let file: std::fs::File = match open_to_judge(lock) {
        Ok(opened) => opened,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Reclaimed::Removed,
        Err(_) => return Reclaimed::Kept,
    };
    // A control reaches here to act as another program would while the reclaimer looks at the file.
    crate::service_fault::reach("reclaim-opened");
    let Some(marked) = marker_of_file(&file) else {
        return Reclaimed::Kept;
    };
    if !is_stale.judges_stale(&marked) {
        // A holder on another machine is not proven gone and not proven running: the person is told, not the lock removed.
        if holder_is_elsewhere(marked.holder.as_ref()) {
            crate::service_error!("{}", foreign_words(lock, &marked));
            return Reclaimed::Foreign { file: lock.to_path_buf(), marked };
        }
        return Reclaimed::Kept;
    }
    // A lock in the format of an earlier release is reclaimed by that release's reclaimers (a `flock`), which cannot see the claims
    //  of this one: removing it here could be a second removal of the same file. It is left, and the person is told.
    if !marked.current {
        crate::service_error!("{}", foreign_words(lock, &marked));
        return Reclaimed::Foreign { file: lock.to_path_buf(), marked };
    }
    crate::service_fault::reach("reclaim-judged");
    // The exclusion this reclaimer is under until it is done.
    let claim: ClaimHold = match take_claim_carrying(platform, lock, &marked, abandoned) {
        Claim::Held(held) => held,
        Claim::Busy => return Reclaimed::Busy,
        Claim::Unjudged { file: in_the_way, marked: theirs } => {
            crate::service_error!("{}", unjudged_words(in_the_way.as_path(), theirs.as_ref()));
            return Reclaimed::Unjudged { file: in_the_way, marked: theirs };
        }
        Claim::Foreign { file: in_the_way, marked: theirs } => {
            crate::service_error!("{}", foreign_words(in_the_way.as_path(), &theirs));
            return Reclaimed::Foreign { file: in_the_way, marked: theirs };
        }
        Claim::Unavailable(why) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it is not removed: no claim file could be placed beside it to keep two reclaimers apart ({why}), so a removal could take a live lock; \
                 remove it by hand once no git command runs in this repository",
                lock.display()
            );
            return Reclaimed::Unguarded(marked);
        }
    };
    crate::service_fault::reach("reclaim-held");
    match same_file(lock, &file) {
        Ok(true) => {}
        Ok(false) => return cleared(claim, Reclaimed::Changed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return cleared(claim, Reclaimed::Removed),
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be looked at ({error})", lock.display());
            return Reclaimed::Stuck;
        }
    }
    return match (platform.unlink)(lock) {
        Ok(()) => cleared(claim, Reclaimed::Removed),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => cleared(claim, Reclaimed::Removed),
        Err(error) => {
            crate::service_error!("the index lock {} was left by a service that is gone, and it cannot be removed ({error}); \
                 remove it by hand, and clear its immutable attribute or fix the directory's permissions first if that is why it stays",
                lock.display()
            );
            Reclaimed::Stuck
        }
    };
}

/// `answer`,
/// after the claims of a reclaimer that saw the stale file leave its name are removed.
fn cleared(claim: ClaimHold, answer: Reclaimed) -> Reclaimed {
    claim.clear_all();
    return answer;
}

/// Opens `lock` to read its marker without following a link or waiting on a pipe,
/// and without a descriptor a child could inherit.
pub(crate) fn open_to_judge(lock: &Path) -> std::io::Result<std::fs::File> {
    return std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC).open(lock);
}

/// Whether the name `lock` refers to the very file `file` has open:
/// the same device and the same inode,
/// so a replaced file whose
///  inode number was reused cannot pass while `file` keeps the old one alive.
pub(crate) fn same_file(lock: &Path, file: &std::fs::File) -> std::io::Result<bool> {
    let named = std::fs::symlink_metadata(lock)?;
    let open = file.metadata()?;
    return Ok(named.dev() == open.dev() && named.ino() == open.ino());
}

/// A path as one word for a shell:
/// in single quotes,
/// a quote inside it closed,
/// escaped and opened again.
pub fn shell_quote(path: &Path) -> String {
    return format!("'{}'", path.display().to_string().replace('\'', "'\\''"));
}

/// What:
///  The words for a stale lock that was not removed:
/// which lock,
/// which batch and process left it,
/// how to tell it is still that
///  lock,
/// and how to remove it.
/// Why:
///   Native Git leaves a stale `index.lock` to the person,
/// and so does a service that cannot place the claim a safe removal needs;
///  the person must be able to act from the notice alone.
/// `marked` is the marker read from the file that was judged,
/// never a
///  second read of the name,
/// which another landing may have taken since;
/// the first line to look for lets the person see that
///  the file is still the one described.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The index lock ${lock} was left by cli-git-service batch ${seq} (process ${pid}, gone). ... remove it yourself: rm -- ${quote(lock)}`
/// ```
pub fn stale_lock_words(lock: &Path, marked: &Marked) -> String {
    let (who, first_line): (String, String) = match marked.holder.as_ref() {
        Some((pid, birth)) => (
            format!("cli-git-service batch {} (process {pid}, which is gone)", marked.seq),
            format!("{LOCK_MARKER}{}{} {pid} {birth}", if marked.current { MARKER_VERSION } else { "" }, marked.seq),
        ),
        None => (format!("cli-git-service batch {}", marked.seq), format!("{LOCK_MARKER}{}", marked.seq)),
    };
    return format!(
        "The index lock {} was left by {who} and is not removed here, because no claim file that makes its removal safe could be placed beside it on this file system. \
         The commit waits as for any held lock. Once no git command and no commit of this service is running in this repository, and the first line of the file \
         still reads `{first_line}`, remove it yourself (`rm -- {}`).",
        lock.display(),
        shell_quote(lock)
    );
}

/// What:
///  The words for a file left because its maker is not proven gone on this machine:
/// the file,
/// the batch and process,
/// why it
///  stays,
/// and how to remove it.
/// Why:
///   A boot ID that differs says nothing about a service on another machine,
/// which may be running and holding the file;
/// a marker
///  of an earlier release names no machine (or no process) and cannot say whether it was written here;
/// and a lock in an earlier
///  release's format is reclaimed by that release's own exclusion,
/// which this release cannot join.
/// So the file is neither removed
///  nor called stale;
/// the person who knows that service is gone can remove it,
/// and needs the notice alone to do so.
/// `marked` is
///  the marker read from the file itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The file ${file} was written by cli-git-service batch ${seq} (process ${pid}) ... remove it yourself: rm -- ${quote(file)}`
/// ```
pub fn foreign_words(file: &Path, marked: &Marked) -> String {
    let who: String = match marked.holder.as_ref() {
        Some((pid, _)) => format!("cli-git-service batch {} (process {pid})", marked.seq),
        None => format!("cli-git-service batch {}", marked.seq),
    };
    let why: &str = if marked.current {
        "on another machine, or on one this machine cannot compare itself with, so that service is not proven gone"
    } else {
        "in the format of an earlier release, whose own stale-lock removal cannot keep this release's apart from it and whose marker \
         does not say which machine wrote it, so this release does not remove it"
    };
    return format!(
        "The file {} was written by {who} {why}. The commit waits as for any held lock. \
         If that service is not running and no git command and no commit of any release's service runs in this repository, remove it yourself (`rm -- {}`).",
        file.display(),
        shell_quote(file)
    );
}

/// What:
///  The words for a claim that cannot be judged:
/// the file,
/// the maker if it could be read,
/// why it stays,
/// and how to remove it.
/// Why:
///   A claim whose maker is in another PID namespace,
/// or that cannot be read,
/// is neither gone nor proven running;
/// it blocks every
///  reclaim of the stale lock for as long as it stands,
/// so the wait must name it and not call it passing contention.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `The claim ${file} (made by batch ${seq}, process ${pid}) cannot be judged ... remove it yourself: rm -- ${quote(file)}`
/// ```
pub fn unjudged_words(file: &Path, marked: Option<&Marked>) -> String {
    let who: String = match marked {
        Some(Marked { seq, holder: Some((pid, _birth)), .. }) => format!("made by cli-git-service batch {seq} (process {pid}) "),
        _ => String::new(),
    };
    return format!(
        "The claim file {} {who}cannot be judged here (its maker is in another PID namespace, or the file cannot be read), so it is neither removed nor passed over, and it keeps the stale index lock from being reclaimed. \
         The commit waits as for any held lock. If that reclaimer is gone and no git command and no commit of any service runs in this repository, remove it yourself (`rm -- {}`).",
        file.display(),
        shell_quote(file)
    );
}
