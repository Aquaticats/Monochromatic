//! What:
//!  Whether one audit generation names a saved copy of a conflicted file that still exists,
//! which is why the pass that
//!  removes generations must keep it.
//! Why:
//!  A `saved-conflict` line says where the content that vanished from a conflicted path went,
//! and it is the only account of that place.
//! A generation that
//!  names a copy still on disk therefore outlives the bound the other generations live under (the owner's answer of
//!  2026-11-10 in the question tool:
//! "an audit line naming a saved copy that still exists is never dropped").
//! The answer
//!  costs disk only in the worst case;
//! the other one costs a person the only pointer to their work.
//! The read is one pass over the generation,
//!  one line at a time,
//! so the memory is the longest line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for await (const line of readLines(generation)) if (line.includes('"outcome":"saved-conflict"') && namesACopyOnDisk(line)) return true;
//! return false;
//! ```

use std::io::BufRead;
use std::path::Path;

/// The words a saved-conflict line carries,
/// which no other line has.
const SAVED_CONFLICT_MARK: &[u8] = b"\"outcome\":\"saved-conflict\"";

/// The characters that stand for a byte the line's writer could not show as text:
/// a place that has one cannot be looked up.
const REPLACEMENT: char = '\u{FFFD}';

/// Whether a line holds the mark of a `saved-conflict` record anywhere in it.
fn holds_saved_conflict_mark(line: &[u8]) -> bool {
    for window in line.windows(SAVED_CONFLICT_MARK.len()) {
        if window == SAVED_CONFLICT_MARK {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether a `saved-conflict` line names a saved copy that exists,
/// or might.
/// Why:
///   The line lists `<path> => <place>`;
/// the path is the person's and may hold ` => `,
/// so every text after such a separator is
///  tried as a place.
/// A line the reader cannot parse,
/// and a place the writer could not show as text,
/// count as naming a copy:
///  the generation is kept,
/// because a wrong answer in that direction costs disk and the other direction costs the only
///  explanation of a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// line.copies.some((c) => c.split(' => ').slice(1).some((_, i) => existsSync(c.split(' => ').slice(i + 1).join(' => '))))
/// ```
fn line_names_a_copy(line: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(line) else {
        return true;
    };
    let Some(copies) = value.get("copies").and_then(serde_json::Value::as_array) else {
        return false;
    };
    for entry in copies {
        let Some(copy) = entry.as_str() else {
            continue;
        };
        for (index, _) in copy.match_indices(" => ") {
            let place: &str = &copy[index + " => ".len()..];
            if place == "none" {
                continue;
            }
            if place.contains(REPLACEMENT) || std::fs::symlink_metadata(place).is_ok() {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  Whether a generation holds a `saved-conflict` line that names a copy still on disk,
/// read in one pass over the file.
/// Why:
///   Only that line says where a copy went;
/// a pruned directory's line records a removal and explains nothing that is still
///  there.
/// This is the `pins` of the audit log's `LogRetention`,
/// so a generation it answers `true` for stays past the bound.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for await (const line of readLines(path)) if (line.includes('"outcome":"saved-conflict"') && namesACopy(line)) return true;
/// return false;
/// ```
///
/// # Errors
/// The system's error when the file cannot be opened or read.
pub fn names_a_copy_on_disk(path: &Path) -> std::io::Result<bool> {
    let mut reader: std::io::BufReader<std::fs::File> = std::io::BufReader::new(std::fs::File::open(path)?);
    let mut line: Vec<u8> = Vec::new();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            return Ok(false);
        }
        if holds_saved_conflict_mark(line.as_slice()) && line_names_a_copy(line.as_slice()) {
            return Ok(true);
        }
    }
}

/// The pin,
/// the mark and the doubtful answers are controlled in `discard_audit_generation_tests.rs`.
#[cfg(test)]
#[path = "discard_audit_generation_tests.rs"]
mod generation_tests;
