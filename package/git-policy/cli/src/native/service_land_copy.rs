//! What:
//!  The landing's private copy of the live index: a directory this landing made for itself beside the live index, holding the copy
//!  and a marker that names the service that owns it.
//! Why:
//!  Every name beside the live index can be another program's (a client's `GIT_INDEX_FILE`, a lock of another client), so a copy
//!  kept under a name there can be taken, shared, or run out of names. A directory with a random name is made exclusively, and
//!  everything the landing builds lives inside it: Git's own lock for the copy, the in-process update's temporary file, and the
//!  copy. It is made owner-only before the first byte goes in, it has the same file system as the live index so that one rename
//!  installs the copy, and it is removed when the value is dropped. The owner holds an exclusive `flock` on the directory for as
//!  long as the value lives, and the kernel drops it when the owner dies, so a directory a service that died left is the one
//!  nobody holds: no process number, boot or PID namespace is asked, which a recycled number, a container or a service killed
//!  before it wrote a marker would answer wrongly.
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
use std::os::unix::fs::{DirBuilderExt as _, OpenOptionsExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};

/// The start of the name of a private copy's directory; a random identifier follows it.
const COPY_PREFIX: &str = "cli-git-private-";

/// The file in a private copy's directory that says whose it is.
const MARKER_FILE: &str = "owner";

/// The first line of the marker, which says whose the directory is to a person reading it; the later lines name the process
///  and the batch. Nothing decides on it: the hold on the directory does.
const MARKER_HEAD: &str = "cli-git-private-copy v1";

/// The name of the copy of the index inside the directory.
const COPY_FILE: &str = "index";

/// How many random names are tried for the directory; a name is taken only by the astronomically unlikely repeat of a 122-bit
///  random identifier, so one try is the measure and a few more cover a hostile directory.
const DIRECTORY_TRIES: usize = 4;

/// Takes the exclusive hold on a directory, without waiting: the open directory when it was taken, nothing when another holder has it
///  or the file system cannot say.
fn hold(directory: &Path) -> Option<std::fs::File> {
    let opened: std::fs::File = std::fs::File::open(directory).ok()?;
    return match opened.try_lock() {
        Ok(()) => Some(opened),
        Err(std::fs::TryLockError::WouldBlock) => None,
        Err(std::fs::TryLockError::Error(error)) => {
            crate::service_log!("the private copy {} cannot be tested for an owner ({error}), so it stays", directory.display());
            None
        }
    };
}

/// Whether `entry` has the name, the kind, the owner and the mode of a directory this service makes.
fn is_ours(entry: &std::fs::DirEntry) -> bool {
    use std::os::unix::fs::MetadataExt as _;
    let name = entry.file_name();
    let Some(tail) = name.to_str().and_then(|text| { return text.strip_prefix(COPY_PREFIX); }) else {
        return false;
    };
    let Ok(metadata) = entry.path().symlink_metadata() else {
        return false;
    };
    // SAFETY: `geteuid` takes no argument and cannot fail.
    let effective: u32 = unsafe { libc::geteuid() };
    return is_canonical_uuid(tail) && metadata.is_dir() && metadata.uid() == effective && std::os::unix::fs::PermissionsExt::mode(&metadata.permissions()) & 0o777 == 0o700;
}

/// What:
///  Removes the private copies in `directory` that a service which has since died left behind.
/// Why:
///   A copy is removed by its owner's drop, so only a kill leaves one. The landing that holds the live index's lock sweeps the
///  directory the copies live in. A directory is removed only when it has this service's name, owner and mode and the sweep can
///  take the hold its owner keeps for as long as it lives: a held directory is another landing's, which may serve another index
///  in the same directory. The sweep removes the directory while it holds it, and an owner that finds its new directory held
///  or gone makes another.
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
        if !is_ours(&entry) {
            continue;
        }
        if let Some(_held) = hold(entry.path().as_path()) {
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
    /// The directory, open and held exclusively with `flock` for as long as the value lives: the kernel releases the hold when
    ///  the owner dies, and a sweep removes only a directory it can hold. Nothing when the file system cannot hold one.
    _hold: Option<std::fs::File>,
    /// Whether the bytes at `path` are already on the disk and nothing has written to them since: the in-process update flushes
    ///  the file it renames onto the copy, so the release need not flush it again.
    flushed: std::cell::Cell<bool>,
}

impl PrivateCopy {
    /// Where the copy is.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// Whether the copy is known to be on the disk as it is.
    pub fn is_flushed(&self) -> bool {
        return self.flushed.get();
    }

    /// Says what is known of the copy's bytes: that they are on the disk (the update that wrote them flushed them before it
    ///  renamed them into place), or that something wrote to them after.
    pub fn set_flushed(&self, flushed: bool) {
        self.flushed.set(flushed);
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
        let (directory, held) = Self::make_directory(parent)?;
        let copy: PrivateCopy = PrivateCopy { path: directory.join(COPY_FILE), directory, _hold: held, flushed: std::cell::Cell::new(false) };
        let marker_text: String = format!("{MARKER_HEAD}\npid={}\nbatch={seq}\n", std::process::id());
        write_new(copy.directory.join(MARKER_FILE).as_path(), marker_text.as_bytes()).map_err(|error| { return storage_failure(&error); })?;
        match std::fs::File::open(live) {
            Ok(mut source) => {
                let live_permissions: std::fs::Permissions = source.metadata().map_err(|error| { return storage_failure(&error); })?.permissions();
                let mut made: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(copy.path.as_path()).map_err(|error| { return storage_failure(&error); })?;
                std::io::copy(&mut source, &mut made).map_err(|error| { return storage_failure(&error); })?;
                // No flush here: every writer of the copy replaces it by rename, and the copy that is installed is flushed by the
                //  release of the lock, after its last writer (`service_index_hold.rs`).
                std::fs::set_permissions(copy.path.as_path(), live_permissions).map_err(|error| { return storage_failure(&error); })?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(storage_failure(&error)),
        }
        return Ok(copy);
    }

    /// Makes the owner-only directory under a random name that did not exist, and takes the hold on it.
    ///  A directory that a sweep holds or has already removed is given up for another name.
    fn make_directory(parent: &Path) -> Result<(PathBuf, Option<std::fs::File>), FailureCode> {
        for _ in 0..DIRECTORY_TRIES {
            let identifier: String = random_uuid().map_err(|_| { return FailureCode::Unavailable; })?;
            let directory: PathBuf = parent.join(format!("{COPY_PREFIX}{identifier}"));
            match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(storage_failure(&error)),
            }
            // A mask that narrows the owner's own bits is not this service's to honour: the sweep knows its directories by 0700.
            match std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(storage_failure(&error)),
            }
            let opened: std::fs::File = match std::fs::File::open(&directory) {
                Ok(found) => found,
                // A sweep that held the new directory has already removed it.
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(storage_failure(&error)),
            };
            match opened.try_lock() {
                Ok(()) => return Ok((directory, Some(opened))),
                Err(std::fs::TryLockError::WouldBlock) => continue,
                Err(std::fs::TryLockError::Error(error)) => {
                    crate::service_log!("the private copy {} cannot be held with flock ({error}); a later sweep leaves it", directory.display());
                    return Ok((directory, None));
                }
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
