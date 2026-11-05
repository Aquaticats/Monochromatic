//! What:
//!  The conflicted files of a merge tree: the three stages of a path, the conflict they make, and the reading of `git merge-tree` output into conflicts.
//! Why:
//!  `merge_second_pass.rs` decides which conflicts the word pass may re-merge and stays under the file-size limit; reading the conflicts needs nothing from it, so it lives in a file of its own that the pass imports.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [tree, conflicts] = conflictsOfMergeTree(stdout);
//! ```

use std::collections::BTreeMap;

/// The stage numbers of the common ancestor, the first side and the second side.
const BASE_STAGE: u8 = 1;

const OURS_STAGE: u8 = 2;

const THEIRS_STAGE: u8 = 3;

/// One side of a conflicted path: its mode and object name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage {
    /// The file mode with its type bits.
    pub mode: u32,
    /// The object name, as hexadecimal text.
    pub oid: String,
}

/// A conflicted path with the stages Git recorded for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Conflict {
    /// The path, `/` separated, as bytes.
    pub path: Vec<u8>,
    /// The common ancestor, absent for a path both sides added.
    pub base: Option<Stage>,
    /// The first side, absent for a path it deleted.
    pub ours: Option<Stage>,
    /// The second side, absent for a path it deleted.
    pub theirs: Option<Stage>,
}

/// Whether one byte is the NUL byte that ends each field of a `-z` listing. `&u8` borrows the byte, as `split` hands it over.
pub fn is_nul_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// Whether one byte is the space that separates the words of a stage record. `&u8` borrows the byte, as `split` hands it over.
fn is_space_byte(byte: &u8) -> bool {
    return *byte == b' ';
}

/// What:
///  The conflicted paths a merge-tree answer lists, with their stages, in the order Git printed them.
/// Why:
///   `git merge-tree --write-tree -z` prints the tree, then one `<mode> <object> <stage>\t<path>` field per stage of every
///  conflicted path, then an empty field, then the messages. The name-only form does not carry the object names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function conflictsOfMergeTree(stdout: Uint8Array): { tree: string; conflicts: Conflict[] };
/// ```
pub fn conflicts_of_merge_tree(stdout: &[u8]) -> (String, Vec<Conflict>) {
    let mut fields = stdout.split(is_nul_byte);
    let tree: String = String::from_utf8_lossy(fields.next().unwrap_or_default()).into_owned();
    let mut by_path: BTreeMap<Vec<u8>, Conflict> = BTreeMap::new();
    let mut order: Vec<Vec<u8>> = Vec::new();
    for field in fields {
        if field.is_empty() {
            break;
        }
        let Some((facts, path)) = split_once(field, b'\t') else { continue };
        let words: Vec<&[u8]> = facts.split(is_space_byte).collect();
        let [mode_bytes, oid_bytes, stage_bytes] = words.as_slice() else { continue };
        let (Ok(mode_text), Ok(oid_text), Ok(stage_text)) = (std::str::from_utf8(mode_bytes), std::str::from_utf8(oid_bytes), std::str::from_utf8(stage_bytes)) else { continue };
        let (Ok(mode), Ok(stage)) = (u32::from_str_radix(mode_text, 8), stage_text.parse::<u8>()) else { continue };
        if !by_path.contains_key(path) {
            order.push(path.to_vec());
        }
        let conflict: &mut Conflict = match by_path.entry(path.to_vec()) {
            std::collections::btree_map::Entry::Occupied(known) => known.into_mut(),
            std::collections::btree_map::Entry::Vacant(slot) => slot.insert(Conflict { path: path.to_vec(), base: None, ours: None, theirs: None }),
        };
        let side: Option<Stage> = Some(Stage { mode, oid: oid_text.to_owned() });
        match stage {
            BASE_STAGE => conflict.base = side,
            OURS_STAGE => conflict.ours = side,
            THEIRS_STAGE => conflict.theirs = side,
            _ => {}
        }
    }
    let mut ordered: Vec<Conflict> = Vec::new();
    for path in order {
        if let Some(conflict) = by_path.remove(&path) {
            ordered.push(conflict);
        }
    }
    return (tree, ordered);
}

/// The text before the first `separator` and the text after it.
pub fn split_once(bytes: &[u8], separator: u8) -> Option<(&[u8], &[u8])> {
    let mut found: Option<usize> = None;
    for (position, byte) in bytes.iter().enumerate() {
        if *byte == separator {
            found = Some(position);
            break;
        }
    }
    let at: usize = found?;
    return Some((&bytes[..at], &bytes[at + 1..]));
}
