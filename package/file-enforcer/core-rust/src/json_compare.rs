//! What:
//!  What a parsed JSON value compares as,
//! and whether a config value already equals it.
//! Why:
//!  A comparison must use the decoded text and not the source spelling,
//! so a conforming manifest is left byte-identical;
//! the comparison lives in its own file so the splice and it each stay inside the size one file may reach.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // private module: type Comparable; comparableOf(value); valueMatches(found, wanted); tokenText(value)
//! ```

/// Imports `Evaluated` from `crate::value_eval`.
use crate::value_eval::Evaluated;
/// Imports `JsoncKind`,
/// `JsoncValue`,
/// `encode_quoted`,
/// `units_to_string` from `monochromatic_jsonc_edit`.
use monochromatic_jsonc_edit::{
    JsoncKind,
    JsoncValue,
    encode_quoted,
    units_to_string,
};

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
pub fn comparable_of(value: &JsoncValue) -> Comparable {
    match &value.kind {
        JsoncKind::Text {
            units,
            ..
        } => match units_to_string(units) {
            Ok(text) => return Comparable::Text(text),
            Err(_) => return Comparable::Other,
        },
        JsoncKind::Boolean {
            value: flag,
        } => return Comparable::Bool(*flag),
        JsoncKind::Number {
            raw,
            ..
        } => match raw.parse::<i64>() {
            Ok(number) => return Comparable::Integer(number),
            Err(_) => return Comparable::Other,
        },
        JsoncKind::Array {
            elements,
        } => {
            let mut items: Vec<Comparable> = Vec::new();
            for element in elements {
                items.push(comparable_of(element));
            }
            return Comparable::List(items);
        }
        JsoncKind::Null
        | JsoncKind::Record {
            ..
        } => return Comparable::Other,
    }
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
pub fn value_matches(
    found: &Comparable,
    wanted: &Evaluated,
) -> bool {
    match (found, wanted) {
        (Comparable::Text(written), Evaluated::Text(asked)) => return written == asked,
        (Comparable::Bool(written), Evaluated::Bool(asked)) => return written == asked,
        (Comparable::Integer(written), Evaluated::Integer(asked)) => return written == asked,
        (Comparable::List(written), Evaluated::List(asked)) => {
            if written.len() != asked.len() {
                return false;
            }
            for (index, item) in asked.iter().enumerate() {
                let Some(element) = written.get(index) else {
                    return false;
                };
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
