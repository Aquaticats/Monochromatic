//! The shipped tree and source bindings follow external changes through inotify notifications,
//! faster than the safety sweep or the old polling could have produced them.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use super::gpu_snapshot::install_gpu_platform;
/// Tree-row lookup by label,
///  shared with the navigation tests.
use super::navigation_tests::row;
/// The production window,
///  source state,
///  and the bindings the shipped application installs.
use super::{
    AppWindow, State, bind_appearance, bind_keys, bind_pointer, bind_viewport, navigation, reload,
    render,
};
/// Bounded waiting that advances native timers until a state the test checks holds.
use crate::native::waiting_tests::{NAVIGATION_UNREACHED, Waiting};
/// Reading positions and the canonical project boundary.
use ide_app::{document::ReadingPosition, workspace::Workspace};
/// Real headless timers drive the same refresh timers as the shipped event loop.
use slint::{ComponentHandle, Model, SharedString, Timer, platform::update_timers_and_animations};
/// What:
///  `Rc<RefCell<State>>` is the window's shared source state;
///  `Path` borrows a fixture path.
/// Why:
///  Assertions read the model the window renders,
///  not pixels.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = { current: T };
/// ```
use std::{
    cell::RefCell,
    fs,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};

/// Atomically replace the displayed file with `text`,
///  staging the new content in `folder` first.
fn replace(folder: &Path, displayed: &Path, text: &str) {
    let staged = folder.join(".view.txt.tmp");
    fs::write(&staged, text).expect("staged replacement");
    fs::rename(&staged, displayed).expect("atomic replace");
}

/// A shipped window over `root`,
///  displaying `file`,
///  with refresh and navigation timers kept alive.
pub(super) fn open(root: &Path, file: &Path) -> (AppWindow, Rc<RefCell<State>>, [Timer; 2]) {
    let text = fs::read_to_string(file).expect("displayed source");
    let workspace = Workspace::new(root).expect("workspace");
    install_gpu_platform();
    let window = AppWindow::new().expect("native watch window");
    // `Some` marks an on-disk displayed file that the source refresh rereads.
    let state = Rc::new(RefCell::new(State::new(&text, Some(file.to_path_buf()))));
    window.set_file_label(SharedString::from(file.display().to_string()));
    bind_pointer(&window, &state);
    bind_viewport(&window, &state);
    bind_keys(&window, &state);
    bind_appearance(&window, &state);
    let refresh = reload::bind(&window, &state).expect("source refresh");
    let project = navigation::bind(&window, &state, workspace).expect("project navigation");
    window.show().expect("show watch window");
    render(&window, &state);
    // What: Wait for the highlighting reply of the opened file before the test starts its steps.
    // Why: The reply renders again after the first frame, so a step timed or counted from here met it in the middle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await waitForHighlighting(state);
    // ```
    super::find_tests::wait_for_highlighting(&state);
    return (window, state, [refresh, project]);
}

/// Depth of the visible row labelled `label`.
fn depth(window: &AppWindow, label: &str) -> Option<i32> {
    let index = row(window, label)?;
    let entry = window.get_tree_entries().row_data(index as usize)?;
    return Some(entry.depth);
}

/// Create,
///  rename,
///  move in,
///  move out,
///  and delete in the last of four expanded folders all show promptly.
#[test]
fn native_tree_follows_changes_in_one_of_several_expanded_folders() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let displayed = fixture.path().join("view.txt");
    fs::write(&displayed, "view\n").expect("displayed source");
    fs::write(fixture.path().join("outer.txt"), "outer\n").expect("root file");
    for index in 0..4 {
        let folder = fixture.path().join(format!("folder-{index:02}"));
        fs::create_dir(&folder).expect("fixture folder");
        fs::write(folder.join(format!("seed-{index:02}.txt")), "").expect("folder seed");
    }
    let (window, _state, _timers) = open(fixture.path(), &displayed);
    for index in 0..4 {
        let label = format!("folder-{index:02}");
        {
            let mut waiting = Waiting::eventually();
            while !(row(&window, &label).is_some()) {
                waiting.step(NAVIGATION_UNREACHED);
            }
        }
        window.invoke_tree_activate(row(&window, &label).expect("folder row"));
        let seed = format!("seed-{index:02}.txt");
        {
            let mut waiting = Waiting::eventually();
            while !(row(&window, &seed).is_some()) {
                waiting.step(NAVIGATION_UNREACHED);
            }
        }
    }
    let last = fixture.path().join("folder-03");
    fs::write(last.join("created.txt"), "").expect("external create");
    {
        let mut waiting = Waiting::prompt();
        while !(row(&window, "created.txt").is_some()) {
            waiting.step_missing("a created file");
        }
    }
    fs::rename(last.join("created.txt"), last.join("renamed.txt")).expect("external rename");
    {
        let mut waiting = Waiting::prompt();
        while !(row(&window, "renamed.txt").is_some() && row(&window, "created.txt").is_none()) {
            waiting.step_missing("a renamed file");
        }
    }
    fs::rename(fixture.path().join("outer.txt"), last.join("outer.txt"))
        .expect("move into the folder");
    {
        let mut waiting = Waiting::prompt();
        while !(depth(&window, "outer.txt") == Some(1)) {
            waiting.step_missing("a file moved into the folder");
        }
    }
    fs::rename(last.join("outer.txt"), fixture.path().join("outer.txt"))
        .expect("move out of the folder");
    {
        let mut waiting = Waiting::prompt();
        while !(depth(&window, "outer.txt") == Some(0)) {
            waiting.step_missing("a file moved out of the folder");
        }
    }
    fs::remove_file(last.join("renamed.txt")).expect("external delete");
    {
        let mut waiting = Waiting::prompt();
        while !(row(&window, "renamed.txt").is_none()) {
            waiting.step_missing("a deleted file's removal");
        }
    }
    window.hide().expect("close watch window");
}

/// Atomic replace keeps both correspondence examples;
///  delete then recreate shows the error and recovers.
/// The displayed file's folder is collapsed in the tree,
///  so only its own watch can report these changes.
#[test]
fn native_source_follows_atomic_replace_and_delete_then_recreate() {
    let fixture = tempfile::tempdir().expect("disposable project");
    let folder = fixture.path().join("nested");
    fs::create_dir(&folder).expect("collapsed folder");
    let displayed = folder.join("view.txt");
    fs::write(&displayed, "I am a big cat.\n").expect("displayed source");
    let (window, state, _timers) = open(fixture.path(), &displayed);
    {
        let mut waiting = Waiting::eventually();
        while !(state.borrow().refresh.is_watched()) {
            waiting.step(NAVIGATION_UNREACHED);
        }
    }
    // Let the startup reads (first read, highlighting, the new watch's extra read) finish,
    // so only a change notification can explain the next read.
    let settled = Instant::now();
    while settled.elapsed() < Duration::from_millis(300) {
        update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(2));
    }

    // Caret example: `I am a bi|g cat.` becomes `I was a bi|g cat.`.
    state.borrow_mut().document.select(ReadingPosition {
        anchor: 9,
        head: 9,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    replace(&folder, &displayed, "I was a big cat.\n");
    {
        let mut waiting = Waiting::prompt();
        while !(window.get_source_text() == "I was a big cat.\n") {
            waiting.step_missing("the caret replacement");
        }
    }
    assert_eq!(
        state.borrow().document.position().head,
        10,
        "the caret left the g"
    );
    fs::write(&displayed, "I am a big cat\n").expect("reset text");
    {
        let mut waiting = Waiting::prompt();
        while !(window.get_source_text() == "I am a big cat\n") {
            waiting.step_missing("the reset text");
        }
    }
    // Selection example: `I [am a] big cat` follows the replaced region, not the later literal.
    state.borrow_mut().document.select(ReadingPosition {
        anchor: 2,
        head: 6,
        viewport: 0,
        affinity: ide_app::document::ReadingAffinity::Downstream,
    });
    render(&window, &state);
    replace(
        &folder,
        &displayed,
        "I was a big cat, but now I am a human!\n",
    );
    {
        let mut waiting = Waiting::prompt();
        while !(window.get_source_text() == "I was a big cat, but now I am a human!\n") {
            waiting.step_missing("the selection replacement");
        }
    }
    assert_eq!(
        window.get_selected_text(),
        "was a",
        "selection followed the later literal"
    );
    fs::remove_file(&displayed).expect("delete displayed file");
    {
        let mut waiting = Waiting::prompt();
        while !(state.borrow().file_error.is_some()) {
            waiting.step_missing("the missing-file diagnostic");
        }
    }
    assert_eq!(
        window.get_source_text(),
        "I was a big cat, but now I am a human!\n"
    );
    fs::write(&displayed, "recreated\n").expect("recreate displayed file");
    {
        let mut waiting = Waiting::prompt();
        while !(window.get_source_text() == "recreated\n" && state.borrow().file_error.is_none()) {
            waiting.step_missing("the recreated file");
        }
    }
    assert!(
        state.borrow().refresh.is_watched(),
        "the source fell back to polling"
    );
    window.hide().expect("close watch window");
}
