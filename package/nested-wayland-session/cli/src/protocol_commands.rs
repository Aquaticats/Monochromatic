//! The one request type the control socket parses a line into,
//!  whose variants carry each verb's own arguments.
//!
//! Split out of `protocol.rs` to keep that file under the code-line limit.

/// An owned filesystem path a screenshot request carries.
use std::path::PathBuf;

/// The accent color a runtime `accent-color` request carries.
use crate::accent_color::AccentColor;
/// The appearance value a runtime appearance request carries.
use crate::appearance_portal::ColorSchemePreference;
/// The output-scale value a `scale` request carries.
use crate::screen_geometry::OutputScale;

/// The small value types its variants carry,
///  and the notification and job verbs it wraps.
use super::{
    ButtonAction,
    JobCommand,
    KeyAction,
    NotificationCommand,
    PointerButton,
};
/// A parsed control command.
///
/// What:
///      `pub enum Command { ... }`.
///  A tagged union where each variant carries its
///           own arguments (`Screenshot` a path,
///  `Click` coordinates,
///  and so on).
/// Why:
///       One typed value the executor matches on,
///  decoupling parsing from action.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Command =
///   | { kind: "ping" }
///   | { kind: "screenshot"; path: string }
///   | { kind: "click"; x: number; y: number; button: PointerButton }
///   | { kind: "key"; name: string; action: KeyAction }
///   | { kind: "type"; text: string }
///   | { kind: "resize"; width: number; height: number }
///   | { kind: "dropFile"; path: string; x?: number; y?: number }
///   | { kind: "colorScheme"; preference: "dark" | "light" }
///   | { kind: "scale"; scale: OutputScale }
///   | { kind: "move"; x: number; y: number }
///   | { kind: "button"; action: "down" | "up"; button: PointerButton }
///   | { kind: "title" } | { kind: "appId" }
///   | { kind: "launcher" } | { kind: "launcherLog"; count: number } | { kind: "launcherClear" }
///   | { kind: "accentColor"; color: AccentColor }
///   | { kind: "notification"; command: NotificationCommand } | { kind: "job"; command: JobCommand }
///   | { kind: "quit" };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Liveness check.
    Ping,
    /// Capture the current frame to a PNG at the given path.
    Screenshot(
        /// Destination PNG path.
        PathBuf,
    ),
    /// Click at a logical point with a button.
    Click {
        /// Logical x coordinate.
        x: f64,
        /// Logical y coordinate.
        y: f64,
        /// Which button to click.
        button: PointerButton,
        /// Presses in a row,
        ///  2 for a double click.
        count: u32,
        /// Milliseconds between the presses.
        interval_ms: u64,
        /// Chord of keys held for the whole click,
        ///  empty for none.
        modifiers: String,
    },
    /// Press at one point,
    ///  move to another in steps,
    ///  and release.
    Drag {
        /// Logical start point.
        from: (f64, f64),
        /// Logical end point.
        to: (f64, f64),
        /// Which button to hold.
        button: PointerButton,
        /// Equal motion steps between the points.
        steps: u32,
        /// Milliseconds between the steps.
        step_ms: u64,
        /// Chord of keys held for the whole drag,
        ///  empty for none.
        modifiers: String,
    },
    /// Touchpad-style pixel scrolling at a logical point.
    Scroll {
        /// Logical x coordinate.
        x: f64,
        /// Logical y coordinate.
        y: f64,
        /// Total horizontal travel in logical pixels.
        dx: f64,
        /// Total vertical travel in logical pixels.
        dy: f64,
        /// Events the travel is split into.
        steps: u32,
        /// Milliseconds between the events.
        step_ms: u64,
        /// Chord of keys held while scrolling,
        ///  empty for none.
        modifiers: String,
    },
    /// Deliver actual Wayland wheel notches at a logical point.
    Wheel {
        /// Logical x coordinate.
        x: f64,
        /// Logical y coordinate.
        y: f64,
        /// Positive notches move right;
        ///  fractions are high-resolution steps of 1/120 notch.
        horizontal: f64,
        /// Positive notches move down;
        ///  fractions are high-resolution steps of 1/120 notch.
        vertical: f64,
        /// Chord of keys held while scrolling,
        ///  empty for none.
        modifiers: String,
    },
    /// Press/release/tap a named key.
    Key {
        /// Key name (e.g. `a`,
        ///  `enter`,
        ///  `space`).
        name: String,
        /// What to do with it.
        action: KeyAction,
    },
    /// Type a run of text as individual key taps.
    Type(
        /// Text to type verbatim.
        String,
    ),
    /// Resize the nested screen,
    ///  keeping the output scale.
    Resize {
        /// New width in logical pixels;
        ///  the framebuffer is this times the scale.
        width: i32,
        /// New height in logical pixels;
        ///  the framebuffer is this times the scale.
        height: i32,
    },
    /// Originate a compositor-side file drag toward the hosted app (inbound DnD test).
    DropFile {
        /// Files to advertise to the app as one `text/uri-list` drop.
        paths: Vec<PathBuf>,
        /// Optional drop x in logical coordinates (defaults to the window centre).
        x: Option<f64>,
        /// Optional drop y in logical coordinates (defaults to the window centre).
        y: Option<f64>,
    },
    /// Start recording frames at a steady rate into a directory.
    Record {
        /// Output directory for the frame sequence.
        dir: PathBuf,
        /// Target capture rate in frames per second.
        fps: f64,
        /// Output format name (validated when the command runs).
        format: String,
    },
    /// Stop the running recording.
    RecordStop,
    /// Answer or change the key repeat rate and delay announced to clients.
    RepeatInfo(
        /// New `(rate per second, delay in milliseconds)`,
        ///  or `None` to only read the current pair.
        Option<(i32, i32)>,
    ),
    /// Name the window that has keyboard focus.
    Focus,
    /// List every window with its title,
    ///  app ID,
    ///  and whether it has keyboard focus.
    Windows,
    /// Ask a window to close,
    ///  as its close button does.
    CloseWindow(
        /// Index in the `windows` listing,
        ///  or `None` for the window with keyboard focus.
        Option<usize>,
    ),
    /// Name the cursor the hosted client asked for.
    Cursor,
    /// Report the device the compositor renders on.
    Gpu,
    /// Read the hosted application's accessibility tree.
    Accessibility,
    /// Answer the Wayland socket name hosted clients connect to.
    Display,
    /// Move the pointer to a logical point without pressing a button.
    Move {
        /// Logical x coordinate.
        x: f64,
        /// Logical y coordinate.
        y: f64,
    },
    /// Press or release a button at the current pointer position.
    Button {
        /// Whether the button goes down or comes up.
        action: ButtonAction,
        /// Which button.
        button: PointerButton,
    },
    /// Report the title the hosted client set on its toplevel.
    Title,
    /// Report the app ID the hosted client set on its toplevel.
    AppId,
    /// Report the newest taskbar progress update.
    Launcher,
    /// Report the newest taskbar progress updates,
    ///  oldest first.
    LauncherLog {
        /// How many of the newest updates to list.
        count: usize,
    },
    /// Empty the taskbar progress history.
    LauncherClear,
    /// Change the private appearance portal's color scheme while the client runs.
    ColorScheme(
        /// Appearance value the private portal serves from now on.
        ColorSchemePreference,
    ),
    /// Change the output scale while keeping the logical screen size.
    Scale(
        /// Scale every hosted surface is told to render at from now on.
        OutputScale,
    ),
    /// Change the private appearance portal's accent color while the client runs.
    AccentColor(
        /// Accent color the private portal serves from now on.
        AccentColor,
    ),
    /// Read or drive the fake notification service.
    Notification(
        /// The notification verb and its arguments.
        NotificationCommand,
    ),
    /// Read or drive the fake Plasma job view server.
    Job(
        /// The job verb and its arguments.
        JobCommand,
    ),
    /// Stop the compositor.
    Quit,
}
