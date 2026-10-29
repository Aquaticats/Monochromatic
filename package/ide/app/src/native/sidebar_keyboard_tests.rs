//! The divider is a Tab stop between tree and source,
//!  adjusts by keys,
//!  and does not exist without a project.

/// Real key events through the window,
///  shared with the find tests.
use super::find_tests::{chord, key};
/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Shared window fixture,
///  pointer helpers,
///  and the pinned layout measurements.
use super::sidebar_tests::{
    DIVIDER, HEADER, MINIMUM, SOURCE_MINIMUM, click, fixture, resize, settle, text_area,
};
/// The shipped window,
///  source state,
///  and bindings for a window without a project.
use super::{AppWindow, State, bind_keys, bind_pointer, bind_viewport, render};
/// Toolkit key names and window ownership.
use slint::{ComponentHandle, platform::Key};
/// What:
///  `Rc` is a shared pointer for one thread (sibling `Arc` works across threads);
///  `RefCell` allows
/// changing a value behind that pointer (sibling `Cell` for small copied values).
/// Why:
///  The shortcut callback records the forwarded key while the test keeps reading it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{cell::RefCell, rc::Rc};

/// Tab order is tree,
///  divider,
///  source;
///  arrows step by 16px;
///  Home and End go to the bounds.
#[test]
fn keyboard_reaches_and_adjusts_the_divider_between_tree_and_source() {
    let shared = fixture(6);
    // What: `&shared.window` borrows the window out of the fixture record.
    // Why: The helpers take a borrowed window, and the fixture keeps owning it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = shared.window;
    // ```
    let window = &shared.window;
    // What: `Rc::new(RefCell::new(String::new()))` is a shared, changeable, initially empty string;
    // `move |...|` is a callback owning a copy of the pointer, and `*x.borrow_mut() = v` replaces the value.
    // Why: Ctrl+digit pressed on the divider must reach recent-file navigation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const shortcut = { current: '' };
    // window.onTreeShortcut((text, control) => { if (control) shortcut.current = text; });
    // ```
    let shortcut = Rc::new(RefCell::new(String::new()));
    let observer = Rc::clone(&shortcut);
    window.on_tree_shortcut(move |text, control, _shift, _alt| {
        if control {
            *observer.borrow_mut() = text.to_string();
        }
    });
    let widest = 1100.0 - DIVIDER - SOURCE_MINIMUM;
    window.invoke_focus_tree();
    assert!(window.get_tree_has_focus(), "tree focus positive control");
    key(window, Key::Tab);
    assert!(
        window.get_sidebar_divider_has_focus(),
        "Tab from the tree did not reach the divider"
    );
    assert!(!window.get_tree_has_focus());
    key(window, Key::RightArrow);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        272.0,
        "Right did not widen the sidebar by one step"
    );
    key(window, Key::LeftArrow);
    key(window, Key::LeftArrow);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        240.0,
        "Left did not narrow the sidebar by one step each"
    );
    key(window, Key::Home);
    key(window, Key::LeftArrow);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        MINIMUM,
        "Home did not go to the minimum, or Left went below it"
    );
    key(window, Key::End);
    key(window, Key::RightArrow);
    settle(window);
    assert_eq!(
        window.get_sidebar_width(),
        widest,
        "End did not go to the maximum, or Right went above it"
    );
    assert_eq!(window.get_sidebar_requested_width(), widest);
    chord(window, Key::Control, "3");
    assert_eq!(
        shortcut.borrow().as_str(),
        "3",
        "Ctrl+digit on the divider did not reach recent-file navigation"
    );
    key(window, Key::Tab);
    assert!(
        !window.get_sidebar_divider_has_focus(),
        "Tab did not leave the divider"
    );
    let before = shared.source.borrow().document.position().head;
    key(window, Key::RightArrow);
    settle(window);
    assert_eq!(
        shared.source.borrow().document.position().head,
        before + 1,
        "Tab from the divider did not reach the source view"
    );
    assert_eq!(
        window.get_sidebar_width(),
        widest,
        "a source key resized the sidebar"
    );
    chord(window, Key::Shift, Key::Tab);
    assert!(
        window.get_sidebar_divider_has_focus(),
        "Shift+Tab from the source did not reach the divider"
    );
    chord(window, Key::Shift, Key::Tab);
    assert!(
        window.get_tree_has_focus(),
        "Shift+Tab from the divider did not reach the tree"
    );
    window.hide().expect("close sidebar window");
}

/// Without a project the divider has no width,
///  takes no clicks,
///  and is not a Tab stop.
#[test]
fn hidden_project_has_no_divider() {
    // What: `expect` returns the window or fails the test with this message.
    // Why: A window that cannot be created is a broken fixture, not behavior under test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const window = new AppWindow();
    // ```
    install_gpu_platform();
    let window = AppWindow::new().expect("native window without a project");
    let text = "first line\nsecond line\nthird line\n";
    // `None` means the source has no file on disk.
    let source = Rc::new(RefCell::new(State::new(text, None)));
    bind_pointer(&window, &source);
    bind_viewport(&window, &source);
    bind_keys(&window, &source);
    window.show().expect("show window without a project");
    resize(&window, 1100.0, 660.0);
    render(&window, &source);
    settle(&window);
    assert_eq!(
        window.get_sidebar_width(),
        0.0,
        "a hidden project kept sidebar width"
    );
    assert_eq!(
        window.get_viewport_width(),
        text_area(1100.0),
        "a hidden divider kept width beside the source"
    );
    // The window's first pixel column is source gutter: the click places the caret on the third line.
    click(&window, 0.0, HEADER + 2.0 * 24.0 + 12.0);
    assert_eq!(
        source.borrow().document.position().head,
        text.find("third").expect("third line"),
        "the first window pixel is not source without a project"
    );
    window.invoke_focus_source();
    for _ in 0..4 {
        key(&window, Key::Tab);
        assert!(
            !window.get_sidebar_divider_has_focus(),
            "Tab reached the divider of a hidden project"
        );
    }
    for _ in 0..4 {
        chord(&window, Key::Shift, Key::Tab);
        assert!(
            !window.get_sidebar_divider_has_focus(),
            "Shift+Tab reached the divider of a hidden project"
        );
    }
    window.hide().expect("close window without a project");
}
