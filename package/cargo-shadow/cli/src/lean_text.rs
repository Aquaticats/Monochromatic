//! What:
//! Small named reads of paths and text that the lean-folder tools share.
//! Why:
//! Each is a named function in place of a closure that captures nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const nameOf = (path: string): string => basename(path);
//! ```

/// Directory entries and paths.
use std::{fs::DirEntry, path::Path};

/// What:
/// `pub fn name_of(path: &Path) -> String` is the final component of a path as text,
/// empty for a path with none.
/// Why:
/// Names are compared as text throughout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const nameOf = (path: string): string => basename(path);
/// ```
pub fn name_of(path: &Path) -> String {
    return match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => String::new(),
    };
}

/// What:
/// `pub fn extension_is(path: &Path, wanted: &str) -> bool` says whether a path ends in the extension.
/// Why:
/// The readers pick `*.json`,
/// `*.d` and source files by extension.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const extensionIs = (path: string, wanted: string): boolean => extname(path) === `.${wanted}`;
/// ```
pub fn extension_is(path: &Path, wanted: &str) -> bool {
    return match path.extension() {
        Some(extension) => extension == wanted,
        None => false,
    };
}

/// What:
/// `pub fn is_directory(entry: &DirEntry) -> bool` says whether a directory entry is a directory (a link is not).
/// Why:
/// Walks never follow links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const isDirectory = (entry: Dirent): boolean => entry.isDirectory();
/// ```
pub fn is_directory(entry: &DirEntry) -> bool {
    return match entry.file_type() {
        Ok(kind) => kind.is_dir(),
        Err(_) => false,
    };
}

/// What:
/// `pub fn all_hex(text: &str) -> bool` says whether every character is a hexadecimal digit.
/// Why:
/// Shard directories and unit hashes are hexadecimal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const allHex = (text: string): boolean => /^[0-9a-fA-F]*$/.test(text);
/// ```
pub fn all_hex(text: &str) -> bool {
    for character in text.chars() {
        if !character.is_ascii_hexdigit() {
            return false;
        }
    }
    return true;
}
