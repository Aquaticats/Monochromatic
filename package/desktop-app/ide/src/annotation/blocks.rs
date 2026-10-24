//! The blocks of virtual rows the store's hints and diagnostics call for,
//!  one per annotated line.
//!
//! Three things decide a block:
//!  the hints accumulated for the displayed revision,
//!  the message rows of its
//! diagnostics,
//!  and space held over from the previous revision after an external change.
//!  Assembly packs hint
//! rows with the production shaper,
//!  so a block's height is exactly what a frame will paint.

/// The store these methods extend,
///  its records,
///  and the hint label type.
use super::{Annotations, Fitted, Held, Label, Placed};
/// Hint rows are packed against the shaped code line.
use crate::annotation_layout::pack;
/// The displayed text and its line starts.
use crate::document::Document;
/// One accepted hint snapshot and the stamp of the text it describes.
use crate::language::{hints::HintsSnapshot, identity::DocumentStamp};
/// The shaper that paints the source measures hint labels and their anchor positions.
use crate::shaped_text::TextShaper;
/// The assembled record of one line's rows,
///  and the sizes that fit a message row to the text area.
use crate::virtual_row::{Block, CONTINUATION_INDENT, MESSAGE_ROWS, MessageRow, ROW_TEXT};
/// What:
///  `Rope` is Helix's character-indexed text buffer.
/// Why:
///  A hint belongs to the line its position lies on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// What:
///  `BTreeMap` is a map kept in key order;
///  `Arc` shares one immutable block between the window state and
///       the frames that paint it;
///  `Instant` is a point on a monotonic clock.
/// Why:
///  Blocks are handed out in line order and compared by content when deciding whether to repaint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = Readonly<T>;
/// ```
use std::{
    collections::{BTreeMap, btree_map::Entry},
    ops::Range,
    sync::Arc,
    time::Instant,
};

/// What:
///  The rows of one line's messages broken so that each fits `limit` logical pixels,
///  capped again at [`MESSAGE_ROWS`] rows for each message,
///  with the number of rows that had to be broken by shaping.
///  `widths` holds the shaped width of every row of `rows`;
///  only a row wider than the limit is shaped again.
/// Why:
///  The source view has no horizontal scrolling,
///  so a message must fit the text area it stands in.
///  The first row of a message may use the whole width,
///  and every later row starts [`CONTINUATION_INDENT`] further right,
///  so the pieces of a row that is not itself continued break in two steps.
///  A cap that counted rows before they were broken would let a narrow window show many times more rows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fitMessages(rows: MessageRow[], widths: number[], shaper: TextShaper, scale: number,
///   limit: number): [rows: MessageRow[], broken: number];
/// ```
fn fit_messages(
    rows: &[MessageRow],
    widths: &[f32],
    shaper: &mut TextShaper,
    scale: f32,
    limit: f32,
) -> (Vec<MessageRow>, u64) {
    let mut broken: u64 = 0;
    let narrowest = ROW_TEXT;
    let mut fitted: Vec<MessageRow> = Vec::new();
    let mut from = 0;
    while from < rows.len() {
        // What: A line of the message is the row that starts it and the rows that follow it softly.
        // Why: The rows of one line were cut at 80 columns; breaking each of them alone would leave a short tail
        //      on every one, so a line with a row wider than the limit is joined and broken again as a whole.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const line = rows.slice(from, rows.findIndex((row, i) => i > from && row.gap === undefined));
        // ```
        let mut to = from + 1;
        while to < rows.len() && rows[to].gap.is_some() {
            to += 1;
        }
        let head = &rows[from];
        let indent = if head.continued {
            CONTINUATION_INDENT
        } else {
            0.0
        };
        // Every row of the line must fit the limit for the line to stay as it was.
        let mut fits = true;
        for index in from..to {
            let own = if rows[index].continued {
                CONTINUATION_INDENT
            } else {
                0.0
            };
            if !(own + widths[index] <= limit) {
                fits = false;
                break;
            }
        }
        if fits {
            fitted.extend_from_slice(&rows[from..to]);
            from = to;
            continue;
        }
        broken += (to - from) as u64;
        let mut joined = head.text.clone();
        for row in &rows[from + 1..to] {
            joined.push_str(row.gap.unwrap_or(" "));
            joined.push_str(&row.text);
        }
        let mut pieces = shaper.wrap_row_text(&joined, scale, (limit - indent).max(narrowest));
        if !head.continued && pieces.len() > 1 {
            // The rest of the line starts a continuation, which is indented, so it breaks again at its own width.
            let rest = joined[pieces[0].len()..].trim_start();
            let tail =
                shaper.wrap_row_text(rest, scale, (limit - CONTINUATION_INDENT).max(narrowest));
            pieces.truncate(1);
            pieces.extend(tail);
        }
        if pieces.is_empty() {
            fitted.extend_from_slice(&rows[from..to]);
            from = to;
            continue;
        }
        for (index, piece) in pieces.into_iter().enumerate() {
            fitted.push(MessageRow {
                start: head.start,
                continued: head.continued || index > 0,
                severity: head.severity,
                gap: if index == 0 { head.gap } else { Some(" ") },
                text: piece,
            });
        }
        from = to;
    }
    // Each message is its first row and the continued rows after it.
    let mut capped = Vec::new();
    let mut first = 0;
    while first < fitted.len() {
        let mut end = first + 1;
        while end < fitted.len() && fitted[end].continued {
            end += 1;
        }
        if end - first > MESSAGE_ROWS {
            capped.extend_from_slice(&fitted[first..first + MESSAGE_ROWS - 1]);
            capped.push(MessageRow {
                text: format!("… {} more lines", end - first + 1 - MESSAGE_ROWS),
                ..fitted[first + MESSAGE_ROWS - 1].clone()
            });
        } else {
            capped.extend_from_slice(&fitted[first..end]);
        }
        first = end;
    }
    return (capped, broken);
}

/// Group hints by line,
///  hold space across a reload,
///  and assemble blocks.
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
    pub(super) fn merge_labels(&mut self, text: &Rope, snapshot: &HintsSnapshot) {
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
            // `entry(...).or_default()` lends the line's list, creating an empty one first when absent.
            self.labels.entry(line).or_default().push(Label {
                position: hint.position,
                // `to_string` copies the borrowed trimmed text into an owned `String`.
                text: trimmed.to_string(),
            });
            self.placed.remove(&line);
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
    pub fn hold(&mut self, next: DocumentStamp, parts: Vec<Held>, until: Instant) {
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
    pub fn expire(&mut self, now: Instant) -> bool {
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

    /// What:
    ///  The blocks of every annotated line of the text `displayed`,
    ///  in line order.
    ///  `&mut TextShaper` is
    ///       lent to pack hint rows of lines not packed yet at `scale` and `wrap`,
    ///  the width of the text area in logical pixels,
    /// or nothing while the source view has none;
    ///  `focus` is the range of lines near the view,
    /// the only ones that are packed again for a new width;
    ///  `&Document` lends the displayed text.
    /// Why:
    ///  This is the single source of block heights for the vertical mapping and of the rows a frame
    ///      paints.
    ///  Snapshots and held space of any other text contribute nothing.
    ///  Hints are packed against the line as it is broken at the wrap width,
    ///  and message rows are broken to fit it,
    ///  so a block never reaches past the text area.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// assemble(displayed: DocumentStamp, document: Document, shaper: TextShaper, scale: number,
    ///   wrap: number | undefined, focus: Range): Block[];
    /// ```
    pub fn assemble(
        &mut self,
        displayed: DocumentStamp,
        document: &Document,
        shaper: &mut TextShaper,
        scale: f32,
        wrap: Option<f32>,
        focus: &Range<usize>,
    ) -> Vec<Arc<Block>> {
        if self.placed_scale != scale {
            // Label widths and caret positions are measured in pixels of one scale.
            self.placed.clear();
            self.natural.clear();
            self.wrapped.clear();
            self.placed_scale = scale;
        }
        let labelled = self.labelled == Some(displayed);
        let diagnosed = self.diagnostics_current(displayed);
        // Packings and breaks of snapshots that no longer describe the displayed text are never used again.
        if !labelled {
            self.placed.clear();
        }
        if !diagnosed {
            self.wrapped.clear();
        }
        // Every line that owns something, with the space held for it.
        let mut lines: BTreeMap<usize, (f32, f32)> = BTreeMap::new();
        if labelled {
            for line in self.labels.keys() {
                lines.insert(*line, (0.0, 0.0));
            }
        }
        if diagnosed {
            for line in self.messages.keys() {
                lines.insert(*line, (0.0, 0.0));
            }
        }
        if self.held_for == Some(displayed) {
            for part in &self.held {
                lines.insert(part.line, (part.hints, part.messages));
            }
        }
        let count = document.text().len_lines();
        let mut blocks = Vec::new();
        for (line, held) in lines {
            if line >= count {
                continue;
            }
            let mut hint_rows = 0;
            let mut hints = Vec::new();
            // `get` lends the line's labels when the line has any.
            if labelled && let Some(labels) = self.labels.get(&line) {
                // What: A line is packed when it was never packed, or when it was packed for another width and is in the
                //       focus; a line outside the focus keeps its stale packing until the view comes near.
                // Why: A line is shaped and packed once per snapshot, scale, and width, and a new width must not cost
                //      the window thread one packing for every line the view ever passed.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // if (!placed.has(line) || (placed.get(line).wrap !== wrap && focus.has(line))) placed.set(line, pack(...));
                // ```
                let repack = match self.placed.get(&line) {
                    None => true,
                    Some(known) => known.wrap != wrap && focus.contains(&line),
                };
                if repack {
                    // A hint stands where its character is drawn, so the line is broken as the source view breaks it.
                    let row = match wrap {
                        Some(width) => shaper.wrapped_row(document, line, scale, width),
                        None => shaper.row(document, line, scale),
                    };
                    self.work.packed += 1;
                    let (rows, packed) = pack(&row, labels, shaper, scale, wrap);
                    self.placed.insert(
                        line,
                        Placed {
                            wrap,
                            rows,
                            hints: packed,
                        },
                    );
                }
                if let Some(known) = self.placed.get(&line) {
                    // `clone` copies the packed rows so the block owns them independently of the cache.
                    (hint_rows, hints) = (known.rows, known.hints.clone());
                }
            }
            let mut messages = Vec::new();
            if diagnosed && let Some(rows) = self.messages.get(&line) {
                messages = match wrap {
                    Some(limit) => {
                        let refit = match self.wrapped.get(&line) {
                            None => true,
                            Some(known) => known.wrap != wrap && focus.contains(&line),
                        };
                        if refit {
                            // The natural widths are measured once per line and kept for later refits.
                            let widths = match self.natural.entry(line) {
                                Entry::Occupied(known) => known.into_mut(),
                                Entry::Vacant(slot) => {
                                    self.work.measured += rows.len() as u64;
                                    let mut measured: Vec<f32> = Vec::new();
                                    for row in rows.iter() {
                                        measured.push(
                                            shaper.row_layout(&row.text, scale).full_width()
                                                / scale,
                                        );
                                    }
                                    slot.insert(measured)
                                }
                            };
                            let (fitted, broken) = fit_messages(rows, widths, shaper, scale, limit);
                            self.work.broken += broken;
                            self.wrapped.insert(line, Fitted { wrap, rows: fitted });
                        }
                        match self.wrapped.get(&line) {
                            Some(known) => known.rows.clone(),
                            None => rows.clone(),
                        }
                    }
                    None => rows.clone(),
                };
            }
            let block = Block {
                line,
                hint_rows,
                hints,
                messages,
                held,
            };
            if block.height() > 0.0 {
                // `Arc::new` makes the block shareable between the window state and its frames.
                blocks.push(Arc::new(block));
            }
        }
        return blocks;
    }
}
