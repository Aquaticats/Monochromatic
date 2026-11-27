//! The plan's chunks and textures:
//!  a cut that keeps every item and a line on a seam in both chunks,
//!  a tall slice cut into chunks of at most the chunk rows,
//!  chunks grouped into textures that fit the limit,
//!  a wide texture held to the byte rule as well as the side,
//!  and the texture's size at the display scale.

use crate::fixtures::{
    Never,
    fixture,
    items_of,
    size_of,
};
use ide_app::{
    minimap::{
        CHUNK_ROWS,
        Geometry,
        LineCache,
        build,
        chunk_rows,
        cut,
        texture_rows,
        texture_runs,
    },
    shaped_text::TextShaper,
    tile_scene::read::drawn_runs,
};

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
