//! What:
//!      The address type used to reach one key or element inside a parsed JSONC document.
//!           A path is a slice of segments (`&[JsoncPathSegment]`,
//!  a borrowed read-only view of a
//!           sequence,
//!  not an owned `Vec` or a fixed array),
//!  and each segment is either an object
//!           key or an array index.
//! Why:
//!       The read and edit API must address members without inventing a string syntax,
//!  so a
//!           caller states exactly which key or index it means and cannot be confused by a key
//!           that looks like a number.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module path: type JsoncPathSegment = { kind: 'key'; key: string } | { kind: 'index'; index: number };
//! ```

/// What:
///      One step of a document address:
///  an object key or an array index.
/// Why:
///       A single enum keeps the two address kinds distinct at the type level,
///  so an index can
///           never be silently compared against a key.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JsoncPathSegment = string | number;
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum JsoncPathSegment {
    /// Address one object member by its decoded key text.
    ///
    /// What:
    ///      holds the member's key text.
    /// Why:
    ///       object members are addressed by name,
    ///  and the decoded text is what a document
    ///           actually stores after escapes are resolved.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'key', key: string }
    /// ```
    Key {
        /// What:
        ///     Decoded key text of the addressed member.
        /// Why:
        ///      `key` stores the decoded text,
        ///  so lookups compare against the document's own
        ///          decoded keys rather than their escaped spellings.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// key: string;
        /// ```
        key: String,
    },
    /// Address one array element by its zero-based position.
    ///
    /// What:
    ///      holds the element position.
    /// Why:
    ///       array elements have no names,
    ///  so position is the only address,
    ///  and `usize` (not
    ///           `u32` or `isize`) matches Rust's own indexing type and cannot be negative.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// { kind: 'index', index: number }
    /// ```
    Index {
        /// What:
        ///     Zero-based position of the addressed element.
        /// Why:
        ///      `index` stores the position,
        ///  so callers cannot express a negative or fractional
        ///          element address.
        ///
        /// In TS you'd write (pseudocode):
        /// ```ts
        /// index: number;
        /// ```
        index: usize,
    },
}

/// What:
///      Questions a caller can ask about one address step.
/// Why:
///       Mixed paths need to reject an index used against an object,
///  and a key used against an
///           array,
///  without every call site repeating a pattern match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class JsoncPathSegment { isKey(): boolean; isIndex(): boolean }
/// ```
impl JsoncPathSegment {
    /// What:
    ///      Report whether this segment addresses an object member.
    /// Why:
    ///       Callers that accept mixed paths need to reject an index used against an object.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isKey(): boolean { return typeof this === 'string'; }
    /// ```
    pub fn is_key(&self) -> bool {
        // What:     `matches!` is the standard-library pattern test that returns a `bool`.
        // Why:      It reads as a question about the variant rather than a destructuring binding.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return typeof segment === 'string';
        // ```
        return matches!(self, JsoncPathSegment::Key { .. });
    }

    /// What:
    ///      Report whether this segment addresses an array element.
    /// Why:
    ///       The mirror of [`JsoncPathSegment::is_key`] for array targets.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// isIndex(): boolean { return typeof this === 'number'; }
    /// ```
    pub fn is_index(&self) -> bool {
        return matches!(self, JsoncPathSegment::Index { .. });
    }
}

/// What:
///      Build a path slice from borrowed key texts.
/// Why:
///       Most callers address object members only,
///  and repeating the enum constructor at every
///           call site hides the intent of the test or example.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function keyPath(...keys: string[]): JsoncPathSegment[];
/// ```
pub fn jsonc_key_path<'a>(keys: impl IntoIterator<Item = &'a str>) -> Vec<JsoncPathSegment> {
    // What:     `Vec<JsoncPathSegment>` owns the built address.
    // Why:      The returned address outlives the borrowed key texts it was built from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const path: JsoncPathSegment[] = [];
    // ```
    let mut path: Vec<JsoncPathSegment> = Vec::new();
    for key in keys {
        path.push(JsoncPathSegment::Key { key: String::from(key) });
    }
    return path;
}
