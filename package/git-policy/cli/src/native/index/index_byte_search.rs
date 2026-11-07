//! What:
//!  Byte searches over a slice that the index readers share:
//! where a byte first appears,
//! how long a run of one byte is,
//!  and whether any byte is not zero.
//! Why:
//!  The index formats are NUL-terminated names,
//! `/`-separated paths and zero-filled hashes.
//! A closure per search was
//!  reported by `rust/avoidable-closure`;
//! one named loop per question reads the same everywhere and breaks the same way.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const nul = bytes.indexOf(0);          // firstIndexOfByte(bytes, 0)
//! const hasData = bytes.some((b) => b !== 0); // hasNonzeroByte(bytes)
//! ```

/// Whether `byte` is the path separator `/`;
/// the shape `split` and `all` want.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSlash = (byte: number): boolean => byte === 0x2f;
/// ```
pub fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// The position of the first byte of `bytes` equal to `wanted`,
/// or `None` when no byte is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstIndexOfByte(bytes: Uint8Array, wanted: number): number | undefined; // bytes.indexOf(wanted)
/// ```
pub fn first_index_of_byte(bytes: &[u8], wanted: u8) -> Option<usize> {
    // `enumerate` pairs each byte with its position, as `bytes.entries()` does in TS.
    for (position, byte) in bytes.iter().enumerate() {
        if *byte == wanted {
            return Some(position);
        }
    }
    return None;
}

/// How many bytes at the start of `bytes` equal `wanted` in a row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leadingRunLength(bytes: Uint8Array, wanted: number): number;
/// ```
pub fn leading_run_length(bytes: &[u8], wanted: u8) -> usize {
    let mut length: usize = 0;
    while length < bytes.len() && bytes[length] == wanted {
        length += 1;
    }
    return length;
}

/// How many bytes at the start of `bytes` are ASCII digits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leadingDigitCount(bytes: Uint8Array): number;
/// ```
pub fn leading_digit_count(bytes: &[u8]) -> usize {
    let mut length: usize = 0;
    while length < bytes.len() && bytes[length].is_ascii_digit() {
        length += 1;
    }
    return length;
}

/// Whether any byte of `bytes` is not zero;
/// an empty slice has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hasNonzeroByte(bytes: Uint8Array): boolean; // bytes.some((b) => b !== 0)
/// ```
pub fn has_nonzero_byte(bytes: &[u8]) -> bool {
    for byte in bytes {
        if *byte != 0 {
            return true;
        }
    }
    return false;
}

/// How many bytes `left` and `right` share at their start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commonPrefixLength(left: Uint8Array, right: Uint8Array): number;
/// ```
pub fn common_prefix_length(left: &[u8], right: &[u8]) -> usize {
    let limit: usize = left.len().min(right.len());
    let mut length: usize = 0;
    while length < limit && left[length] == right[length] {
        length += 1;
    }
    return length;
}
