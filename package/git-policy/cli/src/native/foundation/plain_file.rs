//! What:
//!  Reading a file only when it is a plain one:
//! opened without blocking and without following a link,
//! refused when
//!  the open handle is a named pipe,
//! a device or a directory,
//! and never read past a limit;
//! and writing one the same way.
//! Why:
//!  Opening a named pipe for reading waits for a writer that may never come,
//! and a command that waits while it
//!  owns a checkout or a worktree blocks every other command that wants it,
//! with nothing said and no end.
//! A
//!  file beside a checkout can be made by any process that may write that directory,
//! and a `.git`,
//! a
//!  registration's `gitdir`,
//! a journal and a list of files are all read this way,
//! and a name the wrapper writes inside a directory of its own is written this way,
//! so the type is taken from the open
//!  handle:
//! a name swapped for a link or a pipe between a check and the open cannot redirect the read or the write,
//!  and the read is bounded by the caller's limit.
//! A file a person names in a configuration is read through a link
//!  (`read_plain_file_through_links`):
//! the pipe is what must not be waited on there,
//!  and a link of the person's own is the path the configuration means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = readPlainFile(path, limit); // throws when the file is not a plain one
//! writePlainFile(path, bytes);              // the same, for a name the wrapper writes
//! ```

use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

/// How many bytes of a small file this reads:
/// a journal,
/// a registration's `gitdir`,
/// a repository's configuration or a list of paths.
///  A file past the bound is refused with its path named rather than read whole,
/// so no record beside a checkout decides how much memory a command reads.
///  The measurement beside the value,
/// on this repository on 2026-11-09:
/// its `.git/config` is 74,878 bytes and
///  its longest registration `gitdir` 122.
///  A list of unreadable files has no measurement:
/// it holds one path per entry the migration
///  could not read.
/// A worktree's `.git` file is read with the smaller `GIT_FILE_LIMIT`
///  (`worktree_convert_inspection.rs`).
pub const SMALL_FILE_LIMIT: u64 = 1 << 20;

/// What:
///  The words that refuse everything but a plain file,
///  shared by both opens so the two read alike.
/// Why:
///   A person meets this text when a rules file is not one,
/// and it names what the path
///  holds instead.
const NOT_A_PLAIN_FILE: &str = "it is not a plain file (a named pipe, a device or a directory)";

/// What:
///  Whether the open follows a link the name holds,
///  or refuses it.
/// Why:
///   A named argument reads at the call site,
/// where a bare flag would not.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Links {
    /// The name is the file:
    /// a link is refused,
    /// so a name swapped for one cannot redirect the read.
    NotFollowed,
    /// The name may hold a link to the file,
    /// which is read:
    /// a link of a person's own is the path a
    /// configuration means.
    Followed,
}

/// What:
///  The file `path` names,
///  opened without blocking,
/// with `links` deciding whether a link is followed,
///  and refused when the open handle is not a plain file.
/// Why:
///   One body decides what a plain file is,
/// so the two opens differ in their flags alone and neither can be
///  read as a pipe or a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openedPlain(path, links) { const flags = O_NONBLOCK | O_CLOEXEC | (links === 'followed' ? 0 : O_NOFOLLOW); const file = openSync(path, 'r', 0o666, flags); return fstatSync(file).isFile() ? file : refuse(); }
/// ```
fn opened_plain(path: &Path, links: Links) -> std::io::Result<std::fs::File> {
    let following: libc::c_int = match links {
        Links::NotFollowed => libc::O_NOFOLLOW,
        Links::Followed => 0,
    };
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | following).open(path)?;
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE));
    }
    return Ok(file);
}

/// What:
///  The plain file at `path` itself:
/// opened without blocking and without following a link,
///  and refused when the open handle is a named pipe,
///  a device or a directory.
/// Why:
///   A caller that reads the file line by line needs the handle,
/// and the type is taken from it so a name
///  swapped for a pipe between a check and the open cannot redirect the read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openPlainFile(path: string): File; // throws
/// ```
pub fn open_plain_file(path: &Path) -> std::io::Result<std::fs::File> {
    return opened_plain(path, Links::NotFollowed);
}

/// What:
///  The bytes of the plain file at `path`,
///  read through the open `links` makes,
///  and refused when it is larger than `limit`.
/// Why:
///   One body decides the limit for both readers,
/// so the two can differ in the open alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPlainFileWith(path, limit, links) { const file = openedPlain(path, links); ...; }
/// ```
fn read_plain_file_with(path: &Path, limit: u64, links: Links) -> std::io::Result<Vec<u8>> {
    #[cfg(test)]
    note_a_read(path);
    let file: std::fs::File = opened_plain(path, links)?;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, limit + 1), &mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is larger than this command reads"));
    }
    return Ok(bytes);
}

/// What:
///  The bytes of the plain file at `path`,
/// opened without blocking and without following a link,
/// refused when it is
///  anything else or larger than `limit`.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPlainFile(path: string, limit: number): Buffer; // throws
/// ```
pub fn read_plain_file(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    return read_plain_file_with(path, limit, Links::NotFollowed);
}

/// What:
///  The bytes of the plain file `path` names,
///  opened without blocking and following a link,
///  refused when the open handle is
///  anything but a plain file or larger than `limit`.
/// Why:
///   A rules file is named in a configuration by a person,
///  who may keep it behind a link of their own:
/// following that
///  link is the path the configuration means,
/// while opening a named pipe for reading waits for a writer that may never
///  come.
/// The type is taken from the open handle here too,
/// so a name swapped for a pipe after a look cannot be
///  read at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPlainFileThroughLinks(path: string, limit: number): Buffer; // throws
/// ```
pub fn read_plain_file_through_links(path: &Path, limit: u64) -> std::io::Result<Vec<u8>> {
    return read_plain_file_with(path, limit, Links::Followed);
}

/// The text of the plain file at `path`,
/// read like `read_plain_file`.
pub fn read_plain_text(path: &Path) -> std::io::Result<String> {
    return read_plain_text_at(path, SMALL_FILE_LIMIT);
}

/// How many bytes of a mount table this reads:
/// the kernel's own,
/// under `/proc/<pid>/mountinfo` or this process's.
/// Why:
///  The number of mounts is the system's state and not this command's:
/// a container's namespace can hold tens of thousands of entries,
/// and each one is read into this process's memory whole.
///  Measured on this host on 2026-11-12:
/// this process's own table was 5,250 bytes.
///  A table past the bound is refused rather than read,
///  and a caller that refuses a mount it cannot see treats the file as one that may be held.
pub const MOUNT_TABLE_LIMIT: u64 = 4 << 20;

/// What:
///  The text of the plain file at `path`,
///  refused when it is larger than `limit` or not text.
/// Why:
///   A file under `/proc` is text the kernel writes,
///  and what it holds can be the system's state or another process's:
///  the limit is the caller's,
///  because one bound for every such file would be too small for a mount table
///  and too large for a process name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPlainTextAt(path: string, limit: number): string;
/// ```
pub fn read_plain_text_at(path: &Path, limit: u64) -> std::io::Result<String> {
    return match String::from_utf8(read_plain_file(path, limit)?) {
        Ok(text) => Ok(text),
        Err(_not_text) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is not text")),
    };
}

/// What:
///  The file `path` names,
/// created when it is not there and emptied when it is,
/// opened without blocking and without following a link,
///  and refused when the open handle is not a plain file.
/// Why:
///   A write that follows a link replaces the file it points at,
/// and one that opens a named pipe waits for a reader;
///  both are entries another process may put at a name the wrapper writes,
/// so the type of what was opened decides and never the name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openedForWriting(path: string): File { const file = openSync(path, 'w', 0o666, O_NONBLOCK | O_CLOEXEC | O_NOFOLLOW); return fstatSync(file).isFile() ? file : refuse(); }
/// ```
fn opened_for_writing(path: &Path) -> std::io::Result<std::fs::File> {
    let file: std::fs::File = match std::fs::OpenOptions::new().write(true).create(true).truncate(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW).open(path) {
        Ok(opened) => opened,
        // A named pipe opened for writing without a reader answers this at once, so the caller's message says what the name
        //  holds rather than what the kernel calls it.
        Err(error) if error.raw_os_error() == Some(libc::ENXIO) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE)),
        Err(error) => return Err(error),
    };
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE));
    }
    return Ok(file);
}

/// What:
///  The plain file at `path` opened to append to it,
/// created when it is not there,
/// without blocking,
/// without following a link,
///  and refused when the name holds anything but a plain file.
/// Why:
///   A record is appended to while a command runs,
///  and the folder it lives in is one a person may write:
///  a named pipe at that name held a Git start for good before this open existed,
///  because opening a pipe for writing waits for a reader
///  (a review of `429503238b`,
/// finding 4).
///  `O_NONBLOCK` makes a pipe answer `ENXIO` at once and `O_NOFOLLOW` refuses a link,
///  so the type of what was opened, not the name, decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openSync(path, 'a', 0o666, O_NONBLOCK | O_CLOEXEC | O_NOFOLLOW); if (!fstatSync(file).isFile()) refuse();
/// ```
pub fn open_plain_file_to_append(path: &Path) -> std::io::Result<std::fs::File> {
    let file: std::fs::File = match std::fs::OpenOptions::new().append(true).create(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW).open(path) {
        Ok(opened) => opened,
        // A named pipe opened for writing without a reader answers this at once, and the caller's message says what the name
        //  holds rather than what the kernel calls it.
        Err(error) if error.raw_os_error() == Some(libc::ENXIO) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE)),
        Err(error) => return Err(error),
    };
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE));
    }
    return Ok(file);
}

/// What:
///  The plain file at `path` made for the first time,
/// refusing a name that already holds anything,
/// a link,
/// a pipe or a device,
/// and without blocking.
/// Why:
///   A record's first line creates its file,
///  and a name that is already taken is another process's record or a person's file,
///  which this call must not truncate or follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openSync(path, 'wx', 0o666, O_NONBLOCK | O_CLOEXEC | O_NOFOLLOW); if (!fstatSync(file).isFile()) refuse();
/// ```
pub fn create_plain_file(path: &Path) -> std::io::Result<std::fs::File> {
    let file: std::fs::File = match std::fs::OpenOptions::new().write(true).create_new(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW).open(path) {
        Ok(opened) => opened,
        Err(error) if error.raw_os_error() == Some(libc::ENXIO) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE)),
        Err(error) => return Err(error),
    };
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE));
    }
    return Ok(file);
}

/// What:
///  Which file a plain-file open really reached:
///  the device it lives on,
///  its inode,
///  and its length.
/// Why:
///  A record read a moment ago is not the record that stands now:
///  a writer's child may append a line after the reading,
///  and a removal that acted on the older text unlinked the proof of a lock nobody could then explain
///  (a review of `b0ea691cc1`,
/// finding 3).
///  The device and inode say it is the same file,
///  and the length says no byte has been added or taken away since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlainFileIdentity = { device: number; inode: number; length: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlainFileIdentity {
    /// The device the file lives on.
    device: u64,
    /// Its inode on that device.
    inode: u64,
    /// Its length in bytes when it was read.
    length: u64,
}

/// What:
///  Whether two readings are of one unchanged file.
/// Why:
///  Only then may a decision made on the first be acted on with the second:
///  a replacement on one device,
///  a different inode,
///  or another length is another file or another text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const agrees = (a, b) => a.device === b.device && a.inode === b.inode && a.length === b.length;
/// ```
impl PlainFileIdentity {
    /// The identity of an open file's own metadata.
    pub fn of(metadata: &std::fs::Metadata) -> Self {
        use std::os::unix::fs::MetadataExt as _;
        return PlainFileIdentity { device: metadata.dev(), inode: metadata.ino(), length: metadata.len() };
    }

    /// Whether `other` is this same file with the same length.
    pub fn agrees_with(&self, other: &PlainFileIdentity) -> bool {
        return self.device == other.device && self.inode == other.inode && self.length == other.length;
    }
}

/// What:
///  The folder at `path` itself,
///  opened without following a link at its own last component,
///  and refused when what was opened is not a directory.
/// Why:
///  A folder the wrapper writes records in is one a person may write:
///  a link planted at its name sent a child's record line into a folder outside the repository,
///  and the check-then-open order left that window open
///  (a review of `b0ea691cc1`,
/// finding 2).
///  Every open below this descriptor is then relative to the folder that was really opened,
///  so no name between here and there can be a link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = openSync(path, O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC); if (!fstatSync(folder).isDirectory()) refuse();
/// ```
pub fn open_plain_directory(path: &Path) -> std::io::Result<std::fs::File> {
    let flags: rustix::fs::OFlags =
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::DIRECTORY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::CLOEXEC | rustix::fs::OFlags::NONBLOCK;
    let descriptor: rustix::fd::OwnedFd = rustix::fs::open(path, flags, rustix::fs::Mode::empty()).map_err(std::io::Error::from)?;
    let folder: std::fs::File = std::fs::File::from(descriptor);
    if !folder.metadata()?.file_type().is_dir() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE));
    }
    return Ok(folder);
}

/// What:
///  The bytes of the plain file at `path`,
///  with the identity of the file they were read from.
/// Why:
///  A removal that decides on a reading must act only while the file is still the one it read,
///  and the identity is what says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [bytes, identity] = readPlainFileWithIdentity(path, limit);
/// ```
pub fn read_plain_file_with_identity(path: &Path, limit: u64) -> std::io::Result<(Vec<u8>, PlainFileIdentity)> {
    let file: std::fs::File = opened_plain(path, Links::NotFollowed)?;
    let identity: PlainFileIdentity = PlainFileIdentity::of(&file.metadata()?);
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, limit + 1), &mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is larger than this command reads"));
    }
    return Ok((bytes, identity));
}

/// What:
///  The bytes of the plain file `name` in the open folder `directory`,
///  with the identity of the file they were read from.
/// Why:
///  A record is read beside the removal that decides about it,
///  and the file the decision acts on must be the one this reading saw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [bytes, identity] = readPlainFileInWithIdentity(folder, name, limit);
/// ```
pub fn read_plain_file_in_with_identity(directory: &std::fs::File, name: &std::ffi::OsStr, limit: u64) -> std::io::Result<(Vec<u8>, PlainFileIdentity)> {
    #[cfg(test)]
    note_a_read(directory_name_and(directory, name).as_path());
    let flags: rustix::fs::OFlags = rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC;
    let file: std::fs::File = opened_in(directory, name, flags)?;
    let identity: PlainFileIdentity = PlainFileIdentity::of(&file.metadata()?);
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, limit + 1), &mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is larger than this command reads"));
    }
    return Ok((bytes, identity));
}

/// What:
///  Remove the plain file `name` from the open folder `directory`.
/// Why:
///  The removal is relative to the folder this process opened itself,
///  so a link at the folder's name cannot send it to another folder's file of the same name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unlinkatSync(folder, name, 0);
/// ```
pub fn unlink_in(directory: &std::fs::File, name: &std::ffi::OsStr) -> std::io::Result<()> {
    return rustix::fs::unlinkat(directory, name, rustix::fs::AtFlags::empty()).map_err(std::io::Error::from);
}

/// What:
///  The plain file `name` in the open folder `directory`,
///  created when it is not there,
///  and refused when the name holds a link,
///  a pipe,
///  a device or a folder.
/// Why:
///  The record file is made once and appended to afterwards,
///  and the folder is the one this process opened itself,
///  so a link at the folder's name cannot redirect this:
///  a link at the name itself is still refused,
///  and a pipe is opened without waiting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openatSync(folder, name, O_WRONLY | O_CREAT | O_APPEND | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC, 0o600); if (!fstatSync(file).isFile()) refuse();
/// ```
pub fn open_plain_file_to_append_in(directory: &std::fs::File, name: &std::ffi::OsStr) -> std::io::Result<std::fs::File> {
    let flags: rustix::fs::OFlags =
        rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::APPEND | rustix::fs::OFlags::CREATE | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC;
    return opened_in(directory, name, flags);
}

/// What:
///  The plain file `name` in the open folder `directory`,
///  made for the first time,
///  refusing a name that already holds anything.
/// Why:
///  See `open_plain_file_to_append_in`: the folder is one this process opened,
///  and the name is refused rather than followed or truncated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openatSync(folder, name, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC, 0o600); if (!fstatSync(file).isFile()) refuse();
/// ```
pub fn create_plain_file_in(directory: &std::fs::File, name: &std::ffi::OsStr) -> std::io::Result<std::fs::File> {
    let flags: rustix::fs::OFlags =
        rustix::fs::OFlags::WRONLY | rustix::fs::OFlags::CREATE | rustix::fs::OFlags::EXCL | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC;
    return opened_in(directory, name, flags);
}

/// What:
///  The bytes of the plain file `name` in the open folder `directory`,
///  read without following a link and refused when it is larger than `limit`.
/// Why:
///  A record is read beside the removal that decides about it,
///  and both reads must be of the one folder this process opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openatSync(folder, name, O_RDONLY | O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC); if (!fstatSync(file).isFile()) refuse(); readSync(file, limit)
/// ```
pub fn read_plain_file_in(directory: &std::fs::File, name: &std::ffi::OsStr, limit: u64) -> std::io::Result<Vec<u8>> {
    #[cfg(test)]
    note_a_read(directory_name_and(directory, name).as_path());
    let flags: rustix::fs::OFlags = rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::CLOEXEC;
    let file: std::fs::File = opened_in(directory, name, flags)?;
    let mut bytes: Vec<u8> = Vec::new();
    std::io::Read::read_to_end(&mut std::io::Read::take(&file, limit + 1), &mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is larger than this command reads"));
    }
    return Ok(bytes);
}

/// What:
///  The plain file `name` of the open folder `directory`,
///  opened with `flags`,
///  and refused when what was opened is anything but a plain file.
/// Why:
///  One body decides the type check for every relative open,
///  so a pipe, a device or a folder at a name is refused the same way wherever it is opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = openatSync(folder, name, flags, 0o600); if (!fstatSync(file).isFile()) refuse(file);
/// ```
fn opened_in(directory: &std::fs::File, name: &std::ffi::OsStr, flags: rustix::fs::OFlags) -> std::io::Result<std::fs::File> {
    let descriptor: rustix::fd::OwnedFd = match rustix::fs::openat(directory, name, flags, rustix::fs::Mode::RUSR | rustix::fs::Mode::WUSR) {
        Ok(opened) => opened,
        // A named pipe opened for writing without a reader answers this at once, so the caller's message says what the name
        //  holds rather than what the kernel calls it.
        Err(rustix::io::Errno::NXIO) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE)),
        Err(error) => return Err(std::io::Error::from(error)),
    };
    let file: std::fs::File = std::fs::File::from(descriptor);
    if !file.metadata()?.file_type().is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, NOT_A_PLAIN_FILE));
    }
    return Ok(file);
}

/// What:
///  A path for the test-only read note,
///  which records what was read by path.
/// Why:
///   The note exists for controls that ask which files a pass read,
///  and a relative open has no path of its own;
///  the join is only a name for the note and is never opened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = join(describe(folder), nameOf(name));
/// ```
#[cfg(test)]
fn directory_name_and(directory: &std::fs::File, name: &std::ffi::OsStr) -> std::path::PathBuf {
    use std::os::fd::AsRawFd as _;
    return std::path::PathBuf::from("/proc/self/fd").join(directory.as_raw_fd().to_string()).join(name);
}

/// What:
///  Write `bytes` to the plain file at `path`,
/// created when it is not there,
/// opened without blocking and without following a link,
///  and refused when the name holds anything else.
/// Why:
///   See the module description:
/// a write through a link loses the file it points at,
/// and one to a named pipe
///  waits for a reader that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePlainFile(path, bytes); // throws when the name holds a link, a pipe, a device or a directory
/// ```
pub fn write_plain_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;
    let mut file: std::fs::File = opened_for_writing(path)?;
    return file.write_all(bytes);
}

// What: The plain files this thread has read, and the writer that records them.
// Why:  A control says how many files a step opens, which is the cost that a machine under I/O pressure makes a person wait
//       for; a step that reads a queue of thousands of records to use one of them shows here
//       (`doc/planning/cli-git-commit-latency-alpha-11-against-alpha-10.md`). The record is per thread, so controls that
//       nextest runs beside each other do not see one another's reads.
#[cfg(test)]
thread_local! {
    /// The paths read on this thread since the last control took them.
    static READS: std::cell::RefCell<Vec<std::path::PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Records one read of the thread's own list,
/// which the controls read.
#[cfg(test)]
fn note_a_read(path: &Path) {
    READS.with(|reads| { reads.borrow_mut().push(path.to_path_buf()); });
}

/// What:
///  The plain files this thread has read since the last call,
///  taken and forgotten.
/// Why:
///  A control asks what a step it is about to run opens,
///  so the list starts empty at the question rather than at the start of the test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paths = takeReads();
/// ```
#[cfg(test)]
pub(crate) fn take_reads() -> Vec<std::path::PathBuf> {
    return READS.with(|reads| { return std::mem::take(&mut *reads.borrow_mut()); });
}

/// Plain-file controls stay out of the release executable.
#[cfg(test)]
#[path = "plain_file_tests.rs"]
mod tests;
