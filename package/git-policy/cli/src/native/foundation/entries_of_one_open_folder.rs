//! What:
//!  Opening one folder by path and holding it,
//!  then reading and listing the entries below it by their names,
//!  every call relative to that one descriptor.
//! Why:
//!  A caller that opens a folder once and names entries beneath it never resolves a path component a second time,
//!  so the entry it judges is the entry it opened,
//!  and a name the account could change below the folder is never followed:
//!  `O_NOFOLLOW` on the folder refuses a link planted at its own name,
//!  `O_NOFOLLOW` on an entry refuses a link planted at that name,
//!  and `O_NONBLOCK` makes an open answer a named pipe at once instead of waiting for a writer.
//!  This is the descriptor-relative form of the reads `private_storage` makes by path.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const folder = openFolder('/var/lib/monochromatic-git-policy-cli'); const names = entryNames(folder);
//! const bytes = readEntry(folder, 'record', 262538); // at most that many bytes, or a refusal
//! ```

use crate::private_storage::ReadRefusal;
use std::ffi::{OsStr, OsString};
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// What:
///  Open the folder at `path` and hold it,
///  refusing a link at that name and any other non-folder.
/// Why:
///  Everything the caller reads or writes below the folder is named relative to this descriptor,
///  so the folder judged is the folder held even while the account above it renames or replaces a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = open(path, O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
/// ```
///
/// # Errors
/// The system's own answer:
/// `NotFound` when nothing is there,
/// `ELOOP` when the name holds a link,
/// `ENOTDIR` when it holds another file.
pub fn open_folder(path: &Path) -> std::io::Result<std::fs::File> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.read(true).custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    return options.open(path);
}

/// What:
///  Open the entry `name` below `folder` for reading,
///  refusing a link,
///  a folder or another non-regular file.
/// Why:
///  A record is always a file a writer created with `O_EXCL`,
///  so a link planted at its name must be refused rather than read through,
///  and the handle's own metadata is the type answer a swap between the open and the check cannot change.
///  A named pipe at the name is answered at once rather than waited on (`O_NONBLOCK`),
///  so a reader never waits for a writer that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openat(folder, name, O_RDONLY | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC);
/// if (!(await file.stat()).isFile()) throw 'not regular';
/// ```
///
/// # Errors
/// `Missing` when the name is absent,
/// `NotRegular` when it holds a link,
/// a folder or another non-regular file,
/// and the system's own answer for any other refusal.
pub fn open_entry_for_reading(folder: &std::fs::File, name: &OsStr) -> Result<std::fs::File, ReadRefusal> {
    let file: std::fs::File = match rustix::fs::openat(folder, name, rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC, rustix::fs::Mode::empty()) {
        Ok(opened) => std::fs::File::from(opened),
        Err(error) => return Err(open_refusal(error)),
    };
    let metadata: std::fs::Metadata = file.metadata().map_err(ReadRefusal::Io)?;
    if !metadata.is_file() {
        return Err(ReadRefusal::NotRegular);
    }
    return Ok(file);
}

/// The refusal of one open,
/// told apart the way a caller repairs each cause.
fn open_refusal(error: rustix::io::Errno) -> ReadRefusal {
    return match error {
        rustix::io::Errno::NOENT => ReadRefusal::Missing,
        rustix::io::Errno::LOOP => ReadRefusal::NotRegular,
        other => ReadRefusal::Io(std::io::Error::from(other)),
    };
}

/// What:
///  Read the whole of an open regular file,
///  up to `limit` bytes.
/// Why:
///   A runaway or replaced file must be named rather than read into memory,
///  so the read is capped even when the file grew after it was opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytes = await readAll(file, limit); // TooLarge past the limit
/// ```
///
/// # Errors
/// `TooLarge` when the file holds more than `limit` bytes,
/// and the system's own answer for a read that failed.
pub fn read_entry_bytes(file: &mut std::fs::File, limit: u64) -> Result<Vec<u8>, ReadRefusal> {
    let mut bytes: Vec<u8> = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes).map_err(ReadRefusal::Io)?;
    if bytes.len() as u64 > limit {
        return Err(ReadRefusal::TooLarge);
    }
    return Ok(bytes);
}

/// What:
///  Every entry name below `folder`,
///  and nothing of the folder above it.
/// Why:
///  The adoption refuses a foreign name and `doctor` reports what it finds,
///  so both need the folder's own listing rather than a path's.
///  `Dir::read_from` reads a duplicate of the held descriptor,
///  so the held folder is not rewound under the caller.
///  A name is kept as the bytes the kernel listed,
///  so a name that is not UTF-8 reaches the caller as itself and is refused there,
///  rather than being decoded into a name the folder does not hold.
///  The listing has no entry cap:
///  the accepted design says the store folder has no total bound
///  (`doc/planning/cli-git-zfs-slot-identity-store-per-slot-files-design.md`,
///  "A record file's bound,
///  and what replaces the whole-store bound"),
///  and what sizes a listing is the account's clone count,
///  not a foreign writer:
///  the folder is root-owned `0755`,
///  so no other account adds an entry,
///  and root adds one per clone it makes on the account's behalf.
///  `doctor` reports the entry count beside the records,
///  which is what makes a grown store visible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = readdir(folder); // without '.' and '..'
/// ```
///
/// # Errors
/// The system's own answer for a folder that cannot be listed.
pub fn entry_names(folder: &std::fs::File) -> std::io::Result<Vec<OsString>> {
    let mut directory: rustix::fs::Dir = rustix::fs::Dir::read_from(folder)?;
    let mut names: Vec<OsString> = Vec::new();
    while let Some(read) = directory.read() {
        let entry: rustix::fs::DirEntry = read?;
        let name: OsString = OsString::from(OsStr::from_bytes(entry.file_name().to_bytes()));
        // The kernel's own listing starts with the folder and its parent, which name no entry of it.
        if name == OsStr::new(".") || name == OsStr::new("..") {
            continue;
        }
        names.push(name);
    }
    return Ok(names);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "entries_of_one_open_folder_tests.rs"]
mod tests;
