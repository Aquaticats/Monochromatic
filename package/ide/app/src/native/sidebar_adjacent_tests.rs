//! Tree rows and source text beside the divider's five-column pointer zone keep their own clicks at every
//! sidebar width,
//!  and the tree scrollbar works up to the zone.

/// Shared window fixture,
///  pointer helpers,
///  and the pinned layout measurements.
use super::sidebar_tests::{
    DIVIDER, HEADER, MINIMUM, REACH, SOURCE_MINIMUM, click, drag_to, fixture, motion, press,
    release, settle,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::Waiting;
/// Window ownership for closing the fixture,
///  the left pointer button for a scrollbar drag,
///  and the
/// wheel event a windowing backend reports.
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{PointerEventButton, WindowEvent},
};

/// The last tree pixel left of the zone activates its row,
///  a click on any of the zone's five columns does
/// nothing,
///  and the first source pixel right of the zone places the caret,
///  at the default,
///  narrowest,
///  and
/// widest sidebar.
#[test]
fn clicks_beside_the_divider_reach_tree_rows_and_source_at_every_width() {
    // Six rows do not overflow, so no tree scrollbar sits between the rows and the divider.
    let shared = fixture(6);
    // What: `&shared.window` borrows the window out of the fixture record.
    // Why: The helpers take a borrowed window, and the fixture keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = shared.window;
    // ```
    let window = &shared.window;
    let widest = 1100.0 - DIVIDER - SOURCE_MINIMUM;
    // What: Each `(row, target)` pair is a two-value tuple taken apart by the loop header.
    // Why: Every width clicks its own tree row and source line, so a result left over from the
    // previous width cannot pass.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [row, target] of [[1, 256], [2, MINIMUM], [3, widest]]) { ... }
    // ```
    for (row, target) in [(1, 256.0), (2, MINIMUM), (3, widest)] {
        drag_to(window, target);
        let width = window.get_sidebar_width();
        assert_eq!(width, target, "the drag did not reach the tested width");
        // What: `row as f32` converts the integer row to a float for pixel arithmetic.
        // Why: Tree rows are 48px tall below the 32px project label.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const y = HEADER + row * 48 + 24;
        // ```
        let y = HEADER + row as f32 * 48.0 + 24.0;
        shared.activated.set(-1);
        click(window, width - REACH - 1.0, y);
        assert_eq!(
            shared.activated.get(),
            row,
            "the last tree pixel before the divider's zone did not activate its row at width {width}"
        );
        shared.activated.set(-1);
        let before = shared.source.borrow().document.position();
        // What: The array holds the five column offsets of the zone, counted from the line's own column.
        // Why: Two tree columns, the line, and two source columns belong to the divider and to nothing else.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // for (const offset of [-REACH, -1, 0, 1, REACH]) click(window, width + offset, y);
        // ```
        for offset in [-REACH, -1.0, 0.0, 1.0, REACH] {
            click(window, width + offset, y);
        }
        assert_eq!(
            shared.activated.get(),
            -1,
            "the divider's zone activated a tree row at width {width}"
        );
        assert_eq!(
            shared.source.borrow().document.position(),
            before,
            "the divider's zone moved the source caret at width {width}"
        );
        assert_eq!(
            window.get_sidebar_width(),
            width,
            "clicks in the divider's zone resized the sidebar"
        );
        // Source lines are 24px tall below the 32px file label; the first source pixels are line-number gutter.
        let line = row as usize + 3;
        click(
            window,
            width + DIVIDER + REACH,
            HEADER + line as f32 * 24.0 + 12.0,
        );
        // What: `borrow()` reads the shared state; `line_to_char` is the character index where a line starts.
        // Why: A gutter click places the caret at the start of the clicked line.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const expected = lineStart(source.current.document, line);
        // ```
        let state = shared.source.borrow();
        let expected = state.document.text().line_to_char(line);
        assert_eq!(
            state.document.position().head,
            expected,
            "the first source pixel after the divider's zone did not place the caret on its line at width {width}"
        );
        drop(state);
    }
    window.hide().expect("close sidebar window");
}

/// Far reveal,
///  row clicks after a reveal,
///  and the tree scrollbar beside the divider work at both bounds.
/// The scrollbar is dragged on its thumb and on the last tree column left of the divider's zone.
#[test]
fn tree_windowing_and_scrollbar_follow_the_sidebar_width() {
    let shared = fixture(60);
    let window = &shared.window;
    for target in [MINIMUM, 1100.0 - DIVIDER - SOURCE_MINIMUM] {
        drag_to(window, target);
        let width = window.get_sidebar_width();
        assert_eq!(width, target, "the drag did not reach the tested width");
        for index in [43, 59, 2] {
            window.invoke_reveal_tree(index);
            settle(window);
            let top = window.get_tree_scroll_y() + index as f32 * 48.0;
            assert!(
                top >= -0.1 && top + 48.0 <= window.get_tree_viewport_height() + 0.1,
                "row {index} is not fully revealed at width {width}: top {top}"
            );
            // The overflowing tree shows a 14px scrollbar at its right edge; rows end left of it.
            shared.activated.set(-1);
            click(window, width - 20.0, HEADER + top + 24.0);
            assert_eq!(
                shared.activated.get(),
                index,
                "a revealed row did not take its click at width {width}"
            );
        }
        window.invoke_reveal_tree(0);
        settle(window);
        // The 14px scrollbar's thumb ends 4px before the tree's right edge; its track continues to the zone.
        for column in [width - 5.0, width - REACH - 1.0] {
            let scrolled = window.get_tree_scroll_y();
            press(window, column, HEADER + 100.0, PointerEventButton::Left);
            motion(window, column, HEADER + 200.0);
            release(window, column, HEADER + 200.0, PointerEventButton::Left);
            settle(window);
            assert!(
                window.get_tree_scroll_y() < scrolled,
                "the tree scrollbar did not scroll from column {column} at width {width}"
            );
            assert_eq!(
                window.get_sidebar_width(),
                width,
                "dragging the tree scrollbar resized the sidebar"
            );
        }
    }
    window.hide().expect("close sidebar window");
}

/// A wheel turn over the zone's tree columns scrolls the tree,
///  and over its source columns the source:
/// the divider takes presses there,
///  not the wheel.
#[test]
fn wheel_over_the_divider_zone_scrolls_what_lies_under_it() {
    let shared = fixture(60);
    let window = &shared.window;
    let tree_before = window.get_tree_scroll_y();
    // What: `WindowEvent::PointerScrolled { .. }` is one variant of the event union, built with named
    // fields; a negative `delta_y` is a wheel turn toward the user, which moves content up.
    // Why: The position is the zone's first column, which lies over the tree's last columns.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // window.dispatchEvent({ kind: 'wheel', position: { x: 254.5, y: 300 }, deltaX: 0, deltaY: -96 });
    // ```
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: LogicalPosition::new(256.0 - REACH + 0.5, 300.0),
            delta_x: 0.0,
            delta_y: -96.0,
        });
    // What: `|| return ...` is a zero-argument arrow function that `eventually` calls until it holds.
    // Why: The toolkit eases a wheel scroll over several frames.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // eventually(message, () => window.treeScrollY < treeBefore);
    // ```
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_tree_scroll_y() < tree_before) {
            waiting.step("a wheel turn over the zone's tree columns did not scroll the tree");
        }
    }
    let source_before = window.get_scroll_y();
    // The zone's last column lies over the source column's first columns.
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: LogicalPosition::new(256.0 + DIVIDER + REACH - 0.5, 300.0),
            delta_x: 0.0,
            delta_y: -96.0,
        });
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_scroll_y() < source_before) {
            waiting.step("a wheel turn over the zone's source columns did not scroll the source");
        }
    }
    assert_eq!(
        window.get_sidebar_width(),
        256.0,
        "a wheel turn over the zone resized the sidebar"
    );
    window.hide().expect("close sidebar window");
}
