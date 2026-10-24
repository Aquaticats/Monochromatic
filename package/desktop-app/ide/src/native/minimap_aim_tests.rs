//! A press on the minimap is aimed again,
//! once for each change of the layout,
//! while nothing else has moved the view.
//!
//! The press lands where the layout of that moment puts it,
//! and the hints of lines never asked about arrive later
//!  and take rows,
//! so the same press would land elsewhere a moment after.
//!  These tests give a window a long text,
//! press the minimap,
//! and then let hints arrive:
//!  the view goes to where the same press lands in the new layout when the reader did nothing in between,
//!  and keeps its place,
//! kept still across the new rows,
//! after any other input.

/// The window the tests drive.
use super::AppWindow;
/// The production setter of annotation snapshots and the stamp of the displayed text.
use super::annotate::{displayed, set_annotations};
/// The scroll rule's quiet time passes in real time.
use super::annotation_stability_tests::idle;
/// The test hint.
use super::annotation_tests::hint;
/// The reader fixture,
///  its key press,
///  and the external replacement of a file.
use super::find_tests::{Reader, key, replace_file};
/// Pointer helpers shared with the minimap tests.
use super::minimap_tests::{calm, geometry, lift, offset, over, press, reading};
/// A scroll the toolkit reports to the change handler.
use super::overscan_tile_tests::scroll_to;
/// The rule that keeps the view still across new rows,
///  and the scroll limit.
use super::rows;
/// The bounded wait for an external change.
use super::waiting_tests::Waiting;
/// Opening a long plain text and waiting for its first layout.
use super::wrap_tests::{open, settled};
/// Hint snapshots as the Language module builds them.
use ide_app::language::hints::HintsSnapshot;
/// The vertical mapping.
use ide_app::row_map::RowMap;
/// Toolkit events and sizes.
use slint::{
    ComponentHandle, LogicalSize,
    platform::{Key, update_timers_and_animations},
};
/// Shared snapshot pointers.
use std::sync::Arc;

/// Lines of the fixture text.
const LINES: usize = 1500;

/// Hints of the first answer after a press:
/// on the first lines only.
const FIRST: usize = 400;

/// Hints of a later answer:
/// on more lines,
/// so the layout grows again.
const LATER: usize = 900;

/// Hints of an answer after that.
const LAST: usize = 1300;

/// The text:
/// `LINES` numbered lines of one length each.
fn numbered() -> String {
    let mut text = String::new();
    for index in 0..LINES {
        text.push_str(&format!("line {index:04}: value\n"));
    }
    return text;
}

/// What:
///  Install a snapshot with one hint on each of the first `count` lines,
///  as an answer that asked for the whole text.
/// Why:
///  Every hint takes one block of rows above its line,
/// so the layout grows with `count`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hintFirst(reader: Reader, count: number): void;
/// ```
fn hint_first(reader: &Reader, count: usize) {
    let (stamp, hints) = {
        let current = reader.source.borrow();
        let mut hints = Vec::new();
        for line in 0..count {
            hints.push(hint(
                current.document.text().line_to_char(line) + 8,
                ": u32",
            ));
        }
        (displayed(&current), hints)
    };
    let snapshot = Arc::new(HintsSnapshot {
        stamp,
        first_line: 0,
        last_line: LINES,
        hints,
    });
    set_annotations(&reader.window, &reader.source, Some(snapshot), None);
    update_timers_and_animations();
}

/// The mapping the window holds now.
fn map(reader: &Reader) -> RowMap {
    return reader.source.borrow().row_map.clone();
}

/// What:
///  The first line and the scroll offset a press at `y` gives in the layout the window holds now,
///  when the view stood at `share` of the scroll range before the press.
/// Why:
///  The expected value comes from the library's geometry alone,
///  so it holds the window's mapping against an independent computation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pressedAt(reader: Reader, y: number, share: number): [line: number, offset: number];
/// ```
fn pressed_at(reader: &Reader, y: f32, share: f32) -> (usize, f32) {
    let shape = geometry(reader);
    let current = reader.source.borrow();
    let found = shape.target(&current.row_map, y, share * shape.max_offset());
    let top = current.row_map.code_top(found.line) + found.row as f32 * 24.0;
    return (found.line, shape.centered(top + 12.0).round());
}

/// The share of the scroll range the view stands at.
fn share(reader: &Reader) -> f32 {
    return offset(reader) / geometry(reader).max_offset();
}

/// The offset the rule that keeps the view still gives for a view at `at` when the layout changes from `before`.
fn kept(reader: &Reader, before: &RowMap, at: f32) -> f32 {
    let current = reader.source.borrow();
    return rows::anchored(before, &current.row_map, at).min(rows::limit(
        &current.row_map,
        reader.window.get_viewport_height(),
    ));
}

/// The line the caret stands on.
fn caret_line(reader: &Reader) -> usize {
    let current = reader.source.borrow();
    return current
        .document
        .text()
        .char_to_line(current.document.position().head);
}

/// A place on the minimap near `wanted` that is outside the viewport box,
///  where a press jumps instead of grabbing the box.
fn outside_the_box(window: &AppWindow, wanted: f32) -> f32 {
    let top = window.get_minimap_box_top();
    let bottom = top + window.get_minimap_box_height();
    if wanted > top - 4.0 && wanted < bottom + 4.0 {
        return bottom + 30.0;
    }
    return wanted;
}

/// What:
///  A reader on the numbered text,
///  scrolled a quarter of the way down.
///  The pair is the fixture directory and the reader.
/// Why:
///  A view that stands away from the start makes the share of the scroll range matter to the mapping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scene(): [TempDir, Reader];
/// ```
fn scene() -> (tempfile::TempDir, Reader) {
    let (project, opened) = open(&numbered());
    settled(&opened);
    let shape = geometry(&opened);
    scroll_to(&opened.window, (shape.max_offset() / 4.0).round());
    calm(&opened);
    // The scroll above counts as the reader's, and rows wait for the quiet time after it.
    idle();
    return (project, opened);
}

/// Press at `y` and release;
///  the answer is the share of the scroll range the view stood at before the press.
fn click(reader: &Reader, y: f32) -> f32 {
    let from = share(reader);
    press(reader, over(&reader.window, 10.0, y));
    lift(reader, over(&reader.window, 10.0, y));
    calm(reader);
    return from;
}

/// A press well down the minimap,
/// outside the box.
fn low(reader: &Reader) -> f32 {
    return outside_the_box(&reader.window, geometry(reader).track() * 0.8);
}

/// A press up the minimap,
/// outside the box.
fn high(reader: &Reader) -> f32 {
    return outside_the_box(&reader.window, geometry(reader).track() * 0.3);
}

/// What:
///  Press at `y`,
/// let hints arrive for `count` lines,
///  and check that the view and the caret went to where the same press lands in the new layout,
///  not to where the view would have been kept still.
///  The answer is the offset the view took.
/// Why:
///  Every test of a press that was followed by another input starts from a press that was aimed again,
///  so it fails without the rule instead of passing for lack of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aimedOnce(reader: Reader, y: number, count: number): number;
/// ```
fn aimed_once(reader: &Reader, y: f32, count: usize) -> f32 {
    let from = click(reader, y);
    let at_press = offset(reader);
    let before = map(reader);
    hint_first(reader, count);
    let (line, expected) = pressed_at(reader, y, from);
    let kept_still = kept(reader, &before, at_press);
    assert_ne!(
        expected, kept_still,
        "the press lands where the view is kept still, so the test cannot tell the rule from its absence"
    );
    assert_eq!(
        offset(reader),
        expected,
        "the view is at the place the same press gives in the new layout"
    );
    assert_eq!(
        caret_line(reader),
        line,
        "and the caret on the line it names there"
    );
    return expected;
}

/// What:
///  The step every test of an input after a press ends with:
///  `count` more lines get hints,
///  and the view stays where the layout change keeps it still,
///  which is not where the press lands in the new layout,
///  with the caret where the input left it.
/// Why:
///  An input after a press means the press has been followed,
///  so the press is no longer aimed again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staysKept(reader: Reader, from: number, y: number, count: number): void;
/// ```
fn stays_kept(reader: &Reader, from: f32, y: f32, count: usize) {
    let at_input = offset(reader);
    let caret = reading(reader);
    let before = map(reader);
    hint_first(reader, count);
    let (_, aim) = pressed_at(reader, y, from);
    let kept_still = kept(reader, &before, at_input);
    assert_ne!(
        aim, kept_still,
        "the press would land where the view is kept still, so the test cannot tell"
    );
    assert_eq!(
        offset(reader),
        kept_still,
        "the view is kept still across the new rows, not aimed at the press"
    );
    assert_eq!(reading(reader), caret, "and the caret stays");
}

/// A press that no input followed goes to the place it names in the layout the next answer gives,
///  and the caret goes to the line it names there.
#[test]
fn a_press_is_aimed_again_when_the_layout_changes() {
    let (_project, opened) = scene();
    let y = low(&opened);
    aimed_once(&opened, y, FIRST);
    opened.window.hide().expect("close minimap window");
}

/// The rule applies once for each change:
///  a second change with no input in between aims the same press again,
///  and the view stays there while the layout does not change.
#[test]
fn a_second_layout_change_aims_the_press_again_until_something_else_happens() {
    let (_project, opened) = scene();
    let y = low(&opened);
    let from = click(&opened, y);
    hint_first(&opened, FIRST);
    let first = offset(&opened);
    hint_first(&opened, LATER);
    let (line, expected) = pressed_at(&opened, y, from);
    assert_ne!(
        expected, first,
        "the second layout puts the press elsewhere"
    );
    assert_eq!(offset(&opened), expected, "the second change aims it again");
    assert_eq!(caret_line(&opened), line);
    // The same layout again changes nothing: the view does not move for an answer with the same hints.
    hint_first(&opened, LATER);
    calm(&opened);
    assert_eq!(offset(&opened), expected, "no change, no step");
    // A key ends it: the caret moves, and the next change keeps the view still.
    key(&opened.window, Key::DownArrow);
    calm(&opened);
    assert_ne!(
        caret_line(&opened),
        line,
        "the arrow key moved the caret off the pressed line"
    );
    stays_kept(&opened, from, y, LAST);
    opened.window.hide().expect("close minimap window");
}

/// A wheel step after the press means the reader followed it by scrolling:
///  the next change of the layout keeps the view still.
#[test]
fn a_layout_change_after_a_wheel_step_keeps_the_view_still() {
    let (_project, opened) = scene();
    aimed_once(&opened, low(&opened), FIRST);
    let y = high(&opened);
    let from = click(&opened, y);
    scroll_to(&opened.window, offset(&opened) + 100.0);
    // Rows above the view wait for the scroll to stop; the quiet time passes first.
    idle();
    stays_kept(&opened, from, y, LATER);
    opened.window.hide().expect("close minimap window");
}

/// A key that moves the caret after the press means the reader went on from it:
///  the next change of the layout keeps the view still,
/// and the caret stays where the key put it.
#[test]
fn a_layout_change_after_a_key_keeps_the_view_still() {
    let (_project, opened) = scene();
    aimed_once(&opened, low(&opened), FIRST);
    let y = high(&opened);
    let from = click(&opened, y);
    let line = caret_line(&opened);
    key(&opened.window, Key::DownArrow);
    calm(&opened);
    assert_eq!(caret_line(&opened), line + 1, "the key moved the caret");
    stays_kept(&opened, from, y, LATER);
    opened.window.hide().expect("close minimap window");
}

/// A resize after the press changes the view the press was mapped for:
///  the next change of the layout keeps the view still.
#[test]
fn a_layout_change_after_a_resize_keeps_the_view_still() {
    let (_project, opened) = scene();
    aimed_once(&opened, low(&opened), FIRST);
    let y = high(&opened);
    let from = click(&opened, y);
    let window = &opened.window;
    let size = window
        .window()
        .size()
        .to_logical(window.window().scale_factor());
    window
        .window()
        .set_size(LogicalSize::new(size.width, size.height - 60.0));
    calm(&opened);
    stays_kept(&opened, from, y, LATER);
    opened.window.hide().expect("close minimap window");
}

/// An edit from outside after the press means another text:
///  the next change of the layout keeps the view still.
#[test]
fn a_layout_change_after_a_reload_keeps_the_view_still() {
    let (project, opened) = scene();
    aimed_once(&opened, low(&opened), FIRST);
    let y = high(&opened);
    let from = click(&opened, y);
    let revision = opened.source.borrow().document.revision();
    // A line near the end changes without changing its length, so every position stays valid.
    let changed = numbered().replace("line 1400: value", "line 1400: other");
    replace_file(&project.path().join("a.txt"), &changed);
    {
        let mut waiting = Waiting::eventually();
        while !(opened.source.borrow().document.revision() > revision) {
            waiting.step("the external change was not reloaded");
        }
    }
    calm(&opened);
    stays_kept(&opened, from, y, LATER);
    opened.window.hide().expect("close minimap window");
}

/// A press that is still held has not been followed:
///  the layout change aims it,
/// and the drag that follows moves the box from the new place,
///  with the live layout,
/// as before.
///  A drag that has moved the view has followed the press:
/// the next change keeps the view still.
#[test]
fn a_drag_follows_the_aimed_press_and_then_the_live_layout() {
    let (_project, opened) = scene();
    let window = &opened.window;
    let y = low(&opened);
    let from = share(&opened);
    press(&opened, over(window, 10.0, y));
    hint_first(&opened, FIRST);
    let (_, aimed) = pressed_at(&opened, y, from);
    assert_eq!(offset(&opened), aimed, "the held press is aimed again");
    let dragged = y - 40.0;
    window.invoke_minimap_drag(dragged);
    update_timers_and_animations();
    let shape = geometry(&opened);
    assert_eq!(
        offset(&opened),
        shape
            .drag_offset(aimed, y, dragged)
            .round()
            .min(shape.max_offset()),
        "the drag moves the box from the aimed place, with the live layout"
    );
    assert_ne!(offset(&opened), aimed, "the drag moved the view");
    stays_kept(&opened, from, y, LATER);
    lift(&opened, over(window, 10.0, dragged));
    window.hide().expect("close minimap window");
}
