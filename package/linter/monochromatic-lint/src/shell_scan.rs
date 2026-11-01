//! What:
//!  The scan that finds banned command-composition constructs in one shell run body.
//! Why:
//!  The rules count occurrences,
//!  so one linear pass per dialect collects every hit with its byte range.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Hit = { construct: Construct; start: number; end: number };
//! function scan(text: string, dialect: ShellDialect): Hit[];
//! ```

/// Import the dialect and construct tables.
use crate::shell_dialect::{Construct, ShellDialect};

/// What:
///  One banned construct and the bytes it occupies in the run body.
/// Why:
///  A finding needs the exact range to label,
///  and the rules count hits per construct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Hit = { construct: Construct; start: number; end: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Hit {
    /// Which construct was found.
    pub construct: Construct,
    /// First byte of the construct.
    pub start: usize,
    /// One past the last byte of the construct.
    pub end: usize,
}

/// What:
///  End of a GitHub expression `${{ ... }}` that starts at `index`,
///  or `None` when none starts there.
/// Why:
///  Expressions are GitHub syntax,
///  not shell,
///  so `${{ a && b }}` must never count.
///  An expression with no closing braces runs to the end of the body.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function expressionEnd(text: string, index: number): number | undefined;
/// ```
pub(crate) fn expression_end(bytes: &[u8], index: usize) -> Option<usize> {
    // `&[u8]` is a borrowed view of bytes, like a read-only Uint8Array; `starts_with` compares a prefix.
    if !bytes[index..].starts_with(b"${{") {
        return None;
    }
    let mut cursor: usize = index + 3;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'}' && bytes[cursor + 1] == b'}' {
            return Some(cursor + 2);
        }
        cursor += 1;
    }
    return Some(bytes.len());
}

/// What:
///  Find every banned construct the dialect gives a meaning to.
/// Why:
///  A construct the dialect does not give a command-composing meaning is dropped here,
///  so no rule has to know the dialect table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(text: string, dialect: ShellDialect): Hit[];
/// ```
pub fn scan(text: &str, dialect: ShellDialect) -> Vec<Hit> {
    let found: Vec<Hit> = match dialect {
        ShellDialect::Posix => crate::shell_scan_posix::scan(text, false),
        ShellDialect::Fish => crate::shell_scan_posix::scan(text, true),
        ShellDialect::PowerShell => crate::shell_scan_powershell::scan(text),
        ShellDialect::Cmd => crate::shell_scan_cmd::scan(text),
    };
    let mut kept: Vec<Hit> = Vec::new();
    for hit in found {
        if hit.construct.applies_to(dialect) {
            kept.push(hit);
        }
    }
    return kept;
}

/// Cross-dialect scan controls stay outside release artifacts.
#[cfg(test)]
#[path = "shell_scan_tests.rs"]
mod tests;
