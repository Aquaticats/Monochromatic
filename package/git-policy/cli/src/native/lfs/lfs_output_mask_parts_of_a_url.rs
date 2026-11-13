//! What:
//!  Each part of a URL as the wrapper shows it:
//! every path segment,
//! every query setting and every fragment setting,
//! with what a rule recognizes as a secret replaced by the mask.
//! Why:
//!  A segment is the part of a URL a person most needs after the host:
//! `owner/repo.git/info/lfs` says which repository the push went to,
//! and `objects/batch` says which call failed,
//! so a segment is shown unless a rule recognizes a secret in it.
//! A query's key is what says whether its value is a secret,
//! so the key stays and the value is hidden under a name that means one.
//! An LFS object's name is shown where an LFS server puts one,
//! because it is the thing a person needs of a failed object upload.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const shown = url.replace(/\/([^/?#]*)/g, (_, segment) => `/${holdsSecret(segment) ? MASK : segment}`)
//! ```

use super::CREDENTIALS_MASK;
use super::lfs_output_mask_reading_a_url::{begins_the_rest, is_ampersand, is_equals_sign, next_hash, next_slash};
use super::shapes::{holds_a_secret, is_a_credential, names_a_secret};

/// What:
///  How many lowercase hexadecimal characters an LFS object's name has:
/// the 256-bit hash the batch API calls its `oid`.
/// Why:
///   An object's name is not a secret:
/// it is a hash of the object's own bytes,
/// it is how a person names the object a failed upload lost,
/// and LFS writes it in a path segment
/// (`docs/api/batch.md` of git-lfs,
/// "`oid` - String OID of the LFS object",
/// and this repository's own server,
/// which answers a batch with `href` of `<origin>/<oid>`,
/// `package/config/lfs-r2-worker/src/batch.ts`,
/// "const href = `${origin}/${oid}`").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OBJECT_ID_BYTES = 64
/// ```
const OBJECT_ID_BYTES: usize = 64;

/// The name of the namespace an object's name sits in when an LFS server puts it there.
const OBJECTS_SEGMENT: &[u8] = b"objects";

/// Whether a byte is one this module keeps as the punctuation that ends a URL in a sentence.
/// `&&u8` borrows the byte twice,
/// as a reversed iterator's `take_while` hands it over.
fn ends_a_sentence(byte: &&u8) -> bool {
    return matches!(**byte, b',' | b';' | b')' | b']' | b'}' | b'.' | b'!' | b'?' | b':' | b'\'');
}

/// What:
///  Append one path segment:
/// as it is when it holds no secret,
/// the mask when it does,
/// and nothing when it is empty.
/// Why:
///   An empty segment (the one before a leading `/`,
/// after a trailing `/`,
/// or between two) carries nothing and stays empty,
/// and a segment that carries anything is shown unless a rule recognizes a secret in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// masked.push(segment === '' ? '' : maskIfSecret(segment))
/// ```
fn push_masked_segment(segment: &[u8], after_objects: bool, is_last: bool, masked: &mut Vec<u8>) {
    if segment.is_empty() {
        return;
    }
    if is_an_object_id(segment, after_objects, is_last) {
        masked.extend_from_slice(segment);
        return;
    }
    if holds_a_secret(segment) {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
        return;
    }
    masked.extend_from_slice(segment);
}

/// What:
///  Whether a path segment is an LFS object's name,
/// which is shown though its shape is a run of hexadecimal characters.
/// Why:
///   The mask hides a run of 32 or more hexadecimal characters,
/// and an object's name is 64 of them,
/// so without this rule the one thing a person needs to see of a failed object upload is hidden.
/// The name is recognized by where the batch API puts it,
/// not by its shape:
/// it is a whole segment of exactly [`OBJECT_ID_BYTES`] lowercase hexadecimal characters,
/// directly after a segment named `objects`,
/// or the last segment of the path,
/// which is where a content-addressed store puts the object it names
/// (this repository's server answers `<origin>/<oid>`,
/// and an S3-compatible bucket answers `<bucket>/<oid>`).
///  A secret written as a whole path segment of exactly that shape in one of those two places is shown
/// with it:
/// no rule can tell the two apart,
/// and this module resolves that in favour of the object's name,
/// which is the reading the owner gave the mask ("the thing a person needs to see").
/// The same 64 characters in a query value,
/// in a fragment,
/// or in the middle of a path are hidden.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAnObjectId = (segment, afterObjects, isLast) => (afterObjects || isLast) && /^[0-9a-f]{64}$/.test(segment)
/// ```
fn is_an_object_id(segment: &[u8], after_objects: bool, is_last: bool) -> bool {
    if !after_objects && !is_last {
        return false;
    }
    if segment.len() != OBJECT_ID_BYTES {
        return false;
    }
    return segment.iter().all(is_a_lowercase_hexadecimal_byte);
}

/// Whether a byte is one a lowercase hexadecimal number is written with.
fn is_a_lowercase_hexadecimal_byte(byte: &u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(byte);
}

/// Append the path of a URL with each of its secret-shaped segments masked.
fn push_masked_path(path: &[u8], masked: &mut Vec<u8>) {
    let mut from: usize = 0;
    let mut after_objects: bool = false;
    loop {
        match path.get(from..).and_then(next_slash) {
            Some(offset) => {
                let segment: &[u8] = &path[from..from + offset];
                push_masked_segment(segment, after_objects, false, masked);
                after_objects = segment == OBJECTS_SEGMENT;
                masked.push(b'/');
                from += offset + 1;
            }
            None => {
                let segment: &[u8] = path.get(from..).unwrap_or_default();
                push_masked_segment(segment, after_objects, true, masked);
                return;
            }
        }
    }
}

/// What:
///  Append a query or a fragment with the value of every setting that names a secret hidden,
/// and every value of a recognized shape hidden,
/// showing the keys and the separators.
/// Why:
///   The name of a setting is what says whether its value is a secret:
/// `?token=` and `?X-Amz-Signature=` hide their values whatever the values look like,
/// while `?ref=main` keeps a value a person reads,
/// so the mask hides the value and keeps the name,
/// which is the part that tells a person what the URL carried.
/// A value of a recognized shape is hidden under any name,
/// and a key that is itself a token of a recognized shape is hidden too,
/// because a token can be written as the name of a setting.
/// A setting with no `=` is a value with no name,
/// and is hidden when its shape is a secret's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chunks.map((chunk) => chunk.includes('=') ? `${key}=${value}` : maskBare(chunk)).join('&')
/// ```
fn push_masked_values(values: &[u8], masked: &mut Vec<u8>) {
    let mut from: usize = 0;
    loop {
        let (chunk, separator): (&[u8], Option<usize>) = match values.get(from..) {
            Some(rest) => match rest.iter().position(is_ampersand) {
                Some(offset) => (&values[from..from + offset], Some(from + offset)),
                None => (&values[from..], None),
            },
            None => (&[], None),
        };
        push_masked_setting(chunk, masked);
        match separator {
            Some(offset) => {
                masked.push(b'&');
                from = offset + 1;
            }
            None => return,
        }
    }
}

/// Append one query or fragment setting with the part of it that is a secret masked.
fn push_masked_setting(chunk: &[u8], masked: &mut Vec<u8>) {
    let Some(equals) = chunk.iter().position(is_equals_sign) else {
        if chunk.is_empty() {
            return;
        }
        if holds_a_secret(chunk) {
            masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
            return;
        }
        masked.extend_from_slice(chunk);
        return;
    };
    let key: &[u8] = &chunk[..equals];
    let value: &[u8] = &chunk[equals + 1..];
    if is_a_credential(key) {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
    } else {
        masked.extend_from_slice(key);
    }
    masked.push(b'=');
    if value.is_empty() {
        return;
    }
    if names_a_secret(key) || is_a_credential(value) {
        masked.extend_from_slice(CREDENTIALS_MASK.as_bytes());
        return;
    }
    masked.extend_from_slice(value);
}

/// What:
///  The punctuation that ends a URL written in a sentence,
/// and that is printed after the mask,
/// as in `(https://host/plain)` or `https://host/lfs,` .
/// Why:
///   A closing bracket or a comma cannot be told from a URL's own last byte,
/// and printing it as it stands costs nothing:
/// a segment that ends in it is shown whole or hidden whole,
/// so the most a reader learns from it is that punctuation was there.
///  A URL that really ends with such a byte keeps it,
/// with everything before it read as before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const punctuation = text.match(/[,;)\]}.!?:]*$/)[0];
/// ```
fn trailing_punctuation(rest: &[u8]) -> &[u8] {
    let kept: usize = rest
        .iter()
        .rev()
        .take_while(ends_a_sentence)
        .count();
    return &rest[rest.len() - kept..];
}

/// Append the rest of a URL (its path,
/// query and fragment) with its secrets masked,
/// and the punctuation that follows it as it stands.
pub(super) fn push_masked_rest(rest: &[u8], masked: &mut Vec<u8>) {
    let sentence_end: &[u8] = trailing_punctuation(rest);
    push_masked_url(&rest[..rest.len() - sentence_end.len()], masked);
    masked.extend_from_slice(sentence_end);
}

/// Append the path,
/// query and fragment of a URL with their secrets masked.
fn push_masked_url(rest: &[u8], masked: &mut Vec<u8>) {
    let path_end: usize = rest.iter().position(begins_the_rest).unwrap_or(rest.len());
    push_masked_path(&rest[..path_end], masked);
    let mut from: usize = path_end;
    if rest.get(from) == Some(&b'?') {
        masked.push(b'?');
        from += 1;
        let query_end: usize = match rest.get(from..).and_then(next_hash) {
            Some(offset) => from + offset,
            None => rest.len(),
        };
        push_masked_values(&rest[from..query_end], masked);
        from = query_end;
    }
    if rest.get(from) == Some(&b'#') {
        masked.push(b'#');
        push_masked_values(rest.get(from + 1..).unwrap_or_default(), masked);
    }
}
