//! Parse pointer hover (`move`) and held-button (`button`) control requests.
//!
//! Both verbs act on the nested seat without a client test API,
//!  so their grammar lives beside the click grammar in the protocol module.

/// Reuse the protocol's numeric parsing, button names, and command representation.
use super::{parse_button, parse_f64, ButtonAction, Command, PointerButton};

/// Parse the coordinates of a `move` request (`move X Y`).
///
/// What:
///      `pub(super) fn parse_move(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String>`.
///  Both coordinates must be present, finite, and followed by nothing else.
/// Why:
///       A non-finite coordinate has no pointer position, and an extra token is ambiguous.
pub(super) fn parse_move(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let x = parse_f64(tokens.next(), "move x")?;
    let y = parse_f64(tokens.next(), "move y")?;
    if !x.is_finite() || !y.is_finite() {
        return Err("move coordinates must be finite".to_string());
    }
    if tokens.next().is_some() {
        return Err("move expects X Y".to_string());
    }
    return Ok(Command::Move { x, y });
}

/// Parse the arguments of a `button` request (`button down|up [left|right|middle]`).
///
/// What:
///      `pub(super) fn parse_button_command(tokens: &mut std::str::SplitWhitespace) ->
///           Result<Command, String>`.
///  The action comes first; the button is optional and defaults to `left`.
/// Why:
///       Matches the `click` default so a bare `button down` holds the primary button.
pub(super) fn parse_button_command(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let action = match tokens.next() {
        Some("down") => ButtonAction::Down,
        Some("up") => ButtonAction::Up,
        Some(other) => return Err(format!("button action must be down or up, got: {other}")),
        None => return Err("button requires down or up".to_string()),
    };
    let button = match tokens.next() {
        Some(name) => parse_button(name)?,
        None => PointerButton::Left,
    };
    if tokens.next().is_some() {
        return Err("button expects down|up and an optional left|right|middle".to_string());
    }
    return Ok(Command::Button { action, button });
}
