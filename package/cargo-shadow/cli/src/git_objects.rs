//! What:
//! The check that an object name in a reference names a commit that exists,
//! made from the object files of a repository (a loose object,
//! or an entry of a pack index) without starting Git.
//! Why:
//! Git's `X^{commit}` establishes that the object exists,
//! is a commit (a tag is peeled),
//! and is not replaced.
//! A reference file alone proves none of it,
//! so the reader accepts a commit only after it has seen the object's type,
//! and everything it cannot see (alternates,
//! grafts,
//! replace references,
//! deltified or tag objects,
//! a missing object)
//! is an error that leaves the answer to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-objects.ts
//! export function confirmCommit(commonDir: string, object: string): void  // throws when the object is not seen to be a commit
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
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

// Imports `flate2::read::ZlibDecoder`.
/// Imports the zlib decoder that reads a loose object header.
use flate2::{Decompress, FlushDecompress, Status};

/// The object name length of a SHA-1 repository,
/// in hexadecimal digits.
pub const OBJECT_NAME_DIGITS: usize = 40;

/// The bytes of a SHA-1 object name.
const OBJECT_NAME_BYTES: usize = OBJECT_NAME_DIGITS / 2;

/// The signature and version that start a version 2 pack index.
const PACK_INDEX_HEADER: [u8; 8] = [0xff, b't', b'O', b'c', 0, 0, 0, 2];

/// The number of fan-out entries of a pack index.
const FANOUT_ENTRIES: usize = 256;

/// The pack entry type of a commit that is stored whole.
const PACK_TYPE_COMMIT: u8 = 1;

/// What:
/// `fn index_failure(error: std::io::Error) -> String` words a failed read of a pack index.
/// Why:
/// Every read of the index reports the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const indexFailure = (error: Error) => `a pack index cannot be read: ${error}`;
/// ```
fn index_failure(error: std::io::Error) -> String {
    return format!("a pack index cannot be read: {error}");
}

/// What:
/// `fn pack_failure(error: std::io::Error) -> String` words a failed read of a pack.
/// Why:
/// Every read of a pack reports the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const packFailure = (error: Error) => `a pack cannot be read: ${error}`;
/// ```
fn pack_failure(error: std::io::Error) -> String {
    return format!("a pack cannot be read: {error}");
}

/// What:
/// `fn hex_to_bytes(hex: &str) -> Option<[u8; OBJECT_NAME_BYTES]>` decodes an object name.
/// Why:
/// The pack index stores names as raw bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hexToBytes = (hex: string): Uint8Array | undefined => ...
/// ```
fn hex_to_bytes(hex: &str) -> Option<[u8; OBJECT_NAME_BYTES]> {
    if hex.len() != OBJECT_NAME_DIGITS {
        // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
        // Why:      Absence is a value the caller must check; there is no `null`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Some(value) is  value
        // // None        is  undefined
        // ```
        return None;
    }
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut bytes: [u8; OBJECT_NAME_BYTES] = [0; OBJECT_NAME_BYTES];
    // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
    // Why:      The collection stays usable after the walk and no item is copied until it must be.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const item of list) { ... }  // or list.map(...)
    // ```
    for (index, byte) in bytes.iter_mut().enumerate() {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // What:     `.ok()` turns a `Result` into an `Option` (dropping the error); `.ok_or(e)` turns an `Option` into a `Result` with the given error.
        // Why:      The next step wants a value-or-nothing, or a value-or-error, not the other form.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { return compute(); } catch { return undefined; }
        // ```
        *byte = u8::from_str_radix(hex.get(index * 2..index * 2 + 2)?, 16).ok()?;
    }
    // `Some`: the present or the absent value of an `Option`.
    return Some(bytes);
}

/// What:
/// `fn read_at(file: &File, offset: u64, length: usize) -> std::io::Result<Vec<u8>>` reads exactly `length` bytes at `offset`.
/// Why:
/// A pack index is searched by positions,
/// and reading only the needed bytes keeps a large index out of memory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readAt = (file: number, offset: number, length: number): Buffer => ...
/// ```
fn read_at(file: &File, offset: u64, length: usize) -> std::io::Result<Vec<u8>> {
    // `let mut`: a binding that may change.
    // Typed binding: `bytes: Vec<u8>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut bytes: Vec<u8> = vec![0; length];
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&mut bytes`: lends the value without copying or giving it away.
    file.read_exact_at(&mut bytes, offset)?;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(bytes);
}

/// What:
/// `fn fanout_entry(fanout: &[u8], position: usize) -> u64` reads one four-byte entry of a pack index fan-out table.
/// Why:
/// A function replaces the closure that captured the table.
fn fanout_entry(fanout: &[u8], position: usize) -> u64 {
    return big_endian(&fanout[position * 4..position * 4 + 4]);
}

/// What:
/// `fn is_object_name(bytes: &[u8]) -> bool` says whether bytes are the hexadecimal name of a Git object.
/// Why:
/// A function replaces the closure that checked the tree and parent names of a commit.
fn is_object_name(bytes: &[u8]) -> bool {
    return bytes.len() == OBJECT_NAME_DIGITS && bytes.iter().all(u8::is_ascii_hexdigit);
}

/// What:
/// `fn is_digit_byte(byte: u8) -> bool` says whether a byte is `0` to `9`.
/// Why:
/// `bytes().all` takes it by path instead of a closure.
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
/// `fn big_endian(bytes: &[u8]) -> u64` decodes a big-endian unsigned integer.
/// Why:
/// Pack indexes store their numbers that way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bigEndian = (bytes: Uint8Array): bigint => ...
/// ```
fn big_endian(bytes: &[u8]) -> u64 {
    // Closure `|total, byte|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    let mut total: u64 = 0;
    for byte in bytes {
        total = (total << 8) | u64::from(*byte);
    }
    return total;
}

/// What:
/// `fn pack_offset(index: &File, name: &[u8; OBJECT_NAME_BYTES]) -> Result<Option<u64>, String>` finds the offset of an object in
/// a version 2 pack index,
/// or `None` when the index does not list it.
/// Why:
/// The fan-out table narrows the search to the names that start with the same byte,
/// then a binary search over the sorted names finds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packOffset(index: number, name: Uint8Array): number | undefined
/// ```
fn pack_offset(index: &File, name: &[u8; OBJECT_NAME_BYTES]) -> Result<Option<u64>, String> {
    // Closure `|error: std::io::Error|`: an anonymous function, like a TS arrow function.
    // Macro `format!`: builds or prints text (or a list) from a template.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `header: Vec<u8>`.
    let header: Vec<u8> = read_at(index, 0, PACK_INDEX_HEADER.len()).map_err(index_failure)?;
    if header != PACK_INDEX_HEADER {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a pack index is not of version 2".to_string());
    }
    // Typed binding: `fanout_start: u64`.
    // Cast `as u64`: converts between numeric types.
    let fanout_start: u64 = PACK_INDEX_HEADER.len() as u64;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `fanout: Vec<u8>`.
    let fanout: Vec<u8> = read_at(index, fanout_start, FANOUT_ENTRIES * 4).map_err(index_failure)?;
    // Closure `|position: usize|`: an anonymous function, like a TS arrow function.
    // Borrow `&fanout`: lends the value without copying or giving it away.
    // Typed binding: `total: u64`.
    let total: u64 = fanout_entry(&fanout, FANOUT_ENTRIES - 1);
    // Typed binding: `first_byte: usize`.
    let first_byte: usize = usize::from(name[0]);
    // What:     `if let Pattern = value { ... }` runs the block only when the value fits the pattern and binds its parts; `let Pattern = value else { ... }` does the same but must leave the function or loop in its `else` block.
    // Why:      It extracts the inside of an `Option` or `Result` in one step when only one case matters.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value !== undefined) { const inner = value; ... }
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `low: u64`.
    let mut low: u64 = if first_byte == 0 { 0 } else { fanout_entry(&fanout, first_byte - 1) };
    // `let mut`: a binding that may change.
    // Typed binding: `high: u64`.
    let mut high: u64 = fanout_entry(&fanout, first_byte);
    // Typed binding: `names_start: u64`.
    // Cast `as u64`: converts between numeric types.
    let names_start: u64 = fanout_start + (FANOUT_ENTRIES as u64) * 4;
    while low < high {
        // Typed binding: `middle: u64`.
        let middle: u64 = low + (high - low) / 2;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `found: Vec<u8>`.
        // Cast `as u64`: converts between numeric types.
        let found: Vec<u8> = read_at(index, names_start + middle * OBJECT_NAME_BYTES as u64, OBJECT_NAME_BYTES).map_err(index_failure)?;
        // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
        // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
        // ```
        match found.as_slice().cmp(name.as_slice()) {
            std::cmp::Ordering::Equal => {
                // Typed binding: `offsets_start: u64`.
                // Cast `as u64`: converts between numeric types.
                let offsets_start: u64 = names_start + total * OBJECT_NAME_BYTES as u64 + total * 4;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Borrow `&read_at`: lends the value without copying or giving it away.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                // Typed binding: `small: u64`.
                let small: u64 = big_endian(&read_at(index, offsets_start + middle * 4, 4).map_err(index_failure)?);
                if small & 0x8000_0000 == 0 {
                    // `Ok(...)`: builds the success or the failure variant of a `Result`.
                    // `Some`: the present or the absent value of an `Option`.
                    return Ok(Some(small));
                }
                // Typed binding: `large_start: u64`.
                let large_start: u64 = offsets_start + total * 4;
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                // Typed binding: `large: Vec<u8>`.
                let large: Vec<u8> = read_at(index, large_start + (small & 0x7fff_ffff) * 8, 8).map_err(index_failure)?;
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // `Some`: the present or the absent value of an `Option`.
                // Borrow `&large`: lends the value without copying or giving it away.
                return Ok(Some(big_endian(&large)));
            }
            std::cmp::Ordering::Less => low = middle + 1,
            std::cmp::Ordering::Greater => high = middle,
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    return Ok(None);
}

/// The most bytes of one commit object (compressed or not) that the reader will read to confirm it.
/// A bound chosen for the owner's rule of 2026-11-03 ("nothing should ever wait forever or indefinitely") and far above any
/// real commit;
/// a larger object is Git's to read.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
const MAX_COMMIT_BYTES: u64 = 16 * 1024 * 1024;

/// The length of a pack file's header (`PACK`,
/// version,
/// object count).
const PACK_HEADER_BYTES: usize = 12;

/// What:
/// `pub(crate) fn open_regular(path: &Path) -> Result<File, String>` opens a file that the object reader may read,
/// or says why only Git reads it.
/// Why:
/// Round 6 (5a F7):
/// the check and the open must be one operation on the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openRegular(path: string): FileHandle  // throws when the path is not a regular file
/// ```
pub(crate) fn open_regular(path: &Path) -> Result<File, String> {
    // Opened without waiting and without following a final link: a path swapped for a FIFO after any earlier check cannot block
    // the open, and the descriptor that was opened is the one inspected (round 6, 5a F7).
    let flags: i32 = (rustix::fs::OFlags::NONBLOCK | rustix::fs::OFlags::NOFOLLOW).bits().try_into().unwrap_or(0);
    let file: File = std::os::unix::fs::OpenOptionsExt::custom_flags(std::fs::OpenOptions::new().read(true), flags).open(path).described(format_args!("{} cannot be read", path.display()))?;
    let is_file: bool = match file.metadata() {
        Ok(metadata) => metadata.file_type().is_file(),
        Err(_) => false,
    };
    if !is_file {
        return Err(format!("{} is not a regular file, so only Git reads it", path.display()));
    }
    return Ok(file);
}

/// What:
/// `fn looks_like_commit(body: &[u8]) -> bool` says whether a commit body passes the checks of Git's `parse_commit_buffer`.
/// Why:
/// The tree line and every parent line must be well formed and followed by more bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looksLikeCommit(body: Uint8Array): boolean
/// ```
fn looks_like_commit(body: &[u8]) -> bool {
    // Git's `parse_commit_buffer` (commit.c): the tree line must be followed by at least one more byte, and so must every
    // parent line (`tail <= bufptr + entry_len + 1` is an error). Mirrored exactly, never loosened (round 6, 5a F3 A).
    let tree_entry: usize = "tree ".len() + OBJECT_NAME_DIGITS;
    if body.len() <= tree_entry + 1 || !body.starts_with(b"tree ") || !is_object_name(&body[5..tree_entry]) || body[tree_entry] != b'\n' {
        return false;
    }
    let parent_entry: usize = "parent ".len() + OBJECT_NAME_DIGITS;
    let mut position: usize = tree_entry + 1;
    while position + parent_entry < body.len() && body[position..].starts_with(b"parent ") {
        if body.len() <= position + parent_entry + 1 || !is_object_name(&body[position + 7..position + parent_entry]) || body[position + parent_entry] != b'\n' {
            return false;
        }
        position += parent_entry + 1;
    }
    return true;
}

/// What:
/// `fn inflate_complete(source: impl Read, output_limit: usize, input_limit: u64) -> Result<Vec<u8>, String>` inflates a zlib
/// stream and succeeds only when the inflater reports the end of the stream (the checksum included).
/// Why:
/// Review finding (4a 2):
/// `read_to_end` returning is not proof that the stream ended,
/// and a limit reached is not an end;
/// the compressed input is bounded as well as the output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inflateComplete(source: Readable, outputLimit: number, inputLimit: number): Buffer  // throws unless the stream ended
/// ```
fn inflate_complete(mut source: impl Read, output_limit: usize, input_limit: u64) -> Result<(Vec<u8>, u64), String> {
    // `let mut`: a binding that may change.
    // Typed binding: `inflater: Decompress`.
    let mut inflater: Decompress = Decompress::new(true);
    // `let mut`: a binding that may change.
    // Typed binding: `input: Vec<u8>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut input: Vec<u8> = vec![0; 16 * 1024];
    // `let mut`: a binding that may change.
    // Typed binding: `available: usize`.
    let mut available: usize = 0;
    // `let mut`: a binding that may change.
    // Typed binding: `used: usize`.
    let mut used: usize = 0;
    // `let mut`: a binding that may change.
    // Typed binding: `read_total: u64`.
    let mut read_total: u64 = 0;
    // `let mut`: a binding that may change.
    // Typed binding: `produced_all: Vec<u8>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut produced_all: Vec<u8> = Vec::new();
    // `let mut`: a binding that may change.
    // Typed binding: `chunk: Vec<u8>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut chunk: Vec<u8> = vec![0; 16 * 1024];
    loop {
        // `let mut`: a binding that may change.
        // Typed binding: `at_end: bool`.
        let mut at_end: bool = false;
        if used == available {
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|error|`: an anonymous function, like a TS arrow function.
            // Borrow `&mut input`: lends the value without copying or giving it away.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            // Macro `format!`: builds or prints text (or a list) from a template.
            available = source.read(&mut input).described(format_args!("an object could not be read"))?;
            used = 0;
            // Cast `as u64`: converts between numeric types.
            read_total += available as u64;
            at_end = available == 0;
            if read_total > input_limit {
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                return Err("an object is larger than this reader confirms".to_string());
            }
        }
        // Typed binding: `before_in: u64`.
        let before_in: u64 = inflater.total_in();
        // Typed binding: `before_out: u64`.
        let before_out: u64 = inflater.total_out();
        // `None`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&input`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let status = inflater.decompress(&input[used..available], &mut chunk, FlushDecompress::None).described(format_args!("an object is not a valid zlib stream"))?;
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        used += usize::try_from(inflater.total_in() - before_in).unwrap_or(0);
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Typed binding: `produced: usize`.
        let produced: usize = usize::try_from(inflater.total_out() - before_out).unwrap_or(0);
        // Borrow `&chunk`: lends the value without copying or giving it away.
        produced_all.extend_from_slice(&chunk[..produced]);
        if produced_all.len() > output_limit {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Err("an object inflates to more than this reader confirms".to_string());
        }
        // `match`: picks the arm whose pattern fits the value.
        match status {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Status::StreamEnd => return Ok((produced_all, inflater.total_in())),
            Status::Ok | Status::BufError => {
                if at_end && produced == 0 && inflater.total_in() == before_in {
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    return Err("an object's zlib stream ends before its end marker".to_string());
                }
            }
        }
    }
}

/// What:
/// `fn canonical_size(text: &str) -> Option<usize>` reads a size the way Git's loose header does:
/// decimal digits,
/// no sign,
/// and
/// no leading zero unless the size is zero.
/// Why:
/// Review finding (4a 1):
/// Rust's `parse` also takes `+12` and `012`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const canonicalSize = (text: string): number | undefined => /^(0|[1-9][0-9]*)$/.test(text) ? Number(text) : undefined;
/// ```
fn canonical_size(text: &str) -> Option<usize> {
    // Closure `|byte|`: an anonymous function, like a TS arrow function.
    if text.is_empty() || !text.bytes().all(is_digit_byte) || (text.len() > 1 && text.starts_with('0')) {
        // `None`: the present or the absent value of an `Option`.
        return None;
    }
    // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
    return text.parse().ok();
}

/// What:
/// `fn loose_commit(path: &Path) -> Result<(), String>` inflates a whole loose object and confirms that it is a commit:
/// a complete stream,
/// a `commit <size>` header with a canonical size,
/// exactly that many bytes of body,
/// and a body Git's commit
/// parser accepts.
/// Why:
/// A prefix that merely says `commit`,
/// or a stream that is cut off,
/// is not an object `X^{commit}` accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looseCommit(path: string): void  // throws when the object is not a whole commit
/// ```
fn loose_commit(path: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<(), String> {
    let file: File = open_regular(path)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|message|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `inflated: Vec<u8>`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let length: u64 = file.metadata().described(format_args!("{} cannot be inspected", path.display()))?.len();
    let (inflated, consumed): (Vec<u8>, u64) = inflate_complete(file, usize::try_from(MAX_COMMIT_BYTES).unwrap_or(usize::MAX), MAX_COMMIT_BYTES).described(format_args!("{}", path.display()))?;
    let mut header_end: Option<usize> = None;
    for (position, byte) in inflated.iter().enumerate() {
        if *byte == 0 {
            header_end = Some(position);
            break;
        }
    }
    let Some(end) = header_end else {
        return Err(format!("{} has no object header", path.display()));
    };
    let header: String = String::from_utf8_lossy(&inflated[..end]).to_string();
    let Some((kind, size)) = header.split_once(' ') else {
        return Err(format!("{} has the object header {header:?}", path.display()));
    };
    if kind != "commit" {
        return Err(format!("{} is a {kind}, not a commit", path.display()));
    }
    let body: &[u8] = &inflated[end + 1..];
    // `Some`: the present or the absent value of an `Option`.
    if canonical_size(size) != Some(body.len()) || !looks_like_commit(body) {
        return Err(format!("{} is not a well-formed commit", path.display()));
    }
    // Git rejects bytes after the end of a loose object's stream ("garbage at end of loose object") and checks the signature.
    if consumed != length {
        return Err(format!("{} has bytes after its compressed stream, which only Git judges", path.display()));
    }
    if crate::git_sha1::object_name("commit", body) != *name {
        return Err(format!("{} does not hash to its own name, which only Git judges", path.display()));
    }
    return Ok(());
}

/// A reader of a file from a fixed offset,
/// for the inflater.
struct At<'a> {
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
fn check_pack_opens(pack: &File, index: &File) -> Result<(), String> {
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
fn packed_entry_is_commit(pack: &File, offset: u64, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String> {
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
fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String> {
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

/// What:
/// `fn refs_replace_in_use(common_dir: &Path) -> Result<bool, String>` says whether any `refs/replace/` reference exists,
/// loose or
/// packed.
/// Why:
/// A replace reference changes what Git reads for an object name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceRefsInUse(commonDir: string): boolean
/// ```
fn replace_refs_in_use(common_dir: &Path) -> Result<bool, String> {
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = common_dir.join("refs/replace");
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&loose`: lends the value without copying or giving it away.
    match fs::read_dir(&loose) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(mut entries) => {
            if entries.next().is_some() {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                return Ok(true);
            }
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(error) => return Err(format!("{} cannot be listed: {error}", loose.display())),
    }
    // Typed binding: `packed: PathBuf`.
    let packed: PathBuf = common_dir.join("packed-refs");
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&packed`: lends the value without copying or giving it away.
    return match fs::read_to_string(&packed) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(text) => Ok(text.contains(" refs/replace/")),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(error) => Err(format!("{} cannot be read: {error}", packed.display())),
    };
}

/// What:
/// `pub fn confirm_commit(common_dir: &Path, object: &str) -> Result<(), String>` succeeds only when the object is seen to be a
/// commit stored whole,
/// in the repository's own object files,
/// with no alternates,
/// grafts or replace references.
/// Why:
/// This is the part of `X^{commit}` that a reference file cannot supply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function confirmCommit(commonDir: string, object: string): void  // throws when the object is not seen to be a commit
/// ```
pub fn confirm_commit(common_dir: &Path, object: &str) -> Result<(), String> {
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    let Some(name) = hex_to_bytes(object) else {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{object:?} is not an object name of a SHA-1 repository"));
    };
    for unsupported in ["objects/info/alternates", "info/grafts"] {
        if common_dir.join(unsupported).exists() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("the repository has {unsupported}, which only Git applies"));
        }
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    if replace_refs_in_use(common_dir)? {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("the repository has replace references, which only Git applies".to_string());
    }
    // Typed binding: `objects: PathBuf`.
    let objects: PathBuf = common_dir.join("objects");
    // Borrow `&object`: lends the value without copying or giving it away.
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = objects.join(&object[..2]).join(&object[2..]);
    if loose.exists() {
        // Borrow `&loose`: lends the value without copying or giving it away.
        return loose_commit(&loose, &name);
    }
    if packed_commit(&objects, &name)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(format!("{object} was not seen as a commit in the object files"));
}
