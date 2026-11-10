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
//!  Each primitive is the one place that rule is written,
//!  so every record of the wrapper is created,
//!      synced and read the same way.
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
/// `Path` is a borrowed filesystem path of raw bytes.
use std::path::Path;
/// Diagnostics for a decision no failure reports.
use crate::diagnostic_log::debug;
/// The in-memory claim that makes the checked rename one step to the other threads of the process.
use crate::name_held_while_one_thread_publishes_it::{HeldName, hold_name_while_publishing};
use std::time::Duration;

/// How long the checked rename waits for another thread of this process that is publishing the same name:
/// the bound of "nothing
///  should ever wait forever" for a step that takes a check and a rename.
/// It is the owner's multiple of the longest delay measured
///  (`foundation/worst_delay_seen_on_this_host.rs`:
/// 6.03 s of wall time for a check and a rename,
/// once,
/// under a load average of about
///  70;
/// the owner,
/// 2026-11-07,
/// "3x worst case"),
/// so about 18.1 s.
/// Measurements:
/// `foundation/private_storage_hold_time_measurement_tests.rs`
///  (20,000 publishes:
/// median 181 us,
/// 99.9th percentile 312143 us;
/// three runs of 4,000:
/// longest 93 to 296227 us;
/// eight threads:
/// longest
///  453 to 1126 us;
/// a rerun in a scope capped at twenty-four processors:
/// longest 94 us).
/// Whether the disk or the scheduler caused the 6.03 s
///  is not measured.
/// Only a filesystem with neither exclusive publish reaches this wait,
/// and none was available to measure (an S3
///  bucket through `s3fs-fuse`).
const HELD_NAME_WAIT: Duration = crate::worst_delay_seen_on_this_host::WORST_CASE_WITH_MARGIN;

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

/// Create the file at `staged` with `exclusive_options`,
/// the file-making call the staging-file builder takes by name.
fn open_exclusively(staged: &std::path::Path) -> std::io::Result<std::fs::File> {
    return exclusive_options().open(staged);
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
///  A new staging file beside `path`:
/// hidden,
/// unique,
/// private (mode `0600`),
/// named `.<name>.<random>.partial`.
/// Why:
///   Recovery finds records by their exact names,
/// so a staging file never looks like one;
/// the file is made with
///  `O_EXCL` and `O_NOFOLLOW`,
/// so a second writer never shares it and a planted link is never written through,
/// and the
///  `tempfile` crate removes it when the value is dropped before it was published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await open(join(dirname(path), `.${basename(path)}.${random()}.partial`), O_CREAT | O_EXCL | O_RDWR, 0o600);
/// ```
fn staging_beside(path: &Path) -> std::io::Result<tempfile::NamedTempFile> {
    let Some(name) = path.file_name() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} names no file", path.display()),
        ));
    };
    let mut prefix: OsString = OsString::from(".");
    prefix.push(name);
    prefix.push(".");
    // A bare file name has an empty parent; the staging file then goes in the current directory like the record.
    let directory: &Path = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    };
    // The file is made by `exclusive_options` (exclusive, no-follow, mode 0600) and handed to the crate, which names it and removes
    //  it when dropped; the crate's own file-making call wraps the system error with the staging path, which hides the error code.
    return tempfile::Builder::new()
        .prefix(prefix.as_os_str())
        .suffix(".partial")
        .make_in(directory, open_exclusively);
}

/// What:
///  Make `staged` appear at `path` when the filesystem answered neither `renameat2` with `RENAME_NOREPLACE` nor a hard link:
///  `AlreadyExists` when `path` is taken,
/// the rename otherwise.
/// Why:
///   The owner,
/// 2026-11-07,
/// on a filesystem such as an S3 bucket through `s3fs-fuse` that has neither:
/// "Keep the fallback
///  (Recommended)",
/// after being told that "Commits keep working on those disks.
/// We own 10 lines plus a 15-line test.
/// On those
///  disks only,
/// two programs saving the same file at the same instant could overwrite one another."
/// The check then rename is
///  atomic but not exclusive:
/// a name taken between the two is replaced without an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (await exists(path)) throw alreadyExists(path); await rename(staged, path);
/// ```
fn publish_by_checked_rename(staged: &Path, path: &Path) -> std::io::Result<()> {
    // The check and the rename are one step to every other thread of this process that publishes the name by this fallback; the
    //  claim ends when this function returns, on every way out. `TimedOut` means nothing was published.
    let _held: HeldName = hold_name_while_publishing(path, HELD_NAME_WAIT)?;
    if std::fs::symlink_metadata(path).is_ok() {
        return Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
    }
    #[cfg(test)]
    BETWEEN_THE_CHECK_AND_THE_RENAME.with(|hook| { if let Some(pause) = hook.borrow().as_ref() { pause(); } });
    return std::fs::rename(staged, path);
}

/// What:
///  The outcome of the exclusive publish `persist_noclobber` made:
/// a taken name is the operating system's own `EEXIST`,
/// and
///  any other refusal publishes the staged file by the checked rename.
/// Why:
///   `persist_noclobber` tries `renameat2` with `RENAME_NOREPLACE`,
/// then a hard link;
/// a filesystem that answers neither
///  leaves the staged file where it was,
/// and "Commits should never fail" asks for the rename there.
/// A function of the result
///  lets a test feed the refusal that no filesystem on a test host produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { await publishNoReplace(staged, path); } catch (error) { if (error.code === 'EEXIST') throw error; await renameIfFree(staged, path); }
/// ```
fn settle_exclusive_publish(
    path: &Path,
    outcome: Result<File, tempfile::PersistError>,
) -> std::io::Result<()> {
    let refused: tempfile::PersistError = match outcome {
        Ok(_published) => return Ok(()),
        Err(refusal) => refusal,
    };
    if refused.error.kind() == std::io::ErrorKind::AlreadyExists {
        // The staged file is removed when `refused.file` drops here.
        return Err(refused.error);
    }
    debug(
        "publishExclusive",
        format!("the filesystem refused the exclusive publish of {} ({}); publishing by rename", path.display(), refused.error).as_str(),
    );
    let mut staged: tempfile::NamedTempFile = refused.file;
    let renamed: std::io::Result<()> = publish_by_checked_rename(staged.path(), path);
    // After the rename the staging name is gone, and whatever later takes that random name is not ours to remove.
    if renamed.is_ok() {
        staged.disable_cleanup(true);
    }
    return renamed;
}

#[cfg(test)]
#[path = "private_storage_syncs_test_support.rs"]
mod syncs_test_support;

#[cfg(test)]
pub(crate) use syncs_test_support::skip_the_durability_syncs_here;
#[cfg(test)]
pub(crate) use syncs_test_support::{record_directory_sync, record_file_sync, take_directory_syncs, take_file_syncs};

/// Whether this process's syncs must reach the disk;
/// always,
/// in the release executable.
#[cfg(test)]
pub(crate) fn syncing_is_faithful() -> bool {
    return syncs_test_support::syncing_is_faithful();
}

/// Always,
/// in the release executable.
#[cfg(not(test))]
pub(crate) fn syncing_is_faithful() -> bool {
    return true;
}

/// What:
///  Flushes the staging file to disk when the write is durable,
/// and does nothing otherwise.
/// Why:
///  `write_then_publish` runs it only after the bytes were written;
/// a record that tolerates a torn write skips the sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (durable) { await handle.sync(); }
/// ```
fn sync_when_durable(file: &File, path: &Path, durable: bool) -> std::io::Result<()> {
    if !durable {
        return Ok(());
    }
    // The path is read only by the test-only record; a release build leaves it unused.
    #[cfg(not(test))]
    let _path_for_tests: &Path = path;
    #[cfg(test)]
    record_file_sync(path);
    if !syncing_is_faithful() {
        return Ok(());
    }
    // `sync_all` flushes data and metadata, the incumbent's `handle.sync()`.
    return file.sync_all();
}

/// What:
///  Sets the permission bits of the staging file when a mode was asked for,
/// and does nothing otherwise.
/// Why:
///  `write_then_publish` applies the mode before the file appears at its final name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (mode !== undefined) { await chmod(staged, mode); }
/// ```
fn protect_when_asked(staged: &Path, mode: Option<u32>) -> std::io::Result<()> {
    return match mode {
        Some(bits) => protect_path(staged, bits),
        None => Ok(()),
    };
}

/// What:
///  Write `bytes` to a staging file beside `path`,
/// sync it when `durable`,
/// and publish it at `path`;
/// any failure
///  removes the staging file,
/// so `path` holds all the bytes or does not exist.
///  `mode` is applied to the file before it is published,
/// when given.
/// Why:
///   A file created at its final name and then written is a record that a kill,
/// or a full disk,
/// between the two
///  steps leaves empty or cut:
/// recovery then stops every guarded command until a person removes it.
/// Publishing a
///  complete file with `persist_noclobber` makes the record appear whole or not at all,
/// whether or not its bytes are
///  synced;
/// a record that the tearing table shows to be benign when a power loss tears it skips the sync.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const staged = await stagingBeside(path); await writeAndSync(staged, bytes); await publishExclusive(staged, path);
/// ```
fn write_then_publish(path: &Path, bytes: &[u8], mode: Option<u32>, durable: bool) -> std::io::Result<()> {
    let mut staged: tempfile::NamedTempFile = staging_beside(path)?;
    // Each step runs only when the one before it succeeded, and the first failure is the result; a failure drops `staged`,
    //  which removes the staging file.
    staged.as_file_mut().write_all(bytes)?;
    sync_when_durable(staged.as_file(), path, durable)?;
    protect_when_asked(staged.path(), mode)?;
    #[cfg(test)]
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| { return hook.borrow().as_ref().map_or(Ok(()), |fail| { return fail(); }); })?;
    #[cfg(test)]
    let outcome: Result<File, tempfile::PersistError> = if THE_FILESYSTEM_ANSWERS_NEITHER_EXCLUSIVE_PUBLISH.with(std::cell::Cell::get) {
        Err(tempfile::PersistError { error: std::io::Error::from_raw_os_error(libc::ENOTSUP), file: staged })
    } else {
        staged.persist_noclobber(path)
    };
    #[cfg(not(test))]
    let outcome: Result<File, tempfile::PersistError> = staged.persist_noclobber(path);
    return settle_exclusive_publish(path, outcome);
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
///  The file is written under a staging name and published whole (`write_then_publish`),
/// so that the promise
///  also holds for a kill between the creation and the write and for a write cut short by a full disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writePrivateFile({ path, bytes }): Promise<void>;
/// ```
pub fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    return write_then_publish(path, bytes, Some(PRIVATE_FILE_MODE), true);
}

/// What:
///  Write one private file exclusively and publish it whole,
///  without syncing its bytes.
/// Why:
///   A record that recovery completes or discards whatever a power loss left of it (the tearing table in
///  `doc/planning/cli-git-spawn-and-sync-reduction.md`) gains nothing from a flush that makes every commit wait for the
///  disk;
/// a kill still leaves it whole or absent,
/// because it is published by link as `write_private_file` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePrivateFile({ path, bytes, sync: false });
/// ```
pub fn write_private_file_unsynced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    return write_then_publish(path, bytes, Some(PRIVATE_FILE_MODE), false);
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
///  The file is published whole,
/// as `write_private_file` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(path, O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW, 0o600); await handle.writeFile(bytes); await handle.sync();
/// ```
pub fn write_exclusive_synced(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    return write_then_publish(path, bytes, None, true);
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

/// The operating system's answer from a publish that kept its staged file,
/// which is the failure the caller reports.
fn without_the_staged_file(error: tempfile::PersistError) -> std::io::Error {
    return error.error;
}

/// What:
///  Publish `bytes` at `path` in one rename,
///  replacing whatever the name held.
/// Why:
///   A record this program rewrites (the migration's list of files this user may not read) is read
/// by a later run as a whole;
/// a truncating write leaves it empty for a kill that lands between the truncation and the
///  bytes,
/// and the next run reads an empty record as nothing to report.
/// The bytes are staged beside the
///  name and made durable before the rename,
/// so a kill leaves either the record that was there or the complete
///  new one,
/// and the staging file is created exclusively with mode `0600` and never follows a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function replacePrivateFile({ path, bytes }): Promise<void>;
/// ```
pub fn replace_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut staged: tempfile::NamedTempFile = staging_beside(path)?;
    staged.as_file_mut().write_all(bytes)?;
    sync_when_durable(staged.as_file(), path, true)?;
    staged.persist(path).map_err(without_the_staged_file)?;
    return match path.parent() {
        Some(directory) if !directory.as_os_str().is_empty() => sync_directory(directory),
        _ => sync_directory(Path::new(".")),
    };
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
///  Whether a failed directory sync means the filesystem has no such operation,
/// not that the flush failed.
/// Why:
///   `EINVAL`,
/// `ENOTSUP` and `ENOSYS` are what a filesystem without the operation returns;
/// `EIO`,
/// `ENOSPC` and
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
///  The outcome of syncing the directory at `path`:
/// a filesystem without the operation counts as synced,
/// any other
///  failure stays an error.
/// Why:
///   A filesystem with no directory flush (procfs,
/// some network and userspace filesystems) answers with one of the
///  unsupported codes;
/// the entry cannot be made durable there by any means,
/// and failing would stop every commit.
/// A
///  function of the sync result lets a test feed the flush failures (`EIO`,
/// `ENOSPC`,
/// `EDQUOT`) that no
///  unprivileged filesystem produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (error !== undefined && !['EINVAL', 'ENOTSUP', 'ENOSYS'].includes(error.code)) throw error;
/// ```
#[cfg(unix)]
fn settle_directory_sync(path: &Path, outcome: std::io::Result<()>) -> std::io::Result<()> {
    return match outcome {
        Ok(()) => Ok(()),
        Err(error) if directory_sync_is_unsupported(&error) => {
            debug(
                "syncDirectory",
                format!("the filesystem of {} cannot sync a directory ({error}); skipped", path.display()).as_str(),
            );
            Ok(())
        }
        Err(error) => Err(error),
    };
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
    if !syncing_is_faithful() {
        return Ok(());
    }
    #[cfg(unix)]
    {
        // Opening a directory read-only and syncing it is the POSIX way to flush its entries.
        let directory: File = File::open(path)?;
        settle_directory_sync(path, rustix::fs::fsync(&directory).map_err(std::io::Error::from))?;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    let _ = path;
    return Ok(());
}

#[cfg(test)]
thread_local! {
    /// A control may fail a publish on this thread after the staging file was made and written,
    /// which is where a full disk or a
    ///  kill ends one.
    pub(crate) static AFTER_THE_STAGING_FILE_IS_WRITTEN: std::cell::RefCell<Option<Box<dyn Fn() -> std::io::Result<()>>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
thread_local! {
    /// A control may make every exclusive publish on this thread get the refusal of a filesystem with neither `renameat2` with
    ///  `RENAME_NOREPLACE` nor hard links (`s3fs-fuse` answers `ENOTSUP`),
    /// which no filesystem of a test host gives.
    pub(crate) static THE_FILESYSTEM_ANSWERS_NEITHER_EXCLUSIVE_PUBLISH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// A control may stop the checked rename on this thread between its check and its rename,
    /// where a second publisher of the
    ///  same name can slip in.
    pub(crate) static BETWEEN_THE_CHECK_AND_THE_RENAME: std::cell::RefCell<Option<Box<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
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
///  Open a file for reading without following a final link,
///  and without waiting for a writer when the name holds a named pipe.
/// Why:
///   Records are always files the wrappers created;
///  a link planted at a record path must
///       be refused,
///  never read through.
///  `O_NONBLOCK` makes the open answer whatever the name holds,
///  so the type check that follows refuses a pipe at once instead of waiting inside `open(2)`
///  for a writer that may never come
///  (`SPEC.md`,
///  "Every wait has a bound";
///  a trial reviewer's finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await open(path, O_RDONLY | O_NOFOLLOW | O_NONBLOCK);
/// ```
fn open_no_follow(path: &Path) -> std::io::Result<File> {
    // `mut` allows the builder to be configured step by step.
    let mut options: OpenOptions = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        // The trait adds `.custom_flags` to open options on Unix.
        /// The Unix open-option extensions,
        ///  for the no-follow flag and the non-blocking open.
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
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

/// Publication controls stay out of the release executable.
#[cfg(test)]
#[path = "private_storage_publish_tests.rs"]
mod publish_tests;

/// The measurement of how long a name is held,
/// kept out of the release executable.
#[cfg(test)]
#[path = "private_storage_hold_time_measurement_tests.rs"]
mod hold_time_measurement_tests;
