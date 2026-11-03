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
use flate2::read::ZlibDecoder;

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
    let entry = |position: usize| { return big_endian(&fanout[position * 4..position * 4 + 4]); };
    // Typed binding: `total: u64`.
    let total: u64 = entry(FANOUT_ENTRIES - 1);
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
    let mut low: u64 = if first_byte == 0 { 0 } else { entry(first_byte - 1) };
    // `let mut`: a binding that may change.
    // Typed binding: `high: u64`.
    let mut high: u64 = entry(first_byte);
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
const MAX_COMMIT_BYTES: u64 = 16 * 1024 * 1024;

/// The length of a pack file's header (`PACK`,
/// version,
/// object count).
const PACK_HEADER_BYTES: usize = 12;

/// What:
/// `fn looks_like_commit(body: &[u8]) -> bool` says whether a body starts the way every commit does:
/// `tree `,
/// a full object name
/// and a newline.
/// Why:
/// Review finding (3a 4):
/// a type word in a header proves nothing about the body Git must parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const looksLikeCommit = (body: Buffer): boolean => /^tree [0-9a-f]{40}\n/.test(body.toString('latin1', 0, 46));
/// ```
fn looks_like_commit(body: &[u8]) -> bool {
    // Typed binding: `prefix: &[u8]`.
    let prefix: &[u8] = b"tree ";
    return body.len() > prefix.len() + OBJECT_NAME_DIGITS
        && body.starts_with(prefix)
        // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
        && body[prefix.len()..prefix.len() + OBJECT_NAME_DIGITS].iter().all(u8::is_ascii_hexdigit)
        && body[prefix.len() + OBJECT_NAME_DIGITS] == b'\n';
}

/// What:
/// `fn loose_commit(path: &Path) -> Result<(), String>` inflates a whole loose object and confirms that it is a commit:
/// a complete stream,
/// a `commit <size>` header,
/// exactly that many bytes of body,
/// and a body that starts like a commit.
/// Why:
/// A prefix that merely says `commit`,
/// or a stream that is cut off,
/// is not an object `X^{commit}` accepts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looseCommit(path: string): void  // throws when the object is not a whole commit
/// ```
fn loose_commit(path: &Path) -> Result<(), String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `file: File`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let file: File = File::open(path).map_err(|error| { return format!("{} cannot be read: {error}", path.display()); })?;
    // `let mut`: a binding that may change.
    // Typed binding: `inflated: Vec<u8>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    let mut inflated: Vec<u8> = Vec::new();
    ZlibDecoder::new(file.take(MAX_COMMIT_BYTES))
        .take(MAX_COMMIT_BYTES + 1)
        // Borrow `&mut inflated`: lends the value without copying or giving it away.
        .read_to_end(&mut inflated)
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Macro `format!`: builds or prints text (or a list) from a template.
        .map_err(|error| { return format!("{} is not a complete object: {error}", path.display()); })?;
    // `Some`: the present or the absent value of an `Option`.
    // Closure `|byte|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    let Some(end) = inflated.iter().position(|byte| { return *byte == 0; }) else {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{} has no object header", path.display()));
    };
    // Borrow `&inflated`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `header: String`.
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let header: String = String::from_utf8_lossy(&inflated[..end]).to_string();
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    let Some((kind, size)) = header.split_once(' ') else {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{} has the object header {header:?}", path.display()));
    };
    if kind != "commit" {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{} is a {kind}, not a commit", path.display()));
    }
    // Borrow `&inflated`: lends the value without copying or giving it away.
    // Typed binding: `body: &[u8]`.
    let body: &[u8] = &inflated[end + 1..];
    // `Some`: the present or the absent value of an `Option`.
    // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
    if size.parse::<usize>().ok() != Some(body.len()) || !looks_like_commit(body) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{} is not a well-formed commit", path.display()));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
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
/// `fn packed_entry_is_commit(pack: &File, offset: u64) -> Result<bool, String>` confirms a pack entry:
/// the pack header,
/// a whole
/// commit entry (not a delta),
/// and an inflated body of exactly the declared size that starts like a commit.
/// Why:
/// Review finding (3a 4):
/// one type byte does not show the entry is readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packedEntryIsCommit(pack: number, offset: number): boolean  // throws when the entry cannot be read
/// ```
fn packed_entry_is_commit(pack: &File, offset: u64) -> Result<bool, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `header: Vec<u8>`.
    let header: Vec<u8> = read_at(pack, 0, PACK_HEADER_BYTES).map_err(pack_failure)?;
    // Borrow `&header`: lends the value without copying or giving it away.
    // Typed binding: `version: u64`.
    let version: u64 = big_endian(&header[4..8]);
    // Borrow `&header`: lends the value without copying or giving it away.
    if &header[..4] != b"PACK" || (version != 2 && version != 3) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a pack has an unknown header".to_string());
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `first: Vec<u8>`.
    let first: Vec<u8> = read_at(pack, offset, 1).map_err(pack_failure)?;
    // `let mut`: a binding that may change.
    // Typed binding: `byte: u8`.
    let mut byte: u8 = first[0];
    // Typed binding: `kind: u8`.
    let kind: u8 = (byte >> 4) & 0x7;
    if kind != PACK_TYPE_COMMIT {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(false);
    }
    // `let mut`: a binding that may change.
    // Typed binding: `size: u64`.
    let mut size: u64 = u64::from(byte & 0x0f);
    // `let mut`: a binding that may change.
    // Typed binding: `shift: u32`.
    let mut shift: u32 = 4;
    // `let mut`: a binding that may change.
    // Typed binding: `position: u64`.
    let mut position: u64 = offset + 1;
    while byte & 0x80 != 0 {
        if shift > 56 {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Err("a pack entry has an oversized length".to_string());
        }
        // `.map_err(...)`: converts the callee's error into this function's error type.
        byte = read_at(pack, position, 1).map_err(pack_failure)?[0];
        size |= u64::from(byte & 0x7f) << shift;
        shift += 7;
        position += 1;
    }
    if size > MAX_COMMIT_BYTES {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a packed commit is larger than this reader confirms".to_string());
    }
    // `let mut`: a binding that may change.
    // Typed binding: `body: Vec<u8>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut body: Vec<u8> = Vec::new();
    ZlibDecoder::new(At { file: pack, offset: position })
        .take(size + 1)
        // Borrow `&mut body`: lends the value without copying or giving it away.
        .read_to_end(&mut body)
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Macro `format!`: builds or prints text (or a list) from a template.
        .map_err(|error| { return format!("a packed commit is not a complete stream: {error}"); })?;
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // Borrow `&body`: lends the value without copying or giving it away.
    // Cast `as u64`: converts between numeric types.
    return Ok(body.len() as u64 == size && looks_like_commit(&body));
}

/// What:
/// `fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String>` says whether some pack stores the
/// object as a whole,
/// readable commit.
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
        let path: PathBuf = item.map_err(|error| { return format!("{} cannot be listed: {error}", pack_directory.display()); })?.path();
        if path.extension().and_then(OsStr::to_str) != Some("idx") {
            continue;
        }
        let index: File = File::open(&path).map_err(|error| { return format!("{} cannot be read: {error}", path.display()); })?;
        let Some(offset) = pack_offset(&index, name)? else {
            continue;
        };
        let pack: File = File::open(path.with_extension("pack")).map_err(pack_failure)?;
        // Borrow `&pack`: lends the value without copying or giving it away.
        return packed_entry_is_commit(&pack, offset);
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
        return loose_commit(&loose);
    }
    if packed_commit(&objects, &name)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(format!("{object} was not seen as a commit in the object files"));
}
