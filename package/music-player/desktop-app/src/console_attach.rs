//! Console attachment decisions for the Windows GUI-subsystem build.
//!
//! The Windows executable is linked with the `windows` subsystem (`#![windows_subsystem = "windows"]` in `main.rs`), so
//! double-clicking it opens no console window. A program of that subsystem starts detached from any console, and its
//! standard handles are invalid unless the parent passed some on (a pipe, a file). To keep `music-player --version`
//! and the `tracing` output usable in a terminal, the process attaches to the console of its parent at startup.
//!
//! This file holds the pure part: which standard handles need work after the attach, and what a startup reports.
//! The Win32 calls live in `console_attach_win32.rs`, which compiles on Windows only. Sources and the decision:
//! `doc/decision/music-player-windows-gui-subsystem.md`.

/// What:
///      `pub enum StdHandleAction`.
///  What the startup does to one standard handle (stdout or stderr) after the console attach.
/// Why:
///       Redirection must survive: a handle the parent passed on (`> out.txt`, a pipe) is kept or put back,
///           and only a handle that is still missing is pointed at the console.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StdHandleAction = { kind: "keep" } | { kind: "restore"; original: number } | { kind: "openConsoleOutput" };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StdHandleAction {
    /// The handle is usable as it is: either the parent passed it on and the attach left it alone, or the attach
    /// supplied the console's own handle for a handle that was missing.
    Keep,
    /// The parent passed the handle on and the attach replaced or cleared it; the original goes back.
    Restore {
        /// The raw handle value the process started with.
        original: isize,
    },
    /// No handle was passed on and the attach supplied none; the process opens `CONOUT$` and uses that.
    OpenConsoleOutput,
}

/// What:
///      `pub fn needs_console_attach(stdout_before: Option<isize>, stderr_before: Option<isize>) -> bool`.
///  Whether the startup tries to attach to the parent console at all.
/// Why:
///       When both output handles were passed on (a pipe, a file, or the CI runner's capture), nothing needs a
///           console, and skipping the attach keeps the shell's redirection untouched. `None` means the handle was
///           null or invalid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needsConsoleAttach = (stdout?: number, stderr?: number) => stdout === undefined || stderr === undefined;
/// ```
pub fn needs_console_attach(stdout_before: Option<isize>, stderr_before: Option<isize>) -> bool {
    return stdout_before.is_none() || stderr_before.is_none();
}

/// What:
///      `pub fn std_handle_action_after_attach(before: Option<isize>, after: Option<isize>) -> StdHandleAction`.
///  Picks the action for one standard handle from its raw value before the attach and its raw value after.
/// Why:
///       Microsoft's `AttachConsole` page does not say whether the call fills or replaces the standard handles, so the
///           outcome is decided from what is observed instead of assumed. A handle the parent passed on always wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function action(before?: number, after?: number) {
///   if (before === undefined) return after === undefined ? "openConsoleOutput" : "keep";
///   return before === after ? "keep" : "restore"; // restore carries `before`
/// }
/// ```
pub fn std_handle_action_after_attach(before: Option<isize>, after: Option<isize>) -> StdHandleAction {
    return match (before, after) {
        (Some(original), Some(current)) if original == current => StdHandleAction::Keep,
        (Some(original), _) => StdHandleAction::Restore { original },
        (None, Some(_)) => StdHandleAction::Keep,
        (None, None) => StdHandleAction::OpenConsoleOutput,
    };
}

/// What:
///      `pub struct StreamReport`.
///  The action taken for one standard handle and the Win32 error code when taking it failed.
/// Why:
///       The startup runs before the logger exists, so the report is carried to `main` and logged after
///           `logging::init()`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StreamReport = { action: StdHandleAction; failureCode?: number };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamReport {
    /// The action decided for the handle.
    pub action: StdHandleAction,
    /// The Win32 error code when carrying the action out failed, so the handle stays missing or unchanged.
    pub failure_code: Option<u32>,
}

/// What:
///      `pub enum ConsoleAttachOutcome`.
///  How the startup attach ended.
/// Why:
///       Double-click (no parent console) and a terminal launch (a parent console) are different, expected
///           situations; the debug log line names which one happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConsoleAttachOutcome =
///   | { kind: "handlesPassedOn" }
///   | { kind: "noParentConsole"; errorCode: number }
///   | { kind: "attached"; stdout: StreamReport; stderr: StreamReport };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleAttachOutcome {
    /// Both output handles were passed on by the parent, so no attach was tried.
    HandlesPassedOn,
    /// The attach failed with this Win32 error code. `ERROR_INVALID_HANDLE` (6) means the parent has no console, which
    /// is what a double-click from Explorer gives: the program runs with no console and no output.
    NoParentConsole {
        /// The Win32 error code from `GetLastError`.
        error_code: u32,
    },
    /// The attach worked; each output handle got the action in its report.
    Attached {
        /// The standard output handle.
        stdout: StreamReport,
        /// The standard error handle.
        stderr: StreamReport,
    },
}

/// What:
///      `pub fn log_console_attach(outcome: &ConsoleAttachOutcome)`.
///  Writes the attach outcome to the `tracing` log at debug level.
/// Why:
///       The attach runs before the logger is installed. Debug level keeps `--version` and a normal launch from
///           printing an extra line at the default `info` level.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// logger.debug("console attach", outcome);
/// ```
pub fn log_console_attach(outcome: &ConsoleAttachOutcome) {
    match outcome {
        ConsoleAttachOutcome::HandlesPassedOn => {
            tracing::debug!("console attach skipped: the parent passed stdout and stderr on");
        }
        ConsoleAttachOutcome::NoParentConsole { error_code } => {
            tracing::debug!(error_code, "console attach skipped: the parent process has no console (a double-click start)");
        }
        ConsoleAttachOutcome::Attached { stdout, stderr } => {
            tracing::debug!(?stdout, ?stderr, "attached to the console of the parent process");
        }
    }
}

/// What:
///      `#[cfg(test)] #[path = "console_attach_tests.rs"] mod tests;`.
///  Unit tests for the decision functions above.
/// Why:
///       The attach branches that matter (a passed-on handle survives, a missing one is supplied) are pure and run
///           on every host.
#[cfg(test)]
#[path = "console_attach_tests.rs"]
mod tests;
