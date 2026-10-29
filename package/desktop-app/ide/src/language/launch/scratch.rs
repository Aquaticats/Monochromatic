//! The private temporary directory of one process for one server.
//!
//! Two windows (processes) of the application on one project share the private state folder of each server,
//!  so a temporary directory named only for the server was emptied by the second window's launch
//!  while the first window's running server had it as `/tmp`.
//! Each process now names its own directory `tmp-<process id>-<hex nonce>` beside the server's other state,
//!  holds an exclusive advisory lock on `tmp-<...>.lock` beside it for as long as it lives,
//!  and removes the directories of other processes whose lock nobody holds any more.
//! The system releases a lock when its process ends however it ends
//!  (exit,
//!  crash,
//!  `kill -9`),
//!  and a process id alone would be reused;
//!  the nonce keeps two live processes apart even when two process-id namespaces share one cache folder.

/// What:
///  `File` is an open file;
///  `TryLockError` tells a lock held by another from a failed system call.
///  `Mutex` guards the locks this process holds;
///  `OnceLock` computes the process's name once.
/// Why:
///  A lock lasts as long as the open file,
///  so the files are kept in a static for the life of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { open } from 'node:fs/promises'; // a flock held until the handle closes
/// ```
use std::{
    fs::{File, OpenOptions, TryLockError},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock, PoisonError},
    time::{SystemTime, UNIX_EPOCH},
};

/// The start of every scratch directory name.
const PREFIX: &str = "tmp-";

/// The name of this process's scratch directory,
///  made on first use.
static NAME: OnceLock<String> = OnceLock::new();

/// The scratch directories this process holds the lock of,
///  with the open lock file.
///  Closing a file would release its lock,
///  so none is ever removed.
static HELD: Mutex<Vec<(PathBuf, File)>> = Mutex::new(Vec::new());

/// What:
///  The name of this process's scratch directory below a server's private state:
///  `tmp-<process id>-<hex nanoseconds since the epoch at first use>`.
/// Why:
///  A name that differs for every live process is what keeps two windows on one project from sharing a
///  directory;
///  the clock reading separates two live processes with one id in different process-id namespaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const scratchName = (): string // the same string on every call within one process
/// ```
pub fn scratch_name() -> &'static str {
    return NAME.get_or_init(|| {
        let nanoseconds = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(since) => since.as_nanos(),
            // A clock before the epoch still gives a valid name: the id alone separates live processes of one namespace.
            Err(_) => 0,
        };
        return format!("{PREFIX}{}-{nanoseconds:x}", std::process::id());
    });
}

/// What:
///  True for a name this module makes:
///  `tmp-`,
///  decimal digits,
///  `-`,
///  hexadecimal digits.
/// Why:
///  Only names of this shape are claimed or removed,
///  so the shared `tmp` directory of earlier versions
///  (possibly the `/tmp` of a running older process) and any other entry stay as they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isProcessName = (name: string) => /^tmp-[0-9]+-[0-9a-f]+$/i.test(name);
/// ```
fn is_process_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix(PREFIX) else {
        return false;
    };
    let Some((id, nonce)) = rest.split_once('-') else {
        return false;
    };
    return !id.is_empty()
        && id.bytes().all(|byte| {
            return byte.is_ascii_digit();
        })
        && !nonce.is_empty()
        && nonce.bytes().all(|byte| {
            return byte.is_ascii_hexdigit();
        });
}

/// True when the last component of a path is a name this module makes.
fn is_process_directory(directory: &Path) -> bool {
    let Some(name) = directory.file_name().and_then(std::ffi::OsStr::to_str) else {
        return false;
    };
    return is_process_name(name);
}

/// What:
///  The lock file of a scratch directory,
///  beside it.
/// Why:
///  Beside the directory,
///  not inside,
///  so emptying the directory never removes the lock and the sandbox's `/tmp` never shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lockPath = (directory: string) => directory + '.lock';
/// ```
pub(super) fn lock_path(directory: &Path) -> PathBuf {
    let mut name = directory.file_name().unwrap_or_default().to_os_string();
    name.push(".lock");
    return directory.with_file_name(name);
}

/// What:
///  Take the right to use a scratch directory:
///  an exclusive lock on its lock file,
///  held until this process ends.
///  A directory whose name is not a process name,
///  or one this process already holds,
///  needs nothing.
/// Why:
///  The lock is what tells another process that this one is alive.
///  Refusing when another process holds it
///  (a name collision across process-id namespaces that the nonce makes practically impossible)
///  is safer than sharing the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claim(directory: string): void // throws a reason
/// ```
pub(super) fn claim(directory: &Path) -> Result<(), String> {
    if !is_process_directory(directory) {
        return Ok(());
    }
    let mut held = HELD.lock().unwrap_or_else(PoisonError::into_inner);
    for (path, _) in held.iter() {
        if path == directory {
            return Ok(());
        }
    }
    let lock = lock_path(directory);
    let file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock)
    {
        Ok(file) => file,
        Err(error) => return Err(format!("cannot create {}: {error}", lock.display())),
    };
    match file.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => {
            return Err(format!(
                "{} is locked by another running process, so its scratch directory is not shared; restart the application",
                lock.display()
            ));
        }
        Err(TryLockError::Error(error)) => {
            return Err(format!("cannot lock {}: {error}", lock.display()));
        }
    }
    held.push((directory.to_path_buf(), file));
    return Ok(());
}

/// What:
///  Remove the scratch directories of ended processes that lie beside a claimed one,
///  with their lock files.
///  A directory whose lock is held stays;
///  so does everything that is not named like a process's scratch directory.
///  A removal that fails is logged and does not stop the launch.
/// Why:
///  A process that was killed cannot clean up after itself,
///  and nothing else removes what its server left in `/tmp`.
///  The lock is taken to remove,
///  so two processes removing one directory cannot both succeed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reapEnded(claimed: string): void // logs failures
/// ```
pub(super) fn reap_ended(directory: &Path) {
    if !is_process_directory(directory) {
        return;
    }
    let Some(parent) = directory.parent() else {
        return;
    };
    let entries = match std::fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(parent = %parent.display(), %error, "cannot list the scratch directories beside this process's");
            return;
        }
    };
    for listed in entries {
        let entry = match listed {
            Ok(entry) => entry,
            Err(error) => {
                tracing::warn!(parent = %parent.display(), %error, "cannot read an entry beside this process's scratch directory");
                continue;
            }
        };
        let name = entry.file_name();
        if directory.file_name() == Some(name.as_os_str()) {
            continue;
        }
        let Some(text) = name.to_str() else {
            continue;
        };
        if !is_process_name(text) {
            continue;
        }
        // A link or a file with such a name is not a directory of this module: left as it is.
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => {}
            _ => continue,
        }
        remove_if_owner_ended(&entry.path());
    }
}

/// What:
///  Remove one scratch directory and its lock file when no process holds the lock.
///  A missing lock file means the same:
///  the owner makes the lock file before the directory.
/// Why:
///  The lock held by the removing process excludes a second remover,
///  and a live owner's held lock keeps the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeIfOwnerEnded(directory: string): void // logs failures
/// ```
fn remove_if_owner_ended(directory: &Path) {
    let lock = lock_path(directory);
    match File::open(&lock) {
        Ok(file) => match file.try_lock() {
            Ok(()) => {
                remove_directory(directory);
                if let Err(error) = std::fs::remove_file(&lock)
                    && error.kind() != std::io::ErrorKind::NotFound
                {
                    tracing::warn!(lock = %lock.display(), %error, "cannot remove the lock file of an ended process");
                }
            }
            Err(TryLockError::WouldBlock) => {
                tracing::trace!(directory = %directory.display(), "scratch directory of a running process kept");
            }
            Err(TryLockError::Error(error)) => {
                tracing::warn!(lock = %lock.display(), %error, "cannot test the lock of a scratch directory; it is kept");
            }
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => remove_directory(directory),
        Err(error) => {
            tracing::warn!(lock = %lock.display(), %error, "cannot open the lock file of a scratch directory; it is kept");
        }
    }
}

/// Remove a directory of an ended process,
///  logging the outcome.
fn remove_directory(directory: &Path) {
    match std::fs::remove_dir_all(directory) {
        Ok(()) => {
            tracing::debug!(directory = %directory.display(), "removed the scratch directory of an ended process");
        }
        // Another process removed it at the same time.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            tracing::warn!(directory = %directory.display(), %error, "cannot remove the scratch directory of an ended process");
        }
    }
}

/// The ownership and cleanup rules of the scratch directories.
#[cfg(test)]
#[path = "scratch_tests.rs"]
mod tests;
