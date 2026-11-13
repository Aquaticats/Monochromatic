//! What:
//!  Where a URL begins,
//! where its authority ends,
//! and where the rest of it ends in running text,
//! read as bytes.
//! Why:
//!  The mask,
//! the host a destination names and the stream that holds a word back all need the same answer to
//!  "where does this URL end",
//! so the answer is written once.
//! A URL's authority ends at the first `/`,
//! `?`,
//! `#`,
//! `\`,
//! quote,
//! angle bracket,
//! backtick or white space,
//! and the rest of the URL ends at the first `\`,
//! quote,
//! angle bracket,
//! backtick,
//! white space or control character.
//! The separator `://` starts an authority only after a scheme character,
//! so a bare `://` in a path starts nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const authorityEnd = (text: string, from: number) => text.slice(from).search(/[/?#\\"'<>`\s]/)
//! ```

/// The separator between a URL's scheme and its authority.
pub(super) const SCHEME_SEPARATOR: &[u8] = b"://";

/// Whether a byte can end a scheme,
/// so that the separator after it starts a URL's authority.
pub(super) fn ends_scheme(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'+' || byte == b'-' || byte == b'.';
}

/// Whether a byte ends the authority of a URL.
fn ends_authority(byte: u8) -> bool {
    return byte.is_ascii_whitespace()
        || byte.is_ascii_control()
        || matches!(byte, b'/' | b'?' | b'#' | b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`');
}

/// What:
///  Whether a byte ends the path,
/// the query and the fragment of a URL.
/// Why:
///   A `/`,
/// `?` and `#` are part of them,
/// so only the characters that end a URL in running text are here.
pub(super) fn ends_url(byte: u8) -> bool {
    return byte.is_ascii_whitespace() || byte.is_ascii_control() || matches!(byte, b'\\' | b'"' | b'\'' | b'<' | b'>' | b'`');
}

/// Whether a byte is the `@` that ends the user information of a URL.
/// `&u8` borrows the byte,
/// as `rposition` hands it over.
pub(super) fn is_at_sign(byte: &u8) -> bool {
    return *byte == b'@';
}

/// Whether a byte is the `=` that ends a query or fragment key.
/// `&u8` borrows the byte,
/// as `position` hands it over.
pub(super) fn is_equals_sign(byte: &u8) -> bool {
    return *byte == b'=';
}

/// Whether a byte separates two parts of a path.
pub(super) fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// The index of the next `/` of a path,
/// or nothing when the path ends first.
pub(super) fn next_slash(path: &[u8]) -> Option<usize> {
    return path.iter().position(is_slash);
}

/// The index of the next `#` of a query,
/// or nothing when the query ends first.
pub(super) fn next_hash(query: &[u8]) -> Option<usize> {
    return query.iter().position(is_hash);
}

/// Whether a byte separates two parts of a query or a fragment.
pub(super) fn is_ampersand(byte: &u8) -> bool {
    return *byte == b'&';
}

/// Whether a byte begins the fragment of a URL.
pub(super) fn is_hash(byte: &u8) -> bool {
    return *byte == b'#';
}

/// Whether a byte begins the query of a URL,
/// or its fragment.
pub(super) fn begins_the_rest(byte: &u8) -> bool {
    return *byte == b'?' || *byte == b'#';
}

/// Whether an offset into an authority is a byte for the user information,
/// which needs at least one byte before it.
pub(super) fn is_not_the_first(offset: &usize) -> bool {
    return *offset > 0;
}

/// What:
///  The index of the next `://` at or after `from`.
/// Why:
///   Every URL of a text is found by this one scan,
/// so a text with many URLs is read a constant number of times.
pub(super) fn next_separator(text: &[u8], from: usize) -> Option<usize> {
    for (offset, window) in text.get(from..)?.windows(SCHEME_SEPARATOR.len()).enumerate() {
        if window == SCHEME_SEPARATOR {
            return Some(from + offset);
        }
    }
    return None;
}

/// The end of the authority that starts at `from`:
/// the first byte that ends one,
/// or the end of the text.
pub(super) fn authority_end_of(text: &[u8], from: usize) -> usize {
    for (offset, byte) in text[from..].iter().enumerate() {
        if ends_authority(*byte) {
            return from + offset;
        }
    }
    return text.len();
}

/// The end of the URL whose authority ended at `from`:
/// the first byte that ends a URL in running text,
/// or the end of the text.
pub(super) fn url_end_of(text: &[u8], from: usize) -> usize {
    for (offset, byte) in text[from..].iter().enumerate() {
        if ends_url(*byte) {
            return from + offset;
        }
    }
    return text.len();
}
