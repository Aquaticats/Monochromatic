//! Escape and verbose-whitespace rewrite pass for the dialect porter.
//!
//! Rewrites the escapes and unescaped whitespace the always-verbose engine
//! rejects,
//!  after normalization has run:
//!  `\z` to `$`,
//!  bare `\n`/`\r` dropped,
//! unnecessary escapes reduced,
//!  whitespace escaped,
//!  classes sanitized in place.
//! Split out of the `dialectport` bin to satisfy the max-lines budget.

/// Imports the character-class span helper shared with the porter.
use crate::port::class_end;

/// Reports whether `\x` is an escape the engine accepts in this position.
///
/// What:
///  the engine's fixed escape vocabulary (shorthands,
///  `\t`,
///  the escaped
/// metacharacters,
///  and escaped whitespace),
///  with `\b` legal only outside a class.
///  Why:
///  any
/// escape outside this set is a hard compile error,
///  so an unnecessary escape of a literal
/// (`\"`) must have its backslash dropped,
///  `\z`/`\n`/`\r` handled specially by the caller.
fn is_allowed_escape(x: u8, in_class: bool) -> bool {
    match x {
        b'd' | b'D' | b'w' | b'W' | b's' | b'S' | b't' => return true,
        b'b' => return !in_class,
        b'.' | b'[' | b']' | b'(' | b')' | b'{' | b'}' | b'?' | b'|' | b'&' | b'~' | b'^'
        | b'$' | b'\\' | b'#' | b'-' | b'/' | b'*' | b'+' => return true,
        other => return other.is_ascii_whitespace(),
    }
}

/// Rewrites escapes and unescaped whitespace that the always-verbose dialect rejects.
///
/// What:
///  `\z` becomes `$` (line end),
///  bare `\n`/`\r` are dropped (a scanned line carries no
/// newline byte),
///  an unnecessary escape of a non-metacharacter becomes the bare literal,
/// unescaped spaces and tabs outside a class are escaped (verbose mode swallows them
/// otherwise),
///  and classes are sanitized in place.
///  Why:
///  turns a PCRE body into one the
/// engine's escape and verbose rules accept without changing which bytes it matches.
pub(crate) fn fix_escapes(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c == b'\\' {
            if i + 1 >= b.len() {
                out.push('\\');
                i += 1;
                continue;
            }
            let n = b[i + 1];
            if n == b'z' {
                out.push('$');
            } else if n == b'n' || n == b'r' {
                // Dropped: CR and LF never occur inside a single scanned line.
            } else if is_allowed_escape(n, false) {
                out.push('\\');
                out.push(n as char);
            } else {
                out.push(n as char);
            }
            i += 2;
        } else if c == b'[' {
            let end = class_end(b, i);
            out.push_str(&sanitize_class(&s[i..end]));
            i = end;
        } else if c == b' ' || c == b'\t' {
            out.push('\\');
            out.push(c as char);
            i += 1;
        } else {
            out.push(c as char);
            i += 1;
        }
    }
    return out;
}

/// Sanitizes the escapes inside one `[...]` class.
///
/// What:
///  drops `\n`/`\r`,
///  reduces an unnecessary escape to its bare literal,
///  and keeps every
/// engine-legal escape;
///  whitespace and `#` stay literal (a class never runs verbose mode).
/// Why:
///  line 440's base64 class carries `\r\n` continuation bytes the engine rejects,
///  and no
/// class needs a non-metacharacter escaped.
fn sanitize_class(class: &str) -> String {
    let b = class.as_bytes();
    let end = b.len().saturating_sub(1);
    let mut out = String::from("[");
    let mut i = 1;
    while i < end {
        let c = b[i];
        if c == b'\\' && i + 1 < end {
            let n = b[i + 1];
            if n == b'n' || n == b'r' {
                // Dropped inside the class as well.
            } else if is_allowed_escape(n, true) {
                out.push('\\');
                out.push(n as char);
            } else {
                out.push(n as char);
            }
            i += 2;
        } else {
            out.push(c as char);
            i += 1;
        }
    }
    out.push(']');
    return out;
}
