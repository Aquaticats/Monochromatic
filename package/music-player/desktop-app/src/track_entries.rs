//! Flat track rows for one folder:
//!  titles,
//!  library indexes,
//!  the current flag and supporting lines.
//!
//! Ported from `core/TrackListEntries.kt`,
//!  except the template evaluation.
//!  The supporting line
//! comes from a caller-supplied function that stands in for the template engine,
//!  and this module
//! owns only the trim and empty-to-`None` step that follows it.

/// Folder list types that the track rows are built from.
use crate::folder_index::{FolderEntry, FolderListItem, FolderTrackRow};
/// Template fields that the supporting-line function reads.
use crate::template_model::TrackFields;
/// Measures keyed by display name, supplying each row's length and exact peak.
use crate::track_measure::{TrackMeasure, fields_with_measure};
/// The map type the measures arrive in.
use std::collections::HashMap;

/// What:
///      `const TRACK_PATH_SEPARATOR: char = '/';`.
///  Splits a display path into folder and file.
/// Why:
///       Display paths join segments with `/`,
///  so the split uses the same character.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TRACK_PATH_SEPARATOR = "/";
/// ```
const TRACK_PATH_SEPARATOR: char = '/';

/// What:
///      `const EXTENSION_SEPARATOR: char = '.';`.
///  Splits a file name from its extension.
/// Why:
///       Only the last dot counts,
///  and a leading dot keeps the name whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EXTENSION_SEPARATOR = ".";
/// ```
const EXTENSION_SEPARATOR: char = '.';

/// What:
///      `pub struct TrackRowModel { index_in_library, title, supporting_line, is_current }`.
///  One track row as the picker draws it.
/// Why:
///       The row carries its library index so playback can address it,
///  plus the flag that
///  marks the playing row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackRowModel = { indexInLibrary: number; title: string; supportingLine: string | null; isCurrent: boolean };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackRowModel {
    /// Position of the track in the library list.
    pub index_in_library: usize,
    /// Title with the last extension removed.
    pub title: String,
    /// Trimmed template text under the title,
    ///  or `None` when blank or not shown.
    pub supporting_line: Option<String>,
    /// Whether this row is the playing track.
    pub is_current: bool,
}

/// What:
///      `pub enum TrackListEntry { RowEntry(TrackRowModel), HeaderEntry { name, track_count } }`.
///  One entry of the flat list:
///  a track row or a subfolder header.
/// Why:
///       The picker renders both row kinds from one list,
///  in folder order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TrackListEntry = { kind: "row"; row: TrackRowModel } | { kind: "header"; name: string; trackCount: number };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrackListEntry {
    /// A playable track row.
    RowEntry(
        /// The row this entry draws.
        TrackRowModel,
    ),
    /// A subfolder header with the number of tracks directly under it.
    HeaderEntry {
        /// Subfolder path shown as the header.
        name: String,
        /// Number of tracks listed directly under this header.
        track_count: usize,
    },
}

/// What:
///      `fn file_name_of(display_path: &str) -> &str`.
///  The part after the last `/`,
///  or the whole path.
/// Why:
///       A path with no folder is just a file name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fileNameOf = (p: string) => p.substring(p.lastIndexOf("/") + 1);
/// ```
fn file_name_of(display_path: &str) -> &str {
    return match display_path.rsplit_once(TRACK_PATH_SEPARATOR) {
        Some((_, file_name)) => file_name,
        None => display_path,
    };
}

/// What:
///      `fn parent_path_of(display_path: &str) -> &str`.
///  The part before the last `/`,
///  or empty.
/// Why:
///       A root-level file has no parent,
///  so the folder field reads as empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parentPathOf = (p: string) => (p.includes("/") ? p.slice(0, p.lastIndexOf("/")) : "");
/// ```
fn parent_path_of(display_path: &str) -> &str {
    return match display_path.rsplit_once(TRACK_PATH_SEPARATOR) {
        Some((parent, _)) => parent,
        None => "",
    };
}

/// What:
///      `fn folder_name_of(display_path: &str) -> &str`.
///  The innermost folder that holds the file,
///  or empty for a root-level file.
/// Why:
///       The template's `folder` field shows the folder directly around the file,
///  not the whole path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folderNameOf = (p: string) => parentPathOf(p).split("/").pop() ?? "";
/// ```
fn folder_name_of(display_path: &str) -> &str {
    let parent = parent_path_of(display_path);
    return match parent.rsplit_once(TRACK_PATH_SEPARATOR) {
        Some((_, name)) => name,
        None => parent,
    };
}

/// What:
///      `fn extension_of(file_name: &str) -> &str`.
///  The text after the last dot,
///  or empty.
/// Why:
///       A dot at position zero starts a hidden name,
///  not an extension.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const extensionOf = (name: string) => { const i = name.lastIndexOf("."); return i <= 0 ? "" : name.slice(i + 1); };
/// ```
fn extension_of(file_name: &str) -> &str {
    return match file_name.rfind(EXTENSION_SEPARATOR) {
        Some(dot_index) if dot_index > 0 => &file_name[dot_index + 1..],
        _ => "",
    };
}

/// What:
///      `pub fn track_title_for(display_path: &str) -> String`.
///  The file name without its last extension.
/// Why:
///       Only the last extension is dropped,
///  so `01 Song.v2.flac` keeps `.v2`,
///  and a hidden name stays whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trackTitleFor = (p: string) => { const f = p.split("/").pop()!; const i = f.lastIndexOf("."); return i <= 0 ? f : f.slice(0, i); };
/// ```
pub fn track_title_for(display_path: &str) -> String {
    let file_name = file_name_of(display_path);
    return match file_name.rfind(EXTENSION_SEPARATOR) {
        Some(dot_index) if dot_index > 0 => file_name[..dot_index].to_string(),
        _ => file_name.to_string(),
    };
}

/// What:
///      `pub fn deck_folder_label(display_path: &str, page_label: &str) -> String`.
///  The folder that directly holds the track,
///  or `page_label` when the path has no folder part.
///  An empty path gives an empty label.
/// Why:
///       The deck names the track's own folder,
///  and a root-level track has no folder of its own,
///  so the label of the page it is listed under stands in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const deckFolderLabel = (p: string, pageLabel: string) => (p === "" ? "" : folderNameOf(p) || pageLabel);
/// ```
pub fn deck_folder_label(display_path: &str, page_label: &str) -> String {
    if display_path.is_empty() {
        return String::new();
    }
    let folder = folder_name_of(display_path);
    return if folder.is_empty() {
        page_label.to_string()
    } else {
        folder.to_string()
    };
}

/// What:
///      `fn track_fields_for(display_path: &str) -> TrackFields`.
///  Builds the template fields for one track.
/// Why:
///       `file` is the title,
///  as in the Kotlin.
///  `len` and `peak` are unknown in this port,
///  so they are `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fields = { title, file: title, ext, folder, path: displayPath, len: null, peak: null };
/// ```
fn track_fields_for(display_path: &str) -> TrackFields {
    let title = track_title_for(display_path);
    return TrackFields {
        file: title.clone(),
        title,
        ext: extension_of(file_name_of(display_path)).to_string(),
        folder: folder_name_of(display_path).to_string(),
        path: display_path.to_string(),
        len: None,
        peak: None,
    };
}

/// What:
///      `fn supporting_line_for(display_path, supporting_line) -> Option<String>`.
///  Runs the
///  caller's function on the track's fields,
///  then trims the text and drops it when blank.
/// Why:
///       A blank template result must never draw an empty line,
///  and the trimming happens here
///  regardless of what the template engine returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = supportingLine(fields)?.trim();
/// return text ? text : null;
/// ```
fn supporting_line_for(display_path: &str, measures: &HashMap<String, TrackMeasure>, supporting_line: &dyn Fn(&TrackFields) -> Option<String>) -> Option<String> {
    let fields = fields_with_measure(track_fields_for(display_path), measures.get(display_path));
    return supporting_line(&fields)
        .map(|text| return text.trim().to_string())
        .filter(|text| return !text.is_empty());
}

/// What:
///      `fn row_for(row, current_index, supporting_line) -> TrackRowModel`.
///  Builds one track row.
/// Why:
///       The row's current flag is true only when its library index equals the playing index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const rowFor = (row, current) => ({ indexInLibrary: row.indexInLibrary, title: trackTitleFor(row.track.displayPath), supportingLine, isCurrent: row.indexInLibrary === current });
/// ```
fn row_for(row: &FolderTrackRow, current_index: Option<usize>, measures: &HashMap<String, TrackMeasure>, supporting_line: &dyn Fn(&TrackFields) -> Option<String>) -> TrackRowModel {
    let display_path = row.track.display_path.as_str();
    return TrackRowModel {
        index_in_library: row.index_in_library,
        title: track_title_for(display_path),
        supporting_line: supporting_line_for(display_path, measures, supporting_line),
        is_current: current_index == Some(row.index_in_library),
    };
}

/// What:
///      `fn entry_for(item, current_index, supporting_line) -> TrackListEntry`.
///  Maps one folder
///  item to its row or header entry.
/// Why:
///       Track items become rows,
///  and subfolder headers pass through with their count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// item.kind === "track" ? { kind: "row", row: rowFor(item.row) } : { kind: "header", name: item.name, trackCount: item.trackCount };
/// ```
fn entry_for(item: &FolderListItem, current_index: Option<usize>, measures: &HashMap<String, TrackMeasure>, supporting_line: &dyn Fn(&TrackFields) -> Option<String>) -> TrackListEntry {
    return match item {
        FolderListItem::TrackItem(row) => TrackListEntry::RowEntry(row_for(row, current_index, measures, supporting_line)),
        FolderListItem::SubfolderHeader { name, track_count } => TrackListEntry::HeaderEntry {
            name: name.clone(),
            track_count: *track_count,
        },
    };
}

/// What:
///      `pub fn track_list_entries(folder, current_index_in_library, supporting_line) -> Vec<TrackListEntry>`.
///  Maps a folder's flat list to the picker entries,
///  in the same order.
/// Why:
///       The supporting line is a parameter so the template engine stays outside this module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trackListEntries(folder: FolderEntry, current: number | null, supportingLine: (f: TrackFields) => string | null): TrackListEntry[] {
///   return folder.items.map((item) => entryFor(item, current, supportingLine));
/// }
/// ```
pub fn track_list_entries(folder: &FolderEntry, current_index_in_library: Option<usize>, measures: &HashMap<String, TrackMeasure>, supporting_line: &dyn Fn(&TrackFields) -> Option<String>) -> Vec<TrackListEntry> {
    return folder
        .items
        .iter()
        .map(|item| return entry_for(item, current_index_in_library, measures, supporting_line))
        .collect();
}

/// What:
///      `#[cfg(test)] #[path = "track_entries_tests.rs"] mod tests;` declares a test-only submodule
///  whose code lives in the sibling file `track_entries_tests.rs`.
/// Why:
///       Keep `track_entries.rs` to production code,
///  with the tests beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // trackEntries.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "track_entries_tests.rs"]
mod tests;
