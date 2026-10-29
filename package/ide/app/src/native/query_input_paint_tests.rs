//! Rendered pixels of the search box in both color schemes:
//!  placeholder,
//!  focus mark,
//!  selection colors,
//! and the clear control's resting,
//!  hovered,
//!  and pressed states.
//!
//! The search panel is 800px wide in the middle of the 1100px by 660px window,
//!  so its box spans columns
//! 162 to 937 and rows 98 to 145 on whole pixels,
//!  and the clear cell spans columns 890 to 937.

/// Real key events through the window,
///  shared with the find tests.
use super::find_tests::{chord, key, type_text};
/// Bounded waits and tree-row lookup shared with the navigation tests.
use super::navigation_tests::row;
/// The source-empty reader and the double-Shift opener of the search tests.
use super::search_tests::{Reader, open, reader};
/// Rendered frames and single pixels,
///  shared with the sidebar paint tests.
use super::sidebar_paint_tests::{frame, pixel};
/// Pointer helpers that dispatch real window events.
use super::sidebar_tests::{motion, press, release, resize, settle};
/// The scheme switch the desktop-settings watcher makes,
///  and color bytes.
use super::theme_tests::{rgba, switch};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// What:
///  `ColorScheme` is the toolkit's scheme enum (`Unknown`,
///  `Dark`,
///  `Light`),
///  reached through its
/// unstable re-export module.
/// Why:
///  The box copies palette values for both schemes,
///  so every check runs in both.
/// Gotcha:
///  This module is not stable API;
///  a toolkit upgrade can rename it and break only these tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ColorScheme } from 'slint/private';
/// ```
use slint::private_unstable_api::re_exports::ColorScheme;
/// Window ownership,
///  pixel types,
///  key names,
///  and the left pointer button for a held press.
use slint::{
    ComponentHandle, Rgba8Pixel, SharedPixelBuffer,
    platform::{Key, PointerEventButton},
};
/// Fixture files are written into a disposable project directory.
use std::fs;

/// What:
///  `const NAME: usize` is a compile-time whole number (`usize` is the index type;
///  siblings `u32`
/// and `i32`).
/// Why:
///  The box's left edge:
///  the panel starts at column 150 and has 12px of padding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const BOX_LEFT = 162;
/// ```
const BOX_LEFT: usize = 162;
/// The box's top edge:
///  the panel starts at row 50,
///  then 12px padding,
///  the 28px title,
///  and 8px spacing.
const BOX_TOP: usize = 98;
/// The clear cell's left edge while it is shown:
///  the box ends at column 938 and the cell is 48px wide.
const CELL_LEFT: usize = 890;
/// The middle row of the 48px tall box.
const MIDDLE: usize = BOX_TOP + 24;
/// The middle column of the clear cell,
///  where its glyph is.
const CELL_CENTER: usize = CELL_LEFT + 24;

/// What:
///  The answer is a pair:
///  the disposable directory (`TempDir`,
///  deleted when dropped) and the reader.
/// Why:
///  The directory must outlive the reader's workers,
///  so the caller keeps both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function searching(): [TempDir, Reader];
/// ```
fn searching() -> (tempfile::TempDir, Reader) {
    // What: `expect` returns the successful value or fails the test with this message.
    // Why: A fixture that cannot be written is a broken test, not behavior under test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fixture = mkdtempSync(...); writeFileSync(join(fixture, 'only.txt'), 'only');
    // ```
    let fixture = tempfile::tempdir().expect("disposable box project");
    fs::write(fixture.path().join("only.txt"), "only").expect("box fixture");
    let opened = reader(fixture.path());
    resize(&opened.window, 1100.0, 660.0);
    // What: `|| return ...` is a zero-argument arrow function that `wait_until` calls until it holds;
    // `is_some()` asks whether the optional row index is present.
    // Why: The search scope comes from the tree, which lists the project asynchronously.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // waitUntil(() => row(window, 'only.txt') !== undefined);
    // ```
    {
        let mut waiting = Waiting::eventually();
        while !(row(&opened.window, "only.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    open(&opened.window);
    // Park the pointer in the panel's title row, away from the box and the result rows.
    motion(&opened.window, 500.0, 70.0);
    return (fixture, opened);
}

/// What:
///  `&SharedPixelBuffer<Rgba8Pixel>` lends a frame;
///  the four `usize` bounds are whole pixels;
///  the
/// answer is `true` when any pixel inside the bounds differs from `background`.
/// Why:
///  Placeholder text is "something drawn here";
///  an empty text area is "nothing drawn here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function painted(frame: Frame, bounds: [number, number, number, number], background: Pixel): boolean;
/// ```
fn painted(
    frame: &SharedPixelBuffer<Rgba8Pixel>,
    bounds: [usize; 4],
    background: Rgba8Pixel,
) -> bool {
    for y in bounds[2]..bounds[3] {
        for x in bounds[0]..bounds[1] {
            if pixel(frame, x, y) != background {
                return true;
            }
        }
    }
    return false;
}

/// The name of a scheme for assertion messages.
fn name(scheme: ColorScheme) -> &'static str {
    if scheme == ColorScheme::Dark {
        return "dark";
    }
    return "light";
}

/// The placeholder shows only while the box is empty,
///  and focus is marked by a line along the bottom
/// edge together with a different fill,
///  in both schemes.
#[test]
fn search_box_shows_its_placeholder_and_marks_focus_by_line_and_fill() {
    // `_fixture` keeps the directory alive until the test ends.
    let (_fixture, reader) = searching();
    // What: `&reader.window` borrows the window out of the reader record.
    // Why: The helpers take a borrowed window, and the reader keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = reader.window;
    // ```
    let window = &reader.window;
    // The placeholder starts 12px inside the box; these columns skip the caret at the text start.
    let words = [BOX_LEFT + 20, BOX_LEFT + 120, BOX_TOP + 12, BOX_TOP + 36];
    for scheme in [ColorScheme::Dark, ColorScheme::Light] {
        switch(window, scheme);
        let empty = frame(window);
        // A column far right of the placeholder and left of the trailing padding is plain fill.
        let fill = pixel(&empty, 800, MIDDLE);
        let line = pixel(&empty, 800, BOX_TOP + 47);
        assert!(
            painted(&empty, words, fill),
            "{}: the empty box does not show its placeholder",
            name(scheme)
        );
        type_text(window, " ");
        assert!(
            !painted(&frame(window), words, fill),
            "{}: the placeholder is still drawn behind text",
            name(scheme)
        );
        key(window, Key::Backspace);
        assert!(
            painted(&frame(window), words, fill),
            "{}: the placeholder did not return to the emptied box",
            name(scheme)
        );
        // Focus moves to the tree behind the overlay; the overlay itself stays open.
        window.invoke_focus_tree();
        let unfocused = frame(window);
        assert_ne!(
            pixel(&unfocused, 800, BOX_TOP + 47),
            line,
            "{}: focus is not marked by a line along the bottom edge",
            name(scheme)
        );
        assert_ne!(
            pixel(&unfocused, 800, MIDDLE),
            fill,
            "{}: focus is not marked by a different fill",
            name(scheme)
        );
        window.invoke_focus_search();
        settle(window);
    }
    window.hide().expect("close placeholder window");
}

/// What:
///  `bounds` is left,
///  right,
///  top,
///  bottom in whole pixels;
///  `background` is the box's own fill;
///  the answer is
/// a pair of 32-bit floats (sibling `f64`):
///  the darkest and the lightest pixel inside the bounds that is not the
/// background,
///  on WCAG's lightness scale from 0 for black to 1 for white.
/// Why:
///  Selected glyphs sit on the selection fill;
///  light ink keeps every such pixel above the fill's 0.4,
/// while dark ink brings the darkest close to 0.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inkRange(frame: Frame, bounds: [number, number, number, number], background: Pixel): [number, number];
/// ```
pub(super) fn ink_range(
    frame: &SharedPixelBuffer<Rgba8Pixel>,
    bounds: [usize; 4],
    background: Rgba8Pixel,
) -> (f32, f32) {
    let mut darkest: f32 = 1.0;
    let mut lightest: f32 = 0.0;
    for y in bounds[2]..bounds[3] {
        for x in bounds[0]..bounds[1] {
            let found = pixel(frame, x, y);
            if found == background {
                continue;
            }
            // What: `f32::from(found.r)` widens a byte to a 32-bit float; the weights are WCAG's.
            // Why: The same lightness scale as the scheme tests.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const value = (0.2126 * found.r + 0.7152 * found.g + 0.0722 * found.b) / 255;
            // ```
            let value = (0.2126 * f32::from(found.r)
                + 0.7152 * f32::from(found.g)
                + 0.0722 * f32::from(found.b))
                / 255.0;
            darkest = darkest.min(value);
            lightest = lightest.max(value);
        }
    }
    // What: `(darkest, lightest)` is a pair, returned as one value.
    // Why: Callers check both ends of the range.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [darkest, lightest];
    // ```
    return (darkest, lightest);
}

/// Selected text has the selection fill behind the ink native code chooses from that fill,
///  the ink of every
/// other selection:
///  light ink in both schemes,
///  where the toolkit box drew dark ink in the dark scheme.
#[test]
fn search_box_selection_uses_the_ink_chosen_from_the_fill_in_both_schemes() {
    let (_fixture, reader) = searching();
    let window = &reader.window;
    type_text(window, "WWWWWW");
    chord(window, Key::Control, "a");
    // Columns inside the first selected characters, away from the caret at the end of the text. The rows are those
    // wholly inside the selection rectangle: it is about 14.5 px tall at a fractional offset, so the GPU renderer
    // anti-aliases its first and last pixel rows (measured on 2026-10-20 as the box background blended with about
    // a quarter of the fill, lightness 0.165 in the dark scheme), and a blend of fill and background is not ink.
    // Rows BOX_TOP + 17 through BOX_TOP + 30 hold every glyph pixel and nothing of the two edge rows.
    let glyphs = [BOX_LEFT + 14, BOX_LEFT + 44, BOX_TOP + 17, BOX_TOP + 31];
    for scheme in [ColorScheme::Dark, ColorScheme::Light] {
        switch(window, scheme);
        let shown = frame(window);
        let fill = rgba(window.get_selection_fill().color());
        let mut filled = false;
        for y in glyphs[2]..glyphs[3] {
            for x in glyphs[0]..glyphs[1] {
                let found = pixel(&shown, x, y);
                if [found.r, found.g, found.b, found.a] == fill {
                    filled = true;
                }
            }
        }
        assert!(
            filled,
            "{}: selected text has no selection fill behind it",
            name(scheme)
        );
        // The fill's lightness is about 0.4; light ink pixels lie above it only. Rows above and below the fill
        // show the box's own background, sampled right of the text, and are left out.
        let (darkest, lightest) = ink_range(&shown, glyphs, pixel(&shown, 800, MIDDLE));
        assert!(
            lightest > 0.7 && darkest > 0.3,
            "{}: selected text is not drawn in light ink: darkest {darkest}, lightest {lightest}",
            name(scheme)
        );
    }
    window.hide().expect("close selection window");
}

/// What:
///  The answer is a growable array (`Vec<usize>`,
///  sibling fixed array `[usize; N]`) of run lengths:
/// how many adjacent pixels on `MIDDLE` share one color,
///  from column 890 up to 904.
/// Why:
///  Those columns cross the plate's boundary at the cell's left edge and the plate's fill left of the
/// glyph.
///  A boundary is a run of its own,
///  and its length is its weight.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runs(frame: Frame): number[];
/// ```
fn runs(frame: &SharedPixelBuffer<Rgba8Pixel>) -> Vec<usize> {
    // What: `Vec::new()` is an empty growable array; `mut` allows pushing into it.
    // Why: Runs are counted with a plain loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const lengths: number[] = [];
    // ```
    let mut lengths = Vec::new();
    let mut previous = pixel(frame, CELL_LEFT, MIDDLE);
    let mut length = 0;
    for x in CELL_LEFT..CELL_LEFT + 14 {
        let current = pixel(frame, x, MIDDLE);
        if current != previous {
            lengths.push(length);
            length = 0;
            previous = current;
        }
        length += 1;
    }
    lengths.push(length);
    return lengths;
}

/// What:
///  `rest` and `marked` lend two frames,
///  before and with the plate;
///  `column` is a column inside the
/// cell;
///  `state` and `scheme` name the case for messages.
/// Why:
///  The plate is translucent:
///  the focus line,
///  drawn after it,
///  keeps its pixels,
///  and the box's border
/// under the plate's top boundary changes that boundary's color,
///  which an opaque boundary would not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function through(rest: Frame, marked: Frame, column: number, state: string, scheme: string): void;
/// ```
fn through(
    rest: &SharedPixelBuffer<Rgba8Pixel>,
    marked: &SharedPixelBuffer<Rgba8Pixel>,
    column: usize,
    state: &str,
    scheme: &str,
) {
    // The focus line is the box's last two rows.
    for y in [BOX_TOP + 46, BOX_TOP + 47] {
        assert_eq!(
            pixel(marked, column, y),
            pixel(rest, column, y),
            "{scheme}: under {state} the focus line in row {y} is not whole"
        );
    }
    // The boundary over the box's top border row against the boundary over plain fill at the cell's left edge.
    assert_ne!(
        pixel(marked, column, BOX_TOP),
        pixel(marked, CELL_LEFT, MIDDLE),
        "{scheme}: under {state} the box's border does not show through the plate's boundary"
    );
}

/// At rest the clear control is its glyph alone.
///  Hover fills the whole 48px cell with a translucent plate and
/// a one-pixel boundary;
///  a press makes the fill stronger and the boundary two pixels wide,
///  in both schemes.
/// The box's border shows through the plate's boundary,
///  and the focus line under the cell stays whole.
#[test]
fn search_clear_control_marks_hover_and_press_by_fill_and_boundary() {
    let (_fixture, reader) = searching();
    let window = &reader.window;
    type_text(window, "needle");
    settle(window);
    assert_eq!(
        window.get_search_clear_x(),
        CELL_LEFT as f32,
        "the clear cell is not where this test samples it"
    );
    // The plate spans the whole cell, columns 890 to 937; the 16px glyph spans columns 906 to 921.
    let inside = CELL_LEFT + 6;
    // A column of the focus line under the cell, and the box's top border row inside the cell.
    let under = CELL_LEFT + 10;
    for scheme in [ColorScheme::Dark, ColorScheme::Light] {
        switch(window, scheme);
        motion(window, 500.0, 70.0);
        let rest = frame(window);
        assert_eq!(
            runs(&rest),
            [14],
            "{}: the resting clear control draws more than its glyph",
            name(scheme)
        );
        assert!(
            painted(
                &rest,
                [CELL_CENTER - 8, CELL_CENTER + 8, MIDDLE - 8, MIDDLE + 8],
                pixel(&rest, inside, MIDDLE)
            ),
            "{}: the clear control's glyph is not drawn",
            name(scheme)
        );
        motion(window, CELL_CENTER as f32, MIDDLE as f32);
        let hovered = frame(window);
        assert_eq!(
            runs(&hovered),
            [1, 13],
            "{}: hover did not fill the whole cell with a plate and a one-pixel boundary",
            name(scheme)
        );
        through(&rest, &hovered, under, "hover", name(scheme));
        assert_ne!(
            pixel(&hovered, inside, MIDDLE),
            pixel(&rest, inside, MIDDLE),
            "{}: hover did not fill the plate",
            name(scheme)
        );
        press(
            window,
            CELL_CENTER as f32,
            MIDDLE as f32,
            PointerEventButton::Left,
        );
        let pressed = frame(window);
        assert_eq!(
            runs(&pressed),
            [2, 12],
            "{}: a press did not widen the plate's boundary to two pixels",
            name(scheme)
        );
        through(&rest, &pressed, under, "a press", name(scheme));
        assert_ne!(
            pixel(&pressed, inside, MIDDLE),
            pixel(&hovered, inside, MIDDLE),
            "{}: a press did not strengthen the plate's fill",
            name(scheme)
        );
        // Releasing outside the cell cancels the click, so the text stays for the next scheme.
        motion(window, 500.0, 70.0);
        release(window, 500.0, 70.0, PointerEventButton::Left);
        settle(window);
        assert_eq!(
            window.get_search_query(),
            "needle",
            "a release outside the clear cell cleared the query"
        );
    }
    window.hide().expect("close clear-state window");
}
