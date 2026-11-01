//! Space held open above the lines of a replaced text:
//!  what an external change carries over,
//!  and how the held space is installed once the new text is shown.

/// The map refresh that follows holding.
use super::refresh;
/// The stamp of the displayed text.
use crate::native::{State, annotate::displayed};
/// `ChangeSet` is Helix's edit list,
///  and `Assoc` says which side of inserted text a mapped position stays on.
use helix_core::{Assoc, ChangeSet};
/// Space held open above a line of a reloaded text.
use ide_app::annotation::Held;
/// `Duration` and `Instant` time held space.
use std::time::{Duration, Instant};

/// What:
///  How long the space of a replaced text's virtual rows stays open while nothing is painted in it.
/// Why:
///  Hints for a reloaded text were measured to arrive 20 to 40 ms after the reload;
///  one second covers that
///      many times over,
///  and space that no answer fills does not stay empty for long.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ROW_HOLD = 1000; // milliseconds
/// ```
const ROW_HOLD: Duration = Duration::from_millis(1000);

/// What:
///  Before an external change is applied,
///  where each block of the displayed text will be in the new one:
///       the mapped character position of its line's start and the heights of its hint part and message part.
///       `&ChangeSet` lends the edits from the displayed text to the new one;
///  the answer is a list of triples.
/// Why:
///  The rows of the old text are never painted for the new one,
///  but their space is held open so lines the
///      change did not touch stay where they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function carried(current: State, changes: ChangeSet): [position: number, hints: number, messages: number][];
/// ```
pub(in crate::native) fn carried(current: &State, changes: &ChangeSet) -> Vec<(usize, f32, f32)> {
    let text = current.document.text();
    let mut kept = Vec::new();
    for block in &current.blocks {
        let start = text.line_to_char(block.line.min(text.len_lines()));
        // Text inserted at the line's start pushes the line, and its rows with it, further down.
        let position = changes.map_pos(start, Assoc::After);
        kept.push((position, block.hint_part(), block.message_part()));
    }
    return kept;
}

/// What:
///  After the change was applied,
///  hold the `carried` space above the lines the positions now lie on,
///  and
///       rebuild the map.
///  `Vec<...>` is moved in.
/// Why:
///  Hint space is given up as soon as hints for those lines arrive;
///  message space stays until its time
///      has passed,
///  because diagnostics of one text arrive in several parts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hold(current: State, carried: [number, number, number][], scale: number): void;
/// ```
pub(in crate::native) fn hold(current: &mut State, carried: Vec<(usize, f32, f32)>, scale: f32) {
    let text = current.document.text();
    let length = text.len_chars();
    let mut parts: Vec<Held> = Vec::new();
    for (position, hints, messages) in carried {
        let line = text.char_to_line(position.min(length));
        // What: `last_mut()` lends the latest entry for change, or nothing for an empty list.
        // Why: Two old lines that became one keep the taller space of the two; positions ascend, so such
        //      lines follow each other.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const last = parts.at(-1); if (last?.line === line) { ... } else parts.push(...);
        // ```
        if let Some(last) = parts.last_mut()
            && last.line == line
        {
            last.hints = last.hints.max(hints);
            last.messages = last.messages.max(messages);
        } else {
            parts.push(Held {
                line,
                hints,
                messages,
            });
        }
    }
    let next = displayed(current);
    tracing::debug!(
        lines = parts.len(),
        ?next,
        "held the space of replaced rows"
    );
    current
        .annotations
        .hold(next, parts, Instant::now() + ROW_HOLD);
    refresh(current, scale);
}
