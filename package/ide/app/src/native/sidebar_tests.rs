//! Real pointer press,
//!  move,
//!  and release on the sidebar divider:
//!  resize,
//!  both bounds,
//!  and the drag origin.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// The shipped window and source state,
///  with the bindings that repaint source when its width changes.
use super::{
    AppWindow, State, bind_appearance, bind_keys, bind_pointer, bind_viewport, render,
    ui::TreeEntry,
};
/// Window events reach the same items as seat input delivered by a windowing backend.
use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, ModelRc, SharedString, VecModel,
    platform::{PointerEventButton, WindowEvent, update_timers_and_animations},
};
/// What:
///  `Rc` is a shared pointer for one thread (sibling `Arc` works across threads);
///  `RefCell` and
/// `Cell` allow changing a value behind a shared pointer,
///  `Cell` for small copied values such as a number.
/// Why:
///  The window callbacks and the test body both need the same source state and recorded row index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// What:
///  `pub(super) const NAME: f32` is a compile-time constant visible to sibling test modules;
/// `f32` is a 32-bit float (sibling `f64`),
///  the type Slint uses for logical pixels.
/// Why:
///  The narrowest sidebar the layout allows is a documented value the tests pin.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const MINIMUM = 160;
/// ```
pub(super) const MINIMUM: f32 = 160.0;
/// Width of the divider's own layout cell,
///  which is its line.
pub(super) const DIVIDER: f32 = 1.0;
/// Pixel columns on each side of the line that also belong to the divider's pointer zone.
pub(super) const REACH: f32 = 2.0;
/// Narrowest source column,
/// the gutter,
/// the minimap,
/// and 182 px of text;
///  the widest sidebar is the window width minus this and the divider.
pub(super) const SOURCE_MINIMUM: f32 = 288.0;
/// Width of the line-number gutter at the left edge of the source column.
pub(super) const GUTTER: f32 = 58.0;

/// What:
///  The width of the minimap beside a text area `text` logical pixels wide in a source column that is also
///  as wide as the gutter and the minimap together need.
///  The minimap is a sixth of its column,
/// so the answer is the fixed point of that rule.
/// Why:
///  Tests ask for a text area of an exact width;
/// the window has to be that wide and the minimap wider.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function minimapBeside(text: number): number;
/// ```
pub(super) fn minimap_beside(text: f32) -> f32 {
    let mut minimap = ide_app::minimap::WIDTH;
    for _ in 0..8 {
        minimap = ide_app::minimap::width_for(text + GUTTER + minimap);
    }
    return minimap;
}

/// Width of the text area in a source column `source` logical pixels wide:
///  what the gutter and the minimap leave.
pub(super) fn text_area(source: f32) -> f32 {
    return source - GUTTER - ide_app::minimap::width_for(source);
}
/// Width of the tree column every fixture starts with.
pub(super) const TREE: f32 = 256.0;
/// What:
///  Where source text starts in the window:
///  right of the tree,
///  the divider,
///  and the gutter,
///  whose 58 px for
///       files of fewer than 1000 lines are 6 px,
///  a 9 px letter cell,
///  a 4 px gap,
///  three 9 px digits,
///  and 12 px.
/// Why:
///  Pointer tests aim at text positions;
///  one derivation keeps them right when any of the three changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const TEXT_LEFT = TREE + DIVIDER + GUTTER;
/// ```
pub(super) const TEXT_LEFT: f32 = TREE + DIVIDER + GUTTER;
/// Height of the project and file label rows above the tree and the source.
pub(super) const HEADER: f32 = 32.0;
/// Vertical position used for divider drags,
///  inside the tree rows and the source lines.
pub(super) const DRAG_Y: f32 = 300.0;

/// What:
///  A record owning the window,
///  the shared source state,
///  and the last activated tree row.
/// Why:
///  Every sidebar test drives one window and then reads these three things.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Fixture = { window: AppWindow; source: Shared<State>; activated: Shared<number> };
/// ```
pub(super) struct Fixture {
    /// Native window built from the shipped markup.
    pub(super) window: AppWindow,
    /// Source state shared with the pointer,
    ///  key,
    ///  and viewport bindings.
    pub(super) source: Rc<RefCell<State>>,
    /// Model index of the last tree row a click activated,
    ///  or -1.
    pub(super) activated: Rc<Cell<i32>>,
}

/// What:
///  `usize` is an unsigned index-sized integer (siblings `u32`,
///  `i32`);
///  `ModelRc<TreeEntry>` is the
/// shared list type the window accepts for tree rows.
/// Why:
///  Tests choose a row count that does or does not overflow the tree,
///  which decides whether the
/// tree shows its scrollbar beside the divider.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entries(count: number): TreeEntry[];
/// ```
fn entries(count: usize) -> ModelRc<TreeEntry> {
    // What: `Vec::new()` is an empty growable array; `mut` allows pushing into it.
    // Why: Rows are built one at a time with a plain loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rows: TreeEntry[] = [];
    // ```
    let mut rows = Vec::new();
    for index in 0..count {
        // What: `..TreeEntry::default()` fills every field not named here with its default value;
        // `format!` builds an owned string and `SharedString::from` converts it to Slint's string type.
        // Why: Only the label matters for these rows.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // rows.push({ ...defaultEntry, label: `source-${index}.txt` });
        // ```
        rows.push(TreeEntry {
            label: SharedString::from(format!("source-{index}.txt")),
            ..TreeEntry::default()
        });
    }
    // What: `VecModel::from` wraps the array as a Slint model, `Rc::new` shares it, `ModelRc::from` erases
    // the concrete model type.
    // Why: The window property accepts only the erased shared model.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return rows;
    // ```
    return ModelRc::from(Rc::new(VecModel::from(rows)));
}

/// What:
///  `&AppWindow` lends the window without giving it away;
///  the two `f32` values are logical pixels.
/// Why:
///  Tests set an exact window size before measuring layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resize(window: AppWindow, width: number, height: number): void;
/// ```
pub(super) fn resize(window: &AppWindow, width: f32, height: f32) {
    // What: `WindowEvent::Resized { size }` is one variant of an event union, built with a named field.
    // Why: Dispatching the event is how a windowing backend reports a new window size.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.dispatchEvent({ kind: 'resized', size: { width, height } });
    // ```
    window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(width, height),
    });
    settle(window);
}

/// Run queued change handlers,
///  force layout through a rendered frame,
///  then run handlers the frame queued.
pub(super) fn settle(window: &AppWindow) {
    update_timers_and_animations();
    // What: `expect` returns the successful value or fails the test with this message.
    // Why: A frame that cannot render is a broken fixture, not behavior under test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.takeSnapshot(); // throws on failure
    // ```
    window
        .window()
        .take_snapshot()
        .expect("settled sidebar layout");
    update_timers_and_animations();
}

/// Press one pointer button at a window position.
pub(super) fn press(window: &AppWindow, x: f32, y: f32, button: PointerEventButton) {
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: LogicalPosition::new(x, y),
        button,
    });
}

/// Move the pointer to a window position;
///  a held button turns this into a drag.
pub(super) fn motion(window: &AppWindow, x: f32, y: f32) {
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(x, y),
    });
}

/// Release one pointer button at a window position.
pub(super) fn release(window: &AppWindow, x: f32, y: f32, button: PointerEventButton) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: LogicalPosition::new(x, y),
            button,
        });
}

/// Press and release the left button at one position,
///  then let the window settle.
pub(super) fn click(window: &AppWindow, x: f32, y: f32) {
    press(window, x, y, PointerEventButton::Left);
    release(window, x, y, PointerEventButton::Left);
    settle(window);
}

/// Drag the divider's line with the left button until the sidebar is `target` wide.
pub(super) fn drag_to(window: &AppWindow, target: f32) {
    // The middle of the 1px line, which is the pixel column right after the sidebar.
    let line = window.get_sidebar_width() + DIVIDER / 2.0;
    let end = line + target - window.get_sidebar_width();
    press(window, line, DRAG_Y, PointerEventButton::Left);
    motion(window, end, DRAG_Y);
    release(window, end, DRAG_Y, PointerEventButton::Left);
    settle(window);
}

/// A 1100 by 660 window with a visible project of `rows` files and a displayed source of short lines.
pub(super) fn fixture(rows: usize) -> Fixture {
    // What: `String::new()` is an empty owned string; `push_str(&line)` appends a borrowed string.
    // Why: 120 lines overflow the source view vertically; 20 columns fit the narrowest source column
    //      (240 px), so no line wraps and line `n` stands on row `n` at every width.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = `${'0123456789'.repeat(2)}\n`.repeat(120);
    // ```
    let line = "0123456789".repeat(2);
    let mut text = String::new();
    for _ in 0..120 {
        text.push_str(&line);
        text.push('\n');
    }
    install_gpu_platform();
    let window = AppWindow::new().expect("native sidebar window");
    // What: `Rc::new(RefCell::new(value))` puts the state behind a shared, checked-mutable pointer;
    // `None` means the source has no file on disk.
    // Why: Production bindings take the state in exactly this form.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const source = { current: new State(text, undefined) };
    // ```
    let source = Rc::new(RefCell::new(State::new(&text, None)));
    window.set_project_visible(true);
    window.set_tree_entries(entries(rows));
    let activated = Rc::new(Cell::new(-1));
    // What: `Rc::clone` copies the pointer, not the number; `move |index|` is a callback that takes
    // ownership of that copy.
    // Why: The callback outlives this function and records which row a click activated.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.onTreeActivate(index => { activated.current = index; });
    // ```
    let observer = Rc::clone(&activated);
    window.on_tree_activate(move |index| {
        observer.set(index);
    });
    bind_pointer(&window, &source);
    bind_viewport(&window, &source);
    bind_keys(&window, &source);
    bind_appearance(&window, &source);
    window.show().expect("show sidebar window");
    resize(&window, 1100.0, 660.0);
    render(&window, &source);
    settle(&window);
    // The layout gave the view its width; a tile for it may still be on the worker.
    super::overscan_tile_tests::settle_tile(&source);
    return Fixture {
        window,
        source,
        activated,
    };
}

/// A drag follows the pointer from its press point,
///  stops at both bounds,
///  and stores no width outside them.
#[test]
fn divider_drag_resizes_and_stops_at_both_bounds() {
    let shared = fixture(60);
    // What: `&shared.window` borrows the window out of the fixture record.
    // Why: The helpers take a borrowed window, and the fixture keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = shared.window;
    // ```
    let window = &shared.window;
    assert_eq!(
        window.get_sidebar_width(),
        256.0,
        "the sidebar must start at its default width"
    );
    let line = 256.0 + DIVIDER / 2.0;
    let maximum = 1100.0 - DIVIDER - SOURCE_MINIMUM;
    press(window, line, DRAG_Y, PointerEventButton::Left);
    motion(window, line + 100.0, DRAG_Y);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        356.0,
        "a 100px drag must widen the sidebar by 100px"
    );
    motion(window, 5000.0, DRAG_Y);
    settle(window);
    assert_eq!(
        window.get_sidebar_requested_width(),
        maximum,
        "a drag stored a width above the maximum"
    );
    assert_eq!(
        window.get_sidebar_width(),
        maximum,
        "the sidebar must stop where the source keeps its minimum width"
    );
    motion(window, line + 50.0, DRAG_Y);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        306.0,
        "returning from beyond a bound must follow the press point again"
    );
    motion(window, -4000.0, DRAG_Y);
    settle(window);
    assert_eq!(
        window.get_sidebar_requested_width(),
        MINIMUM,
        "a drag stored a width below the minimum"
    );
    assert_eq!(
        window.get_sidebar_width(),
        MINIMUM,
        "the sidebar must stop at its minimum"
    );
    release(window, -4000.0, DRAG_Y, PointerEventButton::Left);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        MINIMUM,
        "releasing the button must keep the dragged width"
    );
    window.hide().expect("close sidebar window");
}

/// A click or double-click without movement,
///  a right-button drag,
///  and a drag while the tree has focus
/// change nothing else.
#[test]
fn divider_ignores_plain_clicks_and_other_buttons_and_keeps_keyboard_focus() {
    let shared = fixture(60);
    let window = &shared.window;
    window.invoke_focus_tree();
    assert!(window.get_tree_has_focus(), "tree focus positive control");
    let line = 256.0 + DIVIDER / 2.0;
    let before = shared.source.borrow().document.position();
    click(window, line, DRAG_Y);
    assert_eq!(
        window.get_sidebar_width(),
        256.0,
        "a click without movement resized the sidebar"
    );
    // Slint counts a second press within 500 ms and 10 px of the first as a repeat, so two presses
    // with no frame between them are a double-click (`i-slint-core` 1.18.1 `input.rs` `check_repeat`).
    press(window, line, DRAG_Y, PointerEventButton::Left);
    release(window, line, DRAG_Y, PointerEventButton::Left);
    press(window, line, DRAG_Y, PointerEventButton::Left);
    release(window, line, DRAG_Y, PointerEventButton::Left);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        256.0,
        "a double-click on the divider resized the sidebar"
    );
    assert_eq!(
        shared.activated.get(),
        -1,
        "a click on the divider activated a tree row"
    );
    assert_eq!(
        shared.source.borrow().document.position(),
        before,
        "a click on the divider moved the source caret"
    );
    press(window, line, DRAG_Y, PointerEventButton::Right);
    motion(window, line + 100.0, DRAG_Y);
    release(window, line + 100.0, DRAG_Y, PointerEventButton::Right);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        256.0,
        "a right-button drag resized the sidebar"
    );
    drag_to(window, 216.0);
    assert_eq!(
        window.get_sidebar_width(),
        216.0,
        "left-button drag positive control"
    );
    assert!(
        window.get_tree_has_focus(),
        "dragging the divider took keyboard focus from the tree"
    );
    assert!(
        !window.get_sidebar_divider_has_focus(),
        "a pointer press focused the divider"
    );
    window.hide().expect("close sidebar window");
}

/// What:
///  `window: &AppWindow` lends the window;
///  `start` is the logical x of the press in window pixels.
/// Why:
///  A short left-button drag by 10px either resizes the sidebar by 10px or leaves it alone,
/// which tells whether the pressed pixel column belongs to the divider.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shortDrag(window: AppWindow, start: number): void;
/// ```
fn short_drag(window: &AppWindow, start: f32) {
    press(window, start, DRAG_Y, PointerEventButton::Left);
    motion(window, start + 10.0, DRAG_Y);
    release(window, start + 10.0, DRAG_Y, PointerEventButton::Left);
    settle(window);
}

/// The pointer zone is the line's column and two columns on each side:
///  a drag starts on each of the five
/// and on no column beside them.
#[test]
fn divider_zone_takes_drags_on_five_columns_only() {
    let shared = fixture(6);
    let window = &shared.window;
    // What: The array holds the zone's five column offsets from the line's own column, 256.
    // Why: Each of them must start a drag that follows the pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const offset of [-REACH, -1, 0, 1, REACH]) { ... }
    // ```
    for offset in [-REACH, -1.0, 0.0, 1.0, REACH] {
        short_drag(window, 256.0 + offset + 0.5);
        assert_eq!(
            window.get_sidebar_width(),
            266.0,
            "the zone column at offset {offset} did not start a drag"
        );
        drag_to(window, 256.0);
        assert_eq!(
            window.get_sidebar_width(),
            256.0,
            "drag back to the default width"
        );
    }
    // The next column on each side belongs to the tree or to the source.
    for offset in [-REACH - 1.0, DIVIDER + REACH] {
        short_drag(window, 256.0 + offset + 0.5);
        assert_eq!(
            window.get_sidebar_width(),
            256.0,
            "a drag from offset {offset}, outside the zone, resized the sidebar"
        );
    }
    window.hide().expect("close sidebar window");
}
