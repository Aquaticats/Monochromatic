//! What:
//!  Three small checks on bytes and names that the object-name abbreviation uses:
//! a NUL separator,
//! a hexadecimal digit,
//! and
//!  how long two names agree at their start.
//! Why:
//!  They read only bytes,
//! so they sit in a file of their own that does not use `object_abbreviation.rs`,
//! which keeps that file
//!  under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const sharedPrefixLength = (left: string, right: string): number => ...;
//! ```


/// Whether `byte` is zero,
/// the separator between pack names in a multi-pack-index.
pub(crate) fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether `byte` is an ASCII hexadecimal digit.
pub(crate) fn is_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// How many characters `left` and `right` share at their start.
pub(crate) fn shared_prefix_length(left: &str, right: &str) -> usize {
    let mut count: usize = 0;
    for (one, other) in left.bytes().zip(right.bytes()) {
        if one != other {
            break;
        }
        count += 1;
    }
    return count;
}
