//! What:
//!  The object database of one repository read in process:
//! loose objects,
//! packs and alternates.
//! Why:
//!  A lookup that finds the object answers exactly what `git cat-file` would print for it;
//!  a lookup that does not (missing,
//! corrupt,
//! too large,
//! replaced by a `refs/replace` ref,
//! in a shallow or grafted
//!  history) returns `None`,
//! and the caller asks Git,
//! which words the failure or fetches a promised object itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const odb = openObjectDatabase(commonDir, format); const object = odb.read(name); // undefined: ask Git
//! ```

#![cfg(unix)]

use crate::private_storage::{ReadRefusal, read_regular_file};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{OBJECT_SIZE_LIMIT, RawObject, name_length, parse_loose, to_hex};
use crate::repo_read_pack::{BaseByName, Pack};
use std::path::{Path, PathBuf};

/// How many alternate object directories deep the reader follows `info/alternates`.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const ALTERNATE_DEPTH_LIMIT: usize = 5;

/// The largest `info/alternates` file the reader looks at.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const ALTERNATES_LIMIT: u64 = 1 << 20;

/// The largest `packed-refs` file the reader scans for replace refs.
///  Exempt from a citation under `LQM` (class:
/// reader fallback):
/// a file,
/// chain or count past this bound makes the reader
///  decline and real Git answer,
/// so a Git start is added and no result changes.
/// Classed once in
///  `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
///  open to the owner's veto.
const PACKED_REFS_LIMIT: u64 = 256 << 20;

#[cfg(test)]
thread_local! {
    /// How many loose paths the readers of this thread have looked at,
    /// counted by the controls only.
    /// It is per thread,
    /// so that a control which shows a packed
    ///  object costs none cannot be disturbed by another test's reads.
    static LOOSE_LOOKS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Counts one loose path looked at,
/// for the controls;
/// the release executable counts nothing.
#[cfg(test)]
fn note_loose_look() {
    LOOSE_LOOKS.set(LOOSE_LOOKS.get() + 1);
}

/// The release executable does not count loose looks.
#[cfg(not(test))]
fn note_loose_look() {}

/// The loose paths the readers of this thread have looked at so far.
#[cfg(test)]
pub(crate) fn loose_looks() -> u64 {
    return LOOSE_LOOKS.with(std::cell::Cell::get);
}

/// The loose directories and packs of a repository and its alternates.
#[derive(Debug)]
pub struct ObjectDatabase {
    /// The object directories,
    /// the repository's own first.
    roots: Vec<PathBuf>,
    /// Every pack of every directory.
    packs: Vec<Pack>,
    /// The object format,
    /// which fixes the name length.
    format: ObjectFormat,
}

/// The extra object directories `objects/info/alternates` names;
/// `None` when the file cannot be read.
pub fn alternates_of(root: &Path, depth: usize, found: &mut Vec<PathBuf>) -> Option<()> {
    if depth > ALTERNATE_DEPTH_LIMIT {
        return None;
    }
    let file: PathBuf = root.join("info/alternates");
    // A bounded read of a regular file: a link, a pipe or a huge file is left to Git.
    let text: String = match read_regular_file(&file, ALTERNATES_LIMIT) {
        Ok(bytes) => String::from_utf8(bytes).ok()?,
        Err(ReadRefusal::Missing) => return Some(()),
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

/// Whether the repository changes what an object name means:
/// replace refs,
/// a shallow file,
/// grafts.
fn objects_are_replaced(common_dir: &Path) -> bool {
    let replace_dir: PathBuf = common_dir.join("refs/replace");
    if let Ok(mut entries) = std::fs::read_dir(&replace_dir)
        && entries.next().is_some()
    {
        return true;
    }
    if common_dir.join("shallow").exists() || common_dir.join("info/grafts").exists() {
        return true;
    }
    return match read_regular_file(common_dir.join("packed-refs").as_path(), PACKED_REFS_LIMIT) {
        Ok(bytes) => {
            let mut names_replace_refs: bool = false;
            for window in bytes.windows(b" refs/replace/".len()) {
                if window == b" refs/replace/" {
                    names_replace_refs = true;
                    break;
                }
            }
            names_replace_refs
        }
        Err(ReadRefusal::Missing) => false,
        Err(_) => true,
    };
}

/// What:
///  The base of a `REF_DELTA` found through the database,
/// one step deeper in the delta chain than the object that asked.
/// Why:
///   The depth counts the bases already followed through names so a cycle ends;
/// the database and the depth are the state the lookup needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class BaseOneDeeper implements BaseByName { baseNamed(name) { return this.database.readAtDepth(name, this.depth + 1); } }
/// ```
struct BaseOneDeeper<'a> {
    /// The database that holds the base.
    database: &'a ObjectDatabase,
    /// How many bases were followed through names before this one.
    depth: usize,
}

impl BaseByName for BaseOneDeeper<'_> {
    fn base_named(&self, name: &[u8]) -> Option<RawObject> {
        return self.database.read_at_depth(name, self.depth + 1);
    }
}

impl ObjectDatabase {
    /// What:
    ///  Open the database under `common_dir/objects` (or `objects_dir` when given) with its alternates;
    ///  `None` when anything makes object names mean more than the files say.
    /// Why:
    ///   Opening lists the pack indexes once;
    /// every later lookup reads only the few bytes it needs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// openObjectDatabase({ commonDir, objectsDir, format }): ObjectDatabase | undefined;
    /// ```
    pub fn open(common_dir: &Path, objects_dir: Option<&Path>, format: ObjectFormat) -> Option<ObjectDatabase> {
        if objects_are_replaced(common_dir) {
            return None;
        }
        let objects: PathBuf = match objects_dir {
            Some(directory) => directory.to_path_buf(),
            None => common_dir.join("objects"),
        };
        let own: PathBuf = std::fs::canonicalize(objects).ok()?;
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
                if path.extension() == Some(std::ffi::OsStr::new("idx")) {
                    packs.push(Pack::open(path.as_path(), name_length(format))?);
                }
            }
        }
        return Some(ObjectDatabase { roots, packs, format });
    }

    /// The object named `name`,
    /// or `None` when it is absent,
    /// unreadable or beyond the size limit.
    pub fn read(&self, name: &[u8]) -> Option<RawObject> {
        return self.read_at_depth(name, 0);
    }

    /// How many pack entries this database has decoded so far,
    /// counting an entry each time its bytes are rebuilt.
    pub fn decoded_entries(&self) -> u64 {
        return self.packs.iter().map(Pack::decoded_entries).sum();
    }

    /// Whether the object named `name` is stored loose or in a pack,
    /// without reading it.
    pub fn contains(&self, name: &[u8]) -> bool {
        if name.len() != name_length(self.format) {
            return false;
        }
        let hex: String = to_hex(name);
        for root in &self.roots {
            if root.join(&hex[..2]).join(&hex[2..]).is_file() {
                return true;
            }
        }
        for pack in &self.packs {
            if pack.find(name).is_some() {
                return true;
            }
        }
        return false;
    }

    /// The same read,
    /// counting the delta bases already followed through names so a cycle ends.
    fn read_at_depth(&self, name: &[u8], depth: usize) -> Option<RawObject> {
        if name.len() != name_length(self.format) || depth > 8 {
            return None;
        }
        // Git searches the packs first and the loose files after, and so does this: nearly every object a commit reads is packed,
        //  and a look at a loose path that is not there is a system call for each of thousands of objects.
        // The pack follows a delta chain through this value, which carries the database and the depth so far.
        let by_name: BaseOneDeeper<'_> = BaseOneDeeper { database: self, depth };
        for pack in &self.packs {
            if let Some(offset) = pack.find(name) {
                if let Some(found) = pack.read(offset, &by_name, 0) {
                    return Some(found);
                }
                // A pack entry that cannot be rebuilt leaves the loose copy, if there is one, to answer.
                break;
            }
        }
        let hex: String = to_hex(name);
        for root in &self.roots {
            let path: PathBuf = root.join(&hex[..2]).join(&hex[2..]);
            note_loose_look();
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if !metadata.is_file() || metadata.len() > OBJECT_SIZE_LIMIT as u64 {
                return None;
            }
            return parse_loose(std::fs::read(&path).ok()?.as_slice());
        }
        return None;
    }
}

#[cfg(test)]
#[path = "repo_read_odb_tests.rs"]
mod tests;
