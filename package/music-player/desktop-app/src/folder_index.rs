//! Folder index for the folder picker and the In order playback walk.
//!
//! Ported from `core/FolderIndex.kt`.
//!  It groups the library by top-level folder,
//!  then lays out
//! each folder as one flat list:
//!  the folder's own tracks first,
//!  then each subfolder as a header
//! row followed by its tracks.
//!  Playback order follows the same list.

/// Ordering result used when sorting folder and track names.
use std::cmp::Ordering;
/// Sorted map that groups tracks by folder path.
use std::collections::BTreeMap;

/// Joins a folder prefix and a name into one display path.
use crate::display_path::join_display_path;
/// Sort key that orders names the way the folder rail collates them.
use crate::folder_rail::collation_key;
/// One track with its URI and display path.
use crate::track::Track;

/// What:
///      `pub const UNFILED_FOLDER_NAME: &str = "Unfiled";`.
///  The entry for tracks at the library root.
/// Why:
///       Root-level tracks have no folder,
///  so they need a visible name that sorts last in the picker.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const UNFILED_FOLDER_NAME = "Unfiled";
/// ```
pub const UNFILED_FOLDER_NAME: &str = "Unfiled";

/// What:
///      `const PATH_SEPARATOR: char = '/';`.
///  The separator between display path segments.
/// Why:
///       The split must cut on the same character that the path builders join with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PATH_SEPARATOR = "/";
/// ```
const PATH_SEPARATOR: char = '/';

/// What:
///      `const CURRENT_DIRECTORY: &str = "."`,
///  the segment that names the current directory.
/// Why:
///       Dropping it makes `A/./x.flac` group exactly like `A/x.flac`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CURRENT_DIRECTORY = ".";
/// ```
const CURRENT_DIRECTORY: &str = ".";

/// What:
///      `const PARENT_DIRECTORY: &str = ".."`,
///  the segment that names the parent directory.
/// Why:
///       Dropping it keeps a folder from appearing that the display path does not show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PARENT_DIRECTORY = "..";
/// ```
const PARENT_DIRECTORY: &str = "..";

/// What:
///      `const MIN_FOLDERED_SEGMENTS: usize = 2`.
///  The fewest segments a track needs to sit in a folder:
///  the folder name and the file name.
/// Why:
///       A single-segment path names a file at the root,
///  so it goes to the Unfiled entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MIN_FOLDERED_SEGMENTS = 2;
/// ```
const MIN_FOLDERED_SEGMENTS: usize = 2;

/// What:
///      `pub struct FolderTrackRow { track, index_in_library }`.
///  One track together with its
///  position in the library list it came from.
/// Why:
///       Playback and selection address tracks by library index,
///  so the index travels with the track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderTrackRow = { track: Track; indexInLibrary: number };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderTrackRow {
    /// Track as it appears in the library.
    pub track: Track,
    /// Position of the track in the library list the index was built from.
    pub index_in_library: usize,
}

/// What:
///      `pub enum FolderListItem`.
///  One row of a folder's flat list:
///  a track row or a subfolder header.
/// Why:
///       A sum type lets the picker render both row kinds exhaustively.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderListItem = { kind: "track"; row: FolderTrackRow } | { kind: "header"; name: string; trackCount: number };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FolderListItem {
    /// A playable track row inside a folder's list.
    TrackItem(
        /// The track row this item draws.
        FolderTrackRow,
    ),
    /// A header row that names a subfolder path and counts the tracks that follow it.
    SubfolderHeader {
        /// Subfolder path between the top-level folder and the tracks,
        ///  such as `Album/Disc 1`.
        name: String,
        /// Number of tracks listed directly under this header.
        track_count: usize,
    },
}

/// What:
///      `pub struct FolderEntry { name, items, track_count }`.
///  One top-level folder,
///  its flat
///  list of rows,
///  and the total number of tracks under it.
/// Why:
///       The picker shows each folder's name and count,
///  and playback walks `items` in order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderEntry = { name: string; items: FolderListItem[]; trackCount: number };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderEntry {
    /// Top-level folder name,
    ///  or `UNFILED_FOLDER_NAME` for root-level tracks.
    pub name: String,
    /// Flat list of track rows and subfolder headers in playback order.
    pub items: Vec<FolderListItem>,
    /// Every track under this folder,
    ///  including tracks in deeper subfolders.
    pub track_count: usize,
}

/// What:
///      `struct PlacedRow { subfolder, row }`.
///  One track inside a top-level folder,
///  with the
///  subfolder path between that folder and the file.
///  An empty subfolder means the track sits
///  directly in the folder.
/// Why:
///       Placement is computed once per track,
///  so grouping and sorting work on plain values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlacedRow = { subfolder: string; row: FolderTrackRow };
/// ```
struct PlacedRow {
    /// Subfolder path between the top-level folder and the file,
    ///  empty when directly inside.
    subfolder: String,
    /// Track and its library position.
    row: FolderTrackRow,
}

/// What:
///      `fn split_segments(display_path: &str) -> Vec<&str>`.
///  Cuts a display path into its
///  non-empty segments,
///  dropping `.` and `..`.
/// Why:
///       RelPath keeps its splitter private,
///  so folder grouping repeats the same rule here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const splitSegments = (p: string) => p.split("/").filter((s) => s !== "" && s !== "." && s !== "..");
/// ```
fn split_segments(display_path: &str) -> Vec<&str> {
    return display_path
        .split(PATH_SEPARATOR)
        .filter(|segment| return !segment.is_empty() && *segment != CURRENT_DIRECTORY && *segment != PARENT_DIRECTORY)
        .collect();
}

/// What:
///      `fn subfolder_path(segments: &[&str]) -> String`.
///  Joins the segments between the top-level
///  folder and the file name,
///  through `join_display_path` so each name is sanitized the same way.
///  Callers pass at least two segments.
/// Why:
///       Deeper nesting stays flat:
///  the header carries the whole path below the folder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const subfolder = segments.slice(1, -1).reduce((prefix, s) => joinDisplayPath(prefix, s), "");
/// ```
fn subfolder_path(segments: &[&str]) -> String {
    let middle = &segments[1..segments.len() - 1];
    return middle
        .iter()
        .fold(String::new(), |prefix, segment| return join_display_path(&prefix, segment));
}

/// What:
///      `fn folder_name_order(left: &str, right: &str) -> Ordering`.
///  Orders names by collation key,
///  then by code point when the keys tie.
/// Why:
///       `Camellia` and `camellia` tie on the key,
///  and the code point tie-break keeps the picker
///  order stable across runs.
///  Rust `str` ordering is byte order,
///  which is code point order for UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folderNameOrder = (a: string, b: string) => collator.compare(a, b) || (a < b ? -1 : a > b ? 1 : 0);
/// ```
fn folder_name_order(left: &str, right: &str) -> Ordering {
    return collation_key(left)
        .cmp(&collation_key(right))
        .then_with(|| return left.cmp(right));
}

/// What:
///      `fn folder_entry(name: String, placed: Vec<PlacedRow>) -> FolderEntry`.
///  Builds one top-level
///  folder's flat list:
///  its own tracks first,
///  then each subfolder as a header and its tracks.
/// Why:
///       Own tracks come first,
///  then subfolders in name order.
///  Rows keep library order inside each group.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // direct tracks, then for each sorted subfolder: header followed by its tracks
/// ```
fn folder_entry(name: String, placed: Vec<PlacedRow>) -> FolderEntry {
    let track_count = placed.len();
    let mut items: Vec<FolderListItem> = Vec::new();
    let mut by_subfolder: BTreeMap<String, Vec<FolderTrackRow>> = BTreeMap::new();
    for placed_row in placed {
        if placed_row.subfolder.is_empty() {
            items.push(FolderListItem::TrackItem(placed_row.row));
        } else {
            by_subfolder
                .entry(placed_row.subfolder)
                .or_default()
                .push(placed_row.row);
        }
    }
    let mut subfolders: Vec<(String, Vec<FolderTrackRow>)> = by_subfolder.into_iter().collect();
    subfolders.sort_by(|left, right| return folder_name_order(&left.0, &right.0));
    for (subfolder, rows) in subfolders {
        items.push(FolderListItem::SubfolderHeader {
            name: subfolder,
            track_count: rows.len(),
        });
        items.extend(rows.into_iter().map(FolderListItem::TrackItem));
    }
    return FolderEntry {
        name,
        items,
        track_count,
    };
}

/// What:
///      `fn unfiled_entry(rows: Vec<FolderTrackRow>) -> FolderEntry`.
///  The Unfiled entry for root tracks.
/// Why:
///       Root-level tracks form one entry in library order,
///  listed after the named folders.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { name: "Unfiled", items: rows.map((row) => ({ kind: "track", row })), trackCount: rows.length };
/// ```
fn unfiled_entry(rows: Vec<FolderTrackRow>) -> FolderEntry {
    let track_count = rows.len();
    return FolderEntry {
        name: UNFILED_FOLDER_NAME.to_string(),
        items: rows.into_iter().map(FolderListItem::TrackItem).collect(),
        track_count,
    };
}

/// What:
///      `pub fn folder_index(tracks: &[Track]) -> Vec<FolderEntry>`.
///  Groups the library by top-level
///  folder and returns the folder entries in name order.
///  Root-level tracks form the Unfiled entry,
///  listed last and only when such tracks exist.
///  An empty library gives an empty list.
/// Why:
///       The folder picker and In order playback both need the same flat list,
///  so it is built once here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function folderIndex(tracks: Track[]): FolderEntry[] { /* group by first segment, sort, flatten */ }
/// ```
pub fn folder_index(tracks: &[Track]) -> Vec<FolderEntry> {
    let mut root_rows: Vec<FolderTrackRow> = Vec::new();
    let mut by_folder: BTreeMap<String, Vec<PlacedRow>> = BTreeMap::new();
    for (index_in_library, track) in tracks.iter().enumerate() {
        let row = FolderTrackRow {
            track: track.clone(),
            index_in_library,
        };
        let segments = split_segments(&track.display_path);
        if segments.len() < MIN_FOLDERED_SEGMENTS {
            root_rows.push(row);
            continue;
        }
        let folder = segments[0].to_string();
        let subfolder = subfolder_path(&segments);
        by_folder
            .entry(folder)
            .or_default()
            .push(PlacedRow { subfolder, row });
    }
    let mut named: Vec<(String, Vec<PlacedRow>)> = by_folder.into_iter().collect();
    named.sort_by(|left, right| return folder_name_order(&left.0, &right.0));
    let mut entries: Vec<FolderEntry> = named
        .into_iter()
        .map(|(name, placed)| return folder_entry(name, placed))
        .collect();
    if !root_rows.is_empty() {
        entries.push(unfiled_entry(root_rows));
    }
    return entries;
}

/// What:
///      `#[cfg(test)] #[path = "folder_index_tests.rs"] mod tests;` declares a test-only submodule
///  whose code lives in the sibling file `folder_index_tests.rs`.
/// Why:
///       Keep `folder_index.rs` to production code,
///  with the tests beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // folderIndex.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "folder_index_tests.rs"]
mod tests;
