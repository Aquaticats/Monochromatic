//! What:
//! The reader of a version 2 pack index:
//! its fan-out table and the offset of one object's entry.
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
/// Imports the object-name length,
/// so a name is read and compared by the length a SHA-1 name has.
use crate::git_objects::OBJECT_NAME_BYTES;
/// Imports `File`,
/// which the positioned reads below take.
use std::fs::File;
/// Imports `FileExt`,
/// which adds positioned reads (`read_exact_at`) to an open file.
use std::os::unix::fs::FileExt;

/// The signature and version that start a version 2 pack index.
const PACK_INDEX_HEADER: [u8; 8] = [0xff, b't', b'O', b'c', 0, 0, 0, 2];

/// The number of fan-out entries of a pack index.
const FANOUT_ENTRIES: usize = 256;

/// What:
/// `fn index_failure(error: std::io::Error) -> String` words a failed read of a pack index.
/// Why:
/// Every read of the index reports the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const indexFailure = (error: Error) => `a pack index cannot be read: ${error}`;
/// ```
pub(crate) fn index_failure(error: std::io::Error) -> String {
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
pub(crate) fn pack_failure(error: std::io::Error) -> String {
    return format!("a pack cannot be read: {error}");
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
pub(crate) fn read_at(
    file: &File,
    offset: u64,
    length: usize,
) -> std::io::Result<Vec<u8>> {
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
pub(crate) fn fanout_entry(
    fanout: &[u8],
    position: usize,
) -> u64 {
    return big_endian(&fanout[position * 4..position * 4 + 4]);
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
pub(crate) fn big_endian(bytes: &[u8]) -> u64 {
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
pub(crate) fn pack_offset(
    index: &File,
    name: &[u8; OBJECT_NAME_BYTES],
) -> Result<Option<u64>, String> {
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
    let fanout: Vec<u8> =
        read_at(index, fanout_start, FANOUT_ENTRIES * 4).map_err(index_failure)?;
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
    let mut low: u64 = if first_byte == 0 {
        0
    } else {
        fanout_entry(&fanout, first_byte - 1)
    };
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
        let found: Vec<u8> = read_at(
            index,
            names_start + middle * OBJECT_NAME_BYTES as u64,
            OBJECT_NAME_BYTES,
        )
        .map_err(index_failure)?;
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
                let small: u64 = big_endian(
                    &read_at(index, offsets_start + middle * 4, 4).map_err(index_failure)?,
                );
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
                let large: Vec<u8> = read_at(index, large_start + (small & 0x7fff_ffff) * 8, 8)
                    .map_err(index_failure)?;
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
