//! Key chords through the nested seat's keyboard.

/// Error handling with context.
use anyhow::{bail, Result};
/// Smithay compositor types.
use smithay::backend::input::KeyState;

/// Sibling modules of this crate.
use crate::{input::send_key, key_chord::Chord, protocol::KeyAction, state::Compositor};

/// Press,
///  release,
///  or tap a chord.
///
/// What:
///      `pub fn apply(state: &mut Compositor, chord: &Chord, action: KeyAction) -> Result<()>`.
///  Press goes through the keys in order,
///  release in reverse,
///  and tap is a press then a release.
///  A hold is timed and runs through the timeline instead.
/// Why:
///       `ctrl+shift+f10` must reach the client as Ctrl down,
///  Shift down,
///  F10 down,
///  F10 up,
///  Shift up,
///  Ctrl up,
///  the order a person's fingers use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function apply(state: Compositor, chord: Chord, action: KeyAction): void { ... }
/// ```
pub fn apply(state: &mut Compositor, chord: &Chord, action: KeyAction) -> Result<()> {
    match action {
        KeyAction::Press => press_all(state, chord),
        KeyAction::Release => release_all(state, chord),
        KeyAction::Tap => {
            press_all(state, chord);
            release_all(state, chord);
        }
        KeyAction::Hold { millis } => bail!("a {millis} ms hold is timed and cannot run as an immediate key action"),
    }
    return Ok(())
}

/// Press every key of a chord in order.
///
/// What:
///      `fn press_all(state: &mut Compositor, chord: &Chord)`.
/// Why:
///       Modifiers must be down before the key they modify.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const code of chord.keys) send(code, "down");
/// ```
fn press_all(state: &mut Compositor, chord: &Chord) {
    for code in &chord.keys {
        send_key(state, *code, KeyState::Pressed);
    }
}

/// Release every key of a chord in reverse order.
///
/// What:
///      `fn release_all(state: &mut Compositor, chord: &Chord)`.
/// Why:
///       The key goes up before the modifiers do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const code of [...chord.keys].reverse()) send(code, "up");
/// ```
fn release_all(state: &mut Compositor, chord: &Chord) {
    for code in chord.keys.iter().rev() {
        send_key(state, *code, KeyState::Released);
    }
}
