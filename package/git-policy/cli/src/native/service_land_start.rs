//! What:
//!  The index entries a landing started from, recorded in its intent, and the decision of a replay from them.
//! Why:
//!  An index update that never installed is replayed by a restart after the lock was given up, and another session may have
//!  staged into the same paths since. The replay then resets only the paths whose entries are still the ones the landing
//!  started from, and leaves every other path as that session left it, because a path staged since is user data
//!  (`doc/planning/cli-git-main-commit-within-native-plus-five.md`, "A replay that keeps later staging").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pathsToReplay = unchangedPaths(recorded, current, names) // the paths still at the entry the landing started from
//! ```

use super::repo_read_object::to_hex;

/// One index entry of a path, as far as a replay compares it: stat data is left out, since a later `git add` of the same
///  content changes only that.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartEntry {
    /// The path, as bytes.
    pub path: Vec<u8>,
    /// The merge stage, 0 for a resolved path.
    pub stage: u8,
    /// The file mode with its type bits.
    pub mode: u32,
    /// The object name, in hexadecimal.
    pub id: String,
    /// The extended flags (intent-to-add, skip-worktree).
    pub extended: u16,
}

/// The text of a list of entries: items separated by tabs, the fields of an item by colons, the path in hexadecimal so that
///  no byte of a name can be taken for a separator.
pub fn encode(entries: &[StartEntry]) -> String {
    return entries
        .iter()
        .map(|entry| { return format!("{}:{}:{:o}:{}:{}", to_hex(entry.path.as_slice()), entry.stage, entry.mode, entry.id, entry.extended); })
        .collect::<Vec<String>>()
        .join("\t");
}

/// The bytes a hexadecimal path spells, or nothing when it is not hexadecimal.
fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    let digits: Vec<u8> = text.bytes().map(|byte| { return char::from(byte).to_digit(16).map(|digit| { return digit as u8; }); }).collect::<Option<Vec<u8>>>()?;
    return Some(digits.chunks_exact(2).map(|pair| { return (pair[0] << 4) | pair[1]; }).collect());
}

/// The entries a text spells, or nothing when any item is damaged.
pub fn decode(text: &str) -> Option<Vec<StartEntry>> {
    return text
        .split('\t')
        .filter(|item| { return !item.is_empty(); })
        .map(|item| {
            let mut fields = item.split(':');
            let path: Vec<u8> = unhex(fields.next()?)?;
            let stage: u8 = fields.next()?.parse().ok()?;
            let mode: u32 = u32::from_str_radix(fields.next()?, 8).ok()?;
            let id: String = fields.next()?.to_string();
            let extended: u16 = fields.next()?.parse().ok()?;
            return fields.next().is_none().then_some(StartEntry { path, stage, mode, id, extended });
        })
        .collect();
}

/// What:
///  The names of `names` split into those whose entries in `current` are the ones in `recorded`, and the others.
/// Why:
///   A path with no entry at the start has none in the record, so a path nobody staged since compares equal whether or not it
///  was in the index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { unchanged, changed } = splitByStart(recorded, current, names)
/// ```
pub fn split_by_start(recorded: &[StartEntry], current: &[StartEntry], names: &[Vec<u8>]) -> (Vec<Vec<u8>>, Vec<Vec<u8>>) {
    let entries_of = |entries: &[StartEntry], name: &[u8]| -> Vec<StartEntry> {
        return entries.iter().filter(|entry| { return entry.path.as_slice() == name; }).cloned().collect();
    };
    return names.iter().cloned().partition(|name| { return entries_of(recorded, name.as_slice()) == entries_of(current, name.as_slice()); });
}

#[cfg(test)]
#[path = "service_land_start_tests.rs"]
mod tests;
