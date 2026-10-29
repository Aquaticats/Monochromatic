//! Native tree callbacks exercise real background reads,
//!  file installation,
//!  and recent-file reveal.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Use the same source and navigation bindings as the shipped window.
use super::{
    AppWindow, State, bind_appearance, bind_keys, bind_pointer, bind_viewport, navigation, reload,
    render,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Reading positions and canonical project identity remain independent of UI labels.
use ide_app::{document::ReadingPosition, workspace::Workspace};
/// Headless snapshots use the real window and system-time timer processing.
use slint::{ComponentHandle, Model, SharedString, platform::update_timers_and_animations};
/// Fixtures and waits are bounded,
///  while UI ownership remains single-threaded.
use std::{cell::RefCell, fs, rc::Rc};

/// Find a visible fixture label;
///  actions still use the model index backed by its native path.
pub(super) fn row(window: &AppWindow, label: &str) -> Option<i32> {
    let model = window.get_tree_entries();
    for index in 0..model.row_count() {
        if let Some(entry) = model.row_data(index)
            && entry.label == label
        {
            return Some(index as i32);
        }
    }
    return None;
}

/// Ctrl+0 reveals a collapsed ancestor without reloading;
///  Ctrl+1 promotes and alternates successful opens.
#[test]
fn native_tree_switches_files_and_preserves_current_file_reading_state() {
    let fixture = tempfile::tempdir().expect("disposable native project");
    let nested = fixture.path().join("src/deep");
    fs::create_dir_all(&nested).expect("nested fixture directories");
    let initial = nested.join("view.txt");
    let text = "I am a big cat.\n".repeat(100);
    fs::write(&initial, &text).expect("initial source");
    fs::write(fixture.path().join("other.txt"), "other source").expect("second source");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    install_gpu_platform();
    let window = AppWindow::new().expect("native project window");
    let state = Rc::new(RefCell::new(State::new(&text, Some(initial.clone()))));
    window.set_file_label(SharedString::from(initial.display().to_string()));
    bind_pointer(&window, &state);
    bind_viewport(&window, &state);
    bind_keys(&window, &state);
    bind_appearance(&window, &state);
    let _source_refresh = reload::bind(&window, &state).expect("source refresh");
    let _navigation = navigation::bind(&window, &state, workspace).expect("project navigation");
    window.show().expect("show project window");
    render(&window, &state);
    // What: Wait for the highlighting reply of the opened file before the test starts its steps.
    // Why: The reply renders again after the first frame, so a step timed or counted from here met it in the middle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await waitForHighlighting(state);
    // ```
    super::find_tests::wait_for_highlighting(&state);
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "view.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    let selected = row(&window, "view.txt").expect("revealed initial source");
    let entry = window
        .get_tree_entries()
        .row_data(selected as usize)
        .expect("initial row");
    assert_eq!(entry.recency, "0");
    assert!(entry.selected);
    state.borrow_mut().document.select(ReadingPosition {
        anchor: 2,
        head: 6,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    render(&window, &state);
    window.set_scroll_y(-120.5);
    update_timers_and_animations();
    let position = state.borrow().document.position();
    let generation = state.borrow().file_generation;
    let offset = window.get_scroll_y();
    window.invoke_tree_activate(row(&window, "src").expect("root folder"));
    assert!(row(&window, "view.txt").is_none());
    window.invoke_tree_shortcut(SharedString::from("0"), true, false, false);
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "view.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(state.borrow().file_generation, generation);
    assert_eq!(state.borrow().document.position(), position);
    assert_eq!(window.get_scroll_y(), offset);
    assert_eq!(window.get_selected_text(), "am a");
    window.invoke_tree_activate(row(&window, "other.txt").expect("other source row"));
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_source_text() == "other source") {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(state.borrow().document.position().head, 0);
    window.invoke_tree_shortcut(SharedString::from("1"), true, false, false);
    {
        let mut waiting = Waiting::eventually();
        while !(state.borrow().file_path.as_ref() == Some(&initial)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    window.invoke_tree_shortcut(SharedString::from("1"), true, false, false);
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_source_text() == "other source") {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    let opened_generation = state.borrow().file_generation;
    window.invoke_tree_shortcut(SharedString::from("9"), true, false, false);
    assert_eq!(state.borrow().file_generation, opened_generation);
    window.hide().expect("close project window");
}

/// Empty startup can open a file,
///  and a later failed open retains it through successful source refreshes.
#[test]
fn native_failed_open_keeps_source_and_does_not_promote_history() {
    let fixture = tempfile::tempdir().expect("disposable native project");
    fs::write(fixture.path().join("source.txt"), "readable source").expect("readable fixture");
    fs::write(fixture.path().join("binary.txt"), [0xff]).expect("invalid UTF-8 fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    install_gpu_platform();
    let window = AppWindow::new().expect("native project window");
    let state = Rc::new(RefCell::new(State::new("", None)));
    window.set_source_available(false);
    bind_viewport(&window, &state);
    bind_keys(&window, &state);
    bind_appearance(&window, &state);
    let _source_refresh = reload::bind(&window, &state).expect("source refresh");
    let _navigation = navigation::bind(&window, &state, workspace).expect("project navigation");
    window.show().expect("show project window");
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "source.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert!(!window.get_source_available());
    window.invoke_tree_activate(row(&window, "source.txt").expect("source row"));
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_source_text() == "readable source") {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert!(window.get_source_available());
    let generation = state.borrow().file_generation;
    window.invoke_tree_activate(row(&window, "binary.txt").expect("binary row"));
    {
        let mut waiting = Waiting::eventually();
        while !(state.borrow().navigation_error.is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(window.get_source_text(), "readable source");
    assert_eq!(state.borrow().file_generation, generation);
    let binary = row(&window, "binary.txt").expect("binary remains visible");
    assert_eq!(
        window
            .get_tree_entries()
            .row_data(binary as usize)
            .expect("binary metadata")
            .recency,
        ""
    );
    fs::write(
        fixture.path().join("source.txt"),
        "externally refreshed source",
    )
    .expect("external replacement");
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_source_text() == "externally refreshed source") {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert!(window.get_error_message().contains("binary.txt"));
    window.invoke_tree_shortcut(SharedString::from("0"), true, false, false);
    assert!(state.borrow().navigation_error.is_none());
    window.hide().expect("close project window");
}
