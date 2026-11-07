//! What:
//!  Hide the credentials inside URLs in the text the wrapper relays from `git-lfs`.
//! Why:
//!  `git-lfs` names the LFS server in its errors,
//! and an `lfs.url` written as `https://user:token@host/...` carries the
//!  token into every one of them.
//! The wrapper relays that text to the caller's terminal and,
//! for the auto-push,
//! records
//!  it in files,
//! so the userinfo of every URL is replaced at the one place where the text leaves the wrapper.
//! The
//!  authority of a URL ends at the first `/`,
//! `?`,
//! `#`,
//! `\`,
//! quote,
//! angle bracket,
//! backtick or white space,
//! and its
//!  userinfo ends at the last `@` before that,
//! so an `@` in a path,
//! a query or a bracketed IPv6 host is left alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const masked = text.replace(/([a-z][a-z0-9+.-]*:\/\/)[^\/?#\\\s"'<>`]*@/gi, '$1<credentials>@');
//! ```

/// What replaces the userinfo of a URL.
pub const CREDENTIALS_MASK: &str = "<credentials>";

/// The separator between a URL's scheme and its authority.
const SCHEME_SEPARATOR: &[u8] = b"://";

/// Whether a byte can end a scheme,
/// so that the separator after it starts a URL's authority.
fn ends_scheme(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'-' || byte == b'.';
}

/// Whether a byte ends the authority of a URL.
fn ends_authority(byte: u8) -> bool {
    return byte.is_ascii_whitespace()
        || byte.is_ascii_control()
        || matches!(byte, b'/' | b'?' | b'#' | b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`');
}

/// Whether a byte is the `@` that ends the userinfo of a URL.
/// `&u8` borrows the byte,
/// as `rposition` hands it over.
fn is_at_sign(byte: &u8) -> bool {
    return *byte == b'@';
}

/// The index of the next scheme separator at or after `from`.
fn next_separator(text: &[u8], from: usize) -> Option<usize> {
    for (offset, window) in text.get(from..)?.windows(SCHEME_SEPARATOR.len()).enumerate() {
        if window == SCHEME_SEPARATOR {
            return Some(from + offset);
        }
    }
    return None;
}

/// What:
///  The text with the userinfo of every `scheme://userinfo@host` URL replaced by [`CREDENTIALS_MASK`].
/// Why:
///   One pass over the bytes:
/// each search starts where the previous authority ended,
/// so the work is linear however many
///  URLs a line holds.
/// Bytes outside the userinfo are copied unchanged,
/// whatever their encoding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function maskUserinfo(text: Uint8Array): Uint8Array;
/// ```
pub fn mask_userinfo(text: &[u8]) -> Vec<u8> {
    let mut masked: Vec<u8> = Vec::with_capacity(text.len());
    let mut copied: usize = 0;
    let mut search_from: usize = 0;
    while let Some(separator) = next_separator(text, search_from) {
        let authority_start: usize = separator + SCHEME_SEPARATOR.len();
        let mut authority_end: usize = text.len();
        for (offset, byte) in text[authority_start..].iter().enumerate() {
            if ends_authority(*byte) {
                authority_end = authority_start + offset;
                break;
            }
        }
        search_from = authority_end;
        let after_scheme: bool = separator > 0 && ends_scheme(text[separator - 1]);
        if !after_scheme {
            continue;
        }
        let last_at: Option<usize> = text[authority_start..authority_end].iter().rposition(is_at_sign);
        // Userinfo needs at least one byte before the last `@`.
        let userinfo_end: Option<usize> = match last_at {
            Some(offset) if offset > 0 => Some(offset),
            _ => None,
        };
        if let Some(offset) = userinfo_end {
            masked.extend_from_slice(&text[copied..authority_start]);
            masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
            copied = authority_start + offset;
        }
    }
    masked.extend_from_slice(&text[copied..]);
    return masked;
}

/// The same for text:
/// bytes that are not UTF-8 stay as they were and the result is read back lossily.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function maskUserinfoText(text: string): string;
/// ```
pub fn mask_userinfo_text(text: &str) -> String {
    return String::from_utf8_lossy(mask_userinfo(text.as_bytes()).as_slice()).into_owned();
}

/// Masking is controlled in the test module.
#[cfg(test)]
#[path = "lfs_output_mask_tests.rs"]
mod tests;
