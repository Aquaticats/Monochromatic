//! Rendered pixels:
//!  divider states by weight and ink,
//!  and tree painting inside the narrowest sidebar.

/// Real key events through the window,
///  shared with the find tests.
use super::find_tests::key;
/// Shared window fixture,
///  pointer helpers,
///  and the pinned layout measurements.
use super::sidebar_tests::{HEADER, MINIMUM, drag_to, fixture, motion, press, release, settle};
/// Generated window and tree row types from the shipped markup.
use super::{AppWindow, ui::TreeEntry};
/// What:
///  `Rgba8Pixel` is one pixel of four bytes;
///  `SharedPixelBuffer<Rgba8Pixel>` is a rendered frame
/// (the `<...>` names the element type,
///  like `Array<Pixel>`).
/// Why:
///  Line weight,
///  ink,
///  and painting outside the sidebar exist only in rendered pixels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Frame = { width: number; pixels: Pixel[] };
/// ```
use slint::{
    ComponentHandle, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, VecModel,
    platform::{Key, PointerEventButton},
};
/// The replacement tree model is shared with the window like the fixture's own model.
use std::rc::Rc;

/// Pixel row used for divider state checks,
///  inside the tree rows and the source lines,
/// above the handle that keyboard focus draws in the middle of the line.
const ROW: usize = 260;
/// Pixel row through the middle of that handle,
///  which is 96px tall and centered in the 660px window,
/// so it spans rows 282 to 377.
const HANDLE_ROW: usize = 330;
/// Rows 3px inside the handle's ends,
///  clear of its rounded corners.
const HANDLE_ENDS: [usize; 2] = [285, 374];
/// Rows 3px outside the handle's ends,
///  where the focused line is three columns again.
const BEYOND_HANDLE: [usize; 2] = [279, 380];

/// What:
///  `&SharedPixelBuffer<Rgba8Pixel>` lends the frame;
///  the four `usize` bounds are whole pixels
/// (`usize` is the index type,
///  siblings `u32` and `i32`);
///  the answer is `true` when any pixel inside
/// the bounds differs from `background`.
/// Why:
///  Text and badges are "something painted here";
///  an untouched region is "nothing painted here".
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
    // What: `as usize` converts the frame's `u32` width to the index type; `as_slice()` borrows all pixels.
    // Why: Pixel (x, y) lives at index `y * width + x`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const width = frame.width; const pixels = frame.pixels;
    // ```
    let width = frame.width() as usize;
    let pixels = frame.as_slice();
    for y in bounds[2]..bounds[3] {
        for x in bounds[0]..bounds[1] {
            if pixels[y * width + x] != background {
                return true;
            }
        }
    }
    return false;
}

/// One pixel of the frame at whole-pixel coordinates.
pub(super) fn pixel(frame: &SharedPixelBuffer<Rgba8Pixel>, x: usize, y: usize) -> Rgba8Pixel {
    return frame.as_slice()[y * frame.width() as usize + x];
}

/// What:
///  The answer is a growable array (`Vec<usize>`,
///  sibling fixed array `[usize; N]`) of the pixel
/// columns from 248 up to 264 on `row` that differ from `background`.
/// Why:
///  The divider's line weight is the number of adjacent painted columns.
///  At the default width the
/// line is column 256,
///  so this span holds the five-column zone and five untouched columns on each side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function columns(frame: Frame, row: number, background: Pixel): number[];
/// ```
fn columns(
    frame: &SharedPixelBuffer<Rgba8Pixel>,
    row: usize,
    background: Rgba8Pixel,
) -> Vec<usize> {
    // What: `Vec::new()` is an empty growable array; `mut` allows pushing into it.
    // Why: The number of painted columns is the measurement.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found: number[] = [];
    // ```
    let mut found = Vec::new();
    for x in 248..264 {
        if pixel(frame, x, row) != background {
            found.push(x);
        }
    }
    return found;
}

/// Render the window and return the frame;
///  the headless window renders one pixel per logical pixel.
pub(super) fn frame(window: &AppWindow) -> SharedPixelBuffer<Rgba8Pixel> {
    settle(window);
    // What: `expect` returns the rendered frame or fails the test with this message.
    // Why: Pixel assertions need an actual frame.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const frame = window.takeSnapshot();
    // ```
    let rendered = window.window().take_snapshot().expect("sidebar frame");
    assert_eq!(
        rendered.width(),
        1100,
        "pixel checks assume one pixel per logical pixel"
    );
    return rendered;
}

/// Idle is one faint column;
///  hover and drag are three columns in stronger ink;
///  keyboard focus is three
/// columns in another color with a five-column,
///  96px handle in the middle of the line.
#[test]
fn divider_states_change_line_weight_and_ink() {
    let shared = fixture(6);
    // What: `&shared.window` borrows the window out of the fixture record.
    // Why: The helpers take a borrowed window, and the fixture keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = shared.window;
    // ```
    let window = &shared.window;
    // At the default width the line is pixel column 256 and its pointer zone spans columns 254 to 258.
    motion(window, 800.0, 500.0);
    let idle = frame(window);
    let background = pixel(&idle, 250, ROW);
    assert_eq!(
        columns(&idle, ROW, background),
        [256],
        "the idle divider is not a single pixel column"
    );
    let idle_ink = pixel(&idle, 256, ROW);
    motion(window, 256.5, ROW as f32);
    let hovered = frame(window);
    assert_eq!(
        columns(&hovered, ROW, background),
        [255, 256, 257],
        "hover did not widen the line to three columns"
    );
    let hover_ink = pixel(&hovered, 256, ROW);
    assert_ne!(hover_ink, idle_ink, "hover did not change the line's ink");
    press(window, 256.5, ROW as f32, PointerEventButton::Left);
    let dragged = frame(window);
    assert_eq!(
        columns(&dragged, ROW, background),
        [255, 256, 257],
        "a held press did not keep the line at three columns"
    );
    let drag_ink = pixel(&dragged, 256, ROW);
    assert_ne!(
        drag_ink, hover_ink,
        "a held press did not strengthen the line's ink beyond hover"
    );
    assert_eq!(
        columns(&dragged, HANDLE_ROW, background),
        [255, 256, 257],
        "a drag drew the keyboard-focus handle"
    );
    release(window, 256.5, ROW as f32, PointerEventButton::Left);
    motion(window, 800.0, 500.0);
    assert_eq!(
        columns(&frame(window), ROW, background),
        [256],
        "leaving the divider did not return it to idle"
    );
    window.invoke_focus_tree();
    key(window, Key::Tab);
    assert!(
        window.get_sidebar_divider_has_focus(),
        "keyboard focus positive control"
    );
    let focused = frame(window);
    assert_eq!(
        columns(&focused, ROW, background),
        [255, 256, 257],
        "keyboard focus did not paint a three-column line"
    );
    let focus_ink = pixel(&focused, 256, ROW);
    assert!(
        focus_ink != drag_ink && focus_ink != hover_ink && focus_ink != idle_ink,
        "keyboard focus does not have a line color of its own"
    );
    assert_eq!(
        columns(&focused, HANDLE_ROW, background),
        [254, 255, 256, 257, 258],
        "keyboard focus did not draw its handle across the five-column zone"
    );
    assert_eq!(
        pixel(&focused, 256, HANDLE_ROW),
        focus_ink,
        "the keyboard-focus handle does not use the focus color"
    );
    // What: `for row in HANDLE_ENDS` walks the fixed array of two row numbers.
    // Why: The handle reaches both ends of its 96px and no further.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const row of HANDLE_ENDS) { ... }
    // ```
    for row in HANDLE_ENDS {
        assert_eq!(
            columns(&focused, row, background),
            [254, 255, 256, 257, 258],
            "the keyboard-focus handle does not reach row {row}, so it is shorter than 96px"
        );
    }
    for row in BEYOND_HANDLE {
        assert_eq!(
            columns(&focused, row, background),
            [255, 256, 257],
            "the keyboard-focus handle reaches row {row}, so it is longer than 96px"
        );
    }
    key(window, Key::Tab);
    assert_eq!(
        columns(&frame(window), HANDLE_ROW, background),
        [256],
        "leaving the divider by Tab did not return it to idle"
    );
    window.hide().expect("close sidebar window");
}

/// At the narrowest sidebar a long name and a slot badge still paint,
///  and nothing paints over the divider.
#[test]
fn long_names_and_slot_badges_stay_inside_the_narrowest_sidebar() {
    let shared = fixture(6);
    let window = &shared.window;
    // What: `vec![...]` builds an array; `..TreeEntry::default()` fills the unnamed fields;
    // `SharedString::from` converts text to Slint's string type.
    // Why: One deep row with a slot badge and a name far wider than the sidebar is the worst case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rows = [{ ...defaultEntry, label: longName, depth: 2, recency: '3' }];
    // ```
    let rows = vec![TreeEntry {
        label: SharedString::from(
            "an-extremely-long-file-name-that-cannot-fit-inside-a-narrow-sidebar.txt",
        ),
        depth: 2,
        recency: SharedString::from("3"),
        ..TreeEntry::default()
    }];
    window.set_tree_entries(ModelRc::from(Rc::new(VecModel::from(rows))));
    drag_to(window, MINIMUM);
    // Leave the divider so its hover emphasis does not change the line's width.
    motion(window, 800.0, 500.0);
    let rendered = frame(window);
    // The window background, sampled in the tree below its only row.
    let background = pixel(&rendered, 150, ROW);
    let top = HEADER as usize;
    let bottom = top + 48;
    // Depth 2 puts the badge column at 40..64 and the name from 64 to 12px before the sidebar edge.
    assert!(
        painted(&rendered, [40, 64, top, bottom], background),
        "the slot badge is not painted at the narrowest sidebar"
    );
    assert!(
        painted(&rendered, [64, 148, top, bottom], background),
        "the long name is not painted at the narrowest sidebar"
    );
    assert!(
        !painted(&rendered, [148, 160, top, bottom], background),
        "the long name painted into the row's trailing padding"
    );
    // The idle line is the single pixel column right after the 160px sidebar; the first line number
    // of the source gutter beside it starts more than 20px further right.
    assert!(
        !painted(&rendered, [161, 180, top, bottom], background),
        "the long name painted past the divider"
    );
    assert!(
        painted(&rendered, [160, 161, top, bottom], background),
        "the divider line is not painted"
    );
    window.hide().expect("close sidebar window");
}
