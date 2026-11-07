//! What:
//!  Git's `core.fsync` value read into the set of components it flushes,
//! and the value that keeps every component of a set
//!  except the pack's own.
//! Why:
//!  Git flushes only the components `core.fsync` names,
//! and the wrapper asks the same question for two writes it makes
//!  itself:
//! whether the index it writes in process is flushed,
//! and whether the pack that `git fast-import` writes for a
//!  landing is flushed.
//! Read from Git 2.56.0 (`environment.c`,
//! `parse_fsync_components`;
//! `write-or-die.h`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const set = parseFsyncComponents('added,-pack'); // a bit set of components
//! set.has('index'); // true
//! ```

/// A component of the repository that Git can flush,
/// one bit each as in Git's `enum fsync_component`.
const LOOSE_OBJECT: u8 = 1 << 0;
/// A pack file.
const PACK: u8 = 1 << 1;
/// A pack's index and reverse index.
const PACK_METADATA: u8 = 1 << 2;
/// The commit-graph files.
const COMMIT_GRAPH: u8 = 1 << 3;
/// The index file.
const INDEX: u8 = 1 << 4;
/// The files of the reference store.
const REFERENCE: u8 = 1 << 5;
/// The mapping between hash algorithms;
/// no `core.fsync` word names it alone,
/// only `all`.
const OBJECT_MAP: u8 = 1 << 6;

/// What `objects` names.
const OBJECTS: u8 = LOOSE_OBJECT | PACK;
/// What `derived-metadata` names.
const DERIVED_METADATA: u8 = PACK_METADATA | COMMIT_GRAPH;
/// What Git flushes when `core.fsync` is not set:
/// the objects and the derived metadata,
/// but not the loose objects.
const DEFAULT: u8 = (OBJECTS | DERIVED_METADATA) & !LOOSE_OBJECT;
/// What `committed` names.
const COMMITTED: u8 = OBJECTS | REFERENCE;
/// What `added` names.
const ADDED: u8 = COMMITTED | INDEX;
/// What `all` names.
const ALL: u8 = LOOSE_OBJECT | PACK | PACK_METADATA | COMMIT_GRAPH | INDEX | REFERENCE | OBJECT_MAP;

/// The words `core.fsync` accepts,
/// in the order Git lists them,
/// with the components each names.
const WORDS: [(&str, u8); 11] = [
    ("loose-object", LOOSE_OBJECT),
    ("pack", PACK),
    ("pack-metadata", PACK_METADATA),
    ("commit-graph", COMMIT_GRAPH),
    ("index", INDEX),
    ("objects", OBJECTS),
    ("reference", REFERENCE),
    ("derived-metadata", DERIVED_METADATA),
    ("committed", COMMITTED),
    ("added", ADDED),
    ("all", ALL),
];

/// The separator characters Git skips before a word.
const SEPARATORS: [char; 5] = [',', ' ', '\t', '\n', '\r'];

/// What:
///  The components one `core.fsync` value makes Git flush.
/// Why:
///   One value type for both questions,
/// so the index writer and the import step cannot disagree about how Git reads the
///  setting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FsyncComponents = { bits: number };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FsyncComponents {
    /// The bits of the components that are flushed.
    bits: u8,
}

/// Reading and asking.
impl FsyncComponents {
    /// What:
    ///  The set Git flushes when `core.fsync` is not set.
    /// Why:
    ///   An unset key is the default,
    /// which flushes packs and their metadata but not loose objects,
    /// references or the index.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// FsyncComponents.default()
    /// ```
    pub fn default_set() -> Self {
        return Self { bits: DEFAULT };
    }

    /// What:
    ///  The set one `core.fsync` value makes Git flush.
    /// Why:
    ///   The value starts again from the default at each entry.
    /// In one value `none` (only as the whole rest of the value)
    ///  clears the set,
    /// a word with a `-` is removed and any other is added,
    /// a word names every component whose name it is a
    ///  prefix of (`ind` is `index`,
    /// `pack` also names `pack-metadata`),
    /// a word that names none is ignored,
    /// and the result
    ///  is the set without the removed components plus the added ones,
    /// so an added component stays even when it is also
    ///  removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// FsyncComponents.parse('added,-pack')
    /// ```
    pub fn parse(value: &str) -> Self {
        let mut current: u8 = DEFAULT;
        let (mut positive, mut negative): (u8, u8) = (0, 0);
        let mut rest: &str = value;
        loop {
            rest = rest.trim_start_matches(SEPARATORS);
            let end: usize = rest.find(',').unwrap_or(rest.len());
            if rest == "none" {
                current = 0;
            } else {
                let (negated, word): (bool, &str) = match rest[..end].strip_prefix('-') {
                    Some(removed) => (true, removed),
                    None => (false, &rest[..end]),
                };
                // Git stops at an empty word, which is also how the end of the value reads.
                if word.is_empty() {
                    break;
                }
                for (name, bits) in WORDS {
                    if name.starts_with(word) {
                        if negated {
                            negative |= bits;
                        } else {
                            positive |= bits;
                        }
                    }
                }
            }
            if end >= rest.len() {
                break;
            }
            rest = &rest[end..];
        }
        return Self { bits: (current & !negative) | positive };
    }

    /// Whether loose objects are flushed.
    pub fn names_loose_object(self) -> bool {
        return self.bits & LOOSE_OBJECT != 0;
    }

    /// Whether the index file is flushed.
    pub fn names_index(self) -> bool {
        return self.bits & INDEX != 0;
    }

    /// Whether a pack file or a pack's metadata is flushed.
    pub fn names_pack(self) -> bool {
        return self.bits & (PACK | PACK_METADATA) != 0;
    }

    /// Whether both a pack file and a pack's metadata are flushed.
    pub fn names_pack_and_metadata(self) -> bool {
        return self.bits & (PACK | PACK_METADATA) == PACK | PACK_METADATA;
    }

    /// Whether the files of the reference store are flushed.
    pub fn names_reference(self) -> bool {
        return self.bits & REFERENCE != 0;
    }

    /// What:
    ///  This set without the pack file and the pack's metadata.
    /// Why:
    ///   The import step of a landing writes a temporary pack that it deletes when the batch is small,
    /// and native Git writes
    ///  loose objects,
    /// so a repository that does not flush loose objects gets no pack flush for it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// set.without('pack', 'pack-metadata')
    /// ```
    pub fn without_packs(self) -> Self {
        return Self { bits: self.bits & !(PACK | PACK_METADATA) };
    }

    /// What:
    ///  This set with the pack file and the pack's metadata added.
    /// Why:
    ///   Where loose objects are flushed,
    /// a pack that the import keeps instead of unpacking holds the same objects and is
    ///  flushed with its index.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// set.with('pack', 'pack-metadata')
    /// ```
    pub fn with_packs(self) -> Self {
        return Self { bits: self.bits | PACK | PACK_METADATA };
    }

    /// What:
    ///  A `core.fsync` value that makes Git flush exactly this set.
    /// Why:
    ///   Each word is spelled in full,
    /// because a word is a prefix match (`pack` alone would also name `pack-metadata`),
    /// and
    ///  `none` ends the value,
    /// which starts the set empty.
    /// A component that no single word names (the mapping between hash
    ///  algorithms) is left out.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// set.toValue() // 'commit-graph,reference,none'
    /// ```
    pub fn value(self) -> String {
        let mut words: Vec<&str> = Vec::new();
        for (name, bits) in WORDS {
            // Only the words that name one component are spelled; the grouped words would bring back what is not in the set.
            if bits.is_power_of_two() && self.bits & bits != 0 {
                words.push(name);
            }
        }
        words.push("none");
        return words.join(",");
    }
}

/// Controls for reading the value,
/// each against real Git's flushes.
#[cfg(test)]
#[path = "config_fsync_tests.rs"]
mod tests;
