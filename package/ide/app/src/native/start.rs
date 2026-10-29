//! Starting one project window:
//!  logging,
//!  the project and its language worker,
//!  the window with every binding,
//!  and the event loop until the window closes.

/// Inspection-only hint and diagnostic injection from a JSON file;
///  debug builds only.
#[cfg(debug_assertions)]
use super::inspect;
/// The window,
///  the shared state,
///  and the modules whose bindings are installed at startup.
use super::{
    AppWindow, State, bind_appearance, bind_keys, bind_pointer, bind_viewport, find, frame_trace,
    language, minimap, navigation, reload, render,
};
/// Source-open errors identify their input instead of exposing an unlabelled I/O failure.
use anyhow::{Context, bail};
/// The Language module's handle,
///  and its startup rule.
use ide_app::language::{LanguageWorker, enter_project_directory};
/// The log filter,
///  the subscriber with the helix-lsp bridge,
///  and the writer thread.
use ide_app::logging::{self, background};
/// Explicit startup paths retain one canonical project boundary.
use ide_app::{cli::Options, workspace::Workspace};
/// Toolkit handles bridge owned Rust state to the window.
use slint::{ComponentHandle, SharedString};
/// Callbacks need the same document without cross-thread Arc/Mutex overhead.
use std::{cell::RefCell, path::Path, rc::Rc};

/// What:
///  Success when this window may start language servers,
///  otherwise the reason it starts none.
///       `anyhow::Result<()>` is success without a value,
///  or an error.
/// Why:
///  A debug build started with `IDE_INSPECT_ANNOTATIONS` shows that file's hints and diagnostics.
///  A worker
///      without servers still publishes "no problems" for the displayed text,
///  which would replace the file's
///      diagnostics,
///  so such a window runs without the worker;
///  asking for a language feature explains why.
///      Release builds do not read the variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function serversWanted(): void { if (DEBUG && process.env.IDE_INSPECT_ANNOTATIONS) throw new Error('...'); }
/// ```
fn servers_wanted() -> anyhow::Result<()> {
    // `var_os` reads the variable without requiring UTF-8; `is_some` asks whether it is set.
    #[cfg(debug_assertions)]
    if std::env::var_os("IDE_INSPECT_ANNOTATIONS").is_some() {
        bail!(
            "this window shows the annotations of IDE_INSPECT_ANNOTATIONS and starts no language server"
        );
    }
    // `Ok(())` reports success without a payload.
    return Ok(());
}

/// What:
///  Enter the project directory,
///  check that servers are wanted,
///  and start the language worker.
/// Why:
///  Either failure leaves the window usable;
///  language features then explain the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startLanguageWorker(root: string): LanguageWorker // throws
/// ```
fn start_language_worker(root: &Path) -> anyhow::Result<LanguageWorker> {
    enter_project_directory(root)?;
    servers_wanted()?;
    return LanguageWorker::new(root);
}

/// Run one project window after display-independent argument parsing has completed.
pub fn run(options: Options) -> anyhow::Result<()> {
    // What: The writer thread owns standard output, where the log has always gone. `_log_flush` is
    //       bound first, so it is dropped last, after every other value of this function, and waits
    //       briefly for the queued records.
    // Why: The window must never wait for a slow reader of its log, and the shutdown records written
    //      after the window closed must still reach the output (`src/logging/background.rs`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // using logFlush = startLogWriter(process.stdout); installLog(filterFromEnv(''), logFlush.writer);
    // ```
    let (log_writer, _log_flush) =
        background::background(std::io::stdout()).context("Cannot start the log writer thread")?;
    // Warnings and errors unless `RUST_LOG` asks for more; helix-lsp's healthy-server records are re-labelled.
    logging::install(logging::filter(""), log_writer, None)?;
    let workspace = Workspace::new(&options.project)?;
    // Helix roots every server at the working directory it reads first, so the project root
    // becomes the working directory before any thread starts or any Helix call is made.
    // What: `and_then` starts the worker only when entering the directory succeeded.
    // Why: Either failure leaves the window usable; language features then explain the reason.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let worker: LanguageWorker | Error;
    // try { process.chdir(root); worker = new LanguageWorker(root); } catch (error) { worker = error; }
    // ```
    let language_worker = start_language_worker(workspace.root());
    let project_root = workspace.root().to_path_buf();
    // Resolve initial-file paths relative to the explicit root, never the caller's ambient cwd.
    let file_path = if let Some(file) = options.file {
        Some(workspace.resolve(&file)?)
    } else {
        None
    };
    // What: if let extracts a present command-line argument without unwrap.
    // Why: A project without an initial file starts empty rather than displaying fabricated source.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (path !== undefined) { source = readFile(path); }
    // ```
    let (source, label) = if let Some(path) = &file_path {
        // What: ? returns an I/O failure to main, preserving its diagnostic.
        // Why: A failed open must not silently substitute the fixture.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // source = await readFile(path, 'utf8');
        // ```
        let metadata = match std::fs::metadata(path) {
            Ok(found) => found,
            Err(error) => {
                return Err(anyhow::Error::new(error)
                    .context(format!("Cannot inspect source file {}", path.display())));
            }
        };
        if !metadata.is_file() {
            bail!(
                "Cannot open source {}: it is not a regular file",
                path.display()
            );
        }
        let text = match std::fs::read_to_string(path) {
            Ok(read) => read,
            Err(error) => {
                return Err(anyhow::Error::new(error)
                    .context(format!("Cannot read UTF-8 source file {}", path.display())));
            }
        };
        (text, path.display().to_string())
    } else {
        (String::new(), String::new())
    };
    // What: `RuntimeSource::Embedded(&...)` lends the table compiled into this executable to the
    //       process-wide language runtime; `&` borrows it, and it lives as long as the program.
    // Why: Highlighting reads only these files, never Helix's runtime directories, and it must be
    //      chosen before the reload worker (bound below) prepares the first file. Both calls come
    //      after logging starts so their records are kept.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // runtime.install({ kind: 'embedded', runtime: EMBEDDED_RUNTIME });
    // ```
    ide_app::runtime::install(ide_app::runtime::RuntimeSource::Embedded(
        &crate::embedded_runtime::EMBEDDED_RUNTIME,
    ));
    // Renew this build's parser cache folder, then remove other builds' folders unused for 30 days.
    ide_app::runtime::tidy_cache(&crate::embedded_runtime::EMBEDDED_RUNTIME);
    // The app id must be stamped by the backend before the window below exists.
    crate::launcher::install_backend()?;
    let window = AppWindow::new()?;
    // The launcher has opened the device and recorded its texture size: the window is never larger than that.
    window.set_texture_limit(
        i32::try_from(ide_app::shaped_text::texture_dimension()).unwrap_or(i32::MAX),
    );
    window.set_source_available(file_path.is_some());
    let state = Rc::new(RefCell::new(State::new(&source, file_path)));
    window.set_file_label(SharedString::from(label));
    bind_pointer(&window, &state);
    bind_viewport(&window, &state);
    minimap::bind(&window, &state);
    frame_trace::bind(&window, &state);
    bind_keys(&window, &state);
    // Retain the timer until window shutdown; its Drop also closes and joins the worker.
    let _reload_timer = reload::bind(&window, &state)?;
    bind_appearance(&window, &state);
    // Retain the timer until window shutdown; its Drop also closes and joins the find worker.
    let _find_timer = find::bind(&window, &state)?;
    let (_navigation_timer, navigation_state) =
        navigation::bind_shared(&window, &state, workspace)?;
    let language_binding = language::bind(
        &window,
        &state,
        &navigation_state,
        &project_root,
        language_worker,
    );
    render(&window, &state);
    // Debug builds started with `IDE_INSPECT_ANNOTATIONS` show that file's hints and diagnostics; see `inspect`.
    // The timer of delayed arrivals, when the file asks for any, lives until the window closes.
    #[cfg(debug_assertions)]
    let _inspection = inspect::inject(&window, &state)?;
    if state.borrow().file_path.is_none() {
        window.invoke_focus_tree();
    }
    window.run()?;
    // The window is gone; stopping servers may take up to about a second without freezing it.
    language_binding.close();
    // What: Ok(()) reports success without a payload; Err would carry a failure.
    // Why: Clean window closure is not a process error.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return;
    // ```
    return Ok(());
}
