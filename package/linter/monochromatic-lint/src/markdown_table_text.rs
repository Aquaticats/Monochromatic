//! What:
//!  Encode authored Markdown cell text for a raw HTML text boundary.
//! Why:
//!  HTML-sensitive escapes must become literal characters before HTML encoding,
//!  never active markup.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Normalize only the incumbent's HTML-sensitive Markdown escapes, then encode at interpolation.
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
    /// Scalar values that ECMAScript treats as whitespace when trimming table cell text.
    const CHARACTERS: &[char] = &[
        '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}', '\u{00a0}',
        '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
        '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200a}', '\u{2028}', '\u{2029}',
        '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}',
    ];
    // Borrow the character for this fixed-set membership test.
    return CHARACTERS.contains(&character);
}

/// Return whether one escaped character loses its Markdown backslash in raw HTML.
fn escape_target(character: char) -> bool {
    // Only the incumbent's HTML-sensitive set is normalized; ordinary Markdown markers stay literal.
    return ['|', '&', '<', '>', '"', '\''].contains(&character);
}

/// Append a single character encoded for the final HTML/MDX text context.
fn append_text(output: &mut String, character: char, mdx: bool) {
    // Mutably borrow the caller's owned String so each character is appended once.
    if character == '&' {
        output.push_str("&amp;");
    } else if character == '<' {
        output.push_str("&lt;");
    } else if character == '>' {
        output.push_str("&gt;");
    } else if character == '"' {
        output.push_str("&quot;");
    } else if character == '\'' {
        output.push_str("&#39;");
    } else if mdx && character == '{' {
        // JSX text braces must not turn an authored literal into an executable expression.
        output.push_str("&#123;");
    } else if mdx && character == '}' {
        output.push_str("&#125;");
    } else {
        output.push(character);
    }
}

/// What:
///  Convert raw cell content while preserving nonspecial Markdown spelling.
/// Why:
///  Code markers and emphasis remain literal;
///  angle brackets and quotes cannot inject HTML.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function htmlTableCellText(written, mdx): string;
/// ```
pub(crate) fn html_table_cell_text(written: &str, mdx: bool) -> String {
    // Own normalized content; &str would borrow immutable original text.
    let mut normalized: String = String::new();
    // Track an unconsumed escape marker without recursion or a second character-index buffer.
    let mut pending_slash: bool = false;
    for character in written.chars() {
        if pending_slash {
            if escape_target(character) {
                normalized.push(character);
                pending_slash = false;
                continue;
            }
            normalized.push('\\');
        }
        pending_slash = character == '\\';
        if !pending_slash {
            normalized.push(character);
        }
    }
    if pending_slash {
        normalized.push('\\');
    }
    // Borrow the normalized, trimmed view only while producing independently owned encoded output.
    let trimmed: &str = normalized.trim_matches(trim_space);
    let mut encoded: String = String::new();
    for character in trimmed.chars() {
        append_text(&mut encoded, character, mdx);
    }
    return encoded;
}

/// Keep adversarial syntax-boundary controls out of release artifacts.
#[cfg(test)]
#[path = "markdown_table_text_tests.rs"]
mod tests;
