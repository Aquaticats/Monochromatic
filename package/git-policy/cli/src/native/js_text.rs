//! What:
//!  Text operations with the exact semantics of the JavaScript built-ins the incumbent
//!       applies to strings it shares with the native wrapper.
//! Why:
//!  Durable strings such as process birth identities are produced by `.trim()` in the
//!      incumbent;
//!  the native wrapper must produce byte-identical strings,
//!  and Rust's
//!      `str::trim` uses a different whitespace set (it trims U+0085 and keeps U+FEFF).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! text.trim()
//! ```

/// What:
///  Whether a character is whitespace or a line terminator to ECMAScript `trim`.
///       `char` is one Unicode scalar value.
/// Why:
///   ECMA-262 `TrimString` removes `WhiteSpace` (tab,
///  vertical tab,
///  form feed,
///  space,
///       no-break space,
///  U+FEFF and every `Zs` character) and `LineTerminator` (LF,
///  CR,
///       U+2028,
///  U+2029);
///  nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^\s$/u.test(character) // JavaScript's \s is exactly this set
/// ```
pub fn is_javascript_whitespace(character: char) -> bool {
    // The `Zs` block U+2000 to U+200A is a range; every other member is listed.
    if ('\u{2000}'..='\u{200a}').contains(&character) {
        return true;
    }
    // `matches!` tests a value against a pattern of alternatives.
    return matches!(
        character,
        '\u{0009}'
            | '\u{000a}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000d}'
            | '\u{0020}'
            | '\u{00a0}'
            | '\u{1680}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202f}'
            | '\u{205f}'
            | '\u{3000}'
            | '\u{feff}'
    );
}

/// What:
///  Remove leading and trailing JavaScript whitespace,
///  as `String.prototype.trim`.
///       `&str` borrows text;
///  the result borrows a slice of it.
/// Why:
///   Shared strings must be trimmed exactly as the incumbent trims them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trimJavascript(text: string): string { return text.trim(); }
/// ```
pub fn trim_javascript(text: &str) -> &str {
    return text.trim_matches(is_javascript_whitespace);
}

/// What:
///  Remove trailing JavaScript whitespace,
///  as `String.prototype.trimEnd`.
///       `&str` borrows text;
///  the result borrows a slice of it.
/// Why:
///   Git's PID file text is trimmed at the end only before its `pid ` prefix is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trimJavascriptEnd(text: string): string { return text.trimEnd(); }
/// ```
pub fn trim_javascript_end(text: &str) -> &str {
    return text.trim_end_matches(is_javascript_whitespace);
}

/// The whitespace table stays out of the release executable.
#[cfg(test)]
#[path = "js_text_tests.rs"]
mod tests;
