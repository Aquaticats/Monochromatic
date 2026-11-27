//! Assembling the blocks the vertical mapping and the frame read:
//!  one block per annotated line,
//!  its hint rows packed against the shaped line,
//!  its message rows fitted to the text area,
//!  and the space held for it.

/// The store these methods extend,
///  and the packings it keeps.
use super::super::{
    Annotations,
    Fitted,
    Placed,
};
/// The message rows of a line,
///  fitted by the part beside this one.
use super::message_rows::fit_messages;
/// Hint rows are packed against the shaped code line.
use crate::annotation_layout::pack;
/// The displayed text and its line starts.
use crate::document::Document;
/// The stamp of the text a block is assembled for.
use crate::language::identity::DocumentStamp;
/// The shaper that paints the source measures hint labels and their anchor positions.
use crate::shaped_text::TextShaper;
/// The assembled record of one line's rows.
use crate::virtual_row::Block;
/// Blocks are handed out in line order.
use std::{
    collections::{
        BTreeMap,
        btree_map::Entry,
    },
    ops::Range,
    sync::Arc,
};

/// Assemble the blocks of every annotated line.
impl Annotations {
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
                            self.wrapped.insert(
                                line,
                                Fitted {
                                    wrap,
                                    rows: fitted,
                                },
                            );
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
