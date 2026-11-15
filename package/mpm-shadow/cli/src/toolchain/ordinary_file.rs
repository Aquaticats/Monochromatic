//! What:
//! Opening and reading a file this tool did not just create,
//! without waiting,
//! and refusing anything that is not an ordinary file.
//!
//! Why:
//! A name this tool did not create holds whatever someone put there,
//! so every read of one here asks for two things the standard library's open does not:
//! no waiting, because opening a named pipe for reading waits for a writer that may never come
//! (a `mpm-shadow install` whose launch record was a pipe waited for ever, which the seventh review reproduced),
//! and no following a symbolic link, so what is read is the entry that was named.
//! What comes back is inspected through the descriptor that was opened before one byte is read,
//! and anything but an ordinary file is refused with the sentence that names what stands there.
//! A caller that gets that sentence leaves the entry alone:
//! an undeclared launcher kept one install longer costs nothing,
//! and a file removed that was not the one judged is a loss.
//!
//! A platform without the call to ask for no waiting reads what it opened and refuses it;
//! only Linux, macOS and Redox have a rename that replaces nothing either,
//! so the reads here are the part that holds everywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // ordinary-file.ts
//! export function readOrdinary(file: string): Uint8Array  // throws
//! export function readOrdinaryThroughLinks(file: string): Uint8Array  // throws
//! export function createLockFile(file: string): FileHandle  // throws
//! export function sha256OfOrdinary(file: string): Uint8Array  // throws
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `Read` from `std::io`,
/// for the block read the digest makes.
use std::io::Read;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports the `Digest` trait and the SHA-256 type,
/// so a digest of bytes read here is the same hash the lock and the archive checks use.
use sha2::{Digest, Sha256};

/// The number of bytes a digest reads at a time,
/// so an ordinary file the size of a launcher never sits in memory whole.
const READ_BLOCK: usize = 64 * 1024;

/// What:
/// The most bytes a file this tool reads may hold.
/// Why:
/// Every file read here is one this tool wrote or was written for it, so the bound comes from what this tool writes:
/// the largest is a launcher, a byte copy of the `mpm-shadow` program (release build 5 332 336 bytes; the same
/// program in a debug build 72 620 408 bytes, and the fixture tests copy a debug test binary of 86 831 664 bytes;
/// all measured 2026-11-15), while the plan (23 871), the lock (63 625) and the records are far smaller.
/// Three times the largest measured copy is the bound.
/// A file past it is answered as unreadable rather than read:
/// a command that keeps a file open for appending can keep it growing for ever, and a read that follows it never ends
/// and holds every byte it read (the open's `O_NONBLOCK` says nothing about an ordinary file's length).
pub(crate) const MAX_ORDINARY_BYTES: u64 = 256 * 1024 * 1024;

/// The sentence for a file past `MAX_ORDINARY_BYTES`.
pub(crate) fn larger_than_read(path: &Path, limit: u64) -> String {
    return format!("{} is larger than the {limit} bytes this tool reads at most, so it was not read", path.display());
}

/// What:
/// The bytes of an ordinary file,
/// read in blocks,
/// and ending with a sentence when the file holds more than `MAX_ORDINARY_BYTES`.
/// Why:
/// The read ends at the bound whether or not the file has an end:
/// a file another command keeps growing is refused, not followed,
/// and the refusal names the file and the bound.
/// A kill inside the read leaves nothing: no name has been written and no entry moved.
///
/// How a growing file is shown to end the read is the control beside this module:
/// `seam` appends to the file between two blocks, which is the only way to make a file grow under a reader on purpose.
fn read_bounded(file: &mut fs::File, path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut buffer: Vec<u8> = vec![0; READ_BLOCK];
    loop {
        if bytes.len() as u64 > limit {
            return Err(larger_than_read(path, limit));
        }
        let read: usize = match file.read(&mut buffer) {
            Ok(count) => count,
            Err(error) => return Err(format!("{} cannot be read: {error}", path.display())),
        };
        if read == 0 {
            return Ok(bytes);
        }
        bytes.extend_from_slice(&buffer[..read]);
        #[cfg(test)]
        seam::after_a_block(path);
    }
}

/// What:
/// The mode a lock file is created with, before the umask narrows it:
/// the same mode the standard library gives a new file,
/// so a lock file this module creates is no wider than one it created before.
#[cfg(unix)]
const NEW_FILE_MODE: rustix::fs::Mode = rustix::fs::Mode::from_bits_truncate(0o666);

/// What:
/// The file at `path`, open for reading,
/// when it is an ordinary file,
/// and the sentence naming what stands there otherwise.
/// Why:
/// Every reader of a file this run did not create starts here,
/// so no path in this tool can wait on a named pipe or read through a symbolic link by accident.
pub(crate) fn open_ordinary(path: &Path) -> Result<fs::File, String> {
    let file: fs::File = match open_without_waiting(path) {
        Ok(opened) => opened,
        Err(error) => return Err(format!("{} cannot be read: {error}", path.display())),
    };
    let metadata: fs::Metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => return Err(format!("{} cannot be inspected: {error}", path.display())),
    };
    if !metadata.is_file() {
        return Err(not_ordinary(path, &metadata));
    }
    return Ok(file);
}

/// What:
/// `open_ordinary` for a path that may be a symbolic link:
/// the link is followed by resolving it first,
/// and the file it names must still be an ordinary file.
/// Why:
/// A person's plan and lock may be reached through a link,
/// and a link is what a tool's `current` directory is made of;
/// following one is not a read of a file nobody named,
/// while a pipe or a directory at the end of it is still refused.
pub(crate) fn open_ordinary_through_links(path: &Path) -> Result<fs::File, String> {
    let resolved: PathBuf = match fs::canonicalize(path) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("{} cannot be resolved: {error}", path.display())),
    };
    return open_ordinary(&resolved);
}

/// The bytes of the ordinary file at `path`.
pub(crate) fn read_ordinary(path: &Path) -> Result<Vec<u8>, String> {
    let mut file: fs::File = open_ordinary(path)?;
    return read_bounded(&mut file, path, MAX_ORDINARY_BYTES);
}

/// The bytes of the ordinary file at `path`, read with `limit` instead of `MAX_ORDINARY_BYTES`.
/// Why:
/// One loop serves the tool and the control that shows a growing file ending, and the control passes a small bound so
/// it needs no file of the shipped size.
#[cfg(test)]
pub(crate) fn read_ordinary_at_most(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let mut file: fs::File = open_ordinary(path)?;
    return read_bounded(&mut file, path, limit);
}

/// The text of the ordinary file at `path`,
/// and the sentence naming what is wrong with it when it is not text.
pub(crate) fn read_ordinary_text(path: &Path) -> Result<String, String> {
    let bytes: Vec<u8> = read_ordinary(path)?;
    return match String::from_utf8(bytes) {
        Ok(text) => Ok(text),
        Err(_) => Err(format!("{} is not text this tool can read", path.display())),
    };
}

/// The bytes of the ordinary file at `path`,
/// reached through a symbolic link when the path is one.
pub(crate) fn read_ordinary_through_links(path: &Path) -> Result<Vec<u8>, String> {
    let mut file: fs::File = open_ordinary_through_links(path)?;
    return read_bounded(&mut file, path, MAX_ORDINARY_BYTES);
}

/// The text of the ordinary file at `path`,
/// reached through a symbolic link when the path is one.
pub(crate) fn read_ordinary_text_through_links(path: &Path) -> Result<String, String> {
    let bytes: Vec<u8> = read_ordinary_through_links(path)?;
    return match String::from_utf8(bytes) {
        Ok(text) => Ok(text),
        Err(_) => Err(format!("{} is not text this tool can read", path.display())),
    };
}

/// What:
/// The file at `path`, opened for reading and writing and created when it is absent,
/// without waiting,
/// when it is (or becomes) an ordinary file.
/// Why:
/// A guard or a record lock is a file this tool creates,
/// and a name that already holds something else is not one to lock:
/// opening a named pipe for writing waits for a reader,
/// which would hang the install that a lock is meant to order.
pub(crate) fn create_lock_file(path: &Path) -> Result<fs::File, String> {
    let file: fs::File = match create_without_waiting(path) {
        Ok(created) => created,
        Err(error) => return Err(format!("{} cannot be opened: {error}", path.display())),
    };
    let metadata: fs::Metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => return Err(format!("{} cannot be inspected: {error}", path.display())),
    };
    if !metadata.is_file() {
        return Err(not_ordinary(path, &metadata));
    }
    return Ok(file);
}

/// What:
/// SHA-256 of the whole ordinary file at `path`,
/// read in blocks so no file is held in memory.
/// Why:
/// The judgement of a removal keeps this digest,
/// and it is what catches an edit in place whose writer put the modification time back.
pub(crate) fn sha256_of_ordinary(path: &Path) -> Result<[u8; 32], String> {
    let mut file: fs::File = open_ordinary(path)?;
    let mut hasher: Sha256 = Sha256::new();
    let mut buffer: Vec<u8> = vec![0; READ_BLOCK];
    let mut seen: u64 = 0;
    loop {
        if seen > MAX_ORDINARY_BYTES {
            return Err(larger_than_read(path, MAX_ORDINARY_BYTES));
        }
        let read: usize = match file.read(&mut buffer) {
            Ok(count) => count,
            Err(error) => return Err(format!("{} cannot be read: {error}", path.display())),
        };
        if read == 0 {
            break;
        }
        seen += u64::try_from(read).unwrap_or(u64::MAX);
        hasher.update(&buffer[..read]);
        #[cfg(test)]
        seam::after_a_block(path);
    }
    return Ok(hasher.finalize().into());
}

/// SHA-256 of bytes already in memory,
/// such as a symbolic link's stated target.
pub(crate) fn sha256_of_bytes(bytes: &[u8]) -> [u8; 32] {
    return Sha256::digest(bytes).into();
}

/// What:
/// The sentence for a name that holds something this tool refuses to read.
/// Why:
/// The reader has to say what stands there,
/// because what to do about it (a pipe, a directory, a socket) is the owner's decision,
/// not this tool's.
pub(crate) fn not_ordinary(path: &Path, metadata: &fs::Metadata) -> String {
    return format!("{} is not an ordinary file (it is {})", path.display(), kind_of(metadata));
}

/// The kind of entry `metadata` describes,
/// in the words a person reads in a warning.
fn kind_of(metadata: &fs::Metadata) -> &'static str {
    /// Imports `FileTypeExt` from `std::os::unix::fs`,
    /// which tells the kinds of entry the standard library's own methods do not name.
    #[cfg(unix)]
    use std::os::unix::fs::FileTypeExt;
    #[cfg(unix)]
    {
        let kind: fs::FileType = metadata.file_type();
        if kind.is_fifo() {
            return "a named pipe";
        }
        if kind.is_socket() {
            return "a socket";
        }
        if kind.is_char_device() {
            return "a character device";
        }
        if kind.is_block_device() {
            return "a block device";
        }
        if kind.is_symlink() {
            return "a symbolic link";
        }
    }
    if metadata.is_dir() {
        return "a directory";
    }
    return "not an ordinary file";
}

/// Opens `path` for reading without waiting and without following a link.
#[cfg(unix)]
fn open_without_waiting(path: &Path) -> std::io::Result<fs::File> {
    // O_NONBLOCK: a named pipe answers at once with nothing to read instead of waiting for a writer.
    // O_NOFOLLOW: a symbolic link is refused, so the entry read is the entry named.
    // O_CLOEXEC: the descriptor never reaches a program this tool starts.
    let descriptor: rustix::fd::OwnedFd = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )?;
    return Ok(fs::File::from(descriptor));
}

/// Opens `path` for reading on a platform with no call that asks a read to wait for nothing;
/// what the name holds is refused after the open instead, which is all that platform gives.
#[cfg(not(unix))]
fn open_without_waiting(path: &Path) -> std::io::Result<fs::File> {
    return fs::File::open(path);
}

/// Opens or creates `path` for reading and writing without waiting and without following a link.
#[cfg(unix)]
fn create_without_waiting(path: &Path) -> std::io::Result<fs::File> {
    let descriptor: rustix::fd::OwnedFd = rustix::fs::open(
        path,
        rustix::fs::OFlags::RDWR
            | rustix::fs::OFlags::CREATE
            | rustix::fs::OFlags::NONBLOCK
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        NEW_FILE_MODE,
    )?;
    return Ok(fs::File::from(descriptor));
}

/// Opens or creates `path` for reading and writing on a platform with no such call,
/// and refuses what it is not after the open.
#[cfg(not(unix))]
fn create_without_waiting(path: &Path) -> std::io::Result<fs::File> {
    return fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(path);
}

/// What:
/// The seam a control uses to make a file grow between two blocks of a read.
/// Why:
/// A file another command keeps open for appending grows under a reader only if that writer runs between two reads;
/// this is where it runs, in tests only, so the growth is placed by the code and never by a sleep.
#[cfg(test)]
pub(crate) mod seam {
    /// Imports `Mutex` from `std`.
    use std::sync::Mutex;
    /// Imports `Path` from `std::path`.
    use std::path::Path;

    /// The hook a control sets, run after every block a read takes.
    static AFTER_A_BLOCK: Mutex<Option<Box<dyn Fn(&Path) + Send>>> = Mutex::new(None);

    /// Sets (or with `None` clears) the hook run after a read block.
    pub(crate) fn set_after_a_block(hook: Option<Box<dyn Fn(&Path) + Send>>) {
        *AFTER_A_BLOCK.lock().expect("the seam is not poisoned") = hook;
    }

    /// Runs the hook, when a test set one.
    pub(crate) fn after_a_block(path: &Path) {
        if let Some(hook) = AFTER_A_BLOCK.lock().expect("the seam is not poisoned").as_ref() {
            hook(path);
        }
    }
}
