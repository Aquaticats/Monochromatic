//! What:
//!  Creating one entry below a folder the caller holds open,
//!  filling it,
//!  and publishing it under its real name in one step that replaces nothing,
//!  plus the two renames and the removal that follow it.
//! Why:
//!  A record must never be visible under its own name holding a prefix of its bytes,
//!  so a writer fills a staging name and gives a file its real name in one call:
//!  `linkat` publishes it and answers `EEXIST` to the second writer of one name,
//!  and a folder that refuses a hard link publishes with `renameat2` and `RENAME_NOREPLACE`,
//!  which answers `EEXIST` just the same.
//!  A rename that must replace a record is the adoption's one write,
//!  and it replaces a name and never a byte.
//!  Every call is relative to the folder descriptor the caller opened,
//!  which is the descriptor-relative form of `private_storage`'s path-based publish.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const staged = createEntry(folder, '.a1b2', 0o644); fillAndSync(staged, bytes, 0o644);
//! linkEntry(folder, '.a1b2', 'add-cache-layer'); removeEntry(folder, '.a1b2'); syncFolder(folder);
//! ```

use std::ffi::OsStr;
use std::io::Write;

/// What:
///  Create the entry `name` below `folder` with an exclusive creation,
///  never following a link at that name.
/// Why:
///   Two writers of one staging name never share a file,
///  and a link planted at the name is refused rather than written through;
///  the mode is set at creation, so the file is never readable wider than it keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openat(folder, name, O_CREAT | O_EXCL | O_WRONLY | O_NOFOLLOW | O_CLOEXEC, mode);
/// ```
///
/// # Errors
/// The system's own answer, including `EEXIST` when the name is taken.
pub fn create_new_entry(folder: &std::fs::File, name: &OsStr, mode: u32) -> std::io::Result<std::fs::File> {
    let opened: rustix::fd::OwnedFd = rustix::fs::openat(
        folder,
        name,
        rustix::fs::OFlags::CREATE | rustix::fs::OFlags::EXCL | rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::from_bits_truncate(mode),
    )?;
    return Ok(std::fs::File::from(opened));
}

/// What:
///  Write `bytes` into a freshly created entry,
///  set its mode, and flush it,
///  so the bytes are durable before any name can hold them.
/// Why:
///  The staging file holds every byte before it is published,
///  so the record's own name never holds a prefix of them;
///  the mode is set before the name appears,
///  so the record is never readable at a wider mode than the one it keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await file.writeAll(bytes); await file.chmod(mode); await file.sync();
/// ```
///
/// # Errors
/// The system's own answer for a write, a mode change or a flush.
pub fn fill_entry_and_sync(file: &mut std::fs::File, bytes: &[u8], mode: u32) -> std::io::Result<()> {
    file.write_all(bytes)?;
    rustix::fs::fchmod(&*file, rustix::fs::Mode::from_bits_truncate(mode))?;
    return file.sync_all();
}

/// What:
///  Give the entry `from` below `folder` the second name `to` in one call that replaces nothing.
/// Why:
///   `linkat` publishes a filled staging file under its record name,
///  and its `EEXIST` is the answer that decides the one winner of two recorders:
///  the loser's name is never written and the winner's bytes are never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// linkat(folder, from, folder, to, 0); // EEXIST when to is taken
/// ```
///
/// # Errors
/// The system's own answer, including `EEXIST` when `to` is taken and `EPERM` or `EOPNOTSUPP` on a folder that refuses a hard link.
pub fn link_entry(folder: &std::fs::File, from: &OsStr, to: &OsStr) -> std::io::Result<()> {
    return rustix::fs::linkat(folder, from, folder, to, rustix::fs::AtFlags::empty()).map_err(std::io::Error::from);
}

/// What:
///  Whether a folder refused a hard link because it does not make one,
///  rather than because the name was taken or storage failed.
/// Why:
///  A publication falls back to the rename that never replaces only for the folders that have no hard links,
///  so a taken name must not be mistaken for one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noLinks = error.code === 'EPERM' || error.code === 'EOPNOTSUPP';
/// ```
pub fn link_refused_because_the_folder_makes_none(error: &std::io::Error) -> bool {
    return matches!(error.raw_os_error(), Some(code) if code == libc::EPERM || code == libc::EOPNOTSUPP || code == libc::ENOTSUP);
}

/// What:
///  Rename `from` to `to` below `folder` in one call that replaces nothing.
/// Why:
///   It is the publication's fallback where the folder refuses a hard link,
///  and it answers `EEXIST` onto a taken name while leaving the staging name in place,
///  so the same one-winner property holds without a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renameat2(folder, from, folder, to, RENAME_NOREPLACE);
/// ```
///
/// # Errors
/// The system's own answer, including `EEXIST` when `to` is taken and `EINVAL` on a folder without the flag.
pub fn rename_entry_never_replacing(folder: &std::fs::File, from: &OsStr, to: &OsStr) -> std::io::Result<()> {
    return rustix::fs::renameat_with(folder, from, folder, to, rustix::fs::RenameFlags::NOREPLACE).map_err(std::io::Error::from);
}

/// What:
///  Rename `from` to `to` below `folder`, replacing what stands at `to` in one step.
/// Why:
///   The adoption's new parent record replaces the old one in one rename,
///  so a reader sees either whole record and never a third state,
///  and a rename moves a name and never a byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// renameat(folder, from, folder, to);
/// ```
///
/// # Errors
/// The system's own answer for a rename that failed.
pub fn rename_entry_replacing(folder: &std::fs::File, from: &OsStr, to: &OsStr) -> std::io::Result<()> {
    return rustix::fs::renameat(folder, from, folder, to).map_err(std::io::Error::from);
}

/// What:
///  Remove the entry `name` below `folder`.
/// Why:
///  A staging name is removed by its writer once the record has its own name,
///  and an adoption removes the staging names a killed writer left behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unlinkat(folder, name, 0);
/// ```
///
/// # Errors
/// The system's own answer for a removal that failed, including `ENOENT` for a name already gone.
pub fn remove_entry(folder: &std::fs::File, name: &OsStr) -> std::io::Result<()> {
    return rustix::fs::unlinkat(folder, name, rustix::fs::AtFlags::empty()).map_err(std::io::Error::from);
}

/// What:
///  Flush a folder's own entries to the disk.
/// Why:
///   A record's name is durable only after its folder is synced,
///  so a publisher syncs the folder after the link or the rename,
///  and a remover syncs it after the unlink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fsync(folder);
/// ```
///
/// # Errors
/// The system's own answer for a flush that failed.
pub fn sync_folder(folder: &std::fs::File) -> std::io::Result<()> {
    return rustix::fs::fsync(folder).map_err(std::io::Error::from);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "publishing_entries_in_one_open_folder_tests.rs"]
mod tests;
