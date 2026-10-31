//! What:
//!  Convert a parsed JSONC config into a plain `serde_json::Value` and a table of where each node sits.
//! Why:
//!  Duplicate keys, unpaired surrogates and unrepresentable numbers cannot live in a `serde_json::Value`,
//!  so they are rejected here, each with `file:line:col` and the JSON pointer of the node.
//!  The table lets the decoder give every later problem the same position information.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function convertDocument(spanned: JsoncSpanned, displayPath: string): [unknown, ConfigLocations]; // throws Diagnostic
//! ```

/// What:
///  Imports the location table types and the pointer escape.
/// Why:
///  The converter builds the table while it builds the value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ConfigLocations, Location, LocationNode, pointerSegment } from './configLocations';
/// ```
use crate::config_locations::{ConfigLocations, Location, LocationNode, pointer_segment};
/// What:
///  Imports `Diagnostic` from `crate::diagnostic`.
/// Why:
///  Every rejection is a `Diagnostic` with a code and a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Diagnostic } from './diagnostic';
/// ```
use crate::diagnostic::Diagnostic;
/// What:
///  Imports the JSONC tree, span list and span type from `monochromatic_jsonc_edit`.
/// Why:
///  The converter walks the tree and the span lists together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { JsoncKind, JsoncSpan, JsoncSpans, JsoncValue } from 'monochromatic-jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{JsoncKind, JsoncSpan, JsoncSpans, JsoncValue};
/// What:
///  Imports `HashMap` from `std::collections`.
/// Why:
///  Each object node keeps a key-to-index table for lookups by pointer segment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Map is global in TS
/// ```
use std::collections::HashMap;
/// What:
///  Imports `Map`, `Number`, `Value` from `serde_json`.
/// Why:
///  They are the plain JSON value the decoder reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // JSON values are plain JS objects in TS
/// ```
use serde_json::{Map, Number, Value};

/// What:
///  The location of the start of a span.
///  `JsoncSpan` is `Copy`, so it is passed by value.
/// Why:
///  Messages and the table point at where a token starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function startOf(span: JsoncSpan): Location { return { line: span.start.line, column: span.start.column }; }
/// ```
fn start_of(span: JsoncSpan) -> Location {
    return Location { line: span.start.line, column: span.start.column };
}

/// What:
///  Format a decode message as `file:line:col: /pointer: message`.
///  An empty pointer (the document root) is shown as `/`.
/// Why:
///  One format for every decode problem lets a person click or search the location and read the pointer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function located(display: string, at: Location, pointer: string, message: string): string
/// ```
pub fn located_message(display_path: &str, at: Location, pointer: &str, message: &str) -> String {
    let shown = if pointer.is_empty() { "/" } else { pointer };
    return format!("{display_path}:{}:{}: {shown}: {message}", at.line, at.column);
}

/// What:
///  The walk state: the spans still to be consumed, in document order, and the file name for messages.
///  `&'a JsoncSpans` is a borrow that must outlive the walk.
/// Why:
///  The two counters are the consumer contract of `JsoncSpans`: one per list, advanced in pre-order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Converter { constructor(spans: JsoncSpans, displayPath: string) {} valueIndex = 0; keyIndex = 0; }
/// ```
struct Converter<'a> {
    /// Spans recorded by the parse.
    spans: &'a JsoncSpans,
    /// Config path shown in messages.
    display_path: &'a str,
    /// Next unread entry of `spans.values`.
    value_index: usize,
    /// Next unread entry of `spans.keys`.
    key_index: usize,
}

/// What:
///  The conversion steps.
/// Why:
///  A recursive walk is fine here: the parser already refuses more than 512 nested containers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class Converter { convert(value: JsoncValue, pointer: string): [unknown, LocationNode] }
/// ```
impl Converter<'_> {
    /// What:
    ///  Take the next value span.
    /// Why:
    ///  Pre-order means a node takes its span before its children do.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextValueSpan(): JsoncSpan
    /// ```
    fn next_value_span(&mut self) -> JsoncSpan {
        let span = self.spans.values[self.value_index];
        self.value_index += 1;
        return span;
    }

    /// What:
    ///  Take the next key span.
    /// Why:
    ///  A member's key comes before its value in the walk.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// nextKeySpan(): JsoncSpan
    /// ```
    fn next_key_span(&mut self) -> JsoncSpan {
        let span = self.spans.keys[self.key_index];
        self.key_index += 1;
        return span;
    }

    /// What:
    ///  A `config-syntax` diagnostic at a location.
    /// Why:
    ///  All conversion failures share the code, the path and the message format.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// reject(at: Location, pointer: string, message: string): Diagnostic
    /// ```
    fn reject(&self, at: Location, pointer: &str, message: &str) -> Diagnostic {
        let full = located_message(self.display_path, at, pointer, message);
        return Diagnostic::new("config-syntax", &full).with_path(self.display_path);
    }

    /// What:
    ///  Convert one node and its children into a value and a location node.
    /// Why:
    ///  Duplicate keys, lone surrogates and unrepresentable numbers fail here, at the token that caused them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// convert(value: JsoncValue, pointer: string): [unknown, LocationNode]
    /// ```
    fn convert(&mut self, value: &JsoncValue, pointer: &str) -> Result<(Value, LocationNode), Diagnostic> {
        let here = start_of(self.next_value_span());
        match &value.kind {
            JsoncKind::Null => return Ok((Value::Null, scalar_node(here))),
            JsoncKind::Boolean { value: flag } => return Ok((Value::Bool(*flag), scalar_node(here))),
            JsoncKind::Text { units, .. } => {
                let Ok(text) = String::from_utf16(units) else {
                    return Err(self.reject(here, pointer, "the string holds an unpaired surrogate"));
                };
                return Ok((Value::String(text), scalar_node(here)));
            }
            JsoncKind::Number { raw, .. } => {
                let Some(number) = number_of(raw) else {
                    return Err(self.reject(here, pointer, &format!("the number {raw} is not representable")));
                };
                return Ok((Value::Number(number), scalar_node(here)));
            }
            JsoncKind::Array { elements } => {
                let mut items: Vec<Value> = Vec::new();
                let mut nodes: Vec<LocationNode> = Vec::new();
                for (index, element) in elements.iter().enumerate() {
                    let (converted, node) = self.convert(element, &format!("{pointer}/{index}"))?;
                    items.push(converted);
                    nodes.push(node);
                }
                return Ok((Value::Array(items), LocationNode { value: here, key: None, items: nodes, by_key: None }));
            }
            JsoncKind::Record { entries } => return self.convert_record(entries, here, pointer),
        }
    }

    /// What:
    ///  Convert an object's members, rejecting a key that appears twice or holds a lone surrogate.
    /// Why:
    ///  `serde_json::Map` would silently keep only the last of two equal keys.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// convertRecord(entries: JsoncEntry[], here: Location, pointer: string): [unknown, LocationNode]
    /// ```
    fn convert_record(
        &mut self,
        entries: &[monochromatic_jsonc_edit::JsoncEntry],
        here: Location,
        pointer: &str,
    ) -> Result<(Value, LocationNode), Diagnostic> {
        let mut object: Map<String, Value> = Map::new();
        let mut nodes: Vec<LocationNode> = Vec::new();
        let mut by_key: HashMap<String, usize> = HashMap::new();
        for entry in entries {
            let key_at = start_of(self.next_key_span());
            let Ok(key) = String::from_utf16(&entry.key.units) else {
                return Err(self.reject(key_at, pointer, "a key holds an unpaired surrogate"));
            };
            if object.contains_key(&key) {
                return Err(self.reject(key_at, pointer, &format!("the key {key:?} appears twice in this object")));
            }
            let child_pointer = format!("{pointer}/{}", pointer_segment(&key));
            let (converted, mut node) = self.convert(&entry.value, &child_pointer)?;
            node.key = Some(key_at);
            by_key.insert(key.clone(), nodes.len());
            nodes.push(node);
            object.insert(key, converted);
        }
        let table = LocationNode { value: here, key: None, items: nodes, by_key: Some(by_key) };
        return Ok((Value::Object(object), table));
    }
}

/// What:
///  A location node with no children.
/// Why:
///  Scalars have nothing below them to look up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scalarNode(at: Location): LocationNode
/// ```
fn scalar_node(at: Location) -> LocationNode {
    return LocationNode { value: at, key: None, items: Vec::new(), by_key: None };
}

/// What:
///  Read a number token as an integer when it is one, else as a finite float.
/// Why:
///  Integers keep exactness; `serde_json::Number` cannot hold NaN or infinity.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function numberOf(raw: string): number | undefined
/// ```
fn number_of(raw: &str) -> Option<Number> {
    if let Ok(integer) = raw.parse::<i64>() {
        return Some(Number::from(integer));
    }
    if let Ok(float) = raw.parse::<f64>() {
        return Number::from_f64(float);
    }
    return None;
}

/// What:
///  Convert a whole parsed document, pairing the value with its location table.
/// Why:
///  This is the only entry point the loader needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function convertDocument(root: JsoncValue, spans: JsoncSpans, displayPath: string): [unknown, ConfigLocations]
/// ```
pub fn convert_document(root: &JsoncValue, spans: &JsoncSpans, display_path: &str) -> Result<(Value, ConfigLocations), Diagnostic> {
    let mut converter = Converter { spans, display_path, value_index: 0, key_index: 0 };
    let (value, table) = converter.convert(root, "")?;
    return Ok((value, ConfigLocations { root: table }));
}
