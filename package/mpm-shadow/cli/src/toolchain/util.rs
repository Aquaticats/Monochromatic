//! What:
//! Small named helpers the toolchain modules share:
//! text predicates,
//! a tool lookup by id,
//! and file-name conversions.
//! Why:
//! The unified linter asks for a named function wherever a closure captures nothing or only borrows;
//! these are those functions,
//! written once and tested once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // util.ts
//! export const allDigits = (text: string) => /^\d*$/.test(text);
//! export const findTool = (tools: LockedTool[], id: string) => tools.find((tool) => tool.declaration.id === id);
//! ```

/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// Imports the locked tool type.
use super::model::LockedTool;

/// Whether every character of `text` is an ASCII digit (true for the empty text).
pub fn all_ascii_digits(text: &str) -> bool {
    for character in text.chars() {
        if !character.is_ascii_digit() {
            return false;
        }
    }
    return true;
}

/// Whether the first character of `text` is an ASCII digit.
pub fn starts_with_ascii_digit(text: &str) -> bool {
    return match text.chars().next() {
        Some(character) => character.is_ascii_digit(),
        None => false,
    };
}

/// Whether `text` contains any of `needles`.
pub fn contains_any(text: &str, needles: &[&str]) -> bool {
    for needle in needles {
        if text.contains(needle) {
            return true;
        }
    }
    return false;
}

/// Whether `text` ends with any of `suffixes`.
pub fn ends_with_any(text: &str, suffixes: &[&str]) -> bool {
    for suffix in suffixes {
        if text.ends_with(suffix) {
            return true;
        }
    }
    return false;
}

/// The tool with this id,
/// or `None`.
pub fn find_tool<'a>(tools: &'a [LockedTool], id: &str) -> Option<&'a LockedTool> {
    for tool in tools {
        if tool.declaration.id == id {
            return Some(tool);
        }
    }
    return None;
}

/// The final path component as text (invalid UTF-8 replaced),
/// or the empty text.
pub fn file_name_text(path: &Path) -> String {
    return match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => String::new(),
    };
}

/// The final path component as an owned `OsString`,
/// or an empty one.
pub fn file_name_os(path: &Path) -> OsString {
    return match path.file_name() {
        Some(name) => name.to_os_string(),
        None => OsString::new(),
    };
}

/// A version segment as a number (0 when it is not one).
pub fn number_or_zero(part: &str) -> u64 {
    return part.parse::<u64>().unwrap_or(0);
}
