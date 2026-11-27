//! What:
//!  Private,
//!  crash-safe file primitives shared by every durable record of the wrapper:
//!       exclusive creation under a staging name with private modes,
//!  and the publish that makes a record appear whole.
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
//!  This file holds writing a record whole:
//!  the staging file beside it,
//!  the exclusive publish,
//!  the rename that never replaces a name,
//!  and the four writers a caller chooses between.
//!  The rest of the primitives are files of their own,
//!  because this one reached the repository's line limit,
//!  and this file re-exports the names their callers reach:
//!  `a_record_or_folder_the_account_alone_may_read_and_write.rs` holds the permission bits and the folders,
//!  `a_record_made_durable_with_a_sync.rs` holds the flush of a file and the sync of a folder,
//!  `a_record_read_with_the_reason_it_was_refused.rs` holds the reads and their refusals,
//!  and `private_storage_replacing_a_record.rs` holds the publish over a name already taken.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writePrivateFile({ path, bytes }); await syncDirectory(dirname(path));
//! ```

/// Diagnostics for a decision no failure reports.
use crate::diagnostic_log::debug;
/// The in-memory claim that makes the checked rename one step to the other threads of the process.
use crate::name_held_while_one_thread_publishes_it::{
    HeldName,
    hold_name_while_publishing,
};
/// Owned operating-system text for the staging name.
use std::ffi::OsString;
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
use std::fs::{
    File,
    OpenOptions,
};
/// The trait that gives files `.write_all`.
use std::io::Write;
/// `Path` is a borrowed filesystem path of raw bytes.
use std::path::Path;
use std::time::Duration;

/// The sync of a folder that makes a new or renamed entry durable.
pub use crate::a_record_made_durable_with_a_sync::sync_directory;
/// The flush of the staging file when the write must be durable,
/// which the other file of this folder reaches this module for.
pub(super) use crate::a_record_made_durable_with_a_sync::sync_when_durable;
/// Whether this process's syncs must reach the disk,
/// as the readers of a durable routine ask it.
pub(crate) use crate::a_record_made_durable_with_a_sync::syncing_is_faithful;
/// What this process's durability syncs left,
/// which a control of a durable routine reads and empties,
/// and the switch that makes them no-ops.
#[cfg(test)]
pub(crate) use crate::a_record_made_durable_with_a_sync::{
    skip_the_durability_syncs_here,
    take_directory_syncs,
    take_file_syncs,
};
/// The change that makes the staging file's permission bits exact,
/// which every write of this file applies before the record appears.
use crate::a_record_or_folder_the_account_alone_may_read_and_write::protect_when_asked;
/// The permission bits a private file and a private folder are made with,
/// the change that makes them exact whatever the caller's `umask` is,
/// the folders the wrapper creates with them,
/// and the removal of a tree it retires.
pub use crate::a_record_or_folder_the_account_alone_may_read_and_write::{
    PRIVATE_DIRECTORY_MODE,
    PRIVATE_FILE_MODE,
    create_private_directory,
    create_private_directory_all,
    protect_path,
    remove_tree,
};
/// The record a read refuses,
/// and the two reads that name the reason.
pub use crate::a_record_read_with_the_reason_it_was_refused::{
    ReadRefusal,
    open_regular_file,
    read_regular_file,
};

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
pub(super) fn staging_beside(path: &Path) -> std::io::Result<tempfile::NamedTempFile> {
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
fn publish_by_checked_rename(
    staged: &Path,
    path: &Path,
) -> std::io::Result<()> {
    // The check and the rename are one step to every other thread of this process that publishes the name by this fallback; the
    //  claim ends when this function returns, on every way out. `TimedOut` means nothing was published.
    let _held: HeldName = hold_name_while_publishing(path, HELD_NAME_WAIT)?;
    if std::fs::symlink_metadata(path).is_ok() {
        return Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
    }
    #[cfg(test)]
    BETWEEN_THE_CHECK_AND_THE_RENAME.with(|hook| {
        if let Some(pause) = hook.borrow().as_ref() {
            pause();
        }
    });
    return std::fs::rename(staged, path);
}

/// What:
///  Renames `from` to `to` unless `to` exists,
///  by the one kernel call that is as exclusive as `create_new`.
///  `&Path` lends a path;
/// `std::io::Result<()>` is the operating system's own answer.
/// Why:
///  A lock is placed and an earlier name is moved to its current name by this step:
///  when the target exists the call fails and both names stay,
/// so no data is written over.
///  The sibling `exchange` of `rename_that_never_replaces.rs` replaces a name and hands back what stood there;
/// this one refuses to replace anything.
///  A platform without `renameat2` answers `Unsupported`,
/// which the caller reads through `rename_that_never_replaces::rename_unsupported`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const done: void = renameNoReplace(from, to); // no Node API; the kernel call is renameat2(RENAME_NOREPLACE)
/// ```
#[cfg(target_os = "linux")]
pub fn rename_noreplace(
    from: &Path,
    to: &Path,
) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let source =
        std::ffi::CString::new(from.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    let target =
        std::ffi::CString::new(to.as_os_str().as_bytes()).map_err(std::io::Error::other)?;
    // SAFETY: both strings are valid NUL-terminated paths for the call.
    let done: i32 = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            target.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    return if done == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    };
}

/// Declines the rename on a platform with no no-replace request,
/// so a caller reads one `Unsupported` answer instead of another call.
#[cfg(not(target_os = "linux"))]
pub fn rename_noreplace(
    _from: &Path,
    _to: &Path,
) -> std::io::Result<()> {
    return Err(std::io::Error::from(std::io::ErrorKind::Unsupported));
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
        format!(
            "the filesystem refused the exclusive publish of {} ({}); publishing by rename",
            path.display(),
            refused.error
        )
        .as_str(),
    );
    let mut staged: tempfile::NamedTempFile = refused.file;
    let renamed: std::io::Result<()> = publish_by_checked_rename(staged.path(), path);
    // After the rename the staging name is gone, and whatever later takes that random name is not ours to remove.
    if renamed.is_ok() {
        staged.disable_cleanup(true);
    }
    return renamed;
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
fn write_then_publish(
    path: &Path,
    bytes: &[u8],
    mode: Option<u32>,
    durable: bool,
) -> std::io::Result<()> {
    let mut staged: tempfile::NamedTempFile = staging_beside(path)?;
    // Each step runs only when the one before it succeeded, and the first failure is the result; a failure drops `staged`,
    //  which removes the staging file.
    staged.as_file_mut().write_all(bytes)?;
    sync_when_durable(staged.as_file(), path, durable)?;
    protect_when_asked(staged.path(), mode)?;
    #[cfg(test)]
    AFTER_THE_STAGING_FILE_IS_WRITTEN.with(|hook| {
        return hook.borrow().as_ref().map_or(Ok(()), |fail| {
            return fail();
        });
    })?;
    #[cfg(test)]
    let outcome: Result<File, tempfile::PersistError> =
        if THE_FILESYSTEM_ANSWERS_NEITHER_EXCLUSIVE_PUBLISH.with(std::cell::Cell::get) {
            Err(tempfile::PersistError {
                error: std::io::Error::from_raw_os_error(libc::ENOTSUP),
                file: staged,
            })
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
pub fn write_private_file(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
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
pub fn write_private_file_unsynced(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
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
pub fn write_exclusive_synced(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
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
pub fn write_exclusive_scratch(
    path: &Path,
    bytes: &[u8],
) -> std::io::Result<()> {
    let mut file: File = exclusive_options().open(path)?;
    // A write cut short by a full disk or a size limit leaves nothing at its name: a caller reads these bytes as the
    //  input of one command, and half of them would be read as an answer.
    if let Err(error) = file.write_all(bytes) {
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
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
