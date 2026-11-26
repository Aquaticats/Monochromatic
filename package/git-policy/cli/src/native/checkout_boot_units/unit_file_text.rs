//! What:
//!  The shape every file of this folder is written in:
//!  one comment naming the file's own path,
//!  its INI sections,
//!  and one final newline.
//! Why:
//!  a unit file without a final newline is read by systemd but not by every
//!  parser a reader uses,
//!  and a second newline leaves a blank tail that reads as an empty line,
//!  so the terminator is written once here rather than three times beside three
//!  texts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = fileText(['# /etc/systemd/system/x.service', '[Unit]', 'Description=x']);
//! ```

use std::path::Path;

/// One file's lines,
/// ending with exactly one newline.
///
/// Why:
///  every text in this folder ends the same way,
/// so a control reads the last
///  line without a special case per file.
pub fn file_text(lines: Vec<String>) -> String {
    let mut text: String = lines.join("\n");
    text.push('\n');
    return text;
}

/// The comment line naming the path a file is written to.
///
/// Why:
///  a file found on a machine names its own place,
/// so a reader compares what it
///  holds with where it is without a second document.
pub fn path_comment(path: &Path) -> String {
    return format!("# {}", path.display());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unit_file_text_tests.rs"]
mod tests;
