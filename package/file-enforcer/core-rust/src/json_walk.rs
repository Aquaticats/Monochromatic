//! What:
//!  Address a value of a parsed JSON document by its key path.
//! Why:
//!  A rule asks about a handful of paths in a file it does not own the rest of,
//! so the address is found by walking the parse tree instead of indexing every node at parse time:
//! no node holds a copied path,
//! and nothing is built for a value no edit names.
//!  A repeated key answers as its last occurrence for every path below it,
//! the way a JSON reader resolves duplicates,
//! so an edit never lands in a copy a later member shadows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // private module: type JsonNode; function locate(value, spans, cursor, wanted): JsonNode | undefined
//! ```

/// Imports `Comparable`,
/// `comparable_of` from `crate::json_compare`.
use crate::json_compare::{
    Comparable,
    comparable_of,
};
/// Imports `JsoncEntry`,
/// `JsoncKind`,
/// `JsoncSpan`,
/// `JsoncSpans`,
/// `JsoncValue`,
/// `units_to_string` from `monochromatic_jsonc_edit`.
use monochromatic_jsonc_edit::{
    JsoncEntry,
    JsoncKind,
    JsoncSpan,
    JsoncSpans,
    JsoncValue,
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
///  How far a walk has read each span list.
/// Why:
///  The parser records spans in pre-order,
/// so one counter per list is enough to pair every node with its span.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class SpanCursor { values = 0; keys = 0 }
/// ```
pub(crate) struct SpanCursor {
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
    /// What:
    ///  A counter pair that has read nothing yet.
    /// Why:
    ///  A caller starts one walk per address it asks for,
    /// and the counters are this module's to keep.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static new(): SpanCursor
    /// ```
    pub(crate) fn new() -> SpanCursor {
        return SpanCursor {
            values: 0,
            keys: 0,
        };
    }

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
pub(crate) fn locate<'a>(
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
