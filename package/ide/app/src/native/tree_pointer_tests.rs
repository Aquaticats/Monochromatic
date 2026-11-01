//! Reusing fixed-window row components must not redirect a pointer press to a different file.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Use the actual native project component and its activation callback.
use super::{AppWindow, ui::TreeEntry};
/// Public pointer events exercise the real TouchArea press/release lifecycle.
use slint::{
    ComponentHandle, LogicalPosition, ModelRc, SharedString, VecModel,
    platform::{PointerEventButton, WindowEvent},
};
/// A shared scalar records which model row was activated without mutating real files.
use std::{cell::Cell, rc::Rc};

/// Sixty tree entries labeled `<prefix>-0.txt` to `<prefix>-59.txt`.
pub(super) fn sixty_entries(prefix: &str) -> Vec<TreeEntry> {
    let mut entries: Vec<TreeEntry> = Vec::new();
    for index in 0..60 {
        entries.push(TreeEntry {
            label: SharedString::from(format!("{prefix}-{index}.txt")),
            ..TreeEntry::default()
        });
    }
    return entries;
}

/// A scroll during a held click cannot turn the release into activation of a newly rebound row.
#[test]
fn windowed_row_rebinding_cancels_an_in_progress_click() {
    install_gpu_platform();
    let window = AppWindow::new().expect("native tree window");
    window.set_project_visible(true);
    window.set_source_available(false);
    let entries = sixty_entries("source");
    window.set_tree_entries(ModelRc::from(Rc::new(VecModel::from(entries))));
    let activated = Rc::new(Cell::new(-1));
    let callback_state = Rc::clone(&activated);
    window.on_tree_activate(move |index| {
        callback_state.set(index);
    });
    window.show().expect("show native tree");
    window
        .window()
        .take_snapshot()
        .expect("initial tree layout");
    window.invoke_reveal_tree(20);
    window
        .window()
        .take_snapshot()
        .expect("first scrolled window");
    let point = LogicalPosition::new(120.0, window.get_tree_scroll_y() + 9.0 * 48.0 + 32.0 + 24.0);
    // Positive control proves these coordinates hit the original model row.
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert_eq!(activated.get(), 9);
    activated.set(-1);
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    window.invoke_reveal_tree(43);
    window
        .window()
        .take_snapshot()
        .expect("rebound scrolled window");
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert_eq!(
        activated.get(),
        -1,
        "pointer release activated a different row after window rebinding"
    );
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    let replacement = sixty_entries("replacement");
    window.set_tree_entries(ModelRc::from(Rc::new(VecModel::from(replacement))));
    window
        .window()
        .take_snapshot()
        .expect("replaced row identities");
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert_eq!(
        activated.get(),
        -1,
        "model replacement redirected a held click at the same row index"
    );
    // The guard must still admit an ordinary new click after replacement.
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    let expected = ((point.y - 32.0 - window.get_tree_scroll_y()) / 48.0).floor() as i32;
    assert_eq!(activated.get(), expected);
    window.hide().expect("close native tree");
}
