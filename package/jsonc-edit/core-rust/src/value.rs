//! What:
//!      The parsed JSONC document model:
//!  values,
//!  containers,
//!  object members,
//!  keys,
//!  numbers and
//!           attached comments.
//! Why:
//!       Comments are data in this crate rather than discarded trivia,
//!  so the model gives every
//!           key and every value its own optional comment slot,
//!  and every scalar keeps the source
//!           spelling it was written with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module value: type JsoncValue = { kind: JsoncKind; comment?: JsoncComment };
//! ```

/// What:
///      Import the failures a constructor can report.
/// Why:
///       Building a value from a raw token can hit malformed text or a malformed number,
///  and both are
///           ordinary results rather than panics.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncNumberError, JsoncParseError } from './error';
/// ```
use crate::error::{JsoncNumberError, JsoncParseError};
/// What:
///      Import the exact mathematical identity used by number values.
/// Why:
///       A number value must compare by value while its token text stays available for output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import type { JsoncNumberIdentity } from './number';
/// ```
use crate::number::JsoncNumberIdentity;
/// What:
///      Import the quoted-text conversions the string constructors need.
/// Why:
///       A constructed string value must hold decoded code units and a legal quoted spelling that
///           agree with each other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { decodeQuoted, encodeQuoted } from './textUnits';
/// ```
use crate::text_units::{decode_quoted, encode_quoted};

/// What:
///      How one attached comment was written in the source.
/// Why:
///       Canonical emission keeps a single-line comment trailing its value and moves a
///           multi-line comment above it,
///  so the style has to survive merging and reparsing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncCommentKind = 'inline' | 'block' | 'mixed';
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JsoncCommentKind {
    /// A `//` comment;
    ///  its text excludes the two slashes.
    Line,
    /// A `/* */` comment;
    ///  its text excludes both delimiters.
    Block,
    /// Several comments merged in source order,
    ///  spanning more than one style.
    Mixed,
}

/// What:
///      One normalized attached comment.
/// Why:
///       Callers query and replace comments without re-scanning source,
///  and an owned body
///           outlives the document text it came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncComment = { type: JsoncCommentKind; text: string };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JsoncComment {
    /// Style of this comment,
    ///  or `Mixed` after merging different styles.
    pub kind: JsoncCommentKind,
    /// Comment body with delimiters removed and no trimming applied.
    pub text: String,
}

/// What:
///      One object member's key:
///  decoded UTF-16 code units,
///  the original quoted spelling,
///  and
///           its own comment.
/// Why:
///       A key can hold an escaped unpaired surrogate that Rust `String` cannot represent,
///  and
///           its spelling must survive emission even when its comment is edited.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncKey = { value: string; raw: string; comment?: JsoncComment };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JsoncKey {
    /// What:
    ///      `Vec<u16>` owns the decoded UTF-16 code units.
    /// Why:
    ///       Unlike Rust `String`,
    ///  it can hold a lone surrogate such as `0xD800`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// units: number[]; // UTF-16 code units
    /// ```
    pub units: Vec<u16>,
    /// Original quoted spelling,
    ///  including both quote characters.
    pub raw: String,
    /// Comment attached to this key,
    ///  if the source had one.
    pub comment: Option<JsoncComment>,
}

/// What:
///      One object member:
///  its key and its value,
///  in source order.
/// Why:
///       A vector of members preserves order and keeps duplicate keys visible instead of
///           silently collapsing them the way a map would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncEntry = { key: JsoncKey; value: JsoncValue };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JsoncEntry {
    /// Comment-bearing member key.
    pub key: JsoncKey,
    /// Comment-bearing member value.
    pub value: JsoncValue,
}

/// What:
///      The payload of one JSONC value,
///  without its comment.
/// Why:
///       Separating payload from comment lets an edit replace a value while its comment stays
///           attached,
///  which is the behavior the maintained TypeScript package guarantees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncKind = { kind: 'string'; value: string; raw: string } | { kind: 'number'; ... } | ...;
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum JsoncKind {
    /// A quoted string:
    ///  decoded UTF-16 units plus the original quoted spelling.
    Text {
        /// Decoded code units,
        ///  which may include lone surrogates.
        units: Vec<u16>,
        /// Original quoted spelling,
        ///  including both quote characters.
        raw: String,
    },
    /// A number:
    ///  the original token plus its exact mathematical identity.
    Number {
        /// Original token text,
        ///  emitted unchanged when the value was not edited.
        raw: String,
        /// Canonical identity that makes `1`,
        ///  `1.0` and `1e0` compare equal.
        identity: JsoncNumberIdentity,
    },
    /// A `true` or `false` literal.
    ///
    /// What:
    ///      holds the literal's truth value.
    /// Why:
    ///       a named field keeps this variant self-describing at construction and match sites,
    ///           matching the other variants that already carry names.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'boolean', value: boolean }
    /// ```
    Boolean {
        /// What:
        ///     Truth value of the literal.
        /// Why:
        ///      `value` stores the parsed literal,
        ///  so emission and comparison read it by name.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// value: boolean;
        /// ```
        value: bool,
    },
    /// A `null` literal.
    Null,
    /// An array of comment-bearing elements in source order.
    ///
    /// What:
    ///      holds every element,
    ///  each with its own comment slot.
    /// Why:
    ///       order is document data in JSONC,
    ///  and per-element comments must survive edits.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'array', elements: JsoncValue[] }
    /// ```
    Array {
        /// What:
        ///     Elements in source order.
        /// Why:
        ///      `elements` stores the ordered children,
        ///  so emission and indexing agree with the
        ///          document the caller parsed.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// elements: JsoncValue[];
        /// ```
        elements: Vec<JsoncValue>,
    },
    /// An object of comment-bearing members in source order.
    ///
    /// What:
    ///      holds every member,
    ///  each with a key comment and a value comment.
    /// Why:
    ///       a vector keeps duplicates visible and preserves author order,
    ///  which a map would not.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'record', entries: JsoncEntry[] }
    /// ```
    Record {
        /// What:
        ///     Members in source order.
        /// Why:
        ///      `entries` stores the ordered pairs,
        ///  so lookups can pick the last duplicate the way
        ///          the maintained TypeScript package does.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// entries: JsoncEntry[];
        /// ```
        entries: Vec<JsoncEntry>,
    },
}

/// What:
///      One JSONC value:
///  a payload and at most one attached comment.
/// Why:
///       This is the unit the read,
///  edit and emit surfaces all exchange,
///  so a caller can hold a
///           document,
///  derive a new one from an edit,
///  and keep using the old one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncValue = { kind: JsoncKind; comment?: JsoncComment };
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct JsoncValue {
    /// Payload of this value.
    pub kind: JsoncKind,
    /// Comment attached to this value,
    ///  if the source had one.
    pub comment: Option<JsoncComment>,
}

/// What:
///      Read-only questions about one parsed value.
/// Why:
///       Callers ask for members or elements by name instead of matching on the payload enum at
///           every site,
///  which keeps the container shape in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncValue { entries(): JsoncEntry[] | undefined; elements(): JsoncValue[] | undefined }
/// ```
impl JsoncValue {
    /// What:
    ///      Borrow the members of a record value.
    /// Why:
    ///       Readers and editors need the ordered members without matching on the payload enum at
    ///           every call site.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// entries(): JsoncEntry[] | undefined;
    /// ```
    pub fn entries(&self) -> Option<&[JsoncEntry]> {
        // What:     An `if let` narrows the payload to its record variant.
        // Why:      Any other payload has no members, so the answer is absence rather than an error.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return this.kind.kind === 'record' ? this.kind.entries : undefined;
        // ```
        if let JsoncKind::Record { entries } = &self.kind {
            return Some(entries.as_slice());
        }
        return None;
    }

    /// What:
    ///      Borrow the elements of an array value.
    /// Why:
    ///       The mirror of [`JsoncValue::entries`] for array targets.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// elements(): JsoncValue[] | undefined;
    /// ```
    pub fn elements(&self) -> Option<&[JsoncValue]> {
        if let JsoncKind::Array { elements } = &self.kind {
            return Some(elements.as_slice());
        }
        return None;
    }
}

/// What:
///      Constructors for the document model.
/// Why:
///       An edit replaces a value with a new one,
///  and building that value by hand would force every
///           caller to repeat escape decoding,
///  number validation and comment-slot initialization.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncValue { static null(): JsoncValue; static text(raw: string): JsoncValue }
/// ```
impl JsoncValue {
    /// What:
    ///      Build a `null` value with no comment.
    /// Why:
    ///       Callers set absent values explicitly rather than reaching into the payload enum.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static null(): JsoncValue { return { kind: { kind: 'null' } }; }
    /// ```
    pub fn null() -> JsoncValue {
        return JsoncValue { kind: JsoncKind::Null, comment: None };
    }

    /// What:
    ///      Build a boolean value with no comment.
    /// Why:
    ///       A replacement value must carry the same shape the parser would have produced.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static boolean(value: boolean): JsoncValue;
    /// ```
    pub fn boolean(value: bool) -> JsoncValue {
        return JsoncValue { kind: JsoncKind::Boolean { value }, comment: None };
    }

    /// What:
    ///      Build a string value from one already-quoted JSON token.
    /// Why:
    ///       Keeping the caller's spelling preserves escapes exactly,
    ///  including a lone surrogate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static textFromToken(raw: string): JsoncValue;
    /// ```
    ///
    /// # Errors
    /// Returns the scanner's failure when the token is not one complete quoted JSON string.
    pub fn text_from_token(raw: &str) -> Result<JsoncValue, JsoncParseError> {
        let units = decode_quoted(raw)?;
        return Ok(JsoncValue {
            kind: JsoncKind::Text { units, raw: String::from(raw) },
            comment: None,
        });
    }

    /// What:
    ///      Build a string value from decoded code units,
    ///  deriving its quoted spelling.
    /// Why:
    ///       A caller holding text (possibly with a lone surrogate) should not have to write JSON
    ///           escapes by hand.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static textFromUnits(units: number[]): JsoncValue;
    /// ```
    pub fn text_from_units(units: Vec<u16>) -> JsoncValue {
        let raw = encode_quoted(&units);
        return JsoncValue { kind: JsoncKind::Text { units, raw }, comment: None };
    }

    /// What:
    ///      Build a number value from one JSON number token.
    /// Why:
    ///       The token is validated and normalized once,
    ///  so an edited document cannot hold a number
    ///           that compares inconsistently or emits as invalid JSON.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static numberFromToken(token: string): JsoncValue;
    /// ```
    ///
    /// # Errors
    /// Returns which grammar part the token violates.
    pub fn number_from_token(token: &str) -> Result<JsoncValue, JsoncNumberError> {
        let identity = JsoncNumberIdentity::from_token(token)?;
        return Ok(JsoncValue {
            kind: JsoncKind::Number { raw: String::from(token), identity },
            comment: None,
        });
    }

    /// What:
    ///      Build an array value from its elements.
    /// Why:
    ///       Element order and per-element comments are document data,
    ///  so the caller supplies them
    ///           already assembled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static array(elements: JsoncValue[]): JsoncValue;
    /// ```
    pub fn array(elements: Vec<JsoncValue>) -> JsoncValue {
        return JsoncValue { kind: JsoncKind::Array { elements }, comment: None };
    }

    /// What:
    ///      Build a record value from its members.
    /// Why:
    ///       Members keep their own key and value comments,
    ///  which a plain map could not express.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static record(entries: JsoncEntry[]): JsoncValue;
    /// ```
    pub fn record(entries: Vec<JsoncEntry>) -> JsoncValue {
        return JsoncValue { kind: JsoncKind::Record { entries }, comment: None };
    }

    /// What:
    ///      Read a string value's decoded code units.
    /// Why:
    ///       A caller that wants Rust text needs the units first,
    ///  because they may hold a lone
    ///           surrogate that `String` cannot represent.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// textUnits(): number[] | undefined;
    /// ```
    pub fn text_units(&self) -> Option<&[u16]> {
        if let JsoncKind::Text { units, .. } = &self.kind {
            return Some(units.as_slice());
        }
        return None;
    }

    /// What:
    ///      Read a number value's original token text.
    /// Why:
    ///       Unedited literals are emitted exactly as written,
    ///  so callers need that spelling.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// numberToken(): string | undefined;
    /// ```
    pub fn number_token(&self) -> Option<&str> {
        if let JsoncKind::Number { raw, .. } = &self.kind {
            return Some(raw.as_str());
        }
        return None;
    }
}

/// What:
///      Constructors for one object member key.
/// Why:
///       A key carries decoded units,
///  its quoted spelling and its own comment,
///  and an edit that
///           inserts a member must build all three consistently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncKey { static fromText(text: string): JsoncKey }
/// ```
impl JsoncKey {
    /// What:
    ///      Build a key from one already-quoted JSON token.
    /// Why:
    ///       An existing document's spelling survives,
    ///  including escapes and lone surrogates.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromToken(raw: string): JsoncKey;
    /// ```
    ///
    /// # Errors
    /// Returns the scanner's failure when the token is not one complete quoted JSON string.
    pub fn from_token(raw: &str) -> Result<JsoncKey, JsoncParseError> {
        let units = decode_quoted(raw)?;
        return Ok(JsoncKey { units, raw: String::from(raw), comment: None });
    }

    /// What:
    ///      Build a key from ordinary Rust text,
    ///  deriving its quoted spelling.
    /// Why:
    ///       Inserting a member by name is the common case,
    ///  and escaping should not be manual.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromText(text: string): JsoncKey;
    /// ```
    pub fn from_text(text: &str) -> JsoncKey {
        let units: Vec<u16> = text.encode_utf16().collect();
        let raw = encode_quoted(&units);
        return JsoncKey { units, raw, comment: None };
    }
}
