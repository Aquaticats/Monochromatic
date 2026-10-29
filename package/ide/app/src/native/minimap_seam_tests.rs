//! The seams of the minimap's picture draw nothing the picture does not have:
//!  the chunks the picture is rendered in,
//!  stacked row by row,
//!  are the pixels one render of the whole picture gives,
//!  at a scale and a size one render holds.
//!  Every cut between two textures of a stacked picture is a cut between two chunks,
//!  so this is the check of the seam between two stacked textures against an unsplit render.
//! The scenes are rendered by the same Vello code as the window's,
//! on the harness device.

/// Renders a scene on the harness device and reads it back.
use super::gpu_tile::render_scene;
/// The inks and the helpers of the dense text's fixture.
use super::minimap_dense_tests::{DARK, FILLS, INKS, Never, styles_of};
/// The comparison of two renders to within the renderer's own noise.
use super::tile_reads::same_pixels;
/// The minimap's build and its parts.
use ide_app::{
    document::Document,
    line_rows::LineRows,
    minimap::{
        BAND, Geometry, Input, LineCache, Source, build, chunk_rows, cut, slice_items, text_span,
    },
    row_map::{CODE_ROW, RowMap},
    shaped_text::TextShaper,
    tile_scene::{TileScene, encode},
};
/// The texels of a read.
use slint::Rgba8Pixel;
/// A scene is placed by an affine transform.
use vello::{Scene, kurbo::Affine};

/// The width the lines wrap at,
///  in logical pixels.
const WRAP: f32 = 730.0;
/// The width of the minimap,
///  in logical pixels.
const WIDTH: f32 = 110.0;
/// The height of the view,
///  in logical pixels.
const VIEW: f32 = 660.0;

/// Lines of code of ordinary density,
///  some of which wrap.
fn code_text(lines: usize) -> String {
    let mut text = String::new();
    for number in 0..lines {
        text.push_str(&format!(
            "fn item_{number}(value: u32) -> u32 {{ value + {number} }} // trailing {}\n",
            "words ".repeat(number % 4 * 12)
        ));
    }
    return text;
}

/// How the scenes of a picture are made.
#[derive(Clone, Copy)]
enum Mode {
    /// As the product makes them:
    ///  chunks of at most 512 rows,
    ///  each keeping the items whose ink can reach its rows.
    Product,
    /// One scene for all the rows,
    ///  which one render draws.
    Unsplit,
    /// Chunks as the product cuts them,
    ///  but each keeping only the items whose layout box lies inside its rows.
    Bare,
}

/// The scenes of the picture of the first `bands` bands at `scale` made in `mode`,
/// and the rows the picture holds.
fn scenes(scale: f32, bands: usize, mode: Mode) -> (Vec<TileScene>, usize) {
    let text = code_text(900);
    let document = Document::new(&text);
    let mut shaper = TextShaper::new();
    let mut line_rows = LineRows::plain(document.text().len_lines());
    let mut measured = Vec::new();
    for line in 0..document.text().len_lines() {
        let count = shaper
            .wrapped_row(&document, line, scale, WRAP)
            .visual_rows();
        measured.push((line, count));
    }
    line_rows.apply(measured);
    let map = RowMap::plain(document.text().len_lines()).with_rows(&line_rows);
    let styles = styles_of(&text);
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
        wrap: Some(WRAP),
        scale,
    };
    let input = Input {
        source,
        view: VIEW,
        width: WIDTH,
        window: 0..bands,
    };
    let mut cache = LineCache::default();
    if let Mode::Product = mode {
        let built = build(&mut shaper, &mut cache, &input, &Never).expect("a picture");
        let rows = built
            .chunks
            .iter()
            .map(|chunk| return chunk.height as usize)
            .sum();
        return (built.chunks, rows);
    }
    let (items, span, _) = slice_items(&mut shaper, &mut cache, &input, &Never).expect("a picture");
    let ratio = f64::from(Geometry::new(map.height(), VIEW).scale());
    let width = ((WIDTH * scale).round() as usize).max(1);
    let across = f64::from(text_span(Some(WRAP))) * f64::from(scale);
    let spans = match mode {
        Mode::Unsplit => vec![span.clone()],
        Mode::Product | Mode::Bare => chunk_rows(&span),
    };
    let margin = match mode {
        Mode::Bare => 0.0,
        Mode::Product | Mode::Unsplit => 2.0 * CODE_ROW * scale,
    };
    let reaches: Vec<(f32, f32)> = spans
        .iter()
        .map(|rows| {
            return (
                (rows.start as f64 / ratio) as f32,
                (rows.end as f64 / ratio) as f32,
            );
        })
        .collect();
    let cuts = cut(&items, &reaches, margin);
    let extent = (map.height().max(CODE_ROW) * scale).ceil() + 1.0;
    let mut scenes = Vec::new();
    for (rows, kept) in spans.iter().zip(&cuts) {
        let mut scene = Scene::new();
        scene.append(
            &encode(kept, extent),
            Some(
                Affine::translate((0.0, -(rows.start as f64)))
                    * Affine::scale_non_uniform(width as f64 / across, ratio),
            ),
        );
        scenes.push(TileScene {
            scene,
            width: width as u32,
            height: rows.len() as u32,
        });
    }
    return (scenes, span.len());
}

/// The texels of `scenes` rendered one by one and stacked row by row.
fn stacked(scenes: &[TileScene]) -> Vec<Rgba8Pixel> {
    let mut texels = Vec::new();
    for scene in scenes {
        texels.extend_from_slice(render_scene(scene).as_slice());
    }
    return texels;
}

/// The chunks the picture is rendered in,
///  stacked,
///  equal the one render of all its rows,
/// at a scale where a seam falls between lines and at one where it falls through a line,
/// and the rows on both sides of each seam are inked.
#[test]
fn chunks_stacked_equal_one_render_of_the_picture() {
    for scale in [1.0_f32, 1.5] {
        // About 1,500 rows: three chunks, so two seams.
        let bands = (1500.0 / (f64::from(BAND) / 6.0 * f64::from(scale))).ceil() as usize;
        let (chunks, rows) = scenes(scale, bands, Mode::Product);
        let (whole, whole_rows) = scenes(scale, bands, Mode::Unsplit);
        assert_eq!(rows, whole_rows);
        assert_eq!(chunks.len(), rows.div_ceil(512), "scale {scale}");
        assert!(
            chunks.len() >= 3,
            "only {} chunks at scale {scale}",
            chunks.len()
        );
        let one = render_scene(&whole[0]);
        let width = one.width() as usize;
        let together = stacked(&chunks);
        assert_eq!(together.len(), one.as_slice().len());
        assert!(
            same_pixels(one.as_slice(), &together),
            "the chunks differ from the one render at scale {scale}"
        );
        for seam in (1..chunks.len()).map(|number| return number * 512) {
            let around = &together[(seam - 6) * width..(seam + 6) * width];
            assert!(
                around.iter().any(|pixel| return pixel.a > 0),
                "the rows around the seam at {seam} are blank at scale {scale}"
            );
        }
        // Positive control: chunks that keep only the items that sit inside their rows lose the ink of a line that
        // stands on a seam, so the comparison can tell a bad seam from a good one.
        // At a scale of one a seam of 512 rows is the 128th code row exactly, between two lines, and loses nothing.
        if scale > 1.0 {
            let (bare, _) = scenes(scale, bands, Mode::Bare);
            assert!(
                !same_pixels(one.as_slice(), &stacked(&bare)),
                "chunks cut with no reach past their rows still equal the one render at scale {scale}"
            );
        }
    }
}
