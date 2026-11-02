//! What:
//!  How the service puts its marked file in place as `index.lock`: the system calls it makes, the staging files, the atomic
//!  placement itself, the removal of a stale lock, and the cleanup of staging files a killed attempt left.
//! Why:
//!  The lock must never exist without its marker, so the marker is written to a file that is new and then made the lock in
//!  one step (a rename that does not replace, else a hard link). The calls are one `Platform` value so a control can refuse
//!  or count them for one acquisition alone. The module is split from `service_index_lock.rs`, which re-exports the public
//!  items here under their old names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await placeMarked(platform, staged, lock) // renameNoReplace ?? link ?? refuse
//! ```

use super::fs_mount_options::read_mount_facts;
use super::process_identity::{ProcessIdentityError, current_birth_identity, process_birth_identity};
use super::random_id::random_uuid;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What follows the lock's file name in the name of a staging file: the marker is written there before it becomes the lock.
pub const STAGED_INFIX: &str = ".cli-git-marker";

/// What:
///  The system calls the lock makes that a control must be able to refuse or count: the rename that does not replace, the
///  hard link, the removal of a stale lock, and the question of the process's own birth identity.
/// Why:
///   A filesystem with neither atomic placement, a stale lock that cannot be unlinked and a process with no identity are
///  environments no fixture can be made to have, and a count kept in a process-wide static is read by whichever test runs
///  beside it; each acquisition takes the calls it makes from one value, so a control substitutes them for that
///  acquisition alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Platform = { rename(from, to): void; link(from, to): void; unlink(path): void; identity(): string };
/// ```
pub struct Platform<'a> {
    /// A rename that fails when the target exists (`renameat2` with `RENAME_NOREPLACE`).
    pub rename: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// A hard link that fails when the target exists.
    pub link: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// The removal of a lock whose holder is gone.
    pub unlink: &'a dyn Fn(&Path) -> std::io::Result<()>,
    /// The birth identity of this process, which a marker names.
    pub identity: &'a dyn Fn() -> Result<String, ProcessIdentityError>,
}

/// The operating system's own calls.
pub const SYSTEM: Platform<'static> = Platform { rename: &rename_noreplace, link: &link_file, unlink: &unlink_file, identity: &current_birth_identity };

fn link_file(from: &Path, to: &Path) -> std::io::Result<()> {
    return std::fs::hard_link(from, to);
}

fn unlink_file(path: &Path) -> std::io::Result<()> {
    return std::fs::remove_file(path);
}

/// What:
///  The name of a staging file of the lock at `lock`: this process, the batch, and a token that no other attempt has.
/// Why:
///   The process number alone recurs once the process is gone, and a name that recurs can be the name of a hard-link alias a
///  killed holder left, which is the lock's own inode; the token makes every name new, and the process number lets a later
///  attempt tell from the name alone whether the writer still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${lock}.cli-git-marker-${process.pid}-${seq}-${token}`
/// ```
pub fn staged_path_for(lock: &Path, seq: u64, token: &str) -> PathBuf {
    let mut name = lock.as_os_str().to_os_string();
    name.push(format!("{STAGED_INFIX}-{}-{seq}-{token}", std::process::id()));
    return PathBuf::from(name);
}

/// What:
///  Creates the staging file of the lock at `lock` for batch `seq`, new and holding `marker`.
/// Why:
///   Opened with exclusive creation and never truncation, so a name that already exists (whatever it is a name of) is an
///  error and no file of another attempt is emptied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(stagedPath, marker, { flag: 'wx' })
/// ```
///
/// # Errors
///
/// The storage error when the file cannot be created or written; it is removed again when only the write failed.
pub fn stage_marker(lock: &Path, seq: u64, marker: &str) -> std::io::Result<PathBuf> {
    let token: String = random_uuid().map_err(|error| { return std::io::Error::other(error.0); })?;
    let staged: PathBuf = staged_path_for(lock, seq, token.as_str());
    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&staged)?;
    if let Err(error) = file.write_all(marker.as_bytes()) {
        drop(file);
        let _removed = std::fs::remove_file(&staged);
        return Err(error);
    }
    return Ok(staged);
}

/// Renames `from` to `to` unless `to` exists: one step that is as exclusive as `create_new` and leaves no second name.
#[cfg(target_os = "linux")]
pub fn rename_noreplace(from: &Path, to: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source = std::ffi::CString::new(from.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let target = std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    // SAFETY: both strings are valid NUL-terminated paths for the call.
    let done: i32 = unsafe { libc::renameat2(libc::AT_FDCWD, source.as_ptr(), libc::AT_FDCWD, target.as_ptr(), libc::RENAME_NOREPLACE) };
    return if done == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) };
}

#[cfg(not(target_os = "linux"))]
pub fn rename_noreplace(_from: &Path, _to: &Path) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
}

/// Whether a failed rename without replacing means the filesystem cannot do one (renameat2(2): `EINVAL`), not that the lock
///  exists or storage failed.
fn rename_unsupported(error: &std::io::Error) -> bool {
    if error.kind() == std::io::ErrorKind::Unsupported {
        return true;
    }
    #[cfg(unix)]
    {
        return matches!(error.raw_os_error(), Some(code) if code == libc::EINVAL || code == libc::ENOSYS || code == libc::EOPNOTSUPP);
    }
    #[cfg(not(unix))]
    {
        return false;
    }
}

/// Whether a failed hard link means the filesystem makes none (link(2): `EPERM`), not that the lock exists or storage failed.
fn links_unsupported(error: &std::io::Error) -> bool {
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

/// What:
///  Makes the marked `staged` file the lock at `lock`; `AlreadyExists` when any process holds it, and `staged` then stays for
///  the next attempt.
/// Why:
///   Git's lock is taken by exclusive creation, and a restart tells the service's own stale lock from a foreign one only by
///  its marker, so the lock must never exist without it. A rename that does not replace is exclusive and atomic and leaves
///  no second name; where the filesystem has none, a hard link is exclusive too and its staging name is removed after.
///  Where it has neither, nothing is created: exclusive creation followed by a write would leave an empty lock for as long
///  as a kill between the two lasts, and an empty lock may be a Git mid-write, so no restart could take it over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await renameNoReplace(staged, lock) ?? await link(staged, lock) ?? refuse()
/// ```
///
/// # Errors
///
/// `AlreadyExists` when the lock exists, `Unsupported` (naming both refusals) when the filesystem can place neither way, and
///  the storage error otherwise.
pub fn place_marked(platform: &Platform, staged: &Path, lock: &Path) -> std::io::Result<()> {
    let refused_rename: std::io::Error = match (platform.rename)(staged, lock) {
        Ok(()) => return Ok(()),
        Err(error) if !rename_unsupported(&error) => return Err(error),
        Err(error) => error,
    };
    match (platform.link)(staged, lock) {
        Ok(()) => {
            let _removed = std::fs::remove_file(staged);
            return Ok(());
        }
        Err(error) if !links_unsupported(&error) => return Err(error),
        Err(refused_link) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                format!("renaming without replacing failed ({refused_rename}) and hard links failed ({refused_link})"),
            ));
        }
    }
}

/// What:
///  Says in the service's log why a lock was not placed: the lock, the cause, and for a filesystem that can place a lock in
///  neither atomic way, the filesystem and what to do.
/// Why:
///   The clients are answered with a code only; the person who reads the log must find the filesystem and the remedy there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.error(`cli-git-service: could not place ${lock}: ${error}`)
/// ```
pub fn log_placement_failure(lock: &Path, error: &std::io::Error) {
    if error.kind() != std::io::ErrorKind::Unsupported {
        eprintln!("cli-git-service: cannot take the index lock {}: {error}", lock.display());
        return;
    }
    let directory: &Path = lock.parent().unwrap_or(Path::new("."));
    let filesystem: String = match read_mount_facts(directory) {
        Ok(Some(facts)) => format!("the {} filesystem mounted at {}", facts.filesystem_type, facts.mount_point.display()),
        Ok(None) => format!("the filesystem of {}", directory.display()),
        Err(reason) => format!("the filesystem of {} ({reason})", directory.display()),
    };
    eprintln!(
        "cli-git-service: cannot take the index lock {}: {filesystem} can neither rename a file without replacing another nor make a hard link ({error}), \
         so the lock cannot be created with its marker in one step, and the service takes no lock that a kill could leave empty. \
         Keep this repository's Git directory on a filesystem that supports one of the two (ext4, xfs, btrfs, tmpfs and overlayfs do), or update the kernel.",
        lock.display()
    );
}

/// Whether the process that a staging file's name carries is gone. A name that carries none, and a process that cannot be
///  asked about, count as running, which keeps the file.
fn staging_writer_is_gone(suffix: &[u8]) -> bool {
    if suffix.is_empty() {
        // The fixed name an earlier release used: no attempt of this or the last release writes it.
        return true;
    }
    let digits: Vec<u8> = suffix.strip_prefix(b"-").unwrap_or(&[]).iter().copied().take_while(u8::is_ascii_digit).collect();
    let Some(pid) = std::str::from_utf8(digits.as_slice()).ok().and_then(|text| { return text.parse::<i64>().ok(); }) else { return false };
    return matches!(process_birth_identity(pid), Ok(None));
}

/// What:
///  Removes the staging files of the lock at `lock` that a killed attempt left: the fixed name an earlier release used, and
///  any whose writer is gone.
/// Why:
///   A kill before the rename leaves the staging file behind. The writer is read from the name, never from the content: a
///  file that is empty or half written is the file of a writer that is still at it, and removing it fails that writer's
///  placement. A live writer's file is its own and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(dir)) if (name.startsWith(prefix) && writerIsGone(name)) await rm(name)
/// ```
pub fn remove_dead_staging(lock: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let (Some(directory), Some(file)) = (lock.parent(), lock.file_name()) else { return };
    let mut prefix: Vec<u8> = file.as_bytes().to_vec();
    prefix.extend_from_slice(STAGED_INFIX.as_bytes());
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(suffix) = name.as_bytes().strip_prefix(prefix.as_slice()) else { continue };
        if staging_writer_is_gone(suffix) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

/// Removes the stale `lock`, whose holder is gone. False, after saying why in the log, when it is still there; a lock that
///  is already gone counts as removed.
pub fn reclaim(platform: &Platform, lock: &Path) -> bool {
    return match (platform.unlink)(lock) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => {
            eprintln!(
                "cli-git-service: the index lock {} was left by a service that is gone, and it cannot be removed ({error}); \
                 remove it by hand, and clear its immutable attribute or fix the directory's permissions first if that is why it stays",
                lock.display()
            );
            false
        }
    };
}
