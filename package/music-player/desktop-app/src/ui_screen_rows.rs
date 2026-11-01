//! Pure conversions from the shared player-content model into the flat track list and the folder
//! picker lists that the window shows.
//!
//! What: this module turns `TrackListEntry` values and rail sections into plain Rust structs.
//! Why: no Slint type appears here, so the row order, the current row and the picker rules can be
//! tested without a window.

use music_player::folder_rail::RailSection;
use music_player::title_prefix::{display_title, shared_prefix_len};
use music_player::track_entries::TrackListEntry;

/// What: `ListRow` is one plain row of the track list (kind 0 a track, kind 1 a subfolder header).
/// Why: the window copies these into its `TrackListRow` struct, so the mapping stays testable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListRow = { kind: 0 | 1; title: string; supporting: string; current: boolean; index: number };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListRow {
    /// 0 for a playable track, 1 for a subfolder header.
    pub kind: i32,
    /// File name without extension for a track, or the subfolder name for a header.
    pub title: String,
    /// Second line of a track, or the count text for a header; empty when there is none.
    pub supporting: String,
    /// Whether this track is the playing track. Always false for a header.
    pub current: bool,
    /// Library index for a track, or -1 for a header.
    pub index: i32,
}

/// What: `PickerCell` is one letter or script cell of the folder picker rail.
/// Why: the rail key doubles as the heading text in the picker pane, so it is the cell label.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PickerCell = { key: string; label: string; sectionStart: boolean };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerCell {
    /// Key the pane reports back and scrolls to; equal to `label`.
    pub key: String,
    /// Text drawn in the rail.
    pub label: String,
    /// True for the first cell of a writing system other than the first one, which the rail separates with a hairline (D28).
    pub section_start: bool,
}

/// What: `PickerName` is one row of the rail-ordered name list (a heading row or a folder name).
/// Why: the pane draws heading rows full width and name rows as wrapped cells.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PickerName = { name: string; sectionStart: boolean; heading: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PickerName {
    /// Folder label for a name row, or empty for a heading row.
    pub name: String,
    /// True for a heading row that starts a rail cell.
    pub section_start: bool,
    /// Key of the rail cell this row belongs to.
    pub heading: String,
}

/// What: `Picker` bundles the rail cells, the name rows and the highlighted cell.
/// Why: one value feeds both picker lists so they cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Picker = { cells: PickerCell[]; names: PickerName[]; currentCell: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picker {
    /// Rail cells in rail order.
    pub cells: Vec<PickerCell>,
    /// Heading rows and name rows in display order.
    pub names: Vec<PickerName>,
    /// Key of the cell that holds the current folder, or empty when none does.
    pub current_cell: String,
}

/// What: `list_rows` maps every entry of the flat track list into a `ListRow`.
/// Why: the window shows one flat list, so headers and tracks share one row type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listRows = (entries: TrackListEntry[]): ListRow[] => entries.map(listRow);
/// ```
pub fn list_rows(entries: &[TrackListEntry]) -> Vec<ListRow> {
    return entries.iter().map(list_row).collect();
}

/// What: `list_row` converts one entry, reading a header's track count into its count text.
/// Why: a header's second line is its count, and a track's second line is its supporting line.
fn list_row(entry: &TrackListEntry) -> ListRow {
    return match entry {
        TrackListEntry::RowEntry(row) => ListRow {
            kind: 0,
            title: row.title.clone(),
            supporting: row.supporting_line.clone().unwrap_or_default(),
            current: row.is_current,
            index: i32::try_from(row.index_in_library).unwrap_or(-1),
        },
        TrackListEntry::HeaderEntry { name, track_count } => ListRow {
            kind: 1,
            title: name.clone(),
            supporting: track_count_text(*track_count),
            current: false,
            index: -1,
        },
    };
}

/// What: `UNPLAYABLE_ROW_TEXT` is the second line of a row whose file could not be opened for playback.
/// Why: PB1 asks for a mark on the row; the second line replaces the length and loudness, which are not known for a file
/// that cannot be read, and states the problem in words.
pub const UNPLAYABLE_ROW_TEXT: &str = "Can't be played";

/// What: `mark_unplayable` gives every row whose file failed to open the second line `UNPLAYABLE_ROW_TEXT`.
/// Why: the engine reports the failed paths once, and the rows are rebuilt on every projection, so the mark is applied
/// here from the remembered set.
/// In TS you'd write (pseudocode):
/// ```ts
/// rows.forEach((row) => { if (row.kind === 0 && unplayable.has(paths[row.index])) row.supporting = "Can't be played"; });
/// ```
pub fn mark_unplayable(rows: &mut [ListRow], paths: &[std::path::PathBuf], unplayable: &std::collections::HashSet<std::path::PathBuf>) {
    if unplayable.is_empty() {
        return;
    }
    for row in rows.iter_mut().filter(|row| return row.kind == 0) {
        let path = usize::try_from(row.index).ok().and_then(|index| return paths.get(index));
        if path.is_some_and(|known| return unplayable.contains(known)) {
            row.supporting = UNPLAYABLE_ROW_TEXT.to_string();
        }
    }
}

/// What: `folder_name_of_header` is the name of the directory a header row stands for: its last path component.
/// Why: a header shows the subfolder path (`Album/Disc 1`), and only the directory's own name can repeat in a title.
fn folder_name_of_header(header: &str) -> &str {
    return header.rsplit_once('/').map_or(header, |(_parent, name)| return name);
}

/// What: `strip_shared_prefixes` removes, from the titles of each run of track rows, the text every title of the run
/// starts with (D86), or the directory's own name for a run of one.
/// Why: a run of rows between two headers is the tracks of one directory; the first run is the page's own directory,
/// named `page_label`. The polish review finding LS3: rows repeated the artist (`かめりあ(Camellia) - Aegis`).
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const run of runsOfTrackRows(rows)) { const n = sharedPrefixLen(run.map((r) => r.title)); run.forEach((r) => r.title = displayTitle(r.title, n, run.folder)); }
/// ```
pub fn strip_shared_prefixes(rows: &mut [ListRow], page_label: &str) {
    let mut folder: String = page_label.to_string();
    let mut start: usize = 0;
    for position in 0..=rows.len() {
        let is_boundary: bool = position == rows.len() || rows[position].kind != 0;
        if !is_boundary {
            continue;
        }
        strip_run(&mut rows[start..position], &folder);
        if position < rows.len() {
            folder = folder_name_of_header(&rows[position].title).to_string();
        }
        start = position + 1;
    }
}

/// What: `strip_run` strips one run of track rows of one directory.
/// Why: the shared prefix is found once for the run and applied to each title.
fn strip_run(run: &mut [ListRow], folder: &str) {
    if run.is_empty() {
        return;
    }
    let titles: Vec<String> = run.iter().map(|row| return row.title.clone()).collect();
    let borrowed: Vec<&str> = titles.iter().map(String::as_str).collect();
    let prefix: usize = shared_prefix_len(&borrowed);
    for (row, title) in run.iter_mut().zip(titles.iter()) {
        row.title = display_title(title, prefix, folder).to_string();
    }
}

/// What: `track_count_text` gives "1 track" or "N tracks".
/// Why: headers show a count in the same words the Android list uses.
pub fn track_count_text(count: usize) -> String {
    let noun = if count == 1 { "track" } else { "tracks" };
    return format!("{count} {noun}");
}

/// What: `current_row_position` gives the position of the playing row in the flat list, or -1.
/// Why: the list scrolls the playing row into view, and -1 means there is nothing to scroll to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const currentRowPosition = (rows: ListRow[]) => rows.findIndex((row) => row.current);
/// ```
pub fn current_row_position(rows: &[ListRow]) -> i32 {
    return rows
        .iter()
        .position(|row| return row.current)
        .map_or(-1, |position| return i32::try_from(position).unwrap_or(-1));
}

/// What: `picker` builds the rail cells and the rail-ordered name rows from the rail sections.
/// Why: the folder picker shows a heading row per cell followed by that cell's folder names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const picker = (sections: RailSection[], currentFolder: string): Picker => { ... };
/// ```
pub fn picker(sections: &[RailSection], current_folder: &str) -> Picker {
    let mut cells: Vec<PickerCell> = Vec::new();
    let mut names: Vec<PickerName> = Vec::new();
    let mut current_cell: String = String::new();
    let cells_with_section = sections
        .iter()
        .flat_map(|section| return section.cells.iter().enumerate().map(|(position, cell)| return (position == 0, cell)));
    for (first_of_section, cell) in cells_with_section {
        cells.push(PickerCell {
            key: cell.label.clone(),
            label: cell.label.clone(),
            // The first cell overall has nothing above it to separate from.
            section_start: first_of_section && !cells.is_empty(),
        });
        names.push(PickerName {
            name: String::new(),
            section_start: true,
            heading: cell.label.clone(),
        });
        if current_cell.is_empty() && cell.names.iter().any(|name| return name == current_folder) {
            current_cell = cell.label.clone();
        }
        for name in &cell.names {
            names.push(PickerName {
                name: name.clone(),
                section_start: false,
                heading: cell.label.clone(),
            });
        }
    }
    return Picker {
        cells,
        names,
        current_cell,
    };
}

#[cfg(test)]
#[path = "ui_screen_rows_tests.rs"]
mod tests;
