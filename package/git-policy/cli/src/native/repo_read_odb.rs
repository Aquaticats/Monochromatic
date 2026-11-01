//! What:
//!  The object database of one repository read in process: loose objects, packs and alternates.
//! Why:
//!  A lookup that finds the object answers exactly what `git cat-file` would print for it;
//!  a lookup that does not (missing, corrupt, too large, replaced by a `refs/replace` ref, in a shallow or grafted
//!  history) returns `None`, and the caller asks Git, which words the failure or fetches a promised object itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const odb = openObjectDatabase(commonDir, format); const object = odb.read(name); // undefined: ask Git
//! ```

#![cfg(unix)]

use super::repo_read_layout::ObjectFormat;
use super::repo_read_object::{OBJECT_SIZE_LIMIT, RawObject, name_length, parse_loose, to_hex};
use super::repo_read_pack::Pack;
use std::path::{Path, PathBuf};

/// How many alternate object directories deep the reader follows `info/alternates`.
const ALTERNATE_DEPTH_LIMIT: usize = 5;

/// The loose directories and packs of a repository and its alternates.
#[derive(Debug)]
pub struct ObjectDatabase {
    /// The object directories, the repository's own first.
    roots: Vec<PathBuf>,
    /// Every pack of every directory.
    packs: Vec<Pack>,
    /// The object format, which fixes the name length.
    format: ObjectFormat,
}

/// The extra object directories `objects/info/alternates` names; `None` when the file cannot be read.
fn alternates_of(root: &Path, depth: usize, found: &mut Vec<PathBuf>) -> Option<()> {
    if depth > ALTERNATE_DEPTH_LIMIT {
        return None;
    }
    let file: PathBuf = root.join("info/alternates");
    let text: String = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(()),
        Err(_) => return None,
    };
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('"') || line.contains('\\') {
            return None;
        }
        let target: PathBuf = std::fs::canonicalize(root.join(line)).ok()?;
        if found.contains(&target) {
            continue;
        }
        found.push(target.clone());
        alternates_of(target.as_path(), depth + 1, found)?;
    }
    return Some(());
}

/// Whether the repository changes what an object name means: replace refs, a shallow file, grafts.
fn objects_are_replaced(common_dir: &Path) -> bool {
    let replace_dir: PathBuf = common_dir.join("refs/replace");
    if std::fs::read_dir(&replace_dir).map_or(false, |mut entries| { return entries.next().is_some(); }) {
        return true;
    }
    if common_dir.join("shallow").exists() || common_dir.join("info/grafts").exists() {
        return true;
    }
    return match std::fs::read(common_dir.join("packed-refs")) {
        Ok(bytes) => bytes.windows(b" refs/replace/".len()).any(|window| { return window == b" refs/replace/"; }),
        Err(error) => error.kind() != std::io::ErrorKind::NotFound,
    };
}

impl ObjectDatabase {
    /// What:
    ///  Open the database under `common_dir/objects` (or `objects_dir` when given) with its alternates;
    ///  `None` when anything makes object names mean more than the files say.
    /// Why:
    ///   Opening lists the pack indexes once; every later lookup reads only the few bytes it needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// openObjectDatabase({ commonDir, objectsDir, format }): ObjectDatabase | undefined;
    /// ```
    pub fn open(common_dir: &Path, objects_dir: Option<&Path>, format: ObjectFormat) -> Option<ObjectDatabase> {
        if objects_are_replaced(common_dir) {
            return None;
        }
        let own: PathBuf = std::fs::canonicalize(objects_dir.map_or_else(|| { return common_dir.join("objects"); }, Path::to_path_buf)).ok()?;
        let mut roots: Vec<PathBuf> = vec![own.clone()];
        alternates_of(own.as_path(), 0, &mut roots)?;
        let mut packs: Vec<Pack> = Vec::new();
        for root in &roots {
            let listing = match std::fs::read_dir(root.join("pack")) {
                Ok(listing) => listing,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(_) => return None,
            };
            for entry in listing {
                let path: PathBuf = entry.ok()?.path();
                if path.extension().is_some_and(|extension| { return extension == "idx"; }) {
                    packs.push(Pack::open(path.as_path(), name_length(format))?);
                }
            }
        }
        return Some(ObjectDatabase { roots, packs, format });
    }

    /// The object named `name`, or `None` when it is absent, unreadable or beyond the size limit.
    pub fn read(&self, name: &[u8]) -> Option<RawObject> {
        return self.read_at_depth(name, 0);
    }

    /// The same read, counting the delta bases already followed through names so a cycle ends.
    fn read_at_depth(&self, name: &[u8], depth: usize) -> Option<RawObject> {
        if name.len() != name_length(self.format) || depth > 8 {
            return None;
        }
        let hex: String = to_hex(name);
        for root in &self.roots {
            let path: PathBuf = root.join(&hex[..2]).join(&hex[2..]);
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if !metadata.is_file() || metadata.len() > OBJECT_SIZE_LIMIT as u64 {
                return None;
            }
            return parse_loose(std::fs::read(&path).ok()?.as_slice());
        }
        let by_name = |base: &[u8]| -> Option<RawObject> { return self.read_at_depth(base, depth + 1); };
        for pack in &self.packs {
            if let Some(offset) = pack.find(name) {
                return pack.read(offset, &by_name, 0);
            }
        }
        return None;
    }
}

#[cfg(test)]
#[path = "repo_read_odb_tests.rs"]
mod tests;
