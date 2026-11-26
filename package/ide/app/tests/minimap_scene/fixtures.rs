//! The palettes, fixture source and item readers every test builds:
//!  a text of ordinary code with long lines,
//!  syntax classes, marks and find matches,
//!  the two cancel handles,
//!  the fixture and its source and input,
//!  and the items of a window.

use ide_app::{
    annotation::Mark,
    annotation_layout::AnnotationColors,
    code_colors::CodeColors,
    document::{
        Document,
        ReadingAffinity,
        ReadingPosition,
    },
    find::FindRange,
    line_rows::LineRows,
    minimap::{
        BAND,
        Cancel,
        Fills,
        Input,
        LineCache,
        Source,
        Stats,
        band_count,
        slice_items,
    },
    row_map::RowMap,
    shaped_text::TextShaper,
    source_style::StyleSpan,
    tile_scene::Item,
};
use std::ops::Range;

/// The dark scheme's palette of the tests.
pub const DARK: CodeColors = CodeColors {
    foreground: [220, 220, 220, 255],
    selected: [255, 255, 255, 255],
    dark: true,
};

/// The annotation inks of the tests.
pub const INKS: AnnotationColors = AnnotationColors {
    hint: [150, 160, 170, 255],
    error: [255, 110, 120, 255],
    warning: [240, 200, 40, 255],
    information: [90, 200, 250, 255],
    suggestion: [170, 170, 170, 255],
};

/// The fill inks of the tests:
/// a selection no glyph has the color of.
pub const FILLS: Fills = Fills {
    selection: [30, 90, 200, 255],
    foreground: [220, 220, 220, 255],
    boundary: false,
};

/// A build that is never cancelled.
pub struct Never;

impl Cancel for Never {
    /// Never cancelled.
    fn is_cancelled(&self) -> bool {
        return false;
    }
}

/// A build that is cancelled before it starts.
pub struct Always;

impl Cancel for Always {
    /// Always cancelled.
    fn is_cancelled(&self) -> bool {
        return true;
    }
}

/// Everything a source borrows,
/// owned.
pub struct Fixture {
    /// The text and its selection.
    pub document: Document,
    /// Syntax classes.
    pub styles: Vec<StyleSpan>,
    /// The vertical mapping,
    /// with the rows the wrapped lines really have.
    pub map: RowMap,
    /// Diagnostic marks.
    pub marks: Vec<Mark>,
    /// Find matches.
    pub ranges: Vec<FindRange>,
    /// The match the selection covers.
    pub active: Option<FindRange>,
    /// The inks of the fills.
    pub fills: Fills,
    /// The width lines are broken at.
    pub wrap: Option<f32>,
    /// Physical pixels per logical pixel.
    pub scale: f32,
}

/// A text of ordinary code with some lines long enough to wrap.
pub fn code(lines: usize) -> String {
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
pub fn styles_of(text: &str) -> Vec<StyleSpan> {
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
pub fn fixture_of(
    text: &str,
    wrap: f32,
    scale: f32,
) -> Fixture {
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
pub fn fixture(
    lines: usize,
    wrap: f32,
    scale: f32,
) -> Fixture {
    return fixture_of(&code(lines), wrap, scale);
}

/// A source of the fixture.
impl Fixture {
    /// The source the picture is drawn from.
    pub fn source(&self) -> Source<'_> {
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
    pub fn input(
        &self,
        window: Range<usize>,
        view: f32,
        width: f32,
    ) -> Input<'_> {
        return Input {
            source: self.source(),
            view,
            width,
            window,
        };
    }

    /// The numbers of every band of the text.
    pub fn all(&self) -> Range<usize> {
        return 0..band_count(self.map.height(), BAND);
    }

    /// Select the characters `anchor` to `head`.
    pub fn select(
        &mut self,
        anchor: usize,
        head: usize,
    ) {
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
pub fn items_of(
    fixture: &Fixture,
    window: Range<usize>,
    shaper: &mut TextShaper,
    cache: &mut LineCache,
) -> (Vec<Item>, Range<usize>, Stats) {
    let input = fixture.input(window, 660.0, 110.0);
    return slice_items(shaper, cache, &input, &Never).expect("a picture");
}

/// The picture of the fixture in one go.
pub fn whole(fixture: &Fixture) -> (Vec<Item>, Range<usize>, Stats) {
    return items_of(
        fixture,
        fixture.all(),
        &mut TextShaper::new(),
        &mut LineCache::default(),
    );
}

/// The size of the slice a build makes:
/// as wide as its chunks and as tall as they are together,
/// or nothing without a chunk.
pub fn size_of(built: &ide_app::minimap::Built) -> (u32, u32) {
    let width = built.chunks.first().map_or(0, |chunk| return chunk.width);
    assert!(built.chunks.iter().all(|chunk| return chunk.width == width));
    return (
        width,
        built.chunks.iter().map(|chunk| return chunk.height).sum(),
    );
}
