//! A complete reader with the scripted language server,
//!  for window tests of language navigation.
//!
//! `test:native` runs every test in its own process (nextest's default),
//!  so a test may make its
//! disposable project the working directory,
//!  as the application does at startup;
//!  Helix roots
//! servers there.
//!  The scripted server is the `ide-scripted-lsp` binary beside the test binary,
//! built by the task before the tests run;
//!  its answers come from `IDE_SCRIPTED_*` variables.

/// The production window,
///  state,
///  bindings,
///  and the language binding under test.
use super::{LanguageBinding, bind};
/// Shared helpers of the other window tests:
///  key events and bounded waiting.
use crate::native::find_tests::{chord, key};
/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use crate::native::gpu_snapshot::install_gpu_platform;
/// The production window,
///  source state,
///  and every binding the application installs.
use crate::native::{
    AppWindow, State, bind_appearance, bind_keys, bind_pointer, bind_viewport, find, navigation,
    reload, render,
};
/// The handle,
///  its setup,
///  the launch policy seam,
///  and the startup rule.
use ide_app::{
    language::{
        LanguageWorker,
        config::LanguageSetup,
        enter_project_directory,
        launch::{LaunchPolicy, launch_directly},
    },
    workspace::Workspace,
};
/// Window events as a seat delivers them.
use slint::{
    ComponentHandle, LogicalPosition, SharedString, Timer,
    platform::{Key, PointerEventButton, WindowEvent},
};
/// What:
///  Disposable files,
///  owned paths,
///  the shared state cell,
///  and bounded waits.
/// Why:
///  Every fixture lives in a temporary directory removed when the test ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { writeFileSync, mkdtempSync } from 'node:fs';
/// ```
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    time::{Duration, Instant},
};

/// Waiting for timers and for the scripted server,
///  in their own module.
pub(super) use super::test_waits::ready;

/// Name of the scripted server definition.
pub(super) const SERVER: &str = "scripted-ls";

/// Where source text starts in the window,
///  derived in `sidebar_tests` from the tree,
///  divider,
///  and gutter widths.
pub(super) use super::super::sidebar_tests::TEXT_LEFT;

/// Source rows start below the 32 px file label.
pub(super) const TEXT_TOP: f32 = 32.0;

/// What:
///  A disposable project:
///  the directory guard,
///  the canonical root,
///  and a scratch directory
///       beside the project for outside-project files.
/// Why:
///  Dropping the guard removes everything;
///  servers report canonical paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Project = { guard: TempDir; root: string; outside: string };
/// ```
pub(super) struct Project {
    /// Removes the directory when dropped.
    _guard: tempfile::TempDir,
    /// Canonical project root.
    pub(super) root: PathBuf,
    /// Canonical directory beside the project,
    ///  outside it.
    pub(super) outside: PathBuf,
}

/// Create a disposable project with `files` (relative path and text) and an outside directory.
pub(super) fn project(files: &[(&str, &str)]) -> Project {
    // The project lives in memory when `/dev/shm` exists (see `memory_project`): on btrfs, the first
    // read of a fresh file and the creation of files wait for filesystem transactions, which took
    // longer than the waits of these tests while the machine flushed.
    let guard = crate::native::find_tests::memory_project("ide-native-language-");
    let base = guard.path().canonicalize().expect("canonical base");
    let root = base.join("project");
    let outside = base.join("outside");
    fs::create_dir(&root).expect("project directory");
    fs::create_dir(&outside).expect("outside directory");
    for (relative, text) in files {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("fixture directories");
        fs::write(&path, text).expect("fixture file");
    }
    return Project {
        _guard: guard,
        root,
        outside,
    };
}

/// The `file` address of a canonical path,
///  as a server names it.
pub(super) fn address(path: &Path) -> String {
    return format!("file://{}", path.display());
}

/// A JSON location for the scripted server:
///  an address and a zero-based line and column range.
pub(super) fn location(uri: &str, line: u32, start: u32, end: u32) -> String {
    return format!(
        r#"{{"uri":"{uri}","range":{{"start":{{"line":{line},"character":{start}}},"end":{{"line":{line},"character":{end}}}}}}}"#
    );
}

/// What:
///  Language definitions for the scripted server with extra `IDE_SCRIPTED_*` variables.
///       `command` replaces the server program,
///  for the missing-program case.
/// Why:
///  Each test names exactly the answers it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function definitions(variables: [string, string][], command?: string): string
/// ```
pub(super) fn definitions(variables: &[(&str, &str)], command: Option<&str>) -> String {
    // The scripted server is built beside the test binary's `deps` directory.
    let program = std::env::current_exe()
        .expect("test executable")
        .parent()
        .and_then(Path::parent)
        .expect("target directory")
        .join("ide-scripted-lsp");
    assert!(
        command.is_some() || program.is_file(),
        "build ide-scripted-lsp before the native tests: {}",
        program.display()
    );
    let program_text = program.display().to_string();
    let chosen = command.unwrap_or(&program_text);
    // What: `Vec<String>` is a growable list of owned strings (sibling: a fixed array `[String; N]`);
    //       `iter().any(...)` asks whether one pair names `PUSH`; `*name` reads the borrowed name.
    // Why: The server pushes a diagnostic on every publish unless told not to. Tests that do not
    //      ask for it turn it off, and a test that names `PUSH` decides alone: an inline TOML
    //      table refuses the same key twice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const entries: string[] = variables.some(([name]) => name === 'PUSH') ? [] : ["IDE_SCRIPTED_PUSH = '0'"];
    // ```
    let mut entries: Vec<String> = Vec::new();
    let mut names_push = false;
    for (name, _value) in variables {
        if *name == "PUSH" {
            names_push = true;
            break;
        }
    }
    if !names_push {
        entries.push(String::from("IDE_SCRIPTED_PUSH = '0'"));
    }
    for (name, value) in variables {
        entries.push(format!("IDE_SCRIPTED_{name} = '{value}'"));
    }
    let environment = entries.join(", ");
    return format!(
        r#"
[language-server.{SERVER}]
command = '{chosen}'
timeout = 5
environment = {{ {environment} }}

[[language]]
name = "scripted"
scope = "source.scripted"
file-types = ["scripted"]
roots = []
language-servers = ["{SERVER}"]
"#
    );
}

/// What:
///  A reader with every production binding plus language navigation.
/// Why:
///  The tests drive the same callbacks,
///  timers,
///  and markup the application installs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LanguageReader = { window: AppWindow; source: Shared<State>; binding?: LanguageBinding };
/// ```
pub(super) struct LanguageReader {
    /// Native window built from the shipped markup.
    pub(super) window: AppWindow,
    /// Source state shared with every binding.
    pub(super) source: Rc<RefCell<State>>,
    /// The language binding;
    ///  `close` consumes it.
    pub(super) binding: Option<LanguageBinding>,
    /// Dropping the timers releases the reload,
    ///  find,
    ///  and navigation workers.
    _timers: [Timer; 3],
}

/// What:
///  Open `name` of `project` with the scripted server configured by `variables`.
/// Why:
///  The project becomes the working directory first,
///  before any thread or Helix call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reader(project: Project, name: string, definitions: string): LanguageReader
/// ```
pub(super) fn reader(project: &Project, name: &str, languages: String) -> LanguageReader {
    // Why direct launch: these tests run in the build container, whose image has no bubblewrap,
    // so the production policy would refuse every launch; the server here is this crate's own
    // scripted binary and the project is a disposable directory.
    return reader_launching(project, name, languages, launch_directly);
}

/// What:
///  Open `name` of `project` with the scripted server,
///  started through `launch`.
///       `LaunchPolicy` is a plain function pointer;
///  sibling:
///  a boxed closure,
///  which could carry data.
/// Why:
///  A test passes a refusing policy to stand in for the production one refusing,
///      without depending on what the test machine has installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readerLaunching(project: Project, name: string, languages: string, launch: LaunchPolicy): LanguageReader
/// ```
pub(super) fn reader_launching(
    project: &Project,
    name: &str,
    languages: String,
    launch: LaunchPolicy,
) -> LanguageReader {
    enter_project_directory(&project.root).expect("project as working directory");
    // What: `..LanguageSetup::unconfined()` fills every field not named here from the unconfined setup.
    // Why: Only the launch policy and the language definitions differ between tests.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const setup = { ...LanguageSetup.unconfined(), launch, extraLanguages: languages };
    // ```
    let setup = LanguageSetup {
        launch,
        extra_languages: Some(languages),
        ..LanguageSetup::unconfined()
    };
    let worker = LanguageWorker::with_setup(&project.root, setup);
    return reader_with(project, name, worker);
}

/// Open `name` with an explicit worker result,
///  such as a start failure.
pub(super) fn reader_with(
    project: &Project,
    name: &str,
    worker: anyhow::Result<LanguageWorker>,
) -> LanguageReader {
    let path = project.root.join(name);
    let text = fs::read_to_string(&path).expect("fixture source");
    let workspace = Workspace::new(&project.root).expect("disposable workspace");
    install_gpu_platform();
    let window = AppWindow::new().expect("native language window");
    let source = Rc::new(RefCell::new(State::new(&text, Some(path.clone()))));
    window.set_file_label(SharedString::from(path.display().to_string()));
    bind_pointer(&window, &source);
    bind_viewport(&window, &source);
    bind_keys(&window, &source);
    bind_appearance(&window, &source);
    let refresh = reload::bind(&window, &source).expect("source refresh");
    let finder = find::bind(&window, &source).expect("in-file find");
    let (project_timer, navigation) =
        navigation::bind_shared(&window, &source, workspace).expect("project navigation");
    let binding = bind(&window, &source, &navigation, &project.root, worker);
    window.show().expect("show language window");
    render(&window, &source);
    window.window().take_snapshot().expect("initial layout");
    // The layout gave the view its width; a tile for it may still be on the worker.
    crate::native::overscan_tile_tests::settle_tile(&source);
    // What: Wait for the highlighting reply of the opened file before the test starts its steps.
    // Why: The reply renders again after the first frame, so a step timed or counted from here met it in the middle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await waitForHighlighting(source);
    // ```
    crate::native::find_tests::wait_for_highlighting(&source);
    crate::native::overscan_tile_tests::settle_tile(&source);
    window.invoke_focus_source();
    return LanguageReader {
        window,
        source,
        binding: Some(binding),
        _timers: [refresh, finder, project_timer],
    };
}

/// Advance timers for `millis` milliseconds without expecting anything.
pub(super) fn idle(millis: u64) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(millis) {
        slint::platform::update_timers_and_animations();
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Place the caret at `position` through the document,
///  as a click would.
pub(super) fn caret(reader: &LanguageReader, position: usize) {
    let mut current = reader.source.borrow_mut();
    let mut reading = current.document.position();
    reading.anchor = position;
    reading.head = position;
    reading.affinity = ide_app::document::ReadingAffinity::Downstream;
    current.document.select(reading);
    drop(current);
    render(&reader.window, &reader.source);
}

/// Caret head in source characters.
pub(super) fn head(reader: &LanguageReader) -> usize {
    return reader.source.borrow().document.position().head;
}

/// Ctrl+B,
///  as a seat delivers it.
pub(super) fn definition(reader: &LanguageReader) {
    chord(&reader.window, Key::Control, "b");
}

/// Ctrl+Q,
///  as a seat delivers it.
pub(super) fn hover(reader: &LanguageReader) {
    chord(&reader.window, Key::Control, "q");
}

/// Escape,
///  as a seat delivers it.
pub(super) fn escape(reader: &LanguageReader) {
    key(&reader.window, Key::Escape);
}

/// What:
///  Window point over the code row of source `line`,
///  `x` logical pixels into its text,
///  in the middle of
///       the row.
///  The row's place comes from the reader's vertical mapping at the moment of the call.
/// Why:
///  The scripted server's hint and diagnostics take rows above the first line once they arrive,
///  so a
///      line's code row is not at its number times one row height.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function point(reader: LanguageReader, line: number, x: number): LogicalPosition;
/// ```
pub(super) fn point(reader: &LanguageReader, line: usize, x: f32) -> LogicalPosition {
    let top = reader.source.borrow().row_map.code_top(line);
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + top + 12.0 + reader.window.get_scroll_y(),
    );
}

/// What:
///  Window point `rise` logical pixels above the top of `line`'s code row,
///  `x` logical pixels into the
///       text:
///  inside the line's virtual rows when it has any.
/// Why:
///  Resting and Ctrl+clicking on a hint or message row must ask the server nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function above(reader: LanguageReader, line: number, x: number, rise: number): LogicalPosition;
/// ```
pub(super) fn above(reader: &LanguageReader, line: usize, x: f32, rise: f32) -> LogicalPosition {
    let top = reader.source.borrow().row_map.code_top(line);
    return LogicalPosition::new(
        TEXT_LEFT + x,
        TEXT_TOP + top - rise + reader.window.get_scroll_y(),
    );
}

/// What:
///  How many hint texts and message-row texts the displayed frame paints;
///  a pair (tuple).
/// Why:
///  Hints and messages are pixels of the source image;
///  the frame's positioned records say what was drawn.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rows(reader: LanguageReader): [hints: number, messages: number];
/// ```
pub(super) fn rows(reader: &LanguageReader) -> (usize, usize) {
    let source = reader.source.borrow();
    // `let ... else` answers zeros when no frame or no annotations of it exist.
    let Some(view) = source.shaped.as_ref() else {
        return (0, 0);
    };
    let Some(frame) = view.annotations.as_ref() else {
        return (0, 0);
    };
    let mut hints = 0;
    let mut messages = 0;
    for text in &frame.texts {
        if text.severity.is_some() {
            messages += 1;
        } else {
            hints += 1;
        }
    }
    return (hints, messages);
}

/// Move the pointer to a window point without a button held.
pub(super) fn move_to(window: &AppWindow, position: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
}

/// A left click at a window point.
pub(super) fn click(window: &AppWindow, position: LogicalPosition) {
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}

/// A Ctrl+click at a window point.
pub(super) fn control_click(window: &AppWindow, position: LogicalPosition) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    click(window, position);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}

/// Logical x of the caret before character `position` on its shaped row.
pub(super) fn caret_x(reader: &LanguageReader, position: usize) -> f32 {
    let state = reader.source.borrow();
    let line = state.document.text().char_to_line(position);
    let view = state.shaped.as_ref().expect("shaped source");
    for row in &view.rows {
        if row.row == line {
            return row.caret_x(position, view.viewport.scale);
        }
    }
    panic!("line {line} is not materialized");
}

/// The popup text,
///  or empty when no popup is shown.
pub(super) fn popup(reader: &LanguageReader) -> String {
    return reader.window.get_language_popup_text().to_string();
}
