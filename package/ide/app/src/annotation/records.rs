//! The records a frame paints from and the store keeps:
//!  one diagnostic as it is underlined,
//!  one with the text of its message rows,
//!  one hint label,
//!  everything one frame paints,
//!  space held after a reload,
//!  the packing and breaking of one line,
//!  and the shaping one assembly did.

/// The diagnostics and their severities as the Language module reports them.
use crate::language::diagnostics::Severity;
/// The rows a line shows above its code row.
use crate::virtual_row::{
    Block,
    HintPlace,
    MessageRow,
};
/// What:
///  `Arc` shares one immutable allocation between owners (siblings:
///  single-thread `Rc`,
///  owning `Box`).
/// Why:
///  A visible frame holds the same blocks the window holds,
/// so taking one copies pointers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Shared<T> = Readonly<T>;
/// ```
use std::sync::Arc;

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
pub struct Placed {
    /// The wrap width in logical pixels the line was packed at,
    /// or nothing while the view had none.
    pub(super) wrap: Option<f32>,
    /// Number of hint rows.
    pub(super) rows: usize,
    /// The hints with their rows and x.
    pub(super) hints: Vec<HintPlace>,
}

/// The message rows of one line broken to one wrap width.
#[derive(Clone, Debug)]
pub struct Fitted {
    /// The wrap width in logical pixels the rows were broken at.
    pub(super) wrap: Option<f32>,
    /// The rows.
    pub(super) rows: Vec<MessageRow>,
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
