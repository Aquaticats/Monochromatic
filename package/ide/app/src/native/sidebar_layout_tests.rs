//! Source repaint,
//!  window resizing,
//!  the find bar,
//!  and the search overlay beside a resized sidebar.

/// Shared window fixture,
///  pointer helpers,
///  and the pinned layout measurements.
use super::sidebar_tests::{
    DIVIDER, GUTTER, HEADER, MINIMUM, REACH, SOURCE_MINIMUM, click, drag_to, fixture, motion,
    press, release, resize, settle, text_area,
};
/// Generated window and search row types from the shipped markup,
///  and the single rendering boundary.
use super::{AppWindow, render, ui::SearchEntry};
/// Toolkit models and the left pointer button for a drag that must not resize.
use slint::{ComponentHandle, ModelRc, VecModel, platform::PointerEventButton};
/// What:
///  `Rc` is a shared pointer for one thread (sibling `Arc` works across threads);
///  `Cell` holds a
/// small copied value that can change behind that pointer (sibling `RefCell` for larger values).
/// Why:
///  Overlay callbacks record what they were asked to do while the test keeps reading it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::Cell, rc::Rc};

/// What:
///  `&AppWindow` lends the window;
///  the `bool` answer says whether the source image spans the whole visible source width.
/// Why:
///  A stale raster tile shows as blank source at the right edge after the source column widens.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function covers(window: AppWindow): boolean;
/// ```
fn covers(window: &AppWindow) -> bool {
    return window.get_image_width() >= window.get_viewport_width();
}

/// Shrinking and widening the sidebar repaints the source tile at its new width and keeps hit testing.
#[test]
fn source_relayouts_and_repaints_after_a_sidebar_resize() {
    let shared = fixture(6);
    // What: `&shared.window` borrows the window out of the fixture record.
    // Why: The helpers take a borrowed window, and the fixture keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = shared.window;
    // ```
    let window = &shared.window;
    let initial = text_area(1100.0 - 256.0 - DIVIDER);
    assert_eq!(
        window.get_viewport_width(),
        initial,
        "source viewport positive control"
    );
    drag_to(window, MINIMUM);
    let widened = text_area(1100.0 - MINIMUM - DIVIDER);
    assert_eq!(
        window.get_viewport_width(),
        widened,
        "the source did not take the width the sidebar released"
    );
    // What: `borrow()` reads the shared state without changing it.
    // Why: The raster tile width is a paint input held by native code, not a window property.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const tile = source.current.width;
    // ```
    // The wrap width follows the text width; the worker builds the tile for it and the picture is swapped in.
    super::overscan_tile_tests::settle_tile(&shared.source);
    assert_eq!(
        shared.source.borrow().shown.width,
        widened - super::viewport::CARET_ROOM,
        "the raster tile kept the previous source width"
    );
    assert!(
        covers(window),
        "the source image does not span the widened source view"
    );
    // The text origin is right of the sidebar, the divider, and the line-number gutter.
    let origin = MINIMUM + DIVIDER + GUTTER;
    click(window, origin + 92.0, HEADER + 4.0 * 24.0 + 12.0);
    let state = shared.source.borrow();
    let head = state.document.position().head;
    assert_eq!(
        state.document.text().char_to_line(head),
        4,
        "a source click after the resize landed on another line"
    );
    drop(state);
    let caret = window.get_caret_x();
    assert!(
        (caret - 92.0).abs() <= 6.0,
        "a source click 92px into its line placed the caret at {caret}px after the resize"
    );
    drag_to(window, 500.0);
    assert_eq!(
        window.get_viewport_width(),
        text_area(1100.0 - 500.0 - DIVIDER),
        "the source did not give up the width the sidebar took"
    );
    // The wrap width follows the new text width: the worker builds the tile for it and the picture is swapped in.
    super::overscan_tile_tests::settle_tile(&shared.source);
    assert!(
        covers(window),
        "the source image does not span the scrolled source view after the resize"
    );
    window.hide().expect("close sidebar window");
}

/// A narrow window shrinks the shown sidebar,
///  keeps the request,
///  and never goes below the minimum.
#[test]
fn window_resize_shrinks_the_sidebar_and_restores_the_request() {
    let shared = fixture(6);
    let window = &shared.window;
    // What: `repeat(60)` builds one long owned string; `.into()` converts it to Slint's string type.
    // Why: The file label's own preferred width is far wider than the window.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.fileLabel = 'long-directory-name/'.repeat(60);
    // ```
    window.set_file_label("long-directory-name/".repeat(60).into());
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        256.0,
        "a long file label narrowed the sidebar"
    );
    // What: `borrow_mut()` lends the shared state for one change; `Some(text)` stores a present diagnostic.
    // Why: A long diagnostic under the source is the widest content the source column can prefer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // source.current.fileError = 'unreadable '.repeat(200);
    // ```
    shared.source.borrow_mut().file_error = Some("unreadable ".repeat(200));
    render(window, &shared.source);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        256.0,
        "a long source diagnostic narrowed the sidebar"
    );
    shared.source.borrow_mut().file_error = None;
    render(window, &shared.source);
    settle(window);
    let widest = 1100.0 - DIVIDER - SOURCE_MINIMUM;
    drag_to(window, widest);
    assert_eq!(window.get_sidebar_width(), widest, "widest sidebar control");
    resize(window, 480.0, 320.0);
    let narrow = 480.0 - DIVIDER - SOURCE_MINIMUM;
    assert_eq!(
        window.get_sidebar_width(),
        narrow,
        "the sidebar did not shrink to keep the source minimum"
    );
    assert_eq!(
        window.get_viewport_width(),
        text_area(SOURCE_MINIMUM),
        "the source lost its minimum width in a narrow window"
    );
    assert_eq!(
        window.get_sidebar_requested_width(),
        widest,
        "a narrow window discarded the requested width"
    );
    shared.activated.set(-1);
    click(window, narrow - REACH - 1.0, HEADER + 24.0);
    assert_eq!(
        shared.activated.get(),
        0,
        "the last tree pixel before the divider's zone did not take its click in a narrow window"
    );
    resize(window, 1100.0, 660.0);
    assert_eq!(
        window.get_sidebar_width(),
        widest,
        "widening the window did not restore the requested width"
    );
    // Below the declared 480px minimum window width the sidebar minimum wins over the source minimum.
    resize(window, 400.0, 320.0);
    assert_eq!(
        window.get_sidebar_width(),
        MINIMUM,
        "the sidebar shrank below its minimum"
    );
    assert!(
        window.get_sidebar_maximum_width() >= MINIMUM,
        "the reported maximum fell below the minimum"
    );
    resize(window, 480.0, 320.0);
    drag_to(window, narrow - 16.0);
    assert_eq!(
        window.get_sidebar_requested_width(),
        narrow - 16.0,
        "a drag in a narrow window did not start from the shown width"
    );
    resize(window, 1100.0, 660.0);
    assert_eq!(
        window.get_sidebar_width(),
        narrow - 16.0,
        "the width chosen in a narrow window was not kept"
    );
    window.hide().expect("close sidebar window");
}

/// The find bar stays inside the source column and the search overlay stays centered above the divider.
#[test]
fn find_bar_and_search_overlay_keep_their_layout_beside_a_resized_sidebar() {
    let shared = fixture(6);
    let window = &shared.window;
    window.set_find_open(true);
    resize(window, 480.0, 320.0);
    let narrow = 480.0 - DIVIDER - SOURCE_MINIMUM;
    assert_eq!(window.get_sidebar_width(), narrow, "narrow sidebar control");
    // The 56px find bar is the bottom of the source column; its input starts 12px inside the column.
    let bar = 320.0 - 28.0;
    click(window, narrow + DIVIDER / 2.0, bar);
    click(window, narrow + DIVIDER + REACH, bar);
    assert!(
        !window.get_find_has_focus(),
        "the find input reaches the divider or the bar's leading padding"
    );
    click(window, narrow + DIVIDER + 12.0 + 20.0, bar);
    assert!(
        window.get_find_has_focus(),
        "the find input is not at the left edge of the narrowest source column"
    );
    window.set_find_open(false);
    resize(window, 1100.0, 660.0);
    let widest = 1100.0 - DIVIDER - SOURCE_MINIMUM;
    drag_to(window, widest);
    // What: `vec![...]` builds an array; `.into()` converts each string literal to Slint's string type.
    // Why: One result row is enough to prove where the overlay panel is.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const rows = [{ path: 'result.txt', detail: 'File', preview: '' }];
    // ```
    let rows = vec![SearchEntry {
        path: "result.txt".into(),
        detail: "File".into(),
        preview: "".into(),
    }];
    window.set_search_entries(ModelRc::from(Rc::new(VecModel::from(rows))));
    let chosen = Rc::new(Cell::new(-1));
    let dismissed = Rc::new(Cell::new(false));
    // What: `Rc::clone` copies the pointer; `move |index|` is a callback owning that copy.
    // Why: The callbacks outlive this statement and record what the overlay asked for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.onSearchChoose(index => { chosen.current = index; });
    // ```
    let choose_observer = Rc::clone(&chosen);
    window.on_search_choose(move |index| {
        choose_observer.set(index);
    });
    let dismiss_observer = Rc::clone(&dismissed);
    window.on_search_dismiss(move || {
        dismiss_observer.set(true);
    });
    window.set_search_open(true);
    settle(window);
    // The same point as the search pointer test: the first result row of the window-centered panel.
    click(window, 500.0, 186.0);
    assert_eq!(
        chosen.get(),
        0,
        "the search overlay is not centered on the window beside a wide sidebar"
    );
    // The divider line above the panel: the overlay takes the press as an outside click.
    let line = widest + DIVIDER / 2.0;
    press(window, line, 20.0, PointerEventButton::Left);
    motion(window, line - 200.0, 20.0);
    release(window, line - 200.0, 20.0, PointerEventButton::Left);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        widest,
        "the divider was dragged through the open search overlay"
    );
    assert!(
        dismissed.get(),
        "an outside click over the divider did not dismiss the search overlay"
    );
    window.hide().expect("close sidebar window");
}
