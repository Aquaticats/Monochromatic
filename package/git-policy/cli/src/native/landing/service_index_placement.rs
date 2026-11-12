//! What:
//!  How the service puts its marked file in place as `index.lock`:
//! the system calls it makes,
//! the atomic placement itself,
//! and the removal of a stale lock.
//! Why:
//!  The lock must not exist without its marker where the filesystem lets it,
//! so the marker is written to a file that is new and then made the lock in one step (a rename that does not replace,
//! else a hard link).
//! Where it does neither,
//! the lock is taken as Git takes its own:
//! created by exclusive creation and then given its marker,
//! so a kill between the two leaves a lock without its marker,
//! as plain Git leaves one,
//! and nothing removes it unless its holder is proven gone.
//! The calls are one `Platform` value so a control can refuse or count them for one acquisition alone.
//! The module is split from `service_index_lock.rs`,
//! which re-exports the public items here under their old names;
//! the staging files are in `service_index_staging.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await placeLock(platform, staged, marker, lock) // renameNoReplace ?? link ?? createExclusive then write
//! ```

use crate::fs_mount_options::read_mount_facts;
use crate::process_identity::ProcessIdentityError;
use crate::service_index_marker::own_identity;
use crate::service_index_ownership::{Ownership, inspect_ownership};
use crate::service_index_watch::Watch;
use std::path::Path;

/// What:
///  The system calls the lock makes that a control must be able to refuse,
/// count or observe:
/// the rename that does not replace,
/// the hard link,
/// the exclusive creation of a lock file,
/// the removal of a stale lock,
/// the process's own birth identity,
/// the setting of a watch on the lock's directory,
/// the flush of the staged marker to storage,
/// the question whether a lock is still the holder's,
/// and the removal of a lock the holder owns.
/// Why:
///   A filesystem with neither atomic placement,
/// a stale lock that cannot be unlinked,
/// a process with no identity,
/// a process out of descriptors and a watch quota that is used up
/// are environments no fixture can be made to have without changing the whole test process,
/// and a count kept in a process-wide static is read by whichever test runs beside it;
/// each acquisition takes the calls it makes from one value,
/// so a control substitutes them for that acquisition alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Platform = { rename; link; unlink; identity; watch; scan; sync; inspect };
/// ```
pub struct Platform<'a> {
    /// A rename that fails when the target exists (`renameat2` with `RENAME_NOREPLACE`).
    pub rename: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// A hard link that fails when the target exists.
    pub link: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// The creation of a file that must not exist yet (`open` with `O_CREAT | O_EXCL`),
    /// which is how Git creates `index.lock`;
    /// used only where the filesystem refuses both of the calls above.
    pub create: &'a dyn Fn(&Path) -> std::io::Result<std::fs::File>,
    /// The removal of a lock whose holder is gone.
    pub unlink: &'a dyn Fn(&Path) -> std::io::Result<()>,
    /// The birth identity of this process,
    /// which a marker names.
    pub identity: &'a dyn Fn() -> Result<String, ProcessIdentityError>,
    /// A watch on the directory of the lock.
    pub watch: &'a dyn Fn(&Path) -> Option<Watch>,
    /// The scan for the processes that have the lock open,
    /// which decides whether a lock nothing is known to hold is removed.
    pub scan: fn(&Path, u64, u64) -> crate::index_lock_holders::HolderEvidence,
    /// The flush of the staged marker to storage,
    /// before it becomes the lock.
    pub sync: &'a dyn Fn(&std::fs::File) -> std::io::Result<()>,
    /// Whether the lock at a path is still the file the holder placed.
    pub inspect: fn(&Path, &std::fs::File, &str) -> Ownership,
    /// The removal of the lock the holder owns,
    /// at its release or when it goes out of scope.
    pub release: fn(&Path) -> std::io::Result<()>,
}

/// The operating system's own calls.
pub const SYSTEM: Platform<'static> = Platform {
    rename: &rename_noreplace,
    link: &link_file,
    create: &create_exclusive,
    unlink: &unlink_file,
    identity: &own_identity,
    watch: &Watch::on,
    scan: crate::index_lock_holders::scan_platform_holders_bounded,
    sync: &sync_file,
    inspect: inspect_ownership,
    release: unlink_file,
};

/// The name the record of the lock marker's flush carries,
/// in the sync figures,
/// in a landing's timing records and in the background task's own line.
/// Why:
///  A control of what a landing waits for tells the marker's flush from a record's by this name,
/// so both
///  read it here instead of writing the word twice.
pub const LOCK_MARKER_SYNC: &str = "lock-marker";

/// Flushes the lock marker the index lock is made of,
/// in the background task,
/// and never waits for it.
/// Why:
///  The marker names the process that holds the lock and nothing else,
/// and a power cut that tears it leaves a lock
///  the wrapper judges stale and removes (`stale_ref_lock_judge.rs`),
/// so nothing a person's commit needs is lost by the flush happening
///  later;
/// and the flush itself sat on every commit's path,
/// 8 to 12.5 s of it under load
/// (`doc/planning/git-policy-cli-btrfs-transaction-waits.md`).
pub fn sync_file(file: &std::fs::File) -> std::io::Result<()> {
    return crate::service_durable::defer_file(file, LOCK_MARKER_SYNC);
}

fn link_file(from: &Path, to: &Path) -> std::io::Result<()> {
    return std::fs::hard_link(from, to);
}

fn create_exclusive(path: &Path) -> std::io::Result<std::fs::File> {
    return std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(path);
}

fn unlink_file(path: &Path) -> std::io::Result<()> {
    return std::fs::remove_file(path);
}

pub use crate::rename_that_never_replaces::{rename_noreplace, rename_unsupported};

/// Whether a failed hard link means the filesystem makes none (link(2):
/// `EPERM`),
/// not that the lock exists or storage failed.
pub fn links_unsupported(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(unix)]
    {
        return matches!(error.raw_os_error(), Some(code) if code == libc::EPERM || code == libc::EOPNOTSUPP);
    }
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// What the filesystem answered when the lock was asked to be placed whole.
enum Atomic {
    /// Neither a rename that does not replace nor a hard link can be made there;
    /// the error names both refusals.
    Refused(std::io::Error),
    /// Any other failure:
    /// the lock exists,
    /// or storage failed.
    Failed(std::io::Error),
}

/// What:
///  Places `staged` as `lock` in one step:
/// a rename that does not replace,
/// else a hard link.
/// Why:
///   Both are exclusive and atomic,
/// so the lock carries its marker from the instant it exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameNoReplace(staged, lock) ?? await link(staged, lock) ?? refused()
/// ```
fn place_whole(platform: &Platform, staged: &Path, lock: &Path) -> Result<(), Atomic> {
    let refused_rename: std::io::Error = match (platform.rename)(staged, lock) {
        Ok(()) => return Ok(()),
        Err(error) if !rename_unsupported(&error) => return Err(Atomic::Failed(error)),
        Err(error) => error,
    };
    match (platform.link)(staged, lock) {
        Ok(()) => {
            let _removed = std::fs::remove_file(staged);
            return Ok(());
        }
        Err(error) if !links_unsupported(&error) => return Err(Atomic::Failed(error)),
        Err(refused_link) => {
            return Err(Atomic::Refused(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("renaming without replacing failed ({refused_rename}) and hard links failed ({refused_link})"),
            )));
        }
    }
}

/// The storage error a refused or failed placement carries.
fn error_of(refusal: Atomic) -> std::io::Error {
    return match refusal {
        Atomic::Refused(error) | Atomic::Failed(error) => error,
    };
}

/// What:
///  Makes the marked `staged` file the file at `lock`,
/// whole;
/// `AlreadyExists` when it exists,
/// and `staged` then stays for the next attempt.
/// Why:
///   A claim file is read by other reclaimers while it is being made,
/// so it must never exist half written;
/// a rename that does not replace is exclusive and atomic and leaves no second name,
/// and where the filesystem has none a hard link is exclusive too and its staging name is removed after.
/// Where it has neither nothing is created,
/// because exclusive creation followed by a write would show a reclaimer an empty claim for as long as a kill between the two lasts.
/// The index lock itself does not stop here (`place_lock`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameNoReplace(staged, claim) ?? await link(staged, claim) ?? refuse()
/// ```
///
/// # Errors
///
/// `AlreadyExists` when the file exists,
/// `Unsupported` (naming both refusals) when the filesystem can place neither way,
/// and the storage error otherwise.
pub fn place_marked(platform: &Platform, staged: &Path, lock: &Path) -> std::io::Result<()> {
    return place_whole(platform, staged, lock).map_err(error_of);
}

/// How a lock came to be placed.
#[derive(Debug)]
pub enum Placed {
    /// The staged file is the lock:
    /// the holder keeps its descriptor.
    Whole,
    /// The lock was created by exclusive creation and given its marker afterwards;
    /// this descriptor is the lock's own inode,
    /// and the holder keeps it instead of the staged file's.
    Created(std::fs::File),
}

impl Placed {
    /// The descriptor that names the lock's inode:
    /// the one the lock was created with,
    /// else the staged file's.
    pub fn held_or(self, staged: std::fs::File) -> std::fs::File {
        return match self {
            Placed::Whole => staged,
            Placed::Created(created) => created,
        };
    }
}

/// Write `marker` to the lock file just created and flush it,
/// with the fault points of a kill between the two.
fn write_marker_and_sync(platform: &Platform, file: &mut std::fs::File, marker: &str) -> std::io::Result<()> {
    std::io::Write::write_all(file, marker.as_bytes())?;
    // A kill here leaves a lock with its marker, not yet flushed.
    crate::service_fault::reach(crate::service_fault::LOCK_MARKED_UNSYNCED);
    return (platform.sync)(file);
}

/// What:
///  Takes `index.lock` for the marked `staged` file:
/// whole where the filesystem has a rename that does not replace or a hard link,
/// else by exclusive creation with `marker` written afterwards.
/// Why:
///   Git's own lock is taken by exclusive creation (`open` with `O_CREAT | O_EXCL`,
/// `lockfile.c` `lock_file`),
/// which needs neither call,
/// so a filesystem that refuses both still lets Git commit and must let this service land a commit too ("Commits should never fail.").
/// The two steps leave a window:
/// a kill between them leaves a lock without its marker,
/// as plain Git leaves one when it is killed holding its lock.
/// Such a lock may be a Git that is mid-write,
/// so nothing removes it unless its holder is proven gone,
/// and the wait for it ends at the ceiling with the file named.
/// The fallback is taken only when both calls were refused,
/// never as a first choice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await placeWhole(staged, lock) ?? await createExclusive(lock).then(write(marker))
/// ```
///
/// # Errors
///
/// `AlreadyExists` when the lock exists,
/// and the storage error otherwise;
/// a lock created here whose marker could not be written or flushed is removed again.
pub fn place_lock(platform: &Platform, staged: &Path, marker: &str, lock: &Path) -> std::io::Result<Placed> {
    let refusal: std::io::Error = match place_whole(platform, staged, lock) {
        Ok(()) => return Ok(Placed::Whole),
        Err(Atomic::Failed(error)) => return Err(error),
        Err(Atomic::Refused(error)) => error,
    };
    let mut file: std::fs::File = (platform.create)(lock)?;
    crate::service_log!(
        "the index lock {} is taken by exclusive creation, as Git takes it, because this filesystem can place a file in neither atomic way ({refusal})",
        lock.display()
    );
    // A kill here leaves a lock without its marker.
    crate::service_fault::reach(crate::service_fault::LOCK_CREATED_UNMARKED);
    let marked: std::io::Result<()> = write_marker_and_sync(platform, &mut file, marker);
    if let Err(error) = marked {
        drop(file);
        if let Err(removal) = std::fs::remove_file(lock) {
            crate::service_error!("the index lock {} could not be given its marker ({error}) and could not be removed again ({removal}); it stays", lock.display());
        }
        return Err(error);
    }
    return Ok(Placed::Created(file));
}

/// What:
///  Says in the service's log why a lock was not placed:
/// the lock,
/// the cause,
/// and for a filesystem that answers that it cannot place a file at all,
/// the filesystem and what to do.
/// Why:
///   The clients are answered with a code only;
/// the person who reads the log must find the filesystem and the remedy there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.error(`cli-git-service: could not place ${lock}: ${error}`)
/// ```
pub fn log_placement_failure(lock: &Path, error: &std::io::Error) {
    if let Some(text) = placement_failure_text(lock, error) {
        crate::service_error!("{text}");
    }
}

/// What:
///  The log line for a failure to place `lock`,
/// or nothing when the failure is that the lock exists.
/// Why:
///   A lock that is held is the ordinary case of a wait and is no failure to place;
/// every other error says what happened,
/// and one that means the filesystem cannot place the file at all names the filesystem and what to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// placementFailureText(lock, error) // string | undefined
/// ```
pub fn placement_failure_text(lock: &Path, error: &std::io::Error) -> Option<String> {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        return None;
    }
    if error.kind() != std::io::ErrorKind::Unsupported {
        return Some(format!("cannot take the index lock {}: {error}", lock.display()));
    }
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let filesystem: String = match read_mount_facts(directory) {
        Ok(Some(facts)) => format!("the {} filesystem mounted at {}", facts.filesystem_type, facts.mount_point.display()),
        Ok(None) => format!("the filesystem of {}", directory.display()),
        Err(reason) => format!("the filesystem of {} ({reason})", directory.display()),
    };
    return Some(format!(
        "cannot take the index lock {}: {filesystem} answered that it cannot place a file ({error}), and Git creates this file the same way, \
         so Git cannot lock the repository there either. \
         Keep this repository's Git directory on a filesystem that supports it (ext4, xfs, btrfs, tmpfs and overlayfs do), or update the kernel.",
        lock.display()
    ));
}

/// Removes the stale `lock`,
/// whose holder is gone,
/// while no other reclaimer holds it;
/// what came of it is the answer (`Reclaimed`),
/// and the log says why a stale lock stays.
pub fn reclaim(platform: &Platform, lock: &Path) -> Reclaimed {
    return reclaim_if(platform, lock, StaleMark::HolderIsGone);
}

/// What:
///  `reclaim` with the landing's abandoned claims carried rather than read from its state,
/// for the reclaim of a job of the blocking pool.
/// Why:
///  The call chain a job takes cannot reach the landing's state,
/// and what a claim's failed placement left must still be removed by the same landing's next attempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = reclaimCarrying(platform, lock, abandoned)
/// ```
pub(crate) fn reclaim_carrying(platform: &Platform, lock: &Path, abandoned: &mut Vec<(std::path::PathBuf, String)>) -> Reclaimed {
    return reclaim_if_carrying(platform, lock, StaleMark::HolderIsGone, abandoned);
}

pub use crate::service_index_reclaim::{Reclaimed, StaleMark, reclaim_if};

/// The reclaim a job of the blocking pool makes,
/// with the landing's abandoned claims carried.
use crate::service_index_reclaim::reclaim_if_carrying;
