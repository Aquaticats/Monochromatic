//! A held search-result click must not activate a row from a replacement query model.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Use the production overlay and its real generated row type.
use super::{AppWindow, ui::SearchEntry};
/// Native pointer events exercise TouchArea ownership rather than invoking choose directly.
use slint::{
    ComponentHandle, LogicalPosition, LogicalSize, ModelRc, VecModel,
    platform::{PointerEventButton, WindowEvent},
};
/// A shared scalar observes the row chosen by the widget without opening any source file.
use std::{cell::Cell, rc::Rc};

/// Bounded fixture labels differ so replacement is not a no-op model update.
fn model(label: &str) -> ModelRc<SearchEntry> {
    return ModelRc::from(Rc::new(VecModel::from(vec![SearchEntry {
        path: label.into(),
        detail: "File".into(),
        preview: "".into(),
    }])));
}

/// Press belongs to the old model;
///  release cannot acquire a newly bound row at the same index.
#[test]
fn replacement_query_cancels_a_held_result_click() {
    install_gpu_platform();
    let window = AppWindow::new().expect("native search pointer window");
    window.set_source_available(false);
    window.set_search_open(true);
    window.set_search_entries(model("old.txt"));
    let chosen = Rc::new(Cell::new(-1));
    let observer = Rc::clone(&chosen);
    window.on_search_choose(move |index| {
        observer.set(index);
    });
    window.show().expect("show pointer fixture");
    window.window().dispatch_event(WindowEvent::Resized {
        size: LogicalSize::new(1100.0, 660.0),
    });
    window.window().take_snapshot().expect("overlay layout");
    // Positive control proves this point reaches the first 64px result row in the actual layout.
    let point = LogicalPosition::new(500.0, 186.0);
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
    assert_eq!(chosen.get(), 0, "ordinary result click positive control");
    chosen.set(-1);
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    window.set_search_entries(model("replacement.txt"));
    window
        .window()
        .take_snapshot()
        .expect("replacement model layout");
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert_eq!(
        chosen.get(),
        -1,
        "a held click activated a replacement search result"
    );
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
    assert_eq!(
        chosen.get(),
        0,
        "new click must still activate the replacement row"
    );
    window.hide().expect("close pointer fixture");
}
