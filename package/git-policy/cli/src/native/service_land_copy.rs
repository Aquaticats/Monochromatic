//! What:
//!  The landing's private copy of the live index: a directory this landing made for itself beside the live index, holding the copy
//!  and a marker that names the service that owns it.
//! Why:
//!  Every name beside the live index can be another program's (a client's `GIT_INDEX_FILE`, a lock of another client), so a copy
//!  kept under a name there can be taken, shared, or run out of names. A directory with a random name is made exclusively, and
//!  everything the landing builds lives inside it: Git's own lock for the copy, the in-process update's temporary file, and the
//!  copy. It is made owner-only before the first byte goes in, it has the same file system as the live index so that one rename
//!  installs the copy, and it is removed when the value is dropped. A directory a service that died left is recognised by its
//!  name, its marker and the death of the process the marker names, never by the name alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const copy = await PrivateCopy.make(lock, seq) // mkdir beside the index, owner only; copy the index into it
//! ```

use super::random_id::{is_canonical_uuid, random_uuid};
use super::service_index_lock::IndexLock;
use super::service_land_follow::storage_failure;
use super::service_protocol::FailureCode;
use std::io::Write as _;
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _};
use std::path::{Path, PathBuf};

/// The start of the name of a private copy's directory; a random identifier follows it.
const COPY_PREFIX: &str = "cli-git-private-";

/// The file in a private copy's directory that says whose it is.
const MARKER_FILE: &str = "owner";

/// The first line of the marker; the second line is `pid=` and the process that made the copy.
const MARKER_HEAD: &str = "cli-git-private-copy v1";

/// The name of the copy of the index inside the directory.
const COPY_FILE: &str = "index";

/// How many random names are tried for the directory; a name is taken only by the astronomically unlikely repeat of a 122-bit
///  random identifier, so one try is the measure and a few more cover a hostile directory.
const DIRECTORY_TRIES: usize = 4;

/// Whether the process `pid` is alive; a process that cannot be asked about is taken as alive, so nothing is removed on a doubt.
fn process_alive(pid: i32) -> bool {
    if pid <= 0 {
        return true;
    }
    // SAFETY: signal 0 only checks that the process exists; no pointer is passed.
    let status: libc::c_int = unsafe { libc::kill(pid, 0) };
    return status == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH);
}

/// The process a marker names, when the text is exactly a marker.
fn marker_pid(text: &str) -> Option<i32> {
    let mut lines = text.lines();
    if lines.next()? != MARKER_HEAD {
        return None;
    }
    // Later lines say which batch made the copy, for a person reading the directory; only the process matters here.
    return lines.next()?.strip_prefix("pid=")?.parse().ok();
}

/// What:
///  Removes the private copies in `directory` that a service which has since died left behind.
/// Why:
///   A copy is removed by its owner's drop, so only a kill leaves one. The landing that holds the live index's lock sweeps the
///  directory the copies live in; a directory with the name pattern but no marker, a marker of a living process, and anything
///  that is not a directory are left as they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sweepLeftovers(dirname(liveIndex))
/// ```
fn sweep_leftovers(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(tail) = name.to_str().and_then(|text| { return text.strip_prefix(COPY_PREFIX); }) else {
            continue;
        };
        if !is_canonical_uuid(tail) || !entry.file_type().is_ok_and(|kind| { return kind.is_dir(); }) {
            continue;
        }
        let owner: Option<i32> = std::fs::read_to_string(entry.path().join(MARKER_FILE)).ok().and_then(|text| { return marker_pid(text.as_str()); });
        if owner.is_some_and(|pid| { return !process_alive(pid); }) {
            let _removed = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// What:
///  The landing's private copy of the live index.
/// Why:
///   Only a value of this type can be handed to the in-process update, so the update can work on no other file. The directory it
///  owns goes with the value, after the copy has been installed or abandoned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copy = await PrivateCopy.make(lock, seq) // throws when the directory cannot be made
/// ```
#[derive(Debug)]
pub struct PrivateCopy {
    /// The directory this landing made, beside the live index.
    directory: PathBuf,
    /// Where the copy is; no file exists there when the live index did not exist.
    path: PathBuf,
}

impl PrivateCopy {
    /// Where the copy is.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// The directory the copy lives in, beside the live index.
    pub fn directory(&self) -> &Path {
        return self.directory.as_path();
    }

    /// What:
    ///  Makes the directory and copies the live index into it, while the lock is held.
    /// Why:
    ///   The directory is owner-only from its creation and the copy is created owner-only, so no other user can read the index
    ///  while it is filled; the copy is then given the live index's own mode, which is the mode Git gave that file and the one an
    ///  index installed without a change keeps.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const copy = await PrivateCopy.make(lock, seq)
    /// ```
    ///
    /// # Errors
    ///
    /// The failure the storage error stands for when the directory or the copy cannot be made.
    pub fn make(lock: &IndexLock, seq: u64) -> Result<PrivateCopy, FailureCode> {
        let live: &Path = lock.index();
        let parent: &Path = live.parent().ok_or(FailureCode::Unavailable)?;
        sweep_leftovers(parent);
        let directory: PathBuf = Self::make_directory(parent)?;
        let copy: PrivateCopy = PrivateCopy { path: directory.join(COPY_FILE), directory };
        let marker_text: String = format!("{MARKER_HEAD}\npid={}\nbatch={seq}\n", std::process::id());
        write_new(copy.directory.join(MARKER_FILE).as_path(), marker_text.as_bytes()).map_err(|error| { return storage_failure(&error); })?;
        match std::fs::File::open(live) {
            Ok(mut source) => {
                let live_permissions: std::fs::Permissions = source.metadata().map_err(|error| { return storage_failure(&error); })?.permissions();
                let mut made: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(copy.path.as_path()).map_err(|error| { return storage_failure(&error); })?;
                std::io::copy(&mut source, &mut made).map_err(|error| { return storage_failure(&error); })?;
                made.sync_all().map_err(|error| { return storage_failure(&error); })?;
                std::fs::set_permissions(copy.path.as_path(), live_permissions).map_err(|error| { return storage_failure(&error); })?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(storage_failure(&error)),
        }
        return Ok(copy);
    }

    /// Makes the owner-only directory under a random name that did not exist.
    fn make_directory(parent: &Path) -> Result<PathBuf, FailureCode> {
        for _ in 0..DIRECTORY_TRIES {
            let identifier: String = random_uuid().map_err(|_| { return FailureCode::Unavailable; })?;
            let directory: PathBuf = parent.join(format!("{COPY_PREFIX}{identifier}"));
            match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
                Ok(()) => return Ok(directory),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(storage_failure(&error)),
            }
        }
        return Err(FailureCode::Unavailable);
    }
}

/// Writes a new file nobody else can open, failing when the name exists.
fn write_new(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
    return file.write_all(bytes);
}

impl AsRef<Path> for PrivateCopy {
    fn as_ref(&self) -> &Path {
        return self.path.as_path();
    }
}

impl Drop for PrivateCopy {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(self.directory.as_path());
    }
}
