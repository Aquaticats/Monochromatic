//! The Win32 half of the startup failure report (compiled on Windows only; the text and the decisions are in
//! `startup_failure.rs`).
//!
//! Calls used, from `user32.dll` and `kernel32.dll` through the `windows` crate:
//! - `GetStdHandle(STD_ERROR_HANDLE)` (feature `Win32_System_Console`) tells whether stderr reaches anybody at the time
//!   of the failure. The crate maps a null and an invalid handle to `Err`.
//! - `MessageBoxW` (feature `Win32_UI_WindowsAndMessaging`) shows the text with a stop-sign icon and one OK button.
//!   `MB_SETFOREGROUND` brings the box to the front, because the program has no window of its own at that point, and
//!   `MB_TASKMODAL` keeps it modal without an owner window. Alacritty shows its Windows panic message with the same flags
//!   (`alacritty/src/panic.rs`).

use crate::startup_failure::{
    StartupStep, failure_message, failure_title, message_box_wanted, panic_detail, without_nul,
};
use windows::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MB_SETFOREGROUND, MB_TASKMODAL, MessageBoxW};
use windows::core::{HSTRING, PCWSTR};

/// What:
///      `fn stderr_handle() -> Option<isize>`.
///  The raw value of the standard error handle now, or `None` when it is null or invalid.
/// Why:
///       Read at the time of the failure, after the console attach repaired what it could.
fn stderr_handle() -> Option<isize> {
    // SAFETY: `GetStdHandle` only reads the process parameters and has no preconditions.
    return unsafe { GetStdHandle(STD_ERROR_HANDLE) }.ok().map(|handle| return handle.0 as isize);
}

/// What:
///      `fn executable_path() -> String`.
///  The path of this program, for the Command Prompt line in the message.
/// Why:
///       The person can type the line as it is shown. When Windows cannot tell the path, the release file name stands in.
fn executable_path() -> String {
    return match std::env::current_exe() {
        Ok(path) => path.display().to_string(),
        Err(error) => {
            tracing::warn!(error = %error, "the path of the program is unknown; the message names the file instead");
            "music-player.exe".to_string()
        }
    };
}

/// What:
///      `fn show_message_box(title: &str, text: &str)`.
///  Shows a modal error message box and returns when it is closed.
/// Why:
///       The only way to tell a person who double-clicked the program what failed.
fn show_message_box(title: &str, text: &str) {
    let wide_title = HSTRING::from(without_nul(title));
    let wide_text = HSTRING::from(without_nul(text));
    // SAFETY: both pointers are NUL-terminated wide strings owned by the two `HSTRING`s, which outlive the call.
    let pressed = unsafe {
        MessageBoxW(
            None,
            PCWSTR(wide_text.as_ptr()),
            PCWSTR(wide_title.as_ptr()),
            MB_ICONERROR | MB_OK | MB_SETFOREGROUND | MB_TASKMODAL,
        )
    };
    tracing::debug!(pressed = pressed.0, "the startup failure message box was closed");
}

/// What:
///      `pub fn report_startup_failure(step: StartupStep, detail: &str)`.
///  Tells the person that a startup step failed: in a message box when stderr reaches nobody, on stderr otherwise.
/// Why:
///       A double-click start has no console, so the error `main` returns is written nowhere. Started from a terminal
///           or a script, the same text is printed there and nothing waits for a click.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportStartupFailure(step: StartupStep, detail: string): void { ... }
/// ```
pub fn report_startup_failure(step: StartupStep, detail: &str) {
    tracing::error!(?step, detail, "startup failed");
    let message = failure_message(step, detail, &executable_path());
    if message_box_wanted(stderr_handle()) {
        tracing::debug!(?step, "stderr reaches nobody; showing the message box");
        show_message_box(failure_title(step), &message);
    } else {
        // Exact terminal output for the person who started the program; `main` prints the error itself after it.
        eprintln!("{message}\n");
    }
}

/// What:
///      `pub fn install_panic_message_box()`.
///  Adds a panic hook that shows the panic in a message box when the main thread panics and stderr reaches nobody.
/// Why:
///       A panic of the main thread ends the program; without a console its message is written nowhere. The previous
///           hook still runs first, so a terminal start prints the panic as before. Panics of other threads do not end
///           the program and show no box.
pub fn install_panic_message_box() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        if std::thread::current().name() != Some("main") || !message_box_wanted(stderr_handle()) {
            return;
        }
        let location = info.location().map(|place| return place.to_string());
        let detail = panic_detail(info.payload_as_str(), location.as_deref());
        show_message_box(
            failure_title(StartupStep::Panic),
            &failure_message(StartupStep::Panic, &detail, &executable_path()),
        );
    }));
}
