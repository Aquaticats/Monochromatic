//! Turning one request line into a command:
//!  the two verbs whose payload keeps its spaces are handled first,
//!  and every other verb goes to the parser that knows its arguments.
//!
//! Split out of `protocol.rs` to keep that file under the code-line limit.

/// An owned filesystem path a screenshot request carries.
use std::path::PathBuf;

/// Error text for a missing argument.
use crate::lazy_error_text::option_or_text;

/// The command a line parses into,
///  and the two helpers shared by the verbs without a parser of their own.
use super::{
    Command,
    accent,
    click,
    color_scheme,
    dropfile,
    job,
    key,
    launcher,
    no_arguments,
    notification,
    parse_resize,
    pointer,
    record,
    scale,
    session,
    wheel,
};
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
    let verb = option_or_text(tokens.next(), format_args!("empty command"))?;

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
        "launcher-clear" => {
            return no_arguments(&mut tokens, "launcher-clear", Command::LauncherClear);
        }
        "launcher-log" => return launcher::parse_log(&mut tokens),
        "accent-color" => return accent::parse(&mut tokens),
        "notifications" | "notification-log" | "notification-clear" | "notification-close" => {
            return notification::parse(verb, &mut tokens);
        }
        "jobs" | "job-log" | "job-clear" | "job-suspend" | "job-resume" | "job-cancel"
        | "job-refresh" => {
            return job::parse(verb, &mut tokens);
        }
        other => return Err(format!("unknown command: {other}")),
    }
}
