//! What:
//!      `monochromatic-jsonc-edit` parses JSONC into a comment-bearing document model,
//!  edits it
//!           immutably,
//!  and emits canonical JSONC.
//!           A Rust crate is closest to a TypeScript package module:
//!  it holds private implementation
//!           files and one public API surface,
//!  and the public items are re-exported here so callers
//!           write `monochromatic_jsonc_edit::parse_jsonc` instead of reaching into modules.
//! Why:
//!       The maintained TypeScript package `@monochromatic-dev/jsonc-edit-core-ts` treats comments
//!           as queryable data rather than discarded trivia,
//!  compares numbers by exact mathematical
//!           value while retaining unedited spelling,
//!  and keeps escaped unpaired UTF-16 surrogates.
//!           This crate carries the same supported behavior into native Rust with no dependencies and
//!           no `unsafe` block,
//!  as recorded in `doc/decision/jsonc-edit-rust-foundation.md`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { parseJsonc, jsoncStringify } from '@monochromatic-dev/jsonc-edit-core-ts';
//! ```

/// What:
///      The crate's public failure types.
/// Why:
///       Parse,
///  number,
///  address and shape failures answer different caller questions,
///  so each has
///           its own type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './errors';
/// ```
pub mod error;

/// What:
///      Decimal digit-string arithmetic for unbounded number exponents.
/// Why:
///       A JSON exponent may carry more digits than any machine integer holds,
///  and this crate
///           refuses to bound or materialize it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: decimal string addition and subtraction.
/// ```
mod number_scale;

/// What:
///      The exact mathematical identity of a JSON number literal.
/// Why:
///       `1`,
///  `1.0` and `1e0` must compare equal while adjacent large integers stay distinct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { JsoncNumberIdentity } from './number';
/// ```
pub mod number;

/// What:
///      Document addresses used by the read and edit surface.
/// Why:
///       A key and an array index are different questions,
///  and the type keeps them apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { JsoncPath } from './path';
/// ```
pub mod path;

/// What:
///      The parsed document model:
///  values,
///  members,
///  keys,
///  numbers and attached comments.
/// Why:
///       Every key and every value carries its own optional comment,
///  which is what makes the
///           comment surface queryable instead of merely preserved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { JsoncValue } from './value';
/// ```
pub mod value;

/// What:
///      The byte scanner for JSONC tokens,
///  strings,
///  numbers and comment trivia.
/// Why:
///       Scanning is linear over borrowed source,
///  with no regular expression and no recursion,
///  so
///           malformed input produces a positioned error instead of a stack or backtracking cost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: the scanner the parser drives.
/// ```
mod scan;

/// What:
///      Quoted-string conversion between JSON tokens and UTF-16 code units.
/// Why:
///       Escaped unpaired surrogates must survive parsing,
///  and replacement values must be written
///           back as legal JSON text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { decodeQuoted, encodeQuoted, unitsToString } from './textUnits';
/// ```
pub mod text_units;

/// What:
///      Comment attachment,
///  merging and canonical rendering helpers.
/// Why:
///       Stacked comments merge into one normalized comment per key or value,
///  and emission needs
///           the same style rules the parser recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { mergeComments } from './mergeComments';
/// ```
mod comment_merge;

/// What:
///      The iterative,
///  depth-bounded JSONC parser.
/// Why:
///       Valid 512-container documents must parse without exhausting the call stack,
///  and a 513th
///           opener must be rejected before deeper state exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseJsonc } from './parse';
/// ```
mod parse;

/// What:
///      The canonical JSONC emitter.
/// Why:
///       Output is normalized rather than byte-preserving,
///  so comments stay attached to the same
///           key or value across an emit-and-reparse cycle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { emitJsoncValue } from './stringify';
/// ```
mod emit;

/// What:
///      Re-export the failure types at the crate root.
/// Why:
///       Callers name `monochromatic_jsonc_edit::JsoncParseError` in signatures without importing
///           a module path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { JsoncParseError, JsoncPathNotFoundError, JsoncTypeError } from './errors';
/// ```
pub use error::{JsoncEditError, JsoncNumberError, JsoncParseError, JsoncPathNotFoundError, JsoncTypeError};
/// What:
///      Re-export the exact-number identity.
/// Why:
///       It appears inside the public value model,
///  so it is part of the API surface.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { JsoncNumberIdentity } from './number';
/// ```
pub use number::JsoncNumberIdentity;
/// What:
///      Re-export the address type and its key-path helper.
/// Why:
///       Read and edit calls take addresses,
///  so callers need the type at the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { jsoncKeyPath, type JsoncPathSegment } from './path';
/// ```
pub use path::{jsonc_key_path, JsoncPathSegment};
/// What:
///      Re-export the document model.
/// Why:
///       These are the types every caller holds,
///  inspects and edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { JsoncValue, JsoncKind, JsoncEntry, JsoncKey, JsoncComment } from './value';
/// ```
pub use value::{JsoncComment, JsoncCommentKind, JsoncEntry, JsoncKey, JsoncKind, JsoncValue};

/// What:
///      Re-export comment merging and the parse and emit entry points.
/// Why:
///       These three functions are the whole document lifecycle a caller needs before the edit
///           surface is added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { mergeComments, parseJsonc, emitJsoncValue } from './index';
/// ```
pub use comment_merge::attach as merge_comments;
/// What:
///      Re-export the canonical emitter.
/// Why:
///       Writing a document is half of the lifecycle,
///  and callers should not name a private module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { emitJsoncValue } from './stringify';
/// ```
pub use emit::emit_jsonc_value;
/// What:
///      Re-export the JSONC parser entry point.
/// Why:
///       Parsing is the other half of the lifecycle and the only way to obtain a document value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseJsonc } from './parse';
/// ```
pub use parse::parse_jsonc;
/// What:
///      Re-export the quoted-string conversions.
/// Why:
///       Callers that hold a raw token or stored code units need the same decoding and encoding
///           rules the parser and emitter use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { decodeQuoted, encodeQuoted, unitsToString } from './textUnits';
/// ```
pub use text_units::{decode_quoted, encode_quoted, units_to_string};

/// What:
///  Source positions (byte offset, line, column) for parsed values and object keys.
/// Why:
///  Callers that decode a document into their own types need `file:line:col` for a value they reject,
///  and the document tree itself stays position-free so existing constructors and comparisons keep working.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './span';
/// ```
mod span;
/// What:
///  Re-export the span types and the span-reporting parser entry point.
/// Why:
///  Positions are a parsing feature, so callers reach them from the crate root beside `parse_jsonc`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseJsoncWithSpans, jsoncPositionAt } from './span';
/// export type { JsoncPosition, JsoncSpan, JsoncSpans, JsoncSpanned } from './span';
/// ```
pub use span::{JsoncPosition, JsoncSpan, JsoncSpanned, JsoncSpans, jsonc_position_at, parse_jsonc_with_spans};

/// What:
///      Parser and emitter conformance tests over the public crate interface.
/// Why:
///       Comment ownership,
///  depth limits,
///  UTF-16 escapes and exact numbers must survive a parse,
///           emit and reparse cycle,
///  which compilation alone cannot show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './parse.test';
/// ```
#[cfg(test)]
mod parse_tests;

/// What:
///  Tests for the span-reporting parse entry point and the position rule.
/// Why:
///  Positions are only trustworthy if slicing the text between a span's offsets returns the node's own token,
///  which compilation cannot show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './span.test';
/// ```
#[cfg(test)]
mod span_tests;

/// What:
///      Exact-number identity tests,
///  including an independent rational oracle.
/// Why:
///       Equality across spellings and rejection of malformed tokens are the crate's numeric
///           contract,
///  and an oracle cross-checks the normalizer rather than itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './number.test';
/// ```
#[cfg(test)]
mod number_tests;

/// What:
///      Quoted-string conversion tests.
/// Why:
///       Lone surrogates,
///  control characters and escape spellings must survive decode,
///  convert and
///           re-encode without changing the code units.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './textUnits.test';
/// ```
#[cfg(test)]
mod text_units_tests;

/// What:
///      The immutable edit state plus its parse and serialize entry points.
/// Why:
///       A caller holds a state,
///  derives new states from edits,
///  and writes any of them out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseJsoncEdit, jsoncStringify, type JsoncEditState } from './editState';
/// ```
mod edit_state;

/// What:
///      Address resolution for reads,
///  existence checks and key listing.
/// Why:
///       Every surface walks addresses with the same last-duplicate and bounds rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { jsoncGetValue, jsoncHas, jsoncKeys } from './editNavigate';
/// ```
mod navigate;

/// What:
///      The iterative spine walk shared by every immutable edit.
/// Why:
///       A recursive rebuild overflows a debug test thread's stack at the accepted 512-container depth,
///           so descent and rebuild keep their own explicit level stack.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // private module: descend(root, path) then rebuild(levels, value).
/// ```
mod edit_spine;

/// What:
///      Immutable value replacement,
///  insertion and deletion.
/// Why:
///       Structural edits return a new document and leave the input state usable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { jsoncSet, jsoncDelete } from './editSet';
/// ```
mod edit_apply;

/// What:
///      The comment-as-data query and edit surface.
/// Why:
///       A value's comment and its key's comment are separate data,
///  and both are editable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { jsoncGetComment, jsoncSetComment, jsoncGetKeyComment, jsoncSetKeyComment } from './editComment';
/// ```
mod edit_comment;

/// What:
///      Re-export the edit state and its lifecycle functions.
/// Why:
///       These are the entry points a consumer needs to hold and serialize a document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { parseJsoncEdit, jsoncStringify, type JsoncEditState } from './index';
/// ```
pub use edit_state::{jsonc_state_from_value, jsonc_stringify, parse_jsonc_edit, JsoncEditState};

/// What:
///      Re-export the read surface.
/// Why:
///       Reads answer what a document holds without deriving a new one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { jsoncGetValue, jsoncHas, jsoncKeys } from './index';
/// ```
pub use navigate::{jsonc_has, jsonc_keys, jsonc_lookup};

/// What:
///      Re-export the structural edit surface.
/// Why:
///       Setting and deleting are the two operations that change document shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { jsoncSet, jsoncDelete } from './index';
/// ```
pub use edit_apply::{jsonc_delete, jsonc_set};

/// What:
///      Re-export the comment surface.
/// Why:
///       Comments are queryable and editable data,
///  not output decoration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { jsoncGetComment, jsoncSetComment, jsoncGetKeyComment, jsoncSetKeyComment } from './index';
/// ```
pub use edit_comment::{jsonc_comment, jsonc_key_comment, jsonc_set_comment, jsonc_set_key_comment};

/// What:
///      Read and structural edit tests.
/// Why:
///       Immutability,
///  insertion limits and the two failure kinds are the edit contract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './edit.test';
/// ```
#[cfg(test)]
mod edit_tests;

/// What:
///      Comment query and edit tests.
/// Why:
///       Key and value comments are separate data and must keep their owner across emission.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './editComment.test';
/// ```
#[cfg(test)]
mod edit_comment_tests;

/// What:
///      Public-surface tests for address helpers,
///  identity accessors,
///  comment merging and errors.
/// Why:
///       Every exported item needs coverage before the crate can be called complete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './api.test';
/// ```
#[cfg(test)]
mod api_tests;

/// What:
///      Shared-fixture tests read from the same corpus the TypeScript conformance suite uses.
/// Why:
///       One language-neutral contract keeps both maintained implementations from drifting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './fixtures.test';
/// ```
#[cfg(test)]
mod fixture_tests;
