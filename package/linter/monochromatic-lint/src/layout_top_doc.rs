//! What:
//!  Read the list of files that a folder's top doc carries.
//! Why:
//!  The top doc is the `//!` lines at the start of the folder's `mod.rs`.
//!  The file rule asks it to list the folder's files,
//! each after the files it uses.
//!  The format is plain enough for someone who knows only `git commit` and `git push`:
//!  a marker sentence,
//! then one line per file,
//!  and everything else in the doc is free text that the rule never reads.
//!  The marker is read as prose:
//! the Markdown rule breaks a sentence after its comma,
//!  so the marker may wrap anywhere a sentence may,
//!  and a blank line inside it ends the paragraph,
//! so the marker is then not found.
//!
//! ```text
//! //! Files in this folder,
//! //! each using only files listed before it:
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

/// The sentence that starts the file list in a top doc,
/// which the doc may wrap across lines.
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
///  How many leading lines of the given doc lines make up the list marker.
/// Why:
///  The Markdown rule breaks prose after a comma,
///  so the marker may wrap anywhere a sentence may.
///  The lines are read as one paragraph:
///  a blank line,
///  a plain comment or a different word ends it,
///  and any run of spaces matches one space.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function markerLineCount(markerWords: string[], bodies: string[]): number | undefined;
/// ```
fn marker_line_count(marker_words: &[&str], bodies: &[(usize, &str)]) -> Option<usize> {
    let mut matched: usize = 0;
    for (covered, (_, body)) in bodies.iter().enumerate() {
        // The words of this line, with every run of spaces gone.
        let words: Vec<&str> = body.split_whitespace().collect::<Vec<&str>>();
        // This line must be the next words of the marker, so a blank line or a different word ends the try.
        if words.is_empty()
            || marker_words.get(matched..matched + words.len()) != Some(words.as_slice())
        {
            // The marker is not here.
            return None;
        }
        matched += words.len();
        if matched == marker_words.len() {
            // The marker ended exactly at the end of this line.
            return Some(covered + 1);
        }
    }
    // The doc ended inside the marker.
    return None;
}

/// What:
///  Where the first marker starts among the doc lines and how many lines it covers.
/// Why:
///  The marker may begin at any line,
///  so every line is tried as its start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findMarker(bodies: string[]): { start: number; covered: number } | undefined;
/// ```
fn find_marker(bodies: &[(usize, &str)]) -> Option<(usize, usize)> {
    // Split once here, not once per line tried.
    let marker_words: Vec<&str> = LIST_MARKER.split_whitespace().collect::<Vec<&str>>();
    for start in 0..bodies.len() {
        if let Some(covered) = marker_line_count(marker_words.as_slice(), &bodies[start..]) {
            // The marker starts at this line and covers `covered` lines.
            return Some((start, covered));
        }
    }
    // No line starts the marker.
    return None;
}

/// What:
///  Read the top doc of a Rust file.
/// Why:
///  Only the leading `//!` lines count;
///  blank lines,
/// ordinary comments and inner attributes between them are skipped,
///  and the first other line ends the doc.
///  A skipped line still ends a paragraph,
///  so it never joins two halves of the marker.
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
    let mut bodies: Vec<(usize, &str)> = Vec::<(usize, &str)>::new();
    for (index, line) in text.lines().enumerate() {
        let number: usize = index + 1;
        let trimmed: &str = line.trim();
        let Some(body) = trimmed.strip_prefix("//!") else {
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("#![") {
                if seen_doc {
                    bodies.push((number, ""));
                }
                continue;
            }
            break;
        };
        if !seen_doc {
            seen_doc = true;
            doc.first_line = number;
        }
        bodies.push((number, body));
    }
    let Some((start, covered)) = find_marker(&bodies) else {
        return doc;
    };
    doc.marker_line = Some(bodies[start].0);
    for &(number, body) in &bodies[start + covered..] {
        if body.trim().is_empty() || body.starts_with("  ") {
            continue;
        }
        match entry_name(body) {
            Some(name) => doc.entries.push(DocEntry { name, line: number }),
            None => break,
        }
    }
    return doc;
}

/// Controls of the top doc reader stay outside release artifacts.
#[cfg(test)]
#[path = "layout_top_doc_tests.rs"]
mod tests;
