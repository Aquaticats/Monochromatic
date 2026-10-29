//! Parse the runtime `scale` control verb independently of the compositor.

/// What:
///      `use super::Command;` brings the parent module's command union into scope.
///           `super` names the module that declared this file,
///  here `protocol`.
/// Why:
///       This parser returns the same typed command every other control verb produces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Command } from "./protocol";
/// ```
use super::Command;

/// What:
///      Import the output-scale value type from the display-independent geometry module.
///           `crate` names this package's own root.
/// Why:
///       The verb accepts exactly what the `--scale` startup option accepts,
///  through the
///           same parser,
///  so the two spellings cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { OutputScale } from "./screen_geometry";
/// ```
use crate::screen_geometry::OutputScale;

/// Parse the tokens after `scale` into the typed runtime output-scale command.
///
/// What:
///      `pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command,
///           String>`.
///  `pub(super)` exposes the function to the parent module only.
///  `&mut`
///           lends the token iterator so this function consumes the remaining tokens.
///           `Result<Command, String>` is success with a command or failure with a message.
/// Why:
///       Exactly one value keeps the verb unambiguous and rejects a second command
///           smuggled behind it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parse(tokens: Iterator<string>): Command { ... } // throws a string on bad input
/// ```
///
/// @example
/// ```ts
/// parse(["1.25"].values()); // => { kind: "scale", scale: { per120: 150 } }
/// ```
pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    // What:     `let Some(text) = tokens.next() else { ... }` binds the next token, or runs the
    //           `else` block, which must leave the function. `Err(...)` is the failure variant
    //           of `Result`; `.to_string()` copies the borrowed literal into an owned `String`.
    // Why:      A bare verb names no scale to apply.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const text = tokens.next().value;
    // if (text === undefined) throw "scale requires a value such as 1, 1.25, or 2";
    // ```
    let Some(text) = tokens.next() else {
        return Err("scale requires a value such as 1, 1.25, or 2".to_string());
    };

    // What:     `.is_some()` asks whether another token follows, without keeping it.
    // Why:      `scale 2 quit` or an embedded newline must fail whole, never switch.
    if tokens.next().is_some() {
        return Err("scale takes exactly one value, such as 1, 1.25, or 2".to_string());
    }

    // What:     `?` unwraps the shared parser's `Ok(scale)`, or returns its `Err(message)`
    //           from this function unchanged; that message already names the scale and the
    //           accepted range.
    // Why:      One parser keeps startup and runtime values identical.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const scale = OutputScale.parse(text); // throws the parser's message
    // ```
    let scale = OutputScale::parse(text)?;

    // What:     `Ok(Command::Scale(scale))` wraps the value in the command, then in success.
    // Why:      Hand the typed request to the control dispatcher.
    return Ok(Command::Scale(scale));
}
