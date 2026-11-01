//! What:
//!  The reversible, flat, case-insensitive-safe file name of one branch's push coordination files.
//! Why:
//!  The per-branch push lock and `last-pushed` record live under `<git-common-dir>/cli-git/push/`,
//!  and the incumbent (`src/auto-push-branch-key.ts`) and this wrapper must name them identically,
//!  because both wrappers contend on the same lock while both are installed.
//!  Uppercase letters are escaped too, so refs that differ only in case never share a file
//!  on a case-insensitive filesystem.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! encodeBranchKey('refs/heads/Main'); // 'refs%2Fheads%2F%4Dain'
//! ```

/// What:
///  A push coordination key that `encode_branch_key` never produces.
/// Why:
///   Decoding a key that the incumbent did not write must stop the caller,
///  never guess a ref name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BranchKeyError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BranchKeyError {
    /// Diagnostic naming the key and the position of the fault.
    pub message: String,
}

/// Displays the diagnostic only.
impl std::fmt::Display for BranchKeyError {
    /// Writes the diagnostic text.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(&self.message);
    }
}

/// Marks the key failure as a standard error.
impl std::error::Error for BranchKeyError {}

/// The escape introducer `%`.
const ESCAPE: u8 = b'%';

/// Whether a byte is kept verbatim: lowercase ASCII letters, digits, `-`, `_` and `.`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const VERBATIM_CHARACTERS = new Set('abcdefghijklmnopqrstuvwxyz0123456789-_.');
/// ```
fn is_verbatim(byte: u8) -> bool {
    return byte.is_ascii_lowercase()
        || byte.is_ascii_digit()
        || byte == b'-'
        || byte == b'_'
        || byte == b'.';
}

/// What:
///  Encode a full ref name as one flat file name component.
///       `&str` is borrowed UTF-8 text.
/// Why:
///   Each UTF-8 byte outside the verbatim set becomes `%XX` with uppercase hexadecimal digits,
///  so every ref maps to one reversible name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encodeBranchKey(refName: string): string;
/// ```
pub fn encode_branch_key(ref_name: &str) -> String {
    let mut key = String::with_capacity(ref_name.len());
    for &byte in ref_name.as_bytes() {
        if is_verbatim(byte) {
            key.push(char::from(byte));
        } else {
            key.push_str(&format!("%{byte:02X}"));
        }
    }
    return key;
}

/// What:
///  Read one `%XX` escape starting at `index`, the position of the introducer.
/// Why:
///   Only the uppercase form `encode_branch_key` writes is accepted,
///  and an escape of a verbatim byte is refused,
///  so every ref name has exactly one accepted key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeEscape({ key, index }): number; // throws BranchKeyError
/// ```
fn decode_escape(key: &[u8], index: usize) -> Result<u8, BranchKeyError> {
    let digits = key.get(index + 1..index + 3).unwrap_or(&[]);
    if digits.len() != 2
        || !digits.iter().all(|digit| {
            return is_upper_hex(*digit);
        })
    {
        return Err(BranchKeyError {
            message: format!(
                "Push coordination key has a malformed escape at {index}: {}",
                String::from_utf8_lossy(key)
            ),
        });
    }
    let high = hex_value(digits[0]);
    let low = hex_value(digits[1]);
    let byte = high * 16 + low;
    if is_verbatim(byte) {
        return Err(BranchKeyError {
            message: format!(
                "Push coordination key escapes a verbatim character at {index}: {}",
                String::from_utf8_lossy(key)
            ),
        });
    }
    return Ok(byte);
}

/// Whether a byte is an uppercase hexadecimal digit, the only digits `encode_branch_key` writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /[0-9A-F]/.test(digit)
/// ```
fn is_upper_hex(digit: u8) -> bool {
    return digit.is_ascii_digit() || (b'A'..=b'F').contains(&digit);
}

/// The value of one uppercase hexadecimal digit; the caller has checked the digit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number.parseInt(digit, 16)
/// ```
fn hex_value(digit: u8) -> u8 {
    if digit.is_ascii_digit() {
        return digit - b'0';
    }
    return digit - b'A' + 10;
}

/// What:
///  Reverse `encode_branch_key` in one left-to-right pass.
///       `Result<A, B>` is either the decoded ref name or a `BranchKeyError`.
/// Why:
///   A key with an unescaped non-verbatim character, a malformed escape,
///  or bytes that are not UTF-8 names no branch and must stop the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodeBranchKey(key: string): string; // throws BranchKeyError
/// ```
pub fn decode_branch_key(key: &str) -> Result<String, BranchKeyError> {
    let bytes = key.as_bytes();
    let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == ESCAPE {
            decoded.push(decode_escape(bytes, index)?);
            index += 3;
        } else if is_verbatim(byte) {
            decoded.push(byte);
            index += 1;
        } else {
            return Err(BranchKeyError {
                message: format!(
                    "Push coordination key holds an unescaped character at {index}: {key}"
                ),
            });
        }
    }
    return match String::from_utf8(decoded) {
        Ok(text) => Ok(text),
        Err(error) => Err(BranchKeyError {
            message: format!("Push coordination key is not UTF-8 ({error}): {key}"),
        }),
    };
}

/// Encoding and decoding behave as the incumbent's branch key codec.
#[cfg(test)]
#[path = "auto_push_branch_key_tests.rs"]
mod tests;
