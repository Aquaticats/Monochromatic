//! What:
//!  Whether a blob is a Git LFS pointer,
//! decided the way `git-lfs` 3.8.1 decides it when it scans a push.
//! Why:
//!  The upload step finds the LFS objects the pushed commits introduce without starting `git-lfs`.
//! A blob counts when
//!  the scanner would hand it to the transfer queue:
//! smaller than the cutoff (`lfs/scanner.go`,
//! `BlobSizeCutoff`),
//! a
//!  header that names `git-media`,
//! `hawser` or `git-lfs` (`lfs/pointer.go`,
//! `matcherRE`),
//! lines in the order `version`,
//!  `oid`,
//! `size` with a known version,
//! an `sha256:` object name of 64 lower-case hexadecimal digits and a non-negative
//!  decimal size;
//! a pointer of size zero is skipped by `prepareUpload` (`commands/uploader.go`).
//! The path of the blob and
//!  `.gitattributes` play no part.
//! A spelling the port is not sure about (extension lines,
//! non-ASCII edges) is a
//!  decline,
//! so the caller falls back to `git-lfs` itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const verdict = decodePointer(blob); // { kind: 'not-a-pointer' } | { kind: 'pointer', oid, size } | { kind: 'decline' }
//! ```

/// The blob size from which a blob is never read as a pointer.
pub const BLOB_SIZE_CUTOFF: usize = 1024;

/// The versions `git-lfs` accepts (`v1Aliases`).
const VERSIONS: [&[u8]; 3] = [b"http://git-media.io/v/2", b"https://hawser.github.com/spec/v1", b"https://git-lfs.github.com/spec/v1"];

/// The keys of a pointer in their order.
const KEYS: [&[u8]; 3] = [b"version", b"oid", b"size"];

/// The words one of which a pointer's text must hold (`matcherRE`).
const MATCHERS: [&[u8]; 3] = [b"git-media", b"hawser", b"git-lfs"];

/// What a blob is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// `git-lfs` would not upload anything for this blob.
    NotUploadable,
    /// `git-lfs` would upload the object with this name (lower-case hexadecimal SHA-256).
    Pointer(String),
    /// The port cannot be sure;
    /// ask `git-lfs`.
    Decline,
}

/// Whether one byte is the line feed that ends a line of a pointer.
/// `&u8` borrows the byte,
/// as `split` hands it over.
fn is_line_feed_byte(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// Whether one byte is the space that separates a key of a pointer from its value.
/// `&u8` borrows the byte,
/// as
///  `position` hands it over.
fn is_space_byte(byte: &u8) -> bool {
    return *byte == b' ';
}

/// Whether `haystack` holds `needle`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    for window in haystack.windows(needle.len()) {
        if window == needle {
            return true;
        }
    }
    return false;
}

/// The object name of an `sha256:<64 digits>` value.
fn parse_oid(value: &[u8]) -> Option<String> {
    let digits: &[u8] = value.strip_prefix(b"sha256:")?;
    if digits.len() != 64 {
        return None;
    }
    for byte in digits {
        if !matches!(byte, b'0'..=b'9' | b'a'..=b'f') {
            return None;
        }
    }
    return Some(String::from_utf8_lossy(digits).into_owned());
}

/// Whether a key starts like an extension key,
/// `ext-` and one digit (`extRE`,
/// which `git-lfs` matches as a prefix).
fn is_extension_key(key: &[u8]) -> bool {
    return key.len() > 6
        && key.starts_with(b"ext-")
        && key[4].is_ascii_digit()
        && key[5] == b'-'
        && (key[6].is_ascii_alphanumeric() || key[6] == b'_');
}

/// What:
///  The verdict for the content of one blob.
/// Why:
///   One pass over at most [`BLOB_SIZE_CUTOFF`] bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decodePointer(blob: Uint8Array): Verdict;
/// ```
pub fn decode_pointer(blob: &[u8]) -> Verdict {
    if blob.len() >= BLOB_SIZE_CUTOFF || blob.is_empty() {
        return Verdict::NotUploadable;
    }
    let mut holds_matcher: bool = false;
    for word in MATCHERS {
        if contains(blob, word) {
            holds_matcher = true;
            break;
        }
    }
    if !holds_matcher {
        return Verdict::NotUploadable;
    }
    let trimmed: &[u8] = blob.trim_ascii();
    // Go trims Unicode white space too; a non-ASCII edge is not worth guessing about.
    let first_is_odd: bool = match trimmed.first() {
        Some(byte) => !byte.is_ascii(),
        None => false,
    };
    let last_is_odd: bool = match trimmed.last() {
        Some(byte) => !byte.is_ascii(),
        None => false,
    };
    let odd_edge: bool = first_is_odd || last_is_odd;
    let mut values: [Option<&[u8]>; 3] = [None; 3];
    let mut line_number: usize = 0;
    for raw in trimmed.split(is_line_feed_byte) {
        let line: &[u8] = raw.strip_suffix(b"\r").unwrap_or(raw);
        if line.is_empty() {
            continue;
        }
        let Some(space) = line.iter().position(is_space_byte) else {
            return if odd_edge { Verdict::Decline } else { Verdict::NotUploadable };
        };
        let (key, value): (&[u8], &[u8]) = (&line[..space], &line[space + 1..]);
        if line_number >= KEYS.len() {
            return if odd_edge { Verdict::Decline } else { Verdict::NotUploadable };
        }
        if key != KEYS[line_number] {
            // Extension lines carry their own objects and priorities; leave them to `git-lfs`.
            return if is_extension_key(key) || odd_edge { Verdict::Decline } else { Verdict::NotUploadable };
        }
        values[line_number] = Some(value);
        line_number += 1;
    }
    let [Some(version), Some(oid), Some(size)] = values else {
        return if odd_edge { Verdict::Decline } else { Verdict::NotUploadable };
    };
    let parsed_size: Option<i64> = match std::str::from_utf8(size) {
        Ok(text) => text.parse::<i64>().ok(),
        Err(_) => None,
    };
    // A negative size is no size at all.
    let size_value: Option<i64> = match parsed_size {
        Some(number) if number >= 0 => Some(number),
        _ => None,
    };
    let (Some(name), Some(bytes)) = (parse_oid(oid), size_value) else {
        return if odd_edge { Verdict::Decline } else { Verdict::NotUploadable };
    };
    if !VERSIONS.contains(&version) {
        return Verdict::NotUploadable;
    }
    return if bytes == 0 { Verdict::NotUploadable } else { Verdict::Pointer(name) };
}

/// The decoding is controlled in the test module.
#[cfg(test)]
#[path = "lfs_pointer_tests.rs"]
mod tests;
