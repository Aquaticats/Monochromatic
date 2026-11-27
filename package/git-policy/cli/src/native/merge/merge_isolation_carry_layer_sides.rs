//! What:
//!  The three sides of one layer of one path the merge changes:
//!  the mode and the object each side holds,
//!  and the mode the layer ends with.
//! Why:
//!   The mode says what kind of entry a side holds,
//!  so a link and a file whose bytes are the link's target are told apart,
//!  and a three-way choice over the three modes is the mode the layer ends with;
//!  the decision that reads a layer and the layer's own end both ask these rules.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ended = endMode(layer); // theSameEntry(left, right) says a side changed nothing
//! ```

/// The Git mode of a symbolic link,
/// which is not a kind of entry the text merge reads.
pub(super) const SYMBOLIC_LINK_MODE: &str = "120000";

/// The Git mode of a submodule,
/// whose entry is a commit rather than text.
pub(super) const SUBMODULE_MODE: &str = "160000";

/// One side of one layer:
/// the mode Git records and the object it holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Side = { mode: string; oid: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Side {
    /// `100644`,
    /// `100755` or `120000`.
    pub mode: String,
    /// The object name.
    pub oid: String,
}

/// The three sides of one layer of one path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Layer = { base: Side | null; ours: Side | null; theirs: Side | null };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Layer {
    /// What the path held before the merge.
    pub base: Option<Side>,
    /// What the person's layer held.
    pub ours: Option<Side>,
    /// What the merge makes of the path.
    pub theirs: Option<Side>,
}

/// The kind of entry one side holds,
/// which its mode records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kindOf = (mode) => mode === '120000' ? 'link' : mode === '160000' ? 'submodule' : 'file';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
    /// A regular file,
    /// whose executable bit is a mode of its own but a kind of the same entry.
    File,
    /// A symbolic link,
    /// whose object is its target's text.
    Link,
    /// A submodule,
    /// whose object is a commit.
    Submodule,
}

/// The kind of entry a mode records.
pub(super) fn the_kind_of(mode: &str) -> Kind {
    return match mode {
        SYMBOLIC_LINK_MODE => Kind::Link,
        SUBMODULE_MODE => Kind::Submodule,
        _ => Kind::File,
    };
}

/// What:
///  Whether one side holds what the other does,
/// as the merge reads a side that changed nothing.
/// Why:
///   An object id says what bytes an entry holds,
/// and a mode says what kind of entry holds them;
///  a link
/// to `keep.txt` and a file whose bytes are `keep.txt` share an object id,
/// so comparing ids alone read a
///  person's link as the branch's own file and wrote the merge's bytes into the link's target
/// (a cross-vendor
///  review,
/// round 133,
/// its third finding).
///  An executable bit changed alone is the same kind,
/// so a mode the person set is still read as unchanged
///  bytes and stays theirs;
/// a path the merge keeps is left as Git's own update wrote it,
///  with Git's mode,
/// and a path the merge deletes goes with the merge
///  (`SPEC.md`,
///  "Carry edits forward").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const equal = (a, b) => (a === null && b === null) || (a !== null && b !== null && a.oid === b.oid && kindOf(a.mode) === kindOf(b.mode));
/// ```
pub(super) fn the_same_entry(
    left: &Option<Side>,
    right: &Option<Side>,
) -> bool {
    return match (left, right) {
        (None, None) => true,
        (Some(one), Some(other)) => {
            one.oid == other.oid
                && the_kind_of(one.mode.as_str()) == the_kind_of(other.mode.as_str())
        }
        _ => false,
    };
}

/// What:
///  The mode a side holds.
/// Why:
///   The mode is one field of a side,
///  and both the layer's end and its stage read it.
pub(super) fn the_mode_of(found: &Side) -> String {
    return found.mode.clone();
}

/// The mode of one side,
/// or nothing when the side holds no entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const modeOf = (side) => side === null ? null : side.mode;
/// ```
pub(super) fn mode_of(side: &Option<Side>) -> Option<String> {
    return side.as_ref().map(the_mode_of);
}

/// What:
///  The mode a side records,
/// for the base's own mode.
/// Why:
///   The base is compared with each side by mode alone,
///  so only the mode is read from it.
pub(super) fn base_mode(side: &Side) -> &str {
    return side.mode.as_str();
}

/// What:
///  The mode one layer ends with,
/// as a three-way choice over the three modes.
/// Why:
///  A mode is one bit,
/// so a side equal to the base changed nothing,
/// and the side that did change it wins;
/// two sides that changed it agree,
/// and the person's is the one left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endMode(layer): string | null; // base === ours ? theirs : base === theirs ? ours : ours
/// ```
pub(super) fn end_mode(layer: &Layer) -> Option<String> {
    let (Some(ours), Some(theirs)) = (&layer.ours, &layer.theirs) else {
        return match mode_of(&layer.ours) {
            Some(mode) => Some(mode),
            None => mode_of(&layer.theirs),
        };
    };
    let base: Option<&str> = layer.base.as_ref().map(base_mode);
    if base == Some(ours.mode.as_str()) {
        return Some(theirs.mode.clone());
    }
    return Some(ours.mode.clone());
}
