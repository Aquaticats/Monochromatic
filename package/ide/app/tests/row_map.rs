//! The vertical mapping between pixels and source lines,
//!  without fonts or a window.

/// What:
///  Import the production mapping through the library's public interface.
/// Why:
///  Painting,
///  hit testing,
///  and scrolling call exactly these functions;
///  the tests must not use a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CODE_ROW, Place, RowMap } from 'ide-app/row-map';
/// ```
use ide_app::row_map::{CODE_ROW, Place, RowMap};

/// A place on a code row.
fn code(line: usize) -> Place {
    return Place {
        line,
        in_block: false,
    };
}

/// A place in the virtual rows above a code row.
fn block(line: usize) -> Place {
    return Place {
        line,
        in_block: true,
    };
}

/// Without blocks,
///  line `n` starts at exactly `n` code rows,
///  as the view placed rows before virtual rows existed.
#[test]
fn plain_text_places_line_n_at_n_code_rows() {
    let map = RowMap::plain(1000);
    assert_eq!(CODE_ROW, 24.0);
    for line in [0_usize, 1, 2, 17, 499, 999] {
        // `as f32` converts the line index to the float pixels are measured in.
        let expected = line as f32 * 24.0;
        assert_eq!(map.block_top(line), expected);
        assert_eq!(map.code_top(line), expected);
        assert_eq!(map.code_bottom(line), expected + 24.0);
        assert_eq!(map.block_height(line), 0.0);
        assert_eq!(map.locate(expected), code(line));
        assert_eq!(map.locate(expected + 23.5), code(line));
    }
    assert_eq!(map.height(), 24_000.0);
    assert_eq!(
        map.block_top(1000),
        24_000.0,
        "the line count names the end of the text"
    );
    assert_eq!(map.lines(), 1000);
}

/// An empty text still has one line,
///  and a block on a line the text lacks or without height is left out.
#[test]
fn degenerate_inputs_are_normalized() {
    let empty = RowMap::plain(0);
    assert_eq!(empty.lines(), 1);
    assert_eq!(empty.height(), 24.0);
    let map = RowMap::new(3, &[(1, 0.0), (2, -4.0), (3, 26.0), (7, 26.0)]);
    assert_eq!(
        map,
        RowMap::plain(3),
        "blocks without height or without a line were kept"
    );
}

/// A block pushes its own code row and every later line down by its height,
///  and nothing above it.
#[test]
fn blocks_push_their_line_and_everything_beneath_down() {
    // Lines 2, 3, and 7 own blocks of 26, 42, and 16 px.
    let map = RowMap::new(10, &[(2, 26.0), (3, 42.0), (7, 16.0)]);
    assert_eq!(map.code_top(0), 0.0);
    assert_eq!(map.code_top(1), 24.0);
    assert_eq!(map.block_top(2), 48.0, "nothing above line 2 moved");
    assert_eq!(map.code_top(2), 74.0);
    assert_eq!(
        map.block_top(3),
        98.0,
        "line 3's block starts where line 2's code row ends"
    );
    assert_eq!(map.code_top(3), 140.0);
    assert_eq!(map.code_top(4), 164.0);
    assert_eq!(map.block_top(7), 236.0);
    assert_eq!(map.code_top(7), 252.0);
    assert_eq!(map.code_top(9), 300.0);
    assert_eq!(map.height(), 10.0 * 24.0 + 26.0 + 42.0 + 16.0);
    assert_eq!(map.block_height(2), 26.0);
    assert_eq!(map.block_height(4), 0.0);
}

/// Every pixel belongs to exactly one line,
///  and a block belongs to the code row beneath it.
#[test]
fn every_pixel_belongs_to_one_line_and_blocks_to_the_row_beneath() {
    let map = RowMap::new(10, &[(2, 26.0), (3, 42.0), (7, 16.0)]);
    assert_eq!(map.locate(47.9), code(1));
    assert_eq!(map.locate(48.0), block(2), "the block's first pixel");
    assert_eq!(map.locate(73.9), block(2), "the block's last pixel");
    assert_eq!(map.locate(74.0), code(2));
    assert_eq!(map.locate(97.9), code(2));
    assert_eq!(
        map.locate(98.0),
        block(3),
        "consecutive annotated lines: line 3's block follows line 2's code row"
    );
    assert_eq!(map.locate(139.9), block(3));
    assert_eq!(map.locate(140.0), code(3));
    assert_eq!(map.locate(164.0), code(4));
    assert_eq!(map.locate(235.9), code(6));
    assert_eq!(map.locate(236.0), block(7));
    assert_eq!(map.locate(252.0), code(7));
    assert_eq!(map.locate(-5.0), code(0), "above the text");
    assert_eq!(map.locate(1.0e9), code(9), "below the text");
    assert_eq!(map.line_at(60.0), 2);
    // Walking every half pixel never skips a line and never goes back.
    let mut previous = 0;
    let mut y: f32 = 0.0;
    while y < map.height() {
        let line = map.line_at(y);
        assert!(
            line == previous || line == previous + 1,
            "line {line} after {previous} at {y}"
        );
        assert!(y >= map.block_top(line) && y < map.code_bottom(line));
        previous = line;
        y += 0.5;
    }
    assert_eq!(previous, 9);
}

/// A block above the first line starts the text;
///  the first code row is then below it.
#[test]
fn first_line_can_own_a_block() {
    let map = RowMap::new(2, &[(0, 26.0)]);
    assert_eq!(map.block_top(0), 0.0);
    assert_eq!(map.code_top(0), 26.0);
    assert_eq!(map.locate(0.0), block(0));
    assert_eq!(map.locate(26.0), code(0));
    assert_eq!(map.code_top(1), 50.0);
}

/// A page is the whole lines that fit in the view and exactly the pixels they take.
#[test]
fn page_steps_count_whole_lines_and_their_pixels() {
    let plain = RowMap::plain(100);
    for height in [0.0_f32, 10.0, 24.0, 95.9, 96.0, 100.0, 628.0] {
        // The rule before virtual rows existed: the view height in rows, rounded down, and at least one.
        let expected = ((height / 24.0).floor() as usize).max(1);
        for forward in [true, false] {
            assert_eq!(
                plain.page(240.0, height, forward),
                (expected, expected as f32 * 24.0),
                "height {height}, forward {forward}"
            );
        }
    }
    assert_eq!(
        plain.page(0.0, 96.0, false),
        (4, 96.0),
        "a page above the first line counts plain rows"
    );
    // Lines 2 and 3 own blocks of 26 and 42 px; the view starts at line 1 and is 100 px tall.
    let map = RowMap::new(10, &[(2, 26.0), (3, 42.0)]);
    assert_eq!(
        map.page(24.0, 100.0, true),
        (2, 74.0),
        "line 1 (24) and line 2 with its block (50) fit; line 3 with its block (66) does not"
    );
    assert_eq!(
        map.page(164.0, 100.0, false),
        (1, 66.0),
        "above line 4: line 3 with its block fits, line 2 with its block does not"
    );
    assert_eq!(
        map.page(24.0, 10.0, true),
        (1, 24.0),
        "a page is never less than one line"
    );
}
