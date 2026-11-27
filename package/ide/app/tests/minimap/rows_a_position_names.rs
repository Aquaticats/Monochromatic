//! The visual row a position on the picture names:
//!  wrapped rows and blocks included,
//!  and the position a row names back at every scroll.

/// The visual row counts and the map built from them.
use ide_app::line_rows::LineRows;
/// The geometry and its constants.
use ide_app::minimap::{
    Geometry,
    Target,
};
/// The vertical mapping the minimap shows.
use ide_app::row_map::{
    CODE_ROW,
    RowMap,
};

/// A position on the minimap shows the visual row of the vertical mapping it lies on,
/// wrapped rows and blocks
///  included,
/// and the row's own position shows that row again.
#[test]
fn a_position_names_the_visual_row_it_shows_and_the_row_names_its_position_back() {
    let mut rows = LineRows::plain(100);
    rows.apply([(10, 3), (40, 2)]);
    // A block of 48 px stands above line 5.
    let map = RowMap::new(100, &[(5, 48.0)]).with_rows(&rows);
    let geometry = Geometry::new(map.height(), 1000.0);
    for target in [
        Target {
            line: 0,
            row: 0,
        },
        Target {
            line: 4,
            row: 0,
        },
        Target {
            line: 5,
            row: 0,
        },
        Target {
            line: 10,
            row: 0,
        },
        Target {
            line: 10,
            row: 1,
        },
        Target {
            line: 10,
            row: 2,
        },
        Target {
            line: 40,
            row: 1,
        },
        Target {
            line: 99,
            row: 0,
        },
    ] {
        // The middle of the row's pixels, which are CODE_ROW over six tall.
        let y = geometry.y_of_row(&map, target, 0.0) + 0.5;
        assert_eq!(geometry.target(&map, y, 0.0), target, "the row at {y}");
    }
    // The block above line 5 shows line 5's first row.
    let in_block = geometry.y_of(map.block_top(5) + 10.0, 0.0);
    assert_eq!(
        geometry.target(&map, in_block, 0.0),
        Target {
            line: 5,
            row: 0
        }
    );
    // Both ends.
    assert_eq!(
        geometry.target(&map, -10.0, 0.0),
        Target {
            line: 0,
            row: 0
        }
    );
    assert_eq!(
        geometry.target(&map, 1e6, 0.0),
        Target {
            line: 99,
            row: 0
        }
    );
}

/// In a long text a position names a row of the part of the picture that is in view:
/// the row lies on that position,
/// and the part in view follows the scroll.
#[test]
fn in_a_long_text_a_position_names_a_row_that_lies_on_it_at_every_scroll() {
    let map = RowMap::plain(50_000);
    let geometry = Geometry::new(map.height(), 600.0);
    for share in [0.0_f32, 0.25, 0.5, 1.0] {
        let offset = geometry.max_offset() * share;
        for y in [0.0_f32, 0.5, 100.0, 299.5, 599.9] {
            let target = geometry.target(&map, y, offset);
            let top = geometry.y_of_row(&map, target, offset);
            // A code row is four pixels of the picture.
            assert!(
                top <= y + 1e-2 && y - top < 4.0 + 1e-2,
                "at {share} of the range position {y} named line {} at {top}",
                target.line
            );
        }
    }
    // The middle of the track in the middle of the range shows the middle of the text.
    let middle = geometry.max_offset() / 2.0;
    assert!((24_999..=25_001).contains(&geometry.target(&map, 300.0, middle).line));
    // The top of the track at the top shows the first line and the bottom at the end shows the last.
    assert_eq!(geometry.target(&map, 0.0, 0.0).line, 0);
    assert_eq!(
        geometry.target(&map, 600.0, geometry.max_offset()).line,
        49_999
    );
}
