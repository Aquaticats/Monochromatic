//! The find box keeps the toolkit text box's editing behavior:
//!  editing keys,
//!  the context menu,
//!  and
//! scrolling of a text wider than the box.

/// The generated window type from the shipped markup.
use super::AppWindow;
/// An opened find bar over a disposable two-match file,
///  with keyboard focus in the find box.
use super::find_clear_tests::opened;
/// Real key events through the window and the wait for the find count,
///  shared with the find tests.
use super::find_tests::{chord, key, status, type_text};
/// The range of ink lightness inside a region,
///  shared with the search box's paint tests.
use super::query_input_paint_tests::ink_range;
/// Rendered frames and single pixels,
///  shared with the sidebar paint tests.
use super::sidebar_paint_tests::{frame, pixel};
/// Pointer helpers and the pinned width of the divider's line.
use super::sidebar_tests::{DIVIDER, press, release, settle};
/// The scheme switch the desktop-settings watcher makes,
///  and color bytes.
use super::theme_tests::{rgba, switch};
/// What:
///  `ColorScheme` is the toolkit's scheme enum (`Unknown`,
///  `Dark`,
///  `Light`),
///  reached through its unstable
/// re-export module.
/// Why:
///  The selection ink is checked in both schemes.
/// Gotcha:
///  This module is not stable API;
///  a toolkit upgrade can rename it and break only these tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ColorScheme } from 'slint/private';
/// ```
use slint::private_unstable_api::re_exports::{ColorScheme, WindowInner};
use slint::{
    ComponentHandle, Rgba8Pixel, SharedPixelBuffer,
    platform::{Key, PointerEventButton, WindowEvent},
};
/// What:
///  `Rgba8Pixel` is one pixel of four bytes;
///  `SharedPixelBuffer<Rgba8Pixel>` is a rendered frame
/// (the `<...>` names the element type,
///  like `Array<Pixel>`);
///  `WindowEvent` is the union of events a
/// windowing backend reports.
/// Why:
///  Scrolling of the find text exists only in rendered pixels,
///  and a held modifier is two events.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Frame = { width: number; pixels: Pixel[] };
/// ```
/// What:
///  `sleep` blocks the calling thread for a span;
///  `Duration` is a span of time (sibling `Instant` is a point in time).
/// Why:
///  A right click is held for a chosen span,
///  so the time between press and release is an input,
///  not host load.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { setTimeout as sleep } from 'node:timers/promises';
/// ```
use std::{thread::sleep, time::Duration};

/// What:
///  `steps: usize` is how many times Down is pressed (`usize` is the unsigned counting type;
/// siblings `u32` and `i32`).
/// Why:
///  The menu opens by a right click on the find text and is driven by keys like the toolkit's:
/// Down moves through Undo,
///  Redo,
///  Cut,
///  Copy,
///  Paste,
///  and Select All,
///  skipping the separator,
///  and Return
/// runs the highlighted entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function menu(window: AppWindow, steps: number): void;
/// ```
fn menu(window: &AppWindow, steps: usize) {
    open_menu(window, Duration::ZERO);
    for _ in 0..steps {
        key(window, Key::DownArrow);
    }
    key(window, Key::Return);
    settle(window);
}

/// What:
///  `hold` is a `Duration`,
///  a span of time (sibling `Instant` is a point in time).
/// Why:
///  A menu entry of the toolkit runs on a pointer release that comes more than 300 ms after the entry was created
///  (`widgets/common/menu-base.slint` in `i-slint-compiler`,
///  the `up` branch of `pointer-event`).
///  The hold makes the time between press and release a chosen input,
///  where it used to be the load the host happened to be under.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function openMenu(window: AppWindow, holdMs: number): void;
/// ```
fn open_menu(window: &AppWindow, hold: Duration) {
    settle(window);
    // The pointer goes on the left part of the text area:
    //  the box starts after the sidebar and its divider,
    //  and the text area is 12px of padding in.
    // The find bar sits at the bottom of the window,
    //  so `place_popup` (`window/popup.rs` in `i-slint-core`) clamps the menu up into the window.
    // At the left the clamp leaves the menu's x at the pointer's x,
    //  so the pointer rests on the menu's left frame, outside every entry, and a slow release runs nothing.
    // Near the clear cell the clamp moved the menu left instead,
    //  the pointer rested on the last entries,
    //  and a release 300 ms after the menu appeared ran the entry under it and closed the menu.
    let x = window.get_sidebar_width() + DIVIDER + 36.0;
    let y = window.get_find_clear_y() + 24.0;
    let popups_before = open_popups(window);
    press(window, x, y, PointerEventButton::Right);
    let popups_pressed = open_popups(window);
    // What: `sleep` blocks this thread for the span; Why: the pointer stays down while the clock runs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await new Promise(resolve => setTimeout(resolve, holdMs));
    // ```
    sleep(hold);
    release(window, x, y, PointerEventButton::Right);
    let popups_released = open_popups(window);
    settle(window);
    let popups_settled = open_popups(window);
    assert_eq!(
        popups_settled,
        1,
        "the right click at {x},{y} left no menu open (popups before the press {popups_before}, after it {popups_pressed}, \
         after the release {popups_released}, after settling {popups_settled}; find status {:?}, find focus {})",
        window.get_find_status(),
        window.get_find_has_focus()
    );
}

/// How many popups (menus,
///  tooltips) the window holds open.
fn open_popups(window: &AppWindow) -> usize {
    return WindowInner::from_pub(window.window()).active_popups().len();
}

/// Entry positions in the menu,
///  counted in Down presses from the opened menu.
const UNDO: usize = 1;
/// Redo is the second entry.
const REDO: usize = 2;
/// Cut is the first entry after the separator,
///  which Down skips.
const CUT: usize = 3;
/// Copy follows Cut.
const COPY: usize = 4;
/// Paste follows Copy.
const PASTE: usize = 5;
/// Select All is the last entry.
const SELECT_ALL: usize = 6;

/// Home,
///  End,
///  Backspace,
///  Delete,
///  Shift selection,
///  select-all,
///  copy,
///  cut,
///  paste,
///  undo,
///  and redo edit the find text.
#[test]
fn find_box_keeps_the_toolkit_editing_keys() {
    // `_fixture` keeps the directory alive until the test ends.
    let (_fixture, reader) = opened();
    // What: `&reader.window` borrows the window out of the reader record.
    // Why: The helpers take a borrowed window, and the reader keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = reader.window;
    // ```
    let window = &reader.window;
    type_text(window, "ab");
    key(window, Key::Home);
    type_text(window, "c");
    assert_eq!(
        window.get_find_query(),
        "cab",
        "Home did not move the caret to the start of the find text"
    );
    key(window, Key::End);
    key(window, Key::Backspace);
    assert_eq!(
        window.get_find_query(),
        "ca",
        "End and Backspace did not delete the last character"
    );
    chord(window, Key::Control, "a");
    chord(window, Key::Control, "c");
    key(window, Key::End);
    chord(window, Key::Control, "v");
    assert_eq!(
        window.get_find_query(),
        "caca",
        "Ctrl+A, Ctrl+C, End, and Ctrl+V did not append a copy of the find text"
    );
    chord(window, Key::Control, "z");
    assert_eq!(
        window.get_find_query(),
        "ca",
        "Ctrl+Z did not undo the paste"
    );
    // What: `WindowEvent::KeyPressed { text }` is one variant of the event union; `Key::Control.into()`
    // converts the key name to the toolkit's encoded key text.
    // Why: Redo is Ctrl+Shift+Z, two held modifiers around one key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.dispatchEvent({ kind: 'keyPressed', text: Key.Control });
    // ```
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    chord(window, Key::Shift, "Z");
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    assert_eq!(
        window.get_find_query(),
        "caca",
        "Ctrl+Shift+Z did not redo the paste"
    );
    chord(window, Key::Shift, Key::LeftArrow);
    chord(window, Key::Shift, Key::LeftArrow);
    key(window, Key::Delete);
    assert_eq!(
        window.get_find_query(),
        "ca",
        "Shift+Left twice and Delete did not remove the last two characters"
    );
    chord(window, Key::Control, "a");
    chord(window, Key::Control, "x");
    assert_eq!(
        window.get_find_query(),
        "",
        "Ctrl+A and Ctrl+X did not cut the find text"
    );
    chord(window, Key::Control, "v");
    assert_eq!(
        window.get_find_query(),
        "ca",
        "Ctrl+V did not paste the cut find text"
    );
    assert!(
        !window.get_search_open(),
        "editing keys opened the search overlay"
    );
    window.hide().expect("close editing window");
}

/// A right click opens the toolkit's menu,
///  and each of its entries acts on the find text.
#[test]
fn find_box_context_menu_runs_each_entry() {
    let (_fixture, reader) = opened();
    let window = &reader.window;
    type_text(window, "needle");
    menu(window, SELECT_ALL);
    type_text(window, "x");
    assert_eq!(
        window.get_find_query(),
        "x",
        "the menu's Select All did not select the whole find text (find status {:?}, find focus {}, search open {})",
        window.get_find_status(),
        window.get_find_has_focus(),
        window.get_search_open()
    );
    assert!(
        window.get_find_has_focus(),
        "closing the menu did not return keyboard focus to the find box"
    );
    // The toolkit's undo history holds a replaced selection as two steps: its removal and the insertion.
    menu(window, UNDO);
    assert_eq!(
        window.get_find_query(),
        "",
        "the menu's Undo did not take back the typed character"
    );
    menu(window, UNDO);
    assert_eq!(
        window.get_find_query(),
        "needle",
        "the menu's Undo did not restore the replaced find text"
    );
    menu(window, REDO);
    menu(window, REDO);
    assert_eq!(
        window.get_find_query(),
        "x",
        "the menu's Redo did not repeat the replacement"
    );
    menu(window, UNDO);
    menu(window, UNDO);
    menu(window, SELECT_ALL);
    menu(window, COPY);
    key(window, Key::End);
    menu(window, PASTE);
    assert_eq!(
        window.get_find_query(),
        "needleneedle",
        "the menu's Copy and Paste did not append a copy of the find text"
    );
    menu(window, SELECT_ALL);
    menu(window, CUT);
    assert_eq!(
        window.get_find_query(),
        "",
        "the menu's Cut did not remove the selected find text"
    );
    menu(window, PASTE);
    assert_eq!(
        window.get_find_query(),
        "needleneedle",
        "the menu's Paste did not insert the cut find text"
    );
    window.hide().expect("close menu window");
}

/// What:
///  `bounds` is a fixed array of four pixel positions,
///  left,
///  right,
///  top,
///  bottom;
///  the answer is a
/// growable array (`Vec<Rgba8Pixel>`,
///  sibling fixed array `[Rgba8Pixel; N]`) of the pixels inside them.
/// Why:
///  Two frames show the same thing in a region exactly when these arrays are equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pixels(frame: Frame, bounds: [number, number, number, number]): Pixel[];
/// ```
fn pixels(frame: &SharedPixelBuffer<Rgba8Pixel>, bounds: [usize; 4]) -> Vec<Rgba8Pixel> {
    // What: `Vec::new()` is an empty growable array; `mut` allows pushing into it.
    // Why: The region is collected pixel by pixel with plain loops.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const found: Pixel[] = [];
    // ```
    let mut found = Vec::new();
    for y in bounds[2]..bounds[3] {
        for x in bounds[0]..bounds[1] {
            found.push(pixel(frame, x, y));
        }
    }
    return found;
}

/// A text wider than the box scrolls so that its end and the caret stay visible,
///  Home scrolls back to its
/// start,
///  and no part of it is drawn inside the clear cell.
#[test]
fn find_box_scrolls_long_text_and_keeps_it_out_of_the_clear_cell() {
    let (_fixture, reader) = opened();
    let window = &reader.window;
    // The fixture holds neither letter typed here, so the count reads "No matches" throughout
    // and the box keeps its width.
    type_text(window, "M");
    status(window, "No matches");
    settle(window);
    // What: `ceil()` rounds a fractional edge up to the next whole pixel, and `as usize` converts that
    // logical position to a pixel index; the headless window renders one pixel per logical pixel.
    // Why: The box's width depends on the count text beside it, so the cell's left edge can be fractional;
    // the sampled columns must lie inside the cell.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const left = Math.ceil(window.findClearX);
    // ```
    let left = window.get_find_clear_x().ceil() as usize;
    let top = window.get_find_clear_y() as usize;
    let cell = [left, left + 47, top, top + 48];
    let begun = frame(window);
    let short = pixels(&begun, cell);
    // Enough narrow letters to be wider than the box at this window width, which the check on the first
    // letter's columns confirms.
    type_text(window, &"i".repeat(499));
    status(window, "No matches");
    let filled = frame(window);
    assert_eq!(
        window.get_find_clear_x().ceil() as usize,
        left,
        "typing moved the clear cell"
    );
    assert!(
        pixels(&filled, cell) == short,
        "a text wider than the box was drawn inside the clear cell"
    );
    // The text area starts 12px inside the bar and 12px inside the box. These columns lie inside the first
    // letter while the text is at its start, clear of the caret before it and of the letter after it,
    // so they change only when the text scrolls.
    let start = (window.get_sidebar_width() + DIVIDER) as usize + 24;
    let first = [start + 3, start + 10, top + 12, top + 36];
    assert!(
        pixels(&filled, first) != pixels(&begun, first),
        "the typed text did not scroll its first letter away, so it is not wider than the box"
    );
    // The toolkit's scrolling keeps the caret 24px inside the text area while it moves through the text,
    // and lets the end of the text reach the area's edge, which is where the cell starts.
    // The last character lies in the columns before the caret's own two.
    let last = [left - 16, left - 3, top + 12, top + 36];
    type_text(window, "M");
    status(window, "No matches");
    let ended = frame(window);
    assert!(
        pixels(&ended, last) != pixels(&filled, last),
        "the end of a text wider than the box was not scrolled into view"
    );
    key(window, Key::Home);
    let home = frame(window);
    assert!(
        pixels(&home, first) != pixels(&ended, first),
        "Home did not scroll a text wider than the box back to its start"
    );
    assert!(
        pixels(&home, cell) == short,
        "scrolling the text changed the clear cell"
    );
    window.hide().expect("close scrolling window");
}

/// Selected find text is drawn in the ink chosen from the selection fill,
///  light in both schemes,
///  like
/// selected text in the search box,
///  the selected rows,
///  and the source view.
#[test]
fn find_box_selection_uses_the_ink_chosen_from_the_fill() {
    let (_fixture, reader) = opened();
    let window = &reader.window;
    type_text(window, "needle");
    chord(window, Key::Control, "a");
    settle(window);
    // The text area starts 12px inside the bar and 12px inside the box; the box's top is the clear cell's.
    let start = (window.get_sidebar_width() + DIVIDER) as usize + 24;
    let top = window.get_find_clear_y() as usize;
    // The rows are those wholly inside the selection rectangle: it is about 14.5 px tall at a fractional offset,
    // so the GPU renderer anti-aliases its first and last pixel rows (measured on 2026-10-20 as the box background
    // blended with about a quarter of the fill, lightness 0.165 in the dark scheme), and a blend of fill and
    // background is not ink. Rows top + 17 through top + 30 hold every glyph pixel and nothing of the two edge rows.
    let glyphs = [start + 2, start + 40, top + 17, top + 31];
    for (scheme, label) in [(ColorScheme::Dark, "dark"), (ColorScheme::Light, "light")] {
        switch(window, scheme);
        let shown = frame(window);
        let fill = rgba(window.get_selection_fill().color());
        // What: `.iter().any(...)` is true when some pixel of the region passes; `|found|` is an arrow function.
        // Why: The selected text has the selection fill behind it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const filled = pixels(shown, glyphs).some(found => [found.r, found.g, found.b, found.a] === fill);
        // ```
        let mut filled = false;
        for found in pixels(&shown, glyphs).iter() {
            if [found.r, found.g, found.b, found.a] == fill {
                filled = true;
                break;
            }
        }
        assert!(
            filled,
            "{label}: selected find text has no selection fill behind it"
        );
        // The box's own fill far right of the text, left of the count.
        let (darkest, lightest) = ink_range(&shown, glyphs, pixel(&shown, start + 300, top + 24));
        assert!(
            lightest > 0.7 && darkest > 0.3,
            "{label}: selected find text is not drawn in light ink: darkest {darkest}, lightest {lightest}"
        );
    }
    window.hide().expect("close find selection window");
}

/// A right click held longer than the toolkit's activation delay still leaves the menu open.
#[test]
fn a_slow_right_click_leaves_the_find_menu_open() {
    let (_fixture, reader) = opened();
    let window = &reader.window;
    type_text(window, "needle");
    open_menu(window, Duration::from_millis(400));
    window.hide().expect("close slow click window");
}
