//! What:
//!  Where each value and key of a config file sits,
//! looked up by JSON pointer.
//! Why:
//!  The decoder reads a plain `serde_json::Value`,
//! which has forgotten line and column;
//!  this table remembers them so every decode message can say `file:line:col` beside its pointer.
//!  Lookups cost the pointer's length,
//! so reporting many problems stays linear in the config.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ConfigLocations { valueAt(pointer: string): Location | undefined; keyAt(pointer: string): Location | undefined }
//! ```

/// What:
///  Imports `HashMap` from `std::collections`.
///  `HashMap<K, V>` is a key-to-value table (TS:
/// `Map<K, V>`;
/// sibling `BTreeMap` keeps keys sorted).
/// Why:
///  Finding an object member by key must not scan every sibling,
/// or a config with many keys would cost quadratic time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Map is global in TS
/// ```
use std::collections::HashMap;

/// What:
///  A one-based line and column in the config file.
/// Why:
///  People read `file:line:col`;
/// offsets are not what an editor jumps to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Location = { line: number; column: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    /// Line,
    /// counting from 1.
    pub line: usize,
    /// Column in Unicode scalar values,
    /// counting from 1.
    pub column: usize,
}

/// What:
///  Where one node of the config sits,
/// and where its children sit.
///  `Vec<LocationNode>` is a growable list of owned nodes (TS:
/// `LocationNode[]`);
///  `Option<HashMap<..>>` is `Some` only for objects.
/// Why:
///  The tree has the same shape as the JSON value,
/// so a pointer's segments walk it directly.
///  Arrays are reached by index;
/// objects through `by_key`,
/// which holds each key's index in `items`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LocationNode = { value: Location; key?: Location; items: LocationNode[]; byKey?: Map<string, number> };
/// ```
#[derive(Clone, Debug)]
pub struct LocationNode {
    /// Where this value starts.
    pub value: Location,
    /// Where this value's own key starts,
    /// when the value is an object member.
    pub key: Option<Location>,
    /// Array elements or object member values,
    /// in source order.
    pub items: Vec<LocationNode>,
    /// For an object,
    /// each key's index into `items`;
    /// `None` for arrays and scalars.
    pub by_key: Option<HashMap<String, usize>>,
}

/// What:
///  The location table of one parsed config.
/// Why:
///  The decoder's `Reader` borrows it to turn a pointer into a position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfigLocations = { root: LocationNode };
/// ```
#[derive(Clone, Debug)]
pub struct ConfigLocations {
    /// The document root.
    pub root: LocationNode,
}

/// What:
///  Escape one object key or array index for use as a JSON pointer segment (RFC 6901).
///  `~` becomes `~0` and `/` becomes `~1`.
/// Why:
///  A key that contains a slash would otherwise read as two segments and point at the wrong node.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pointerSegment(key: string): string { return key.replaceAll('~', '~0').replaceAll('/', '~1'); }
/// ```
pub fn pointer_segment(key: &str) -> String {
    return key.replace('~', "~0").replace('/', "~1");
}

/// What:
///  Undo [`pointer_segment`] for one segment.
/// Why:
///  Lookups compare against the keys as written in the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unescapeSegment(segment: string): string { return segment.replaceAll('~1', '/').replaceAll('~0', '~'); }
/// ```
fn unescape_segment(segment: &str) -> String {
    return segment.replace("~1", "/").replace("~0", "~");
}

/// What:
///  The lookups.
/// Why:
///  A pointer that no longer resolves (a key that is missing from the file) still has a useful position:
///  the deepest ancestor that does exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConfigLocations { valueAt(pointer: string): Location | undefined }
/// ```
impl ConfigLocations {
    /// What:
    ///  Walk a pointer from the root and return the node it reaches,
    /// or the deepest node it did reach.
    ///  The pair's boolean says whether the whole pointer resolved.
    /// Why:
    ///  Both lookups below share the walk;
    /// the boolean keeps "this exact node" apart from "the nearest ancestor".
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// walk(pointer: string): [LocationNode, boolean]
    /// ```
    fn walk(&self, pointer: &str) -> (&LocationNode, bool) {
        let mut node = &self.root;
        if pointer.is_empty() {
            return (node, true);
        }
        for raw in pointer.split('/').skip(1) {
            let segment = unescape_segment(raw);
            let next = match &node.by_key {
                Some(by_key) => by_key.get(&segment).and_then(|index| return node.items.get(*index)),
                None => segment.parse::<usize>().ok().and_then(|index| return node.items.get(index)),
            };
            let Some(found) = next else { return (node, false) };
            node = found;
        }
        return (node, true);
    }

    /// What:
    ///  The position where the value at `pointer` starts,
    ///  or where its nearest existing ancestor starts when the pointer names something absent.
    /// Why:
    ///  A "missing required key" message should point at the object that lacks it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// valueAt(pointer: string): Location
    /// ```
    pub fn value_at(&self, pointer: &str) -> Location {
        return self.walk(pointer).0.value;
    }

    /// What:
    ///  The position of the key of the member at `pointer`;
    ///  falls back to [`ConfigLocations::value_at`] when the member is absent or has no key (the root,
    /// an array element).
    /// Why:
    ///  An "unknown key" message should point at the key the person typed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// keyAt(pointer: string): Location
    /// ```
    pub fn key_at(&self, pointer: &str) -> Location {
        let (node, exact) = self.walk(pointer);
        if exact && let Some(key) = node.key {
            return key;
        }
        return node.value;
    }
}
