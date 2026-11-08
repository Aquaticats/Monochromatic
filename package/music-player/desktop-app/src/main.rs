//! Binary entry point.
//!  Builds the Slint window,
//!  spawns the engine on its own
//! thread,
//!  and wires the two together:
//!  UI callbacks send `Command`s to the
//! engine,
//!  and engine `Update`s are applied to the window's properties from the
//! event-loop thread.
//!  Also handles CLI path arguments and the file-open dialog.
//!
//! On Windows the executable is linked for the GUI subsystem,
//! so a double-click opens no console window;
//!  `main` attaches to the console of the parent process first,
//! so `--version` and the logs still print in a terminal
//! (`src/console_attach.rs`,
//! `doc/decision/music-player-windows-gui-subsystem.md`).
//!  The unit-test build of this binary keeps the console subsystem so the test harness prints as usual.
#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

/// What:
///      `mod slint_generated;` loads the sibling `slint_generated.rs` module,
///           the private lint boundary around Rust emitted by Slint.
///  That file
///           carries the generated-code Clippy allowances,
///  while package-owned
///           Rust remains under the manifest's denied `implicit_return` and
///           shadow lints.
/// Why:
///       Slint's generated output needs restriction-lint exemptions that must
///           not leak into maintained modules,
///  and keeping the boundary in its
///           own file keeps those exemption lines out of this file's max-lines
///           budget;
///  see
///           `doc/troubleshooting/slint-generated-rust-implicit-return.md`.
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
mod slint_generated;

/// Imports every public Slint binding from the generated-only lint boundary.
use slint_generated::*;

/// What:
///      `mod ui_progress;` loads the sibling `ui_progress.rs` module into this
///           binary crate.
/// Why:
///       The progress debounce bridge uses generated Slint types,
///  so it belongs
///           beside `main.rs`,
///  not in the reusable library crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiProgress from "./ui_progress";
/// ```
mod ui_progress;

/// Playback mode conversion and displayed-page scope projection.
mod ui_playback;

/// Window binding for the library screen:
/// applies engine updates to the UI-thread state and pushes the projected view into `AppWindow`.
mod ui_screen;

/// Pure conversions from the shared player-content model into the flat track list and the folder picker lists that the window shows.
mod ui_screen_rows;

/// Plain-language message for an event the user did not ask about:
/// a file that left the folder.
mod ui_notices;

/// Page selection on the screen state:
/// the cached page index,
/// the selected page,
/// and the pages' measures.
mod ui_screen_pages;

/// Scroll requests for the track list:
/// which projection scrolls the list,
/// and to which row.
mod ui_screen_scroll;

/// Coalesced projection for updates that arrive in bursts.
mod ui_screen_throttle;

/// Pure library screen state and its projection into the window's view model.
mod ui_screen_view;

/// Window binding for the Licenses page that the Licenses row of Settings opens.
mod ui_licenses;

/// Window binding for the Search route:
/// opens and closes it,
/// sends each query edit to the Search service,
/// and activates a result.
mod ui_search;

/// Window binding for the Settings page and the template editor.
mod ui_settings;

/// Pure models for the Settings template rows and the template editor page.
mod ui_settings_model;

/// Window binding for the Text size setting:
/// the stepper and reset callbacks of Settings,
/// and the `TextSize` global.
mod ui_text_size;

/// Pure template helpers for the desktop window:
/// track fields from display paths,
/// the playing-track subtitle,
/// and the offset conversion the template editor needs.
mod ui_templates;

/// Pure Search page model:
/// the rows,
/// emphasis segments and status text for one query.
mod ui_search_model;

/// Scroll anchoring for Search results that are re-ranked while the user looks at them (D111).
mod ui_search_anchor;

/// Applying Search answers to the window,
/// in place.
mod ui_search_apply;

/// Window callbacks for the library screen:
/// each user action becomes engine commands.
mod ui_wire;

/// Platform file actions behind the track menu:
/// show in the file manager,
/// copy the file name,
/// move to the trash,
/// and restore from the trash for Undo.
mod ui_files;

/// Keyboard shortcuts for the player:
/// each key becomes a `ShortcutRequest`,
/// the `keymap` classifier names the action,
/// and the action runs against the window and the host (D102,
/// D106).
mod ui_keys;

/// What each classified shortcut does (D102,
/// D106).
mod ui_keys_actions;

/// What:
///      `mod ui_keys_row_actions;` loads the sibling `ui_keys_row_actions.rs`:
/// what the shortcuts of a focused track row and of the track menu do.
/// Why:
///       Row and menu actions share the menu ring,
/// and they fill a file of their own beside the other action groups.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiKeysRowActions from "./ui_keys_row_actions";
/// ```
mod ui_keys_row_actions;

/// Requests for keyboard focus from the Rust side (Alt+1,
/// Alt+2,
/// Escape out of a field).
mod ui_keys_focus;

/// The host of keyboard actions:
/// what a shortcut can ask of the application beyond the window.
mod ui_host;

/// The keyboard ring inside the folder picker and the track list (D106).
mod ui_lists;

/// Key hint badges and shortcut texts on the window (D106).
mod ui_hints;

/// The window's observable state as one log line,
/// for the end-to-end run.
mod ui_state_log;

/// Shared helpers of the window tests that press real key events.
#[cfg(test)]
mod ui_test_support;

/// Window tests of the focus model:
/// Tab order,
/// rings and the lists' focus keys.
#[cfg(test)]
mod ui_keys_focus_tests;

/// Window tests of every D106 shortcut,
/// pressed as real key events on the testing backend.
#[cfg(test)]
mod ui_keys_window_tests;

/// Window tests of Auto-play on select (D129):
/// the Settings switch row and what a click,
/// Enter and a Search result do.
#[cfg(test)]
mod ui_auto_play_window_tests;

/// Window tests of the Text size setting:
/// the steppers,
/// the reset row,
/// the live window,
/// the saves and the layout at the
/// ends of the range.
#[cfg(test)]
mod ui_text_size_window_tests;

/// Window tests of the Licenses row of Settings and the Licenses page:
/// the row,
/// the AboutSlint widget,
/// the keys and the ring.
#[cfg(test)]
mod ui_licenses_window_tests;

/// Window tests of the empty,
/// reading and failure states.
#[cfg(test)]
mod ui_states_window_tests;

/// Window tests of the second polish review's interaction defects:
/// click-through,
/// menu position,
/// rings,
/// headers,
/// fades.
#[cfg(test)]
mod ui_interaction_window_tests;

/// Window tests of the list defects of the second polish review:
/// long names,
/// picker reveal,
/// breakpoint scroll,
/// scrollbars.
#[cfg(test)]
mod ui_lists_window_tests;

/// Window tests of the folded folder picker:
/// it never opens blank,
/// whatever the library,
/// the saved selection or the size.
#[cfg(test)]
mod ui_picker_window_tests;

/// Rendered tests of the list defects:
/// the ellipsis,
/// the playing-row marker and the scrollbar,
/// read from pixels.
#[cfg(test)]
mod ui_lists_render_tests;

/// Shared helpers of the rendered window tests:
/// the software-rendered window,
/// its pixels and rounded-box geometry.
#[cfg(test)]
mod ui_render_support;

/// Tests of the shared render support:
/// its clock step against the animation times written in the markup.
#[cfg(test)]
mod ui_render_support_tests;

/// Rendered tests of the focus ring on the player page (D117):
/// its band,
/// its shape and what focus leaves unchanged.
#[cfg(test)]
mod ui_focus_ring_tests;

/// Rendered tests of the focus ring on the other pages (D117) and of the menu card's color (D119).
#[cfg(test)]
mod ui_focus_ring_pages_tests;

/// Rendered tests of the Search page restyle (D126):
/// the canvas header,
/// the divider,
/// the rounded rows and the status block.
#[cfg(test)]
mod ui_search_restyle_tests;

/// Rendered tests of the track menu:
/// the ring on a row (D117) and the card's color and outline (D119).
#[cfg(test)]
mod ui_focus_ring_menu_tests;

/// Rendered tests of the focus ring component alone:
/// its shapes,
/// and a ring above a clipping parent.
#[cfg(test)]
mod ui_focus_ring_fixture_tests;

/// Window binding for the track menu:
/// opens it for a row,
/// and runs the chosen action.
mod ui_menu;

/// Pure track menu model:
/// the rows,
/// icons and dividers for one track.
mod ui_menu_model;

/// Window binding for the feedback messages and the first-run pane.
mod ui_overlays;

/// Pure models for the first-run pane and the feedback messages.
mod ui_overlays_model;

/// Headless layout tests for the composed window at the cover and unfolded widths.
#[cfg(test)]
#[path = "ui_layout_tests.rs"]
mod ui_layout_tests;

/// Synthetic libraries for the scale tests;
/// the library crate compiles the same file.
#[cfg(test)]
#[path = "scale_library_tests.rs"]
mod scale_library_tests;

/// Scale tests of the window projection and the Search service at 1k to 1M tracks (`src/scale_tests.rs`).
#[cfg(test)]
mod scale_tests;

/// Search page tests with the testing backend:
/// refresh in place while a slow scan adds tracks (D111).
#[cfg(test)]
#[path = "ui_search_ingest_tests.rs"]
mod ui_search_ingest_tests;

/// What:
///      `mod ui_theme;` loads the sibling `ui_theme.rs` module.
/// Why:
///       It writes the OKLCH-derived theme roles into the generated `Theme` global and follows accent changes,
///           so it needs the generated window type and stays out of `main.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiTheme from "./ui_theme";
/// ```
mod ui_theme;

/// What:
///      `mod main_startup_helpers;` loads the sibling `main_startup_helpers.rs`:
/// the music folder lookup,
/// the color scheme override and the two system notifiers.
/// Why:
///       They answer startup questions from the environment,
/// so they stay out of `main.rs`,
/// which wires the window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as mainStartupHelpers from "./main_startup_helpers";
/// ```
mod main_startup_helpers;

/// Imports the startup lookups and constructors that `main` calls.
use main_startup_helpers::{color_scheme_from_env, music_dir, start_analysis_notifier, start_unplayable_notifier};

/// What:
///      `mod ui_font_scale;` loads the sibling `ui_font_scale.rs` module.
/// Why:
///       The OS-font-tracking scale handler uses the generated `AppWindow`,
///  so it
///           belongs beside `main.rs`;
///  splitting it out also keeps `main.rs` under the
///           max-lines limit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as uiFontScale from "./ui_font_scale";
/// ```
mod ui_font_scale;

/// What:
///      `#[cfg(test)] #[path = "ui_binding_tests.rs"] mod ui_binding_tests;` loads
///           the headless UI regression tests,
///  compiled ONLY under `cargo test` /
///           `cargo nextest run`.
///  `#[path]` names the sibling file explicitly because
///           the module name differs from the default `ui_binding_tests/mod.rs` lookup.
/// Why:
///       They instantiate `AppWindow` (in scope here from `include_modules!`) and
///           drive its Sliders via `i-slint-backend-testing`,
///  so they belong beside
///           `main.rs` in the binary crate,
///  not in the reusable library crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // test-only import, tree-shaken from production builds
/// import "./ui_binding_tests.test";
/// ```
/// What:
///      `use std::path::PathBuf;`.
///  The OWNED filesystem path type:
///  a heap-
///           allocated,
///  growable path buffer.
///  Sibling:
///  `&Path`,
///  a BORROWED view
///           that does not own its bytes (the `String` vs `&str` distinction,
///  but
///           for paths).
/// Why:
///       Picked folders and the music dir become owned `PathBuf`s,
///  and the two
///           path helpers below return `Option<PathBuf>`;
///  `PathBuf` (not `&Path`)
///           because these paths outlive the calls that produce them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no import needed; a path is just a string
/// ```
use std::path::PathBuf;

/// What:
///      `use anyhow::Result;` imports `anyhow`'s one-parameter application
///           result alias.
///  Sibling typed results name exact error types like
///           `slint::PlatformError`.
/// Why:
///       Startup and event-loop failures share one user-facing error channel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Result<T> = T; // failures throw Error objects
/// ```
use anyhow::Result;

/// What:
///      `use std::rc::Rc;`.
///  `Rc<T>` is a single-threaded shared-ownership
///           pointer (reference counted).
///  Sibling:
///  `Arc<T>` (atomic refcount,
///  safe
///           to share across threads);
///  `Box<T>` (single owner,
///  no sharing).
/// Why:
///       Several UI callbacks need to share the one `Engine`;
///  they all run on
///           the UI thread,
///  so non-atomic `Rc` is enough (and cheaper than `Arc`'s
///           atomic counter),
///  and we never need `Box`'s single-owner model.
/// Gotcha:
///    `Rc` is NOT thread-safe;
///  sending one across threads does not compile.
///           The cross-thread sharing below uses `Arc` instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const engine = new Engine(); // closures just capture it; GC handles sharing
/// ```
use std::rc::Rc;

/// What:
///      `use std::sync::{Arc, Mutex};`.
///  `Arc<T>` is a thread-safe shared owner
///           (atomic refcount;
///  sibling:
///  single-thread `Rc<T>`),
///  and `Mutex<T>` is a
///           lock that lets one thread mutate `T` at a time (sibling:
///  `RwLock<T>`,
///           many readers OR one writer).
/// Why:
///       The engine update callback must be `Send`,
///  so progress debounce state
///           cannot be an `Rc`;
///  an `Arc<Mutex<_>>` crosses into the UI callback
///           safely and still mutates only one small state object (no need for
///           `RwLock`'s reader/writer split).
/// Gotcha:
///    a `Mutex` in Rust WRAPS the data it guards;
///  you reach the value only
///           by locking.
///  There is no "forgot to lock" path like a bare JS object.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const progressDebouncer = new ProgressDebouncer(); // GC + single thread: no lock
/// ```
use std::sync::{Arc, Mutex};

/// What:
///      `use std::time::Instant;`.
///  `Instant` is a monotonic timestamp (only
///           ever moves forward).
///  Sibling:
///  `Duration`,
///  the elapsed span produced by
///           `Instant::elapsed`;
///  `SystemTime`,
///  the wall clock that can jump.
/// Why:
///       Progress debounce decisions use elapsed time since startup,
///  which needs
///           the monotonic `Instant`,
///  not the jumpy `SystemTime`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startedAt = performance.now(); // monotonic clock
/// ```
use std::time::Instant;

/// What:
///      `use music_player::command::{Command, PlaybackMode, Update};`.
///  The
///           message types from our library crate.
///  The package is `music-player`
///           but a Rust crate identifier cannot contain `-`,
///  so the lib crate is
///           `music_player` (the hyphen becomes an underscore).
/// Why:
///       We build `Command`s and read `Update`s.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Command, PlaybackMode, Update } from "music-player/command";
/// ```
use music_player::command::{Command, Update};

/// What:
///      `use music_player::cli::Cli;`.
///  The clap-derived argument-parser struct
///           from our library crate (its fields are `start_playing` and `paths`).
/// Why:
///       `main` calls `Cli::parse()` to turn the command line into that struct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Cli } from "music-player/cli";
/// ```
/// The Windows build parses through `cli::parse_with_color` instead,
/// so it names neither `Cli` nor the trait.
#[cfg(not(windows))]
use music_player::cli::Cli;

/// What:
///      `use music_player::engine::Engine;`.
///  The controller handle.
/// Why:
///       We spawn it and send commands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Engine } from "music-player/engine";
/// ```
use music_player::engine::Engine;

/// What:
///      `use music_player::progress::ProgressDebouncer;`.
///  The pure debounce
///           state shared with the binary-only UI bridge.
/// Why:
///       The binary owns the state object;
///  `ui_progress` owns the Slint wiring.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ProgressDebouncer } from "music-player/progress";
/// ```
use music_player::progress::ProgressDebouncer;

/// What:
///      `use music_player::session::Session;`.
///  The saved-state record.
/// Why:
///       We load it on launch to restore the last session.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Session } from "music-player/session";
/// ```
use music_player::session::Session;

/// What:
///      `use music_player::launcher::{self, Launcher};`.
///  The desktop-shell
///           integration:
///  `self` re-imports the MODULE itself (so `launcher::set_window_app_id`
///           still resolves),
///  and `Launcher` pulls in the struct that emits KDE
///           taskbar progress.
/// Why:
///       `main` installs the app-id hook via the module path and constructs a
///           `Launcher`,
///  so it needs both the module and the type in scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as launcher from "music-player/launcher";
/// import { Launcher } from "music-player/launcher";
/// ```
use music_player::launcher::{self, Launcher};

/// What:
/// `use music_player::sweep_gate::SweepGate;`.
/// The pause gate of the background analysis sweep.
/// Why:
/// the indicator's Pause and Resume buttons stop and restart the sweep workers through it (D107).
use music_player::sweep_gate::SweepGate;

/// What:
///      `use clap::Parser;`.
///  The `Parser` TRAIT whose `parse()` method reads the
///           process arguments into a `Cli`.
///  The matching `#[derive(Parser)]` MACRO
///           lives beside the struct in `cli.rs`;
///  here we import only the trait so we
///           can CALL `Cli::parse()` (a trait method needs its trait in scope).
/// Why:
///       Without the trait in scope,
///  `Cli::parse()` would not resolve.
/// Gotcha:
///    in Rust a method can come from a TRAIT,
///  and the trait must be imported
///           to call it,
///  even though `Cli` is already in scope.
///  There is no TS
///           analogue:
///  TS methods always live on the value itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseArgs } from "some-cli-parser";
/// ```
#[cfg(not(windows))]
use clap::Parser;

/// What:
/// the startup steps that end the program when they fail,
/// and the debug-build hook that fails one on purpose.
/// Why:
/// a failed double-click start on Windows is reported in a message box (`src/startup_failure.rs`).
use music_player::startup_failure::StartupStep;

/// What:
///      `use i_slint_backend_winit::Backend;`.
///  Slint's winit backend,
///  built
///           explicitly so a window-attributes hook can run.
/// Why:
///       The default backend selector gives no hook to set the Wayland app id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Backend } from "slint-winit-backend";
/// ```
use i_slint_backend_winit::Backend;

/// What:
///      `use slint::{ComponentHandle, Model, SharedString, VecModel};`.
///           `ComponentHandle` is the trait giving `.as_weak()`/`.run()` on the
///           window;
///  `Model` is the trait whose `.iter()` reads a list property
///           back (we re-read the full `queue` model to repaginate);
///  `SharedString`
///           is Slint's cheap-to-clone string (sibling:
///  `String`,
///  which would force
///           a fresh allocation on every clone);
///  `VecModel` builds the list model
///           behind a list property.
///  (The `ModelRc` a setter wants is produced by
///           `.into()`,
///  so it needs no import.)
/// Why:
///       Needed to drive the window,
///  read its `queue`,
///  and set its list props;
///           `SharedString` over `String` because Slint clones these strings often
///           and a refcounted clone is far cheaper than reallocating.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ComponentHandle, Model, SharedString, VecModel } from "slint";
/// ```
use slint::ComponentHandle;

/// What:
///      `fn format_time(secs: f64) -> String`.
///  Format seconds as "m:ss".
///           `f64` is a 64-bit float (sibling:
///  `f32`);
///  `String` is an owned heap
///           string (sibling:
///  `&str`,
///  a borrowed view we could not return here
///           because it would point at this function's freed locals).
/// Why:
///       Slint number-to-string is awkward,
///  so we format here and pass strings;
///           the result is `String` (owned) so the caller can keep it past this call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatTime(secs: number): string { ... }
/// ```
pub(crate) fn format_time(secs: f64) -> String {
    // What:     `let whole = if secs > 0.0 { secs as u64 } else { 0 };`. `if/else`
    //           used as an EXPRESSION (both arms yield a value). `secs as u64` is a
    //           numeric CAST that truncates the float toward zero into a 64-bit
    //           unsigned integer (`u64`; siblings: `u32`, `usize`). The else arm
    //           clamps negatives and NaN to 0.
    // Why:      Avoid negative or garbage times; `u64` because a track length never
    //           needs a sign and easily fits a 64-bit count of seconds.
    // Gotcha:   `as u64` is a SATURATING/truncating cast, not TS's `Number` coercion:
    //           NaN becomes 0 and the fraction is dropped, with no exception.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const whole = secs > 0 ? Math.floor(secs) : 0;
    // ```
    let whole = if secs > 0.0 { secs as u64 } else { 0 };
    // What:     `format!("{}:{:02}", whole / 60, whole % 60)`. The `format!` MACRO
    //           builds an owned `String` from a template: `{}` interpolates minutes
    //           (`whole / 60`, integer division) and `{:02}` interpolates seconds
    //           (`whole % 60`) zero-padded to 2 digits. No trailing `;`, so this is
    //           the function's tail expression and is returned.
    // Why:      "3:07" style display.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, "0")}`;
    // ```
    return format!("{}:{:02}", whole / 60, whole % 60)
}

/// What:
///      `fn forced_failure(step: StartupStep) -> Result<(), slint::PlatformError>`.
///  In a Windows debug build,
/// fails `step` on purpose when `MUSIC_PLAYER_FORCE_STARTUP_FAILURE` names it.
/// Why:
///       The Windows runner proves the failure report by forcing each step and reading the message box back
///           (`.github/workflows/music-player-release.yml`,
/// job `test-windows`).
/// The error goes through the same
///           path as a real one.
#[cfg(all(windows, debug_assertions))]
fn forced_failure(step: StartupStep) -> Result<(), slint::PlatformError> {
    /// Imports the startup failure injection variable and the helpers that read it.
    use music_player::startup_failure::{FORCE_STARTUP_FAILURE_ENV, forced_failure_detail, forced_startup_failure};
    let value = std::env::var(FORCE_STARTUP_FAILURE_ENV).ok();
    if forced_startup_failure(value.as_deref()) == Some(step) {
        tracing::warn!(?step, "failing a startup step on purpose ({FORCE_STARTUP_FAILURE_ENV})");
        return Err(slint::PlatformError::from(forced_failure_detail(step)));
    }
    return Ok(());
}

/// What:
///      `fn forced_failure(_step: StartupStep) -> Result<(), slint::PlatformError>`.
///  Release builds and non-Windows builds:
/// never fails.
/// Why:
///       The test hook is compiled out of the release;
/// the call sites stay the same on every build.
#[cfg(not(all(windows, debug_assertions)))]
fn forced_failure(_step: StartupStep) -> Result<(), slint::PlatformError> {
    return Ok(());
}

/// What:
///      `fn startup_failed(step: StartupStep, error: slint::PlatformError) -> anyhow::Error`.
///  Windows:
/// reports the failed step (a message box when stderr reaches nobody,
/// the text on stderr otherwise),
/// then
///  hands the error on to `main`.
/// Why:
///       A double-click start has no console,
/// so the error `main` returns would be written nowhere.
#[cfg(windows)]
fn startup_failed(step: StartupStep, error: slint::PlatformError) -> anyhow::Error {
    music_player::startup_failure_win32::report_startup_failure(step, &error.to_string());
    tracing::debug!(?step, "the startup failure was reported; main returns the error and drops its state");
    return anyhow::Error::from(error);
}

/// What:
///      `fn startup_failed(_step: StartupStep, error: slint::PlatformError) -> anyhow::Error`.
///  Linux and macOS:
/// hands the error on to `main` unchanged,
/// which prints it as before.
/// Why:
///       Those builds always have a terminal or a session log for stderr.
#[cfg(not(windows))]
fn startup_failed(_step: StartupStep, error: slint::PlatformError) -> anyhow::Error {
    return anyhow::Error::from(error);
}

/// What:
/// `finish_step` gives the value of a startup step,
/// or the startup failure for that step.
/// Why:
/// each step that can fail names itself in the failure the user sees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finishStep<T>(step: StartupStep, outcome: T | PlatformError): T { if (outcome instanceof PlatformError) throw startupFailed(step, outcome); return outcome; }
/// ```
fn finish_step<T>(step: StartupStep, outcome: Result<T, slint::PlatformError>) -> Result<T> {
    return match outcome {
        Ok(value) => Ok(value),
        Err(error) => Err(startup_failed(step, error)),
    };
}

/// What:
///      `fn main() -> Result<()>`.
///  The entry point.
///  The return type is
///           `anyhow`'s success-or-error enum:
///  `Ok(())` (success with the unit
///           value,
///  like `void`) or `Err(anyhow::Error)`.
///  Returning `Err` from
///           `main` makes the process exit non-zero and prints the error.
/// Why:
///       Propagate window/backend failure (e.g. no display server) as the exit
///           status,
///  rather than panicking.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function main(): Promise<void> { ... } // throws on platform failure
/// ```
fn main() -> Result<()> {
    // What:     `let console_attach = attach_parent_console();`. Windows only: attach to the console of the
    //           parent process and repair stdout and stderr (a handle the parent passed on is kept).
    // Why:      The Windows executable is a GUI-subsystem program, which starts with no console. This runs before
    //           the logger and before `Cli::parse()`, because both print. A double-click start has no parent
    //           console; the attach then fails, nothing is printed anywhere and no window opens.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const consoleAttach = attachParentConsole(); // win32 only
    // ```
    #[cfg(windows)]
    let console_attach = music_player::console_attach_win32::attach_parent_console();
    // What:     `let console_colors = enable_console_colors();`. Windows only: switch every console behind stdout and
    //           stderr to interpret ANSI color sequences, and measure what each handle does with them.
    // Why:      The logger and clap write colors only where they become colors: never into a file, never as literal
    //           ESC characters on a console without VT processing (`src/console_color.rs`).
    #[cfg(windows)]
    let console_colors = music_player::console_attach_win32::enable_console_colors();
    // What:     `music_player::logging::init();`. Install the stderr tracing subscriber
    //           before anything else, so startup events have a sink. Idempotent.
    // Why:      Every `tracing` event from this crate and `truepeak-core` needs a sink;
    //           `RUST_LOG` tunes the level (default `info`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // logging.init();
    // ```
    #[cfg(not(windows))]
    music_player::logging::init();
    #[cfg(windows)]
    music_player::logging::init_with_ansi(music_player::console_color::log_colors_wanted(
        console_colors.stderr,
        std::env::var("NO_COLOR").ok().as_deref(),
    ));
    // What:     `log_console_attach(&console_attach);` and `log_console_colors(&console_colors);`. Windows only: write
    //           what the attach and the color setup did at debug level, then add the panic message box.
    // Why:      Both ran before the logger existed, so their outcome is logged now. The panic hook shows a panic of the
    //           main thread in a message box when stderr reaches nobody (a double-click start).
    #[cfg(windows)]
    {
        music_player::console_attach::log_console_attach(&console_attach);
        music_player::console_color::log_console_colors(&console_colors);
        music_player::startup_failure_win32::install_panic_message_box();
    }
    // What:     `let cli = Cli::parse();`. Read and validate the command-line
    //           arguments FIRST, before any backend, window, or GPU setup.
    //           `Cli::parse()` (from the `clap::Parser` trait) reads the real process
    //           arguments; on `--help` / `--version` / a bad argument it prints and
    //           EXITS the process right here, by clap's design.
    // Why:      Parsing up front means `--help`, `--version`, and argument errors
    //           return instantly without first constructing a Slint backend and
    //           window; the parsed `cli` is consumed near the end to seed the queue.
    // Gotcha:   `Cli::parse()` itself calls `process::exit` on `--help`/bad input;
    //           that is clap's library behavior, not our code, and is the standard
    //           CLI contract (so it never returns in those cases).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const cli = parseArgs(process.argv); // prints help / exits on bad args
    // ```
    #[cfg(not(windows))]
    let cli = Cli::parse();
    // Windows: the same parse, with clap's colors off when a console refused VT processing.
    #[cfg(windows)]
    let cli = music_player::cli::parse_with_color(music_player::console_color::cli_color(console_colors));
    // A debug-build test hook: `MUSIC_PLAYER_FORCE_STARTUP_FAILURE=panic` panics here, on the main thread.
    if let Err(error) = forced_failure(StartupStep::Panic) {
        panic!("{error}");
    }

    // What:     `if cli.licenses { ... return Ok(()) }`. Print the embedded license notices to stdout and exit before
    //           any backend, window or audio setup.
    // Why:      The notices must be reachable from the program itself (the Windows release is one executable), and
    //           printing them needs no display, so it works on a headless machine too. A closed pipe (`| head`)
    //           ends the listing without an error (`write_notices`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (cli.licenses) { writeNotices(process.stdout); return; }
    // ```
    if cli.licenses {
        tracing::debug!("printing the license notices for --licenses");
        music_player::licenses::write_notices(&mut std::io::stdout().lock())?;
        return Ok(());
    }

    // What:     Install Slint's winit backend explicitly, with a window-attributes
    //           hook that stamps the Wayland app id. This `{ ... }` is a BARE BLOCK
    //           SCOPE: the locals inside (`builder`, `backend`, ...) are dropped at
    //           its closing brace. This MUST happen before the first window is
    //           created (which would lock in the default platform).
    // Why:      KDE attaches taskbar progress to the running window only when the
    //           window's app id matches the `.desktop` file; the default backend
    //           selector offers no hook to set it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (process.env.SLINT_MCP_PORT === undefined) {
    //   const backend = winitBackend({ windowAttributesHook: setWindowAppId });
    //   slint.setPlatform(backend);
    // }
    // ```
    //
    // What:     Gate the whole explicit-backend block on `SLINT_MCP_PORT` being UNSET
    //           (`std::env::var_os(...).is_none()`).
    // Why:      Slint 1.17's embedded MCP server (the `slint/mcp` feature activated by
    //           SLINT_MCP_PORT, used by the `mcp` mise task for agent-driven UI
    //           testing) only starts when SLINT ITSELF creates the backend through its
    //           selector (`with_global_context` -> `init_testing_backends` ->
    //           `mcp_server::init`). Calling `set_platform` here would bypass that path,
    //           so the server would never bind AND `SLINT_BACKEND=headless` would be
    //           ignored (a real window would open). When the port is set we therefore
    //           let Slint pick the backend, honoring SLINT_BACKEND; the Wayland app-id
    //           hook is the only thing dropped, and only for that test-only run.
    if std::env::var_os("SLINT_MCP_PORT").is_none() {
        // What:     `let mut builder = Backend::builder().with_window_attributes_hook(launcher::set_window_app_id);`.
        //           `Backend::builder()` starts a backend builder; `.with_window_attributes_hook(fn)`
        //           registers the app-id hook (passed by name). `mut` marks the
        //           binding MUTABLE because a later line may reassign `builder`.
        // Why:      The hook runs for each window Slint creates (here, the one); `mut`
        //           is needed because the software-renderer branch reassigns it.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let builder = Backend.builder().withWindowAttributesHook(setWindowAppId);
        // ```
        let mut builder =
            Backend::builder().with_window_attributes_hook(launcher::set_window_app_id);
        // What:     `let force_software = match std::env::var("SLINT_BACKEND") { Ok(value) =>
        //           value.contains("software"), Err(_) => false };` is true when the variable names the software
        //           renderer. `var(...)` returns `Result<String, _>`, and an unset variable counts as false.
        // Why:      Honor the run task's software-renderer escape hatch.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const forceSoftware = (process.env.SLINT_BACKEND ?? "").includes("software");
        // ```
        let force_software = match std::env::var("SLINT_BACKEND") {
            Ok(value) => value.contains("software"),
            Err(_error) => false,
        };
        // What:     `if force_software { builder = builder.with_renderer_name("software"); }`.
        //           Reassign `builder` (allowed because it is `mut`) to one pinned to
        //           the software renderer by Slint winit's renderer token. This is
        //           `software`, not Cargo's `renderer-software` feature name.
        // Why:      Match the previous behaviour for headless / no-GPU runs.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (forceSoftware) builder = builder.withRendererName("software");
        // ```
        if force_software {
            builder = builder.with_renderer_name("software");
        }
        // What:     `let backend = builder.build()?;`. `.build()` returns
        //           `Result<Backend, PlatformError>`; the `?` unwraps `Ok` or RETURNS
        //           the `Err` from `main` (e.g. no display server).
        // Why:      Construction can fail if `WAYLAND_DISPLAY`/`DISPLAY` is unset.
        // Gotcha:   `?` on a `Result` propagates the ERROR out of `main`; it is early
        //           return, not optional chaining.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const backend = builder.build(); // throws PlatformError on failure
        // ```
        let built = match forced_failure(StartupStep::WindowSystem) {
            Ok(()) => builder.build(),
            Err(error) => Err(error),
        };
        let backend = finish_step(StartupStep::WindowSystem, built)?;
        // What:     `slint::platform::set_platform(Box::new(backend)).expect("no Slint platform should already be set");`.
        //           `Box::new(backend)` moves the backend onto the heap behind an
        //           owning pointer (the API wants a `Box<dyn Platform>` trait object;
        //           siblings `Rc`/`Arc` would add sharing we do not need).
        //           `set_platform` returns a `Result`; `.expect(msg)` unwraps `Ok` or
        //           PANICS with `msg` on `Err`.
        // Why:      Make it the process platform so the later `AppWindow::new()` uses
        //           this backend and its hook; `.expect` because the only failure is
        //           "a platform was already set", which cannot happen here (first
        //           Slint call).
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // slint.setPlatform(backend); // throws if a platform was already set
        // ```
        slint::platform::set_platform(Box::new(backend))
            .expect("no Slint platform should already be set");
    }

    // What:     `let app = AppWindow::new()?;`. `AppWindow::new()` returns
    //           `Result<AppWindow, PlatformError>`; the `?` unwraps `Ok` or returns
    //           the error from `main`.
    // Why:      We need the window before wiring anything.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const app = new AppWindow();
    // ```
    let created = match forced_failure(StartupStep::Window) {
        Ok(()) => AppWindow::new(),
        Err(error) => Err(error),
    };
    let app = finish_step(StartupStep::Window, created)?;
    // What:     `app.global::<Theme>().set_scheme_override(color_scheme_from_env());`.
    //           Apply the color scheme override before the first frame.
    // Why:      Verification runs fix the scheme with MUSIC_PLAYER_COLOR_SCHEME; unset follows the system.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // app.theme.schemeOverride = colorSchemeFromEnv();
    // ```
    app.global::<Theme>().set_scheme_override(color_scheme_from_env());
    // What:     `ui_theme::apply_accent_override_from_env(&app);` then `ui_theme::install_accent_tracking(&app);`.
    //           Apply the optional accent override, derive the theme roles for the current root (the system accent,
    //           or the baseline when none exists), and register the callback that re-derives them on an accent change.
    // Why:      A3 and D108: the OS accent is the theme root and every other color is derived from it in OKLCH.
    //           The override comes first so the first derivation already uses it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // applyAccentOverrideFromEnv(app);
    // installAccentTracking(app);
    // ```
    ui_theme::apply_accent_override_from_env(&app);
    ui_theme::install_accent_tracking(&app);

    // What:     `let weak = app.as_weak();`. `.as_weak()` makes a WEAK handle to the
    //           window: it does not keep the window alive, and can be sent to other
    //           threads and `upgrade()`d back to a strong handle ON the event-loop
    //           thread.
    // Why:      The engine's update callback (on another thread) needs to reach the
    //           window without owning it (which would leak or cross-thread-share the
    //           non-`Send` window).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const weak = new WeakRef(app);
    // ```
    let weak = app.as_weak();

    // What:     `ui_font_scale::apply_os_font_scale(&app);` registers the callback that
    //           scales every UI font to 0.9x the OS UI font once Slint's async portal
    //           read lands the real value.
    // Why:      Lives in a sibling module so `main.rs` stays under the max-lines limit;
    //           see `ui_font_scale.rs` for the async-portal and binding-loop rationale.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // applyOsFontScale(app);
    // ```
    ui_font_scale::apply_os_font_scale(&app);

    // What:     `ui_led_rows::apply(&app);` derives measured cap row membership.
    // Why:      Full-width plate paint stays in Slint while Rust marks physical first and
    //           last caps so exposed ends retain source corner geometry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // applyLedRowGeometry(app);
    // ```

    // Derive every runtime LED pigment through OKLCH coordinates.

    // Register settings persistence.

    // What:     `let launcher = Launcher::new();`. Construct the KDE taskbar-progress
    //           emitter (a cheap-to-clone session-bus handle, or a no-op without a
    //           bus).
    // Why:      Each update tick clones one into the event-loop closure to push the
    //           current play position to the taskbar.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const launcher = Launcher.connect();
    // ```
    let launcher = Launcher::new();

    // What:     `let progress_started_at = Instant::now();`. `Instant::now()` captures
    //           a monotonic timestamp once, before update handling begins.
    // Why:      Later update callbacks turn this into elapsed time for debouncing.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const progressStartedAt = performance.now();
    // ```
    let progress_started_at = Instant::now();
    // What:     `let progress_debouncer = Arc::new(Mutex::new(ProgressDebouncer::new()));`.
    //           Nested constructors: `ProgressDebouncer::new()` is the state;
    //           `Mutex::new(...)` wraps it so only one thread mutates it at a time;
    //           `Arc::new(...)` puts that behind a thread-safe shared-ownership
    //           pointer (sibling `Rc` would NOT be `Send`).
    // Why:      The engine callback is cross-thread, but all progress decisions need
    //           one shared baseline; `Arc<Mutex<_>>` is the safe cross-thread shape.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const progressDebouncer = new ProgressDebouncer(); // shared, no lock in JS
    // ```
    let progress_debouncer = Arc::new(Mutex::new(ProgressDebouncer::new()));

    // What:     `let engine = Rc::new(Engine::spawn(move |update| { ... }));`. Start
    //           the engine, giving it a callback that forwards each `Update` to the
    //           UI thread. `move |update| { ... }` is a MOVE closure: it takes
    //           ownership of the captures (`weak`, `launcher`, `progress_*`).
    //           `Rc::new(...)` wraps the engine so multiple UI callbacks can share it.
    // Why:      One engine, shared by all the button handlers.
    // Gotcha:   `move` here transfers OWNERSHIP of the captured variables into the
    //           closure (they cannot be used afterward), unlike a TS arrow that
    //           merely references them.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engine = Engine.spawn(update => postToUiThread(() => applyUpdate(app, update)));
    // ```
    let analysis_notifier = start_analysis_notifier();
    let update_notifier = analysis_notifier.clone();
    let unplayable_notifier = start_unplayable_notifier();
    let unplayable_engine_notifier = unplayable_notifier.clone();
    // The Search service answers on its own thread and takes scan batches from the engine thread (D111, D112).
    let search_service = ui_search::start_service(&app);
    // Joins the service's threads when `main` ends, before the thread-local handle is dropped during process exit.
    let _search_shutdown = ui_search::ServiceShutdown;
    let scan_service = search_service.clone();
    let engine = Rc::new(Engine::spawn(move |update| {
        // What:     post the sweep's progress to the system indicator before the window sees the update.
        // Why:      D107: the indicator runs on its own thread, so this only queues a message; the window draws no
        //           analysis indicator at all.
        if let Update::Analysis { done, total } = &update {
            update_notifier.report(*done, *total);
        }
        // What:     hand the files that cannot be played to the system notification, before the window sees the update.
        // Why:      D122: the window marks the rows and draws no message; the notifier thread posts the notification,
        //           so this only queues a message on the engine thread.
        if let Update::Unplayable(paths) = &update {
            unplayable_engine_notifier.report(paths);
        }
        // What:     hand tracks the folder walk has found to the Search index, on the engine thread.
        // Why:      D111: Search works while the library is still being scanned; the window needs no part in it.
        if matches!(update, Update::ScanBatch { .. }) {
            if let Update::ScanBatch { first_index, names } = update {
                scan_service.stream_batch(first_index, names);
            }
            return;
        }
        // What:     `let update_weak = weak.clone();`. `.clone()` of a weak handle is
        //           cheap (bumps a refcount); the outer closure is called repeatedly so
        //           it cannot move `weak` out, hence a fresh per-call clone under its
        //           own name.
        // Why:      Each update needs its own handle to move into the inner closure.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const w = weak; // GC: no explicit clone needed
        // ```
        let update_weak = weak.clone();
        // What:     `let update_launcher = launcher.clone();`. Clone the progress emitter
        //           for this call (same reason as `update_weak`: the outer closure is `Fn`
        //           and may run many times).
        // Why:      The inner closure moves it to the UI thread to emit progress.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const l = launcher;
        // ```
        let update_launcher = launcher.clone();
        // What:     `let update_debouncer = Arc::clone(&progress_debouncer);`.
        //           `Arc::clone(&x)` makes another owner of the SAME shared state by
        //           bumping the atomic refcount (the `&` lends the `Arc` to clone
        //           from). Written `Arc::clone(&x)` rather than `x.clone()` to make
        //           "this is a cheap refcount bump, not a deep copy" obvious.
        // Why:      The inner event-loop closure needs to own a handle to the same
        //           debounce state.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const progressDebouncerForUpdate = progressDebouncer;
        // ```
        let update_debouncer = Arc::clone(&progress_debouncer);
        // What:     `let progress_elapsed = progress_started_at.elapsed();`.
        //           `.elapsed()` returns a `Duration` measuring monotonic time since
        //           the captured `Instant`.
        // Why:      Debounce decisions use time between accepted progress updates.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const progressElapsed = performance.now() - progressStartedAt;
        // ```
        let progress_elapsed = progress_started_at.elapsed();
        // What:     `let _ = slint::invoke_from_event_loop(move || { ... });`.
        //           `invoke_from_event_loop` schedules the MOVE closure to run ON the
        //           UI/event-loop thread (required for touching window properties) and
        //           returns a `Result`. `let _ =` discards that result (it errs only
        //           if the loop is gone, e.g. during shutdown).
        // Why:      Updates arrive on the engine thread but must be applied on the UI
        //           thread; failure during shutdown is safe to ignore.
        // Gotcha:   `let _ = expr` EXPLICITLY discards a value; for a `#[must_use]`
        //           `Result` it is how you say "I am intentionally ignoring this".
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // queueMicrotaskOnUiThread(() => { ... });
        // ```
        let _ = slint::invoke_from_event_loop(move || {
            // What:     `if let Some(window) = update_weak.upgrade() { ... }`. `upgrade()`
            //           turns the weak handle back into `Option<AppWindow>`; the `if let
            //           Some(window)` pattern runs the body only when the window still
            //           exists, binding the strong handle under its own name.
            // Why:      The window may have closed before this scheduled closure runs.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const app = weak.deref(); if (app) { ... }
            // ```
            if let Some(window) = update_weak.upgrade() {
                // What:     `ui_progress::apply_update_with_progress_debounce(&window, &update_launcher, &update_debouncer, progress_elapsed, update);`.
                //           Call the bridge, lending `window`, `update_launcher`, and the
                //           debouncer by reference (`&`), and moving `progress_elapsed`
                //           and `update` in by value.
                // Why:      State updates stay immediate, while seek-bar and taskbar
                //           progress updates are rate-limited.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // uiProgress.applyUpdateWithProgressDebounce(app, launcher, progressDebouncer, progressElapsed, update);
                // ```
                ui_progress::apply_update_with_progress_debounce(
                    &window,
                    &update_launcher,
                    &update_debouncer,
                    progress_elapsed,
                    update,
                );
            }
        });
    }));
    ui_wire::wire(&app, &engine);

    // What:     `match cli.path { Some(path) => ..., None => ... }`. The single optional
    //           positional path was parsed into `cli` at the top of `main`. A CLI path
    //           takes precedence over a saved session; with no path, restore the session.
    // Why:      Opening a path explicitly should override resuming.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (cli.path) { ... } else { ... }
    // ```
    match cli.path {
        // What:     `Some(path) => { ... }`. A CLI path: a directory becomes the Source
        //           Root with nothing selected; a file becomes its parent directory as the
        //           root with that file preselected. `path.is_dir()` tests the filesystem.
        // Why:      Exactly one directory Source Root; a single file is cued inside its
        //           folder.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (isDir(path)) engine.send(OpenRoot({ root: path, select: null, play }));
        // else engine.send(OpenRoot({ root: dirname(path), select: path, play }));
        // ```
        Some(path) => {
            if path.is_dir() {
                engine.send(Command::OpenRoot {
                    root: path,
                    select: None,
                    play: cli.start_playing,
                });
            } else {
                // What:     `let root = match path.parent() { Some(parent) => parent.to_path_buf(), None =>
                //           PathBuf::from(".") };` is the folder that holds the path, or `.` for a path with no
                //           parent.
                // Why:      The parent folder is the Source Root for a single-file launch.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const root = dirname(path) || ".";
                // ```
                let root = match path.parent() {
                    Some(parent) => parent.to_path_buf(),
                    None => PathBuf::from("."),
                };
                engine.send(Command::OpenRoot {
                    root,
                    select: Some(path),
                    play: cli.start_playing,
                });
            }
        }
        // What:     `None => { ... }`. No CLI path: restore the saved session, falling back
        //           to the music directory.
        // Why:      Resume where the user left off when nothing was named on the CLI.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // else { /* restore or auto-load music dir */ }
        // ```
        None => {
            // What:     `let session = Session::load();`. Read the saved session (defaults
            //           if none/corrupt). It no longer prunes; the engine re-scans the root.
            // Why:      Resume where the user left off.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const session = Session.load();
            // ```
            let session = Session::load();
            // What:     `let root = match session.source_root.clone() { Some(saved) if saved.is_dir() =>
            //           Some(saved), _ => None };` keeps the saved Source Root only when it is still a folder.
            // Why:      A missing root falls back to the music directory (and is replaced on
            //           the next save), so a non-directory root is treated as absent.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const root = (session.sourceRoot && isDir(session.sourceRoot)) ? session.sourceRoot : null;
            // ```
            let root = match session.source_root.clone() {
                Some(saved) if saved.is_dir() => Some(saved),
                _ => None,
            };
            // What:     `if let Some(saved_root) = root { ... } else if let Some(music_dir) = music_dir() { ... }`.
            //           Restore the saved root with its Selected Track and position, else
            //           restore the music directory carrying the saved settings but no
            //           selection, else leave the queue empty. The present path keeps its
            //           own name so it never shadows the filtered `Option`.
            // Why:      Keep the user's settings even when the saved root is gone, and never
            //           leave a usable launch empty when a music directory exists.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (root) restore(root, session.selected, ...);
            // else if (musicDir()) restore(musicDir(), null, settingsOnly);
            // ```
            if let Some(saved_root) = root {
                engine.send(Command::Restore {
                    root: saved_root,
                    selected: session.selected,
                    position: session.position_secs,
                    volume: session.volume,
                    playback_mode: session.playback_mode,
                });
            } else if let Some(music_dir) = music_dir() {
                engine.send(Command::Restore {
                    root: music_dir,
                    selected: None,
                    position: 0.0,
                    volume: session.volume,
                    playback_mode: session.playback_mode,
                });
            } else {
                // What:     `engine.send(Command::NoLibrary);`. Say that there is nothing to open.
                // Why:      The window shows "No folder is open" only when told so (EM1); the engine no longer says it
                //           at start, before this decision.
                tracing::info!("no saved folder and no music folder; the window offers to open one");
                engine.send(Command::NoLibrary);
            }
        }
    }

    // What:     `app.run()?; Ok(())`. Show the window and run the event loop until
    //           it closes; `?` converts any `PlatformError` into `anyhow::Error`.
    //           When it returns, `app` and `engine` drop: the engine's `Drop` sends
    //           `Quit` and joins its thread, tearing down PipeWire.
    // Why:      Hand control to Slint.
    // Gotcha:   Rust runs DESTRUCTORS (`Drop`) at end of scope deterministically;
    //           that is what cleanly stops the engine here, with no GC or manual
    //           teardown call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await app.run();
    // return;
    // ```
    // What:     run the event loop, then close the indicator and release the sweep before the engine drops.
    // Why:      a Plasma job or notification must not outlive the app, and a sweep parked by Pause must not wait for a
    //           resume that can no longer come.
    let run_result = match forced_failure(StartupStep::EventLoop) {
        Ok(()) => app.run(),
        Err(error) => Err(error),
    };
    analysis_notifier.shutdown();
    unplayable_notifier.shutdown();
    SweepGate::shared().close();
    finish_step(StartupStep::EventLoop, run_result)?;
    return Ok(())
}

/// Registers the unit-test scratch root,
/// placed in the build directory instead of `/tmp`.
#[cfg(test)]
mod test_scratch;
