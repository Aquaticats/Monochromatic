//! What:
//!  Private,
//!  crash-safe file primitives shared by every durable record of the wrapper:
//!       exclusive no-follow creation with private modes,
//!  file and directory sync,
//!  and
//!       no-follow reads that refuse links and non-regular files.
//! Why:
//!  Lock records,
//!  transaction journals and capture records must survive a crash in a
//!      readable state,
//!  must never be written through a planted link,
//!  and must be private to
//!      the account.
//!  The incumbent's `src/trust/registry-io.ts` defines these semantics;
//!  both
//!      wrappers write and read the same files.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writePrivateFile({ path, bytes }); await syncDirectory(dirname(path));
//! ```

/// What:
///  `File` is an open file handle;
///  `OpenOptions` configures how one is opened.
/// Why:
///   Exclusive creation and no-follow opening need the configurable form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { open, constants } from 'node:fs/promises';
/// ```
use std::fs::{File, OpenOptions};
/// The trait that gives files `.read_to_end` and `.take`.
use std::io::Read;
/// The trait that gives files `.write_all`.
use std::io::Write;
/// Owned operating-system text for the staging name.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path of raw bytes; `PathBuf` is the owned one.
use std::path::{Path, PathBuf};
/// Diagnostics for a decision no failure reports.
use super::diagnostic_log::debug;
/// Unguessable staging names.
use super::random_id::random_uuid;

/// Mode of every private directory the wrapper creates:
///  owner only.
pub const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

/// Mode of every private file the wrapper creates:
///  owner read and write only.
pub const PRIVATE_FILE_MODE: u32 = 0o600;

/// What:
///  Why a no-follow read refused a path.
///  An `enum` is a closed set of alternatives.
/// Why:
///   A missing file is an expected answer for optional records;
///  a link,
///  a directory or an
///       over-long file is unsafe or corrupt state that recovery must name,
///  never follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadRefusal = { kind: 'missing' } | { kind: 'not-regular' } | { kind: 'too-large' } | { kind: 'io'; error: Error };
/// ```
#[derive(Debug)]
pub enum ReadRefusal {
    /// Nothing exists at the path.
    Missing,
    /// The path names a link,
    ///  directory or other non-regular file.
    NotRegular,
    /// The file holds more bytes than the caller allows.
    TooLarge,
    /// The operating system refused the read for another reason.
    Io(
        /// The operating-system error that refused the read.
        std::io::Error,
    ),
}

/// What:
///  Open options for an exclusive,
///  no-follow,
///  private-mode creation.
///       `#[cfg(unix)]` compiles the block only on Unix-like systems.
/// Why:
///   `O_EXCL` makes a second writer fail instead of replacing the record;
///  `O_NOFOLLOW`
///       refuses a planted final link;
///  mode `0600` keeps the record private from the start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// open(path, O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW, 0o600)
/// ```
fn exclusive_options() -> OpenOptions {
    // `mut` allows the builder to be configured step by step.
    let mut options: OpenOptions = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        // The trait adds `.mode` and `.custom_flags` to open options on Unix.
        /// The Unix open-option extensions,
        ///  for the mode and no-follow flags.
        use std::os::unix::fs::OpenOptionsExt;
        options
            .mode(PRIVATE_FILE_MODE)
            .custom_flags(libc::O_NOFOLLOW);
    }
    return options;
}

/// What:
///  Set a path's permission bits to `mode`,
///  on Unix only.
///       `std::io::Result<()>` is "nothing,
///  or an operating-system error".
/// Why:
///   The creation mode is filtered by the process `umask`;
///  an explicit change makes the
///       mode exactly private whatever the caller's `umask` is,
///  as the incumbent's
///       `protectPath` does.
///  Windows keeps its inherited access rules (recorded under
///       "Choices open to veto" in `doc/handover/cli-git-native-transactions.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.platform !== 'win32') await chmod(path, mode);
/// ```
pub fn protect_path(path: &Path, mode: u32) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        // The trait adds `.from_mode` to permissions on Unix.
        /// The Unix permission extensions,
        ///  for the mode.
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    }
    // `let _ = mode;` marks the argument as used where no mode exists.
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    let _ = (path, mode);
    return Ok(());
}

/// What:
///  The name of the staging file beside `path`: hidden, unique, ending in `.partial`.
/// Why:
///   Recovery finds records by their exact names, so a staging file never looks like one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(dirname(path), `.${basename(path)}.${randomUUID()}.partial`)
/// ```
fn staging_path(path: &Path) -> std::io::Result<PathBuf> {
    let Some(name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} names no file", path.display()),
        ));
    };
    let unique: String = random_uuid().map_err(|error| { return std::io::Error::other(error.0); })?;
    let mut staged: OsString = OsString::from(".");
    staged.push(name);
    staged.push(".");
    staged.push(unique);
    staged.push(".partial");
    return Ok(path.with_file_name(staged));
}

/// What:
///  Make `staged` appear at `path` in one step, failing with `AlreadyExists` when `path` is taken.
/// Why:
///   A hard link is atomic and exclusive. A filesystem that refuses hard links (`EPERM`, `ENOTSUP`, `EXDEV`) gets the
///  rename after a check that nothing is at `path`, which is atomic and, with the writers of one record being
///  one process, as exclusive as the callers need.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await link(staged, path); } catch (error) { if (error.code === 'EEXIST') throw error; await rename(staged, path); }
/// ```
fn publish_exclusive(staged: &Path, path: &Path) -> std::io::Result<()> {
    match std::fs::hard_link(staged, path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Err(error),
        Err(error) => {
            debug(
                "publishExclusive",
                format!("the filesystem refused the link of {} ({error}); publishing by rename", path.display()).as_str(),
            );
        }
    }
    if std::fs::symlink_metadata(path).is_ok() {
        return Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
    }
    return std::fs::rename(staged, path);
}

/// What:
///  Write `bytes` to a staging file beside `path`, sync it, and publish it at `path`; any failure removes the staging
///  file, so `path` holds all the bytes or does not exist.
///  `mode` is applied to the file before it is published, when given.
/// Why:
///   A file created at its final name and then written is a record that a kill, or a full disk, between the two
///  steps leaves empty or cut: recovery then stops every guarded command until a person removes it. Publishing a
///  complete, synced file by link makes the record appear whole or not at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = stagingPath(path); try { await writeAndSync(staged, bytes); await publishExclusive(staged, path); } finally { await rm(staged, { force: true }); }
/// ```
fn write_then_publish(path: &Path, bytes: &[u8], mode: Option<u32>) -> std::io::Result<()> {
    let staged: PathBuf = staging_path(path)?;
    let mut file: File = exclusive_options().open(staged.as_path())?;
    // `sync_all` flushes data and metadata, the incumbent's `handle.sync()`.
    let filled: std::io::Result<()> = file.write_all(bytes).and_then(|()| { return file.sync_all(); });
    drop(file);
    let published: std::io::Result<()> = filled
        .and_then(|()| {
            return match mode {
                Some(bits) => protect_path(staged.as_path(), bits),
                None => Ok(()),
            };
        })
        .and_then(|()| { return publish_exclusive(staged.as_path(), path); });
    match std::fs::remove_file(staged.as_path()) {
        Ok(()) => {}
        // The rename fallback already moved it.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            debug(
                "writeThenPublish",
                format!("removing the staging file {} failed: {error}", staged.display()).as_str(),
            );
        }
    }
    return published;
}

/// What:
///  Write one private file exclusively,
///  refusing links,
///  then make its bytes durable.
///       `&[u8]` borrows the exact bytes.
/// Why:
///   A journal state file is created once and never rewritten,
///  so a crash leaves either
///       no file or a complete one after the caller syncs the directory.
///  The file is written under a staging name and published whole (`write_then_publish`), so that the promise
///  also holds for a kill between the creation and the write and for a write cut short by a full disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writePrivateFile({ path, bytes }): Promise<void>;
/// ```
pub fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    return write_then_publish(path, bytes, Some(PRIVATE_FILE_MODE));
}

/// What:
///  Write one file exclusively with mode `0600`,
///  refusing links,
///  and sync its bytes,
///       without the explicit mode change.
/// Why:
///   Owner-lock candidates are written this way by the incumbent's `writeCandidate`:
///  the
///       creation mode alone,
///  then a sync before publication.
///  The file is published whole, as `write_private_file` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(path, O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW, 0o600); await handle.writeFile(bytes); await handle.sync();
/// ```
pub fn write_exclusive_synced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    return write_then_publish(path, bytes, None);
}

/// What:
///  Write one file exclusively with mode `0600`,
///  refusing links,
///  without syncing it.
/// Why:
///   Scratch bytes that are unlinked right after they are opened for reading are never recovered,
///  so a sync would only make every caller wait for the disk;
///  records that a crash must find stay on `write_exclusive_synced`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(path, O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW, 0o600); await handle.writeFile(bytes);
/// ```
pub fn write_exclusive_scratch(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file: File = exclusive_options().open(path)?;
    return file.write_all(bytes);
}

/// What:
///  Create one directory with mode `0700`;
///  its parent must exist.
/// Why:
///   Lock candidates,
///  transaction directories and stores are private to the account.
///       Only the creation mode is applied,
///  as the incumbent's `mkdir(path, { mode })`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { mode: 0o700 });
/// ```
pub fn create_private_directory(path: &Path) -> std::io::Result<()> {
    return private_directory_builder().create(path);
}

/// What:
///  Create a directory and every missing parent with mode `0700`;
///  an existing directory is accepted.
/// Why:
///   The per-user state directory may not exist yet,
///  and the XDG rules ask for `0700` when a program creates one.
///  Only the directories this call creates get the mode;
///  existing parents keep theirs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
pub fn create_private_directory_all(path: &Path) -> std::io::Result<()> {
    // `mut` allows the builder to be switched to recursive creation.
    let mut builder: std::fs::DirBuilder = private_directory_builder();
    builder.recursive(true);
    return builder.create(path);
}

/// What:
///  The directory builder that creates private directories on this platform.
/// Why:
///   Only Unix configures a mode;
///  a separate function per platform keeps each body free of
///       a `mut` the other platform would leave unused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const builder = { mode: 0o700 };
/// ```
#[cfg(unix)]
fn private_directory_builder() -> std::fs::DirBuilder {
    // The trait adds `.mode` to directory builders on Unix.
    /// The Unix directory-builder extensions,
    ///  for the mode.
    use std::os::unix::fs::DirBuilderExt;
    // `mut` allows the builder to be configured before it is returned.
    let mut builder: std::fs::DirBuilder = std::fs::DirBuilder::new();
    builder.mode(PRIVATE_DIRECTORY_MODE);
    return builder;
}

/// What:
///  The directory builder that creates private directories on this platform.
/// Why:
///   Windows has no mode to configure,
///  as `private_directory_builder` on Unix explains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const builder = {};
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn private_directory_builder() -> std::fs::DirBuilder {
    return std::fs::DirBuilder::new();
}

/// What:
///  Whether a failed directory sync means the filesystem has no such operation, not that the flush failed.
/// Why:
///   `EINVAL`, `ENOTSUP` and `ENOSYS` are what a filesystem without the operation returns; `EIO`, `ENOSPC` and
///  `EDQUOT` mean the flush itself failed and stay errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ['EINVAL', 'ENOTSUP', 'ENOSYS'].includes(error.code)
/// ```
#[cfg(unix)]
fn directory_sync_is_unsupported(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(libc::EINVAL | libc::ENOTSUP | libc::ENOSYS));
}

/// What:
///  Make a directory's entries durable,
///  where the platform supports directory handles.
/// Why:
///   A new or renamed entry survives a crash only after its directory is synced;
///  Windows
///       has no directory handle for this,
///  as the incumbent's `syncDirectory` notes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function syncDirectory(path: string): Promise<void>;
/// ```
pub fn sync_directory(path: &Path) -> std::io::Result<()> {
    #[cfg(test)]
    record_directory_sync(path);
    #[cfg(unix)]
    {
        // Opening a directory read-only and syncing it is the POSIX way to flush its entries.
        let directory: File = File::open(path)?;
        match directory.sync_all() {
            Ok(()) => {}
            // A filesystem with no directory flush (procfs, some network and userspace filesystems) answers with one
            // of these; the entry cannot be made durable there by any means, and failing would stop every commit.
            Err(error) if directory_sync_is_unsupported(&error) => {
                debug(
                    "syncDirectory",
                    format!("the filesystem of {} cannot sync a directory ({error}); skipped", path.display()).as_str(),
                );
            }
            Err(error) => return Err(error),
        }
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    let _ = path;
    return Ok(());
}

#[cfg(test)]
thread_local! {
    /// Canonical paths of the directories this thread asked `sync_directory` to flush, in call order.
    static DIRECTORY_SYNCS: std::cell::RefCell<Vec<std::path::PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// What:
///  Remember that this thread asked for a directory sync.
/// Why:
///   Unit tests prove that a helper syncs a directory exactly when it changed one, which no file on disk shows;
///  the list is per thread, so parallel tests never see each other's syncs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// syncedDirectories.push(await realpath(path));
/// ```
#[cfg(test)]
fn record_directory_sync(path: &Path) {
    let canonical: std::path::PathBuf = std::fs::canonicalize(path).unwrap_or_else(|_| return path.to_path_buf());
    DIRECTORY_SYNCS.with(|syncs| {
        syncs.borrow_mut().push(canonical);
    });
}

/// What:
///  Take and clear the directories this thread has synced so far.
/// Why:
///   A test calls it before the step under test to drop earlier syncs, and after it to read exactly that step's syncs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const synced = syncedDirectories.splice(0);
/// ```
#[cfg(test)]
pub fn take_directory_syncs() -> Vec<std::path::PathBuf> {
    return DIRECTORY_SYNCS.with(|syncs| {
        return std::mem::take(&mut *syncs.borrow_mut());
    });
}

/// What:
///  Remove a directory tree,
///  treating an already absent tree as removed.
/// Why:
///   Retired locks and transactions are removed by name after a rename;
///  another process
///       may have removed them first,
///  which is the same end state (`rm -rf` with `force`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(path, { recursive: true, force: true });
/// ```
pub fn remove_tree(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
}

/// What:
///  Open a file for reading without following a final link.
/// Why:
///   Records are always files the wrappers created;
///  a link planted at a record path must
///       be refused,
///  never read through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await open(path, O_RDONLY | O_NOFOLLOW);
/// ```
fn open_no_follow(path: &Path) -> std::io::Result<File> {
    // `mut` allows the builder to be configured step by step.
    let mut options: OpenOptions = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        // The trait adds `.custom_flags` to open options on Unix.
        /// The Unix open-option extensions,
        ///  for the no-follow flag.
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    return options.open(path);
}

/// What:
///  Read a whole regular file without following links,
///  up to `limit` bytes.
///       `u64` is an unsigned 64-bit size;
///  `Result<Vec<u8>, ReadRefusal>` is "the bytes,
///  or why
///       not".
/// Why:
///   Recovery reads untrusted state:
///  a link or a directory must be named as unsafe,
///  and a
///       runaway file must not be read into memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readPrivateFile(path: string, limit: number): Promise<Uint8Array>; // throws ReadRefusal
/// ```
pub fn read_regular_file(path: &Path, limit: u64) -> Result<Vec<u8>, ReadRefusal> {
    let file: File = match open_no_follow(path) {
        Ok(opened) => opened,
        Err(error) => return Err(open_refusal(error)),
    };
    // The handle's own metadata, so a swap between the check and the read is impossible.
    let metadata: std::fs::Metadata = file.metadata().map_err(ReadRefusal::Io)?;
    if !metadata.is_file() {
        return Err(ReadRefusal::NotRegular);
    }
    if metadata.len() > limit {
        return Err(ReadRefusal::TooLarge);
    }
    // `.take(limit + 1)` caps the read even if the file grows after the size check.
    let mut bytes: Vec<u8> = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(ReadRefusal::Io)?;
    if bytes.len() as u64 > limit {
        return Err(ReadRefusal::TooLarge);
    }
    return Ok(bytes);
}

/// What:
///  Classify an open failure as missing,
///  a refused link,
///  or another error.
/// Why:
///   `O_NOFOLLOW` on a link fails with `ELOOP` on Linux and macOS;
///  that is the unsafe-link
///       answer,
///  not an input-output failure.
///  A path through a non-directory (`ENOTDIR`) names
///       no file,
///  which the incumbent treats like a missing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openRefusal(error: NodeJS.ErrnoException): ReadRefusal;
/// ```
fn open_refusal(error: std::io::Error) -> ReadRefusal {
    if error.kind() == std::io::ErrorKind::NotFound {
        return ReadRefusal::Missing;
    }
    #[cfg(unix)]
    {
        if error.raw_os_error() == Some(libc::ENOTDIR) {
            return ReadRefusal::Missing;
        }
        if error.raw_os_error() == Some(libc::ELOOP) {
            return ReadRefusal::NotRegular;
        }
    }
    return ReadRefusal::Io(error);
}

/// Disposable-directory controls stay out of the release executable.
#[cfg(test)]
#[path = "private_storage_tests.rs"]
mod tests;
