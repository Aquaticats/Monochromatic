//! What: Rewrite the top-level `version` value in a manifest's text, leaving every other
//!       byte unchanged.
//! Why: The bump is delivered as a full-content patch, so indentation, key order, line
//!      endings and the trailing newline must survive exactly. This is the incumbent's
//!      scanner (`manifest-text.ts:184-333`) over bytes: it tracks `{`/`[` depth outside
//!      strings, finds the first `"version"` key at depth one, and requires its raw value to
//!      be exactly the JSON quoting of the expected version.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // replaceManifestVersion({ path, text, from, to });
//! ```

/// What: `use` brings names from sibling files into this file.
/// Why:  Failures are shape problems with the incumbent's messages; versions are quoted as
///       `JSON.stringify` quotes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { shape } from './dependent_version_manifest.ts';
/// ```
use super::dependent_version_content::PolicyIncomplete;
/// The incumbent's shape-problem message for a manifest.
use super::dependent_version_manifest::shape;
/// `JSON.stringify` quoting of the expected and new versions.
use super::dependent_version_release::json_quote_units;

/// What: The raw spelling of the key this module looks for, with its quotes.
/// Why:  An escaped spelling such as `"version"` is a different key to the scanner,
///       as it is to the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const VERSION_KEY = '"version"';
/// ```
const VERSION_KEY: &[u8] = b"\"version\"";

/// What: The closing quote of the JSON string literal whose opening quote is at `start`.
///       `.enumerate().skip(n)` visits `(index, byte)` pairs from index `n` on.
/// Why:  A backslash hides the byte after it, whatever that byte is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stringLiteralEnd({ text, start }): number | undefined;
/// ```
fn literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut escaped: bool = false;
    for (index, byte) in bytes.iter().enumerate().skip(start + 1) {
        if escaped {
            escaped = false;
        } else if *byte == b'\\' {
            escaped = true;
        } else if *byte == b'"' {
            return Some(index);
        }
    }
    return None;
}

/// What: The index of the first byte at or after `start` that is not JSON whitespace, or
///       the length. `.position(...)` returns the offset of the first match.
/// Why:  JSON allows space, tab, carriage return and line feed around `:`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skipWhitespace({ text, start }): number;
/// ```
fn skip_whitespace(bytes: &[u8], start: usize) -> usize {
    return bytes
        .iter()
        .skip(start)
        .position(|byte: &u8| return !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        .map_or(bytes.len(), |offset: usize| return start + offset);
}

/// What: The incumbent's failure for a literal that never closes; it names no manifest.
/// Why:  Unreachable for text that parsed as JSON, but reachable for arbitrary input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new ManifestShapeError('manifest ends inside a string literal');
/// ```
fn unterminated(path: &[u8]) -> PolicyIncomplete {
    return PolicyIncomplete::ManifestShape {
        path: path.to_vec(),
        problem: String::from("manifest ends inside a string literal"),
    };
}

/// What: Replace the value of the first top-level `"version"` key, which must be exactly
///       `from` quoted, with `to` quoted. `&str` borrows text; the result is owned.
/// Why:  Only the version changes; a manifest whose first top-level `version` holds
///       something else (an escaped spelling, a different value, a duplicate key whose
///       first value differs) is refused rather than guessed at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function replaceManifestVersion({ path, text, from, to }): string;
/// ```
pub fn replace_manifest_version(
    path: &[u8],
    text: &str,
    from: &str,
    to: &str,
) -> Result<String, PolicyIncomplete> {
    let bytes: &[u8] = text.as_bytes();
    let mut depth: i64 = 0;
    // Bytes before `resume` belong to a string literal already read.
    let mut resume: usize = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if index < resume {
            continue;
        }
        match byte {
            b'{' | b'[' => depth += 1,
            b'}' | b']' => depth -= 1,
            b'"' => {
                let end: usize =
                    literal_end(bytes, index).ok_or_else(|| return unterminated(path))?;
                let after: usize = skip_whitespace(bytes, end + 1);
                if depth == 1
                    && &bytes[index..=end] == VERSION_KEY
                    && bytes.get(after) == Some(&b':')
                {
                    return replace_value(path, text, after + 1, from, to);
                }
                resume = end + 1;
            }
            _ => {}
        }
    }
    return Err(shape(path, "has no top-level \"version\""));
}

/// What: Replace the string value that starts after optional whitespace at `start`.
/// Why:  The value must be a string literal spelled exactly as `JSON.stringify(from)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// return `${text.slice(0, valueStart)}${JSON.stringify(to)}${text.slice(valueEnd + 1)}`;
/// ```
fn replace_value(
    path: &[u8],
    text: &str,
    start: usize,
    from: &str,
    to: &str,
) -> Result<String, PolicyIncomplete> {
    let bytes: &[u8] = text.as_bytes();
    let value_start: usize = skip_whitespace(bytes, start);
    if bytes.get(value_start) != Some(&b'"') {
        return Err(shape(path, "has a non-string top-level \"version\""));
    }
    let value_end: usize =
        literal_end(bytes, value_start).ok_or_else(|| return unterminated(path))?;
    // Both ends are ASCII quotes, so these byte offsets are character boundaries.
    let literal: &str = &text[value_start..=value_end];
    let expected: String = json_quote_units(&from.encode_utf16().collect::<Vec<u16>>());
    if literal != expected {
        return Err(shape(
            path,
            &format!("declares version {literal}, expected {expected}"),
        ));
    }
    let replacement: String = json_quote_units(&to.encode_utf16().collect::<Vec<u16>>());
    return Ok(format!(
        "{}{replacement}{}",
        &text[..value_start],
        &text[value_end + 1..]
    ));
}

/// The version rewrite, including every case of the incumbent's unit tests.
#[cfg(test)]
#[path = "dependent_version_text_tests.rs"]
mod tests;
