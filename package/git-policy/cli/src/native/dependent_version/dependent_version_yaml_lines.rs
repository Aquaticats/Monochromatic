//! What:
//! Read one line of block YAML:
//! its indentation,
//! the key before a colon,
//! and the scalar
//!       after one.
//! Why:
//! The generated registry configuration is the only YAML this executable reads,
//! and no crate it
//!      links parses YAML
//! (`package/config/pnpr` uses the `yaml` npm package,
// which a Rust
//!      process cannot call).
//! Rather than add a parser the lockfile does not carry,
// this folder reads the
//!      one structure the wrapper needs and refuses every text it cannot read,
//! so an unread
//!      configuration is reported instead of guessed at.
//! This file is the line-level half:
//! comment and blank lines dropped,
//! indentation counted,
//! and a line split into its key and its scalar.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const [key, rest] = splitKey(line.content);
//! ```

/// What:
/// One content line of a document:
/// where it sits and what it holds.
/// Why:
///  Block structure is decided by indentation,
/// and a diagnostic has to name the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type YamlLine = { number: number; indent: number; content: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YamlLine<'text> {
    /// Its place in the file,
    /// counted from one.
    pub number: usize,
    /// How many spaces precede its content.
    pub indent: usize,
    /// Its content without indentation,
    /// trailing spaces or a comment line.
    pub content: &'text str,
}

/// What:
/// Why a text is not block YAML this reader reads.
/// Why:
///  The reader refuses what it cannot read,
/// so the policy can say which line stopped
///       it rather than read a wrong value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type YamlProblem = { line: number; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct YamlProblem {
    /// The line the reader stopped at,
    /// counted from one.
    pub line: usize,
    /// What is wrong with it,
    /// in one clause without a final period.
    pub reason: String,
}

/// What:
/// A problem at one line.
/// Why:
///  Every refusal names the line it read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const problem: YamlProblem = { line, reason };
/// ```
impl YamlProblem {
    /// The problem at `line` for `reason`.
    pub fn new(line: usize, reason: &str) -> YamlProblem {
        return YamlProblem {
            line,
            reason: String::from(reason),
        };
    }
}

/// What:
/// The lines that carry content,
/// with their indentation.
/// `.enumerate()` pairs each line with its index;
///       `.strip_suffix`/`.strip_prefix` remove a known end or start.
/// Why:
///  Blank lines and whole-line comments carry no structure;
/// a tab may not
///       indent a block in YAML,
/// and counting it as spaces would misread the nesting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readLines(text: string): YamlLine[] | YamlProblem;
/// ```
pub fn read_lines(text: &str) -> Result<Vec<YamlLine<'_>>, YamlProblem> {
    let mut lines: Vec<YamlLine<'_>> = Vec::new();
    for (index, raw) in text.split('\n').enumerate() {
        let number: usize = index + 1;
        let line: &str = raw.strip_suffix('\r').unwrap_or(raw);
        let unindented: &str = line.trim_start_matches(' ');
        let indent: usize = line.len() - unindented.len();
        if unindented.starts_with('\t') {
            return Err(YamlProblem::new(
                number,
                "is indented with a tab, which starts no block",
            ));
        }
        let content: &str = unindented.trim_end_matches(' ');
        if content.is_empty() || content.starts_with('#') {
            continue;
        }
        if indent == 0 && (content == "---" || content == "...") {
            return Err(YamlProblem::new(
                number,
                "marks a document boundary, and this reader reads one document",
            ));
        }
        lines.push(YamlLine {
            number,
            indent,
            content,
        });
    }
    return Ok(lines);
}

/// What:
/// Whether a content line opens a sequence item.
/// Why:
///  An item is a dash alone or a dash and its content;
/// `-1` is a scalar
///       because no space follows the dash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isItem = (content: string) => content === '-' || content.startsWith('- ');
/// ```
pub fn is_item(content: &str) -> bool {
    return content == "-" || content.starts_with("- ");
}

/// What:
/// The content of a sequence item,
/// which is nothing for a lone dash.
/// Why:
///  The dash and the space before the content are not part of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itemContent = (content: string) => content.replace(/^- ?/, '').trimStart();
/// ```
pub fn item_content(content: &str) -> &str {
    return content
        .strip_prefix("- ")
        .unwrap_or("")
        .trim_start_matches(' ');
}

/// What:
/// Whether a character separates text on a line:
/// a space or a tab.
/// Why:
///  `-`,
/// `?` and `:` begin a plain scalar only when a non-space follows,
/// and a
///       tab separates as a space does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSpaceOrTab = (c: string) => c === ' ' || c === '\t';
/// ```
fn is_space_or_tab(character: char) -> bool {
    return character == ' ' || character == '\t';
}

/// What:
/// The indicator a plain scalar or key begins with,
/// when this reader does not follow it.
/// Why:
///  A tag (`!`),
/// anchor (`&`),
/// alias (`*`),
/// block scalar (`|`,
///       `>`),
/// directive (`%`),
/// reserved character (`@`,
/// backtick),
/// flow collection
///       (`[`, `]`, `{`, `}`,
/// `,`),
/// comment (`#`) or quote makes the line mean something other
///       than the text this reader would store;
/// `-`,
/// `?` and `:` begin a plain
///       scalar only when a non-space follows,
/// so an item indicator or an explicit
///       key is refused too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refusedIndicator(text: string): string | undefined;
/// ```
fn refused_indicator(text: &str) -> Option<char> {
    let Some(first): Option<char> = text.chars().next() else {
        return None;
    };
    if matches!(
        first,
        '!' | '&' | '*' | '|' | '>' | '%' | '@' | '`' | '[' | ']' | '{' | '}' | ',' | '#' | '\'' | '"'
    ) {
        return Some(first);
    }
    if matches!(first, '-' | '?' | ':') {
        let after: &str = &text[first.len_utf8()..];
        let separated: bool = match after.chars().next() {
            None => true,
            Some(next) => is_space_or_tab(next),
        };
        if separated {
            return Some(first);
        }
    }
    return None;
}

/// What:
/// The key and the rest of a line,
/// or nothing when the line holds no key.
/// `Err` is why a line that looks like a key is not one this reader reads.
/// Why:
///  A key ends at a colon a space or the line end follows;
/// a quoted key ends
///       at its closing quote,
/// so the quoted registry pattern `'@scope/*':` is one key.
/// A key this reader cannot read is refused rather than stored,
/// because
///       a key it kept verbatim would silently answer "no such entry".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function splitKey(content: string): [string, string] | undefined;
/// ```
pub fn split_key(content: &str) -> Result<Option<(String, &str)>, String> {
    if content.starts_with('\'') || content.starts_with('"') {
        let (key, rest): (String, &str) = read_quoted(content)?;
        let unspaced: &str = rest.trim_start_matches(' ');
        let Some(after): Option<&str> = unspaced.strip_prefix(':') else {
            return Ok(None);
        };
        if key.is_empty() {
            return Err(String::from("names no key"));
        }
        return Ok(Some((key, after.trim_start_matches(' '))));
    }
    for (index, character) in content.char_indices() {
        if character != ':' {
            continue;
        }
        let after: &str = &content[index + 1..];
        let separated: bool = after.is_empty() || after.starts_with(' ');
        if !separated {
            continue;
        }
        let key: &str = content[..index].trim_end_matches(' ');
        if key.is_empty() {
            return Err(String::from("names no key"));
        }
        if let Some(first) = refused_indicator(key) {
            return Err(format!(
                "begins with {first:?}, which opens a key this reader does not read"
            ));
        }
        return Ok(Some((
            String::from(key),
            after.trim_start_matches(' '),
        )));
    }
    return Ok(None);
}

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
    let trimmed: &str = text.trim_end_matches(' ');
    if trimmed.starts_with('\'') || trimmed.starts_with('"') {
        let (value, rest): (String, &str) = read_quoted(trimmed)?;
        let unspaced: &str = rest.trim_start_matches(' ');
        if unspaced.is_empty() || unspaced.starts_with('#') {
            return Ok(value);
        }
        return Err(String::from("holds text after its closing quote"));
    }
    let Some(first) = trimmed.chars().next() else {
        return Ok(String::new());
    };
    if matches!(
        first,
        '!' | '&' | '*' | '|' | '>' | '%' | '@' | '`' | '[' | '{'
    ) {
        return Err(format!(
            "begins with {first:?}, which opens a value this reader does not read"
        ));
    }
    let uncommented: &str = match trimmed.find(" #") {
        Some(index) => &trimmed[..index],
        None => trimmed,
    };
    let plain: &str = uncommented.trim_end_matches(' ');
    if plain.contains(": ") || plain.ends_with(':') {
        return Err(String::from(
            "holds a colon a space follows, so it reads as a key and not as one value",
        ));
    }
    return Ok(String::from(plain));
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
fn read_quoted(text: &str) -> Result<(String, &str), String> {
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

/// Line reading,
/// including the forms the generator writes and the ones it refuses.
#[cfg(test)]
#[path = "dependent_version_yaml_lines_tests.rs"]
mod tests;
