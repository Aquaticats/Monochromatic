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

/// Error text that is built only on failure.
use crate::lazy_error_text::{
    option_or_text,
    result_or_text,
};

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

/// The one request type a line parses into.
#[path = "protocol_commands.rs"]
mod commands;

/// The one answer line a request yields.
#[path = "protocol_responses.rs"]
mod responses;

/// One request line turned into a command.
#[path = "protocol_command_line.rs"]
mod command_line;

/// The request type the executor matches on,
///  so it is part of the protocol's interface.
pub use commands::Command;

/// The answer type and its one-line formatting,
///  so they are part of the protocol's interface.
pub use responses::{
    Response,
    format_response,
};

/// Parsing one request line,
///  so it is part of the protocol's interface.
pub use command_line::parse_command;

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
    return Ok(command);
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
    return Ok(Command::Resize {
        width,
        height,
    });
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
fn parse_f64(
    token: Option<&str>,
    field: &str,
) -> Result<f64, String> {
    // What:     `token.ok_or_else(|| format!("{field} missing"))?.parse::<f64>()
    //           .map_err(|_| format!("{field} is not a number"))`. Require the token, then
    //           parse it, converting both the missing and the non-numeric cases into
    //           messages. Tail expression.
    // Why:      One place for the "present and numeric" check.
    let text = option_or_text(token, format_args!("{field} missing"))?;
    return result_or_text(text.parse::<f64>(), format_args!("{field} is not a number"));
}

/// Parse an optional token as `i32`,
///  with a field name for errors.
///
/// What:
///      `fn parse_i32(token: Option<&str>, field: &str) -> Result<i32, String>`.
/// Why:
///       Resize dimensions are signed integers (to match Smithay geometry).
fn parse_i32(
    token: Option<&str>,
    field: &str,
) -> Result<i32, String> {
    // What:     `token.ok_or_else(...)?.parse::<i32>().map_err(...)`. Same shape as
    //           `parse_f64` for integers.
    // Why:      One place for the integer check.
    let text = option_or_text(token, format_args!("{field} missing"))?;
    return result_or_text(
        text.parse::<i32>(),
        format_args!("{field} is not an integer"),
    );
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
