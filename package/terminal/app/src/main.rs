//! Thin Slint binary for the libghostty-vt terminal prototype.

/// What:
///      `mod stderr_filter;` declares a binary-local Rust module from
///           `src/stderr_filter.rs`.
/// Why:
///       Process stderr filtering is an app-shell concern,
///  not terminal library API.
mod stderr_filter;

/// What:
///      `mod slint_generated { ... }` creates a private namespace around
///           Rust emitted by Slint.
///  The lint attribute applies only inside that
///           namespace,
///  while package-owned Rust remains under the manifest's
///           denied `implicit_return` and shadow lints.
/// Why:
///       Slint 1.17 emits tail-expression returns and rebinds generated
///           helpers (`self_rc`,
///  `_self`,
///  `the_struct`,
///  ...) over each other.
///  Its
///           generated header already exempts several Clippy groups but not these
///           restriction lints,
///  so this boundary carries the exemption until Slint
///           includes them itself.
/// Gotcha:
///    The direct attribute on `slint::include_modules!()` is ignored by
///           rustc;
///  a module boundary is required for the lint level to apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namespace SlintGenerated {
///   export * from './app.slint.generated';
/// }
/// ```
#[allow(
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated
)]
mod slint_generated {
    // What:     `slint::include_modules!()` includes build-time generated Rust.
    // Why:      `AppWindow` and `TerminalCell` come from the Slint markup.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // export * from './app.slint.generated';
    // ```
    slint::include_modules!();
}

/// Imports every public Slint binding from the generated-only lint boundary.
use slint_generated::*;

/// What:
///      `use anyhow::Result;` imports `anyhow`'s one-parameter
///           application error result alias.
///  Sibling typed results name their
///           exact error type,
///  for example `Result<T, slint::PlatformError>`.
/// Why:
///       The binary and callbacks propagate several unrelated error types
///           through one user-facing error channel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Result<T> = T; // thrown Error objects carry failure details
/// ```
use anyhow::Result;

/// What:
///      `use std::cell::RefCell;` imports a single-thread interior-mutability
///           wrapper.
///  Sibling `Cell` handles `Copy` values only;
///  `Mutex` is for
///           cross-thread mutation.
/// Why:
///       Slint callbacks share one engine and mutate it on the UI thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const engineRef = { current: engine };
/// ```
use std::cell::RefCell;

/// What:
///      `use std::rc::Rc;` imports single-thread reference counting.
///  Sibling
///           `Arc` is atomic and thread-safe but unnecessary on the UI thread.
/// Why:
///       Multiple Slint callbacks need to share the same `RefCell` engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sharedEngine = engine;
/// ```
use std::rc::Rc;

/// What:
///      `use std::sync::mpsc` imports Rust's multi-producer single-consumer
///           channel module.
///  The sibling `sync_channel` variant adds backpressure.
/// Why:
///       The PTY reader thread sends byte events to the Slint UI thread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { sender, receiver } = channel();
/// ```
use std::sync::mpsc;

/// What:
///      `use std::time::Duration;` imports a time-span type.
///  Sibling integer
///           milliseconds would be less explicit at timer call sites.
/// Why:
///       The Slint timer needs a typed polling interval.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const duration = Duration.fromMillis(16);
/// ```
use std::time::Duration;

/// What:
///      `use i_slint_backend_winit::Backend;` imports Slint's winit backend
///           builder type.
/// Why:
///       The app installs a window-attributes hook for the Wayland app id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Backend } from "slint-winit";
/// ```
use i_slint_backend_winit::Backend;

/// What:
///      `use slint::{Color, ComponentHandle, SharedString, VecModel};` imports
///           Slint runtime helpers.
///  `Color` maps to `.slint color`;
///  `VecModel`
///           backs array properties.
/// Why:
///       Rust converts engine snapshots into Slint properties.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Color, VecModel } from "slint";
/// ```
use slint::{Color, ComponentHandle, SharedString, Timer, TimerMode, VecModel};

/// What:
///      `use terminal_app::{...};` imports this package's library modules.
///           Cargo package `terminal` exposes the lib crate as `terminal_app`.
/// Why:
///       The binary stays thin and delegates VT/render logic to the library.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { encodeTerminalKey, TerminalEngine, PtySession, ViewportGeometry } from "terminal-app";
/// ```
use terminal_app::{
    engine::{TerminalEngine, ViewportGeometry},
    input::encode_terminal_key,
    launcher,
    pty::{PtyEvent, PtySession},
    render::{Rgb, TerminalSnapshot},
};

/// What:
///      `const MAX_SCROLLBACK_ROWS: usize = 10_000;` declares a row-count
///           constant.
///  `usize` is chosen over `u32` because Ghostty returns `usize`.
/// Why:
///       Demo content can keep history while avoiding unbounded scrollback.
const MAX_SCROLLBACK_ROWS: usize = 10_000;

/// What:
///      `const OUTPUT_POLL_INTERVAL_MS: u64 = 16;` declares the PTY output
///           polling interval.
///  `u64` is what `Duration::from_millis` expects.
/// Why:
///       Around 60Hz keeps prompt/output latency low without busy waiting.
const OUTPUT_POLL_INTERVAL_MS: u64 = 16;

/// What:
///      `fn install_backend() -> Result<()>` builds and installs the explicit
///           Slint platform backend,
///  returning `anyhow::Error` on failure.
/// Why:
///       The Wayland app id must be stamped before the window is created.
fn install_backend() -> Result<()> {
    // What:     `if std::env::var_os("SLINT_MCP_PORT").is_some() { return Ok(()); }`
    //           bail out WITHOUT installing an explicit platform.
    // Why:      Slint 1.17's embedded MCP server (the `slint/mcp` feature activated by
    //           SLINT_MCP_PORT, used by the `mcp` mise task for agent-driven UI
    //           testing) only starts when Slint creates the backend itself through its
    //           selector. Calling `set_platform` here would bypass that and leave the
    //           server unbound (and ignore SLINT_BACKEND=headless). Skipping lets Slint
    //           pick the backend, honoring SLINT_BACKEND; only the app-id hook is
    //           dropped, and only for that test-only run. Production runs (port unset)
    //           are unaffected.
    if std::env::var_os("SLINT_MCP_PORT").is_some() {
        return Ok(());
    }
    // What:     `let mut builder = Backend::builder().with_window_attributes_hook(...)`
    //           creates a mutable backend builder with the app-id hook installed.
    // Why:      The hook runs at native window creation time.
    let mut builder = Backend::builder().with_window_attributes_hook(launcher::set_window_app_id);
    // What:     `std::env::var("SLINT_BACKEND")` reads an environment variable and
    //           returns `Result<String, VarError>`. `.map(...).unwrap_or(false)`
    //           converts missing env to `false`.
    // Why:      Preserve the sibling app's software-renderer escape hatch.
    let force_software = std::env::var("SLINT_BACKEND")
        .map(|value| return value.contains("software"))
        .unwrap_or(false);
    // What:     `if force_software { ... }` conditionally changes the builder.
    // Why:      Headless or no-GPU sessions can force Slint's software renderer;
    //           winit expects the renderer token `software`, not Cargo's
    //           `renderer-software` feature name.
    if force_software {
        builder = builder.with_renderer_name("software");
    }
    // What:     `let backend = builder.build()?` finishes backend construction and
    //           propagates platform errors with `?`.
    // Why:      Backend creation can fail without a display server.
    let backend = builder.build()?;
    // What:     `slint::platform::set_platform(Box::new(backend)).expect(...)`
    //           boxes the backend trait object and installs it globally.
    // Why:      The next `AppWindow::new()` must use this backend and hook.
    slint::platform::set_platform(Box::new(backend))
        .expect("no Slint platform should already be set");
    // What:     `Ok(())` returns success from a unit-returning `Result`.
    // Why:      The platform is installed.
    return Ok(())
}

/// What:
///      `fn color_from_rgb(rgb: Rgb) -> Color` converts this crate's color
///           record into Slint's runtime color value.
/// Why:
///       Generated Slint structs expect `slint::Color` fields.
fn color_from_rgb(rgb: Rgb) -> Color {
    // What:     `Color::from_rgb_u8(...)` constructs a Slint color from 8-bit
    //           channels. `::` calls an associated function.
    // Why:      Both Ghostty and Slint use byte RGB for this path.
    return Color::from_rgb_u8(rgb.red, rgb.green, rgb.blue)
}

/// What:
///      `fn to_slint_cell(cell: &terminal_app::render::TerminalCell) -> TerminalCell`
///           borrows one engine cell and returns one generated Slint cell.
/// Why:
///       Keep generated-type construction in one place.
fn to_slint_cell(cell: &terminal_app::render::TerminalCell) -> TerminalCell {
    // What:     `TerminalCell { ... }` constructs the Slint-generated struct.
    //           `as i32` narrows `usize` indexes to Slint `int`; `.as_str().into()`
    //           converts owned `String` to `SharedString`.
    // Why:      Slint model values must use Slint's generated field types.
    return TerminalCell {
        row: cell.row as i32,
        col: cell.col as i32,
        text: SharedString::from(cell.text.as_str()),
        foreground: color_from_rgb(cell.foreground),
        background: color_from_rgb(cell.background),
        bold: cell.bold,
        italic: cell.italic,
        inverse: cell.inverse,
        underline: cell.underline,
    }
}

/// What:
///      `fn apply_snapshot(app: &AppWindow, snapshot: TerminalSnapshot)` borrows
///           the Slint window and consumes one owned engine snapshot.
/// Why:
///       One function updates every UI property derived from a render frame.
fn apply_snapshot(app: &AppWindow, snapshot: TerminalSnapshot) {
    // What:     `let cells: Vec<TerminalCell> = ...collect()` maps engine cells to
    //           generated Slint cells and gathers them into a vector.
    // Why:      Slint array properties are backed by Rust models.
    let cells: Vec<TerminalCell> = snapshot.cells.iter().map(to_slint_cell).collect();
    // What:     `Rc::new(VecModel::from(cells)).into()` wraps the vector in a Slint
    //           model. `Rc` is required by Slint's `ModelRc` conversion.
    // Why:      Generated setters expect model-backed arrays, not raw `Vec`s.
    app.set_cells(Rc::new(VecModel::from(cells)).into());
    app.set_total_rows(snapshot.total_rows as i32);
    app.set_scrollback_rows(snapshot.scrollback_rows as i32);
    app.set_viewport_rows(snapshot.viewport_rows as i32);
    app.set_viewport_cols(snapshot.viewport_cols as i32);
    app.set_whole_row_offset(snapshot.whole_row_offset as i32);
    app.set_content_height(snapshot.total_rows as f32 * snapshot.cell_height_px);
    // What:     `format!(...)` builds a status string with row, pixel, and title
    //           data. The title falls back to a demo label when no OSC title exists.
    // Why:      The prototype visibly documents the row/fraction bridge.
    let status = format!(
        "libghostty-vt  {}x{}  scrollback={}  row={}  fractional={:.1}px  {}",
        snapshot.viewport_cols,
        snapshot.viewport_rows,
        snapshot.scrollback_rows,
        snapshot.whole_row_offset,
        snapshot.fractional_px,
        if snapshot.title.is_empty() {
            "interactive PTY shell"
        } else {
            snapshot.title.as_str()
        },
    );
    app.set_status(SharedString::from(status.as_str()));
}

/// What:
///      `fn refresh_from_scroll(...) -> Result<()>` handles one Slint
///           pixel-scroll notification,
///  returning `anyhow::Error` on failure.
/// Why:
///       It keeps callback bodies short and testable by inspection.
fn refresh_from_scroll(
    app: &AppWindow,
    engine_cell: &Rc<RefCell<TerminalEngine>>,
    pixel_scroll: f32,
) -> Result<()> {
    // What:     `engine_cell.borrow_mut()` takes a checked mutable borrow from `RefCell`.
    // Why:      The shared `Rc` engine can still be mutated inside callbacks. The cell
    //           parameter carries the container name so the borrow keeps `engine`.
    let mut engine = engine_cell.borrow_mut();
    // What:     `let mapping = engine.set_pixel_scroll(pixel_scroll)?` maps Slint
    //           pixels and scrolls Ghostty's whole-row viewport.
    // Why:      This is the bridge required by the prototype.
    let mapping = engine.set_pixel_scroll(pixel_scroll)?;
    // What:     `let snapshot = engine.snapshot(mapping)?` extracts visible cells.
    // Why:      Slint redraws from owned snapshot data.
    let snapshot = engine.snapshot(mapping)?;
    // What:     `drop(engine)` releases the `RefCell` mutable borrow before setting
    //           UI properties that could trigger callbacks.
    // Why:      Avoid nested mutable borrows if Slint emits another notification.
    drop(engine);
    apply_snapshot(app, snapshot);
    return Ok(())
}

/// What:
///      `fn refresh_from_resize(...) -> Result<()>` handles Slint
///           viewport-size and cell-metric notifications through `anyhow`.
/// Why:
///       Resize support must use the same measured font metrics that Slint uses
///           for cell placement.
fn refresh_from_resize(
    app: &AppWindow,
    engine_cell: &Rc<RefCell<TerminalEngine>>,
    pty: &Rc<RefCell<PtySession>>,
    width_px: f32,
    height_px: f32,
    cell_width_px: f32,
    cell_height_px: f32,
) -> Result<()> {
    let geometry = ViewportGeometry::from_pixels(
        width_px,
        height_px,
        cell_width_px,
        cell_height_px,
    );
    let mut engine = engine_cell.borrow_mut();
    engine.resize(geometry)?;
    pty.borrow().resize(geometry)?;
    let pixel_scroll = 0.0 - app.get_scroll_y();
    let mapping = engine.set_pixel_scroll(pixel_scroll)?;
    let snapshot = engine.snapshot(mapping)?;
    drop(engine);
    apply_snapshot(app, snapshot);
    return Ok(())
}

/// What:
///      `fn refresh_from_pty_events(...) -> Result<()>` drains PTY reader
///           events and refreshes the Slint model through `anyhow`.
/// Why:
///       The background reader cannot touch `TerminalEngine`,
///  so the UI thread
///           feeds Ghostty from this timer callback.
fn refresh_from_pty_events(
    app: &AppWindow,
    engine_cell: &Rc<RefCell<TerminalEngine>>,
    receiver: &mpsc::Receiver<PtyEvent>,
) -> Result<()> {
    // What:     `let mut engine = engine_cell.borrow_mut()` takes a checked mutable
    //           borrow of the UI-thread terminal engine. The cell parameter carries the
    //           container name so the borrow keeps the plain `engine` name.
    // Why:      Feeding PTY bytes mutates Ghostty state and render state.
    let mut engine = engine_cell.borrow_mut();
    // What:     `let mut saw_output = false` tracks whether any bytes arrived.
    // Why:      EOF or error events update status but do not require a snapshot.
    let mut saw_output = false;
    // What:     `let mut reader_message = None` stores the latest reader status.
    // Why:      The status line can show PTY closure or read failure after draining.
    let mut reader_message = None;
    // What:     `for event in receiver.try_iter()` drains all queued PTY events without
    //           blocking the Slint event loop.
    // Why:      One timer tick should coalesce bursts into one render refresh.
    for event in receiver.try_iter() {
        // What:     `match event` branches on output vs reader lifecycle messages.
        // Why:      Output feeds Ghostty; stopped messages update the UI status.
        match event {
            // What:     `PtyEvent::Output(bytes)` moves an owned PTY byte chunk.
            // Why:      Ghostty parses exactly these bytes as terminal output.
            PtyEvent::Output(bytes) => {
                engine.feed(bytes.as_slice())?;
                saw_output = true;
            }
            // What:     `PtyEvent::ReaderStopped(message)` moves a reader status string.
            // Why:      The terminal can tell the user when the shell exits or reading fails.
            PtyEvent::ReaderStopped(message) => {
                reader_message = Some(message);
            }
        }
    }
    // What:     `if !saw_output { ... }` handles ticks with only lifecycle events.
    // Why:      Avoid redundant render extraction when no terminal bytes arrived.
    if !saw_output {
        drop(engine);
        if let Some(message) = reader_message {
            app.set_status(SharedString::from(message.as_str()));
        }
        return Ok(());
    }
    // What:     `let bottom_scroll_px = ...` computes the pixel offset for the active
    //           bottom viewport with Slint's current measured cell height.
    // Why:      New terminal output should keep the prompt visible.
    let bottom_scroll_px = engine.scrollback_rows()? as f32 * app.get_effective_cell_height();
    // What:     `engine.set_pixel_scroll(bottom_scroll_px)?` syncs Ghostty's whole-row
    //           viewport to the bottom row.
    // Why:      Slint and Ghostty must agree before rendering the snapshot.
    let mapping = engine.set_pixel_scroll(bottom_scroll_px)?;
    // What:     `engine.snapshot(mapping)?` extracts visible cells after all PTY output.
    // Why:      One snapshot per timer tick keeps output bursts efficient.
    let snapshot = engine.snapshot(mapping)?;
    // What:     `drop(engine)` releases the mutable engine borrow before UI property sets.
    // Why:      Slint setters may trigger callbacks that need the engine again.
    drop(engine);
    // What:     `apply_snapshot(app, snapshot)` updates all render properties.
    // Why:      Slint redraws from the owned cell model.
    apply_snapshot(app, snapshot);
    // What:     `app.set_scroll_y(0.0 - bottom_scroll_px)` moves Slint's Flickable to bottom.
    // Why:      The prompt should stay visible as the shell produces output.
    app.set_scroll_y(0.0 - bottom_scroll_px);
    // What:     `if let Some(message) = reader_message { ... }` applies lifecycle status
    //           after rendering, if one arrived in the same tick.
    // Why:      Shell-exit text should not be overwritten by `apply_snapshot`.
    if let Some(message) = reader_message {
        app.set_status(SharedString::from(message.as_str()));
    }
    // What:     `Ok(())` returns success.
    // Why:      All ready PTY events were handled.
    return Ok(())
}

/// What:
///      `fn write_terminal_key(...) -> Result<()>` converts one Slint key
///           callback into PTY bytes through the `anyhow` error channel.
/// Why:
///       Keyboard input should reach the spawned shell.
fn write_terminal_key(
    pty: &Rc<RefCell<PtySession>>,
    key_text: SharedString,
    control: bool,
    alt: bool,
) -> Result<()> {
    // What:     `encode_terminal_key(...)` returns bytes or `None` for ignored keys.
    // Why:      Modifier-only and unknown non-printable events should not hit the shell.
    if let Some(bytes) = encode_terminal_key(key_text.as_str(), control, alt) {
        // What:     `pty.borrow_mut().write_bytes(bytes.as_slice())?` mutably borrows the
        //           PTY writer and writes the encoded bytes.
        // Why:      The shell receives keyboard input through the PTY master.
        pty.borrow_mut().write_bytes(bytes.as_slice())?;
    }
    // What:     `Ok(())` returns success.
    // Why:      The key was either written or intentionally ignored.
    return Ok(())
}

/// What:
///      `fn log_callback_error(context: &str, error: anyhow::Error)` logs
///           fallible callback work to stderr.
/// Why:
///       Slint callbacks cannot return errors to the event loop.
fn log_callback_error(context: &str, error: anyhow::Error) {
    // What:     `tracing::error!(...)` emits a structured error event to the subscriber.
    // Why:      Prototype failures should be visible when launched from a terminal.
    tracing::error!(context, error = %error, "callback error");
}

/// What:
///      `fn main() -> Result<()>` is the binary entry point using `anyhow`.
///           Returning `Result` lets failures become process errors.
/// Why:
///       No manual `process.exit` style path is needed.
fn main() -> Result<()> {
    // Install the stderr tracing subscriber (RUST_LOG, default info) before backend setup.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| return tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    install_backend()?;
    let app = AppWindow::new()?;
    // What:     `app.get_effective_cell_width()` reads the cell width measured by
    //           Slint from the current system monospace font.
    // Why:      The initial Ghostty geometry must match the UI's actual cell placement.
    let initial_cell_width_px = app.get_effective_cell_width();
    // What:     `app.get_effective_cell_height()` reads the Slint cell height.
    // Why:      Initial resize and scrolling should use one shared metric source.
    let initial_cell_height_px = app.get_effective_cell_height();
    let initial_geometry = ViewportGeometry {
        cols: 80,
        rows: 24,
        cell_width_px: initial_cell_width_px,
        cell_height_px: initial_cell_height_px,
    };
    let mut engine = TerminalEngine::new(initial_geometry, MAX_SCROLLBACK_ROWS)?;
    let (pty_sender, pty_receiver) = mpsc::channel();
    let pty = PtySession::spawn_shell(initial_geometry, pty_sender)?;
    // What:     `stderr_filter::install_ghostty_stderr_filter()?` redirects process
    //           stderr through a line filter after the child shell is already spawned.
    // Why:      Ghostty's debug-only `unimplemented OSC callback` lines should vanish,
    //           while the child shell should not inherit the filter's backup fd.
    stderr_filter::install_ghostty_stderr_filter()?;
    let initial_mapping = engine.set_pixel_scroll(0.0)?;
    let initial_snapshot = engine.snapshot(initial_mapping)?;
    apply_snapshot(&app, initial_snapshot);
    app.set_scroll_y(0.0);
    // The shared cells get their own names so the wrapped values keep the plain
    // names up to the move and the per-callback clones below never shadow them.
    let shared_engine = Rc::new(RefCell::new(engine));
    let shared_pty = Rc::new(RefCell::new(pty));

    let weak_for_scroll = app.as_weak();
    app.on_scroll_changed({
        let scroll_engine = Rc::clone(&shared_engine);
        move |pixel_scroll| {
            if let Some(window) = weak_for_scroll.upgrade()
                && let Err(error) = refresh_from_scroll(&window, &scroll_engine, pixel_scroll) {
                    log_callback_error("scroll refresh failed", error);
                }
        }
    });

    let weak_for_resize = app.as_weak();
    app.on_viewport_resized({
        let resize_engine = Rc::clone(&shared_engine);
        let resize_pty = Rc::clone(&shared_pty);
        move |width_px, height_px, cell_width_px, cell_height_px| {
            if let Some(window) = weak_for_resize.upgrade()
                && let Err(error) = refresh_from_resize(
                    &window,
                    &resize_engine,
                    &resize_pty,
                    width_px,
                    height_px,
                    cell_width_px,
                    cell_height_px,
                ) {
                    log_callback_error("resize refresh failed", error);
                }
        }
    });

    app.on_terminal_key({
        let key_pty = Rc::clone(&shared_pty);
        move |key_text, control, alt| {
            if let Err(error) = write_terminal_key(&key_pty, key_text, control, alt) {
                log_callback_error("terminal input write failed", error);
            }
        }
    });

    let output_timer = Timer::default();
    let weak_for_output = app.as_weak();
    output_timer.start(
        TimerMode::Repeated,
        Duration::from_millis(OUTPUT_POLL_INTERVAL_MS),
        {
            let output_engine = Rc::clone(&shared_engine);
            move || {
                if let Some(window) = weak_for_output.upgrade()
                    && let Err(error) = refresh_from_pty_events(&window, &output_engine, &pty_receiver) {
                        log_callback_error("PTY output refresh failed", error);
                    }
            }
        },
    );

    app.run()?;
    output_timer.stop();
    return Ok(())
}
