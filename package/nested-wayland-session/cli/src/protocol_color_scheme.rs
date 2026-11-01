//! Parse the runtime `color-scheme` control verb independently of the compositor.

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
///      Import the closed set of appearance values from the private-portal module.
///           `crate` names this package's own root.
/// Why:
///       Runtime switching accepts exactly the names the `--color-scheme` startup option
///           accepts,
///  through the same value parser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ColorSchemePreference } from "./appearance_portal";
/// ```
use crate::appearance_portal::ColorSchemePreference;

/// Parse the tokens after `color-scheme` into the typed runtime appearance command.
///
/// What:
///      `pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command,
///           String>`.
///  `pub(super)` exposes the function to the parent module only.
///  `&mut`
///           lends the token iterator so this function can consume the remaining tokens.
///           `Result<Command, String>` is success with a command or failure with a message,
///           because Rust has no exceptions.
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
/// parse(["light"].values()); // => { kind: "colorScheme", preference: "light" }
/// ```
pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    // What:     `tokens.next()` yields `Some(token)` or `None` once the line is exhausted.
    //           `let Some(name) = ... else { ... }` binds the present token, or runs the
    //           `else` block, which must leave the function. `Err(...)` is the failure
    //           variant of `Result`; `.to_string()` copies the borrowed literal into an owned
    //           `String`, the type the error channel holds.
    // Why:      A bare verb names no appearance to apply.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name = tokens.next().value;
    // if (name === undefined) throw "color-scheme requires dark or light";
    // ```
    let Some(name) = tokens.next() else {
        return Err("color-scheme requires dark or light".to_string());
    };

    // What:     `.is_some()` asks whether another token follows, without keeping it.
    // Why:      `color-scheme dark quit` or an embedded newline must fail whole, never switch.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (tokens.next().value !== undefined) throw "color-scheme takes exactly one value";
    // ```
    if tokens.next().is_some() {
        return Err("color-scheme takes exactly one value: dark or light".to_string());
    }

    // What:     `match` branches on the shared value parser's `Result`. `Ok(preference)` binds
    //           the parsed value and wraps it in the success variant; `Err(_)` discards the
    //           startup-option wording because this message names the control verb. `format!`
    //           builds an owned `String` with `{name}` interpolated.
    // Why:      One value parser keeps startup and runtime spellings identical.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { return { kind: "colorScheme", preference: parsePreference(name) }; }
    // catch { throw `color-scheme value must be dark or light, got: ${name}`; }
    // ```
    match ColorSchemePreference::parse(name) {
        Ok(preference) => return Ok(Command::ColorScheme(preference)),
        Err(_) => return Err(format!("color-scheme value must be dark or light, got: {name}")),
    }
}
