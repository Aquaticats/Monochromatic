//! Whether the Windows build writes ANSI color sequences, decided per output handle.
//!
//! A Windows console turns an ANSI escape sequence into color only when its output mode has
//! `ENABLE_VIRTUAL_TERMINAL_PROCESSING`; otherwise the ESC character lands in the screen buffer as a visible character.
//! A file or a pipe never interprets the sequences, so a log written there keeps them as noise.
//! Two writers in this program got that wrong (measured on the Windows runners; sources and runs in
//! `doc/decision/music-player-windows-gui-subsystem.md`):
//! - `tracing-subscriber` 0.3.23 turns colors on whenever `NO_COLOR` is empty (`fmt_layer.rs`, `Layer::default`), without
//!   looking at the handle it writes to and without ever setting the console mode.
//! - clap prints through `anstream` 1.0.0. Its `anstyle_query::windows::enable_virtual_terminal_processing` sets the mode
//!   on stdout first and returns at the first failure, so when stdout is a file the console behind stderr is never
//!   switched; with `TERM` set, `AutoStream::always` then passes the sequences through to that console.
//!
//! The startup therefore sets the mode on stdout and on stderr separately (`console_attach_win32.rs`), and this file
//! decides from the outcome. The decisions are pure, so they are tested on every host.

/// What:
///      `pub const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32`.
///  The console output mode flag that makes a console interpret ANSI escape sequences (Microsoft, `SetConsoleMode`).
/// Why:
///       The pure mode arithmetic needs the value without the Windows-only `windows` crate.
pub const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;

/// What:
///      `pub enum ConsoleColorSupport`.
///  What one standard output handle does with an ANSI escape sequence, after the startup tried to switch it on.
/// Why:
///       Colors are written only where they become colors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConsoleColorSupport = "notConsole" | "virtualTerminal" | { kind: "noVirtualTerminal"; errorCode: number };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleColorSupport {
    /// The handle is a file, a pipe, or missing: escape sequences would be stored as text.
    NotConsole,
    /// The handle is a console whose output mode has `ENABLE_VIRTUAL_TERMINAL_PROCESSING` (it had it, or the startup set it).
    VirtualTerminal,
    /// The handle is a console that refused `ENABLE_VIRTUAL_TERMINAL_PROCESSING` (a Windows console older than Windows 10).
    NoVirtualTerminal {
        /// The Win32 error code from `SetConsoleMode`.
        error_code: u32,
    },
}

/// What:
///      `pub struct ConsoleColors`.
///  The color support of stdout and of stderr.
/// Why:
///       The startup measures both once, before the logger and the argument parser print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsoleColors {
    /// The standard output handle.
    pub stdout: ConsoleColorSupport,
    /// The standard error handle.
    pub stderr: ConsoleColorSupport,
}

/// What:
///      `pub fn mode_with_virtual_terminal(mode: u32) -> u32`.
///  The console mode to set: the current mode plus `ENABLE_VIRTUAL_TERMINAL_PROCESSING`.
/// Why:
///       The other mode flags (line wrapping, processed output) belong to whoever owns the console and stay as they are.
pub fn mode_with_virtual_terminal(mode: u32) -> u32 {
    return mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
}

/// What:
///      `pub fn has_virtual_terminal(mode: u32) -> bool`.
///  Whether a console mode already interprets escape sequences.
/// Why:
///       A console that already has the flag needs no `SetConsoleMode` call.
pub fn has_virtual_terminal(mode: u32) -> bool {
    return mode & ENABLE_VIRTUAL_TERMINAL_PROCESSING != 0;
}

/// What:
///      `pub fn log_colors_wanted(stderr: ConsoleColorSupport, no_color: Option<&str>) -> bool`.
///  Whether the `tracing` log on stderr is written with colors.
/// Why:
///       Colors only on a console that interprets them. `NO_COLOR` (https://no-color.org/) still turns them off, as
///           `tracing-subscriber` does by default; the explicit `with_ansi` call would otherwise override it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const logColorsWanted = (stderr, noColor) => stderr === "virtualTerminal" && !noColor;
/// ```
pub fn log_colors_wanted(stderr: ConsoleColorSupport, no_color: Option<&str>) -> bool {
    let no_color_set = no_color.is_some_and(|value| return !value.is_empty());
    return stderr == ConsoleColorSupport::VirtualTerminal && !no_color_set;
}

/// What:
///      `pub fn cli_color(colors: ConsoleColors) -> clap::ColorChoice`.
///  The color setting for clap's help and error text.
/// Why:
///       With every console handle interpreting escape sequences, clap's own detection (`Auto`) is right: it strips
///           colors for a file or a pipe and writes them to a console. A console that refused the mode would show the
///           sequences as text when `TERM` is set, so then clap prints no colors at all.
pub fn cli_color(colors: ConsoleColors) -> clap::ColorChoice {
    let refused = |support: ConsoleColorSupport| return matches!(support, ConsoleColorSupport::NoVirtualTerminal { .. });
    if refused(colors.stdout) || refused(colors.stderr) {
        return clap::ColorChoice::Never;
    }
    return clap::ColorChoice::Auto;
}

/// What:
///      `pub fn log_console_colors(colors: &ConsoleColors)`.
///  Writes the measured color support to the `tracing` log at debug level.
/// Why:
///       It is measured before the logger exists; the debug line shows which branch ran on a given console.
pub fn log_console_colors(colors: &ConsoleColors) {
    tracing::debug!(stdout = ?colors.stdout, stderr = ?colors.stderr, "console color support");
}

/// What:
///      `#[cfg(test)] #[path = "console_color_tests.rs"] mod tests;`.
///  Unit tests for the decisions above.
/// Why:
///       They run on every host, while the Win32 calls that feed them compile on Windows only.
#[cfg(test)]
#[path = "console_color_tests.rs"]
mod tests;
