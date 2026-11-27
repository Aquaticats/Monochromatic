//! The work a second build redoes:
//!  a slide shaping only the new lines,
//!  an edit shaping only the lines it changes,
//!  new syntax classes shaping only the lines they reach,
//!  a line the budget cannot pay for drawn blank,
//!  a cancelled build answering nothing,
//!  and a long text shaping only the lines near its window.

use crate::fixtures::{
    Always,
    DARK,
    FILLS,
    Fixture,
    INKS,
    Never,
    code,
    fixture,
    fixture_of,
    items_of,
    size_of,
    whole,
};
use ide_app::{
    document::Document,
    minimap::{
        BAND,
        BUILD_WORK,
        CHUNK_ROWS,
        Input,
        LineCache,
        Source,
        band_count,
        build,
    },
    row_map::RowMap,
    shaped_text::TextShaper,
    source_style::StyleSpan,
    tile_scene::read::drawn_runs,
};

/// A second build of the same picture shapes nothing,
/// and one more window over a text shapes only the lines it brings in.
#[test]
fn a_second_build_and_a_slide_shape_only_the_new_lines() {
    let fixture = fixture(600, 730.0, 1.0);
    let mut shaper = TextShaper::new();
    let mut cache = LineCache::default();
    let (_, _, first) = items_of(&fixture, 2..6, &mut shaper, &mut cache);
    assert_eq!(first.shaped, first.lines);
    let (_, _, again) = items_of(&fixture, 2..6, &mut shaper, &mut cache);
    assert_eq!((again.shaped, again.reused), (0, again.lines));
    let (_, _, slid) = items_of(&fixture, 3..7, &mut shaper, &mut cache);
    assert!(slid.shaped > 0, "a band came in");
    assert!(
        slid.shaped * 2 < slid.lines,
        "{} of {} lines were shaped for a slide of one band in four",
        slid.shaped,
        slid.lines
    );
}

/// The text of `fixture` with `edit` applied to one line.
fn edited(
    fixture: &Fixture,
    line: usize,
    replace: &str,
    with: &str,
) -> Fixture {
    let text = fixture.document.text().to_string();
    let changed = text.replacen(replace, with, 1);
    assert_ne!(changed, text);
    let _ = line;
    return fixture_of(&changed, fixture.wrap.expect("wrap"), fixture.scale);
}

/// An edit of one line,
/// and an inserted line that moves every line after it,
/// shape one line each:
/// a line is known by what shaped it,
/// not by its number.
#[test]
fn an_edit_shapes_only_the_lines_it_changes() {
    let before = fixture(600, 730.0, 1.0);
    let mut shaper = TextShaper::new();
    let mut cache = LineCache::default();
    let (_, _, cold) = items_of(
        &before,
        0..band_count(before.map.height(), BAND),
        &mut shaper,
        &mut cache,
    );
    assert_eq!(cold.shaped, cold.lines);
    // One word of line 20 changes to another of the same length.
    let same_length = edited(&before, 20, "value_20 =", "valoe_20 =");
    let window = 0..band_count(same_length.map.height(), BAND);
    let (_, _, one) = items_of(&same_length, window.clone(), &mut shaper, &mut cache);
    assert_eq!(one.shaped, 1, "one line was edited");
    assert_eq!(one.reused, one.lines - 1);
    // A line is inserted before line 20: all the lines after it move to another number.
    let inserted = edited(
        &before,
        20,
        "let value_20 =",
        "let inserted = 1;\nlet value_20 =",
    );
    let after_insert = 0..band_count(inserted.map.height(), BAND);
    let (_, _, moved) = items_of(&inserted, after_insert, &mut shaper, &mut cache);
    assert!(
        moved.shaped <= 2,
        "an inserted line shaped {} lines of {}",
        moved.shaped,
        moved.lines
    );
    // Positive control: another wrap width and another scale shape every line again.
    let narrower = fixture_of(&before.document.text().to_string(), 500.0, 1.0);
    let narrow_window = 0..band_count(narrower.map.height(), BAND);
    let (_, _, wider) = items_of(&narrower, narrow_window, &mut shaper, &mut cache);
    assert_eq!(
        wider.shaped, wider.lines,
        "another wrap width shapes every line again"
    );
    let sharper = fixture_of(&before.document.text().to_string(), 500.0, 2.0);
    let sharp_window = 0..band_count(sharper.map.height(), BAND);
    let (_, _, scaled) = items_of(&sharper, sharp_window, &mut shaper, &mut cache);
    assert_eq!(
        scaled.shaped, scaled.lines,
        "another scale shapes every line again"
    );
}

/// New syntax classes shape only the lines they reach.
#[test]
fn new_syntax_classes_shape_only_the_lines_they_reach() {
    let mut fixture = fixture(300, 730.0, 1.0);
    let mut shaper = TextShaper::new();
    let mut cache = LineCache::default();
    let window = fixture.all();
    items_of(&fixture, window.clone(), &mut shaper, &mut cache);
    let line = fixture.document.text().line_to_char(30);
    let at = fixture
        .styles
        .iter()
        .position(|span| return span.start >= line + 1)
        .expect("a span after the start of line 30");
    fixture.styles[at].style = 7;
    let (_, _, stats) = items_of(&fixture, window, &mut shaper, &mut cache);
    assert_eq!(stats.shaped, 1, "one class changed on one line");
}

/// A line that no budget can pay for is drawn blank and counted,
/// and the lines around it are still drawn.
#[test]
fn a_line_the_budget_cannot_pay_for_is_drawn_blank() {
    let mut text = code(30);
    text.push('\n');
    text.push_str(&"x".repeat(70_000));
    text.push('\n');
    text.push_str(&code(30));
    let fixture = fixture_of(&text, 730.0, 1.0);
    let (items, _, stats) = whole(&fixture);
    assert_eq!(stats.skipped, 1, "the long line was left blank");
    assert!(stats.work <= BUILD_WORK);
    assert!(
        drawn_runs(&items).len() >= 50,
        "the lines around it are drawn"
    );
}

/// A cancelled build answers nothing.
#[test]
fn a_cancelled_build_answers_nothing() {
    let fixture = fixture(100, 730.0, 1.0);
    let input = fixture.input(fixture.all(), 660.0, 110.0);
    let mut shaper = TextShaper::new();
    let mut cache = LineCache::default();
    assert!(build(&mut shaper, &mut cache, &input, &Always).is_none());
    assert!(build(&mut shaper, &mut cache, &input, &Never).is_some());
}

/// A long text builds only the lines near its window:
/// the cost follows the window,
/// not the length of the text.
#[test]
fn a_long_text_shapes_only_the_lines_near_its_window() {
    let mut text = String::new();
    for number in 0..50_000 {
        text.push_str(&format!(
            "fn item_{number}(value: u32) -> u32 {{ value + {number} }} // trailing words\n"
        ));
    }
    let document = Document::new(&text);
    let map = RowMap::plain(document.text().len_lines());
    let styles: Vec<StyleSpan> = Vec::new();
    let source = Source {
        document: &document,
        styles: &styles,
        map: &map,
        colors: DARK,
        inks: INKS,
        fills: FILLS,
        blocks: &[],
        marks: &[],
        ranges: &[],
        active: None,
        wrap: Some(730.0),
        scale: 1.0,
    };
    let input = Input {
        source,
        view: 660.0,
        width: 110.0,
        window: 800..808,
    };
    let built = build(
        &mut TextShaper::new(),
        &mut LineCache::default(),
        &input,
        &Never,
    )
    .expect("a picture");
    assert_eq!(built.stats.units, 8);
    assert!(
        built.stats.lines <= 8 * 32 + 4,
        "{} lines for 8 bands",
        built.stats.lines
    );
    assert_eq!(built.stats.shaped, built.stats.lines);
    assert_eq!(size_of(&built), (110, 8 * 128));
    assert_eq!(
        built.chunks.len(),
        1024_usize.div_ceil(CHUNK_ROWS as usize),
        "a slice of 1,024 rows is cut into chunks of the chunk size"
    );
    assert_eq!(built.top, 800 * 128);
}
