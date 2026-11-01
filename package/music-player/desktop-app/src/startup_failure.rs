//! The text a failed start shows, and when the Windows build shows it in a message box.
//!
//! On Windows the program is a GUI-subsystem executable. Started by a double-click it has no console and no standard
//! handles, so an error returned from `main` (or a panic message) is written nowhere and the program just ends with exit
//! status 1. The Windows build therefore shows the text below in a message box when stderr reaches nobody, and prints it
//! on stderr otherwise (`startup_failure_win32.rs`). Linux and macOS keep printing the error as before.
//!
//! This file holds the pure part: the steps that can fail, the text for each, the box decision and the debug-build test
//! hook that fails a step on purpose. Decision and evidence: `doc/decision/music-player-windows-gui-subsystem.md`.

/// What:
///      `pub const FORCE_STARTUP_FAILURE_ENV: &str`.
///  The environment variable that makes a debug build fail one startup step on purpose.
/// Why:
///       The Windows runner cannot break its window system or graphics driver, so the CI forces each failure to drive
///           the real error path and read the message box back. Only debug builds read the variable
///           (`#[cfg(debug_assertions)]` in `main.rs`); release builds do not contain that code.
pub const FORCE_STARTUP_FAILURE_ENV: &str = "MUSIC_PLAYER_FORCE_STARTUP_FAILURE";

/// What:
///      `pub enum StartupStep`.
///  The steps of the start that end the program when they fail.
/// Why:
///       Each failure has its own cause and its own remedies, so the message names the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StartupStep = "windowSystem" | "window" | "eventLoop" | "panic";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupStep {
    /// Slint's winit backend could not start (`Backend::builder().build()`): no connection to the desktop.
    WindowSystem,
    /// The window could not be created (`AppWindow::new()`): no renderer or no window.
    Window,
    /// The event loop that draws the window ended with an error (`app.run()`).
    EventLoop,
    /// The main thread panicked.
    Panic,
}

/// What:
///      `pub fn forced_startup_failure(value: Option<&str>) -> Option<StartupStep>`.
///  The step `MUSIC_PLAYER_FORCE_STARTUP_FAILURE` names, or `None` for an unset or unknown value.
/// Why:
///       The test hook reads the variable once per step; an unknown value fails nothing, so a typo cannot break a start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const forced = { "window-system": "windowSystem", window: "window", "event-loop": "eventLoop", panic: "panic" }[value];
/// ```
pub fn forced_startup_failure(value: Option<&str>) -> Option<StartupStep> {
    return match value? {
        "window-system" => Some(StartupStep::WindowSystem),
        "window" => Some(StartupStep::Window),
        "event-loop" => Some(StartupStep::EventLoop),
        "panic" => Some(StartupStep::Panic),
        _ => None,
    };
}

/// What:
///      `pub fn forced_failure_detail(step: StartupStep) -> String`.
///  The error text a forced failure carries in place of the error Windows would report.
/// Why:
///       The message then says plainly that the failure was made on purpose, and the CI finds the variable name in it.
pub fn forced_failure_detail(step: StartupStep) -> String {
    let value = match step {
        StartupStep::WindowSystem => "window-system",
        StartupStep::Window => "window",
        StartupStep::EventLoop => "event-loop",
        StartupStep::Panic => "panic",
    };
    return format!(
        "forced by {FORCE_STARTUP_FAILURE_ENV}={value}: this test setting of debug builds makes the step fail on purpose"
    );
}

/// What:
///      `pub fn failure_title(step: StartupStep) -> &'static str`.
///  The title of the message box.
/// Why:
///       A failure before the window exists is a start that did not happen; a failure of the running window or a panic
///           ends a program that had started.
pub fn failure_title(step: StartupStep) -> &'static str {
    return match step {
        StartupStep::WindowSystem | StartupStep::Window => "Music Player could not start",
        StartupStep::EventLoop | StartupStep::Panic => "Music Player stopped",
    };
}

/// What:
///      `pub fn failure_message(step: StartupStep, detail: &str, executable: &str) -> String`.
///  The full message: what failed, what Windows reported, and every remedy the program knows for that step.
/// Why:
///       The person who double-clicked the program sees nothing else. `executable` is the program's own path, so the
///           Command Prompt line in the message can be typed as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failureMessage(step: StartupStep, detail: string, executable: string): string { ... }
/// ```
pub fn failure_message(step: StartupStep, detail: &str, executable: &str) -> String {
    let log_hint = format!(
        "To see this message with the program's log lines, start Music Player from Command Prompt by typing its path \
         in quotes:\n\"{executable}\"\nThe log lines then appear in the Command Prompt window."
    );
    let driver = "Install the newest graphics driver, through Windows Update or from the website of the company that \
                  made your graphics card or computer.";
    return match step {
        StartupStep::WindowSystem => format!(
            "Music Player could not start: it could not open its connection to the Windows desktop, so it cannot show a \
             window.\n\nWindows reported: {detail}\n\nWhat you can do:\n\
             - Start Music Player again from the Start menu, from File Explorer or from a shortcut while you are signed \
             in. A program started by a service, or by a scheduled task set to run whether you are signed in or not, \
             has no desktop to show a window on.\n\
             - Restart Windows, then start Music Player again.\n\n{log_hint}"
        ),
        StartupStep::Window => format!(
            "Music Player could not start: it could not create its window. It tries to draw the window with the graphics \
             card first and with the processor second, and creating the window failed.\n\n\
             Windows reported: {detail}\n\nWhat you can do:\n\
             - Start Music Player again.\n\
             - {driver}\n\
             - Restart Windows, then start Music Player again.\n\n{log_hint}"
        ),
        StartupStep::EventLoop => format!(
            "Music Player stopped: its window stopped working, so the program ended.\n\n\
             Windows reported: {detail}\n\nWhat you can do:\n\
             - Start Music Player again.\n\
             - {driver}\n\
             - Let Music Player draw its window with the processor instead of the graphics card. Open Command Prompt and \
             type these two lines:\nset SLINT_BACKEND=software\n\"{executable}\"\n\
             The setting lasts until that Command Prompt window is closed.\n\n{log_hint}"
        ),
        StartupStep::Panic => format!(
            "Music Player stopped because of an internal error. This is a defect in Music Player, not a problem with your \
             computer or your music files.\n\nDetails: {detail}\n\nWhat you can do:\n\
             - Start Music Player again.\n\
             - If the error comes back, keep this text and the log lines that lead up to it when you report the \
             problem.\n\n{log_hint}"
        ),
    };
}

/// What:
///      `pub fn message_box_wanted(stderr_handle: Option<isize>) -> bool`.
///  Whether the failure is shown in a message box: only when stderr has no usable handle at the time of the failure.
/// Why:
///       A double-click start (no console, no handle) has nowhere else to show the text. A start from a terminal, a
///           script or the CI has stderr on a console, a file or a pipe; the text goes there, and a modal box would
///           leave a script or a CI job waiting for a click.
pub fn message_box_wanted(stderr_handle: Option<isize>) -> bool {
    return stderr_handle.is_none();
}

/// What:
///      `pub fn without_nul(text: &str) -> String`.
///  The text with every NUL character replaced by U+FFFD.
/// Why:
///       `MessageBoxW` reads a NUL-terminated string, so a NUL inside an error detail would cut the message short.
pub fn without_nul(text: &str) -> String {
    return text.replace('\0', "\u{FFFD}");
}

/// What:
///      `pub fn panic_detail(message: Option<&str>, location: Option<&str>) -> String`.
///  The detail line of a panic: its message and the source location.
/// Why:
///       A panic payload is a `&str` or a `String` almost always; anything else has no text, and the message says so.
pub fn panic_detail(message: Option<&str>, location: Option<&str>) -> String {
    let text = message.unwrap_or("the panic carried no text");
    return match location {
        Some(place) => format!("{text} (at {place})"),
        None => text.to_string(),
    };
}

/// What:
///      `#[cfg(test)] #[path = "startup_failure_tests.rs"] mod tests;`.
///  Unit tests for the text and the decisions above.
/// Why:
///       The message box itself exists on Windows only; everything it shows is decided here and tested on every host.
#[cfg(test)]
#[path = "startup_failure_tests.rs"]
mod tests;
