//! What:
//! The scalar a line holds after its key or its dash:
//! single-quoted,
//! double-quoted,
//! and plain,
//! with the comment that ends a plain one.
//! Why:
//!  A scalar read wrongly is a name or a key read wrongly,
//! and the registry
//!       configuration is read for names;
//! this file keeps every scalar form and its refusals in one
//!       place,
//! under the line reader that splits the key from the value.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const scalar = readScalar('  - \'@s/a\'  # one');
//! ```

/// The whitespace test and the indicator test the line reader owns.
use super::{is_space_or_tab, refused_indicator};

/// What:
/// The scalar a line holds after its key or its dash.
/// Why:
///  A single-quoted scalar escapes its quote by doubling it,
/// a double-quoted
///       one uses backslash escapes,
/// and a plain one ends at a comment and may not
///       begin with an indicator this reader does not follow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readScalar(text: string): string;
/// ```
pub fn read_scalar(text: &str) -> Result<String, String> {
    let trimmed: &str = text.trim_end_matches(is_space_or_tab);
    if trimmed.starts_with('\'') || trimmed.starts_with('"') {
        let (value, rest): (String, &str) = read_quoted(trimmed)?;
        if rest.is_empty() {
            return Ok(value);
        }
        let unspaced: &str = rest.trim_start_matches(is_space_or_tab);
        if unspaced.len() < rest.len() && unspaced.starts_with('#') {
            return Ok(value);
        }
        return Err(String::from("holds text after its closing quote"));
    }
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if let Some(first) = refused_indicator(trimmed) {
        return Err(format!(
            "begins with {first:?}, which opens a value this reader does not read"
        ));
    }
    let plain: &str = plain_body(trimmed);
    if holds_a_mapping_colon(plain) {
        return Err(String::from(
            "holds a colon a space or tab follows, so it reads as a key and not as one value",
        ));
    }
    return Ok(String::from(plain));
}

/// What:
/// A plain scalar with its comment removed and its trailing spaces and tabs dropped.
/// Why:
///  YAML starts a comment at a `#` a space or a tab precedes,
/// and
///       trailing whitespace is not part of a scalar;
/// reading the comment as part of a package
///       name would hide the package the line names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plainBody = (text: string) => text.replace(/[ \t]#.*$/, '').trimEnd();
/// ```
fn plain_body(text: &str) -> &str {
    let mut previous: Option<char> = None;
    for (index, character) in text.char_indices() {
        if character == '#' && previous.is_some_and(is_space_or_tab) {
            return text[..index].trim_end_matches(is_space_or_tab);
        }
        previous = Some(character);
    }
    return text.trim_end_matches(is_space_or_tab);
}

/// What:
/// Whether a plain scalar holds a colon a space or a tab follows,
/// or ends with one.
/// Why:
///  Such a colon makes the line a key and a value to a YAML parser,
/// so a reader
///       that stored it as one scalar would report a value it never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsMappingColon = (text: string) => /:(?:[ \t]|$)/.test(text);
/// ```
fn holds_a_mapping_colon(text: &str) -> bool {
    for (index, character) in text.char_indices() {
        if character != ':' {
            continue;
        }
        let after: &str = &text[index + 1..];
        let ends: bool = match after.chars().next() {
            None => true,
            Some(next) => is_space_or_tab(next),
        };
        if ends {
            return true;
        }
    }
    return false;
}

/// What:
/// The text a quoted scalar holds and what follows it.
/// Why:
///  `''` inside single quotes is one quote,
/// and a double-quoted scalar reads
///       `\\` and `\"`,
/// refusing every other escape rather than dropping it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readQuoted(text: string): [string, string];
/// ```
pub(super) fn read_quoted(text: &str) -> Result<(String, &str), String> {
    let Some(quote) = text.chars().next() else {
        return Err(String::from("holds no quote"));
    };
    let body: &str = &text[quote.len_utf8()..];
    let mut value: String = String::new();
    let mut index: usize = 0;
    while index < body.len() {
        let Some(character) = body[index..].chars().next() else {
            break;
        };
        if character == quote && quote == '\'' && body[index + 1..].starts_with('\'') {
            value.push('\'');
            index += 2;
            continue;
        }
        if character == quote {
            return Ok((value, &body[index + 1..]));
        }
        if character == '\\' && quote == '"' {
            let Some(escaped) = body[index + 1..].chars().next() else {
                return Err(String::from("ends inside a backslash escape"));
            };
            let unescaped: char = match escaped {
                '\\' => '\\',
                '"' => '"',
                other => {
                    return Err(format!(
                        "uses the escape \\{other}, which this reader does not read"
                    ));
                }
            };
            value.push(unescaped);
            index += 1 + escaped.len_utf8();
            continue;
        }
        value.push(character);
        index += character.len_utf8();
    }
    return Err(String::from("opens a quote that never closes"));
}
