// What:     Unit tests for `engine.rs`, pulled in by
//           `#[cfg(test)] #[path = "engine_tests.rs"] mod tests;` at
//           the bottom of `engine.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of engine.
// Why:      Keep the tests beside the code without inflating
//           `engine.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

use super::*;
use crate::scroll::{DEFAULT_CELL_HEIGHT_PX, DEFAULT_CELL_WIDTH_PX};

#[test]
fn extracts_vt_text_and_bold_style() -> Result<(), TerminalError> {
    let geometry = ViewportGeometry {
        cols: 20,
        rows: 4,
        cell_width_px: DEFAULT_CELL_WIDTH_PX,
        cell_height_px: DEFAULT_CELL_HEIGHT_PX,
    };
    let mut engine = TerminalEngine::new(geometry, 100)?;
    engine.feed(b"\x1b[1mBold\x1b[0m plain\r\n")?;
    let mapping = engine.set_pixel_scroll(0.0)?;
    let snapshot = engine.snapshot(mapping)?;
    let bold_cell = snapshot
        .cells
        .iter()
        .find(|cell| return cell.text == "B")
        .expect("rendered bold B cell");
    assert!(bold_cell.bold);
    let plain_cell = snapshot
        .cells
        .iter()
        .find(|cell| return cell.text == "p")
        .expect("rendered plain p cell");
    assert!(!plain_cell.bold);
    return Ok(())
}

#[test]
fn scrolls_to_scrollback_top() -> Result<(), TerminalError> {
    let geometry = ViewportGeometry {
        cols: 12,
        rows: 2,
        cell_width_px: DEFAULT_CELL_WIDTH_PX,
        cell_height_px: DEFAULT_CELL_HEIGHT_PX,
    };
    let mut engine = TerminalEngine::new(geometry, 100)?;
    engine.feed(b"one\r\ntwo\r\nthree\r\nfour\r\n")?;
    let mapping = engine.set_pixel_scroll(0.0)?;
    let snapshot = engine.snapshot(mapping)?;
    let top_text: String = snapshot
        .cells
        .iter()
        .filter(|cell| return cell.row == 0)
        .map(|cell| return cell.text.as_str())
        .collect();
    assert!(top_text.contains("one"));
    return Ok(())
}
