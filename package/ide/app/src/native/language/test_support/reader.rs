//! A reader with the scripted language server:
//!  the window,
//!  the shared source state,
//!  the language binding under test,
//!  and the timers whose drop releases the workers.

/// Windows are drawn on the hardware GPU,
///  so the platform is installed before the thread's first window.
use crate::native::gpu_snapshot::install_gpu_platform;
/// The language binding under test,
///  and installing it on a window.
use crate::native::language::{
    LanguageBinding,
    bind,
};
/// The production window,
///  source state,
///  and every binding the application installs.
use crate::native::{
    AppWindow,
    State,
    bind_appearance,
    bind_keys,
    bind_pointer,
    bind_viewport,
    find,
    navigation,
    reload,
    render,
};
/// The handle,
///  its setup,
///  the startup rule,
///  and the project an open file belongs to.
use ide_app::{
    language::{
        LanguageWorker,
        config::LanguageSetup,
        enter_project_directory,
        launch::{
            LaunchPolicy,
            launch_directly,
        },
    },
    workspace::Workspace,
};
/// `SharedString` is Slint's reference-counted text type (siblings: Rust's owned `String`
///       and its borrowed `&str`);
///  `Timer` is the toolkit's repeating timer,
///  and `ComponentHandle` gives the window's own handle.
/// Why:
///  The file label shows a path the toolkit copies,
///  and the timers must live as long as the reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const label = SharedString.from(path); const timers: Timer[] = [];
/// ```
use slint::{
    ComponentHandle,
    SharedString,
    Timer,
};
/// Fixture text and paths,
///  and the bounded waits the reader settles with.
use std::{
    cell::RefCell,
    fs,
    path::Path,
    rc::Rc,
    time::{
        Duration,
        Instant,
    },
};

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
pub(super) fn reader(
    project: &Project,
    name: &str,
    languages: String,
) -> LanguageReader {
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
