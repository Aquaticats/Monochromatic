//! Focus ring movement inside the folder and track lists (D106).
//!
//! What:
//! `move_focus` returns the entry the ring moves to for an arrow,
//! Home,
//! End or page key;
//! `first_starting_with`
//! finds the entry for a typed letter;
//! `start_focus` picks where the ring begins when a list gains focus.
//! Why:
//! both lists mix entries that can hold the ring with entries that cannot (subfolder headers,
//! section headings),
//! so the movement is pure logic over a flag per entry and is unit-tested without a window.

/// What:
///      `pub enum ListMove` names the movement a key asks for.
/// Why:
///      The classifier turns keys into these,
/// and `move_focus` turns them into a new position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ListMove = "up" | "down" | "home" | "end" | "page-up" | "page-down";
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMove {
    /// One entry up.
    Up,
    /// One entry down.
    Down,
    /// The first entry.
    Home,
    /// The last entry.
    End,
    /// One page up.
    PageUp,
    /// One page down.
    PageDown,
}

/// What:
///      `first_selectable` gives the first entry that can hold the ring.
/// Why:
///      Home and the start position need it.
fn first_selectable(selectable: &[bool]) -> Option<usize> {
    for (index, can) in selectable.iter().enumerate() {
        if *can {
            return Some(index);
        }
    }
    return None;
}

/// What:
///      `last_selectable` gives the last entry that can hold the ring.
/// Why:
///      End needs it.
fn last_selectable(selectable: &[bool]) -> Option<usize> {
    for index in (0..selectable.len()).rev() {
        if selectable[index] {
            return Some(index);
        }
    }
    return None;
}

/// What:
///      `step_from` moves `steps` selectable entries from `current` toward the end (`forward`) or the start,
///  stopping at the last or first selectable entry.
/// Why:
///      Up,
///  Down and the page keys all move by whole selectable entries and stop at the ends without wrapping.
fn step_from(selectable: &[bool], current: usize, steps: usize, forward: bool) -> usize {
    let mut position = current;
    let mut remaining = steps;
    while remaining > 0 {
        let mut next: Option<usize> = None;
        if forward {
            for (at, can) in selectable.iter().enumerate().skip(position + 1) {
                if *can {
                    next = Some(at);
                    break;
                }
            }
        } else {
            for at in (0..position.min(selectable.len())).rev() {
                if selectable[at] {
                    next = Some(at);
                    break;
                }
            }
        }
        match next {
            Some(at) => position = at,
            None => break,
        }
        remaining -= 1;
    }
    return position;
}

/// What:
///      `move_focus` returns the entry the ring moves to,
///  or `None` when the list has no entry that can hold it.
/// Why:
///      Up and Down skip entries that cannot hold the ring,
///  Home and End jump to the first and last that can,
///  a page key moves by `page` entries,
///  and a ring that is not on a valid entry starts at the first one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function moveFocus(selectable: boolean[], current: number | null, move: ListMove, page: number): number | null;
/// ```
pub fn move_focus(selectable: &[bool], current: Option<usize>, movement: ListMove, page: usize) -> Option<usize> {
    let first = first_selectable(selectable)?;
    let last = last_selectable(selectable)?;
    let valid = match current {
        Some(at) if selectable.get(at).copied().unwrap_or(false) => Some(at),
        _ => None,
    };
    let Some(from) = valid else {
        return Some(if matches!(movement, ListMove::End | ListMove::PageDown) { last } else { first });
    };
    let page_steps = page.max(1);
    return Some(match movement {
        ListMove::Home => first,
        ListMove::End => last,
        ListMove::Up => step_from(selectable, from, 1, false),
        ListMove::Down => step_from(selectable, from, 1, true),
        ListMove::PageUp => step_from(selectable, from, page_steps, false),
        ListMove::PageDown => step_from(selectable, from, page_steps, true),
    });
}

/// What:
///      `start_focus` picks where the ring begins when a list gains focus.
/// Why:
///      The ring starts on the preferred entry (the playing track or the current folder) when it can hold the ring,
///      and otherwise on the first entry that can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const startFocus = (selectable: boolean[], preferred: number | null) => ...;
/// ```
pub fn start_focus(selectable: &[bool], preferred: Option<usize>) -> Option<usize> {
    let usable = match preferred {
        Some(at) if selectable.get(at).copied().unwrap_or(false) => Some(at),
        _ => None,
    };
    return match usable {
        Some(at) => Some(at),
        None => first_selectable(selectable),
    };
}

/// What:
///      `first_starting_with` finds the first entry whose label starts with the typed character,
///  ignoring case.
/// Why:
///      A letter key in the folder list jumps to the first folder with that initial (D106).
///      Entries without a label (headings) never match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const firstStartingWith = (labels: (string | null)[], letter: string) =>
///   labels.findIndex(label => label?.toLowerCase().startsWith(letter.toLowerCase())) ?? null;
/// ```
pub fn first_starting_with(labels: &[Option<&str>], letter: char) -> Option<usize> {
    let wanted: String = letter.to_lowercase().collect();
    for (index, label) in labels.iter().enumerate() {
        if matches!(label, Some(text) if text.to_lowercase().starts_with(&wanted)) {
            return Some(index);
        }
    }
    return None;
}

/// Unit tests in `list_focus_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "list_focus_tests.rs"]
mod tests;
