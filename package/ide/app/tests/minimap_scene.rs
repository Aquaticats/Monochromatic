//! The minimap's picture without a window:
//!  the bands and the slice of rows they make up,
//!  the paint items of a slice (glyphs in the palette's inks and the marks under them),
//!  the row cache and what an edit shapes again,
//!  the work budget,
//!  a build that is cancelled,
//!  and the size of the texture a slice is drawn into.
//! Nothing here fills a pixel:
//! the items are read as data,
//! and the native tests show that the data lands on the GPU's texture.

/// Diagnostics are marks of a range with a severity.
use ide_app::annotation::Mark;
/// The inks of hints and diagnostics.
use ide_app::annotation_layout::AnnotationColors;
/// The palette whose inks the glyphs are drawn in.
use ide_app::code_colors::{CodeColors, ink};
/// The text and its reading position.
use ide_app::document::{Document, ReadingAffinity, ReadingPosition};
/// The matches of the in-file find.
use ide_app::find::FindRange;
/// Severity of a diagnostic.
use ide_app::language::diagnostics::Severity;
/// The lines' visual rows.
use ide_app::line_rows::LineRows;
/// The picture,
/// its plan,
/// and its parts.
use ide_app::minimap::{
    BAND, BUILD_WORK, CHUNK_ROWS, Cancel, Fills, Geometry, Input, LineCache, Source, Stats,
    band_count, build, chunk_rows, cut, most_bands, picture_rows, slice_items, slice_rows,
    texture_rows, texture_runs,
};
/// The vertical mapping.
use ide_app::row_map::RowMap;
/// Shaping.
use ide_app::shaped_text::TextShaper;
/// Syntax classes.
use ide_app::source_style::StyleSpan;
/// What the items draw.
use ide_app::tile_scene::{
    Item,
    read::{drawn_fills, drawn_runs, drawn_strokes},
};
/// A window of bands is a range of their numbers.
use std::ops::Range;

/// The dark scheme's palette of the tests.
const DARK: CodeColors = CodeColors {
    foreground: [220, 220, 220, 255],
    selected: [255, 255, 255, 255],
    dark: true,
};

/// The annotation inks of the tests.
const INKS: AnnotationColors = AnnotationColors {
    hint: [150, 160, 170, 255],
    error: [255, 110, 120, 255],
    warning: [240, 200, 40, 255],
    information: [90, 200, 250, 255],
    suggestion: [170, 170, 170, 255],
};

/// The fill inks of the tests:
/// a selection no glyph has the color of.
const FILLS: Fills = Fills {
    selection: [30, 90, 200, 255],
    foreground: [220, 220, 220, 255],
    boundary: false,
};

/// A build that is never cancelled.
struct Never;

impl Cancel for Never {
    /// Never cancelled.
    fn is_cancelled(&self) -> bool {
        return false;
    }
}

/// A build that is cancelled before it starts.
struct Always;

impl Cancel for Always {
    /// Always cancelled.
    fn is_cancelled(&self) -> bool {
        return true;
    }
}

/// Everything a source borrows,
/// owned.
struct Fixture {
    /// The text and its selection.
    document: Document,
    /// Syntax classes.
    styles: Vec<StyleSpan>,
    /// The vertical mapping,
    /// with the rows the wrapped lines really have.
    map: RowMap,
    /// Diagnostic marks.
    marks: Vec<Mark>,
    /// Find matches.
    ranges: Vec<FindRange>,
    /// The match the selection covers.
    active: Option<FindRange>,
    /// The inks of the fills.
    fills: Fills,
    /// The width lines are broken at.
    wrap: Option<f32>,
    /// Physical pixels per logical pixel.
    scale: f32,
}

/// A text of ordinary code with some lines long enough to wrap.
fn code(lines: usize) -> String {
    let mut text = String::new();
    for number in 0..lines {
        if number > 0 {
            text.push('\n');
        }
        if number % 7 == 3 {
            text.push_str(&format!(
                "    let value_{number} = compute({number}, \"a long argument that has to wrap\", Some(other.field), [1, 2, 3, 4, 5, 6, 7, 8]);"
            ));
        } else {
            text.push_str(&format!("let value_{number} = compute({number});"));
        }
    }
    return text;
}

/// Syntax classes that color every third word of every line,
/// by the word's place in its line,
/// so what a line is colored with does not depend on the lines before it.
fn styles_of(text: &str) -> Vec<StyleSpan> {
    let mut styles = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let mut at = offset;
        for (index, word) in line.split_inclusive(' ').enumerate() {
            let length = word.chars().count();
            if index % 3 == 0 && length > 1 {
                styles.push(StyleSpan {
                    start: at,
                    end: at + length - 1,
                    style: 1 + index % 5,
                });
            }
            at += length;
        }
        offset += line.chars().count();
    }
    return styles;
}

/// A fixture of `text` at `scale`,
/// wrapped at `wrap` with the rows the shaper finds.
fn fixture_of(text: &str, wrap: f32, scale: f32) -> Fixture {
    let document = Document::new(text);
    let mut shaper = TextShaper::new();
    let mut rows = LineRows::plain(document.text().len_lines());
    let mut measured = Vec::new();
    for line in 0..document.text().len_lines() {
        let count = shaper
            .wrapped_row(&document, line, scale, wrap)
            .visual_rows();
        measured.push((line, count));
    }
    rows.apply(measured);
    let map = RowMap::plain(document.text().len_lines()).with_rows(&rows);
    return Fixture {
        styles: styles_of(text),
        document,
        map,
        marks: Vec::new(),
        ranges: Vec::new(),
        active: None,
        fills: FILLS,
        wrap: Some(wrap),
        scale,
    };
}

/// A fixture of `lines` lines of code.
fn fixture(lines: usize, wrap: f32, scale: f32) -> Fixture {
    return fixture_of(&code(lines), wrap, scale);
}

/// A source of the fixture.
impl Fixture {
    /// The source the picture is drawn from.
    fn source(&self) -> Source<'_> {
        return Source {
            document: &self.document,
            styles: &self.styles,
            map: &self.map,
            colors: DARK,
            inks: INKS,
            fills: self.fills,
            blocks: &[],
            marks: &self.marks,
            ranges: &self.ranges,
            active: self.active,
            wrap: self.wrap,
            scale: self.scale,
        };
    }

    /// The input of a build in a view `view` tall and a minimap `width` wide,
    /// for the bands `window`.
    fn input(&self, window: Range<usize>, view: f32, width: f32) -> Input<'_> {
        return Input {
            source: self.source(),
            view,
            width,
            window,
        };
    }

    /// The numbers of every band of the text.
    fn all(&self) -> Range<usize> {
        return 0..band_count(self.map.height(), BAND);
    }

    /// Select the characters `anchor` to `head`.
    fn select(&mut self, anchor: usize, head: usize) {
        self.document.select(ReadingPosition {
            anchor,
            head,
            viewport: 0,
            affinity: ReadingAffinity::Downstream,
        });
    }
}

/// The items and statistics of the picture of `window`,
/// built with `shaper` and `cache`.
fn items_of(
    fixture: &Fixture,
    window: Range<usize>,
    shaper: &mut TextShaper,
    cache: &mut LineCache,
) -> (Vec<Item>, Range<usize>, Stats) {
    let input = fixture.input(window, 660.0, 110.0);
    return slice_items(shaper, cache, &input, &Never).expect("a picture");
}

/// The picture of the fixture in one go.
fn whole(fixture: &Fixture) -> (Vec<Item>, Range<usize>, Stats) {
    return items_of(
        fixture,
        fixture.all(),
        &mut TextShaper::new(),
        &mut LineCache::default(),
    );
}

/// The slices of consecutive windows meet without a gap or an overlap,
/// the first starts at the picture's first row and the last ends at its last,
/// and the whole picture is as tall as the text at the fixed scale.
#[test]
fn consecutive_slices_meet_and_cover_the_picture() {
    for scale in [1.0_f32, 1.25, 1.5, 2.0, 3.0] {
        let height = 7_310.5_f32;
        let ratio = f64::from(Geometry::new(height, 660.0).scale());
        let count = band_count(height, BAND);
        let total = picture_rows(height, scale, ratio);
        assert!(count > 8);
        let mut at = 0;
        for cut in [0, 3, 3, 4, 9.min(count - 1), count] {
            let rows = slice_rows(&(at..cut.max(at)), height, scale, ratio);
            at = cut.max(at);
            assert!(rows.start <= rows.end, "a slice runs backwards at {scale}");
        }
        let first = slice_rows(&(0..3), height, scale, ratio);
        let second = slice_rows(&(3..count), height, scale, ratio);
        assert_eq!(first.start, 0);
        assert_eq!(
            first.end, second.start,
            "the slices share an edge at scale {scale}"
        );
        assert_eq!(
            second.end, total,
            "the last slice ends at the picture's end"
        );
        assert_eq!(
            slice_rows(&(0..count), height, scale, ratio),
            0..total,
            "all the bands make up the whole picture"
        );
        assert!(
            (total as f64 - f64::from(height) * f64::from(scale) / 6.0).abs() < 1.0,
            "the picture is a sixth of the text"
        );
    }
}

/// A window of the most bands one texture can hold is a slice no taller than the texture,
/// at every scale and for the guaranteed and the larger limits,
/// and two bands more are not (the cap wastes less than a band).
#[test]
fn the_most_bands_fit_the_texture_limit_and_one_more_does_not() {
    let height = 400_000.0_f32;
    let ratio = f64::from(Geometry::new(height, 660.0).scale());
    for limit in [2048_u32, 4096, 8192, 16384] {
        for scale in [1.0_f32, 1.25, 1.5, 2.0, 3.0] {
            let most = most_bands(limit, scale, ratio);
            let rows = slice_rows(&(10..10 + most), height, scale, ratio);
            assert!(
                rows.len() <= limit as usize,
                "{} rows pass the limit {limit} at scale {scale}",
                rows.len()
            );
            let more = slice_rows(&(10..12 + most), height, scale, ratio);
            assert!(
                more.len() > limit as usize || most == 1,
                "two more bands still fit {limit} at scale {scale}"
            );
        }
    }
}

/// The glyphs of a slice are drawn in the palette's inks for their syntax classes,
/// and the slice holds the lines of its bands and little else.
#[test]
fn a_slice_draws_the_glyphs_of_its_lines_in_the_palette_inks() {
    let fixture = fixture(300, 730.0, 1.0);
    let (items, rows, stats) = whole(&fixture);
    let runs = drawn_runs(&items);
    assert!(!rows.is_empty());
    assert_eq!(stats.shaped, stats.lines, "a cold build shapes every line");
    assert_eq!(stats.skipped, 0);
    assert!(runs.len() >= stats.lines, "a run for every line at least");
    let mut classes = std::collections::BTreeSet::new();
    for run in &runs {
        classes.insert(run.ink);
    }
    assert!(
        classes.contains(&DARK.foreground),
        "the plain text is drawn in the foreground"
    );
    for role in 1..=5_u32 {
        let wanted = ink(role, DARK);
        assert!(
            classes.contains(&wanted),
            "no run is drawn in the ink of class {role}: {wanted:?}"
        );
    }
}

/// The selection,
/// the matches,
/// and the active match's boundary are drawn as marks under the glyphs,
/// in the markup's inks and order.
#[test]
fn the_selection_and_the_matches_are_marks_under_the_glyphs() {
    let mut fixture = fixture(40, 730.0, 1.0);
    let plain = whole(&fixture).0;
    assert!(
        drawn_fills(&plain).is_empty(),
        "a text with no selection and no match has no marks"
    );
    // A selection over the middle of the second and third lines.
    let start = fixture.document.text().line_to_char(1) + 4;
    let end = fixture.document.text().line_to_char(2) + 12;
    fixture.select(start, end);
    let selected = whole(&fixture).0;
    let fills = drawn_fills(&selected);
    // The first line's mark and the cell its line break takes touch, and are drawn as one so no seam shows.
    assert_eq!(
        fills.len(),
        2,
        "one mark for each of the two lines the selection crosses: {fills:?}"
    );
    for mark in &fills {
        assert_eq!(mark.ink, FILLS.selection);
        assert!(mark.right > mark.left && mark.bottom > mark.top);
    }
    let first_glyph = selected
        .iter()
        .position(|item| return matches!(item, Item::Glyphs(_)))
        .expect("glyphs");
    let last_fill = selected
        .iter()
        .rposition(|item| return matches!(item, Item::Fill { .. }))
        .expect("marks");
    assert!(last_fill < first_glyph, "the marks stand under the glyphs");
    // The same selection as an active find match is outlined too.
    fixture.fills.boundary = true;
    let outlined = drawn_fills(&whole(&fixture).0);
    assert!(
        outlined.len() > fills.len(),
        "the active match has a boundary"
    );
    assert!(
        outlined
            .iter()
            .any(|mark| return mark.ink == FILLS.foreground),
        "the boundary is drawn in the foreground"
    );
    // A match that is not the selection: a translucent fill and a thin boundary.
    fixture.fills.boundary = false;
    fixture.select(0, 0);
    let from = fixture.document.text().line_to_char(5) + 4;
    fixture.ranges = vec![FindRange {
        start: from,
        end: from + 5,
    }];
    let found = drawn_fills(&whole(&fixture).0);
    assert_eq!(found.len(), 2, "a fill and one frame: {found:?}");
    let fill = found
        .iter()
        .find(|mark| return mark.inset == 0.0)
        .expect("the translucent fill");
    let frame = found
        .iter()
        .find(|mark| return mark.inset > 0.0)
        .expect("the boundary drawn as one frame");
    assert!(
        fill.ink[3] > 0 && fill.ink[3] < 60,
        "the fill is faint: {fill:?}"
    );
    assert!(
        frame.ink[3] > 100,
        "the boundary is 0.7 of the foreground: {frame:?}"
    );
    assert!(
        frame.left == fill.left && frame.right == fill.right,
        "the frame is the outline of the fill"
    );
}

/// A diagnostic's underline is a stroke,
/// and an error's differs from a warning's.
#[test]
fn a_diagnostic_is_drawn_as_a_stroke_in_its_ink() {
    let mut fixture = fixture(40, 730.0, 1.0);
    let line = fixture.document.text().line_to_char(4);
    fixture.marks = vec![
        Mark {
            start: line + 4,
            end: line + 9,
            severity: Severity::Error,
        },
        Mark {
            start: line + 12,
            end: line + 17,
            severity: Severity::Warning,
        },
    ];
    let strokes = drawn_strokes(&whole(&fixture).0);
    assert!(
        strokes
            .iter()
            .any(|drawn| return drawn.stroke.ink == INKS.error && drawn.stroke.dash.is_empty()),
        "a solid wave in the error ink"
    );
    assert!(
        strokes
            .iter()
            .any(|drawn| return drawn.stroke.ink == INKS.warning && !drawn.stroke.dash.is_empty()),
        "dashes in the warning ink"
    );
}

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
fn edited(fixture: &Fixture, line: usize, replace: &str, with: &str) -> Fixture {
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

/// The cut of a slice's items into chunks drops nothing and keeps a line that crosses a seam in both chunks:
/// at a scale of 1.5 the seams of 512-row chunks fall in the middle of a line.
#[test]
fn the_cut_keeps_every_item_and_a_line_on_a_seam_in_both_chunks() {
    let scale = 1.5_f32;
    let fixture = fixture(400, 730.0, scale);
    let (items, rows, _) = items_of(
        &fixture,
        0..6,
        &mut TextShaper::new(),
        &mut LineCache::default(),
    );
    let ratio = f64::from(Geometry::new(fixture.map.height(), 660.0).scale());
    let spans = chunk_rows(&rows);
    assert!(spans.len() >= 2, "{} chunks", spans.len());
    let reaches: Vec<(f32, f32)> = spans
        .iter()
        .map(|span| {
            return (
                (span.start as f64 / ratio) as f32,
                (span.end as f64 / ratio) as f32,
            );
        })
        .collect();
    let cuts = cut(&items, &reaches, 2.0 * 24.0 * scale);
    let runs = drawn_runs(&items);
    let mut seen = vec![0_usize; runs.len()];
    for kept in &cuts {
        let mut count = 0;
        for kept_run in drawn_runs(kept) {
            let at = runs.iter().position(|run| return *run == kept_run);
            seen[at.expect("a kept run is one of the items")] += 1;
            count += 1;
        }
        assert!(count > 0, "a chunk of the dense text kept no glyph run");
    }
    assert!(
        seen.iter().all(|times| return *times >= 1),
        "an item was dropped by the cut"
    );
    let twice = seen.iter().filter(|times| return **times >= 2).count();
    assert!(twice > 0, "no line on a seam was kept in both chunks");
    assert!(
        twice * 4 < runs.len(),
        "{twice} of {} runs were kept twice: the cut keeps too much",
        runs.len()
    );
}

/// The size of the slice a build makes:
/// as wide as its chunks and as tall as they are together,
/// or nothing without a chunk.
fn size_of(built: &ide_app::minimap::Built) -> (u32, u32) {
    let width = built.chunks.first().map_or(0, |chunk| return chunk.width);
    assert!(built.chunks.iter().all(|chunk| return chunk.width == width));
    return (
        width,
        built.chunks.iter().map(|chunk| return chunk.height).sum(),
    );
}

/// A slice taller than a render holds is cut into chunks of at most `CHUNK_ROWS` rows,
/// whole rows from the slice's first,
/// that stack to the slice's height.
#[test]
fn a_tall_slice_is_cut_into_chunks_of_at_most_the_chunk_rows() {
    let fixture = fixture(900, 730.0, 1.0);
    // Twenty bands are 2,560 rows: five full chunks.
    let input = fixture.input(0..20, 660.0, 110.0);
    let built = build(
        &mut TextShaper::new(),
        &mut LineCache::default(),
        &input,
        &Never,
    )
    .expect("a picture");
    let heights: Vec<u32> = built
        .chunks
        .iter()
        .map(|chunk| return chunk.height)
        .collect();
    assert_eq!(heights, vec![CHUNK_ROWS; 5]);
    assert_eq!(size_of(&built), (110, 2560));
    assert_eq!(
        chunk_rows(&(3..3 + 1300)),
        vec![3..515, 515..1027, 1027..1303],
        "the last chunk is the rest"
    );
    assert!(chunk_rows(&(5..5)).is_empty());
}

/// The chunks of a picture are grouped into textures of whole chunks,
///  none taller than the device's limit,
/// top first,
///  with no chunk left out or in two textures,
///  and a limit that is not a multiple of a chunk
/// wastes the rest of the texture rather than cutting a chunk.
#[test]
fn chunks_are_grouped_into_textures_that_fit_the_limit() {
    let full = CHUNK_ROWS;
    // Sixteen full chunks and a short one fill a texture of 8,192 rows and start another.
    let mut seventeen = vec![full; 16];
    seventeen.push(100);
    assert_eq!(texture_runs(&seventeen, 110, 8192), vec![0..16, 16..17]);
    // A limit of 1,024 rows holds two chunks.
    let six = vec![full; 6];
    assert_eq!(texture_runs(&six, 110, 1024), vec![0..2, 2..4, 4..6]);
    // A limit of 1,500 rows holds two chunks and wastes 476 rows.
    assert_eq!(texture_runs(&six, 110, 1500), vec![0..2, 2..4, 4..6]);
    // One chunk is one texture at a limit of exactly a chunk, and a limit under a chunk still makes progress.
    assert_eq!(texture_runs(&[full, full], 110, full), vec![0..1, 1..2]);
    assert_eq!(texture_runs(&[full, full], 110, 100), vec![0..1, 1..2]);
    // Nothing gives no texture, and a short picture is one texture.
    assert!(texture_runs(&[], 110, 8192).is_empty());
    assert_eq!(texture_runs(&[40], 110, 8192), vec![0..1]);
    // Every grouping is a partition of the chunks in order whose textures fit.
    for limit in [512_u32, 700, 1024, 3000, 8192, 16384] {
        let uneven: Vec<u32> = (0..50).map(|index| return 100 + index * 9).collect();
        let runs = texture_runs(&uneven, 220, limit);
        let mut next = 0;
        for run in &runs {
            assert_eq!(run.start, next, "a gap or an overlap at limit {limit}");
            next = run.end;
            let rows: u32 = uneven[run.clone()].iter().sum();
            assert!(
                rows <= texture_rows(220, limit) || run.len() == 1,
                "{rows} rows pass the limit {limit}"
            );
        }
        assert_eq!(next, uneven.len(), "a chunk was left out at limit {limit}");
    }
}

/// A wide picture is stacked from shorter textures than the device's side allows,
/// so no texture passes the byte rule the source tile keeps.
#[test]
fn a_wide_texture_is_held_to_the_byte_rule_as_well_as_the_side() {
    assert_eq!(texture_rows(110, 8192), 8192);
    assert_eq!(texture_rows(0, 8192), 8192);
    // 64 MiB of RGBA over 8,192 columns is 2,048 rows.
    assert_eq!(texture_rows(8192, 16384), 2048);
    assert_eq!(texture_rows(8192, 1024), 1024);
}

/// The texture is as wide as the minimap and as tall as the slice,
/// at the display scale.
#[test]
fn the_texture_is_the_size_of_the_slice_at_the_display_scale() {
    for scale in [1.0_f32, 2.0] {
        let fixture = fixture(300, 730.0, scale);
        let input = fixture.input(1..4, 660.0, 110.0);
        let built = build(
            &mut TextShaper::new(),
            &mut LineCache::default(),
            &input,
            &Never,
        )
        .expect("a picture");
        assert_eq!(
            size_of(&built),
            ((110.0 * scale) as u32, (3.0 * 128.0 * scale) as u32)
        );
        assert_eq!(built.top, (128.0 * scale) as u32);
        // An empty window makes no texture.
        let empty = fixture.input(0..0, 660.0, 110.0);
        let none = build(
            &mut TextShaper::new(),
            &mut LineCache::default(),
            &empty,
            &Never,
        )
        .expect("an empty picture");
        assert_eq!(size_of(&none), (0, 0));
    }
}
