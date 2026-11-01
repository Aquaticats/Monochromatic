//! The densest text the minimap draws still renders:
//!  Vello 0.11 sizes its scratch buffers with fixed constants,
//!  and a scene that needs more than they hold renders blank without an error
//!  (`doc/planning/ide-vello-buffer-sizes.md`),
//!  so a chunk of the picture must stay inside them for the worst text the editor can show.
//! The scenes are rendered by the same Vello code as the window's,
//! on the harness device.

/// Renders a scene on the harness device and reads it back.
use super::gpu_tile::render_scene;
/// The minimap's build and its inputs.
use ide_app::{
    annotation_layout::AnnotationColors,
    code_colors::CodeColors,
    document::{Document, ReadingAffinity, ReadingPosition},
    find::FindRange,
    line_rows::LineRows,
    minimap::{CHUNK_ROWS, Cancel, Fills, Input, LineCache, Source, build},
    row_map::RowMap,
    shaped_text::TextShaper,
    source_style::StyleSpan,
    tile_scene::TileScene,
};

/// The dark scheme's palette.
pub(super) const DARK: CodeColors = CodeColors {
    foreground: [220, 220, 220, 255],
    selected: [255, 255, 255, 255],
    dark: true,
};

/// The annotation inks.
pub(super) const INKS: AnnotationColors = AnnotationColors {
    hint: [150, 160, 170, 255],
    error: [255, 110, 120, 255],
    warning: [240, 200, 40, 255],
    information: [90, 200, 250, 255],
    suggestion: [170, 170, 170, 255],
};

/// The fill inks.
pub(super) const FILLS: Fills = Fills {
    selection: [30, 90, 200, 255],
    foreground: [220, 220, 220, 255],
    boundary: false,
};

/// A build that is never cancelled.
pub(super) struct Never;

impl Cancel for Never {
    /// Never cancelled.
    fn is_cancelled(&self) -> bool {
        return false;
    }
}

/// The glyphs with the most outline in the code font:
/// wide letters,
/// digits and symbols made of many curves.
const DENSE: &str =
    "W@%&#B8$MQRSgmw0O&@%W#B8$MQRSgmw0O&@%W#B8$MQRSgmw0O&@%W#B8$MQRSgmw0O&@%W#B8$MQRSgmw0O&@%";

/// A text of `lines` lines that fill every visual row to the wrap width with [`DENSE`].
fn dense_text(lines: usize) -> String {
    let mut text = String::new();
    for number in 0..lines {
        if number > 0 {
            text.push('\n');
        }
        text.push_str(DENSE);
        text.push_str(&format!("{number}"));
    }
    return text;
}

/// Syntax classes that color a word of every line,
/// so each line has several runs.
pub(super) fn styles_of(text: &str) -> Vec<StyleSpan> {
    let mut styles = Vec::new();
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let length = line.chars().count();
        for third in 0..3 {
            styles.push(StyleSpan {
                start: offset + third * 30,
                end: (offset + third * 30 + 20).min(offset + length - 1),
                style: 1 + third,
            });
        }
        offset += length;
    }
    return styles;
}

/// The chunks a dense text of `lines` lines draws for `bands` bands,
/// every line selected and found,
/// when `crowded` is set.
fn chunks_of(lines: usize, bands: usize, crowded: bool) -> Vec<TileScene> {
    let text = dense_text(lines);
    let mut document = Document::new(&text);
    let mut shaper = TextShaper::new();
    let mut rows = LineRows::plain(document.text().len_lines());
    let mut measured = Vec::new();
    for line in 0..document.text().len_lines() {
        let count = shaper
            .wrapped_row(&document, line, 1.0, 730.0)
            .visual_rows();
        measured.push((line, count));
    }
    rows.apply(measured);
    let map = RowMap::plain(document.text().len_lines()).with_rows(&rows);
    let styles = styles_of(&text);
    let mut ranges = Vec::new();
    if crowded {
        document.select(ReadingPosition {
            anchor: 0,
            head: document.text().len_chars(),
            viewport: 0,
            affinity: ReadingAffinity::Downstream,
        });
        for line in 0..document.text().len_lines() {
            let start = document.text().line_to_char(line);
            ranges.push(FindRange {
                start,
                end: start + 6,
            });
        }
    }
    let source = Source {
        document: &document,
        styles: &styles,
        map: &map,
        colors: DARK,
        inks: INKS,
        fills: FILLS,
        blocks: &[],
        marks: &[],
        ranges: &ranges,
        active: None,
        wrap: Some(730.0),
        scale: 1.0,
    };
    let input = Input {
        source,
        view: 660.0,
        width: 110.0,
        window: 0..bands,
    };
    let built = build(&mut shaper, &mut LineCache::default(), &input, &Never).expect("a picture");
    return built.chunks;
}

/// How many pixels of a rendered chunk carry ink.
fn inked(chunk: &TileScene) -> usize {
    let pixels = render_scene(chunk);
    return pixels
        .as_slice()
        .iter()
        .filter(|pixel| return pixel.a > 0)
        .count();
}

/// Every chunk of the densest slice a window can ask for renders,
/// and the same text with everything selected and found renders too.
#[test]
fn the_densest_chunks_are_not_blank() {
    // The widest window the device allows is 8192 rows: 64 bands.
    for crowded in [false, true] {
        let chunks = chunks_of(1100, 64, crowded);
        assert_eq!(chunks.len(), 8192_usize.div_ceil(CHUNK_ROWS as usize));
        let area = (chunks[0].width * chunks[0].height) as usize;
        for (number, chunk) in chunks.iter().enumerate() {
            let ink = inked(chunk);
            assert!(
                ink * 5 > area,
                "chunk {number} is nearly blank with {ink} inked pixels of {area} (crowded {crowded})"
            );
        }
    }
}

/// The ink of the first `count` chunks drawn as one render,
/// and drawn one render each.
fn together_and_apart(chunks: &[TileScene], count: usize) -> (usize, usize, u32) {
    let kept = &chunks[..count];
    let width = kept[0].width;
    let height: u32 = kept.iter().map(|chunk| return chunk.height).sum();
    let apart: usize = kept.iter().map(inked).sum();
    let mut scene = vello::Scene::new();
    let mut row = 0.0;
    for chunk in kept {
        scene.append(
            &chunk.scene,
            Some(vello::kurbo::Affine::translate((0.0, row))),
        );
        row += f64::from(chunk.height);
    }
    let together = inked(&TileScene {
        scene,
        width,
        height,
    });
    return (together, apart, height);
}

/// Two chunks drawn as one render still draw all their ink:
/// the chunk size keeps the densest text a factor of two inside what one render holds.
#[test]
fn the_chunk_size_leaves_a_factor_of_two_inside_what_one_render_holds() {
    let chunks = chunks_of(1100, 64, true);
    let (together, apart, height) = together_and_apart(&chunks, 2);
    assert_eq!(height, 2 * CHUNK_ROWS);
    assert!(
        together * 100 >= apart * 98,
        "one render of {height} rows drew {together} inked pixels where two renders drew {apart}"
    );
}

/// Positive control:
/// the same text drawn as one render of every chunk of the widest window comes out blank,
/// so the checks above can tell a blank render from a good one,
/// and the cut into chunks is what keeps the renderer's fixed buffers sufficient.
/// Measured on the merged tip of 2026-10-21 (selected rows drawn once):
/// eight chunks (4096 rows) still drew every inked pixel and twelve (6144 rows) came out blank,
/// so this control takes the whole slice rather than a count that the scene's size moves.
#[test]
fn one_render_of_every_dense_chunk_comes_out_blank() {
    let chunks = chunks_of(1100, 64, true);
    let (together, apart, height) = together_and_apart(&chunks, chunks.len());
    assert!(apart > 0);
    assert_eq!(
        together,
        0,
        "one render of {height} rows drew {together} inked pixels where {} renders drew {apart}",
        chunks.len()
    );
}

/// A frame scaled down to a sixth of its height keeps the coverage of its edges:
/// one pixel of a 178-alpha frame is a sixth of 178 on the minimap row that holds it,
/// and the inside stays clear.
/// Drawn with the even-odd rule the same frame came out at half the alpha,
/// because the renderer folds
/// the sum of the two rectangles' partial areas in one pixel and that is not the area between them.
#[test]
fn a_frame_scaled_down_keeps_the_coverage_of_its_edges() {
    use ide_app::tile_scene::{Item, encode};
    let items = vec![Item::Fill {
        left: 12.0,
        top: 25.0,
        right: 90.0,
        bottom: 47.0,
        inset: 1.0,
        ink: [0, 0, 0, 178],
    }];
    let mut scene = vello::Scene::new();
    scene.append(
        &encode(&items, 100.0),
        Some(vello::kurbo::Affine::scale_non_uniform(0.2, 1.0 / 6.0)),
    );
    let pixels = render_scene(&TileScene {
        scene,
        width: 20,
        height: 10,
    });
    let alpha = |row: usize| return i32::from(pixels.as_slice()[row * 20 + 6].a);
    // The top edge covers y 25 to 26 of the row that stands for y 24 to 30, and the bottom edge y 46 to 47 of 42 to 48.
    assert!((alpha(4) - 30).abs() <= 2, "top edge alpha {}", alpha(4));
    assert!((alpha(7) - 30).abs() <= 2, "bottom edge alpha {}", alpha(7));
    assert_eq!(alpha(5), 0, "the inside of the frame is clear");
    assert_eq!(alpha(6), 0, "the inside of the frame is clear");
    assert_eq!(alpha(3), 0, "nothing above the frame");
}
