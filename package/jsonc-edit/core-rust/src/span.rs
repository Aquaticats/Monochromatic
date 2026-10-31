//! What:
//!  Source positions for parsed JSONC values and object keys.
//! Why:
//!  A caller that decodes a document into its own types (a config file,
//! say) must be able to tell the user
//!  `file:line:col` for a value that is wrong,
//! and the parsed tree itself carries no positions because its
//!  structs are compared,
//! hashed and built by hand elsewhere.
//!  Positions therefore travel beside the tree in a parallel list.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Position = { offset: number; line: number; column: number };
//! type Span = { start: Position; end: Position };
//! type Spans = { values: Span[]; keys: Span[] };
//! ```

/// What:
///  Import the parse failure type.
/// Why:
///  A malformed document is an ordinary result of `parse_jsonc_with_spans`,
/// as for `parse_jsonc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncParseError } from './error';
/// ```
use crate::error::JsoncParseError;
/// What:
///  Import the shared parse loop.
/// Why:
///  `parse_jsonc_with_spans` runs the same loop as `parse_jsonc` with the recorder switched on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseDocument } from './parse';
/// ```
use crate::parse::parse_document;
/// What:
///  Import the scanner whose recorder can be switched on.
/// Why:
///  The span-reporting entry point builds the scanner itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Scanner } from './scan';
/// ```
use crate::scan::Scanner;
/// What:
///  Import the document tree type that [`JsoncSpanned`] pairs with its positions.
/// Why:
///  The convenience result holds the tree and its spans together so a caller cannot pair spans with the wrong tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncValue } from './value';
/// ```
use crate::value::JsoncValue;

/// What:
///  One place in the source text:
/// a byte offset plus the line and column of that offset.
///  `usize` is the unsigned integer wide enough to index any byte of a string (siblings:
/// `u32`,
/// `u64`).
/// Why:
///  Offsets slice the source;
/// line and column are what a person types into an editor.
///  `usize` and not `u32` or `u64` because every standard string-slicing API takes `usize`.
/// Gotcha:
///  `line` and `column` both start at 1.
///  The column counts Unicode scalar values (what a TypeScript spread `[...line]` counts),
/// not bytes and not UTF-16 units.
///  Only LF ends a line (CRLF is one LF preceded by a column);
/// a bare CR is one column,
///  the same line rule `README.md` states for comment ownership.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncPosition = { offset: number; line: number; column: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct JsoncPosition {
    /// Byte offset into the source,
    /// counting from 0.
    pub offset: usize,
    /// Line number,
    /// counting from 1.
    pub line: usize,
    /// Column number in Unicode scalar values,
    /// counting from 1.
    pub column: usize,
}

/// What:
///  The source range of one value or one object key:
/// where it starts and where it ends.
/// Why:
///  Error messages point at the start;
///  slicing the source between the two offsets gives back the exact token text.
///  `end` is exclusive:
/// it names the position just after the last byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncSpan = { start: JsoncPosition; end: JsoncPosition };
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct JsoncSpan {
    /// Position of the first byte (the opening quote,
    /// bracket,
    /// brace or first literal character).
    pub start: JsoncPosition,
    /// Position just after the last byte.
    pub end: JsoncPosition,
}

/// What:
///  The positions of every value and every object key of one parsed document,
///  each list in document (pre-)order.
///  `Vec<JsoncSpan>` is a growable list (TS:
/// `JsoncSpan[]`;
/// sibling `[JsoncSpan; N]` is a fixed-size array).
/// Why:
///  Pre-order means a container comes before its children and members come in source order,
///  so a caller walking the tree depth-first with one counter per list reads each node's span without any lookup,
///  and the parser records them in a single forward pass.
///  The root is `values[0]`;
/// the key of the first member of the first object is `keys[0]`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncSpans = { values: JsoncSpan[]; keys: JsoncSpan[] };
/// // values[i] is the i-th value in a depth-first walk; keys[j] is the j-th member key in the same walk.
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JsoncSpans {
    /// One span per value (scalars,
    /// arrays and objects),
    /// the root first.
    pub values: Vec<JsoncSpan>,
    /// One span per object member key,
    /// in source order.
    pub keys: Vec<JsoncSpan>,
}

/// What:
///  A parsed document together with the positions of its nodes.
/// Why:
///  [`crate::parse_jsonc_with_spans`] returns both from one parse,
///  so the tree and its spans always describe the same text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncSpanned = { value: JsoncValue; spans: JsoncSpans };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsoncSpanned {
    /// The document tree,
    /// identical to what [`crate::parse_jsonc`] returns for the same text.
    pub value: JsoncValue,
    /// The positions of the tree's nodes.
    pub spans: JsoncSpans,
}

/// What:
///  A forward-only reader of positions over a source string.
///  `&'a str` is a borrowed view of text the cursor does not own (sibling:
/// `String`,
/// an owned copy);
///  `'a` names how long that borrow lasts.
/// Why:
///  Asking for the position of each offset in increasing order costs one pass over the text in total,
///  where counting lines from the start for every request would cost a pass per request.
///  Borrowed and not owned because the parser already holds the source for its whole run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Cursor { constructor(readonly source: string) {} offset = 0; line = 1; column = 1; }
/// ```
struct Cursor<'a> {
    /// The text being measured.
    source: &'a str,
    /// Byte offset the counters below describe.
    offset: usize,
    /// Line number at `offset`.
    line: usize,
    /// Column number at `offset`.
    column: usize,
}

/// What:
///  The cursor's operations.
/// Why:
///  Keeping them together makes the one invariant easy to see:
///  `advance_to` is only ever called with offsets that never decrease.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Cursor { advanceTo(target: number): JsoncPosition }
/// ```
impl<'a> Cursor<'a> {
    /// What:
    ///  Start a cursor at the beginning of the text,
    /// line 1 column 1.
    /// Why:
    ///  Every document is measured from its first byte.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new Cursor(source)
    /// ```
    fn new(source: &'a str) -> Self {
        return Self { source, offset: 0, line: 1, column: 1 };
    }

    /// What:
    ///  Move forward to `target` (clamped to the end of the text) and report the position there.
    /// Why:
    ///  The parser records span ends and starts in the order it meets them,
    ///  and that order never goes backwards,
    /// so each byte is visited once.
    ///  An offset before the cursor is a parser bug,
    /// not bad input,
    /// so it stops the program loudly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// advanceTo(target: number): JsoncPosition {
    ///   if (target < this.offset) throw new Error('positions must be requested in order');
    ///   while (this.offset < Math.min(target, source.length)) { /* count one byte */ }
    ///   return { offset: this.offset, line: this.line, column: this.column };
    /// }
    /// ```
    fn advance_to(&mut self, target: usize) -> JsoncPosition {
        let end = target.min(self.source.len());
        if end < self.offset {
            unreachable!("span positions must be requested in non-decreasing offset order");
        }
        // What: `as_bytes()` borrows the text as raw bytes (sibling: `.chars()` yields Unicode scalar values).
        // Why: Bytes let one loop count lines and columns without decoding UTF-8.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const bytes = new TextEncoder().encode(source);
        // ```
        let bytes = self.source.as_bytes();
        while self.offset < end {
            let byte = bytes[self.offset];
            if byte == b'\n' {
                self.line += 1;
                self.column = 1;
            } else if (byte & 0b1100_0000) != 0b1000_0000 {
                // A UTF-8 continuation byte (10xxxxxx) is the middle of a character, so only the first byte of
                // each character adds a column.
                self.column += 1;
            }
            self.offset += 1;
        }
        return JsoncPosition { offset: self.offset, line: self.line, column: self.column };
    }
}

/// What:
///  Collects value and key spans while the parser runs,
/// or does nothing when switched off.
/// Why:
///  `parse_jsonc` and `parse_jsonc_with_spans` share one parser,
///  and the plain entry point must not pay for positions it throws away.
///  `open` remembers which container slots are still waiting for their end,
/// innermost last.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SpanRecorder { enabled: boolean; cursor: Cursor; spans: JsoncSpans; open: number[] = []; }
/// ```
pub struct SpanRecorder<'a> {
    /// `false` makes every method return at once.
    enabled: bool,
    /// Position source for starts and ends.
    cursor: Cursor<'a>,
    /// The lists being filled.
    spans: JsoncSpans,
    /// Indexes into `spans.values` of containers whose closing byte has not been seen.
    open: Vec<usize>,
}

/// What:
///  The recorder's operations.
/// Why:
///  The parser calls one method per event (scalar,
/// container start,
/// container end,
/// key),
///  so span bookkeeping stays out of the parser's grammar logic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SpanRecorder { scalar(...): void; containerStart(...): void; containerEnd(...): void; key(...): void }
/// ```
impl<'a> SpanRecorder<'a> {
    /// What:
    ///  Build a recorder;
    /// with `enabled` false it records nothing.
    /// Why:
    ///  The same parse loop serves both entry points.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new SpanRecorder(source, enabled)
    /// ```
    pub fn new(source: &'a str, enabled: bool) -> Self {
        return Self { enabled, cursor: Cursor::new(source), spans: JsoncSpans::default(), open: Vec::new() };
    }

    /// What:
    ///  Record a complete scalar value that spans `start..end` bytes.
    /// Why:
    ///  Scalars finish in the same step they start,
    /// so one call adds the whole span.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scalar(start: number, end: number): void
    /// ```
    pub fn scalar(&mut self, start: usize, end: usize) {
        if !self.enabled {
            return;
        }
        let first = self.cursor.advance_to(start);
        let last = self.cursor.advance_to(end);
        self.spans.values.push(JsoncSpan { start: first, end: last });
    }

    /// What:
    ///  Record that a container's opening byte sits at `start`,
    /// reserving its slot before its children.
    /// Why:
    ///  Pre-order requires the container's entry to come before its children's,
    ///  but its end is unknown until the closing byte,
    /// so the end is a placeholder until `container_end`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// containerStart(start: number): void
    /// ```
    pub fn container_start(&mut self, start: usize) {
        if !self.enabled {
            return;
        }
        let first = self.cursor.advance_to(start);
        self.open.push(self.spans.values.len());
        self.spans.values.push(JsoncSpan { start: first, end: first });
    }

    /// What:
    ///  Record that the innermost open container ended at byte `end` (just after its closing byte).
    /// Why:
    ///  Containers close in last-opened-first order,
    /// so the top of `open` is the one that ended.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// containerEnd(end: number): void
    /// ```
    pub fn container_end(&mut self, end: usize) {
        if !self.enabled {
            return;
        }
        let last = self.cursor.advance_to(end);
        let Some(slot) = self.open.pop() else {
            unreachable!("a container closed without having opened");
        };
        self.spans.values[slot].end = last;
    }

    /// What:
    ///  Record an object member key that spans `start..end` bytes.
    /// Why:
    ///  Messages about an unknown or misplaced key should point at the key,
    /// not at its value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// key(start: number, end: number): void
    /// ```
    pub fn key(&mut self, start: usize, end: usize) {
        if !self.enabled {
            return;
        }
        let first = self.cursor.advance_to(start);
        let last = self.cursor.advance_to(end);
        self.spans.keys.push(JsoncSpan { start: first, end: last });
    }

    /// What:
    ///  Hand over the collected spans.
    /// Why:
    ///  The parser is finished with the recorder,
    /// so ownership moves to the caller instead of copying.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finish(): JsoncSpans
    /// ```
    pub fn finish(self) -> JsoncSpans {
        return self.spans;
    }
}

/// What:
///  Parse one JSONC document and also report where every value and object key sits in the text.
/// Why:
///  The tree is exactly what [`crate::parse_jsonc`] returns;
///  the spans come from the same single pass,
/// so the cost is linear in the text and the nesting limit is unchanged.
///  A caller walks the tree depth-first with one counter per span list (see [`JsoncSpans`]).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseJsoncWithSpans(source: string): JsoncSpanned
/// ```
///
/// # Errors
/// Malformed syntax or a 513th open container returns a byte-positioned parse error,
/// as for `parse_jsonc`.
pub fn parse_jsonc_with_spans(source: &str) -> Result<JsoncSpanned, JsoncParseError> {
    // What: `?` returns the parse error early; on success the tuple is unpacked into the tree and the recorder.
    // Why: Spans are only meaningful beside the tree they were recorded for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [value, recorder] = parseDocument(new ScannerWithSpans(source));
    // return { value, spans: recorder.finish() };
    // ```
    let (value, recorder) = parse_document(Scanner::with_spans(source))?;
    return Ok(JsoncSpanned { value, spans: recorder.finish() });
}

/// What:
///  The line,
/// column and offset of one byte offset in a source text.
///  Offsets past the end report the end of the text.
/// Why:
///  A caller that holds only a byte offset (a parse error's `offset`,
/// for example)
///  gets the same line and column rule the spans use,
/// so every position in a message agrees.
///  This is one pass over the text before the offset;
/// call it for a handful of offsets,
/// not for every node.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncPositionAt(source: string, offset: number): JsoncPosition
/// ```
pub fn jsonc_position_at(source: &str, offset: usize) -> JsoncPosition {
    let mut cursor = Cursor::new(source);
    return cursor.advance_to(offset);
}
