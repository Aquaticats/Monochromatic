//! What:
//!  Quoting of bytes that are printed inside a shell command for a person to copy and run.
//! Why:
//!  A path or a kernel argument can hold spaces,
//! quotes,
//! `;`,
//! `$(...)` or bytes that are not UTF-8.
//! Printed bare,
//! a
//!  pasted command runs whatever follows the separator or substitution,
//! or passes the wrong arguments.
//! A value of safe
//!  characters is printed as it is;
//! any other value is wrapped in single quotes (each single quote inside closed,
//!  escaped and opened again);
//! a value with a control character or a byte that is not UTF-8 is spelled in the ANSI-C
//!  form `$'...'` of bash,
//! zsh and ksh with those bytes written as `\xHH` escapes,
//! so the printed text holds no control
//!  character and no line break.
//! The same rules as `package/wg/cli-quicker/src/shell-quote.ts`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! shellQuote({ value: "it's" }); // "'it'\\''s'"
//! ```

/// Whether a byte is one a POSIX shell leaves alone in a word.
fn is_bare(byte: u8) -> bool {
    return byte.is_ascii_alphanumeric() || matches!(byte, b'@' | b'%' | b'+' | b'=' | b':' | b',' | b'.' | b'/' | b'_' | b'-');
}

/// Whether a character is a C0 control,
/// DEL or a C1 control.
fn is_control(character: char) -> bool {
    return character.is_control();
}

/// The ANSI-C spelling of one byte.
fn escaped_byte(byte: u8) -> String {
    return format!("\\x{}", crate::hexadecimal_of_bytes::hexadecimal_of_byte(byte));
}

/// The ANSI-C spelling of one character.
fn ansi_character(character: char) -> String {
    return match character {
        '\\' => String::from("\\\\"),
        '\'' => String::from("\\'"),
        '\n' => String::from("\\n"),
        '\t' => String::from("\\t"),
        other if is_control(other) => {
            let mut buffer: [u8; 4] = [0; 4];
            other.encode_utf8(&mut buffer).bytes().map(escaped_byte).collect()
        }
        other => other.to_string(),
    };
}

/// What:
///  Whether every byte of a value is one a shell reads back unquoted.
/// Why:
///   A value of only bare bytes needs no quotes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bytes.every((byte) => isBare(byte))
/// ```
fn all_bare(value: &[u8]) -> bool {
    for byte in value {
        if !is_bare(*byte) {
            return false;
        }
    }
    return true;
}

/// What:
///  Quote bytes so a POSIX shell (bash,
/// zsh or ksh for the `$'...'` form) reads them back as one word.
///  `&[u8]` is a borrowed byte view;
/// a path's bytes come from `OsStrExt::as_bytes`.
/// Why:
///   See the module description.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shellQuote({ value: bytes });
/// ```
pub fn shell_quote(value: &[u8]) -> String {
    if value.is_empty() {
        return String::from("''");
    }
    if all_bare(value) {
        return value.iter().copied().map(char::from).collect();
    }
    let mut spelled: String = String::new();
    let mut needs_ansi: bool = false;
    for chunk in value.utf8_chunks() {
        for character in chunk.valid().chars() {
            needs_ansi = needs_ansi || is_control(character);
            spelled.push_str(ansi_character(character).as_str());
        }
        for byte in chunk.invalid() {
            needs_ansi = true;
            spelled.push_str(escaped_byte(*byte).as_str());
        }
    }
    if needs_ansi {
        return format!("$'{spelled}'");
    }
    let plain: String = String::from_utf8_lossy(value).replace('\'', "'\\''");
    return format!("'{plain}'");
}

/// Shell quoting controls stay out of the release executable.
#[cfg(test)]
#[path = "shell_quote_tests.rs"]
mod tests;
