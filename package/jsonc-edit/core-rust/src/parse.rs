//! What:
//!  Iterative JSONC structural parsing:
//!  the two document entry points and the two steps each loop pass
//!  takes.
//! Why:
//!  Native Rust test threads overflowed their stacks before 512 nested containers returned,
//!  so the driver
//!  holds the explicit stack in `parse_frames` and never calls itself per container.
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
///  Import the error type a malformed document is reported with.
/// Why:
///  Every step below reports a byte position instead of panicking on bad input.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncParseError } from './error';
/// ```
use crate::error::JsoncParseError;
/// What:
///  Re-export the nesting bound from the frame module.
/// Why:
///  The bound is a property of the parser a caller and a test name,
///  and the frame list it bounds lives there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { MAX_CONTAINERS } from './parseFrames';
/// ```
pub(crate) use crate::parse_frames::MAX_CONTAINERS;
/// What:
///  Import the container stack the driver keeps.
/// Why:
///  The stack holds one level per open container,
///  which is what replaces the recursive calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Frame, Phase, attachLast, finish } from './parseFrames';
/// ```
use crate::parse_frames::{
    Frame,
    Phase,
    attach_last,
    finish,
};
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
///  Import the recorder type that a finished parse returns beside the tree.
/// Why:
///  `parse_document` hands the recorder back so the span-reporting entry point can collect it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { SpanRecorder } from './span';
/// ```
use crate::span::SpanRecorder;
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
use crate::value::{
    JsoncComment,
    JsoncEntry,
    JsoncKey,
    JsoncKind,
    JsoncValue,
};

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
    // The first byte of this value; its span starts here, before any consumption below.
    let value_start = scanner.offset;
    if scanner.byte() == Some(b'{') || scanner.byte() == Some(b'[') {
        if frames.len() >= MAX_CONTAINERS {
            return Err(scanner.error("JSONC nesting too deep"));
        }
        // The container's slot is reserved before its children, so spans stay in document pre-order.
        scanner.spans.container_start(value_start);
        if scanner.consume(b'{') {
            frames.push(Frame::Record {
                entries: Vec::new(),
                leading,
                pending_key: None,
                phase: Phase::ItemOrClose,
            });
        } else {
            scanner.consume(b'[');
            frames.push(Frame::Array {
                elements: Vec::new(),
                leading,
                phase: Phase::ItemOrClose,
            });
        }
        return Ok(());
    }
    let kind = if scanner.byte() == Some(b'"') {
        let (units, raw) = scanner.string()?;
        JsoncKind::Text {
            units,
            raw,
        }
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
            JsoncKind::Number {
                raw,
                identity,
            }
        } else if scanner.source[scanner.offset..].starts_with("true") {
            scanner.offset += 4;
            JsoncKind::Boolean {
                value: true,
            }
        } else if scanner.source[scanner.offset..].starts_with("false") {
            scanner.offset += 5;
            JsoncKind::Boolean {
                value: false,
            }
        } else if scanner.source[scanner.offset..].starts_with("null") {
            scanner.offset += 4;
            JsoncKind::Null
        } else {
            return Err(scanner.error("expected JSONC value"));
        }
    };
    // A scalar's span is complete the moment its token has been scanned.
    scanner.spans.scalar(value_start, scanner.offset);
    // What: `Some` hands the completed scalar to the parent in the next loop pass.
    // Why: A scalar needs no container frame or recursive call.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // delivered = { kind, comment: merge(leading) };
    // ```
    *delivered = Some(JsoncValue {
        kind,
        comment: attach(None, leading),
    });
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
fn receive_child(
    scanner: &mut Scanner<'_>,
    frames: &mut [Frame],
    mut child: JsoncValue,
) -> Result<(), JsoncParseError> {
    let trailing = scanner.capture_trailing()?;
    child.comment = attach(child.comment, trailing.comments);
    let Some(parent) = frames.last_mut() else {
        unreachable!("child without parent");
    };
    if parent.phase() != Phase::AwaitingChild {
        return Err(scanner.error("unexpected completed JSONC value"));
    }
    if let Frame::Array {
        elements,
        ..
    } = parent
    {
        elements.push(child);
    } else if let Frame::Record {
        entries,
        pending_key,
        ..
    } = parent
    {
        let Some(key) = pending_key.take() else {
            unreachable!("record value without key");
        };
        entries.push(JsoncEntry {
            key,
            value: child,
        });
    }
    parent.set_phase(if trailing.comma {
        Phase::ItemOrClose
    } else {
        Phase::SeparatorOrClose
    });
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
    // What: `Scanner::new` builds a scanner whose span recorder is switched off.
    // Why: A plain parse keeps its old cost; positions are opt-in through `parse_jsonc_with_spans`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [root] = parseDocument(new Scanner(source));
    // return root;
    // ```
    let (root, _spans) = parse_document(Scanner::new(source))?;
    return Ok(root);
}

/// What:
///  Parse one document with a caller-built scanner and hand back the scanner's span recorder.
///  The scanner moves in by value (TS:
/// passing an object the callee may consume).
/// Why:
///  One parse loop serves `parse_jsonc` (recorder off) and `parse_jsonc_with_spans` (recorder on),
///  so the two can never disagree about what a document means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseDocument(scanner: Scanner): [JsoncValue, SpanRecorder]
/// ```
///
/// # Errors
/// Malformed syntax or a 513th open container returns a byte-positioned parse error.
pub fn parse_document<'a>(
    mut scanner: Scanner<'a>
) -> Result<(JsoncValue, SpanRecorder<'a>), JsoncParseError> {
    let source = scanner.source;
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
            let Some(top) = frames.last_mut() else {
                unreachable!("separator without frame");
            };
            attach_last(top, before);
            if scanner.consume(b',') {
                top.set_phase(Phase::ItemOrClose);
                continue;
            }
            if scanner.byte() == Some(top.close_byte()) {
                let closer = top.close_byte();
                scanner.consume(closer);
                scanner.spans.container_end(scanner.offset);
                let Some(done) = frames.pop() else {
                    unreachable!("close without frame");
                };
                delivered = Some(finish(done, Vec::new()));
                continue;
            }
            return Err(scanner.error("expected comma or container close"));
        }
        let Some(top) = frames.last() else {
            unreachable!("item without frame");
        };
        if scanner.byte() == Some(top.close_byte()) {
            let closer = top.close_byte();
            scanner.consume(closer);
            scanner.spans.container_end(scanner.offset);
            let Some(done) = frames.pop() else {
                unreachable!("close without frame");
            };
            delivered = Some(finish(done, before));
            continue;
        }
        if let Some(Frame::Array {
            ..
        }) = frames.last()
        {
            let Some(array_frame) = frames.last_mut() else {
                unreachable!("array frame disappeared");
            };
            array_frame.set_phase(Phase::AwaitingChild);
            begin_value(&mut scanner, before, &mut frames, &mut delivered)?;
            continue;
        }
        if scanner.byte() != Some(b'"') {
            return Err(scanner.error("expected double-quoted object key"));
        }
        let key_start = scanner.offset;
        let (units, raw) = scanner.string()?;
        scanner.spans.key(key_start, scanner.offset);
        let before_colon = scanner.trivia()?;
        if !scanner.consume(b':') {
            return Err(scanner.error("expected colon after JSONC key"));
        }
        let before_value = scanner.trivia()?;
        let key = JsoncKey {
            units,
            raw,
            comment: attach(attach(None, before), before_colon),
        };
        let Some(Frame::Record {
            pending_key,
            phase: record_phase,
            ..
        }) = frames.last_mut()
        else {
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
    return Ok((root, scanner.spans));
}
