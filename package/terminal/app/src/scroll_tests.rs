// What:     Unit tests for `scroll.rs`, pulled in by
//           `#[cfg(test)] #[path = "scroll_tests.rs"] mod tests;` at
//           the bottom of `scroll.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of scroll.
// Why:      Keep the tests beside the code without inflating
//           `scroll.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports every public item from the parent module.
//           The `*` glob is local to tests, not production API.
// Why:      Test names can call `map_pixel_scroll` directly.
//
// In TS you'd write (pseudocode):
// ```ts
// import { mapPixelScroll } from "./scroll";
// ```
use super::*;

// What:     `#[test]` marks this function as a unit test.
// Why:      Cargo runs it under the package `test` mise task.
//
// In TS you'd write (pseudocode):
// ```ts
// test("keeps fractional pixels", () => {});
// ```
#[test]
fn keeps_fractional_pixels() {
    // What:     `let mapping = ...` stores the returned struct.
    // Why:      The assertions inspect every field.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const mapping = mapPixelScroll(47, 18, 20);
    // ```
    let mapping = map_pixel_scroll(47.0, 18.0, 20);
    assert_eq!(mapping.whole_row_offset, 2);
    assert_eq!(mapping.fractional_px, 11.0);
}

// What:     `#[test]` marks another unit test.
// Why:      Top overscroll must clamp to the first scrollback row.
//
// In TS you'd write (pseudocode):
// ```ts
// test("clamps negative pixels", () => {});
// ```
#[test]
fn clamps_negative_pixels() {
    // What:     `let mapping = ...` stores the clamped result.
    // Why:      Negative Slint offsets can appear during overscroll.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const mapping = mapPixelScroll(-5, 18, 20);
    // ```
    let mapping = map_pixel_scroll(-5.0, 18.0, 20);
    assert_eq!(mapping.whole_row_offset, 0);
    assert_eq!(mapping.fractional_px, 0.0);
}

// What:     `#[test]` marks the bottom-clamp unit test.
// Why:      Resize can leave Slint's old pixel offset beyond the new content.
//
// In TS you'd write (pseudocode):
// ```ts
// test("clamps past bottom", () => {});
// ```
#[test]
fn clamps_past_bottom() {
    // What:     `let mapping = ...` stores a result clamped to five rows.
    // Why:      The maximum row offset is authoritative over raw pixels.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const mapping = mapPixelScroll(500, 18, 5);
    // ```
    let mapping = map_pixel_scroll(500.0, 18.0, 5);
    assert_eq!(mapping.pixel_scroll, 90.0);
    assert_eq!(mapping.whole_row_offset, 5);
    assert_eq!(mapping.fractional_px, 0.0);
}
