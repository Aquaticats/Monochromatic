//! What:
//! Small named helpers the toolchain modules share:
//! text predicates,
//! a tool lookup by id,
//! file-name conversions,
//! and the one atomic file write.
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
/// Imports the file-system module the atomic write uses.
use std::fs;
/// Imports `Path` and `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// Imports the toolchain error type.
use super::error::ToolchainError;
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

/// What:
/// Writes `bytes` to `target` through a dot-named temporary beside it and a rename,
/// so a reader sees the old file or the new one,
/// never half of it.
/// Why:
/// Every file this crate replaces (a launcher,
/// a launch record,
/// the lock) is read by other programs without a lock,
/// and one owner keeps the temporary naming and the failure handling from drifting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function writeAtomically(target: string, bytes: Uint8Array): void
/// ```
pub fn write_atomically(target: &Path, bytes: &[u8]) -> Result<(), ToolchainError> {
    let mut temporary_name: OsString = OsString::from(".");
    temporary_name.push(file_name_os(target));
    temporary_name.push(format!(".{}.tmp", std::process::id()));
    let temporary: PathBuf = target.with_file_name(temporary_name);
    if let Err(error) = fs::write(&temporary, bytes) {
        // Best effort: a partial temporary is useless, and the error below is what matters.
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!("cannot write {}: {error}", temporary.display())));
    }
    if let Err(error) = fs::rename(&temporary, target) {
        // Best effort: the temporary file is useless now, and a failed removal changes nothing for the caller.
        let _ = fs::remove_file(&temporary);
        return Err(ToolchainError::failure(format!("cannot replace {}: {error}", target.display())));
    }
    return Ok(());
}
