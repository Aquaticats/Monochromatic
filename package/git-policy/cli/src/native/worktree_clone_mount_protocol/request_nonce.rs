//! What:
//!  The 32-lowercase-hex-digit nonce both the intent and the nudge carry:
//!  its parse,
//!  its fresh value from the operating system's random source,
//!  and the refusal that names what was typed.
//! Why:
//!  The design fixes one nonce format for the durable intent and for the
//!  clone-local sweep nonce,
//!  and the nonce is what a receipt is matched against
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering").
//!  It is data and never a file name,
//!  so a value a caller composed cannot reach a path.
//!  Uppercase digits are refused rather than folded:
//!  a nonce that compared equal under two spellings would match a receipt it
//!  was not sent with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const nonce = RequestNonce.fresh(); nonce.asStr(); // 32 lowercase hex digits
//! ```

use crate::foundation::hexadecimal_of_bytes::write_hexadecimal;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;

/// What:
///  Why a value is not a request nonce.
/// Why:
///   The cause names the length or the one byte that is wrong,
///  so a person can see which character to change rather than only that the
///  value was rejected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NonceRefusal = { kind: 'length'; found: number } | { kind: 'byte'; place: number; byte: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NonceRefusal {
    /// The value is not exactly 32 bytes long.
    Length {
        /// How many bytes it holds.
        found: usize,
    },
    /// One byte is not a lowercase ASCII hexadecimal digit.
    NotLowercaseHex {
        /// Its one-based place in the value.
        place: usize,
        /// The byte as a person reads it,
        /// with a non-printable one written as its escape.
        shown: String,
    },
    /// The operating system's random source failed,
    /// so no fresh nonce exists.
    RandomSource {
        /// What the source answered.
        reason: String,
    },
}

impl NonceRefusal {
    /// The refusal in words,
    /// naming the value,
    /// the place and the way forward.
    pub fn message(&self) -> String {
        return match self {
            NonceRefusal::Length { found } => format!(
                "the nonce is {found} bytes long, and every nonce this route carries is exactly {NONCE_HEX_DIGITS} lowercase ASCII hexadecimal digits. Nothing was read as an intent and nothing was done on its behalf. Ways forward: write the nonce as {NONCE_HEX_DIGITS} digits from 0 to 9 and a to f, with no dashes, spaces or uppercase letters."
            ),
            NonceRefusal::NotLowercaseHex { place, shown } => format!(
                "byte {place} of the nonce is `{shown}`, and every nonce byte is a lowercase ASCII hexadecimal digit from 0 to 9 or a to f. Nothing was read as an intent and nothing was done on its behalf. Ways forward: write the byte as a lowercase hexadecimal digit; an uppercase letter is a different value here and is not folded."
            ),
            NonceRefusal::RandomSource { reason } => format!(
                "no fresh nonce could be made because the operating system's random source failed ({reason}). Nothing was requested and nothing was done. Ways forward: run the command again; if the source keeps failing, the machine has nothing this route can use to tell one request from another, and an administrator has to repair it."
            ),
        };
    }
}

/// What:
///  One validated 32-digit nonce.
/// Why:
///   A value of this type can only have come through the parse,
///  so a receipt comparison and an intent parse never compare an unchecked
///  string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RequestNonce { private constructor(readonly digits: string) {} }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestNonce {
    /// The validated digits,
    /// exactly 32 lowercase hexadecimal characters.
    digits: String,
}

impl RequestNonce {
    /// Read a value as a nonce,
    /// checking its length and every byte.
    ///
    /// # Errors
    /// The value is not 32 bytes,
    /// or one byte is not a lowercase hexadecimal digit.
    pub fn parse(text: &str) -> Result<Self, NonceRefusal> {
        let bytes: &[u8] = text.as_bytes();
        if bytes.len() != NONCE_HEX_DIGITS {
            return Err(NonceRefusal::Length { found: bytes.len() });
        }
        for (index, byte) in bytes.iter().enumerate() {
            if !is_lowercase_hexadecimal(*byte) {
                return Err(NonceRefusal::NotLowercaseHex { place: index + 1, shown: shown_byte(*byte) });
            }
        }
        return Ok(RequestNonce { digits: String::from(text) });
    }

    /// A fresh nonce from the operating system's random source.
    ///
    /// # Errors
    /// The random source failed,
    /// or a nonce could not be made.
    pub fn fresh() -> Result<Self, NonceRefusal> {
        let mut bytes: [u8; NONCE_HEX_DIGITS / 2] = [0; NONCE_HEX_DIGITS / 2];
        if let Err(error) = getrandom::fill(&mut bytes) {
            return Err(NonceRefusal::RandomSource { reason: error.to_string() });
        }
        let mut digits: String = String::with_capacity(NONCE_HEX_DIGITS);
        for byte in bytes {
            write_hexadecimal(&mut digits, byte);
        }
        return Ok(RequestNonce { digits });
    }

    /// The digits as a caller writes them,
    /// which is also how a receipt carries them.
    pub fn as_str(&self) -> &str {
        return self.digits.as_str();
    }
}

/// What:
///  Whether one byte is a lowercase ASCII hexadecimal digit.
/// Why:
///   The parse and any later reader of a stored nonce ask the same question,
///  so the alphabet is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isHex = (byte) => /[0-9a-f]/.test(String.fromCharCode(byte));
/// ```
fn is_lowercase_hexadecimal(byte: u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte);
}

/// What:
///  One byte as a person can read it in a refusal.
/// Why:
///   A newline or a control byte in a message would break the line the reader
///  prints,
///  so anything outside printable ASCII is written as its hexadecimal escape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shown = (byte) => byte >= 0x21 && byte <= 0x7e ? String.fromCharCode(byte) : `\\x${byte.toString(16).padStart(2, '0')}`;
/// ```
fn shown_byte(byte: u8) -> String {
    if byte.is_ascii_graphic() {
        return String::from_utf8_lossy(&[byte]).into_owned();
    }
    return format!("\\x{byte:02x}");
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "request_nonce_tests.rs"]
mod tests;
