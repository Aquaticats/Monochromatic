//! Parse real-wheel control input independently of the compositor.

/// Reuse the protocol's numeric parsing and command representation.
use super::{
    parse_f64,
    protocol_options::split,
    Command,
};

/// Largest wheel travel one command accepts,
///  in notches per axis.
///
/// What:
///      `pub const MAX_NOTCHES: f64 = 10_000.0;`.
/// Why:
///       Keeps the v120 value far inside a 32-bit integer and a typo from scrolling a library for minutes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_NOTCHES = 10_000;
/// ```
pub const MAX_NOTCHES: f64 = 10_000.0;

/// Parse `wheel X Y HORIZONTAL VERTICAL [mods=CHORD]`.
///
/// What:
///      `pub(super) fn parse(tokens: &mut SplitWhitespace) -> Result<Command, String>`.
///  The amounts are notches and may be fractional (`0.25` is a quarter notch,
///  `-1.5` one and a half up);
///  the compositor sends them as Wayland `axis_value120` in 1/120 steps,
///  the way a high-resolution wheel does.
/// Why:
///       High-resolution wheels and free-spinning wheels send partial notches,
///  and a client that smooths scrolling must be tested with them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseWheel(tokens: string[]): Command { ... }
/// ```
pub(super) fn parse(tokens: &mut std::str::SplitWhitespace) -> Result<Command, String> {
    let x = parse_f64(tokens.next(), "wheel x")?;
    let y = parse_f64(tokens.next(), "wheel y")?;
    let horizontal = parse_f64(tokens.next(), "horizontal notches")?;
    let vertical = parse_f64(tokens.next(), "vertical notches")?;
    if !x.is_finite() || !y.is_finite() {
        return Err("wheel coordinates must be finite".to_string());
    }
    if !horizontal.is_finite() || !vertical.is_finite() {
        return Err("wheel notch amounts must be finite".to_string());
    }
    if horizontal.abs() > MAX_NOTCHES || vertical.abs() > MAX_NOTCHES {
        return Err(format!("wheel notch amounts must be within {MAX_NOTCHES} notches"));
    }
    let rest = split(tokens, "wheel", &["mods"])?;
    if !rest.positional.is_empty() {
        return Err("wheel expects X Y HORIZONTAL_NOTCHES VERTICAL_NOTCHES and an optional mods=CHORD".to_string());
    }
    let modifiers = rest.modifiers()?;
    return Ok(Command::Wheel { x, y, horizontal, vertical, modifiers });
}
