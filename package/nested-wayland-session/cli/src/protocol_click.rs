//! The `click`,
//!  `drag`,
//!  and `scroll` commands with their `name=value` options.

/// Sibling modules of this crate.
use super::{
    parse_button, parse_f64,
    protocol_options::{split, Options},
    Command, PointerButton,
};

/// Parse `click X Y [left|right|middle] [mods=CHORD] [count=N] [interval=MS]`.
///
/// What:
///      `pub(super) fn parse_click(tokens: &mut SplitWhitespace) -> Result<Command, String>`.
///  `count` is 1 to 10 presses (2 is a double click),
///  `interval` the milliseconds between them (default 80),
///  and `mods` the keys held for the whole click.
/// Why:
///       A double click needs two presses inside the client's double-click time,
///  and a Ctrl or Shift click needs the key held,
///  as with a real hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseClick(tokens: string[]): Command { ... }
/// ```
pub(super) fn parse_click(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let x = parse_f64(tokens.next(), "click x")?;
    let y = parse_f64(tokens.next(), "click y")?;
    let rest = split(tokens, "click", &["mods", "count", "interval"])?;
    let button = button_argument(&rest, "click")?;
    let count = u32::try_from(rest.number("count", 1, (1, 10))?).map_err(|_| return "click count is too large".to_string())?;
    let interval_ms = rest.number("interval", 80, (0, 5_000))?;
    let modifiers = rest.modifiers()?;
    return Ok(Command::Click { x, y, button, count, interval_ms, modifiers })
}

/// Read the optional positional button.
///
/// What:
///      `fn button_argument(rest: &Options, verb: &str) -> Result<PointerButton, String>`.
///  Left when absent;
///  a second positional is refused.
/// Why:
///       Every pointer command takes the button the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buttonArgument(rest: Options, verb: string): PointerButton { ... }
/// ```
fn button_argument(rest: &Options, verb: &str) -> Result<PointerButton, String> {
    let mut positional = rest.positional.iter();
    let button = match positional.next() {
        Some(name) => parse_button(name)?,
        None => PointerButton::Left,
    };
    if positional.next().is_some() {
        return Err(format!("{verb} takes at most one button after its coordinates"))
    }
    return Ok(button)
}

/// Parse `drag X1 Y1 X2 Y2 [left|right|middle] [mods=CHORD] [steps=N] [step-ms=MS]`.
///
/// What:
///      `pub(super) fn parse_drag(tokens: &mut SplitWhitespace) -> Result<Command, String>`.
///  Presses at the start,
///  moves in `steps` equal steps (default 10) `step-ms` apart (default 16),
///  and releases at the end.
/// Why:
///       Sliders,
///  text selection,
///  and reordering react to the motion between press and release,
///  not only to the two end points.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseDrag(tokens: string[]): Command { ... }
/// ```
pub(super) fn parse_drag(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let from_x = parse_f64(tokens.next(), "drag x1")?;
    let from_y = parse_f64(tokens.next(), "drag y1")?;
    let to_x = parse_f64(tokens.next(), "drag x2")?;
    let to_y = parse_f64(tokens.next(), "drag y2")?;
    let rest = split(tokens, "drag", &["mods", "steps", "step-ms"])?;
    let button = button_argument(&rest, "drag")?;
    let steps = u32::try_from(rest.number("steps", 10, (1, 1_000))?).map_err(|_| return "drag steps is too large".to_string())?;
    let step_ms = rest.number("step-ms", 16, (0, 5_000))?;
    let modifiers = rest.modifiers()?;
    for (name, value) in [("x1", from_x), ("y1", from_y), ("x2", to_x), ("y2", to_y)] {
        if !value.is_finite() {
            return Err(format!("drag {name} must be a finite number"))
        }
    }
    return Ok(Command::Drag { from: (from_x, from_y), to: (to_x, to_y), button, steps, step_ms, modifiers })
}

/// Parse `scroll X Y DX DY [mods=CHORD] [steps=N] [step-ms=MS]`.
///
/// What:
///      `pub(super) fn parse_scroll(tokens: &mut SplitWhitespace) -> Result<Command, String>`.
///  A touchpad two-finger scroll of `DX`,
///  `DY` logical pixels in total,
///  split into `steps` events (default 1) and ended with the finger-lift stop.
/// Why:
///       Wheel notches and touchpad pixel scrolling reach a client as different events,
///  and a laptop user scrolls with the second kind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseScroll(tokens: string[]): Command { ... }
/// ```
pub(super) fn parse_scroll(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let x = parse_f64(tokens.next(), "scroll x")?;
    let y = parse_f64(tokens.next(), "scroll y")?;
    let dx = parse_f64(tokens.next(), "scroll dx")?;
    let dy = parse_f64(tokens.next(), "scroll dy")?;
    let rest = split(tokens, "scroll", &["mods", "steps", "step-ms"])?;
    if !rest.positional.is_empty() {
        return Err("scroll expects X Y DX DY and optional name=value options".to_string())
    }
    let steps = u32::try_from(rest.number("steps", 1, (1, 1_000))?).map_err(|_| return "scroll steps is too large".to_string())?;
    let step_ms = rest.number("step-ms", 16, (0, 5_000))?;
    let modifiers = rest.modifiers()?;
    for (name, value) in [("x", x), ("y", y), ("dx", dx), ("dy", dy)] {
        if !value.is_finite() {
            return Err(format!("scroll {name} must be a finite number"))
        }
    }
    return Ok(Command::Scroll { x, y, dx, dy, steps, step_ms, modifiers })
}
