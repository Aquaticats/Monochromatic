//! `type`:
//!  text as key events,
//!  with a keymap switch for characters the layout does not have.

/// Error handling with context.
use anyhow::{bail, Context, Result};
/// Smithay compositor types.
use smithay::backend::input::KeyState;
/// Structured logging.
use tracing::{debug, info};

/// Sibling modules of this crate.
use crate::{
    input::send_key,
    keymap::{char_to_key, LEFT_SHIFT},
    state::Compositor,
    unicode_keymap::{base_keymap_text, spare_evdev, with_unicode_keys, MAX_SPARE_KEYS},
};

/// Find the end of the chunk that one keymap can type,
///  and the characters that need spare keys.
///
/// What:
///      `pub fn next_chunk(characters: &[char]) -> (usize, Vec<char>)`.
///  Takes characters until a new off-layout one would exceed `MAX_SPARE_KEYS`.
///  Returns the number of characters taken and the distinct off-layout ones in first-use order.
/// Why:
///       A long text of many scripts is typed in as few keymap switches as possible.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nextChunk(characters: string[]): [number, string[]] { ... }
/// ```
#[must_use]
pub fn next_chunk(characters: &[char]) -> (usize, Vec<char>) {
    let mut spare: Vec<char> = Vec::new();
    let mut taken = 0;
    for character in characters {
        if char_to_key(*character).is_none() && !spare.contains(character) {
            if spare.len() == MAX_SPARE_KEYS {
                break;
            }
            spare.push(*character);
        }
        taken += 1;
    }
    return (taken, spare)
}

/// Refuse control characters no key can type.
///
/// What:
///      `pub fn check_typable(text: &str) -> Result<()>`.
///  Tab and line feed map to their keys;
///  every other control character fails the whole text before any key is sent.
/// Why:
///       Silently dropping a character would let a test pass on text it never typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkTypable(text: string): void { ... }
/// ```
pub fn check_typable(text: &str) -> Result<()> {
    for (position, character) in text.chars().enumerate() {
        if character.is_control() && char_to_key(character).is_none() {
            bail!("type cannot send the control character U+{:04X} at position {position}", u32::from(character));
        }
    }
    return Ok(())
}

/// Tap one character on its key,
///  holding Shift when the layout needs it.
///
/// What:
///      `fn tap_character(state: &mut Compositor, character: char, spare: &[char])`.
///  A US-layout character uses its key;
///  anything else uses the spare key at its index in `spare`.
/// Why:
///       Ordinary text keeps its Shift and key events,
///  and only off-layout characters use the keymap trick.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tapCharacter(state: Compositor, character: string, spare: string[]): void { ... }
/// ```
fn tap_character(state: &mut Compositor, character: char, spare: &[char]) {
    if let Some((evdev, shift)) = char_to_key(character) {
        if shift {
            send_key(state, LEFT_SHIFT, KeyState::Pressed);
        }
        send_key(state, evdev, KeyState::Pressed);
        send_key(state, evdev, KeyState::Released);
        if shift {
            send_key(state, LEFT_SHIFT, KeyState::Released);
        }
        return;
    }
    let Some(index) = spare.iter().position(|candidate| return *candidate == character) else {
        debug!("character U+{:04X} has neither a layout key nor a spare key", u32::from(character));
        return;
    };
    let evdev = spare_evdev(index);
    send_key(state, evdev, KeyState::Pressed);
    send_key(state, evdev, KeyState::Released);
}

/// Type text into the focused client.
///
/// What:
///      `pub fn type_text(state: &mut Compositor, text: &str) -> Result<()>`.
///  Each character is a key tap.
///  Characters the US layout lacks (accents,
///  CJK,
///  Hangul,
///  emoji) are typed on spare keys of a temporary keymap that carries their Unicode keysyms,
///  and the original keymap is sent back afterwards.
/// Why:
///       The hosted client reads `KeyEvent.text` from the key's keysym,
///  so this is the text a keyboard with those characters would deliver.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function typeText(state: Compositor, text: string): void { ... }
/// ```
///
/// @example
/// ```ts
/// typeText(state, "Björk"); // B, j, r, k on their keys; ö on a spare key
/// ```
pub fn type_text(state: &mut Compositor, text: &str) -> Result<()> {
    check_typable(text)?;
    let characters: Vec<char> = text.chars().collect();
    let keyboard = state.seat.get_keyboard().context("nested seat has no keyboard")?;
    let mut index = 0;
    while index < characters.len() {
        let (taken, spare) = next_chunk(&characters[index..]);
        let base = if spare.is_empty() {
            None
        } else {
            let base = base_keymap_text().map_err(anyhow::Error::msg)?;
            let extended = with_unicode_keys(&base, &spare).map_err(anyhow::Error::msg)?;
            info!(spare = spare.len(), "type: switching to a keymap with spare Unicode keys");
            keyboard.set_keymap_from_string(state, extended).context("the keymap with spare keys was refused")?;
            Some(base)
        };
        for character in &characters[index..index + taken] {
            tap_character(state, *character, &spare);
        }
        if let Some(original) = base {
            keyboard.set_keymap_from_string(state, original).context("restoring the keymap was refused")?;
            info!("type: restored the original keymap");
        }
        index += taken;
    }
    return Ok(())
}

/// What:
///      `#[cfg(test)] #[path = "input_text_tests.rs"] mod tests;`.
/// Why:
///       Keep the chunking tests beside the typing code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./input_text_tests";
/// ```
#[cfg(test)]
#[path = "input_text_tests.rs"]
mod tests;
