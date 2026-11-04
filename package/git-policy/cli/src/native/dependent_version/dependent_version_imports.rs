//! What: Decide whether source text imports a package or one of its subpaths.
//! Why: A development dependency is bundled, and so carries a bump, only when non-test
//!      source imports it. This is the incumbent's specifier scan (`source-imports.ts:59-264`):
//!      a quoted specifier equal to the name, or starting with the name and `/`, directly
//!      after `from`, `import`, `import(` or `require(`. Every character it inspects is
//!      ASCII, so scanning UTF-8 bytes decides exactly what scanning UTF-16 units decides.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // importsPackage({ sourceText, packageName });
//! ```

/// What: Keywords that put a string literal directly after them in specifier position.
/// Why:  `import x from 'p'`, `export * from 'p'` and `import 'p'`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SPECIFIER_KEYWORDS = ['from', 'import'];
/// ```
const SPECIFIER_KEYWORDS: &[&[u8]] = &[b"from", b"import"];

/// What: Callees whose first argument is a module specifier.
/// Why:  `import('p')` and `require('p')`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SPECIFIER_CALLEES = ['import', 'require'];
/// ```
const SPECIFIER_CALLEES: &[&[u8]] = &[b"import", b"require"];

/// What: Whether a byte can continue a JavaScript identifier, for keyword boundaries.
/// Why:  `reimport 'p'` and `myrequire('p')` are not specifiers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isIdentifierCharacter = (c: string) => /[A-Za-z0-9_$]/.test(c);
/// ```
fn is_identifier_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$';
}

/// What: The exclusive end of the text before `end` once trailing whitespace is dropped.
///       `.rposition(...)` finds the last match.
/// Why:  Whitespace, including line breaks, may separate a keyword from its literal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function skipWhitespaceBackwards({ text, end }): number;
/// ```
fn skip_whitespace_backwards(text: &[u8], end: usize) -> usize {
    return text[..end]
        .iter()
        .rposition(|byte: &u8| return !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        .map_or(0, |index: usize| return index + 1);
}

/// What: Whether one of the keywords ends exactly at `end`, with no identifier byte before
///       it. `.checked_sub` is subtraction that yields nothing instead of going below zero.
/// Why:  A keyword must be a whole word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endsWithKeyword({ text, end, keywords }): boolean;
/// ```
fn ends_with_keyword(text: &[u8], end: usize, keywords: &[&[u8]]) -> bool {
    return keywords.iter().any(|keyword: &&[u8]| {
        let Some(start) = end.checked_sub(keyword.len()) else {
            return false;
        };
        let before: Option<&u8> = start
            .checked_sub(1)
            .and_then(|index: usize| return text.get(index));
        return text[start..end] == **keyword
            && !before.is_some_and(|byte: &u8| return is_identifier_byte(*byte));
    });
}

/// What: Whether the string literal opening at `quote_index` is in specifier position.
/// Why:  `const name = '@scope/b'` mentions a package without importing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSpecifierPosition({ text, quoteIndex }): boolean;
/// ```
fn is_specifier_position(text: &[u8], quote_index: usize) -> bool {
    let token_end: usize = skip_whitespace_backwards(text, quote_index);
    if ends_with_keyword(text, token_end, SPECIFIER_KEYWORDS) {
        return true;
    }
    // `let Some(...) = ... else` handles a literal at the very start, which has no `(`.
    let Some(paren) = token_end.checked_sub(1) else {
        return false;
    };
    return text[paren] == b'('
        && ends_with_keyword(
            text,
            skip_whitespace_backwards(text, paren),
            SPECIFIER_CALLEES,
        );
}

/// What: Whether source text imports the package or one of its subpaths.
///       `.match_indices(...)` yields each non-overlapping occurrence from the left, the
///       same occurrences the incumbent's `indexOf` loop visits.
/// Why:  The character before an occurrence must open a quote, the one after must close it
///       or start a subpath, and the literal must be in specifier position.
/// Gotcha: For an empty name the incumbent's loop never advances and does not terminate;
///         here every position is an occurrence, so the stated rule applies to it too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function importsPackage({ sourceText, packageName }): boolean;
/// ```
pub fn imports_package(source_text: &str, package_name: &str) -> bool {
    let text: &[u8] = source_text.as_bytes();
    return source_text
        .match_indices(package_name)
        .any(|(index, _): (usize, &str)| {
            let Some(quote_index) = index.checked_sub(1) else {
                return false;
            };
            let quote: u8 = text[quote_index];
            let following: Option<&u8> = text.get(index + package_name.len());
            return matches!(quote, b'\'' | b'"' | b'`')
                && (following == Some(&quote) || following == Some(&b'/'))
                && is_specifier_position(text, quote_index);
        });
}

/// Specifier detection, including every case of the incumbent's unit tests.
#[cfg(test)]
#[path = "dependent_version_imports_tests.rs"]
mod tests;
