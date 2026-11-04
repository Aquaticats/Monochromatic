//! What:
//!  The small integrity helpers of the commit protocol's durable records: a CRC-32 for ledger lines, the lowercase
//!  hexadecimal spelling of bytes, a SHA-256 hex digest, and the check for an identity.
//! Why:
//!  The immutable request, the spool, the identity binding and the permanent ledger
//!  (`doc/planning/cli-git-commits-never-fail.md`, step 1) all carry a checksum or a hash and must spell them one way.
//!  `subvolume_sha::sha256` already exists, so no hash is written twice and no dependency is added.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // import { createHash } from 'node:crypto';
//! // const sha256Hex = (bytes: Uint8Array) => createHash('sha256').update(bytes).digest('hex');
//! ```

// What:     `use crate::subvolume_sha::sha256;` imports the repository's SHA-256 from its sibling module
//           (`super` is the parent module, here the library root).
// Why:      The request hash and the spool hashes are SHA-256 digests; reusing the function adds no dependency.
//
// In TS you'd write (pseudocode):
// ```ts
// import { sha256 } from './subvolume_sha';
// ```
use crate::subvolume_sha::sha256;

/// What:
///  The CRC-32 (IEEE 802.3, reflected, polynomial `0xEDB88320`) of `bytes`.
///  `&[u8]` borrows a byte list read-only; `u32` is an unsigned 32-bit integer (siblings: `u8`, `u64`).
/// Why:
///  A ledger line needs a per-line check that detects a torn or damaged line; 32 bits detects every single torn
///  append (a CRC is not a security hash; the request hash is SHA-256).
///  `u32` and not `usize` because the width is part of the format.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function crc32(bytes: Uint8Array): number;
/// ```
pub fn crc32(bytes: &[u8]) -> u32 {
    // The running value starts as all ones and is complemented at the end, as the IEEE definition says.
    let mut value: u32 = 0xFFFF_FFFF;
    for byte in bytes {
        // `*byte` reads the byte through the borrow; `as u32` widens it (TS numbers have no width, so no cast).
        value ^= *byte as u32;
        for _ in 0..8 {
            if value & 1 == 1 {
                value = (value >> 1) ^ 0xEDB8_8320;
            } else {
                value >>= 1;
            }
        }
    }
    // What:     `!value` is bitwise NOT on an integer (TS `~value`), the tail expression returned.
    // Why:      The final complement of the IEEE definition.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return ~value >>> 0;
    // ```
    return !value;
}

/// What:
///  The lowercase hexadecimal spelling of `bytes`, two characters per byte.
///  `String` is an owned text (sibling `&str`, a borrowed view).
/// Why:
///  Every identity, hash and encoded field of the records is hexadecimal, so a field can never contain a space, a
///  newline or a quote, whatever the input holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const hex = (bytes: Uint8Array) => Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
/// ```
pub fn hex(bytes: &[u8]) -> String {
    let digits: &[u8; 16] = b"0123456789abcdef";
    let mut text: String = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(digits[(*byte >> 4) as usize] as char);
        text.push(digits[(*byte & 15) as usize] as char);
    }
    return text;
}

/// What:
///  The bytes a lowercase hexadecimal `text` spells, or `None` when it has an odd length or any other character.
///  `Option<Vec<u8>>` is either `Some(bytes)` or `None` (TS `Uint8Array | undefined`).
/// Why:
///  Decoding is strict (lowercase only) so a record has exactly one spelling and its hash is stable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unhex(text: string): Uint8Array | undefined;
/// ```
pub fn unhex(text: &str) -> Option<Vec<u8>> {
    let raw: &[u8] = text.as_bytes();
    if raw.len() % 2 != 0 {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(raw.len() / 2);
    for pair in raw.chunks(2) {
        // `digit_value(..)?` returns `None` from this function at once when the character is not a digit.
        let high: u8 = digit_value(pair[0])?;
        let low: u8 = digit_value(pair[1])?;
        bytes.push(high * 16 + low);
    }
    // What:     `Some(bytes)` wraps the decoded list as the present case of `Option`.
    // Why:      Every pair was a digit.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return bytes;
    // ```
    return Some(bytes);
}

/// The value of one lowercase hexadecimal digit, or `None`.
fn digit_value(byte: u8) -> Option<u8> {
    if byte.is_ascii_digit() {
        return Some(byte - b'0');
    }
    if (b'a'..=b'f').contains(&byte) {
        return Some(byte - b'a' + 10);
    }
    return None;
}

/// What:
///  The SHA-256 of `bytes` as 64 lowercase hexadecimal characters.
/// Why:
///  The hash of an immutable request, of a spooled file and of a binding is spelled this one way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sha256Hex(bytes: Uint8Array): string;
/// ```
pub fn sha256_hex(bytes: &[u8]) -> String {
    return hex(&sha256(bytes));
}

/// What:
///  Whether `text` is a request identity: exactly 32 lowercase hexadecimal characters (128 random bits).
/// Why:
///  The identity names files (`ids/<xx>/<id>`, `inbox/<id>`), so it must never hold a separator, a dot or a space.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isIdentity = (text: string) => /^[0-9a-f]{32}$/.test(text);
/// ```
pub fn is_identity(text: &str) -> bool {
    return text.len() == 32 && unhex(text).is_some();
}

/// Whether `text` is a Git object name in either object format: 40 or 64 lowercase hexadecimal characters.
pub fn is_object_name(text: &str) -> bool {
    return (text.len() == 40 || text.len() == 64) && unhex(text).is_some();
}
