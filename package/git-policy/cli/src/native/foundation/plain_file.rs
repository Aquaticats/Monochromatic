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
//! by creating it rather than by opening what is there,
//! so the type is taken from the open
//!  handle:
//! a name swapped for a link or a pipe between a check and the open cannot redirect the read or the write,
//!  and the read is bounded by the caller's limit.
//! A name the wrapper writes is one it made itself,
//! so anything already there is another process's entry:
//! no write empties it,
//! no read waits on it,
//! and no file at it is replaced.
//! A file a person names in a configuration is read through a link
//!  (`read_plain_file_through_links`):
//! the pipe is what must not be waited on there,
//!  and a link of the person's own is the path the configuration means.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bytes = readPlainFile(path, limit); // throws when the file is not a plain one
//! writePlainFile(path, bytes);              // the same, for a name the wrapper creates
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
const NOT_A_PLAIN_FILE: &str = "it is not a plain file (a link, a named pipe, a device or a directory)";

/// What:
///  The words that refuse a name something else already holds,
///  chosen by what is there.
/// Why:
///   The wrapper writes names it made itself,
/// so a file at one is somebody else's,
/// and the person reads which kind of entry kept it.
/// A hard link is the case the name alone cannot show:
/// it looks like the wrapper's own record,
/// and emptying it would replace the bytes of the file it is another name of.
fn taken_name_words(path: &Path) -> String {
    use std::os::unix::fs::MetadataExt as _;
    let Ok(found) = std::fs::symlink_metadata(path) else { return String::from(NOT_A_PLAIN_FILE) };
    if !found.file_type().is_file() {
        return String::from(NOT_A_PLAIN_FILE);
    }
    if found.nlink() > 1 {
        return String::from("a file another name is a hard link to is already there, and writing it would replace the bytes of that file too");
    }
    return String::from("a file is already there, and the wrapper writes its own record at that name rather than replacing one");
}

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
    return match String::from_utf8(read_plain_file(path, SMALL_FILE_LIMIT)?) {
        Ok(text) => Ok(text),
        Err(_not_text) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "it is not text")),
    };
}

/// What:
///  The file `path` names,
/// created for writing,
///  and refused when anything is already at the name.
/// Why:
///   The wrapper writes names it made itself,
/// so what is at one now is another process's entry,
/// and every way of writing it is a way of losing somebody's bytes:
/// a symbolic link carries the write to the file it points at,
/// a hard link shares that file's inode itself,
/// a named pipe waits for a reader,
/// and an ordinary file is emptied by the open alone.
/// Creating the name with `O_EXCL` refuses a link without following it,
/// answers at once for a pipe,
/// and leaves every file that is already there with its bytes,
///  so nothing at the name is emptied before the type can be examined,
/// and nothing is examined but the plain file this call made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openedForWriting(path: string): File { return openSync(path, 'wx', 0o666, O_NONBLOCK | O_CLOEXEC | O_NOFOLLOW); }
/// ```
fn opened_for_writing(path: &Path) -> std::io::Result<std::fs::File> {
    return match std::fs::OpenOptions::new().write(true).create_new(true).custom_flags(libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOFOLLOW).open(path) {
        Ok(opened) => Ok(opened),
        // `O_EXCL` refuses a link, a pipe, a device, a directory and a file alike, so the message is the only thing left
        //  that can say which of them kept the name.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, taken_name_words(path))),
        Err(error) => Err(error),
    };
}

/// What:
///  Write `bytes` to the plain file at `path`,
/// created here and never in place of what is there,
///  and refuse the name when it holds anything.
/// Why:
///   See the module description:
/// a write through a link loses the file it points at,
/// a write over a file that is already there loses that one,
/// and one to a named pipe
///  waits for a reader that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePlainFile(path, bytes); // throws when the name holds a link, a pipe, a device, a directory or a file
/// ```
pub fn write_plain_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;
    let mut file: std::fs::File = opened_for_writing(path)?;
    return file.write_all(bytes);
}

/// Plain-file controls stay out of the release executable.
#[cfg(test)]
#[path = "plain_file_tests.rs"]
mod tests;
