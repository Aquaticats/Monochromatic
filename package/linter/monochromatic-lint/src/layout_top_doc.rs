//! What:
//!  Read the list of files that a folder's top doc carries.
//! Why:
//!  The top doc is the `//!` lines at the start of the folder's `mod.rs`.
//!  The file rule asks it to list the folder's files, each after the files it uses.
//!  The format is plain enough for someone who knows only `git commit` and `git push`:
//!  a marker line, then one line per file,
//!  and everything else in the doc is free text that the rule never reads.
//!
//! ```text
//! //! Files in this folder, each using only files listed before it:
//! //!
//! //! - `saving_a_file.rs`: writes a file so a crash never leaves half of it.
//! //! - `saving_many_files.rs`: saves several files together.
//! ```
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DocEntry = { name: string; line: number };
//! type TopDoc = { markerLine?: number; entries: DocEntry[]; firstLine: number };
//! function readTopDoc(text: string): TopDoc;
//! ```

/// The exact line that starts the file list in a top doc.
pub const LIST_MARKER: &str = "Files in this folder, each using only files listed before it:";

/// What:
///  One listed file.
/// Why:
///  Findings about an entry point at its line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DocEntry = { name: string; line: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocEntry {
    /// The file name between the backticks,
    ///  such as `saving_a_file.rs`.
    pub name: String,
    /// One-based line of the entry in the file.
    pub line: usize,
}

/// What:
///  What a top doc says about the folder's files.
/// Why:
///  No marker means the doc has no list at all,
///  which is its own finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TopDoc = { markerLine?: number; entries: DocEntry[]; firstLine: number };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TopDoc {
    /// One-based line of the marker,
    ///  or nothing when the doc has none.
    pub marker_line: Option<usize>,
    /// The listed files in order.
    pub entries: Vec<DocEntry>,
    /// One-based line of the first `//!` line,
    ///  or one when the file has none.
    pub first_line: usize,
}

/// What:
///  The file name inside the first pair of backticks of a list line.
/// Why:
///  A list line is `- ` then the name in backticks,
///  then anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryName(text: string): string | undefined;
/// ```
fn entry_name(text: &str) -> Option<String> {
    let after_dash: &str = text.trim_start().strip_prefix("- `")?;
    let end: usize = after_dash.find('`')?;
    return Some(String::from(&after_dash[..end]));
}

/// What:
///  Read the top doc of a Rust file.
/// Why:
///  Only the leading `//!` lines count;
///  blank lines, ordinary comments and inner attributes between them are skipped,
///  and the first other line ends the doc.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readTopDoc(text: string): TopDoc;
/// ```
pub fn read_top_doc(text: &str) -> TopDoc {
    let mut doc: TopDoc = TopDoc {
        first_line: 1,
        ..TopDoc::default()
    };
    let mut seen_doc: bool = false;
    let mut list_open: bool = false;
    let mut list_done: bool = false;
    for (index, line) in text.lines().enumerate() {
        let number: usize = index + 1;
        let trimmed: &str = line.trim();
        let Some(body) = trimmed.strip_prefix("//!") else {
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#![") {
                continue;
            }
            break;
        };
        if !seen_doc {
            seen_doc = true;
            doc.first_line = number;
        }
        if list_done {
            continue;
        }
        if !list_open {
            if body.trim() == LIST_MARKER && doc.marker_line.is_none() {
                doc.marker_line = Some(number);
                list_open = true;
            }
            continue;
        }
        if body.trim().is_empty() || body.starts_with("  ") {
            continue;
        }
        match entry_name(body) {
            Some(name) => doc.entries.push(DocEntry { name, line: number }),
            None => list_done = true,
        }
    }
    return doc;
}

/// Controls of the top doc reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_top_doc_tests.rs"]
mod tests;
