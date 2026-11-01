//! Escapes a bare literal line into the engine's always-verbose dialect.
//!
//! The forbidden-regex engine is always in verbose mode,
//!  so unescaped whitespace
//! is swallowed and a `#` at a line start begins a comment.
//!  A literal rule must
//! therefore be rewritten so every byte matches itself and nothing else:
//!  the
//! escapable metacharacters and every whitespace byte gain a leading backslash,
//! and every other byte passes through as a one-byte literal.
//!  This is a
//! syntax-boundary transformer (literal text into pattern syntax),
//!  so its one job
//! is that the compiled pattern matches exactly the input literal.

/// Reports whether a character must be backslash-escaped in the verbose dialect.
///
/// The set is exactly the bytes the engine's escape parser accepts as a literal
/// byte (`parse_escape`):
///  the metacharacters below plus every ASCII whitespace
/// byte.
///  Every other byte is a plain literal in atom position,
///  so escaping it is
/// unnecessary and (for non-listed bytes) would be an "unsupported escape".
///  The
/// whitespace test uses `is_ascii_whitespace`,
///  matching the engine's own cursor
/// and escape parser (space,
///  tab,
///  newline,
///  form feed,
///  carriage return;
///  not the
/// vertical tab,
///  which the engine treats as a plain literal byte).
fn needs_escape(ch: char) -> bool {
    // Whitespace is swallowed by verbose mode unless escaped.
    if ch.is_ascii_whitespace() {
        return true;
    }
    // Metacharacters special in atom, postfix, or verbose-comment position, each
    // of which the engine accepts as a literal byte when backslash-escaped.
    return matches!(
        ch,
        '.' | '[' | ']' | '(' | ')' | '{' | '}' | '?' | '|' | '&' | '~' | '^' | '$' | '\\' | '#'
            | '-' | '/' | '*' | '+'
    );
}

/// Escapes one literal line into a verbose-dialect pattern matching it exactly.
///
/// Each escapable metacharacter and whitespace byte gains a leading backslash;
/// every other character (ordinary ASCII,
///  and every byte of a multi-byte UTF-8
/// character,
///  all of which are non-ASCII and never in the escape set) is emitted
/// unchanged.
///  Only ASCII backslashes are inserted,
///  always before an ASCII byte,
/// so the result stays valid UTF-8 without a fallible reassembly step.
///
/// Declared `pub` (not `pub(super)`) so the frx module can re-export it under the
/// `fuzzing` feature for the `fuzz_literal_roundtrip` target;
///  the `rule` module is
/// private,
///  so this never widens the production public surface.
pub fn escape_literal(literal: &str) -> String {
    // Reserve a little slack for the backslashes escapable characters add.
    let mut out = String::with_capacity(literal.len() + literal.len() / 4);
    for ch in literal.chars() {
        if needs_escape(ch) {
            out.push('\\');
        }
        out.push(ch);
    }
    return out;
}

/// Exclusive byte-length ceiling below which a bare literal is matched only at word
/// boundaries.
///
/// A short run of word bytes collides with substrings of longer tokens far more
/// often than a long one:
///  a three-byte run like `ABC` turns up inside base64 blobs,
/// hex dumps,
///  and identifiers by chance,
///  and a bare literal otherwise matches every
/// such coincidence.
///  Gating a short literal on a word boundary at each word-byte end
/// confines it to whole-token matches,
///  while longer literals,
///  which rarely collide,
/// stay plain substring rules.
///  Eight is the smallest ceiling that still treats the
/// common short acronyms as whole words.
const WORD_BOUNDARY_MAX_LEN: usize = 8;

/// The `\b` word-boundary assertion inserted at a word-byte end of a short literal.
const WORD_BOUNDARY: &str = "\\b";

/// Reports whether a byte is an ASCII word byte (`[0-9A-Za-z_]`).
///
/// A `\b` boundary is defined only between a word byte and a non-word byte,
///  so it is
/// meaningful only next to one of these.
///  Every byte of a multi-byte UTF-8 character
/// is non-word,
///  which is why a CJK literal takes no boundary:
///  a `\b` there would
/// assert an ASCII boundary the surrounding CJK text never provides,
///  silencing the
/// rule.
fn is_word_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_';
}

/// Escapes a bare literal,
///  gating a short one behind word boundaries.
///
/// Delegates to [`escape_literal`] for the exact-match body,
///  then,
///  when the literal
/// is under [`WORD_BOUNDARY_MAX_LEN`] bytes,
///  prepends a boundary when its first byte
/// is a word byte and appends one when its last byte is a word byte.
///  So `ABC`
/// compiles to `\bABC\b` and no longer matches inside a base64 run,
///  while a literal
/// ending in a non-word byte (trailing punctuation,
///  or any byte of a CJK character)
/// keeps a plain end there.
///  A literal at or above the ceiling is escaped unchanged.
pub(super) fn literal_pattern(literal: &str) -> String {
    let escaped = escape_literal(literal);
    let bytes = literal.as_bytes();
    // Longer literals rarely collide with token substrings, so they keep the
    // documented plain-substring behavior.
    if literal.len() >= WORD_BOUNDARY_MAX_LEN {
        return escaped;
    }
    let lead = bytes.first().is_some_and(|&byte| return is_word_byte(byte));
    let trail = bytes.last().is_some_and(|&byte| return is_word_byte(byte));
    // Neither end anchors a boundary, so the escaping already is the whole pattern.
    if !lead && !trail {
        return escaped;
    }
    let mut out = String::with_capacity(escaped.len() + 2 * WORD_BOUNDARY.len());
    if lead {
        out.push_str(WORD_BOUNDARY);
    }
    out.push_str(&escaped);
    if trail {
        out.push_str(WORD_BOUNDARY);
    }
    return out;
}

/// Registers the round-trip and adversarial escaping tests (sidecar,
///  lint-exempt).
#[cfg(test)]
#[path = "escape_tests.rs"]
mod escape_tests;
