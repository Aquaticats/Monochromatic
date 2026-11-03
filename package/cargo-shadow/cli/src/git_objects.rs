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

/// The most bytes of a loose object read to learn its type (`commit 123456789\0` is far shorter).
const LOOSE_HEADER_BYTES: usize = 32;

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
/// `fn loose_type(path: &Path) -> Result<String, String>` reads the type word of a loose object from its compressed header.
/// Why:
/// A loose object starts with `<type> <size>\0` inside the zlib stream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function looseType(path: string): string  // throws on an unreadable object
/// ```
fn loose_type(path: &Path) -> Result<String, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
    // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { ... } catch (error) { throw new MyError(String(error)); }
    // ```
    // Typed binding: `file: File`.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    let file: File = File::open(path).map_err(|error| { return format!("{} cannot be read: {error}", path.display()); })?;
    // `let mut`: a binding that may change.
    // Typed binding: `header: Vec<u8>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut header: Vec<u8> = Vec::new();
    ZlibDecoder::new(file)
        // What:     `value as i64` converts a number to another numeric type (siblings: `i32`, `u32`, `u64`, `usize`, `f64`); it truncates silently if the value does not fit.
        // Why:      Time arithmetic and sizes come in different integer widths and must be made equal.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // Number(value)  // JS has one number type, so no cast is needed
        // ```
        .take(LOOSE_HEADER_BYTES as u64)
        // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
        // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
        // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
        // ```
        .read_to_end(&mut header)
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Macro `format!`: builds or prints text (or a list) from a template.
        .map_err(|error| { return format!("{} is not a readable object: {error}", path.display()); })?;
    // Closure `|byte|`: an anonymous function, like a TS arrow function.
    // Borrow `&[`: lends the value without copying or giving it away.
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Typed binding: `word: &[u8]`.
    let header_text: String = String::from_utf8_lossy(&header).to_string();
    let word: &str = header_text.split(' ').next().unwrap_or("");
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    return Ok(word.to_string());
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

/// What:
/// `fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String>` says whether some pack stores the
/// object as a whole commit.
/// Why:
/// The type is in the top bits of the first byte of the pack entry;
/// a tag,
/// tree,
/// blob or deltified entry is not a commit this reader can vouch for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function packedCommit(objects: string, name: Uint8Array): boolean  // throws when a pack cannot be read
/// ```
fn packed_commit(objects: &Path, name: &[u8; OBJECT_NAME_BYTES]) -> Result<bool, String> {
    // Typed binding: `pack_directory: PathBuf`.
    let pack_directory: PathBuf = objects.join("pack");
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&pack_directory`: lends the value without copying or giving it away.
    let listing = match fs::read_dir(&pack_directory) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(error) => return Err(format!("{} cannot be listed: {error}", pack_directory.display())),
    };
    for item in listing {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `path: PathBuf`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let path: PathBuf = item.map_err(|error| { return format!("{} cannot be listed: {error}", pack_directory.display()); })?.path();
        // `Some`: the present or the absent value of an `Option`.
        // Closure `|extension|`: an anonymous function, like a TS arrow function.
        // What:     Iterator and `Option` adapters such as `.map(f)`, `.filter(f)` and `.and_then(f)` transform or select items lazily with a closure.
        // Why:      They express one step of a pipeline without an index loop.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // list.map(f).filter(g)
        // ```
        if path.extension().and_then(OsStr::to_str) != Some("idx") {
            continue;
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&path`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `index: File`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let index: File = File::open(&path).map_err(|error| { return format!("{} cannot be read: {error}", path.display()); })?;
        // `Some`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // Borrow `&index`: lends the value without copying or giving it away.
        let Some(offset) = pack_offset(&index, name)? else {
            continue;
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `pack: File`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let pack: File = File::open(path.with_extension("pack")).map_err(pack_failure)?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|error|`: an anonymous function, like a TS arrow function.
        // Borrow `&pack`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `first: Vec<u8>`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let first: Vec<u8> = read_at(&pack, offset, 1).map_err(pack_failure)?;
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok((first[0] >> 4) & 0x7 == PACK_TYPE_COMMIT);
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
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
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&loose`: lends the value without copying or giving it away.
        // Typed binding: `kind: String`.
        let kind: String = loose_type(&loose)?;
        if kind == "commit" {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(());
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{object} is a {kind}, not a commit"));
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&objects`: lends the value without copying or giving it away.
    if packed_commit(&objects, &name)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(());
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(format!("{object} was not seen as a commit in the object files"));
}
