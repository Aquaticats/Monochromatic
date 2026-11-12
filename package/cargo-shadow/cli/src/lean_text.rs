//! What:
//! Small named reads of paths and text that the lean-folder tools share.
//! Why:
//! Each is a named function in place of a closure that captures nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const nameOf = (path: string): string => basename(path);
//! ```

/// Directory entries,
/// files,
/// paths,
/// and the descriptor a file is opened on.
use std::{fs::{self, DirEntry}, io::{self, Read}, os::fd::OwnedFd, path::Path};

/// The file type of an opened entry.
use rustix::fs::{FileType, Mode, OFlags, fstat, open};

/// What:
/// `pub fn name_of(path: &Path) -> String` is the final component of a path as text,
/// empty for a path with none.
/// Why:
/// Names are compared as text throughout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const nameOf = (path: string): string => basename(path);
/// ```
pub fn name_of(path: &Path) -> String {
    return match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => String::new(),
    };
}

/// What:
/// `pub fn extension_is(path: &Path, wanted: &str) -> bool` says whether a path ends in the extension.
/// Why:
/// The readers pick `*.json`,
/// `*.d` and source files by extension.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const extensionIs = (path: string, wanted: string): boolean => extname(path) === `.${wanted}`;
/// ```
pub fn extension_is(path: &Path, wanted: &str) -> bool {
    return match path.extension() {
        Some(extension) => extension == wanted,
        None => false,
    };
}

/// What:
/// `pub fn is_directory(entry: &DirEntry) -> bool` says whether a directory entry is a directory (a link is not).
/// Why:
/// Walks never follow links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const isDirectory = (entry: Dirent): boolean => entry.isDirectory();
/// ```
pub fn is_directory(entry: &DirEntry) -> bool {
    return match entry.file_type() {
        Ok(kind) => kind.is_dir(),
        Err(_) => false,
    };
}

/// What:
/// `pub fn read_small_text(path: &Path, limit: usize) -> std::io::Result<String>` is the text of a regular file at a path of at most
/// `limit` bytes,
/// opened so that nothing at the path can hold the caller,
/// and a link at the path is followed.
/// Why:
/// Every name these tools read is a name in a directory others can write:
/// the tool's own state directory,
/// or a build tree.
/// A pipe at one holds a plain read until a writer arrives,
/// a device never reaches its end,
/// and a symbolic link reads a file the caller never named.
/// The open never waits on a pipe,
/// the entry it opened must be a regular file,
/// and the read stops one byte past the limit,
/// so a file longer than any file of its kind is reported instead of read.
/// A ledger is read through a link it was given:
/// reading cannot change what the link names,
/// and the workspace it names is the guard a removal needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readSmallText = (path: string, limit: number): string => readFileSync(path, { flag: 'r' }).toString().slice(0, limit + 1);
/// ```
pub fn read_small_text(path: &Path, limit: usize) -> io::Result<String> {
    return read_small_text_opened(path, limit, true);
}

/// What:
/// `pub fn read_unlinked_small_text(path: &Path, limit: usize) -> std::io::Result<String>` is `read_small_text` with one difference:
/// a symbolic link at the path fails instead of being followed.
/// Why:
/// The record of this tool's own is never read out of a file another name holds:
/// its name is fixed inside the tool's state directory,
/// so a link there is not the tool's file,
/// and following it would take the positions from a file this tool never wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readUnlinkedSmallText = (path: string, limit: number): string => readFileSync(path, { flag: 'r', noFollow: true }).toString().slice(0, limit + 1);
/// ```
pub fn read_unlinked_small_text(path: &Path, limit: usize) -> io::Result<String> {
    return read_small_text_opened(path, limit, false);
}

/// What:
/// `pub fn read_small_bytes(path: &Path, limit: usize) -> std::io::Result<Vec<u8>>` is the first `limit` bytes of a regular file at a
/// path,
/// opened so that nothing at the path can hold the caller,
/// and a link at the path is followed.
/// Why:
/// A dependency-information file is read for its first few hundred bytes and may be megabytes long,
/// so the caller wants a prefix and not the whole file,
/// and a pipe or a device at its name must not hold the pass that reads it.
/// The bytes are returned as they are:
/// what a file names is read even when the file is not text this build can decode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readSmallBytes = (path: string, limit: number): Buffer => readFileSync(path, { flag: 'r' }).subarray(0, limit);
/// ```
pub fn read_small_bytes(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    let file: fs::File = open_regular(path, true)?;
    let mut bytes: Vec<u8> = Vec::new();
    file.take(u64::try_from(limit).unwrap_or(u64::MAX)).read_to_end(&mut bytes)?;
    return Ok(bytes);
}

/// What:
/// `fn open_regular(path: &Path, follow_links: bool) -> std::io::Result<fs::File>` is the open every reader here shares.
/// Why:
/// The readers differ in one flag and in how much they read,
/// and the flags of the open are read in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openRegular(path: string, followLinks: boolean): number
/// ```
fn open_regular(path: &Path, follow_links: bool) -> io::Result<fs::File> {
    let mut flags: OFlags = OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NONBLOCK;
    if !follow_links {
        flags |= OFlags::NOFOLLOW;
    }
    let opened: OwnedFd = open(path, flags, Mode::empty()).map_err(io::Error::from)?;
    let kind: FileType = FileType::from_raw_mode(fstat(&opened).map_err(io::Error::from)?.st_mode);
    if kind != FileType::RegularFile {
        return Err(io::Error::other("not a regular file"));
    }
    return Ok(fs::File::from(opened));
}

/// What:
/// `fn read_small_text_opened(path: &Path, limit: usize, follow_links: bool) -> std::io::Result<String>` is the text read both text readers
/// share.
/// Why:
/// The two differ in one flag,
/// and the length no file of its kind exceeds is checked in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readSmallTextOpened(path: string, limit: number, followLinks: boolean): string
/// ```
fn read_small_text_opened(path: &Path, limit: usize, follow_links: bool) -> io::Result<String> {
    let file: fs::File = open_regular(path, follow_links)?;
    let mut text: String = String::new();
    file.take(u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1)).read_to_string(&mut text)?;
    if text.len() > limit {
        return Err(io::Error::other(format!("larger than the {limit} bytes any file of its kind is")));
    }
    return Ok(text);
}

/// What:
/// `pub fn all_hex(text: &str) -> bool` says whether every character is a hexadecimal digit.
/// Why:
/// Shard directories and unit hashes are hexadecimal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const allHex = (text: string): boolean => /^[0-9a-fA-F]*$/.test(text);
/// ```
pub fn all_hex(text: &str) -> bool {
    for character in text.chars() {
        if !character.is_ascii_hexdigit() {
            return false;
        }
    }
    return true;
}
