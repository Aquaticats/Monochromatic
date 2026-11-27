//! What:
//!      The parser's container stack:
//!  the nesting bound,
//!  the grammar phase one open frame expects next,
//!  one `Frame`
//!           per open array or record,
//!  and the three steps that hang trivia on a frame's last child or
//!           turn a finished frame into one node.
//! Why:
//!       A growable work stack replaces recursive calls while keeping source order,
//!  so a valid
//!           512-container document parses without touching the call stack.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module parseFrames: MAX_CONTAINERS, type Phase, type Frame, prepend, attachLast, finish.
//! ```

/// What:
///  Import comment attachment for the trivia a frame collects.
/// Why:
///  Each key or value owns one normalized comment,
///  not a flat token list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { attach } from './comment';
/// ```
use crate::comment_merge::attach;
/// What:
///      Import the document model types a frame collects and builds.
/// Why:
///       Completed frames move into their parents,
///  so the frame names the same types callers hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncComment, JsoncEntry, JsoncKey, JsoncKind, JsoncValue } from './value';
/// ```
use crate::value::{
    JsoncComment,
    JsoncEntry,
    JsoncKey,
    JsoncKind,
    JsoncValue,
};

/// What:
///  Maximum number of simultaneously open JSONC containers.
/// Why:
///  Erroring before a 513th opener bounds the explicit parser frame list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_CONTAINERS = 512;
/// ```
pub(crate) const MAX_CONTAINERS: usize = 512;

/// What:
///  The grammar phase of one open array or record frame.
/// Why:
///  Separator enforcement must not be inferred from a comment or cursor alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Phase = 'item-or-close' | 'awaiting-child' | 'separator-or-close';
/// ```
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// An item or close delimiter may follow.
    ItemOrClose,
    /// A nested child has started;
    ///  the frame waits for its completed node.
    AwaitingChild,
    /// An item completed without a comma;
    ///  a separator or close must follow.
    SeparatorOrClose,
}

/// What:
///  One open array or record,
///  including pending key and leading comments.
/// Why:
///  A growable work stack replaces recursive calls while retaining source order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Frame = { kind: 'array'; elements: JsoncValue[] } | { kind: 'record'; entries: JsoncEntry[]; key?: JsoncKey };
/// ```
pub(crate) enum Frame {
    /// Open array with completed elements and comments preceding its opener.
    ///
    /// What:
    ///      holds the elements parsed so far,
    ///  the comments seen before the opener,
    ///  and the
    ///           grammar phase.
    /// Why:
    ///       an array frame must know whether a value,
    ///  a comma or the closer may come next.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'array', elements: JsoncValue[], leading: JsoncComment[], phase: Phase }
    /// ```
    Array {
        /// What:
        ///     Elements completed so far,
        ///  in source order.
        /// Why:
        ///      `elements` stores finished children so the closer can build one value at the end.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// elements: JsoncValue[];
        /// ```
        elements: Vec<JsoncValue>,
        /// What:
        ///     Comments seen before this array's opening bracket.
        /// Why:
        ///      `leading` stores them until the array value exists,
        ///  so they can be attached to it.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// leading: JsoncComment[];
        /// ```
        leading: Vec<JsoncComment>,
        /// What:
        ///     Grammar phase this frame expects next.
        /// Why:
        ///      `phase` stores the expectation,
        ///  so a separator cannot be inferred from a comment
        ///          or cursor position alone.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// phase: Phase;
        /// ```
        phase: Phase,
    },
    /// Open record with ordered entries and at most one key awaiting its value.
    ///
    /// What:
    ///      holds the members parsed so far,
    ///  the comments before the opener,
    ///  a key awaiting its
    ///           value,
    ///  and the grammar phase.
    /// Why:
    ///       a member's key and value carry separate comments,
    ///  so the frame keeps them apart until
    ///           the value is complete.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'record', entries: JsoncEntry[], leading: JsoncComment[], key?: JsoncKey, phase: Phase }
    /// ```
    Record {
        /// What:
        ///     Members completed so far,
        ///  in source order.
        /// Why:
        ///      `entries` stores finished pairs so duplicate keys stay visible rather than
        ///          collapsing into a map.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// entries: JsoncEntry[];
        /// ```
        entries: Vec<JsoncEntry>,
        /// What:
        ///     Comments seen before this record's opening brace.
        /// Why:
        ///      `leading` stores them until the record value exists,
        ///  so they attach to the record.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// leading: JsoncComment[];
        /// ```
        leading: Vec<JsoncComment>,
        /// What:
        ///     The key whose value has not been parsed yet,
        ///  if any.
        /// Why:
        ///      `pending_key` stores it so the value can be paired with its own key comment.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// key?: JsoncKey;
        /// ```
        pending_key: Option<JsoncKey>,
        /// What:
        ///     Grammar phase this frame expects next.
        /// Why:
        ///      `phase` stores the expectation,
        ///  so a value cannot be accepted where a comma belongs.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// phase: Phase;
        /// ```
        phase: Phase,
    },
}

/// What:
///      Read and update one frame's grammar phase and closer.
/// Why:
///       Both frame variants carry a phase,
///  so the driver asks the frame instead of matching on its
///           shape at every step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Frame { get phase(): Phase; set phase(next: Phase); closeByte(): number }
/// ```
impl Frame {
    /// What:
    ///  Read the frame's current grammar phase.
    /// Why:
    ///  The driver can decide whether a value or a separator is legal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function phase(frame: Frame): Phase;
    /// ```
    pub(crate) fn phase(&self) -> Phase {
        // What: `match` extracts one of the two structural variants.
        // Why: Both variants store a phase but no caller should inspect their fields directly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return frame.phase;
        // ```
        return match self {
            Frame::Array {
                phase,
                ..
            }
            | Frame::Record {
                phase,
                ..
            } => *phase,
        };
    }

    /// What:
    ///  Change the frame's expected next grammar event.
    /// Why:
    ///  Accepting a child or comma must advance exactly one continuation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// frame.phase = next;
    /// ```
    pub(crate) fn set_phase(
        &mut self,
        next: Phase,
    ) {
        match self {
            Frame::Array {
                phase,
                ..
            }
            | Frame::Record {
                phase,
                ..
            } => *phase = next,
        }
    }

    /// What:
    ///  Select the matching close delimiter for this exact frame.
    /// Why:
    ///  A `]` may not close an object or search for an older array.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// return frame.kind === 'array' ? ']'.charCodeAt(0) : '}'.charCodeAt(0);
    /// ```
    pub(crate) fn close_byte(&self) -> u8 {
        return if let Frame::Array {
            ..
        } = self
        {
            b']'
        } else {
            b'}'
        };
    }
}

/// What:
///  Put newly encountered leading comments before comments already on a node.
/// Why:
///  `/*outer*/[/*inner*/]` must retain source order even for empty containers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepend(node: JsoncValue, leading: JsoncComment[]): JsoncValue;
/// ```
fn prepend(
    node: &mut JsoncValue,
    leading: Vec<JsoncComment>,
) {
    // What: `take()` moves the existing optional comment out and leaves absence behind.
    // Why: JsoncComment merging can take ownership without duplicating a potentially long body.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const inside = node.comment; node.comment = undefined;
    // ```
    let inside = node.comment.take();
    let prefixed = attach(None, leading);
    node.comment = if let Some(existing) = inside {
        attach(prefixed, vec![existing])
    } else {
        prefixed
    };
}

/// What:
///  Attach trivia just before a comma or close to the last child value.
/// Why:
///  Interstitial comments survive even when a comma begins a later line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function attachLast(frame: Frame, comments: JsoncComment[]): void;
/// ```
pub(crate) fn attach_last(
    frame: &mut Frame,
    comments: Vec<JsoncComment>,
) {
    if comments.is_empty() {
        return;
    }
    if let Frame::Array {
        elements,
        ..
    } = frame
    {
        let Some(last) = elements.last_mut() else {
            unreachable!("separator requires array element");
        };
        last.comment = attach(last.comment.take(), comments);
        return;
    }
    if let Frame::Record {
        entries,
        ..
    } = frame
    {
        let Some(last) = entries.last_mut() else {
            unreachable!("separator requires record member");
        };
        last.value.comment = attach(last.value.comment.take(), comments);
    }
}

/// What:
///  Finish a matched container and move its completed values into one node.
/// Why:
///  An emitted node is delivered exactly once to its immediate parent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(frame: Frame, beforeClose: JsoncComment[]): JsoncValue;
/// ```
pub(crate) fn finish(
    frame: Frame,
    before_close: Vec<JsoncComment>,
) -> JsoncValue {
    // What: `match` consumes one frame variant and moves its owned children.
    // Why: Closure never re-parses or clones nested values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (frame.kind === 'array') return { kind: 'array', elements: frame.elements };
    // ```
    let (mut node, leading) = match frame {
        Frame::Array {
            mut elements,
            leading,
            ..
        } => {
            let own = if let Some(last) = elements.last_mut() {
                last.comment = attach(last.comment.take(), before_close);
                None
            } else {
                attach(None, before_close)
            };
            (
                JsoncValue {
                    kind: JsoncKind::Array {
                        elements,
                    },
                    comment: own,
                },
                leading,
            )
        }
        Frame::Record {
            mut entries,
            leading,
            pending_key,
            ..
        } => {
            if pending_key.is_some() {
                unreachable!("closing record with pending key");
            }
            let own = if let Some(last) = entries.last_mut() {
                last.value.comment = attach(last.value.comment.take(), before_close);
                None
            } else {
                attach(None, before_close)
            };
            (
                JsoncValue {
                    kind: JsoncKind::Record {
                        entries,
                    },
                    comment: own,
                },
                leading,
            )
        }
    };
    prepend(&mut node, leading);
    return node;
}
