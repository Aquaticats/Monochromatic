//! Versioned read-only text and edit correspondence;
//!  no filesystem writes.

/// What:
///  Import Helix's text rope,
///  diff transaction,
///  its change set,
///  and position association.
/// Why:
///  Reuse the inspected edit model instead of matching selected strings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Rope, compareRopes, Assoc, type ChangeSet } from 'helix-core';
/// ```
use helix_core::{Assoc, ChangeSet, Rope, Transaction, diff::compare_ropes};

/// What:
///  A copyable reading-position record.
///  usize is an address-sized index,
/// unlike signed i32/i64 or fixed-width u32/u64.
/// Why:
///  Helix positions use character indices and its interfaces require usize.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadingPosition = { anchor: number; head: number; viewport: number; affinity: ReadingAffinity };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReadingPosition {
    /// Selection origin in Unicode scalar positions,
    ///  not UTF-8 bytes.
    pub anchor: usize,
    /// Moving end of the selection;
    ///  equality with anchor denotes a caret.
    pub head: usize,
    /// Source position attached to the viewport's top visible source line.
    pub viewport: usize,
    /// The side of a soft-wrap offset the head stands on;
    ///  it matters only when the head is at the offset where a visual row breaks,
    ///  and every move that does not choose a side sets `Downstream`.
    pub affinity: ReadingAffinity,
}

/// What:
///  The side of a soft-wrap offset that a reading position stands on:
///  `Upstream` is the end of the upper visual row,
///  `Downstream` the start of the lower one.
/// Why:
///  One offset is both the end of a row and the start of the next row,
///  so a caret there needs a side to be drawn on.
///  `Downstream` is the default because it matches a position with no wrap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadingAffinity = 'upstream' | 'downstream';
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReadingAffinity {
    /// The end of the upper visual row.
    Upstream,
    /// The start of the lower visual row.
    #[default]
    Downstream,
}

/// What:
///  A prepared replacement retains its base version and new owned text.
/// Why:
///  A worker can compute this without borrowing mutable UI state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reload = { base: number; previous: Rope; text: Rope; changes: Transaction };
/// ```
pub struct Reload {
    /// Version used when computing the correspondence.
    base: u64,
    /// Text of the base version;
    ///  a rope clone shares its chunks instead of copying characters.
    previous: Rope,
    /// Authoritative replacement text read from disk.
    text: Rope,
    /// Mapping from the displayed base revision to this replacement.
    changes: Transaction,
}

/// Read-only access lets background consumers classify the same replacement that will be installed.
impl Reload {
    /// Lend replacement text without applying it or exposing mutation.
    pub fn text(&self) -> &Rope {
        return &self.text;
    }

    /// Lend the base text the change set applies to,
    ///  so language servers can be told what was replaced.
    pub fn previous_text(&self) -> &Rope {
        return &self.previous;
    }

    /// Lend the edit list from the base text to the replacement,
    ///  in character offsets.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get changes(): Readonly<ChangeSet> { return this.transaction.changes; }
    /// ```
    pub fn changes(&self) -> &ChangeSet {
        return self.changes.changes();
    }

    /// Read the document revision this reload was computed from.
    pub fn base_revision(&self) -> u64 {
        return self.base;
    }
}

/// What:
///  A document owns its rope and current reading position.
/// Why:
///  Source,
///  copying,
///  and refresh share one state rather than UI buffers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Document { text: Rope; revision: number; position: ReadingPosition }
/// ```
#[derive(Clone)]
pub struct Document {
    /// Current immutable-by-convention source snapshot.
    text: Rope,
    /// Monotonic content revision;
    ///  u64 avoids tying lifetime count to pointer width.
    revision: u64,
    /// Current user interaction state,
    ///  including movement during background work.
    position: ReadingPosition,
}

/// What:
///  impl groups operations belonging to Document,
///  like class methods.
/// Why:
///  The native view cannot alter text except by applying an external reload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Document { /* methods */ }
/// ```
impl Document {
    /// Construct an owned snapshot from borrowed source text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(text: string) { this.text = Rope.from(text); }
    /// ```
    pub fn new(text: &str) -> Self {
        // What: Rope::from_str copies borrowed &str into owned rope storage.
        // Why: The document must outlive the caller's read buffer.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const rope = Rope.from(text);
        // ```
        let rope = Rope::from_str(text);
        // Return an owned record, with an empty caret at the beginning.
        return Self {
            text: rope,
            revision: 0,
            position: ReadingPosition::default(),
        };
    }

    /// Borrow current text without permitting source modification.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get text(): Readonly<Rope> { return this.text; }
    /// ```
    pub fn text(&self) -> &Rope {
        // The reference lends access without moving or copying the rope.
        return &self.text;
    }

    /// Read the content revision used to identify asynchronous results.
    pub fn revision(&self) -> u64 {
        return self.revision;
    }

    /// Read the current caret,
    ///  selection direction,
    ///  and viewport source anchor.
    pub fn position(&self) -> ReadingPosition {
        return self.position;
    }

    /// Set a reading position,
    ///  clamping external UI coordinates to valid text.
    pub fn select(&mut self, position: ReadingPosition) {
        // What: A typed local records the rope's valid character endpoint.
        // Why: UI hit tests can land beyond the final glyph.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const end: number = this.text.length;
        // ```
        let end: usize = self.text.len_chars();
        self.position = ReadingPosition {
            anchor: position.anchor.min(end),
            head: position.head.min(end),
            viewport: position.viewport.min(end),
            affinity: position.affinity,
        };
    }

    /// Copy only source characters,
    ///  excluding hints and line numbers.
    pub fn selected_text(&self) -> String {
        let start = self.position.anchor.min(self.position.head);
        let end = self.position.anchor.max(self.position.head);
        // What: slice lends a character range; to_string creates an owned String.
        // Why: Clipboard data must remain valid after the document reloads.
        // String owns bytes; &str would borrow text that can be replaced.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.text.slice(start, end);
        // ```
        return self.text.slice(start..end).to_string();
    }

    /// Compute a reload without capturing the current selection.
    pub fn prepare_reload(&self, source: &str) -> Reload {
        // Convert new borrowed source into independent owned storage.
        let text = Rope::from_str(source);
        // What: & lends each rope to the comparison without moving it.
        // Why: Both snapshots remain available for mapping and server sync.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const changes = compareRopes(this.text, text);
        // ```
        let changes = compare_ropes(&self.text, &text);
        return Reload {
            base: self.revision,
            // A rope clone copies a small handle; the document keeps its text until the reload is applied.
            previous: self.text.clone(),
            text,
            changes,
        };
    }

    /// Apply only a reload computed from the current displayed revision.
    ///
    /// Returns whether the revision was accepted,
    ///  not whether text differed.
    pub fn apply_reload(&mut self, reload: Reload) -> bool {
        if reload.base != self.revision {
            tracing::debug!(
                base = reload.base,
                current = self.revision,
                "discard stale document reload"
            );
            return false;
        }
        let changes = reload.changes.changes();
        // Map the latest reading state, not the state when comparison started.
        let position = self.position;
        // What: mut allows replacing the mapping record before installation.
        // Why: Carets and directed selections need different boundary affinities.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let mapped = { ...position };
        // ```
        let mut mapped = position;
        if position.anchor == position.head {
            mapped.head = changes.map_pos(position.head, Assoc::AfterSticky);
            mapped.anchor = mapped.head;
        } else if position.anchor < position.head {
            mapped.anchor = changes.map_pos(position.anchor, Assoc::Before);
            mapped.head = changes.map_pos(position.head, Assoc::After);
        } else {
            mapped.head = changes.map_pos(position.head, Assoc::Before);
            mapped.anchor = changes.map_pos(position.anchor, Assoc::After);
        }
        mapped.viewport = changes.map_pos(position.viewport, Assoc::BeforeSticky);
        // What: The side of a wrap offset survives only a head that stayed at its offset.
        // Why: A head that moved is at an offset the old side was not chosen for.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (mapped.head !== position.head) mapped.affinity = 'downstream';
        // ```
        if mapped.head != position.head {
            mapped.affinity = ReadingAffinity::Downstream;
        }
        self.text = reload.text;
        self.revision += 1;
        self.select(mapped);
        tracing::debug!(
            revision = self.revision,
            anchor = mapped.anchor,
            head = mapped.head,
            "applied external text revision"
        );
        return true;
    }
}
