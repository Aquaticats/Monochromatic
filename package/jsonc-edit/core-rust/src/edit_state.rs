//! What:
//!      The immutable edit state:
//!  one parsed document plus the operations that derive a new one.
//! Why:
//!       Every edit returns a fresh state and leaves the previous state usable,
//!  which is the contract
//!           the maintained TypeScript package documents and the port must keep.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module editState: type JsoncEditState = { root: JsoncValue };
//! ```

/// What:
///      Import the canonical emitter.
/// Why:
///       Serializing a state is the same operation as serializing its root value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { emitJsoncValue } from './emit';
/// ```
use crate::emit::emit_jsonc_value;
/// What:
///      Import the source-positioned parse failure.
/// Why:
///       Parsing a document into a state can fail exactly where parsing a value can.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncParseError } from './error';
/// ```
use crate::error::JsoncParseError;
/// What:
///      Import the depth-bounded parser.
/// Why:
///       A state is built from one document and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc } from './parse';
/// ```
use crate::parse::parse_jsonc;
/// What:
///      Import the document value type.
/// Why:
///       The state holds one root value and hands out references to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncValue } from './value';
/// ```
use crate::value::JsoncValue;

/// What:
///      One parsed JSONC document held for reading and editing.
/// Why:
///       Naming the state separately from its root keeps the door open for cached or derived data
///           without changing call sites,
///  and it is the unit every edit returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncEditState = { root: JsoncValue };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JsoncEditState {
    /// The document's root value,
    ///  which is always a record or an array.
    pub root: JsoncValue,
}

/// What:
///      Parse one JSONC document into an editable state.
/// Why:
///       Callers edit documents rather than bare values,
///  and the state is what every edit returns.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseJsoncEdit(source: string): JsoncEditState;
/// ```
///
/// # Errors
/// Returns the parser's failure for malformed input,
///  a scalar root,
///  or a 513th container.
pub fn parse_jsonc_edit(source: &str) -> Result<JsoncEditState, JsoncParseError> {
    let root = parse_jsonc(source)?;
    return Ok(JsoncEditState { root });
}

/// What:
///      Build a state around an already-parsed root value.
/// Why:
///       A caller that parsed a value directly,
///  or built one from constructors,
///  can still use the
///           edit surface without reparsing text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncStateFromValue(root: JsoncValue): JsoncEditState;
/// ```
pub fn jsonc_state_from_value(root: JsoncValue) -> JsoncEditState {
    return JsoncEditState { root };
}

/// What:
///      Serialize one state to canonical JSONC text.
/// Why:
///       Output is normalized rather than byte-preserving,
///  so an unedited document round-trips to the
///           house style while comments stay attached to the same key or value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function jsoncStringify(state: JsoncEditState): string;
/// ```
pub fn jsonc_stringify(state: &JsoncEditState) -> String {
    return emit_jsonc_value(&state.root);
}
