//! What:
//!  Random identifiers in the layout of the incumbent's `randomUUID`:
//!  lock tokens,
//!       transaction IDs and stale-name suffixes.
//! Why:
//!  A lock token must be unguessable and unique per acquisition,
//!  and a transaction ID
//!      names a registry directory that the incumbent accepts only in the canonical layout
//!      (`isTransactionId` in `src/policy-engine/commit-transaction-registry.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { randomUUID } from 'node:crypto'; const token = randomUUID();
//! ```

/// What:
///  Why no identifier could be made:
///  the operating system's random source failed.
/// Why:
///   Without randomness no unguessable token exists,
///  so the caller must stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RandomSourceError extends Error {}
/// ```
#[derive(Debug)]
pub struct RandomSourceError(pub String);

/// Lowercase hexadecimal digits,
///  the alphabet `randomUUID` prints.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// What:
///  Format 16 bytes as a version-4 UUID in canonical lowercase layout.
///       `[u8; 16]` is a fixed array of sixteen bytes.
/// Why:
///   RFC 9562 version 4:
///  the version nibble is 4 and the variant bits are `10`,
///  exactly
///       what `randomUUID` produces;
///  the formatting is pure so tests can fix the input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatUuid(bytes: Uint8Array): string;
/// ```
pub fn format_uuid_v4(bytes: [u8; 16]) -> String {
    // `mut` allows setting the version and variant bits in place.
    let mut value: [u8; 16] = bytes;
    value[6] = (value[6] & 0x0f) | 0x40;
    value[8] = (value[8] & 0x3f) | 0x80;
    let mut text: String = String::with_capacity(36);
    for (index, byte) in value.iter().enumerate() {
        if index == 4 || index == 6 || index == 8 || index == 10 {
            text.push('-');
        }
        // Each byte is two digits: the high nibble, then the low nibble.
        text.push(char::from(HEX_DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(HEX_DIGITS[usize::from(byte & 0x0f)]));
    }
    return text;
}

/// What:
///  A fresh random version-4 UUID from the operating system's source.
///       `Result<String, RandomSourceError>` is "the identifier,
///  or why not".
/// Why:
///   Tokens and transaction IDs must not be predictable or collide between processes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function randomUuid(): string { return randomUUID(); }
/// ```
pub fn random_uuid() -> Result<String, RandomSourceError> {
    // `mut` allows the random source to fill the buffer.
    let mut bytes: [u8; 16] = [0; 16];
    // `map_err` turns the source's error into this module's error, keeping its text.
    getrandom::fill(&mut bytes).map_err(random_source_error)?;
    return Ok(format_uuid_v4(bytes));
}

/// What:
///  Describe a random-source failure.
/// Why:
///   A named function keeps the conversion out of a closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const randomSourceError = (error: Error) => new RandomSourceError(String(error));
/// ```
fn random_source_error(error: getrandom::Error) -> RandomSourceError {
    return RandomSourceError(format!(
        "the operating system's random source failed: {error}"
    ));
}

/// Formatting controls stay out of the release executable.
#[cfg(test)]
#[path = "random_id_tests.rs"]
mod tests;
