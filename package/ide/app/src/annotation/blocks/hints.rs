//! The store's hint labels and the space held over from the previous text:
//!  one label list per line,
//!  the labels an answer asked for,
//!  and the space an external change keeps.

/// The store these methods extend,
///  and the space a reload holds open.
use super::super::{
    Annotations,
    Held,
    Label,
};
/// One accepted hint snapshot,
///  and the stamp of the text it describes.
use crate::language::{
    hints::HintsSnapshot,
    identity::DocumentStamp,
};
/// The text the hints stand on.
use helix_core::Rope;
/// The point on the clock a held space ends at.
use std::time::Instant;

/// Group hints by line,
///  and hold space across a reload.
impl Annotations {
    /// What:
    ///  Take the labels of `snapshot` into the per-line store:
    ///  lines the snapshot asked about are replaced
    ///       by its answer,
    ///  other lines keep the labels an earlier snapshot of the same revision gave them,
    ///  and a hint of the snapshot that stands on another line is left out.
    ///       `&Rope` lends the text the snapshot describes.
    /// Why:
    ///  Hints are requested for one view height above and two below the visible lines.
    ///  Replacing all hints
    ///      with each answer would remove the rows of lines scrolled away from,
    ///  and put them back when the
    ///      view returns,
    ///  moving the text both times.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// mergeLabels(text: Rope, snapshot: HintsSnapshot): void;
    /// ```
    pub(in crate::annotation) fn merge_labels(
        &mut self,
        text: &Rope,
        snapshot: &HintsSnapshot,
    ) {
        if self.labelled != Some(snapshot.stamp) {
            self.labels.clear();
            self.placed.clear();
            // `Some(...)` records which revision the labels now describe.
            self.labelled = Some(snapshot.stamp);
        }
        let asked = snapshot.first_line..snapshot.last_line;
        // What: `range(...)` walks the stored lines inside the asked range; `copied()` yields the line numbers
        //       themselves and `collect()` gathers them into a list.
        // Why: The lines are removed afterwards; a map cannot be changed while it is being walked.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const stale = [...labels.keys()].filter(line => line >= first && line < last);
        // ```
        // The lines in the answered range are copied out so the maps can be changed while they are walked.
        let mut stale: Vec<usize> = Vec::new();
        for (line, _) in self.labels.range(asked.clone()) {
            stale.push(*line);
        }
        for line in stale {
            self.labels.remove(&line);
            self.placed.remove(&line);
        }
        let length = text.len_chars();
        let mut outside = 0_usize;
        let mut inside = 0_usize;
        for hint in &snapshot.hints {
            let trimmed = hint.label.trim();
            if trimmed.is_empty() {
                continue;
            }
            let line = text.char_to_line(hint.position.min(length));
            // A server answers for every node that reaches into the asked lines (TypeScript 7 answers a request for
            // lines 100 to 200 with the four hints of the statement on line 96 whose body spans the file), so
            // an answer can hold hints of lines it did not ask about. Their old hints were not removed above, so
            // keeping these would add the same hints to that line again with every answer. A line the answer did
            // not ask about keeps what the answer that asked for it gave.
            if !asked.contains(&line) {
                outside += 1;
                continue;
            }
            inside += 1;
            // `entry(...).or_default()` lends the line's list, creating an empty one first when absent.
            self.labels.entry(line).or_default().push(Label {
                position: hint.position,
                // `to_string` copies the borrowed trimmed text into an owned `String`.
                text: trimmed.to_string(),
            });
            self.placed.remove(&line);
        }
        // What: The asked lines count as covered only when the answer put a hint on one of them.
        // Why: A server that is still loading the project answers with no hint or with some; the view that
        //      settles over the same lines then asks again, as it did before lines were counted as covered.
        //      Lines that really hold no hint are asked again on each settle for the same reason.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (inside > 0) answered.note(snapshot.stamp, asked);
        // ```
        if inside > 0 {
            self.answered.note(snapshot.stamp, asked.clone());
        } else {
            // The answer replaced the hints of the asked lines with none, so an earlier answer no longer covers them.
            self.answered.uncover(asked.clone());
        }
        if outside > 0 {
            tracing::debug!(
                first = asked.start,
                last = asked.end,
                outside,
                "an answer held hints of lines it did not ask about; they were left out"
            );
        }
        // Hint space held from the previous revision is given up for the lines this answer covers.
        for part in &mut self.held {
            if asked.contains(&part.line) {
                part.hints = 0.0;
            }
        }
    }

    /// What:
    ///  Hold `parts` open above lines of the text `next` until `until`.
    ///  `Vec<Held>` is moved in.
    /// Why:
    ///  Called when an external change replaced the displayed text:
    ///  the rows of the old text are not
    ///      painted for the new one,
    ///  but their space stays until hints and diagnostics of the new text arrive
    ///      or the time passes,
    ///  so unchanged lines do not move twice.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hold(next: DocumentStamp, parts: Held[], until: number): void;
    /// ```
    pub fn hold(
        &mut self,
        next: DocumentStamp,
        parts: Vec<Held>,
        until: Instant,
    ) {
        self.held = parts;
        self.held_for = Some(next);
        self.held_until = Some(until);
        self.version += 1;
    }

    /// What:
    ///  Give up held space whose time has passed at `now`;
    ///  the answer says whether anything was given up.
    /// Why:
    ///  A server that never answers for the new text must not leave empty rows forever.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// expire(now: number): boolean;
    /// ```
    pub fn expire(
        &mut self,
        now: Instant,
    ) -> bool {
        // No deadline means nothing is due; otherwise the deadline must have passed.
        let due = match self.held_until {
            Some(until) => now >= until,
            None => false,
        };
        if !due {
            return false;
        }
        let had = !self.held.is_empty();
        self.held.clear();
        self.held_until = None;
        if had {
            self.version += 1;
        }
        return had;
    }
}
