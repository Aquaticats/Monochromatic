//! What:
//!  The Git LFS object id of a working-tree file,
//!  whether it holds real bytes or a pointer.
//! Why:
//!  An object URL embeds this id;
//!  the rule compares it with the id already written in Markdown.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // pointer file -> its declared oid; any other file -> sha256(bytes) in lowercase hex
//! ```

/// Import the shared ECMAScript trim and the digest that defines a smudged file's id.
use crate::{markdown_lfs_config::js_trim, markdown_lfs_sha256::sha256_hex};

/// What:
///  The first line of every git-lfs pointer file.
/// Why:
///  Only text starting with this exact header is read as a pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POINTER_HEADER = 'version https://git-lfs.github.com/spec/v1';
/// ```
const POINTER_HEADER: &str = "version https://git-lfs.github.com/spec/v1";

/// What:
///  The key prefix of the pointer line carrying the object id.
/// Why:
///  The id follows this prefix on its own line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const POINTER_OID_PREFIX = 'oid sha256:';
/// ```
const POINTER_OID_PREFIX: &str = "oid sha256:";

/// What:
///  The number of hexadecimal characters in a SHA-256 object id.
/// Why:
///  Anything shorter or longer is not an object id,
///  whatever its characters.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OID_HEX_LENGTH = 64;
/// ```
const OID_HEX_LENGTH: usize = 64;

/// What:
///  Whether text is a well-formed object id:
///  exactly 64 lowercase hexadecimal characters.
/// Why:
///  An object URL's first path segment is an id only in this spelling;
///  uppercase is rejected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isLfsOid(value: string): boolean;
/// ```
pub(crate) fn is_lfs_oid(value: &str) -> bool {
    if value.len() != OID_HEX_LENGTH {
        return false;
    }
    for byte in value.bytes() {
        let digit: bool = byte.is_ascii_digit();
        let lower_hex: bool = (b'a'..=b'f').contains(&byte);
        if !digit && !lower_hex {
            return false;
        }
    }
    return true;
}

/// What:
///  The object id a pointer declares,
///  or `None` when the text is not a pointer or declares no well-formed id.
/// Why:
///  A checked-out pointer names its object without the object's bytes being present.
/// The first well-formed `oid sha256:` line wins;
///  malformed ones before it are skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pointerOid(text: string): string | undefined;
/// ```
fn pointer_oid(text: &str) -> Option<String> {
    if !text.starts_with(POINTER_HEADER) {
        return None;
    }
    for line in text.split('\n') {
        let Some(rest): Option<&str> = line.strip_prefix(POINTER_OID_PREFIX) else {
            continue;
        };
        let candidate: &str = js_trim(rest);
        if is_lfs_oid(candidate) {
            return Some(String::from(candidate));
        }
    }
    return None;
}

/// What:
///  The object id of file contents:
///  a pointer's declared id,
///  otherwise the SHA-256 of the bytes.
/// Why:
///  git-lfs assigns exactly that digest to smudged content,
///  so both checkout states give one answer.
/// Pointer detection decodes the bytes as UTF-8 with replacement,
///  as the incumbent's `toString('utf8')` does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lfsOidOfBytes(bytes: Uint8Array): string;
/// ```
pub(crate) fn lfs_oid_of_bytes(bytes: &[u8]) -> String {
    // Cow borrows valid UTF-8 unchanged and allocates only when replacement characters are needed.
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes);
    if let Some(declared) = pointer_oid(text.as_ref()) {
        return declared;
    }
    return sha256_hex(bytes);
}

/// Pointer and digest controls stay outside release artifacts.
#[cfg(test)]
#[path = "markdown_lfs_oid_tests.rs"]
mod tests;
