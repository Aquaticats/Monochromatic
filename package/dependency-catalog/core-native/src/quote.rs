//! What:
//! Quoting text as a double-quoted string that is valid in both diagnostics and TOML basic strings.
//! Why:
//! The TypeScript tool used `JSON.stringify` for both;
//! this is the same escape set,
//! so the rendered catalog
//! and the messages read the same,
//! and TOML also forbids the raw DEL character,
//! which is escaped here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function quote(value: string): string { return JSON.stringify(value); }
//! ```

/// What:
/// `pub fn quote(value: &str) -> String` returns `value` wrapped in `"` with `"`,
/// `\`,
/// control characters
/// and DEL escaped (`\n`,
/// `\t`,
/// `\u001f` ...).
/// Non-ASCII text stays as is.
/// Why:
/// A git URL or a package name can hold any text;
/// escaping at the final interpolation keeps the TOML valid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function quote(value: string): string { return JSON.stringify(value); }
/// ```
pub fn quote(value: &str) -> String {
    let mut quoted: String = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            '\u{8}' => quoted.push_str("\\b"),
            '\u{c}' => quoted.push_str("\\f"),
            other if (other as u32) < 0x20 || other == '\u{7f}' => {
                quoted.push_str(&format!("\\u{:04x}", other as u32));
            }
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    return quoted;
}
