//! Reveal intent must not continuously replace unchanged models,
//!  and canonical aliases retain reading state.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Share production bindings and native polling helpers without adding test interfaces to the shipped binary.
use super::{
    AppWindow, State, bind_appearance, bind_viewport, navigation, navigation_tests::row, render,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Canonical project identity and source positions are the same types used in navigation.
use ide_app::{document::ReadingPosition, workspace::Workspace};
/// ModelRc equality compares underlying model identity,
///  not only equal row values.
use slint::{ComponentHandle, Model, SharedString};
/// Native symlinks and file replacement live only in disposable directories.
use std::{
    cell::RefCell,
    fs,
    os::unix::fs::symlink,
    rc::Rc,
    time::{Duration, Instant},
};

/// A missing current row waits for real directory changes instead of rebuilding the ListView every timer tick.
#[test]
fn missing_reveal_keeps_the_existing_model_until_tree_changes() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let path = fixture.path().join("source.txt");
    fs::write(&path, "source").expect("source fixture");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    install_gpu_platform();
    let window = AppWindow::new().expect("native window");
    let source = Rc::new(RefCell::new(State::new("source", Some(path.clone()))));
    bind_viewport(&window, &source);
    bind_appearance(&window, &source);
    let _navigation = navigation::bind(&window, &source, workspace).expect("navigation");
    window.show().expect("show window");
    render(&window, &source);
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "source.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    let original = window.get_tree_entries();
    fs::write(fixture.path().join("control.txt"), "positive control").expect("additional source");
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "control.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    // Positive control: the identity assertion must detect an actual published-model replacement.
    assert_ne!(original, window.get_tree_entries());
    fs::remove_file(&path).expect("external removal");
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "source.txt").is_none()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    window.invoke_tree_shortcut(SharedString::from("0"), true, false, false);
    let retained = window.get_tree_entries();
    let start = Instant::now();
    {
        let mut waiting = Waiting::eventually();
        while !(start.elapsed() >= Duration::from_millis(120)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(
        retained,
        window.get_tree_entries(),
        "unresolved reveal replaced an unchanged tree model"
    );
    fs::write(&path, "source").expect("restore source");
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "source.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_ne!(retained, window.get_tree_entries());
    assert_eq!(
        window.get_tree_focused_row(),
        row(&window, "source.txt").expect("restored revealed row")
    );
    window.hide().expect("close window");
}

/// A root lost before its first read must not claim that a previous snapshot exists.
#[test]
fn initial_directory_failure_reports_absent_snapshot_and_recovers() {
    let fixture = tempfile::tempdir().expect("disposable project parent");
    let root = fixture.path().join("project");
    fs::create_dir(&root).expect("initial directory");
    let workspace = Workspace::new(&root).expect("workspace");
    fs::remove_dir(&root).expect("directory disappears before first listing");
    install_gpu_platform();
    let window = AppWindow::new().expect("native window");
    let source = Rc::new(RefCell::new(State::new("", None)));
    window.set_source_available(false);
    bind_viewport(&window, &source);
    bind_appearance(&window, &source);
    let _navigation = navigation::bind(&window, &source, workspace).expect("navigation");
    window.show().expect("show window");
    {
        let mut waiting = Waiting::eventually();
        while !(!window.get_tree_error().is_empty()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert!(
        window
            .get_tree_error()
            .contains("No directory snapshot is available")
    );
    assert!(
        !window
            .get_tree_error()
            .contains("last directory snapshot is retained")
    );
    fs::create_dir(&root).expect("restore project directory");
    fs::write(root.join("new.txt"), "new source").expect("restored directory entry");
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "new.txt").is_some() && window.get_tree_error().is_empty()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    window.hide().expect("close window");
}

/// Inside symlink aliases converge on the canonical row without resetting an already displayed document.
#[test]
fn contained_alias_open_preserves_current_selection_and_stops_reveal_updates() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let path = fixture.path().join("source.txt");
    fs::write(&path, "source").expect("source fixture");
    symlink(&path, fixture.path().join("alias.txt")).expect("inside alias");
    let workspace = Workspace::new(fixture.path()).expect("workspace");
    install_gpu_platform();
    let window = AppWindow::new().expect("native window");
    let source = Rc::new(RefCell::new(State::new("source", Some(path.clone()))));
    bind_viewport(&window, &source);
    bind_appearance(&window, &source);
    let _navigation = navigation::bind(&window, &source, workspace).expect("navigation");
    window.show().expect("show window");
    render(&window, &source);
    {
        let mut waiting = Waiting::eventually();
        while !(row(&window, "alias.txt").is_some()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    source.borrow_mut().document.select(ReadingPosition {
        anchor: 1,
        head: 4,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    render(&window, &source);
    let generation = source.borrow().file_generation;
    let before = window.get_tree_entries();
    window.invoke_tree_activate(row(&window, "alias.txt").expect("alias row"));
    // A completed alias open republishes recency; unchanged periodic reads do not replace the model.
    {
        let mut waiting = Waiting::eventually();
        while !(window.get_tree_entries() != before) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(source.borrow().file_path.as_ref(), Some(&path));
    assert_eq!(source.borrow().file_generation, generation);
    assert_eq!(window.get_selected_text(), "our");
    let canonical = row(&window, "source.txt").expect("canonical row");
    let model = window.get_tree_entries();
    assert!(
        model
            .row_data(canonical as usize)
            .expect("canonical metadata")
            .selected
    );
    assert_eq!(
        model
            .row_data(canonical as usize)
            .expect("canonical slot")
            .recency,
        "0"
    );
    let start = Instant::now();
    {
        let mut waiting = Waiting::eventually();
        while !(start.elapsed() >= Duration::from_millis(120)) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    assert_eq!(model, window.get_tree_entries());
    window.hide().expect("close window");
}
