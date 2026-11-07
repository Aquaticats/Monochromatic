//! What:
//!  A reader of Git configuration file text that gives the entries in file order,
//! and gives up on any spelling it
//!  cannot prove it reads as Git reads it.
//! Why:
//!  The in-process repository reads (location,
//! remotes,
//! upstream) need a few configuration values;
//!  Git answers a wrong or surprising spelling with its own error,
//!  so every spelling outside the plain grammar returns `None` and the caller asks Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries: { key: string; value: Uint8Array | undefined }[] | undefined = parseConfigText(bytes); // undefined: ask Git
//! ```

/// The largest configuration file this reader looks at.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
pub const CONFIG_TEXT_LIMIT: usize = 1 << 20;

/// What:
///  One `name = value` line with its section:
/// the canonical key and the value,
///  or no value for a bare name (Git reads that as the boolean true).
/// Why:
///   Section and name are lower case and the subsection keeps its case,
///  exactly the spelling `git config --list` prints,
/// so a caller compares keys as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigEntry = { key: string; value: Uint8Array | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigEntry {
    /// `section.subsection.name`,
    /// or `section.name` without a subsection.
    pub key: String,
    /// The value after escapes and quotes;
    /// `None` for a name with no `=`.
    pub value: Option<Vec<u8>>,
}

/// What:
///  A cursor over the bytes of one file.
/// Why:
///   The parser consumes the text one byte at a time and every helper must see the same position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Cursor { constructor(public bytes: Uint8Array, public at = 0) {} }
/// ```
struct Cursor<'a> {
    /// The whole file text.
    bytes: &'a [u8],
    /// The next byte to read.
    at: usize,
}

impl Cursor<'_> {
    /// The next byte,
    /// or `None` at the end.
    fn peek(&self) -> Option<u8> {
        return self.bytes.get(self.at).copied();
    }

    /// Step past the next byte.
    fn advance(&mut self) {
        self.at += 1;
    }

    /// Step past the rest of the line,
    /// newline included.
    fn skip_line(&mut self) {
        while let Some(byte) = self.peek() {
            self.advance();
            if byte == b'\n' {
                return;
            }
        }
    }
}

/// Whether `byte` may appear in a section or variable name.
fn is_name_byte(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || byte == b'-';
}

/// Read a `[section]`,
/// `[section "sub"]` or `[section.sub]` header after its `[` and give the key prefix.
///  A newline or any byte outside the grammar returns `None`.
fn read_header(cursor: &mut Cursor<'_>) -> Option<String> {
    let mut section: String = String::new();
    while let Some(byte) = cursor.peek() {
        if !(byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.') {
            break;
        }
        section.push(byte.to_ascii_lowercase() as char);
        cursor.advance();
    }
    if section.is_empty() {
        return None;
    }
    // A header with a quoted subsection: spaces, then `"..."`, in which `\x` stands for `x`.
    if cursor.peek() == Some(b' ') || cursor.peek() == Some(b'\t') {
        while matches!(cursor.peek(), Some(b' ' | b'\t')) {
            cursor.advance();
        }
        if cursor.peek() != Some(b'"') {
            return None;
        }
        cursor.advance();
        let mut subsection: Vec<u8> = Vec::new();
        loop {
            let byte: u8 = cursor.peek()?;
            cursor.advance();
            match byte {
                b'"' => break,
                b'\n' => return None,
                b'\\' => {
                    let escaped: u8 = cursor.peek()?;
                    if escaped == b'\n' {
                        return None;
                    }
                    cursor.advance();
                    subsection.push(escaped);
                }
                other => subsection.push(other),
            }
        }
        if cursor.peek() != Some(b']') {
            return None;
        }
        cursor.advance();
        if subsection.contains(&0) {
            return None;
        }
        let text: String = String::from_utf8(subsection).ok()?;
        return Some(format!("{section}.{text}"));
    }
    if cursor.peek() != Some(b']') {
        return None;
    }
    cursor.advance();
    // The old `[section.sub]` spelling lower-cases the subsection too, which `section` already is.
    return Some(section);
}

/// Read the value after `=`:
/// quotes,
/// the escapes `\n \t \b \\ \"`,
/// a backslash-newline continuation,
/// and a `#` or `;`
///  comment that ends the value.
/// A bad escape or an open quote returns `None`.
fn read_value(cursor: &mut Cursor<'_>) -> Option<Vec<u8>> {
    let mut value: Vec<u8> = Vec::new();
    let mut spaces: usize = 0;
    let mut quoted: bool = false;
    while let Some(byte) = cursor.peek() {
        cursor.advance();
        // Git's values are C strings; a NUL cannot be represented, so the spelling is left to it.
        if byte == 0 {
            return None;
        }
        if byte == b'\n' {
            if quoted {
                return None;
            }
            return Some(value);
        }
        if byte.is_ascii_whitespace() && !quoted {
            if !value.is_empty() {
                spaces += 1;
            }
            continue;
        }
        if (byte == b'#' || byte == b';') && !quoted {
            cursor.skip_line();
            return Some(value);
        }
        // Pending white space is kept only when more text follows it.
        value.extend(std::iter::repeat_n(b' ', spaces));
        spaces = 0;
        if byte == b'\\' {
            let escaped: u8 = cursor.peek()?;
            cursor.advance();
            match escaped {
                b'\n' => {}
                b'\r' if cursor.peek() == Some(b'\n') => cursor.advance(),
                b'n' => value.push(b'\n'),
                b't' => value.push(b'\t'),
                b'b' => value.push(8),
                b'\\' | b'"' => value.push(escaped),
                _ => return None,
            }
        } else if byte == b'"' {
            quoted = !quoted;
        } else {
            value.push(byte);
        }
    }
    if quoted {
        return None;
    }
    return Some(value);
}

/// What:
///  Parse the text of one configuration file into its entries,
/// in file order.
/// Why:
///   `None` means "some spelling here is outside the plain grammar",
/// never "the file is empty".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entries = parseConfigText(bytes); // undefined: ask Git
/// ```
pub fn parse_config_text(text: &[u8]) -> Option<Vec<ConfigEntry>> {
    if text.len() > CONFIG_TEXT_LIMIT {
        return None;
    }
    let body: &[u8] = text.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(text);
    let mut cursor: Cursor<'_> = Cursor { bytes: body, at: 0 };
    let mut prefix: Option<String> = None;
    let mut entries: Vec<ConfigEntry> = Vec::new();
    while let Some(byte) = cursor.peek() {
        if byte.is_ascii_whitespace() {
            cursor.advance();
        } else if byte == b'#' || byte == b';' {
            cursor.skip_line();
        } else if byte == b'[' {
            cursor.advance();
            prefix = Some(read_header(&mut cursor)?);
        } else if byte.is_ascii_alphabetic() {
            let section: &str = prefix.as_deref()?;
            let mut name: String = String::new();
            while let Some(next) = cursor.peek() {
                if !is_name_byte(next) {
                    break;
                }
                name.push(next.to_ascii_lowercase() as char);
                cursor.advance();
            }
            while matches!(cursor.peek(), Some(b' ' | b'\t' | b'\r')) {
                cursor.advance();
            }
            let key: String = format!("{section}.{name}");
            match cursor.peek() {
                Some(b'=') => {
                    cursor.advance();
                    entries.push(ConfigEntry { key, value: Some(read_value(&mut cursor)?) });
                }
                Some(b'\n') | None => entries.push(ConfigEntry { key, value: None }),
                Some(_) => return None,
            }
        } else {
            return None;
        }
    }
    return Some(entries);
}

/// What:
///  The last value of `key`,
/// with `Some(None)` for a bare name and `None` when the key is absent.
/// Why:
///   Git's single-valued reads take the last entry of the files in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// entries.findLast((entry) => entry.key === key)
/// ```
pub fn last_entry<'a>(entries: &'a [ConfigEntry], key: &str) -> Option<&'a ConfigEntry> {
    for entry in entries.iter().rev() {
        if entry.key == key {
            return Some(entry);
        }
    }
    return None;
}

/// What:
///  Git's boolean reading of a value:
/// `true yes on 1` and a bare name are true,
/// `false no off 0` are false.
/// Why:
///   Any other spelling (an empty value,
/// a larger number) is left to Git,
/// so the answer is `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parseBool = (value?: string) => value === undefined ? true : { true: true, false: false /* ... */ }[value.toLowerCase()];
/// ```
pub fn parse_bool(value: Option<&[u8]>) -> Option<bool> {
    let Some(text) = value else {
        return Some(true);
    };
    let lowered: String = String::from_utf8(text.to_vec()).ok()?.to_ascii_lowercase();
    return match lowered.as_str() {
        "true" | "yes" | "on" | "1" => Some(true),
        "false" | "no" | "off" | "0" => Some(false),
        _ => None,
    };
}

#[cfg(test)]
#[path = "repo_read_config_text_tests.rs"]
mod tests;
