//! What:
//!  What a Git delta means:
//!  the base size it states,
//!  the result size,
//!  the bytes it copies from its base,
//!  the bytes it inserts,
//!  and where a pack finds a base the delta names by object.
//! Why:
//!  A delta is one format with no system call in it,
//!  and it is refused whole when any number disagrees,
//!  because a corrupt delta must never read past a buffer.
//!  It stands apart from the pack that finds the bytes (`repo_read_pack.rs`)
//!  and from the file that rebuilds an entry (`the_entries_of_one_pack.rs`),
//!  so both name it without naming each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function applyDelta(base: Uint8Array, delta: Uint8Array): Uint8Array | undefined;
//! interface BaseByName { baseNamed(name: Uint8Array): RawObject | undefined }
//! ```

#![cfg(unix)]

use crate::repo_read_object::{
    OBJECT_SIZE_LIMIT,
    RawObject,
};

/// One delta size varint at `cursor`:
/// seven bits per byte,
/// low first.
fn delta_size(
    delta: &[u8],
    cursor: &mut usize,
) -> Option<usize> {
    let mut size: usize = 0;
    let mut shift: u32 = 0;
    loop {
        let byte: u8 = *delta.get(*cursor)?;
        *cursor += 1;
        size |= usize::from(byte & 0x7f).checked_shl(shift)?;
        if byte & 0x80 == 0 {
            return Some(size);
        }
        shift += 7;
        if shift > 56 {
            return None;
        }
    }
}

/// What:
///  Apply a Git delta to its base.
/// Why:
///   A delta states the base size and the result size and then copies from the base or inserts bytes;
/// every number is
///  checked,
/// because a corrupt delta must be refused and not read past a buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyDelta(base: Uint8Array, delta: Uint8Array): Uint8Array | undefined;
/// ```
pub fn apply_delta(
    base: &[u8],
    delta: &[u8],
) -> Option<Vec<u8>> {
    let mut cursor: usize = 0;
    if delta_size(delta, &mut cursor)? != base.len() {
        return None;
    }
    let result_size: usize = delta_size(delta, &mut cursor)?;
    if result_size > OBJECT_SIZE_LIMIT {
        return None;
    }
    let mut result: Vec<u8> = Vec::with_capacity(result_size);
    while cursor < delta.len() {
        let op: u8 = delta[cursor];
        cursor += 1;
        if op & 0x80 != 0 {
            let mut offset: usize = 0;
            let mut length: usize = 0;
            for bit in 0..4 {
                if op & (1 << bit) != 0 {
                    offset |= usize::from(*delta.get(cursor)?) << (8 * bit);
                    cursor += 1;
                }
            }
            for bit in 0..3 {
                if op & (0x10 << bit) != 0 {
                    length |= usize::from(*delta.get(cursor)?) << (8 * bit);
                    cursor += 1;
                }
            }
            if length == 0 {
                length = 0x10000;
            }
            result.extend_from_slice(base.get(offset..offset.checked_add(length)?)?);
        } else if op != 0 {
            let end: usize = cursor.checked_add(usize::from(op))?;
            result.extend_from_slice(delta.get(cursor..end)?);
            cursor = end;
        } else {
            return None;
        }
        if result.len() > result_size {
            return None;
        }
    }
    if result.len() != result_size {
        return None;
    }
    return Some(result);
}

/// What:
///  Where a pack finds the base of a `REF_DELTA`,
/// which is named by object and may live outside the pack.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The database that owns the pack also owns the other packs and the loose objects,
/// and it follows a delta chain one step
///  deeper each time;
/// a type that holds the database and the depth so far implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface BaseByName { baseNamed(name: Uint8Array): RawObject | undefined }
/// ```
pub trait BaseByName {
    /// The object named `name`,
    /// or nothing when it cannot be read.
    fn base_named(
        &self,
        name: &[u8],
    ) -> Option<RawObject>;
}
