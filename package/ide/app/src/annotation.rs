//! Inlay hints and diagnostics of the displayed file,
//!  reduced to what one frame paints.
//!
//! Snapshots come from the Language module and name the file generation and revision they describe.
//! A snapshot for any other generation or revision is never painted.
//!  Hints and diagnostic messages are drawn on
//! virtual rows above the code row of their line,
//!  and underlines under the glyphs a diagnostic marks.
//!  None of it
//! is source text:
//!  caret movement,
//!  selection,
//!  hit testing inside a code row,
//!  find rectangles,
//!  tab stops,
//!  and
//! copying never see them.
//!  What they do change is where code rows are;
//!  the store's blocks feed the one vertical
//! mapping in `crate::row_map`.

/// The diagnostic records and their severities as the Language module reports them.
use crate::language::diagnostics::{DiagnosticsSnapshot, Severity};
/// The hint records as the Language module reports them.
use crate::language::hints::HintsSnapshot;
/// File generation and content revision of the text a snapshot describes.
use crate::language::identity::DocumentStamp;
/// The rows a line shows above its code row.
use crate::virtual_row::{Block, HintPlace, MessageRow, message_rows};
/// What:
///  `Rope` is Helix's character-indexed text buffer.
/// Why:
///  Hints and diagnostics are grouped by the line their position lies on in the displayed text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { type Rope } from 'helix-core';
/// ```
use helix_core::Rope;
/// What:
///  `BTreeMap` is a map kept in key order (sibling:
///  unordered `HashMap`);
///  `Arc` shares one immutable
///       allocation between owners (siblings:
///  single-thread `Rc`,
///  owning `Box`);
///  `Instant` is a point on a
///       clock that never goes backwards.
/// Why:
///  Blocks are needed in line order;
///  the Language module hands snapshots out as shared values;
///  held
///      space ends at a point in time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = Readonly<T>; // and a Map whose keys are iterated in ascending order
/// ```
use std::{collections::BTreeMap, ops::Range, sync::Arc, time::Instant};

/// Blocks of virtual rows per line:
///  hint accumulation,
///  held space after a reload,
///  and assembly.
mod blocks;

/// The lines of one text revision that stored answers for hints asked about.
mod coverage;
/// The record of asked lines that the store keeps next to its hint labels.
use coverage::Coverage;

/// Severity vocabulary:
///  rank,
///  name,
///  default,
///  and the text of one problem.
mod severity;
/// The default for an omitted severity,
///  used when the diagnostics are indexed.
use severity::shown;
/// What:
///  `pub use` re-exports the severity helpers under this module's name.
/// Why:
///  Callers written against `annotation` keep their imports while the vocabulary lives in its own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { describe, rank, severityName } from './severity';
/// ```
pub use severity::{describe, rank, severity_name};

/// What:
///  One diagnostic as a frame underlines it;
///  `usize` is the character index type Helix ropes use
///       (siblings `u32`,
///  `u64`).
///  `Copy` lets the record be duplicated like a number.
/// Why:
///  Underlining needs the range and severity only;
///  comparing these decides whether pixels change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mark = { start: number; end: number; severity: Severity };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mark {
    /// First marked character.
    pub start: usize,
    /// Character after the marked range;
    ///  equal to `start` for a point.
    pub end: usize,
    /// Severity,
    ///  with an omitted severity shown as a warning,
    ///  as Helix shows it.
    pub severity: Severity,
}

/// What:
///  One diagnostic with the text its message rows show;
///  `Option<String>` is text or nothing.
/// Why:
///  The rows need the source,
///  code,
///  and message,
///  which underlining does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Problem = { mark: Mark; source: string; code?: string; message: string };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// Range and severity.
    pub mark: Mark,
    /// The `source` the server named,
    ///  for example `rustc`;
    ///  empty when it named none.
    pub source: String,
    /// The server's rule or error code.
    pub code: Option<String>,
    /// The server's message,
    ///  possibly several lines.
    pub message: String,
}

/// One hint label:
///  the character it annotates and its trimmed text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Label {
    /// Character offset the server placed the hint before.
    pub position: usize,
    /// The server's label without its padding spaces.
    pub text: String,
}

/// What:
///  Everything one frame paints from the snapshots;
///  `Vec<T>` is a growable list
///       (siblings:
///  fixed `[T; N]`,
///  borrowed `&[T]`).
///  `Default` builds the empty value.
/// Why:
///  Two frames with equal visible annotations have equal annotation pixels,
///  so this is the frame-stamp key.
///      Blocks are shared (`Arc`),
///  so taking the visible part copies pointers,
///  not labels and messages.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Visible = { blocks: Block[]; marks: Mark[] };
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Visible {
    /// Blocks of virtual rows of the materialized lines,
    ///  in line order.
    pub blocks: Vec<Arc<Block>>,
    /// Diagnostics touching the materialized rows,
    ///  in start order.
    pub marks: Vec<Mark>,
}

/// What:
///  Space held open above one line after an external change:
///  the line and the heights of its former
///       hint rows and message rows in logical pixels;
///  `f32` is a 32-bit float (sibling `f64`).
/// Why:
///  Hints and messages of the old text are never painted for the new one,
///  but their space stays until
///      annotations of the new text arrive,
///  so the text does not jump when they vanish and again when they return.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Held = { line: number; hints: number; messages: number };
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Held {
    /// Zero-based line of the new text.
    pub line: usize,
    /// Height held for hint rows.
    pub hints: f32,
    /// Height held for message rows.
    pub messages: f32,
}

/// What:
///  The hint rows of one line packed at one wrap width.
/// Why:
///  A new width makes the packing of a line stale;
/// the lines near the view are packed again at once,
///  and the others keep this packing until the view comes near them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placed = { wrap?: number; rows: number; hints: HintPlace[] };
/// ```
#[derive(Clone, Debug)]
struct Placed {
    /// The wrap width in logical pixels the line was packed at,
    /// or nothing while the view had none.
    wrap: Option<f32>,
    /// Number of hint rows.
    rows: usize,
    /// The hints with their rows and x.
    hints: Vec<HintPlace>,
}

/// The message rows of one line broken to one wrap width.
#[derive(Clone, Debug)]
struct Fitted {
    /// The wrap width in logical pixels the rows were broken at.
    wrap: Option<f32>,
    /// The rows.
    rows: Vec<MessageRow>,
}

/// What:
///  How much shaping assembling blocks has done so far:
///  lines whose hints were packed,
///  message rows whose width was measured,
///  and message rows that had to be broken to a width.
/// Why:
///  Packing and breaking run on the window thread when the text area changes width,
///  and the structural tests compare these counts before and after to prove what a resize costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AssemblyWork = { packed: number; measured: number; broken: number };
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssemblyWork {
    /// Lines whose hints were packed against the shaped line.
    pub packed: u64,
    /// Message rows shaped to learn their width.
    pub measured: u64,
    /// Message rows wider than the text area,
    /// broken by shaping.
    pub broken: u64,
}

/// What:
///  The latest snapshots,
///  a position index over every diagnostic,
///  the hints and message rows grouped by
///       line,
///  and the space held after a reload;
///  `Option<Arc<T>>` is a shared snapshot or nothing.
/// Why:
///  Groups arrive per source and hints per visible range,
///  so both are organized once per snapshot instead
///      of on every frame.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Annotations = { hints?: HintsSnapshot; diagnostics?: DiagnosticsSnapshot; problems: Problem[];
///   reach: number[]; labels: Map<number, Label[]>; messages: Map<number, MessageRow[]>; held: Held[]; version: number };
/// ```
#[derive(Debug, Default)]
pub struct Annotations {
    /// Latest hints,
    ///  painted only while their stamp names the displayed text.
    hints: Option<Arc<HintsSnapshot>>,
    /// Latest diagnostics,
    ///  painted only while their stamp names the displayed text.
    diagnostics: Option<Arc<DiagnosticsSnapshot>>,
    /// Every diagnostic of every group,
    ///  ordered by start,
    ///  then end.
    problems: Vec<Problem>,
    /// `reach[i]` is the largest end among `problems[0..=i]`,
    ///  so a range starting above a window is found by search.
    reach: Vec<usize>,
    /// The text revision `labels` describes.
    labelled: Option<DocumentStamp>,
    /// Hint labels per line,
    ///  in position order,
    ///  from every hint snapshot of the `labelled` revision.
    labels: BTreeMap<usize, Vec<Label>>,
    /// The lines the hint snapshots of the `labelled` revision asked about.
    answered: Coverage,
    /// Packed hint rows per line,
    ///  computed on demand at `placed_scale`,
    /// each at the wrap width it records.
    placed: BTreeMap<usize, Placed>,
    /// Physical pixels per logical pixel the entries of `placed` were packed at.
    placed_scale: f32,
    /// Message rows per line for the held diagnostics,
    ///  worst first,
    ///  broken at 80 columns.
    messages: BTreeMap<usize, Vec<MessageRow>>,
    /// Shaped width of each row of `messages` per line at `placed_scale`,
    ///  measured once so a new wrap width shapes only the rows that are wider than it.
    natural: BTreeMap<usize, Vec<f32>>,
    /// The rows of `messages` broken again to fit a wrap width,
    ///  per line,
    ///  made on demand.
    wrapped: BTreeMap<usize, Fitted>,
    /// Shaping done for blocks so far.
    work: AssemblyWork,
    /// The text revision `held` belongs to.
    held_for: Option<DocumentStamp>,
    /// Space held open after an external change,
    ///  in line order.
    held: Vec<Held>,
    /// When the held space is given up.
    held_until: Option<Instant>,
    /// What:
    ///  A counter that grows with every change that can alter a block;
    ///  `u64` is an unsigned 64-bit
    ///       integer (sibling `u32`),
    ///  wide enough never to wrap.
    /// Why:
    ///  The window rebuilds its vertical mapping only when this number changed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// version: number;
    /// ```
    version: u64,
}

/// What:
///  The sort key that orders problems by start,
///  then end.
/// Why:
///  Window and caret searches binary-search by start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const positionOrder = (problem: Problem): [number, number] => [problem.mark.start, problem.mark.end];
/// ```
fn position_order(problem: &Problem) -> (usize, usize) {
    return (problem.mark.start, problem.mark.end);
}

/// What:
///  The sort key that puts the worst problem first.
///  `&&Problem` is what a sort over a list of lent problems hands its key function.
/// Why:
///  The stable sort keeps position order among equal severities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const severityOrder = (problem: Problem): number => rank(problem.mark.severity);
/// ```
fn severity_order(problem: &&Problem) -> u8 {
    return rank(problem.mark.severity);
}

/// What:
///  The index of the first entry of `reach` that is not below `threshold`.
///  `reach` never decreases,
///  so a halving search finds it.
/// Why:
///  Window and caret searches skip every diagnostic that ends before the window without reading each one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function firstReaching(reach: number[], threshold: number): number;
/// ```
fn first_reaching(reach: &[usize], threshold: usize) -> usize {
    let mut low = 0;
    let mut high = reach.len();
    // Each step halves the candidates between `low` and `high`.
    while low < high {
        let middle = low + (high - low) / 2;
        if reach[middle] < threshold {
            low = middle + 1;
        } else {
            high = middle;
        }
    }
    return low;
}

/// What:
///  Flatten every group of `diagnostics` into one list ordered by start,
///  then end,
///  with the furthest end so
///       far beside it;
///  `Option<&DiagnosticsSnapshot>` lends a snapshot or nothing;
///  the answer is a pair (tuple).
/// Why:
///  Groups arrive per source,
///  so their union is sorted once per snapshot instead of on every frame.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexed(diagnostics?: DiagnosticsSnapshot): [Problem[], number[]];
/// ```
fn indexed(diagnostics: Option<&DiagnosticsSnapshot>) -> (Vec<Problem>, Vec<usize>) {
    let mut problems = Vec::new();
    // What: `if let Some(snapshot) = diagnostics` runs only when a snapshot is present.
    // Why: Absent diagnostics leave the index empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (diagnostics) { for (const group of diagnostics.groups) ... }
    // ```
    if let Some(snapshot) = diagnostics {
        for group in &snapshot.groups {
            for item in &group.items {
                problems.push(Problem {
                    mark: Mark {
                        start: item.start,
                        // A range the server sent backwards still marks its characters.
                        end: item.end.max(item.start),
                        severity: shown(item.severity),
                    },
                    // `clone` copies the text so the index owns it independently of the snapshot.
                    source: group.source.clone(),
                    code: item.code.clone(),
                    message: item.message.clone(),
                });
            }
        }
    }
    // What: `sort_by_key` orders by the pair `(start, end)` that `position_order` builds.
    // Why: Window and caret searches binary-search by start.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // problems.sort((a, b) => a.mark.start - b.mark.start || a.mark.end - b.mark.end);
    // ```
    problems.sort_by_key(position_order);
    let mut reach = Vec::new();
    let mut furthest = 0;
    for problem in &problems {
        furthest = furthest.max(problem.mark.end);
        reach.push(furthest);
    }
    return (problems, reach);
}

/// Build,
///  query,
///  and replace the annotation inputs of the displayed file.
impl Annotations {
    /// What:
    ///  Drop everything held and store both snapshots as given;
    ///  one that describes `displayed` is grouped
    ///       by the lines of `text`.
    ///  `Option<Arc<...>>` parameters are moved in;
    ///  `&Rope` lends the displayed text.
    /// Why:
    ///  Tests and the inspection path install a complete set in one call.
    ///  A snapshot for other text is
    ///      kept for identity checks but never grouped or painted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// replace(displayed: DocumentStamp, text: Rope, hints?: HintsSnapshot, diagnostics?: DiagnosticsSnapshot): void;
    /// ```
    pub fn replace(
        &mut self,
        displayed: DocumentStamp,
        text: &Rope,
        hints: Option<Arc<HintsSnapshot>>,
        diagnostics: Option<Arc<DiagnosticsSnapshot>>,
    ) {
        let version = self.version;
        // `Self::default()` is the empty store; assigning through `*self` replaces every field at once.
        *self = Self::default();
        self.version = version + 1;
        if let Some(snapshot) = hints {
            // A refused snapshot is still held, so handing the same one back is recognized.
            if !self.accept_hints(displayed, text, Arc::clone(&snapshot)) {
                self.hints = Some(snapshot);
            }
        }
        if let Some(snapshot) = diagnostics
            && !self.accept_diagnostics(displayed, text, Arc::clone(&snapshot))
        {
            self.diagnostics = Some(snapshot);
        }
    }

    /// What:
    ///  Store hints that describe `displayed` and group them by the lines of `text`;
    ///  a snapshot for any
    ///       other text is refused,
    ///  the held hints stay,
    ///  and the answer is `false`.
    ///  `Arc<HintsSnapshot>` is moved in.
    /// Why:
    ///  This is the Language poll's entry point:
    ///  hints and diagnostics arrive independently,
    ///  and only a
    ///      snapshot of the displayed file generation and revision may replace what is shown.
    ///  Hints are asked
    ///      for around the visible lines only,
    ///  so a snapshot replaces the hints of its own line range and keeps
    ///      those of lines it did not ask about:
    ///  rows seen once stay where they are when the view returns.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// acceptHints(displayed: DocumentStamp, text: Rope, snapshot: HintsSnapshot): boolean;
    /// ```
    pub fn accept_hints(
        &mut self,
        displayed: DocumentStamp,
        text: &Rope,
        snapshot: Arc<HintsSnapshot>,
    ) -> bool {
        if snapshot.stamp != displayed {
            return false;
        }
        // `&snapshot` lends the snapshot behind the shared pointer to the grouping pass.
        self.merge_labels(text, &snapshot);
        // `Some(...)` stores the snapshot as the present value.
        self.hints = Some(snapshot);
        self.version += 1;
        return true;
    }

    /// What:
    ///  Store diagnostics that describe `displayed`,
    ///  index them,
    ///  and build the message rows of every
    ///       line of `text` where one starts;
    ///  a snapshot for any other text is refused,
    ///  the held diagnostics
    ///       stay,
    ///  and the answer is `false`.
    /// Why:
    ///  The index and the rows are rebuilt once per accepted snapshot,
    ///  never per frame.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// acceptDiagnostics(displayed: DocumentStamp, text: Rope, snapshot: DiagnosticsSnapshot): boolean;
    /// ```
    pub fn accept_diagnostics(
        &mut self,
        displayed: DocumentStamp,
        text: &Rope,
        snapshot: Arc<DiagnosticsSnapshot>,
    ) -> bool {
        if snapshot.stamp != displayed {
            return false;
        }
        let (problems, reach) = indexed(Some(&snapshot));
        self.problems = problems;
        self.reach = reach;
        self.diagnostics = Some(snapshot);
        self.group_messages(text);
        self.version += 1;
        return true;
    }

    /// What:
    ///  Rebuild `messages`:
    ///  the diagnostics starting on each line,
    ///  worst first,
    ///  as rows.
    /// Why:
    ///  The rows at the first break (80 columns) are known for every line of the file without shaping,
    ///  so the
    ///      vertical mapping is complete as soon as diagnostics are;
    ///  a row wider than the text area is broken again when the blocks are assembled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// groupMessages(text: Rope): void;
    /// ```
    fn group_messages(&mut self, text: &Rope) {
        let mut grouped: BTreeMap<usize, Vec<&Problem>> = BTreeMap::new();
        let length = text.len_chars();
        for problem in &self.problems {
            let line = text.char_to_line(problem.mark.start.min(length));
            // `entry(...).or_default()` lends the line's list, creating an empty one first when absent.
            grouped.entry(line).or_default().push(problem);
        }
        let mut messages = BTreeMap::new();
        for (line, mut problems) in grouped {
            // `sort_by_key` is stable, so equal severities keep their position order.
            problems.sort_by_key(severity_order);
            messages.insert(line, message_rows(&problems));
        }
        self.messages = messages;
        // The rows measured and broken for the earlier messages describe lines that may now hold other rows.
        self.natural.clear();
        self.wrapped.clear();
    }

    /// What:
    ///  The held hints while they describe `displayed`,
    ///  otherwise nothing;
    ///  `Option<&HintsSnapshot>` lends
    ///       the snapshot without copying it.
    /// Why:
    ///  Readers other than the renderer,
    ///  such as logging,
    ///  ask with the stamp of the displayed text.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// hints(displayed: DocumentStamp): HintsSnapshot | undefined;
    /// ```
    pub fn hints(&self, displayed: DocumentStamp) -> Option<&HintsSnapshot> {
        // The trailing `?` answers nothing when no snapshot is held.
        let held = self.hints.as_deref()?;
        // The snapshot is lent only while it describes the displayed text.
        if held.stamp == displayed {
            return Some(held);
        }
        return None;
    }

    /// The held diagnostics while they describe `displayed`,
    ///  otherwise nothing.
    pub fn diagnostics(&self, displayed: DocumentStamp) -> Option<&DiagnosticsSnapshot> {
        let held = self.diagnostics.as_deref()?;
        if held.stamp == displayed {
            return Some(held);
        }
        return None;
    }

    /// What:
    ///  Whether these exact snapshots are already held;
    ///  `Arc::ptr_eq` compares identities,
    ///  not contents.
    /// Why:
    ///  A poll that hands back the same snapshots must not rebuild the index.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// holds(hints?: HintsSnapshot, diagnostics?: DiagnosticsSnapshot): boolean;
    /// ```
    pub fn holds(
        &self,
        hints: &Option<Arc<HintsSnapshot>>,
        diagnostics: &Option<Arc<DiagnosticsSnapshot>>,
    ) -> bool {
        // What: `match` on a pair of options: two present snapshots compare identities, two absent ones are
        //       equal, and `_` (any other combination) is a difference.
        // Why: Pointer identity is exact and costs nothing, whatever the snapshot size.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const sameHints = this.hints === hints;
        // ```
        let same_hints = match (&self.hints, hints) {
            (Some(held), Some(given)) => Arc::ptr_eq(held, given),
            (None, None) => true,
            _ => false,
        };
        let same_diagnostics = match (&self.diagnostics, diagnostics) {
            (Some(held), Some(given)) => Arc::ptr_eq(held, given),
            (None, None) => true,
            _ => false,
        };
        return same_hints && same_diagnostics;
    }

    /// What:
    ///  Whether stored hint snapshots of the text revision `displayed` asked about every line in `lines`
    ///  (the line before the last is the last one asked about);
    ///  the answer is `false` for an empty range.
    ///  `Range<usize>` is a half-open range of line numbers.
    /// Why:
    ///  The language poll skips a hint request for lines whose hints this store already holds.
    ///  A request that is pending,
    ///  failed,
    ///  or was cancelled never reaches the store,
    ///  and a snapshot for another revision is refused,
    ///  so neither counts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// answeredCovers(displayed: DocumentStamp, lines: [number, number]): boolean;
    /// ```
    pub fn answered_covers(&self, displayed: DocumentStamp, lines: Range<usize>) -> bool {
        return self.answered.covers(displayed, lines);
    }

    /// What:
    ///  Forget which lines were asked about,
    ///  and keep the hints themselves.
    /// Why:
    ///  A language worker told to open the displayed file again holds no hint request of the old session,
    ///  so the next view must ask afresh.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// forgetAnswered(): void;
    /// ```
    pub fn forget_answered(&mut self) {
        self.answered.clear();
    }

    /// The shaping assembling blocks has done so far.
    pub fn work(&self) -> AssemblyWork {
        return self.work;
    }

    /// What:
    ///  Whether a line in `focus` holds hint rows or message rows made for a wrap width other than `wrap`.
    /// Why:
    ///  A new width is applied to the lines near the view at once and to the others when the view comes near;
    ///  the window asks this to know that the blocks must be assembled again after the view moved.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// staleIn(wrap: number | undefined, focus: Range): boolean;
    /// ```
    pub fn stale_in(&self, wrap: Option<f32>, focus: &Range<usize>) -> bool {
        // A line whose hint rows were made for another width makes the blocks stale.
        for (_, placed) in self.placed.range(focus.clone()) {
            if placed.wrap != wrap {
                return true;
            }
        }
        // A line whose message rows were made for another width does too.
        for (_, fitted) in self.wrapped.range(focus.clone()) {
            if fitted.wrap != wrap {
                return true;
            }
        }
        return false;
    }

    /// The change counter:
    ///  it differs whenever a block may have changed since the caller last looked.
    pub fn version(&self) -> u64 {
        return self.version;
    }

    /// Whether the held diagnostics describe exactly the displayed text.
    fn diagnostics_current(&self, stamp: DocumentStamp) -> bool {
        // `if let Some(...)` answers false for no snapshot and otherwise compares the stamps.
        if let Some(snapshot) = self.diagnostics.as_ref() {
            return snapshot.stamp == stamp;
        }
        return false;
    }

    /// What:
    ///  The diagnostics touching rows `first..last` of `text`,
    ///  or nothing for a stale snapshot.
    ///       `&Rope` lends the displayed text.
    /// Why:
    ///  Underlining is bounded to the materialized rows;
    ///  the search is binary,
    ///  so the cost follows the
    ///      number of visible diagnostics,
    ///  not the file.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// marks(stamp: DocumentStamp, text: Rope, first: number, last: number): Mark[];
    /// ```
    pub fn marks(&self, stamp: DocumentStamp, text: &Rope, first: usize, last: usize) -> Vec<Mark> {
        let mut result = Vec::new();
        let lines = text.len_lines();
        if first >= lines || !self.diagnostics_current(stamp) {
            return result;
        }
        let start = text.line_to_char(first);
        // Past the last line every remaining position, including the end of the text, is inside the window.
        let mut end = usize::MAX;
        if last < lines {
            end = text.line_to_char(last);
        }
        // The first diagnostic whose range, or an earlier one's, reaches the window.
        let from = first_reaching(&self.reach, start);
        for problem in &self.problems[from..] {
            if problem.mark.start >= end {
                break;
            }
            if problem.mark.end >= start {
                result.push(problem.mark);
            }
        }
        return result;
    }

    /// What:
    ///  Diagnostics whose range contains caret boundary `position`,
    ///  worst first;
    ///  `Vec<&Problem>` lends them.
    /// Why:
    ///  The source view's accessible description names every problem at the caret;
    ///  a range touches the
    ///      caret at both of its ends.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// at(stamp: DocumentStamp, position: number): Problem[];
    /// ```
    pub fn at(&self, stamp: DocumentStamp, position: usize) -> Vec<&Problem> {
        let mut result = Vec::new();
        if !self.diagnostics_current(stamp) {
            return result;
        }
        let from = first_reaching(&self.reach, position);
        for problem in &self.problems[from..] {
            if problem.mark.start > position {
                break;
            }
            if problem.mark.end >= position {
                result.push(problem);
            }
        }
        // The worst problem is read first; the stable sort keeps position order among equals.
        result.sort_by_key(severity_order);
        return result;
    }
}
