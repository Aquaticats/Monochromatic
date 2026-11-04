//! What:
//!  The `REUC` (resolve-undo) index extension: read, recorded into when an unmerged entry leaves the index, written back
//!  (`resolve-undo.c`).
//! Why:
//!  Git remembers the three sides of a path whose conflict entries it removes so `git checkout -m` can restore them; an
//!  in-process index update that removes or replaces unmerged entries must record them the way `record_resolve_undo` does.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const undo = ResolveUndo.read(bytes, hashLength); undo.record(entry); const out = undo.write();
//! ```

use std::collections::BTreeMap;

/// The three sides of one path: the mode and object name of stage 1, 2 and 3 (mode 0 means the side was absent).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Sides {
    /// The mode of each stage, 0 for none.
    pub mode: [u32; 3],
    /// The object name of each stage, empty for none.
    pub id: [Vec<u8>; 3],
}

/// The recorded paths in byte order of the path.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ResolveUndo {
    /// Path to its sides.
    pub paths: BTreeMap<Vec<u8>, Sides>,
}

/// The bytes before the first NUL and the bytes after it.
fn split_nul(bytes: &[u8]) -> Option<(&[u8], &[u8])> {
    let nul: usize = bytes.iter().position(|byte| { return *byte == 0; })?;
    return Some((&bytes[..nul], &bytes[nul + 1..]));
}

impl ResolveUndo {
    /// The recorded paths of an extension's bytes, or `None` when Git reports invalid resolve-undo information.
    pub fn read(bytes: &[u8], hash_length: usize) -> Option<ResolveUndo> {
        let mut paths: BTreeMap<Vec<u8>, Sides> = BTreeMap::new();
        let mut rest: &[u8] = bytes;
        while !rest.is_empty() {
            let (path, after) = split_nul(rest)?;
            rest = after;
            let mut sides: Sides = paths.remove(path).unwrap_or_default();
            for mode in &mut sides.mode {
                let (text, following) = split_nul(rest)?;
                let digits: &str = std::str::from_utf8(text).ok()?;
                // `strtoul` also takes a sign or a space; only the plain octal digits Git writes are read here.
                if digits.is_empty() || !digits.bytes().all(|byte| { return (b'0'..=b'7').contains(&byte); }) {
                    return None;
                }
                *mode = u32::from_str_radix(digits, 8).ok()?;
                rest = following;
            }
            for (mode, id) in sides.mode.iter().zip(sides.id.iter_mut()) {
                if *mode != 0 {
                    *id = rest.get(..hash_length)?.to_vec();
                    rest = &rest[hash_length..];
                }
            }
            paths.insert(path.to_vec(), sides);
        }
        return Some(ResolveUndo { paths });
    }

    /// Remember one unmerged entry that is leaving the index (`record_resolve_undo`); a resolved entry is ignored.
    pub fn record(&mut self, path: &[u8], stage: u8, mode: u32, id: &[u8]) {
        if stage == 0 {
            return;
        }
        let sides: &mut Sides = self.paths.entry(path.to_vec()).or_default();
        sides.mode[usize::from(stage) - 1] = mode;
        sides.id[usize::from(stage) - 1] = id.to_vec();
    }

    /// The extension's bytes (`resolve_undo_write`).
    pub fn write(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        for (path, sides) in &self.paths {
            out.extend_from_slice(path.as_slice());
            out.push(0);
            for mode in sides.mode {
                out.extend_from_slice(format!("{mode:o}").as_bytes());
                out.push(0);
            }
            for (mode, id) in sides.mode.iter().zip(sides.id.iter()) {
                if *mode != 0 {
                    out.extend_from_slice(id.as_slice());
                }
            }
        }
        return out;
    }
}

#[cfg(test)]
#[path = "index_resolve_undo_tests.rs"]
mod tests;
