//! The control-socket wire protocol:
//!  newline-delimited text requests and responses.
//!
//! Each request is one line:
//!  a verb plus arguments.
//!  Each request yields exactly one
//! response line:
//!  `ok`,
//!  `ok <data>`,
//!  or `err <message>`.
//!  This module is pure text
//! parsing and formatting with no Wayland types,
//!  so it is unit-tested directly,
//! including adversarial inputs (the tests live in `protocol_tests.rs`).

/// What:
///      `use std::path::PathBuf;`.
///  Owned filesystem path.
/// Why:
///       A screenshot request carries an owned destination path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // path ~ string
/// ```
use std::path::PathBuf;

/// What:
///      Import the closed set of appearance values the private portal can serve.
/// Why:
///       A runtime appearance request carries the same typed value as the startup option,
///           so no second spelling of `dark` and `light` can drift from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type ColorSchemePreference } from "./appearance_portal";
/// ```
use crate::appearance_portal::ColorSchemePreference;

/// Real-wheel command parsing is separate from the existing control verbs.
#[path = "protocol_wheel.rs"]
mod wheel;

/// Shared `name=value` option parsing.
#[path = "protocol_options.rs"]
mod protocol_options;

/// The `key` command.
#[path = "protocol_key.rs"]
mod key;

/// The `click`,
///  `drag`,
///  and `scroll` commands.
#[path = "protocol_click.rs"]
mod click;

/// The `drop-file` command line.
#[path = "protocol_drop.rs"]
mod dropfile;

/// The repeat-info and close commands.
#[path = "protocol_session.rs"]
mod session;

/// Runtime appearance parsing shares the startup option's value names.
#[path = "protocol_color_scheme.rs"]
mod color_scheme;

/// Runtime output-scale parsing shares the startup option's value grammar.
#[path = "protocol_scale.rs"]
mod scale;

/// Pointer hover and held-button parsing sits beside the click grammar it extends.
#[path = "protocol_pointer.rs"]
mod pointer;

/// Taskbar progress verbs parse their optional record count here.
#[path = "protocol_launcher.rs"]
mod launcher;

/// The `record` verb has the longest argument list and is parsed beside the others,
///  not among them.
#[path = "protocol_record.rs"]
mod record;

/// Notification verbs parse their ids,
///  reasons,
///  and action keys here.
#[path = "protocol_notification.rs"]
mod notification;

/// Plasma job verbs parse their job ids here.
#[path = "protocol_job.rs"]
mod job;

/// The runtime accent color verb shares the startup option's grammar.
#[path = "protocol_accent.rs"]
mod accent;

/// Job verbs are matched by the executor,
///  so their type is part of the protocol's interface.
pub use job::JobCommand;

/// Notification verbs are matched by the executor,
///  so their type is part of the protocol's interface.
pub use notification::NotificationCommand;

/// The accent color a runtime `accent-color` request carries.
use crate::accent_color::AccentColor;

/// What:
///      Import the output-scale value a `scale` request carries.
/// Why:
///       The command holds the parsed,
///  validated scale,
///  never the raw text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type OutputScale } from "./screen_geometry";
/// ```
use crate::screen_geometry::OutputScale;

/// Which pointer button a click uses.
///
/// What:
///      `pub enum PointerButton { Left, Right, Middle }`.
///  A closed set of three
///           variants.
///  In Rust an `enum` is a tagged union;
///  here each variant carries
///           no data.
/// Why:
///       Name the button symbolically at the protocol layer;
///  the input layer maps
///           each to its evdev button code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PointerButton = "left" | "right" | "middle";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    /// The primary (left) button.
    Left,
    /// The secondary (right) button.
    Right,
    /// The tertiary (middle) button.
    Middle,
}

/// Button-to-evdev-code mapping for pointer buttons.
///
/// What:
///      `impl PointerButton { ... }`.
///  One method turning the enum into the raw
///           `BTN_*` code.
/// Why:
///       Keep the code mapping beside the button enum.
impl PointerButton {
    /// Map the button to its Linux evdev button code.
    ///
    /// What:
    ///      `pub fn evdev_code(self) -> u32`.
    ///  Takes `self` by value (the enum is
    ///           `Copy`),
    ///  returns the `BTN_*` code as an unsigned 32-bit integer.
    /// Why:
    ///       Smithay's `ButtonEvent.button` field is the raw evdev code.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// evdevCode(): number { ... }
    /// ```
    ///
    /// @example
    /// ```ts
    /// PointerButton.Left.evdevCode(); // => 272 (BTN_LEFT)
    /// ```
    pub fn evdev_code(self) -> u32 {
        // What:     `match self { Left => 0x110, Right => 0x111, Middle => 0x112 }`.
        //           Map each variant to its `BTN_*` constant (`BTN_LEFT` is `0x110`).
        //           Tail expression, so the matched value is returned.
        // Why:      Provide the exact code the input event expects.
        match self {
            PointerButton::Left => return 0x110,
            PointerButton::Right => return 0x111,
            PointerButton::Middle => return 0x112,
        }
    }

    /// The lowercase name the control protocol uses for this button.
    ///
    /// What:
    ///      `pub fn name(self) -> &'static str`.
    /// Why:
    ///       Error answers name the button the way the request spelled it.
    pub fn name(self) -> &'static str {
        match self {
            PointerButton::Left => return "left",
            PointerButton::Right => return "right",
            PointerButton::Middle => return "middle",
        }
    }
}

/// What action a key request performs.
///
/// What:
///      `pub enum KeyAction { Press, Release, Tap, Hold { millis } }`.
/// Why:
///       A test may hold a key down,
///  release it,
///  or tap (press then release).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type KeyAction = "press" | "release" | "tap" | { hold: number };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    /// Press and hold.
    Press,
    /// Release a held key.
    Release,
    /// Press then immediately release.
    Tap,
    /// Press,
    ///  keep the key down for a duration so the client's own key repeat runs,
    ///  then release.
    Hold {
        /// How long the key stays down.
        millis: u64,
    },
}

/// What a `button` request does to a pointer button.
///
/// What:
///      `pub enum ButtonAction { Down, Up }`.
///  Two data-less variants.
/// Why:
///       A test holds a button across a pause and movement,
///  so the press and the release arrive as separate requests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ButtonAction = "down" | "up";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    /// Press the button and keep it held.
    Down,
    /// Release a held button.
    Up,
}

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

/// A response to send back on the control socket.
///
/// What:
///      `pub enum Response { Ok, OkWith(String), Err(String) }`.
///  Success with no
///           data,
///  success with a payload,
///  or an error message.
/// Why:
///       A uniform,
///  machine-readable reply shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Response = { ok: true; data?: string } | { ok: false; error: string };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Success,
    ///  no payload.
    Ok,
    /// Success with a text payload.
    OkWith(
        /// Payload text.
        String,
    ),
    /// Failure with a message.
    Err(
        /// Error message.
        String,
    ),
}

/// Format a response as its single wire line (no trailing newline).
///
/// What:
///      `pub fn format_response(response: &Response) -> String`.
///  Borrows the
///           response,
///  returns an owned `String`.
/// Why:
///       The control thread appends a newline and writes this to the socket.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatResponse(r: Response): string { ... }
/// ```
///
/// @example
/// ```ts
/// formatResponse({ ok: false, error: "bad" }); // => "err bad"
/// ```
pub fn format_response(response: &Response) -> String {
    // What:     `match response { ... }`. Map each variant to its wire form. `Ok` is the
    //           literal `"ok"`; `OkWith(data)` is `"ok "` plus the payload; `Err(message)`
    //           is `"err "` plus the message. `.to_string()` / `format!` allocate owned
    //           strings.
    // Why:      Produce the exact bytes the client parses.
    match response {
        Response::Ok => return "ok".to_string(),
        Response::OkWith(data) => return format!("ok {data}"),
        Response::Err(message) => return format!("err {message}"),
    }
}

/// Parse one request line into a `Command`.
///
/// What:
///      `pub fn parse_command(raw: &str) -> Result<Command, String>`.
///  Borrows the
///           raw line,
///  returns the parsed command or a human-readable error string.
///  The
///           `Result` error type is `String` (not `anyhow`) so the error can be sent
///           straight back as an `err <message>` line.
/// Why:
///       Central grammar for the control protocol.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseCommand(raw: string): Command { ... } // throws string on bad input
/// ```
///
/// @example
/// ```ts
/// parseCommand("click 10 20"); // => { kind: "click", x: 10, y: 20, button: "left" }
/// ```
pub fn parse_command(raw: &str) -> Result<Command, String> {
    // What:     `let line = raw.trim_end_matches(['\r', '\n']);`. Strip only trailing
    //           carriage-return / newline characters (not interior or leading spaces).
    // Why:      A socket line arrives with its terminator; keep the payload otherwise
    //           intact so `type` and `screenshot` preserve embedded spaces.
    let line = raw.trim_end_matches(['\r', '\n']);

    // What:     `if let Some(text) = line.strip_prefix("type ") { return Ok(Command::Type(
    //           text.to_string())); }`. `strip_prefix` returns `Some(rest)` when the line
    //           begins with `type `; the rest is the text verbatim (spaces preserved).
    // Why:      Handle `type` before generic tokenising so its text is not split.
    if let Some(text) = line.strip_prefix("type ") {
        return Ok(Command::Type(text.to_string()));
    }

    // What:     `if line == "type" { return Ok(Command::Type(String::new())); }`. A bare
    //           `type` with no text types nothing. `String::new()` is the empty owned
    //           string.
    // Why:      Accept the degenerate empty-type case rather than erroring.
    if line == "type" {
        return Ok(Command::Type(String::new()));
    }

    // What:     `if let Some(path) = line.strip_prefix("screenshot ") { return Ok(
    //           Command::Screenshot(PathBuf::from(path.trim()))); }`. Take the rest as the
    //           destination path (trimmed of surrounding spaces).
    // Why:      Handle `screenshot` before tokenising so paths with spaces still work.
    if let Some(raw_path) = line.strip_prefix("screenshot ") {
        // Trim surrounding whitespace without confusing raw input with the usable path.
        let path = raw_path.trim();
        // What:     `if path.is_empty() { return Err(...); }`. Reject an empty path.
        // Why:      There is nowhere to write.
        if path.is_empty() {
            return Err("screenshot requires a path".to_string());
        }
        return Ok(Command::Screenshot(PathBuf::from(path)));
    }

    // What:     `drop-file` reads the rest of the line itself, because quoted paths hold spaces.
    // Why:      A path with a space must not be split by the whitespace tokenizer below.
    if let Some(rest) = line.strip_prefix("drop-file ") {
        return dropfile::parse(rest);
    }
    if line == "drop-file" {
        return Err("drop-file requires at least one path".to_string());
    }

    // What:     `notification-action` takes the rest of the line as the action key, so it is
    //           handled before tokenising, like `type`.
    // Why:      Action keys are chosen by the app and may contain spaces.
    if let Some(rest) = line.strip_prefix("notification-action ") {
        return notification::parse_action(rest);
    }
    if line == "notification-action" {
        return Err("notification-action requires an id and an action key".to_string());
    }

    // What:     `let mut tokens = line.split_whitespace();`. Lazy iterator over
    //           whitespace-separated tokens (collapses runs of spaces). `mut` because we
    //           consume it with `.next()`.
    // Why:      The remaining verbs (ping, quit, click, key, resize) are simple token
    //           lists.
    let mut tokens = line.split_whitespace();

    // What:     `let verb = tokens.next().ok_or_else(|| "empty command".to_string())?;`.
    //           First token or an error if the line was blank. `.ok_or_else(closure)`
    //           turns `None` into `Err`; `?` unwraps.
    // Why:      Every command starts with a verb.
    let verb = tokens.next().ok_or_else(|| return "empty command".to_string())?;

    // What:     `match verb { ... }`. Dispatch on the verb. Each arm parses its remaining
    //           tokens with a helper and returns a `Command` or an error.
    // Why:      Route to the right parser.
    match verb {
        "ping" => return Ok(Command::Ping),
        "quit" => return Ok(Command::Quit),
        "click" => return click::parse_click(&mut tokens),
        "drag" => return click::parse_drag(&mut tokens),
        "scroll" => return click::parse_scroll(&mut tokens),
        "wheel" => return wheel::parse(&mut tokens),
        "key" => return key::parse(&mut tokens),
        "resize" => return parse_resize(&mut tokens),
        "repeat-info" => return session::parse_repeat_info(&mut tokens),
        "focus" => return no_arguments(&mut tokens, "focus", Command::Focus),
        "windows" => return no_arguments(&mut tokens, "windows", Command::Windows),
        "display" => return no_arguments(&mut tokens, "display", Command::Display),
        "cursor" => return no_arguments(&mut tokens, "cursor", Command::Cursor),
        "gpu" => return no_arguments(&mut tokens, "gpu", Command::Gpu),
        "a11y" => return no_arguments(&mut tokens, "a11y", Command::Accessibility),
        "close" => return session::parse_close(&mut tokens),
        "record" => return record::parse(&mut tokens),
        "color-scheme" => return color_scheme::parse(&mut tokens),
        "scale" => return scale::parse(&mut tokens),
        "move" => return pointer::parse_move(&mut tokens),
        "button" => return pointer::parse_button_command(&mut tokens),
        "title" => return no_arguments(&mut tokens, "title", Command::Title),
        "app-id" => return no_arguments(&mut tokens, "app-id", Command::AppId),
        "launcher" => return no_arguments(&mut tokens, "launcher", Command::Launcher),
        "launcher-clear" => return no_arguments(&mut tokens, "launcher-clear", Command::LauncherClear),
        "launcher-log" => return launcher::parse_log(&mut tokens),
        "accent-color" => return accent::parse(&mut tokens),
        "notifications" | "notification-log" | "notification-clear" | "notification-close" => {
            return notification::parse(verb, &mut tokens);
        }
        "jobs" | "job-log" | "job-clear" | "job-suspend" | "job-resume" | "job-cancel" | "job-refresh" => {
            return job::parse(verb, &mut tokens);
        }
        other => return Err(format!("unknown command: {other}")),
    }
}

/// Accept a verb that takes no arguments and reject any trailing token.
///
/// What:
///      `fn no_arguments(tokens: &mut std::str::SplitWhitespace, verb: &str, command: Command)
///           -> Result<Command, String>`.
///  Returns `command` when no token follows the verb.
/// Why:
///       A request such as `title extra` must fail whole instead of being read as `title`.
fn no_arguments(
    tokens: &mut std::str::SplitWhitespace,
    verb: &str,
    command: Command,
) -> Result<Command, String> {
    // What:     `if tokens.next().is_some() { return Err(...); }`. Any leftover token is an error.
    // Why:      The verb's meaning does not cover arguments it was not given.
    if tokens.next().is_some() {
        return Err(format!("{verb} takes no arguments"));
    }
    return Ok(command)
}



/// Parse the arguments of a `resize` command.
///
/// What:
///      `fn parse_resize(tokens: &mut std::str::SplitWhitespace) -> Result<Command,
///           String>`.
///  Consumes a width and a height.
/// Why:
///       Isolate resize parsing.
fn parse_resize(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    // What:     `let width = parse_i32(tokens.next(), "resize width")?;`. Parse the width.
    // Why:      New screen width.
    let width = parse_i32(tokens.next(), "resize width")?;

    // What:     `let height = parse_i32(tokens.next(), "resize height")?;`. Parse height.
    // Why:      New screen height.
    let height = parse_i32(tokens.next(), "resize height")?;

    // What:     `if width <= 0 || height <= 0 { return Err(...); }`. Reject non-positive
    //           sizes.
    // Why:      A zero/negative screen cannot exist.
    if width <= 0 || height <= 0 {
        return Err("resize dimensions must be positive".to_string());
    }

    // What:     `Ok(Command::Resize { width, height })`. Build the command.
    // Why:      Return the parsed resize.
    return Ok(Command::Resize { width, height })
}



/// Parse an optional token as `f64`,
///  with a field name for errors.
///
/// What:
///      `fn parse_f64(token: Option<&str>, field: &str) -> Result<f64, String>`.
///           `f64` is a 64-bit float (sibling:
///  32-bit `f32`).
/// Why:
///       Click coordinates are floating-point logical positions.
fn parse_f64(token: Option<&str>, field: &str) -> Result<f64, String> {
    // What:     `token.ok_or_else(|| format!("{field} missing"))?.parse::<f64>()
    //           .map_err(|_| format!("{field} is not a number"))`. Require the token, then
    //           parse it, converting both the missing and the non-numeric cases into
    //           messages. Tail expression.
    // Why:      One place for the "present and numeric" check.
    return token
        .ok_or_else(|| format!("{field} missing"))?
        .parse::<f64>()
        .map_err(|_| format!("{field} is not a number"))
}

/// Parse an optional token as `i32`,
///  with a field name for errors.
///
/// What:
///      `fn parse_i32(token: Option<&str>, field: &str) -> Result<i32, String>`.
/// Why:
///       Resize dimensions are signed integers (to match Smithay geometry).
fn parse_i32(token: Option<&str>, field: &str) -> Result<i32, String> {
    // What:     `token.ok_or_else(...)?.parse::<i32>().map_err(...)`. Same shape as
    //           `parse_f64` for integers.
    // Why:      One place for the integer check.
    return token
        .ok_or_else(|| format!("{field} missing"))?
        .parse::<i32>()
        .map_err(|_| format!("{field} is not an integer"))
}

/// Parse a pointer-button name.
///
/// What:
///      `fn parse_button(name: &str) -> Result<PointerButton, String>`.
///  Maps a name
///           to a variant.
/// Why:
///       Accept `left`/`right`/`middle` in click commands.
fn parse_button(name: &str) -> Result<PointerButton, String> {
    // What:     `match name { "left" => Ok(Left), "right" => Ok(Right), "middle" =>
    //           Ok(Middle), other => Err(...) }`. Map names to variants.
    // Why:      Reject unknown button names cleanly.
    match name {
        "left" => return Ok(PointerButton::Left),
        "right" => return Ok(PointerButton::Right),
        "middle" => return Ok(PointerButton::Middle),
        other => return Err(format!("unknown button: {other}")),
    }
}

/// What:
///      `#[cfg(test)] #[path = "protocol_tests.rs"] mod tests;`.
///  Declares the unit
///           test module (compiled only for tests) from the sibling file.
/// Why:
///       Keep the adversarial parser/response tests beside the grammar they guard.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./protocol_tests";
/// ```
#[cfg(test)]
#[path = "protocol_tests.rs"]
mod tests;
