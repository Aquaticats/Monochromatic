//! What:
//! The check that a commit stored in a pack file is a commit:
//! the pack header,
//! the entry,
//! and the inflated object.
//! Why:
//! Split out of `git_objects.rs`,
//! which confirms that Git objects are commits,
//! to keep that file under the code-line limit.
//! The items are re-exported from `git_objects`,
//! so their paths there are unchanged.

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// The constants and the readers of the object reader that the packed check uses.
use crate::git_objects::{MAX_COMMIT_BYTES, OBJECT_NAME_BYTES, PACK_TYPE_COMMIT, inflate_complete, looks_like_commit, open_regular, pack_offset};
/// The reads and constants of the object reader that the packed check uses.
use crate::git_objects::{PACK_HEADER_BYTES, big_endian, index_failure, pack_failure, read_at};
/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::ExplainedError;
/// Imports `OsStr`,
/// whose `to_str` turns a file extension into text.
use std::ffi::OsStr;
/// Imports `fs` and `File` to open and list the object files.
use std::fs::{self, File};
// Imports `std::io::Read`.
/// Imports `Read`,
/// which adds `read_to_end` to the zlib decoder.
use std::io::Read;
// Imports `std::os::unix::fs::FileExt`.
/// Imports `FileExt`,
/// which adds positioned reads (`read_exact_at`) to an open file.
use std::os::unix::fs::FileExt;
// Imports `std::path::{Path, PathBuf}`.
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};


/// A reader of a file from a fixed offset,
/// for the inflater.
pub(crate) struct At<'a> {
    /// The pack file.
    file: &'a File,
    /// The offset of the next byte.
    offset: u64,
}

/// The methods of the type above.
impl Read for At<'_> {
    /// What:
    /// `fn read(&mut self, buffer) -> io::Result<usize>` reads at the offset and moves it.
    /// Why:
    /// The inflater reads a pack entry without moving the file's own position.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// read(buffer: Buffer): number  // readSync(fd, buffer, 0, buffer.length, position)
    /// ```
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `count: usize`.
        let count: usize = self.file.read_at(buffer, self.offset)?;
        // Cast `as u64`: converts between numeric types.
        self.offset += count as u64;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(count);
    }
}

/// What:
/// `fn check_pack_opens(pack: &File, index: &File) -> Result<(), String>` applies the checks Git makes when it opens a pack:
/// the header,
/// an object count equal to the index's,
/// and the pack's trailing checksum equal to the one the index records.
/// Why:
/// Review finding (4a 2):
/// a pack that Git would refuse to open must not be read as a source of commits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkPackOpens(pack: number, index: number): void  // throws
/// ```
pub(crate) fn check_pack_opens(pack: &File, index: &File) -> Result<(), String> {
    let header: Vec<u8> = read_at(pack, 0, PACK_HEADER_BYTES).map_err(pack_failure)?;
    let version: u64 = big_endian(&header[4..8]);
    if &header[..4] != b"PACK" || (version != 2 && version != 3) {
        return Err("a pack has an unknown header".to_string());
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `fanout_end: Vec<u8>`.
    let fanout_end: Vec<u8> = read_at(index, 8 + 255 * 4, 4).map_err(index_failure)?;
    // Borrow `&header`: lends the value without copying or giving it away.
    if big_endian(&header[8..12]) != big_endian(&fanout_end) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a pack's object count disagrees with its index".to_string());
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `index_length: u64`.
    let index_length: u64 = index.metadata().map_err(index_failure)?.len();
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `pack_length: u64`.
    let pack_length: u64 = pack.metadata().map_err(pack_failure)?.len();
    if index_length < 40 || pack_length < 32 {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a pack or its index is too short".to_string());
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `recorded: Vec<u8>`.
    let recorded: Vec<u8> = read_at(index, index_length - 40, 20).map_err(index_failure)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `actual: Vec<u8>`.
    let actual: Vec<u8> = read_at(pack, pack_length - 20, 20).map_err(pack_failure)?;
    if recorded != actual {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a pack's trailing checksum disagrees with its index".to_string());
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `fn packed_entry_is_commit(pack: &File, offset: u64) -> Result<bool, String>` confirms a pack entry:
/// a whole commit entry (not a
/// delta),
/// a complete zlib stream of exactly the declared size,
/// and a body Git's commit parser accepts.
/// Why:
/// Review finding (4a 2):
/// one type byte does not show the entry is readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packedEntryIsCommit(pack: number, offset: number): boolean  // throws when the entry cannot be read
/// ```
pub(crate) fn packed_entry_is_commit(pack: &File, offset: u64, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String> {
    let first: Vec<u8> = read_at(pack, offset, 1).map_err(pack_failure)?;
    let mut byte: u8 = first[0];
    let kind: u8 = (byte >> 4) & 0x7;
    if kind != PACK_TYPE_COMMIT {
        return Ok(false);
    }
    let mut size: u64 = u64::from(byte & 0x0f);
    let mut shift: u32 = 4;
    let mut position: u64 = offset + 1;
    while byte & 0x80 != 0 {
        if shift > 56 {
            return Err("a pack entry has an oversized length".to_string());
        }
        byte = read_at(pack, position, 1).map_err(pack_failure)?[0];
        size |= u64::from(byte & 0x7f) << shift;
        shift += 7;
        position += 1;
    }
    if size > MAX_COMMIT_BYTES {
        return Err("a packed commit is larger than this reader confirms".to_string());
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `body: Vec<u8>`.
    let (body, _consumed): (Vec<u8>, u64) = inflate_complete(At { file: pack, offset: position }, usize::try_from(size).unwrap_or(usize::MAX), MAX_COMMIT_BYTES)?;
    // A packed entry is the commit whose name the index lists only if its bytes hash to that name.
    return Ok(body.len() as u64 == size && looks_like_commit(&body) && crate::git_sha1::object_name("commit", &body) == *name);
}

/// What:
/// `fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String>` says whether some pack stores the
/// object as a whole,
/// readable commit,
/// in a pack Git would open.
/// Why:
/// A tag,
/// tree,
/// blob or deltified entry is not a commit this reader can vouch for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packedCommit(objects: string, name: Uint8Array): boolean  // throws when a pack cannot be read
/// ```
pub(crate) fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String> {
    let pack_directory: PathBuf = objects.join("pack");
    let listing = match fs::read_dir(&pack_directory) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("{} cannot be listed: {error}", pack_directory.display())),
    };
    for item in listing {
        let path: PathBuf = item.described(format_args!("{} cannot be listed", pack_directory.display()))?.path();
        if path.extension().and_then(OsStr::to_str) != Some("idx") {
            continue;
        }
        // Typed binding: `pack_path: PathBuf`.
        let pack_path: PathBuf = path.with_extension("pack");
        // Borrow `&path`: lends the value without copying or giving it away.
        let index: File = open_regular(&path)?;
        let Some(offset) = pack_offset(&index, name)? else {
            continue;
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&pack_path`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `pack: File`.
        let pack: File = open_regular(&pack_path)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&pack`: lends the value without copying or giving it away.
        check_pack_opens(&pack, &index)?;
        return packed_entry_is_commit(&pack, offset, name);
    }
    return Ok(false);
}
