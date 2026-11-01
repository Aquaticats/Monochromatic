//! Search overlay input rules follow editord without coupling them to native widgets or filesystem reads.

/// Injected elapsed time keeps double-Shift tests independent of wall-clock adjustments and sleep timing.
use std::time::Duration;

/// What:
///  Rust's Unicode whitespace predicate includes NEXT LINE and excludes the byte-order mark.
/// Why:
///  The reference's JavaScript trim does the reverse,
///  so copied queries need these explicit boundaries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inputSpace = (character: string) => character.trim() === '';
/// ```
fn input_space(character: char) -> bool {
    return character == '\u{feff}' || (character != '\u{0085}' && character.is_whitespace());
}

/// Normalized query and the presentation-only content filter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchInput {
    /// Trimmed pattern sent unchanged to filename matching and ripgrep's regex engine.
    pub query: String,
    /// A percent sign at the beginning of raw input hides filename results.
    pub content_only: bool,
}

/// Parse the same raw-prefix and trimming order as the reference overlay.
impl SearchInput {
    /// Empty input and a bare content-only prefix clear results rather than start a search.
    pub fn parse(raw: &str) -> Option<Self> {
        let content_only = raw.starts_with('%');
        // Borrow the stripped view when a percent prefix exists; otherwise keep the original raw input.
        let query = raw
            .strip_prefix('%')
            .unwrap_or(raw)
            .trim_matches(input_space);
        if query.is_empty() {
            return None;
        }
        return Some(Self {
            query: query.to_string(),
            content_only,
        });
    }
}

/// Shift releases form one opening gesture unless another key was pressed between them.
#[derive(Default, Debug)]
pub struct DoubleShift {
    /// The first release's monotonic elapsed time,
    ///  absent after a completed gesture.
    last_release: Option<Duration>,
    /// A non-Shift keypress invalidates the prior release as the start of a double-Shift gesture.
    intervening_key: bool,
}

/// Observe key events without consuming normal source,
///  tree,
///  or input-field behavior.
impl DoubleShift {
    /// Classify the key at the toolkit boundary and retain only the gesture-relevant fact.
    pub fn press(&mut self, is_shift: bool) {
        if !is_shift {
            self.intervening_key = true;
        }
    }

    /// Return true only for a second uninterrupted Shift release strictly within 400 milliseconds.
    pub fn release(&mut self, is_shift: bool, elapsed: Duration) -> bool {
        if !is_shift {
            return false;
        }
        // Timely means a previous release exists and the gap since it is under 400 milliseconds.
        let mut timely = false;
        if let Some(last) = self.last_release
            && let Some(gap) = elapsed.checked_sub(last)
        {
            timely = gap < Duration::from_millis(400);
        }
        if !self.intervening_key && timely {
            self.last_release = None;
            self.intervening_key = false;
            return true;
        }
        self.last_release = Some(elapsed);
        self.intervening_key = false;
        return false;
    }
}
