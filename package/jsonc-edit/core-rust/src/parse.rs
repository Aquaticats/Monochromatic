//! What:
//!  Iterative JSONC structural parsing with explicit container frames.
//! Why:
//!  Native Rust test threads overflowed their stacks before 512 nested containers returned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function parseJsonc(source: string): JsoncValue { const frames: Frame[] = []; /* loop over tokens */ }
//! ```

/// What:
///  Import comment attachment for ordered parser trivia.
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
///  Import the source-borrowing scanner and same-line trivia result.
/// Why:
///  Parser phases share one byte cursor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Scanner } from './scan';
/// ```
use crate::scan::Scanner;
/// What:
///  Import JSON value,
///  key,
///  member,
///  and parse-error types.
/// Why:
///  Completed frame values move into their parent without decoding again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncEntry, JsoncKey, JsoncValue, JsoncParseError, JsoncKind } from './value';
/// ```
use crate::error::JsoncParseError;
/// What:
///      Import the document model types the parser builds.
/// Why:
///       Completed frames move into their parents,
///  so the parser names the same types callers hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncComment, JsoncEntry, JsoncKey, JsoncKind, JsoncValue } from './value';
/// ```
use crate::value::{JsoncComment, JsoncEntry, JsoncKey, JsoncKind, JsoncValue};

/// What:
///  Maximum number of simultaneously open JSONC containers.
/// Why:
///  Erroring before a 513th opener bounds the explicit parser frame list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAX_CONTAINERS = 512;
/// ```
const MAX_CONTAINERS: usize = 512;

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
enum Phase {
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
enum Frame {
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
    fn phase(&self) -> Phase {
        // What: `match` extracts one of the two structural variants.
        // Why: Both variants store a phase but no caller should inspect their fields directly.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return frame.phase;
        // ```
        return match self {
            Frame::Array { phase, .. } | Frame::Record { phase, .. } => *phase,
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
    fn set_phase(&mut self, next: Phase) {
        match self {
            Frame::Array { phase, .. } | Frame::Record { phase, .. } => *phase = next,
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
    fn close_byte(&self) -> u8 {
        return if let Frame::Array { .. } = self { b']' } else { b'}' };
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
fn prepend(node: &mut JsoncValue, leading: Vec<JsoncComment>) {
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
fn attach_last(frame: &mut Frame, comments: Vec<JsoncComment>) {
    if comments.is_empty() {
        return;
    }
    if let Frame::Array { elements, .. } = frame {
        let Some(last) = elements.last_mut() else { unreachable!("separator requires array element"); };
        last.comment = attach(last.comment.take(), comments);
        return;
    }
    if let Frame::Record { entries, .. } = frame {
        let Some(last) = entries.last_mut() else { unreachable!("separator requires record member"); };
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
fn finish(frame: Frame, before_close: Vec<JsoncComment>) -> JsoncValue {
    // What: `match` consumes one frame variant and moves its owned children.
    // Why: Closure never re-parses or clones nested values.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (frame.kind === 'array') return { kind: 'array', elements: frame.elements };
    // ```
    let (mut node, leading) = match frame {
        Frame::Array { mut elements, leading, .. } => {
            let own = if let Some(last) = elements.last_mut() {
                last.comment = attach(last.comment.take(), before_close);
                None
            } else {
                attach(None, before_close)
            };
            (JsoncValue { kind: JsoncKind::Array { elements }, comment: own }, leading)
        }
        Frame::Record { mut entries, leading, pending_key, .. } => {
            if pending_key.is_some() {
                unreachable!("closing record with pending key");
            }
            let own = if let Some(last) = entries.last_mut() {
                last.value.comment = attach(last.value.comment.take(), before_close);
                None
            } else {
                attach(None, before_close)
            };
            (JsoncValue { kind: JsoncKind::Record { entries }, comment: own }, leading)
        }
    };
    prepend(&mut node, leading);
    return node;
}

/// What:
///  Begin a value,
///  either yielding a scalar or pushing one container frame.
/// Why:
///  Containers never call this function recursively,
///  so stack use stays constant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function beginValue(scanner: Scanner, leading: JsoncComment[], frames: Frame[]): JsoncValue | undefined;
/// ```
fn begin_value(
    scanner: &mut Scanner<'_>,
    leading: Vec<JsoncComment>,
    frames: &mut Vec<Frame>,
    delivered: &mut Option<JsoncValue>,
) -> Result<(), JsoncParseError> {
    if scanner.byte() == Some(b'{') || scanner.byte() == Some(b'[') {
        if frames.len() >= MAX_CONTAINERS {
            return Err(scanner.error("JSONC nesting too deep"));
        }
        if scanner.consume(b'{') {
            frames.push(Frame::Record { entries: Vec::new(), leading, pending_key: None, phase: Phase::ItemOrClose });
        } else {
            scanner.consume(b'[');
            frames.push(Frame::Array { elements: Vec::new(), leading, phase: Phase::ItemOrClose });
        }
        return Ok(());
    }
    let kind = if scanner.byte() == Some(b'"') {
        let (units, raw) = scanner.string()?;
        JsoncKind::Text { units, raw }
    } else {
        // What: An absent byte gets an invalid-number sentinel, not a false numeric match.
        // Why: EOF should produce the ordinary expected-value error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const first = sourceBytes[offset] ?? 0;
        // ```
        let first = scanner.byte().unwrap_or(0);
        if first == b'-' || first.is_ascii_digit() {
            let (raw, identity) = scanner.number()?;
            JsoncKind::Number { raw, identity }
        } else if scanner.source[scanner.offset..].starts_with("true") {
            scanner.offset += 4;
            JsoncKind::Boolean { value: true }
        } else if scanner.source[scanner.offset..].starts_with("false") {
            scanner.offset += 5;
            JsoncKind::Boolean { value: false }
        } else if scanner.source[scanner.offset..].starts_with("null") {
            scanner.offset += 4;
            JsoncKind::Null
        } else {
            return Err(scanner.error("expected JSONC value"));
        }
    };
    // What: `Some` hands the completed scalar to the parent in the next loop pass.
    // Why: A scalar needs no container frame or recursive call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // delivered = { kind, comment: merge(leading) };
    // ```
    *delivered = Some(JsoncValue { kind, comment: attach(None, leading) });
    return Ok(());
}

/// What:
///  Deliver one completed child and consume only its same-line trailing trivia.
/// Why:
///  A comma after a later newline belongs to the parent's separator phase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function receiveChild(parent: Frame, child: JsoncValue, scanner: Scanner): void;
/// ```
fn receive_child(scanner: &mut Scanner<'_>, frames: &mut [Frame], mut child: JsoncValue) -> Result<(), JsoncParseError> {
    let trailing = scanner.capture_trailing()?;
    child.comment = attach(child.comment, trailing.comments);
    let Some(parent) = frames.last_mut() else { unreachable!("child without parent"); };
    if parent.phase() != Phase::AwaitingChild {
        return Err(scanner.error("unexpected completed JSONC value"));
    }
    if let Frame::Array { elements, .. } = parent {
        elements.push(child);
    } else if let Frame::Record { entries, pending_key, .. } = parent {
        let Some(key) = pending_key.take() else { unreachable!("record value without key"); };
        entries.push(JsoncEntry { key, value: child });
    }
    parent.set_phase(if trailing.comma { Phase::ItemOrClose } else { Phase::SeparatorOrClose });
    return Ok(());
}

/// What:
///  Parse one complete JSONC document by driving a stack of container phases.
/// Why:
///  Valid 512-depth input must not exhaust the Rust call stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseJsonc(source: string): JsoncValue { while (frames.length) advance(); }
/// ```
///
/// # Errors
/// Malformed syntax or a 513th open container returns a byte-positioned parse error.
pub fn parse_jsonc(source: &str) -> Result<JsoncValue, JsoncParseError> {
    // What: This mutable scanner borrows source and advances only through its own methods.
    // Why: The resulting tree owns text it needs after parsing ends.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const scanner = { source, offset: 0 };
    // ```
    let mut scanner = Scanner::new(source);
    // What: `?` returns a scanner error early; a successful result owns ordered comments.
    // Why: Malformed leading trivia cannot silently disappear.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const leading = skipTrivia(scanner);
    // ```
    let leading = scanner.trivia()?;
    if scanner.byte() != Some(b'{') && scanner.byte() != Some(b'[') {
        return Err(scanner.error("JSONC document root must be an object or array"));
    }
    let mut frames: Vec<Frame> = Vec::new();
    let mut delivered: Option<JsoncValue> = None;
    begin_value(&mut scanner, leading, &mut frames, &mut delivered)?;
    // What: The loop breaks with one completed root; every pass consumes input or a pending event.
    // Why: No recursion or retry of the same byte/phase pair is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let root: JsoncValue; while (frames.length || delivered) { /* advance */ }
    // ```
    let mut root = loop {
        if let Some(child) = delivered.take() {
            if frames.is_empty() {
                break child;
            }
            receive_child(&mut scanner, &mut frames, child)?;
            continue;
        }
        let Some(phase) = frames.last().map(Frame::phase) else {
            return Err(scanner.error("JSONC root did not complete"));
        };
        if phase == Phase::AwaitingChild {
            return Err(scanner.error("JSONC child did not complete"));
        }
        let before = scanner.trivia()?;
        if phase == Phase::SeparatorOrClose {
            let Some(top) = frames.last_mut() else { unreachable!("separator without frame"); };
            attach_last(top, before);
            if scanner.consume(b',') {
                top.set_phase(Phase::ItemOrClose);
                continue;
            }
            if scanner.byte() == Some(top.close_byte()) {
                let closer = top.close_byte();
                scanner.consume(closer);
                let Some(done) = frames.pop() else { unreachable!("close without frame"); };
                delivered = Some(finish(done, Vec::new()));
                continue;
            }
            return Err(scanner.error("expected comma or container close"));
        }
        let Some(top) = frames.last() else { unreachable!("item without frame"); };
        if scanner.byte() == Some(top.close_byte()) {
            let closer = top.close_byte();
            scanner.consume(closer);
            let Some(done) = frames.pop() else { unreachable!("close without frame"); };
            delivered = Some(finish(done, before));
            continue;
        }
        if let Some(Frame::Array { .. }) = frames.last() {
            let Some(array_frame) = frames.last_mut() else { unreachable!("array frame disappeared"); };
            array_frame.set_phase(Phase::AwaitingChild);
            begin_value(&mut scanner, before, &mut frames, &mut delivered)?;
            continue;
        }
        if scanner.byte() != Some(b'"') {
            return Err(scanner.error("expected double-quoted object key"));
        }
        let (units, raw) = scanner.string()?;
        let before_colon = scanner.trivia()?;
        if !scanner.consume(b':') {
            return Err(scanner.error("expected colon after JSONC key"));
        }
        let before_value = scanner.trivia()?;
        let key = JsoncKey { units, raw, comment: attach(attach(None, before), before_colon) };
        let Some(Frame::Record { pending_key, phase: record_phase, .. }) = frames.last_mut() else {
            unreachable!("record frame disappeared");
        };
        *pending_key = Some(key);
        *record_phase = Phase::AwaitingChild;
        begin_value(&mut scanner, before_value, &mut frames, &mut delivered)?;
    };
    let trailing = scanner.trivia()?;
    if scanner.offset != source.len() {
        return Err(scanner.error("unexpected content after JSONC root"));
    }
    root.comment = attach(root.comment, trailing);
    return Ok(root);
}
