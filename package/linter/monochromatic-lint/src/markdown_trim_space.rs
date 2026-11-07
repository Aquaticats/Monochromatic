//! What:
//!  The whitespace set that JavaScript's `String.trim` removes.
//! Why:
//!  The earlier linter trimmed with `String.trim`,
//!  whose set differs from Rust's `White_Space` for the byte order mark and NEXT LINE,
//!  and the LFS rules keep that boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Membership test usable as the pattern of a trim.
//! ```

/// What:
///  JavaScript trim's whitespace set,
///  including BOM and excluding NEXT LINE.
/// Why:
///  The incumbent trims cell padding with String.trim,
///  not Rust's different Unicode White_Space set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trimSpace(character): boolean { return character.trim() === ''; }
/// ```
pub(crate) fn trim_space(character: char) -> bool {
    // These scalar values are the ECMAScript trim characters; u32 conversion is unnecessary for membership.
    /// Scalar values that ECMAScript treats as whitespace when trimming padding.
    const CHARACTERS: &[char] = &[
        '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}', '\u{00a0}',
        '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
        '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}',
        '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
    ];
    // Borrow the character for this fixed-set membership test.
    return CHARACTERS.contains(&character);
}

/// The exact trim set is pinned by its own controls.
#[cfg(test)]
#[path = "markdown_trim_space_tests.rs"]
mod tests;
