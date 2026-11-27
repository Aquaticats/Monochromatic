//! What:
//!  The findings about how a folder's top doc lists its files:
//!  a list that is missing or nearly spells its marker,
//!  a name that is listed twice or names no file,
//!  a file that is not listed,
//!  and a file that is listed before a file it uses.
//! Why:
//!  A reader follows the list top to bottom,
//!  so a missing entry,
//! a foreign name and a backwards order
//!  are what break that reading.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function entryKey(shape: string, index: Path, name: string, base: Path): string;
//! function listFindings(doc: TopDoc, index: Path, listed: Path[], edges, component, base: Path): LayoutFinding[];
//! ```

/// Import the reference that proves one use out of the module the file rule re-exports it from.
use crate::layout_file_order_file_loops::Proof;
/// Import the finding shape,
///  the index file reader,
///  the reference that proves one use,
///  and the top doc with the marker it looks for.
use crate::{
    layout_finding::{
        LayoutFinding,
        key_path,
        shown,
    },
    layout_folder_index_files::file_name,
    layout_top_doc::{
        LIST_MARKER,
        TopDoc,
    },
};
/// Import ordered collections and native paths.
use std::{
    collections::BTreeMap,
    path::{
        Path,
        PathBuf,
    },
};

/// What:
///  The key of a finding whose last word is the name a top doc lists.
/// Why:
///  An entry whose backticks hold no name would end the key in the space a list reader trims away,
///  so an entry that names nothing is keyed by its shape and index alone,
///  and `unique_keys` numbers a second such entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entryKey(shape: string, index: Path, name: string, base: Path): string;
/// ```
fn entry_key(
    shape: &str,
    index: &Path,
    name: &str,
    base: &Path,
) -> String {
    let named: String = format!("{shape} {}", key_path(index, base));
    // What: an entry that names nothing contributes no word.
    // Why: no module file is named by an empty name, so the shape and the index name the finding alone.
    if name.is_empty() {
        return named;
    }
    return format!("{named} {name}");
}

/// What:
///  The findings about how a top doc lists a folder's files.
/// Why:
///  A file must be listed once,
/// a listed name must be a file of the folder,
///  and a file must come after every file it uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listFindings(doc: TopDoc, index: Path, listed: Path[], edges: Map<[number, number], Proof>, component: Map<number, number>, base: Path): LayoutFinding[];
/// ```
pub(crate) fn list_findings(
    doc: &TopDoc,
    index: &Path,
    listed: &[PathBuf],
    edges: &BTreeMap<(usize, usize), Proof>,
    component: &BTreeMap<usize, usize>,
    base: &Path,
) -> Vec<LayoutFinding> {
    let mut findings: Vec<LayoutFinding> = Vec::<LayoutFinding>::new();
    // The line a missing entry is reported on: the marker, or the first doc line when there is no list at all.
    let list_line: usize = doc.marker_line.unwrap_or(doc.first_line);
    // The marker broken after its comma, which is where the Markdown rule breaks it, and where both helps below write it.
    let (head, tail) = LIST_MARKER.split_once(", ").unwrap_or((LIST_MARKER, ""));
    if doc.marker_line.is_none() {
        findings.push(LayoutFinding {
            file: index.to_path_buf(),
            line: doc.first_line,
            message: format!(
                "The top doc of {} has no file list, so its files are not listed in order.",
                shown(index, base)
            ),
            help: Some(format!(
                "Add the two lines `//! {head},` and `//! {tail}` to the top doc, so the marker is wrapped where the Markdown rule `markdown/semantic-line-breaks` breaks it (it may wrap at any other word too), then one line `//! - `name.rs`: what it does` per file, each file after the files it uses."
            )),
            key: format!("no-file-list {}", key_path(index, base)),
        });
    }
    // A doc that spells the marker nearly is read as having no list at all.
    // One finding for each such place names the doc, the line, the words found and the marker expected,
    // so the person fixing it sees the cause, where the missing-list finding and every missing entry name only its effects.
    for near in &doc.near_misses {
        // What: where the words sit, naming both lines when a paragraph break splits them.
        // Why: the finding itself points at the one line the words start on, and the message spells out the rest.
        let place: String = match near.continues_at {
            Some(second) => format!("line {} and line {second}", near.line),
            None => format!("line {}", near.line),
        };
        findings.push(LayoutFinding {
            file: index.to_path_buf(),
            line: near.line,
            message: format!(
                "The top doc of {} nearly spells the file-list marker at {place}: it reads `{}` where the marker is `{LIST_MARKER}`, so the list after that text is read as no list at all.",
                shown(index, base),
                near.text
            ),
            help: Some(format!(
                "Write the marker's words exactly: `//! {head},` then `//! {tail}` on the next line, which is where the Markdown rule `markdown/semantic-line-breaks` breaks it, with no blank line, plain comment or inner attribute inside it."
            )),
            key: format!("near-miss-marker {}", key_path(index, base)),
        });
    }
    // A doc with no list still owes the reader one, so every unlisted file is reported on its own:
    // a file added to this folder later is a finding that no `no-file-list` line tolerates.
    let mut first_place: BTreeMap<String, usize> = BTreeMap::<String, usize>::new();
    for (place, entry) in doc.entries.iter().enumerate() {
        if first_place.contains_key(&entry.name) {
            findings.push(LayoutFinding {
                file: index.to_path_buf(),
                line: entry.line,
                message: format!(
                    "The top doc of {} lists `{}` twice.",
                    shown(index, base),
                    entry.name
                ),
                help: None,
                key: entry_key("listed-twice", index, entry.name.as_str(), base),
            });
            continue;
        }
        first_place.insert(entry.name.clone(), place);
    }
    let mut names: Vec<String> = Vec::<String>::new();
    for file in listed {
        names.push(file_name(file));
    }
    for entry in &doc.entries {
        if !names.contains(&entry.name) {
            findings.push(LayoutFinding {
                file: index.to_path_buf(),
                line: entry.line,
                message: format!(
                    "The top doc of {} lists `{}`, which is not a module file of this folder.",
                    shown(index, base),
                    entry.name
                ),
                help: Some(String::from("Remove the line, or correct the file name.")),
                key: entry_key("foreign-name", index, entry.name.as_str(), base),
            });
        }
    }
    for name in &names {
        if !first_place.contains_key(name) {
            findings.push(LayoutFinding {
                file: index.to_path_buf(),
                line: list_line,
                message: format!(
                    "The top doc of {} does not list the file `{name}`.",
                    shown(index, base)
                ),
                help: Some(format!(
                    "Add a line `//! - `{name}`: what it does` after the files it uses."
                )),
                key: format!("missing-entry {} {}", key_path(index, base), name),
            });
        }
    }
    for ((from, to), proof) in edges {
        let same_loop: bool =
            component.get(from).is_some() && component.get(from) == component.get(to);
        let (Some(at_from), Some(at_to)) =
            (first_place.get(&names[*from]), first_place.get(&names[*to]))
        else {
            continue;
        };
        if same_loop || at_to < at_from {
            continue;
        }
        findings.push(LayoutFinding {
            file: proof.file.clone(),
            line: proof.line,
            message: format!(
                "`{}` uses `{}` (`{}`) but the top doc of {} lists it first; a file may use only files listed before it.",
                names[*from],
                names[*to],
                proof.text,
                shown(index, base)
            ),
            help: Some(format!("List `{}` before `{}`.", names[*to], names[*from])),
            key: format!(
                "order {} {} {}",
                key_path(index, base),
                names[*from],
                names[*to]
            ),
        });
    }
    return findings;
}
