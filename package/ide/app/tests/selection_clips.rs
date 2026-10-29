//! The clips that split a selected row's ink,
//!  as paint items:
//!  each is as tall as its own row and not as tall as the tile,
//!  and a part of a row beyond the ink of a run gets no copy of it.
//!
//! The renderer keeps a buffer of fixed size for every tile a clip covers and another for the lines of every copy of a
//!  glyph,
//!  and it draws nothing when either is full,
//!  so both costs are checked on the items,
//!  with no GPU.
//! `native/tall_selection_tests.rs` shows the pixels.
//! No window is opened.

/// The palette of the tile:
/// black text and red selected text.
use ide_app::code_colors::CodeColors;
/// The document under test and the position that selects a range.
use ide_app::document::{Document, ReadingAffinity, ReadingPosition};
/// The row height of the tile.
use ide_app::row_map::{CODE_ROW, RowMap};
/// The shaper and the viewport that lay the rows out.
use ide_app::shaped_text::{TextShaper, Viewport};
/// What a tile draws,
/// read as data.
use ide_app::tile_scene::{
    Item,
    read::{DrawnRun, drawn_runs},
    tile_items,
};

/// A palette whose two inks differ,
/// so every selected run is clipped.
const COLORS: CodeColors = CodeColors {
    foreground: [0, 0, 0, 255],
    selected: [255, 0, 0, 255],
    dark: false,
};

/// `count` lines of dense code that do not wrap at the width used here.
fn code(count: usize) -> String {
    let mut text = String::new();
    for number in 0..count {
        if number > 0 {
            text.push('\n');
        }
        text.push_str(&format!(
            "pub fn compute_{number}(first_item: u32, second_item: &str) -> Result<u32, String> {{ // {number}"
        ));
    }
    return text;
}

/// The items of a tile of `count` lines with the first `selected` lines selected,
/// and the tile's height.
fn items_of(count: usize, selected: usize) -> (Vec<Item>, f32) {
    let mut document = Document::new(&code(count));
    let head = if selected >= count {
        document.text().len_chars()
    } else {
        document.text().line_to_char(selected)
    };
    document.select(ReadingPosition {
        anchor: 0,
        head,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(count);
    let view = shaper.prepare_rows(
        &document,
        Viewport {
            first: 0,
            count,
            width: 1113.0,
            scale: 1.0,
            wrap: Some(1089.0),
            clip: None,
        },
        &[],
        &plain,
    );
    let height = view.height as f32;
    return (
        tile_items(&view, COLORS, 0.0).expect("the view has items"),
        height,
    );
}

/// Every clip of a selected tile is about as tall as the row it splits and nowhere near as tall as the tile.
#[test]
fn a_selected_rows_clips_are_as_tall_as_the_row_and_not_the_tile() {
    let (items, height) = items_of(40, 40);
    let mut clips = 0;
    for item in &items {
        if let Item::Clip { top, bottom, .. } = item {
            clips += 1;
            let tall = bottom - top;
            assert!(
                (CODE_ROW..=3.0 * CODE_ROW + 2.0).contains(&tall),
                "a clip {tall} px tall in a tile {height} px tall"
            );
        }
    }
    assert!(clips >= 40, "every selected row has a clip, found {clips}");
    // The clips of a row hold the row's own glyphs: every run lies between its clip's top and bottom.
    for run in drawn_runs(&items) {
        let (top, bottom) = run.band.expect("a selected run is clipped");
        assert!(run.baseline > top && run.baseline < bottom);
    }
}

/// The runs drawn in each ink on each row:
/// the selected copies and the ordinary ones.
fn copies(runs: &[DrawnRun], row: usize) -> (usize, usize) {
    let mut selected = 0;
    let mut ordinary = 0;
    for run in runs {
        let index = (run.baseline / CODE_ROW) as usize;
        if index != row {
            continue;
        }
        if run.ink == COLORS.selected {
            selected += 1;
        } else {
            ordinary += 1;
        }
    }
    return (selected, ordinary);
}

/// A row selected from its first glyph to its last is drawn once in the selected ink and not again in the ordinary
/// ink on either side,
/// so the renderer flattens its glyphs once and not three times;
/// only the last line,
/// which ends where the selection ends,
/// keeps a second copy for the ink that may overhang its last glyph.
#[test]
fn a_row_selected_end_to_end_is_drawn_once() {
    let (items, _) = items_of(40, 40);
    let runs = drawn_runs(&items);
    for row in 0..39 {
        assert_eq!(copies(&runs, row), (1, 0), "row {row}");
    }
    let (selected, ordinary) = copies(&runs, 39);
    assert_eq!(selected, 1);
    assert!(ordinary <= 1, "the last row has {ordinary} ordinary copies");
}

/// A row selected only in its middle keeps a copy in the ordinary ink on each side of the selection,
/// as the glyphs there need.
#[test]
fn a_row_selected_in_its_middle_is_drawn_on_both_sides_and_inside() {
    let text = code(3);
    let start = text
        .find("first_item: u32, second_item")
        .expect("the text of line 0");
    let mut document = Document::new(&text);
    document.select(ReadingPosition {
        anchor: start,
        head: start + 20,
        viewport: 0,
        affinity: ReadingAffinity::Downstream,
    });
    let mut shaper = TextShaper::new();
    let plain = RowMap::plain(3);
    let view = shaper.prepare_rows(
        &document,
        Viewport {
            first: 0,
            count: 3,
            width: 1113.0,
            scale: 1.0,
            wrap: Some(1089.0),
            clip: None,
        },
        &[],
        &plain,
    );
    let items = tile_items(&view, COLORS, 0.0).expect("the view has items");
    let runs = drawn_runs(&items);
    assert_eq!(copies(&runs, 0), (1, 2));
    // The rows without a selection are drawn once, unclipped, in the ordinary ink.
    assert_eq!(copies(&runs, 1), (0, 1));
    assert_eq!(copies(&runs, 2), (0, 1));
    assert!(runs.iter().filter(|run| return run.clip.is_none()).count() >= 2);
}
