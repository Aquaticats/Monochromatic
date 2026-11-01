// Unit tests for `crate::band` (the pure sticky-band math). Exempt from require-rustdoc/max-lines
// because the file name ends in `_tests.rs`. The fixture tree mirrors the shared model test
// `a_later_sibling_is_pushed_below_a_grown_subtree`: a root with children a (row 0), b (row 1),
// c (row 3), where b's subtree holds grandchildren x (row 1) and y (row 2).

use file_manager::types::PaneId;

use crate::band::{Band, band_for, overlap_count, positions, root_pinned, row_y, sticky_y};
use crate::band::Placement;

const PANE_HEIGHT: f64 = 520.0;
const STRIDE: f64 = 532.0;

fn place(id: u64, column: usize, row: usize, parent: Option<u64>) -> Placement {
    return Placement {
        id: PaneId(id),
        column,
        row,
        parent: parent.map(PaneId),
    }
}

/// The grown tree:
///  root(0) -> a(1),
///  b(2),
///  c(3);
///  b -> x(4),
///  y(5).
fn grown_tree() -> Vec<Placement> {
    return vec![
        place(0, 0, 0, None),
        place(1, 1, 0, Some(0)),
        place(2, 1, 1, Some(0)),
        place(3, 1, 3, Some(0)),
        place(4, 2, 1, Some(2)),
        place(5, 2, 2, Some(2)),
    ]
}

#[test]
fn rows_tile_at_the_fixed_stride() {
    assert_eq!(row_y(0), 0.0);
    assert_eq!(row_y(2), 2.0 * STRIDE);
}

#[test]
fn leaf_band_is_one_pane_tall_and_never_travels() {
    let tree = grown_tree();
    let leaf = tree[1];
    let band = band_for(leaf, &tree);
    assert_eq!(band, Band { top: 0.0, height: PANE_HEIGHT });
    assert_eq!(sticky_y(band, 0.0), 0.0);
    assert_eq!(sticky_y(band, 10_000.0), 0.0);
}

#[test]
fn parent_band_stretches_to_deepest_direct_child_bottom() {
    let tree = grown_tree();
    // Root's direct children sit at rows 0, 1, 3: band spans rows 0..3 plus one pane height.
    let band = band_for(tree[0], &tree);
    assert_eq!(band, Band { top: 0.0, height: 3.0 * STRIDE + PANE_HEIGHT });
}

#[test]
fn band_ignores_grandchildren() {
    let tree = grown_tree();
    // b's direct children x, y sit at rows 1, 2; c's row 3 must not stretch b's band.
    let band = band_for(tree[2], &tree);
    assert_eq!(band, Band { top: STRIDE, height: STRIDE + PANE_HEIGHT });
}

#[test]
fn sticky_pins_then_releases_at_band_end() {
    let tree = grown_tree();
    let band = band_for(tree[0], &tree);
    // Before the band is reached the pane sits at its grid position.
    assert_eq!(sticky_y(band, 0.0), 0.0);
    // While scrolled inside the band the pane rides the viewport top exactly.
    assert_eq!(sticky_y(band, 700.0), 700.0);
    // Past the band's travel the pane stops at the band end.
    let travel = band.height - PANE_HEIGHT;
    assert_eq!(sticky_y(band, travel + 999.0), travel);
}

#[test]
fn no_pane_pair_overlaps_at_any_scroll() {
    let tree = grown_tree();
    // Sweep the whole content height in coarse steps; the invariant is structural, so every
    // sample must report zero intersecting pairs.
    let mut scroll = 0.0;
    while scroll < 6.0 * STRIDE {
        assert_eq!(overlap_count(&tree, scroll), 0, "overlap at scroll {scroll}");
        scroll += 37.0;
    }
}

#[test]
fn root_pin_reports_only_while_scrolled_inside_the_root_band() {
    let tree = grown_tree();
    assert!(!root_pinned(&tree, 0.0));
    assert!(root_pinned(&tree, 400.0));
    // Root band travel is 3 rows; past it the root scrolls off and is no longer pinned.
    assert!(!root_pinned(&tree, 3.0 * STRIDE + 100.0));
}

#[test]
fn positions_use_the_global_column_grid() {
    let tree = grown_tree();
    let resolved = positions(&tree, 0.0);
    let (_, x, y) = resolved
        .iter()
        .find(|(id, _, _)| return *id == PaneId(4))
        .copied()
        .expect("grandchild x resolved");
    assert_eq!(x, 2.0 * 332.0);
    assert_eq!(y, STRIDE);
}
