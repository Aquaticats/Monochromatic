//! What:
//!  The decode of one type entry's kind-specific fields,
//! and the bytes one kind's
//!  entry holds after its head.
//! Why:
//!  The walk in `btf_types.rs` needs the length of every kind before any parser
//! sees
//!  a type,
//! and the comparison needs the members,
//! the array's element and count and
//!  the aliases' targets;
//! keeping both in one place means a kind this file cannot
//!  measure is also a kind nothing decodes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function entryBytes(kind: number, vlen: number): number | undefined;
//! ```

/// The kind numbers the length table names,
/// and the entry they decode to.
use super::btf_entry::{
    ARRAY_BYTES, DECL_TAG_BYTES, ENTRY_HEAD_BYTES, Entry, INT_BYTES, KIND_ARRAY, KIND_CONST,
    KIND_DATASEC, KIND_DECL_TAG, KIND_ENUM, KIND_ENUM64, KIND_FLAG, KIND_FLOAT, KIND_FUNC,
    KIND_FUNC_PROTO, KIND_FWD, KIND_INT, KIND_MASK, KIND_PTR, KIND_RESTRICT, KIND_SHIFT,
    KIND_STRUCT, KIND_TYPEDEF, KIND_TYPE_TAG, KIND_UNION, KIND_VAR, KIND_VOLATILE, MEMBER_BYTES,
    VAR_BYTES, VLEN_MASK,
};

/// The bytes one type entry holds after its head,
/// or nothing for a kind this reader cannot walk.
pub(super) fn entry_extra_bytes(kind: u8, vlen: u32) -> Option<usize> {
    let count: usize = vlen as usize;
    return match kind {
        KIND_INT => Some(INT_BYTES),
        KIND_PTR | KIND_FWD | KIND_TYPEDEF | KIND_VOLATILE | KIND_CONST | KIND_RESTRICT
        | KIND_FUNC | KIND_FLOAT | KIND_TYPE_TAG => Some(0),
        KIND_ARRAY => Some(ARRAY_BYTES),
        KIND_STRUCT | KIND_UNION => count.checked_mul(MEMBER_BYTES),
        KIND_ENUM => count.checked_mul(8),
        KIND_FUNC_PROTO => count.checked_mul(8),
        KIND_VAR => Some(VAR_BYTES),
        KIND_DATASEC => count.checked_mul(MEMBER_BYTES),
        KIND_DECL_TAG => Some(DECL_TAG_BYTES),
        KIND_ENUM64 => count.checked_mul(12),
        _ => None,
    };
}

/// Decode one entry whose length the walk already checked,
/// with the file's own name reader.
pub(super) fn decode<'a>(
    head: &'a [u8],
    name_at: impl Fn(u32) -> Option<&'a str>,
) -> Entry<'a> {
    let name_offset: u32 = field(head, 0);
    let info: u32 = field(head, 4);
    let size_or_type: u32 = field(head, 8);
    let kind: u8 = ((info >> KIND_SHIFT) & KIND_MASK) as u8;
    let vlen: usize = (info & VLEN_MASK) as usize;
    let name: Option<&'a str> = name_at(name_offset);
    let bitfield: bool = info & KIND_FLAG != 0;
    return match kind {
        KIND_INT => Entry::Integer {
            name,
            bits: size_or_type,
        },
        KIND_PTR => Entry::Pointer { name },
        KIND_ARRAY => Entry::Array {
            name,
            element: field(head, ENTRY_HEAD_BYTES),
            count: field(head, ENTRY_HEAD_BYTES + 8),
        },
        KIND_STRUCT | KIND_UNION => Entry::Record {
            name,
            kind,
            bytes: size_or_type,
            members: slice(head, ENTRY_HEAD_BYTES, vlen.saturating_mul(MEMBER_BYTES)),
            bitfield,
        },
        KIND_ENUM | KIND_ENUM64 => Entry::Enumeration {
            name,
            bytes: size_or_type,
            kind,
            values: slice(head, ENTRY_HEAD_BYTES, vlen.saturating_mul(enum_value_bytes(kind))),
        },
        KIND_FLOAT => Entry::Float {
            name,
            bytes: size_or_type,
        },
        KIND_TYPEDEF | KIND_VOLATILE | KIND_CONST | KIND_RESTRICT | KIND_TYPE_TAG => {
            Entry::Alias {
                name,
                kind,
                target: size_or_type,
            }
        }
        _ => Entry::Other { kind },
    };
}

/// The bytes one value of an enumeration kind holds.
pub(super) fn enum_value_bytes(kind: u8) -> usize {
    if kind == KIND_ENUM64 {
        return 12;
    }
    return 8;
}

/// One little-endian 32-bit field at `at`.
fn field(head: &[u8], at: usize) -> u32 {
    let mut bytes: [u8; 4] = [0; 4];
    let Some(source): Option<&[u8]> = head.get(at..at + 4) else {
        return 0;
    };
    bytes.copy_from_slice(source);
    return u32::from_le_bytes(bytes);
}

/// One slice of an entry's own bytes,
/// empty where the walk's length and this read disagree.
fn slice(head: &[u8], start: usize, length: usize) -> &[u8] {
    let end: usize = start.saturating_add(length);
    return head.get(start..end).unwrap_or_default();
}
