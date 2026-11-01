//! The Win32 half of the console attachment (compiled on Windows only; see `console_attach.rs` for the decisions).
//!
//! Calls used, all from `kernel32.dll` through the `windows` crate (feature `Win32_System_Console`):
//! - `GetStdHandle(STD_OUTPUT_HANDLE | STD_ERROR_HANDLE)` reads the process's standard handles. The crate maps a null
//!   and an `INVALID_HANDLE_VALUE` result to `Err`, so `Err` means "no usable handle".
//! - `AttachConsole(ATTACH_PARENT_PROCESS)` attaches to the console of the parent process. Per Microsoft it fails with
//!   `ERROR_INVALID_HANDLE` when the parent has no console and with `ERROR_ACCESS_DENIED` when the process already has
//!   one. It is meant for programs linked with `/SUBSYSTEM:WINDOWS`.
//! - `SetStdHandle` and `CreateFileW("CONOUT$")` supply or put back a standard handle.
//! - `GetConsoleMode` tells a console from a file or a pipe (it fails for those), and `SetConsoleMode` adds
//!   `ENABLE_VIRTUAL_TERMINAL_PROCESSING` so the console turns ANSI color sequences into colors (`console_color.rs`).
//!
//! Rust's standard library reads the standard handle on the first print, not at startup, and treats a missing handle
//! (`ERROR_INVALID_HANDLE`) as a successful write that goes nowhere, so a double-click start prints into the void
//! without a panic (`library/std/src/io/stdio.rs` `handle_ebadf`, `library/std/src/sys/stdio/windows.rs` `get_handle`).

use crate::console_attach::{
    ConsoleAttachOutcome, StdHandleAction, StreamReport, needs_console_attach, std_handle_action_after_attach,
};
use crate::console_color::{ConsoleColorSupport, ConsoleColors, has_virtual_terminal, mode_with_virtual_terminal};
use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Console::{
    ATTACH_PARENT_PROCESS, AttachConsole, CONSOLE_MODE, GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE, STD_HANDLE,
    STD_OUTPUT_HANDLE, SetConsoleMode, SetStdHandle,
};
use windows::core::w;

/// What:
///      `fn std_handle(kind: STD_HANDLE) -> Option<isize>`.
///  The raw value of a standard handle, or `None` when it is null or invalid.
/// Why:
///       The decisions compare raw values, and raw integers keep them testable on every host.
fn std_handle(kind: STD_HANDLE) -> Option<isize> {
    // SAFETY: `GetStdHandle` only reads the process parameters and has no preconditions.
    return unsafe { GetStdHandle(kind) }.ok().map(|handle| return handle.0 as isize);
}

/// What:
///      `fn win32_code(error: &windows::core::Error) -> u32`.
///  The Win32 error code inside the `HRESULT` the `windows` crate builds from `GetLastError`.
/// Why:
///       `ERROR_INVALID_HANDLE` (6) and `ERROR_ACCESS_DENIED` (5) are the documented codes, and the log line is easier to
///           read with them than with `0x80070006`.
fn win32_code(error: &windows::core::Error) -> u32 {
    return (error.code().0 as u32) & 0xFFFF;
}

/// What:
///      `fn carry_out(kind: STD_HANDLE, action: StdHandleAction) -> Option<u32>`.
///  Applies the action to one standard handle and returns the Win32 error code when that failed.
/// Why:
///       A failed repair must not stop the program; the report keeps the code for the debug log.
fn carry_out(kind: STD_HANDLE, action: StdHandleAction) -> Option<u32> {
    return match action {
        StdHandleAction::Keep => None,
        StdHandleAction::Restore { original } => {
            // SAFETY: `original` is the value `GetStdHandle` returned at startup for this process.
            unsafe { SetStdHandle(kind, HANDLE(original as *mut core::ffi::c_void)) }.err().map(|error| return win32_code(&error))
        }
        StdHandleAction::OpenConsoleOutput => {
            // SAFETY: the file name is a NUL-terminated wide literal and every other argument is a plain value or `None`.
            let opened = unsafe {
                CreateFileW(
                    w!("CONOUT$"),
                    (GENERIC_READ | GENERIC_WRITE).0,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                )
            };
            match opened {
                // The handle is never closed on purpose: it is the process's standard handle until the process ends.
                // SAFETY: `console` is the valid handle `CreateFileW` just returned.
                Ok(console) => unsafe { SetStdHandle(kind, console) }.err().map(|error| return win32_code(&error)),
                Err(error) => Some(win32_code(&error)),
            }
        }
    };
}

/// What:
///      `fn repair_stream(kind: STD_HANDLE, before: Option<isize>) -> StreamReport`.
///  Reads the handle after the attach, decides, and carries the decision out.
/// Why:
///       One path for stdout and stderr, so both follow the same rule.
fn repair_stream(kind: STD_HANDLE, before: Option<isize>) -> StreamReport {
    let after = std_handle(kind);
    let action = std_handle_action_after_attach(before, after);
    let failure_code = carry_out(kind, action);
    return StreamReport { action, failure_code };
}

/// What:
///      `pub fn attach_parent_console() -> ConsoleAttachOutcome`.
///  Attaches the process to the console of its parent when stdout or stderr was not passed on, and repairs the
///  standard handles. Returns what happened for the debug log.
/// Why:
///       A GUI-subsystem program starts with no console. Run from a terminal it must still print `--version`, argument
///           errors and logs there; started from Explorer (no parent console) it stays silent and shows no window.
///           A handle the parent passed on (`> out.txt`, a pipe) is never replaced.
/// Gotcha:
///       Must run before the logger and before argument parsing, because both print to stderr or stdout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = attachParentConsole(); // first line of main, before logging.init()
/// ```
pub fn attach_parent_console() -> ConsoleAttachOutcome {
    let stdout_before = std_handle(STD_OUTPUT_HANDLE);
    let stderr_before = std_handle(STD_ERROR_HANDLE);
    if !needs_console_attach(stdout_before, stderr_before) {
        return ConsoleAttachOutcome::HandlesPassedOn;
    }
    // SAFETY: `AttachConsole` takes a process id or `ATTACH_PARENT_PROCESS` and has no pointer arguments.
    if let Err(error) = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) } {
        return ConsoleAttachOutcome::NoParentConsole { error_code: win32_code(&error) };
    }
    return ConsoleAttachOutcome::Attached {
        stdout: repair_stream(STD_OUTPUT_HANDLE, stdout_before),
        stderr: repair_stream(STD_ERROR_HANDLE, stderr_before),
    };
}

/// What:
///      `fn color_support(kind: STD_HANDLE) -> ConsoleColorSupport`.
///  Whether one standard output handle interprets ANSI escape sequences, switching the console mode on when it can.
/// Why:
///       Each handle is set on its own. `anstyle-query` 1.1.5 stops at the first handle that is not a console, so with
///           stdout in a file it never switches the console behind stderr (`console_color.rs` has the details).
fn color_support(kind: STD_HANDLE) -> ConsoleColorSupport {
    // SAFETY: `GetStdHandle` only reads the process parameters and has no preconditions.
    let Ok(handle) = (unsafe { GetStdHandle(kind) }) else {
        return ConsoleColorSupport::NotConsole;
    };
    let mut mode = CONSOLE_MODE(0);
    // A file or a pipe has no console mode, so the call fails for them: that is the answer, not an error to report.
    // SAFETY: `handle` is a standard handle of this process and `mode` is a valid place to write the mode to.
    if unsafe { GetConsoleMode(handle, &mut mode) }.is_err() {
        return ConsoleColorSupport::NotConsole;
    }
    if has_virtual_terminal(mode.0) {
        return ConsoleColorSupport::VirtualTerminal;
    }
    // SAFETY: `handle` is a console output handle (its mode was just read) and the mode is a plain value.
    return match unsafe { SetConsoleMode(handle, CONSOLE_MODE(mode_with_virtual_terminal(mode.0))) } {
        Ok(()) => ConsoleColorSupport::VirtualTerminal,
        Err(error) => ConsoleColorSupport::NoVirtualTerminal { error_code: win32_code(&error) },
    };
}

/// What:
///      `pub fn enable_console_colors() -> ConsoleColors`.
///  Switches every console behind stdout and stderr to interpret ANSI escape sequences, and reports what each handle
///  does with them.
/// Why:
///       The logger and clap decide from the report whether to write colors, so a console never shows a literal ESC
///           and a file never holds one.
/// Gotcha:
///       Must run after `attach_parent_console` (the handles are final then) and before the logger and argument parsing.
pub fn enable_console_colors() -> ConsoleColors {
    return ConsoleColors { stdout: color_support(STD_OUTPUT_HANDLE), stderr: color_support(STD_ERROR_HANDLE) };
}
