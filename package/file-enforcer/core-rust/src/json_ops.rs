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
//!  Memory stays proportional to the document:
//! the parse tree is kept and each address is found by walking it,
//! so no node holds a copied path or a second copy of its own subtree
//! (the release pass measured 181404 kB of peak resident size for a 15 KB document when it did).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function jsonText(source: string): { node(path: string[]): Node | undefined; replaced(n: Node, token: string): string; withMember(n: Node, key: string, token: string): string }
//! ```

/// Imports `Comparable`,
/// `comparable_of` from `crate::json_compare`.
use crate::json_compare::{
    Comparable,
    comparable_of,
};
/// Imports `JsoncEntry`,
/// `JsoncKind`,
/// `JsoncParseError`,
/// `JsoncSpan`,
/// `JsoncSpans`,
/// `JsoncValue`,
/// `encode_quoted`,
/// `parse_jsonc_with_spans`,
/// `units_to_string` from `monochromatic_jsonc_edit`.
use monochromatic_jsonc_edit::{
    JsoncEntry,
    JsoncKind,
    JsoncParseError,
    JsoncSpan,
    JsoncSpans,
    JsoncValue,
    encode_quoted,
    parse_jsonc_with_spans,
    units_to_string,
};

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
///  One addressable value of a document,
/// as a read-only view of the parse tree:
/// where it sits,
/// which members it holds when it is an object,
/// and how to read it.
///  `&'a JsoncValue` borrows from the tree the document keeps alive
/// (sibling: an owned `JsoncValue`, a copy);
/// `JsonText` owns that tree for as long as the document exists.
/// Why:
///  A borrower costs nothing,
/// where a copy of the value per node cost the whole subtree per node.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsonNode = { span: Span; members?: JsonMember[]; comparable(): Comparable };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonNode<'a> {
    /// The value itself, borrowed from the document's tree.
    value: &'a JsoncValue,
    /// Span of the whole value,
    /// closing brace or quote included.
    pub span: JsoncSpan,
    /// The members in source order,
    /// when the value is an object.
    pub members: Option<Vec<JsonMember>>,
}

/// What:
///  How one node answers the two questions a rule asks of it,
/// without copying anything into the node.
/// Why:
///  The comparison and the shape are only built for the paths an edit names,
///  so a large unowned value costs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsonNode = { comparable(): Comparable; shape(): string };
/// ```
impl<'a> JsonNode<'a> {
    /// What:
    ///  What this value compares as,
    /// built now rather than kept beside every node.
    /// Why:
    ///  A rule compares one value per edit,
    /// so only that value pays for a comparable form.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// comparable(): Comparable
    /// ```
    pub fn comparable(&self) -> Comparable {
        return comparable_of(self.value);
    }

    /// What:
    ///  What kind of thing the value is, in the words a message uses: `an object`,
    ///  `an array`,
    ///  `a string`,
    ///  `a number`,
    ///  `a boolean` or `null`.
    /// Why:
    ///  A refusal only helps when it says what it found.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// shape(): string
    /// ```
    pub fn shape(&self) -> &'static str {
        return shape_name(self.value);
    }

    /// What:
    ///  Whether the value is an object, the one kind that can hold a member.
    /// Why:
    ///  Adding a member needs an object to add it to,
    /// and a caller that has to refuse the other kinds asks this before it builds a message about them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isObject(): boolean
    /// ```
    pub fn is_object(&self) -> bool {
        return matches!(self.value.kind, JsoncKind::Record { .. });
    }
}

/// What:
///  The plain-words name of a value's kind.
/// Why:
///  One list of names keeps every diagnostic's vocabulary the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shapeName(value: JsoncValue): string
/// ```
fn shape_name(value: &JsoncValue) -> &'static str {
    match &value.kind {
        JsoncKind::Record {
            ..
        } => return "an object",
        JsoncKind::Array {
            ..
        } => return "an array",
        JsoncKind::Text {
            ..
        } => return "a string",
        JsoncKind::Number {
            ..
        } => return "a number",
        JsoncKind::Boolean {
            ..
        } => return "a boolean",
        JsoncKind::Null => return "null",
    }
}

/// What:
///  A parsed document plus the span lists its addresses are found through.
/// Why:
///  Rules address values by key path and edit text,
/// so the source, the tree and the spans belong together,
/// and the walk that found a node and the splice that edits it see one version of one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsonText { source: string; value: JsoncValue; spans: JsoncSpans }
/// ```
#[derive(Clone, Debug)]
pub struct JsonText {
    /// The text the spans were recorded against.
    source: String,
    /// The document tree the addresses are found in.
    value: JsoncValue,
    /// One span per value and one per object key, in pre-order.
    spans: JsoncSpans,
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
///  The counter reads the next span of a list.
/// Why:
///  The walk visits values and keys in the order the parser recorded them,
/// so the pairing cannot drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SpanCursor = { nextValue(spans): Span; nextKey(spans): Span };
/// ```
impl SpanCursor {
    /// The span of the value about to be visited.
    fn next_value(
        &mut self,
        spans: &JsoncSpans,
    ) -> JsoncSpan {
        let Some(span) = spans.values.get(self.values) else {
            unreachable!("the parse recorded fewer value spans than the tree has values");
        };
        self.values += 1;
        return *span;
    }

    /// The span of the object key about to be visited.
    fn next_key(
        &mut self,
        spans: &JsoncSpans,
    ) -> JsoncSpan {
        let Some(span) = spans.keys.get(self.keys) else {
            unreachable!("the parse recorded fewer key spans than the tree has keys");
        };
        self.keys += 1;
        return *span;
    }
}

/// What:
///  Find one value by its key path while reading every span up to it,
/// and build the member list of the object on that path.
///  `Option<&[String]>` is "the path still to match, or nothing to look for" (TS:
/// `string[] | undefined`);
///  `Option<JsonNode<'a>>` is "a node or nothing".
/// Why:
///  One walk keeps the span counters aligned without an index,
/// an array element is addressed by its index as `[n]` (no config path writes one),
/// and a repeated key replaces the earlier member for every path below it,
/// so a lookup answers as the last duplicate and an edit never lands in a shadowed copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function locate(value, spans, cursor, wanted): JsonNode | undefined
/// ```
fn locate<'a>(
    value: &'a JsoncValue,
    spans: &JsoncSpans,
    cursor: &mut SpanCursor,
    wanted: Option<&[String]>,
) -> Option<JsonNode<'a>> {
    let span = cursor.next_value(spans);
    // This node is the answer when the path to match is empty.
    let is_target = matches!(wanted, Some(rest) if rest.is_empty());
    match &value.kind {
        JsoncKind::Record {
            entries,
        } => {
            // The member list is built only for the node an edit will add to.
            let mut members: Option<Vec<JsonMember>> =
                if is_target { Some(Vec::new()) } else { None };
            let mut found: Option<JsonNode<'a>> = None;
            for entry in entries {
                let key_span = cursor.next_key(spans);
                let name = member_name(entry);
                let child = match wanted {
                    Some(rest) if rest.first() == Some(&name) => Some(&rest[1..]),
                    _ => None,
                };
                let child_start = cursor.values;
                let located = locate(&entry.value, spans, cursor, child);
                if child.is_some() {
                    // A later member with the same name replaces an earlier one, so it also decides
                    // every path below it: a match, or nothing when the later value has no such path.
                    found = located;
                }
                if let Some(list) = members.as_mut() {
                    let Some(value_span) = spans.values.get(child_start) else {
                        unreachable!(
                            "the parse recorded fewer value spans than the tree has values"
                        );
                    };
                    list.push(JsonMember {
                        name,
                        key_span,
                        value_span: *value_span,
                    });
                }
            }
            if is_target {
                return Some(JsonNode {
                    value,
                    span,
                    members,
                });
            }
            // `None` here is "no member of this object matched".
            return found;
        }
        JsoncKind::Array {
            elements,
        } => {
            let mut found: Option<JsonNode<'a>> = None;
            for (index, element) in elements.iter().enumerate() {
                let child = match wanted {
                    Some(rest) if !rest.is_empty() => {
                        let address = format!("[{index}]");
                        if rest[0] == address {
                            Some(&rest[1..])
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                let located = locate(element, spans, cursor, child);
                if child.is_some() {
                    found = located;
                }
            }
            if is_target {
                return Some(JsonNode {
                    value,
                    span,
                    members: None,
                });
            }
            return found;
        }
        JsoncKind::Text {
            ..
        }
        | JsoncKind::Number {
            ..
        }
        | JsoncKind::Boolean {
            ..
        }
        | JsoncKind::Null => {
            if is_target {
                return Some(JsonNode {
                    value,
                    span,
                    members: None,
                });
            }
            return None;
        }
    }
}

/// What:
///  The decoded name of one object member.
/// Why:
///  A key spelled with an escape addresses the same member as its plain spelling,
/// and a key an escape cannot decode is addressed by its raw text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function memberName(entry: JsoncMember): string
/// ```
fn member_name(entry: &JsoncEntry) -> String {
    return match units_to_string(&entry.key.units) {
        Ok(decoded) => decoded,
        Err(_) => entry.key.raw.clone(),
    };
}

/// What:
///  Parse a document and keep the text it was parsed from,
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
    return Ok(JsonText {
        source: source.to_string(),
        value: parsed.value,
        spans: parsed.spans,
    });
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
/// class JsonText { node(path): Node | undefined; token(node): string; replaced(node, token): string; withMember(node, key, token): string | undefined }
/// ```
impl JsonText {
    /// What:
    ///  The node at a key path,
    /// or nothing when the path does not exist.
    /// Why:
    ///  A caller asks about one address at a time and never pays for the others.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// node(path: string[]): Node | undefined
    /// ```
    pub fn node(
        &self,
        path: &[String],
    ) -> Option<JsonNode<'_>> {
        let mut cursor = SpanCursor {
            values: 0,
            keys: 0,
        };
        return locate(&self.value, &self.spans, &mut cursor, Some(path));
    }

    /// What:
    ///  The source text a node's span covers, byte for byte.
    /// Why:
    ///  A diagnostic that says what the manifest holds quotes the file, not a re-rendered value.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// token(node: Node): string
    /// ```
    pub fn token(
        &self,
        node: &JsonNode<'_>,
    ) -> &str {
        // The fallback only fires for a span outside the source, which the parser never records.
        return self
            .source
            .get(node.span.start.offset..node.span.end.offset)
            .unwrap_or("");
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
    /// replaced(node: Node, token: string): string
    /// ```
    pub fn replaced(
        &self,
        node: &JsonNode<'_>,
        token: &str,
    ) -> String {
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
    ///  The added line takes the line ending the file itself uses and the indentation unit of its
    ///  first indented line,
    /// so a CRLF or tab-indented manifest keeps its own layout.
    /// Gotcha:
    ///  A comment between the last member's value and its comma moves to the added member;
    /// the owned manifests are strict JSON,
    /// which has no comments.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withMember(node: Node, key: string, token: string): string | undefined
    /// ```
    pub fn with_member(
        &self,
        node: &JsonNode<'_>,
        key: &str,
        token: &str,
    ) -> Option<String> {
        let members = node.members.as_ref()?;
        let units: Vec<u16> = key.encode_utf16().collect();
        let quoted = encode_quoted(&units);
        let ending = self.line_ending();
        let mut out = String::with_capacity(self.source.len() + quoted.len() + token.len() + 8);
        let Some(last) = members.last() else {
            // An empty object gains a member between its own braces, one indentation unit in from the
            // line the object starts on.
            let at = node.span.end.offset.checked_sub(1)?;
            let object_indent = self.line_indent(node.span.start.offset);
            out.push_str(&self.source[..at]);
            out.push_str(ending);
            out.push_str(object_indent);
            out.push_str(self.indentation_unit());
            out.push_str(&quoted);
            out.push_str(": ");
            out.push_str(token);
            out.push_str(ending);
            out.push_str(object_indent);
            out.push_str(&self.source[at..]);
            return Some(out);
        };
        let last_end = last.value_span.end.offset;
        let first = members.first().unwrap_or(last);
        // `String` is owned, growable text (sibling: `&str`), because one branch builds a new indent.
        let member_indent: String = if self.starts_line(first.key_span.start.offset) {
            self.line_indent(first.key_span.start.offset).to_string()
        } else {
            // Members written on one line (a minified or inline object) get a line of their own at the
            // object's indentation plus one unit, rather than at the file's left edge.
            format!(
                "{}{}",
                self.line_indent(node.span.start.offset),
                self.indentation_unit()
            )
        };
        // A member that already ends with a comma lends it to the added one instead of gaining a second.
        let (at, separator) = match self.next_significant(last_end, node.span.end.offset) {
            Some((offset, b',')) => (offset + 1, ""),
            _ => (last_end, ","),
        };
        out.push_str(&self.source[..at]);
        out.push_str(separator);
        out.push_str(ending);
        out.push_str(&member_indent);
        out.push_str(&quoted);
        out.push_str(": ");
        out.push_str(token);
        out.push_str(&self.source[at..]);
        return Some(out);
    }

    /// What:
    ///  The line ending this document uses:
    /// CRLF when it holds one, else LF.
    /// Why:
    ///  An added line must end the way the file's own lines end,
    /// or a CRLF file gains mixed endings.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lineEnding(): string
    /// ```
    fn line_ending(&self) -> &str {
        if self.source.contains("\r\n") {
            return "\r\n";
        }
        return "\n";
    }

    /// What:
    ///  The indentation unit of this document:
    /// the leading whitespace of its first indented line,
    /// or two spaces when no line is indented.
    /// Why:
    ///  A tab-indented file must gain tab-indented members;
    /// the first indented line shows the unit without reading the whole file.
    /// Gotcha:
    ///  A file whose first indented line sits deeper than one level understates its unit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// indentationUnit(): string
    /// ```
    fn indentation_unit(&self) -> &str {
        for line in self.source.split('\n') {
            let trimmed = line.trim_start_matches([' ', '\t']);
            if trimmed.len() != line.len() {
                return &line[..line.len() - trimmed.len()];
            }
        }
        return "  ";
    }

    /// What:
    ///  The offset the line holding `offset` starts at.
    /// Why:
    ///  The indentation and the "does this member start its own line" question are both measured from there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// lineStart(offset: number): number
    /// ```
    fn line_start(
        &self,
        offset: usize,
    ) -> usize {
        // A caller passes an offset inside the source; the empty fallback means "no line before it".
        let head = self.source.get(..offset).unwrap_or("");
        return match head.rfind('\n') {
            Some(index) => index + 1,
            None => 0,
        };
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
    fn line_indent(
        &self,
        offset: usize,
    ) -> &str {
        let bytes = self.source.as_bytes();
        let start = self.line_start(offset);
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
    ///  Whether nothing but indentation sits before `offset` on its line.
    /// Why:
    ///  An inline member shares a line with its object,
    /// and an added member cannot reuse that indentation.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// startsLine(offset: number): boolean
    /// ```
    fn starts_line(
        &self,
        offset: usize,
    ) -> bool {
        return self.line_start(offset) + self.line_indent(offset).len() == offset;
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
    fn next_significant(
        &self,
        from: usize,
        to: usize,
    ) -> Option<(usize, u8)> {
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
