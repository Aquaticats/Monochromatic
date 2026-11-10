//! What:
//!  Format-preserving edits to a JSON manifest:
//! find a member's value in the source text,
//! replace that value alone,
//! or add one member to an object.
//! Why:
//!  A `package.json` is strict JSON with a hand-chosen member order,
//! and `monochromatic_jsonc_edit`'s emitter is canonical JSONC
//! (two-space indentation and a comma after every member),
//! which strict JSON rejects;
//! so the edited document is spliced into the original text instead,
//! a value that already equals the wanted one is not rewritten,
//! and every byte outside the owned member survives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function jsonText(source: string): { node(path: string[]): Node | undefined; replaced(n: Node, token: string): string; withMember(n: Node, key: string, token: string): string }
//! ```

/// Imports `Evaluated` from `crate::value_eval`.
use crate::value_eval::Evaluated;
/// Imports `JsoncKind`,
/// `JsoncParseError`,
/// `JsoncSpan`,
/// `JsoncSpans`,
/// `JsoncValue`,
/// `encode_quoted`,
/// `parse_jsonc_with_spans`,
/// `units_to_string` from `monochromatic_jsonc_edit`.
use monochromatic_jsonc_edit::{
    JsoncKind, JsoncParseError, JsoncSpan, JsoncSpans, JsoncValue, encode_quoted, parse_jsonc_with_spans, units_to_string,
};
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;

/// What:
///  What a document value compares as.
/// Why:
///  A config value is text,
/// a boolean,
/// an integer or a list of those,
/// so anything else (an object,
/// `null`,
/// a number that is not an integer) can never equal one
///  and is kept out of the comparison without a second walk over the source.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Comparable = { kind: 'text'; value: string } | { kind: 'boolean'; value: boolean } | { kind: 'integer'; value: number } | { kind: 'list'; items: Comparable[] } | { kind: 'other' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Comparable {
    /// A string,
    /// decoded so two spellings of one text compare equal.
    Text(
        /// The decoded text.
        String,
    ),
    /// A boolean literal.
    Bool(
        /// The literal's truth value.
        bool,
    ),
    /// An integer literal.
    Integer(
        /// The written number.
        i64,
    ),
    /// An array of comparable values.
    List(
        /// The elements in source order.
        Vec<Comparable>,
    ),
    /// Anything a config value cannot be.
    Other,
}

/// What:
///  One object member's address and place in the source.
/// Why:
///  Replacing a value needs its own span;
/// adding a member needs the last member's span and the indentation its line uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsonMember = { name: string; keySpan: Span; valueSpan: Span };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonMember {
    /// Decoded member name.
    pub name: String,
    /// Span of the quoted key.
    pub key_span: JsoncSpan,
    /// Span of the value.
    pub value_span: JsoncSpan,
}

/// What:
///  One addressable value of a document:
/// what it compares as,
/// where it sits,
/// and the members it holds when it is an object.
/// Why:
///  One record per key path answers both questions a rule asks,
/// whether the address exists and what to write there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsonNode = { comparable: Comparable; span: Span; members?: JsonMember[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonNode {
    /// The value in comparable form.
    pub comparable: Comparable,
    /// Span of the whole value,
    /// closing brace or quote included.
    pub span: JsoncSpan,
    /// The members in source order,
    /// when the value is an object.
    pub members: Option<Vec<JsonMember>>,
}

/// What:
///  A parsed document plus every key path it holds,
/// sorted by path.
/// Why:
///  Rules address values by key path and edit text,
/// so the source and the addresses belong together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsonText { source: string; nodes: Map<string[], JsonNode> }
/// ```
#[derive(Clone, Debug)]
pub struct JsonText {
    /// The text the addresses were recorded against.
    source: String,
    /// One entry per key path;
    /// a repeated key replaces the earlier entry,
    /// so a lookup answers like the last duplicate.
    nodes: BTreeMap<Vec<String>, JsonNode>,
}

/// What:
///  How far a walk has read each span list.
/// Why:
///  The parser records spans in pre-order,
/// so one counter per list is enough to pair every node with its span.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SpanCursor { values = 0; keys = 0 }
/// ```
struct SpanCursor {
    /// Index of the next value span.
    values: usize,
    /// Index of the next key span.
    keys: usize,
}

/// What:
///  Take the next value span,
/// which the parser recorded in pre-order.
/// Why:
///  The walk visits values in the same order,
/// so the pairing cannot drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nextValue(spans: Spans): Span
/// ```
impl SpanCursor {
    /// The span of the value about to be visited.
    fn next_value(&mut self, spans: &JsoncSpans) -> JsoncSpan {
        let Some(span) = spans.values.get(self.values) else {
            unreachable!("the parse recorded fewer value spans than the tree has values");
        };
        self.values += 1;
        return *span;
    }

    /// The span of the object key about to be visited.
    fn next_key(&mut self, spans: &JsoncSpans) -> JsoncSpan {
        let Some(span) = spans.keys.get(self.keys) else {
            unreachable!("the parse recorded fewer key spans than the tree has keys");
        };
        self.keys += 1;
        return *span;
    }
}

/// What:
///  What one parsed value compares as.
/// Why:
///  A comparison against a config value must use the decoded text,
/// not the source spelling,
/// so `"git"` and `"gi\u0074"` are the same value and neither is rewritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function comparableOf(value: JsoncValue): Comparable
/// ```
fn comparable_of(value: &JsoncValue) -> Comparable {
    match &value.kind {
        JsoncKind::Text { units, .. } => match units_to_string(units) {
            Ok(text) => return Comparable::Text(text),
            Err(_) => return Comparable::Other,
        },
        JsoncKind::Boolean { value: flag } => return Comparable::Bool(*flag),
        JsoncKind::Number { raw, .. } => match raw.parse::<i64>() {
            Ok(number) => return Comparable::Integer(number),
            Err(_) => return Comparable::Other,
        },
        JsoncKind::Array { elements } => {
            let mut items: Vec<Comparable> = Vec::new();
            for element in elements {
                items.push(comparable_of(element));
            }
            return Comparable::List(items);
        }
        JsoncKind::Null | JsoncKind::Record { .. } => return Comparable::Other,
    }
}

/// What:
///  Address every value of one document.
/// Why:
///  One walk records the spans,
/// the member layouts and the comparable forms a rule needs,
/// in the order the parser recorded spans.
///  An array element is addressed by its index as `[n]`,
/// which no config path writes,
/// so an element is reachable but never edited by key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function collect(value: JsoncValue, spans: JsoncSpans, cursor: SpanCursor, path: string[], out: Map<string[], JsonNode>): void
/// ```
fn collect(
    value: &JsoncValue,
    spans: &JsoncSpans,
    cursor: &mut SpanCursor,
    path: &mut Vec<String>,
    out: &mut BTreeMap<Vec<String>, JsonNode>,
) {
    let comparable = comparable_of(value);
    let span = cursor.next_value(spans);
    let members = match &value.kind {
        JsoncKind::Record { entries } => {
            let mut members: Vec<JsonMember> = Vec::new();
            for entry in entries {
                let key_span = cursor.next_key(spans);
                let name = match units_to_string(&entry.key.units) {
                    Ok(decoded) => decoded,
                    Err(_) => entry.key.raw.clone(),
                };
                let child = cursor.values;
                path.push(name.clone());
                collect(&entry.value, spans, cursor, path, out);
                path.pop();
                let Some(value_span) = spans.values.get(child) else {
                    unreachable!("the parse recorded fewer value spans than the tree has values");
                };
                members.push(JsonMember { name, key_span, value_span: *value_span });
            }
            Some(members)
        }
        JsoncKind::Array { elements } => {
            for (index, element) in elements.iter().enumerate() {
                path.push(format!("[{index}]"));
                collect(element, spans, cursor, path, out);
                path.pop();
            }
            None
        }
        JsoncKind::Text { .. } | JsoncKind::Number { .. } | JsoncKind::Boolean { .. } | JsoncKind::Null => None,
    };
    out.insert(path.clone(), JsonNode { comparable, span, members });
}

/// What:
///  Whether a located value already holds the wanted value.
/// Why:
///  Equal values are left untouched,
/// so a conforming manifest stays byte-identical and its spelling survives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function valueMatches(found: Comparable, wanted: Evaluated): boolean
/// ```
pub fn value_matches(found: &Comparable, wanted: &Evaluated) -> bool {
    match (found, wanted) {
        (Comparable::Text(written), Evaluated::Text(asked)) => return written == asked,
        (Comparable::Bool(written), Evaluated::Bool(asked)) => return written == asked,
        (Comparable::Integer(written), Evaluated::Integer(asked)) => return written == asked,
        (Comparable::List(written), Evaluated::List(asked)) => {
            if written.len() != asked.len() {
                return false;
            }
            for (index, item) in asked.iter().enumerate() {
                let Some(element) = written.get(index) else { return false };
                if !value_matches(element, item) {
                    return false;
                }
            }
            return true;
        }
        _ => return false,
    }
}

/// What:
///  A computed value as the JSON text that stores it.
/// Why:
///  An edit writes what the config computed,
/// quoted and escaped by the same rules the parser reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function tokenText(value: Evaluated): string
/// ```
pub fn token_text(value: &Evaluated) -> String {
    match value {
        Evaluated::Text(text) => {
            let units: Vec<u16> = text.encode_utf16().collect();
            return encode_quoted(&units);
        }
        Evaluated::Bool(flag) => return flag.to_string(),
        Evaluated::Integer(number) => return number.to_string(),
        Evaluated::List(items) => {
            let mut out = String::from("[");
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&token_text(item));
            }
            out.push(']');
            return out;
        }
    }
}

/// What:
///  Parse a document and read the text it was parsed from,
/// so an address and an edit describe one version of one file.
/// Why:
///  Splicing by offset is only correct against the text the offsets came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseJsonText(source: string): JsonText
/// ```
///
/// # Errors
/// Returns the parser's failure when the source is not JSONC.
pub fn parse_json_text(source: &str) -> Result<JsonText, JsoncParseError> {
    let parsed = parse_jsonc_with_spans(source)?;
    let mut nodes: BTreeMap<Vec<String>, JsonNode> = BTreeMap::new();
    let mut cursor = SpanCursor { values: 0, keys: 0 };
    let mut path: Vec<String> = Vec::new();
    collect(&parsed.value, &parsed.spans, &mut cursor, &mut path, &mut nodes);
    return Ok(JsonText { source: source.to_string(), nodes });
}

/// What:
///   How a rule reads and edits one parsed document:
///   look an address up,
///   replace a value,
///   add a member.
/// Why:
///   Every method splices the recorded text,
/// which it owns,
/// so no caller can pair an address with a different version of the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsonText { node(path): JsonNode | undefined; replaced(node, token): string; withMember(node, key, token): string | undefined }
/// ```
impl JsonText {
    /// The node at a key path,
    /// or nothing when the path does not exist.
    pub fn node(&self, path: &[String]) -> Option<&JsonNode> {
        return self.nodes.get(path);
    }

    /// What:
    ///  The source with one value's token replaced and every other byte kept.
    /// Why:
    ///  The owned value changes;
    /// the member order,
    /// indentation and unowned members of the file do not.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// replaced(node: JsonNode, token: string): string
    /// ```
    pub fn replaced(&self, node: &JsonNode, token: &str) -> String {
        let mut out = String::with_capacity(self.source.len() + token.len());
        out.push_str(&self.source[..node.span.start.offset]);
        out.push_str(token);
        out.push_str(&self.source[node.span.end.offset..]);
        return out;
    }

    /// What:
    ///  The source with one member added to an object,
    /// as its last member.
    ///  `None` when the value is not an object.
    /// Why:
    ///  Appending keeps the existing members where they are;
    /// a value the run cannot place is left alone rather than guessed at.
    /// Gotcha:
    ///  A comment between the last member's value and its comma moves to the added member;
    /// the owned manifests are strict JSON,
    /// which has no comments.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withMember(node: JsonNode, key: string, token: string): string | undefined
    /// ```
    pub fn with_member(&self, node: &JsonNode, key: &str, token: &str) -> Option<String> {
        let members = node.members.as_ref()?;
        let units: Vec<u16> = key.encode_utf16().collect();
        let quoted = encode_quoted(&units);
        let mut out = String::with_capacity(self.source.len() + quoted.len() + token.len() + 8);
        let Some(last) = members.last() else {
            // An empty object gains a member between its own braces, at the closing brace's indentation.
            let at = node.span.end.offset.checked_sub(1)?;
            let indent = self.line_indent(node.span.start.offset);
            out.push_str(&self.source[..at]);
            out.push('\n');
            out.push_str(indent);
            out.push_str("  ");
            out.push_str(&quoted);
            out.push_str(": ");
            out.push_str(token);
            out.push('\n');
            out.push_str(indent);
            out.push_str(&self.source[at..]);
            return Some(out);
        };
        let last_end = last.value_span.end.offset;
        let first = members.first().unwrap_or(last);
        let indent = self.line_indent(first.key_span.start.offset);
        // A member that already ends with a comma lends it to the added one instead of gaining a second.
        let (at, separator) = match self.next_significant(last_end, node.span.end.offset) {
            Some((offset, b',')) => (offset + 1, ""),
            _ => (last_end, ","),
        };
        out.push_str(&self.source[..at]);
        out.push_str(separator);
        out.push('\n');
        out.push_str(indent);
        out.push_str(&quoted);
        out.push_str(": ");
        out.push_str(token);
        out.push_str(&self.source[at..]);
        return Some(out);
    }

    /// What:
    ///  The whitespace that starts the line holding `offset`.
    /// Why:
    ///  An added member takes the indentation of the line the object's first member is on,
    /// and an empty object's braces take the indentation of the line the object starts on,
    /// so no indentation is invented.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lineIndent(offset: number): string
    /// ```
    fn line_indent(&self, offset: usize) -> &str {
        let bytes = self.source.as_bytes();
        let head = match self.source.get(..offset) {
            Some(head) => head,
            None => "",
        };
        let start = match head.rfind('\n') {
            Some(index) => index + 1,
            None => 0,
        };
        let mut end = start;
        while let Some(byte) = bytes.get(end) {
            if *byte != b' ' && *byte != b'\t' {
                break;
            }
            end += 1;
        }
        return &self.source[start..end];
    }

    /// What:
    ///  The offset and the byte of the first non-whitespace between two offsets.
    /// Why:
    ///  Whether a member already ends with a comma,
    /// and where that comma sits,
    /// decides where an added member goes and whether it needs a comma of its own.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextSignificant(from: number, to: number): [number, number] | undefined
    /// ```
    fn next_significant(&self, from: usize, to: usize) -> Option<(usize, u8)> {
        let bytes = self.source.as_bytes();
        let mut at = from;
        while at < to {
            let byte = bytes.get(at)?;
            if !byte.is_ascii_whitespace() {
                return Some((at, *byte));
            }
            at += 1;
        }
        return None;
    }
}
