//! The sidebar divider as an assistive tool reaches it through Slint's element handles:
//!  a horizontal
//! slider named "Sidebar width" with its value,
//!  bounds,
//!  and step,
//!  whose increment,
//!  decrement,
//!  and
//! set-value actions and arrow keys change the width it reports.

/// Finding exactly one element by its accessible label.
use super::accessible_box_tests::only;
/// One key press and release through the window.
use super::find_tests::key;
/// The 1100 by 660 sidebar fixture,
///  its layout constants,
///  and settled layout.
use super::sidebar_tests::{DIVIDER, SOURCE_MINIMUM, fixture, settle};
/// What:
///  `AccessibleRole` is the toolkit's enum of accessible roles;
///  `Orientation` is its enum of
/// `Horizontal` and `Vertical`;
///  `ElementHandle` is Slint's test-only handle on one element.
/// Why:
///  The role and orientation are what tell an assistive tool that the element adjusts a width.
/// Gotcha:
///  The crate is internal to Slint and must have exactly the resolved Slint version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { AccessibleRole, ElementHandle, Orientation } from 'slint-testing';
/// ```
use i_slint_backend_testing::{AccessibleRole, ElementHandle, Orientation};
/// Window ownership and the toolkit's key names.
use slint::{ComponentHandle, platform::Key};

/// What:
///  `divider: &ElementHandle` lends the handle;
///  `expected: f32` is a width in logical pixels (a 32-bit
/// float;
///  sibling `f64`);
///  `action` names what was just done,
///  for messages.
/// Why:
///  After every action the reported value and the window's own width must agree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assertWidth(window: AppWindow, divider: ElementHandle, expected: number, action: string): void;
/// ```
fn assert_width(window: &super::AppWindow, divider: &ElementHandle, expected: f32, action: &str) {
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        expected,
        "{action} did not set the sidebar width"
    );
    // What: `format!("{expected}")` writes the number as text; a whole float prints without decimals.
    // `.as_deref()` borrows the text out of the `Option` the handle returns.
    // Why: The accessible value is text, and an assistive tool reads it as written.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.equal(divider.accessibleValue, String(expected));
    // ```
    assert_eq!(
        divider.accessible_value().as_deref(),
        Some(format!("{expected}").as_str()),
        "after {action} the slider does not report the sidebar width"
    );
}

/// The divider is a horizontal slider named "Sidebar width" with its value,
///  its bounds,
///  and a 16px step.
/// The increment and decrement actions step it,
///  the set-value action sets it within the bounds,
/// and with keyboard focus the arrow keys,
///  Home,
///  and End change the value it reports.
#[test]
fn divider_is_a_slider_with_value_bounds_step_actions_and_keys() {
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
    let divider = only(window, "Sidebar width");
    // What: `Some(AccessibleRole::Slider)` is the present variant of `Option` holding the slider role.
    // Why: A missing or other role would leave assistive tools without the adjustable-value controls.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // assert.equal(divider.accessibleRole, AccessibleRole.Slider);
    // ```
    assert_eq!(
        divider.accessible_role(),
        Some(AccessibleRole::Slider),
        "the divider is not a slider"
    );
    assert_eq!(
        divider.accessible_orientation(),
        Some(Orientation::Horizontal),
        "the divider slider is not horizontal"
    );
    assert_eq!(
        divider.accessible_value_minimum(),
        Some(160.0),
        "the divider does not report the narrowest sidebar width"
    );
    assert_eq!(
        divider.accessible_value_maximum(),
        Some(widest),
        "the divider does not report the widest sidebar width"
    );
    assert_eq!(
        divider.accessible_value_step(),
        Some(16.0),
        "the divider does not report its 16px step"
    );
    assert_width(window, &divider, 256.0, "opening the window");
    divider.invoke_accessible_increment_action();
    assert_width(window, &divider, 272.0, "the increment action");
    divider.invoke_accessible_decrement_action();
    assert_width(window, &divider, 256.0, "the decrement action");
    divider.set_accessible_value("300");
    assert_width(window, &divider, 300.0, "setting the value to 300");
    divider.set_accessible_value("5000");
    assert_width(
        window,
        &divider,
        widest,
        "setting a value above the widest width",
    );
    // Keyboard focus reaches the divider by Tab from the tree; its keys change the reported value.
    window.invoke_focus_tree();
    key(window, Key::Tab);
    assert!(
        window.get_sidebar_divider_has_focus(),
        "Tab from the tree did not reach the divider"
    );
    key(window, Key::LeftArrow);
    assert_width(window, &divider, widest - 16.0, "Left");
    key(window, Key::Home);
    assert_width(window, &divider, 160.0, "Home");
    key(window, Key::RightArrow);
    assert_width(window, &divider, 176.0, "Right");
    key(window, Key::End);
    assert_width(window, &divider, widest, "End");
    window.hide().expect("close accessible divider window");
}
