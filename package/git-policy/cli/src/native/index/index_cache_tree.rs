//! What:
//!  The `TREE` (cache-tree) index extension: read, written back byte for byte, and invalidated along a path
//!  (`cache-tree.c`: `read_one`, `write_one`, `do_invalidate_path`).
//! Why:
//!  Git marks every directory above a changed path as not valid and drops a subtree that the path names; an in-process
//!  index update must leave the extension exactly as that does, or the next `write-tree` would trust a stale tree name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const tree = CacheTree.read(bytes, hashLength); tree.invalidate(path); const out = tree.write();
//! ```

use crate::index_byte_search::{first_index_of_byte, leading_digit_count, leading_run_length};

/// The deepest nesting the reader follows: the depth the tree reader follows (`TREE_DEPTH_LIMIT` in `repo_read_tree.rs`).
const DEPTH_LIMIT: usize = 256;

/// One directory of the cache-tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheTree {
    /// The number of index entries below, or -1 when the directory is not valid.
    pub entry_count: i64,
    /// The tree object name; present exactly when `entry_count` is not negative.
    pub id: Vec<u8>,
    /// The subdirectories by name, in the byte order the file keeps.
    pub subtrees: Vec<(Vec<u8>, CacheTree)>,
}

/// The decimal number at the start of `bytes` (with `-` signs, as `parse_int`), the bytes after it.
fn parse_number(bytes: &[u8]) -> Option<(i64, &[u8])> {
    let signs: usize = leading_run_length(bytes, b'-');
    let digits: usize = leading_digit_count(&bytes[signs..]);
    let text: &str = std::str::from_utf8(&bytes[signs..signs + digits]).ok()?;
    let magnitude: i64 = if text.is_empty() { 0 } else { text.parse::<i64>().ok()? };
    let value: i64 = if signs % 2 == 1 { -magnitude } else { magnitude };
    return Some((value, &bytes[signs + digits..]));
}

/// Read one directory and the subdirectories after it; the bytes left and the name of the directory.
fn read_one<'a>(bytes: &'a [u8], hash_length: usize, depth: usize) -> Option<(Vec<u8>, CacheTree, &'a [u8])> {
    if depth > DEPTH_LIMIT {
        return None;
    }
    let nul: usize = first_index_of_byte(bytes, 0)?;
    let name: Vec<u8> = bytes[..nul].to_vec();
    let (entry_count, after_count) = parse_number(&bytes[nul + 1..])?;
    let after_space: &[u8] = after_count.strip_prefix(b" ")?;
    let (subtree_count, after_subtrees) = parse_number(after_space)?;
    let mut rest: &[u8] = after_subtrees.strip_prefix(b"\n")?;
    let mut id: Vec<u8> = Vec::new();
    if entry_count >= 0 {
        id = rest.get(..hash_length)?.to_vec();
        rest = &rest[hash_length..];
    }
    let mut subtrees: Vec<(Vec<u8>, CacheTree)> = Vec::new();
    for _ in 0..usize::try_from(subtree_count).ok()? {
        let (child_name, child, after) = read_one(rest, hash_length, depth + 1)?;
        // Git sorts subtrees on insertion by length first and then by bytes (`subtree_name_cmp`); a file that is not already
        //  in that order is left to Git.
        if let Some((previous, _)) = subtrees.last()
            && (previous.len(), previous.as_slice()) >= (child_name.len(), child_name.as_slice())
        {
            return None;
        }
        subtrees.push((child_name, child));
        rest = after;
    }
    return Some((name, CacheTree { entry_count, id, subtrees }, rest));
}

/// Append one directory and its subdirectories under `name`.
fn write_one(out: &mut Vec<u8>, name: &[u8], tree: &CacheTree) {
    out.extend_from_slice(name);
    out.extend_from_slice(format!("\0{} {}\n", tree.entry_count, tree.subtrees.len()).as_bytes());
    if tree.entry_count >= 0 {
        out.extend_from_slice(tree.id.as_slice());
    }
    for (child_name, child) in &tree.subtrees {
        write_one(out, child_name.as_slice(), child);
    }
}

impl CacheTree {
    /// The tree of an extension's bytes, or `None` for any malformed or trailing content (Git would drop the extension).
    pub fn read(bytes: &[u8], hash_length: usize) -> Option<CacheTree> {
        let (_, tree, rest) = read_one(bytes, hash_length, 0)?;
        if !rest.is_empty() {
            return None;
        }
        return Some(tree);
    }

    /// The extension's bytes.
    pub fn write(&self) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        write_one(&mut out, b"", self);
        return out;
    }

    /// Mark every directory above `path` as not valid and remove the subtree `path` names (`do_invalidate_path`).
    pub fn invalidate(&mut self, path: &[u8]) {
        self.entry_count = -1;
        self.id.clear();
        let Some(slash) = first_index_of_byte(path, b'/') else {
            // Rebuild the list without the subtree `path` names, keeping the others in their order.
            let mut kept: Vec<(Vec<u8>, CacheTree)> = Vec::new();
            for (name, child) in std::mem::take(&mut self.subtrees) {
                if name.as_slice() != path {
                    kept.push((name, child));
                }
            }
            self.subtrees = kept;
            return;
        };
        for (name, child) in &mut self.subtrees {
            if name.as_slice() == &path[..slash] {
                child.invalidate(&path[slash + 1..]);
                break;
            }
        }
    }
}

#[cfg(test)]
#[path = "index_cache_tree_tests.rs"]
mod tests;
