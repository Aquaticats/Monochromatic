//! What:
//!  Whether a blob is a Git LFS pointer, decided the way `git-lfs` 3.8.1 decides it when it scans a push.
//! Why:
//!  The upload step finds the LFS objects the pushed commits introduce without starting `git-lfs`. A blob counts when
//!  the scanner would hand it to the transfer queue: smaller than the cutoff (`lfs/scanner.go`, `BlobSizeCutoff`), a
//!  header that names `git-media`, `hawser` or `git-lfs` (`lfs/pointer.go`, `matcherRE`), lines in the order `version`,
//!  `oid`, `size` with a known version, an `sha256:` object name of 64 lower-case hexadecimal digits and a non-negative
//!  decimal size; a pointer of size zero is skipped by `prepareUpload` (`commands/uploader.go`). The path of the blob and
//!  `.gitattributes` play no part. A spelling the port is not sure about (extension lines, non-ASCII edges) is a
//!  decline, so the caller falls back to `git-lfs` itself.
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
    /// The port cannot be sure; ask `git-lfs`.
    Decline,
}

/// Whether `haystack` holds `needle`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    return haystack.windows(needle.len()).any(|window| {
        return window == needle;
    });
}

/// The object name of an `sha256:<64 digits>` value.
fn parse_oid(value: &[u8]) -> Option<String> {
    let digits: &[u8] = value.strip_prefix(b"sha256:")?;
    let valid: bool = digits.len() == 64 && digits.iter().all(|byte| {
        return matches!(byte, b'0'..=b'9' | b'a'..=b'f');
    });
    return valid.then(|| {
        return String::from_utf8_lossy(digits).into_owned();
    });
}

/// Whether a key starts like an extension key, `ext-` and one digit (`extRE`, which `git-lfs` matches as a prefix).
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
    if !MATCHERS.iter().any(|word| {
        return contains(blob, word);
    }) {
        return Verdict::NotUploadable;
    }
    let trimmed: &[u8] = blob.trim_ascii();
    // Go trims Unicode white space too; a non-ASCII edge is not worth guessing about.
    let odd_edge: bool = trimmed.first().is_some_and(|byte| {
        return !byte.is_ascii();
    }) || trimmed.last().is_some_and(|byte| {
        return !byte.is_ascii();
    });
    let mut values: [Option<&[u8]>; 3] = [None; 3];
    let mut line_number: usize = 0;
    for raw in trimmed.split(|byte| {
        return *byte == b'\n';
    }) {
        let line: &[u8] = raw.strip_suffix(b"\r").unwrap_or(raw);
        if line.is_empty() {
            continue;
        }
        let Some(space) = line.iter().position(|byte| {
            return *byte == b' ';
        }) else {
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
    let size_value: Option<i64> = std::str::from_utf8(size).ok().and_then(|text| {
        return text.parse::<i64>().ok();
    });
    let (Some(name), Some(bytes)) = (parse_oid(oid), size_value.filter(|number| {
        return *number >= 0;
    })) else {
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
