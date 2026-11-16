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
use crate::repo_read_object::{OBJECT_SIZE_LIMIT, RawObject, name_length, object_name, parse_loose, to_hex};
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

/// Why a list of other object directories was not followed,
/// in the words a person reads when the store cannot be trusted:
/// which file,
/// and what its spelling asks for.
fn alternates_refusal(file: &Path, what: &str) -> String {
    return format!("the object directory list {file} {what}", file = file.display(), what = what);
}

/// The extra object directories `objects/info/alternates` names;
/// the reason when the file cannot be read or names a directory in a spelling this reader does not follow.
pub fn alternates_of(root: &Path, depth: usize, found: &mut Vec<PathBuf>) -> Result<(), String> {
    let file: PathBuf = root.join("info/alternates");
    if depth > ALTERNATE_DEPTH_LIMIT {
        return Err(alternates_refusal(&file, "chains more object directories than this reader follows"));
    }
    // A bounded read of a regular file: a link, a pipe or a huge file is left to Git.
    let text: String = match read_regular_file(&file, ALTERNATES_LIMIT) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => return Err(alternates_refusal(&file, "is not text this reader can read")),
        },
        Err(ReadRefusal::Missing) => return Ok(()),
        Err(ReadRefusal::NotRegular) => return Err(alternates_refusal(&file, "is not a regular file")),
        Err(ReadRefusal::TooLarge) => return Err(alternates_refusal(&file, "is larger than this reader looks at")),
        Err(ReadRefusal::Io(error)) => return Err(alternates_refusal(&file, format!("cannot be read ({error})").as_str())),
    };
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('"') || line.contains('\\') {
            return Err(alternates_refusal(&file, format!("names {line} in a quoting or escaping this reader does not follow; write the path plainly").as_str()));
        }
        let target: PathBuf = match std::fs::canonicalize(root.join(line)) {
            Ok(target) => target,
            Err(error) => return Err(alternates_refusal(&file, format!("names {} which cannot be resolved ({error})", root.join(line).display()).as_str())),
        };
        if found.contains(&target) {
            continue;
        }
        found.push(target.clone());
        alternates_of(target.as_path(), depth + 1, found)?;
    }
    return Ok(());
}

/// What:
///  Why an object name in this repository means another object than the one the files spell,
///  or nothing when every name does.
/// Why:
///   Replace refs,
/// a shallow file and grafts each make Git answer with an object the name does not spell,
///  so a reader of the files alone would compare the index with a tree nobody checked;
///  the words name the store and what makes it plain again,
///  because a person meets them in a removal that refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const replaced = whatReplacesNames(commonDir); // undefined: names mean what the files spell
/// ```
fn what_replaces_names(common_dir: &Path) -> Option<String> {
    let replace_dir: PathBuf = common_dir.join("refs/replace");
    if let Ok(mut entries) = std::fs::read_dir(&replace_dir)
        && let Some(Ok(entry)) = entries.next()
    {
        return Some(format!(
            "the repository holds a replace ref ({replace}), and a replace ref makes Git answer another object than the name spells; `git replace --delete {name}` drops it",
            replace = entry.path().display(),
            name = entry.file_name().to_string_lossy()
        ));
    }
    if common_dir.join("shallow").exists() {
        return Some(format!(
            "the repository is shallow ({file} is there), so the history it names is cut short and its objects may be absent; `git fetch --unshallow` makes the repository complete",
            file = common_dir.join("shallow").display()
        ));
    }
    if common_dir.join("info/grafts").exists() {
        return Some(format!(
            "the repository holds grafts ({file}), and a graft makes Git answer parents the commits do not name; `git filter-repo` replaces a graft with a rewritten history",
            file = common_dir.join("info/grafts").display()
        ));
    }
    return match read_regular_file(common_dir.join("packed-refs").as_path(), PACKED_REFS_LIMIT) {
        Ok(bytes) => {
            let names_replace_refs: bool = bytes.windows(b" refs/replace/".len()).any(|window| { return window == b" refs/replace/"; });
            return names_replace_refs.then(|| {
                return format!(
                    "the repository holds a replace ref in {file}, and a replace ref makes Git answer another object than the name spells; `git replace -d <name>` drops it",
                    file = common_dir.join("packed-refs").display()
                );
            });
        }
        Err(ReadRefusal::Missing) => None,
        Err(_) => Some(format!(
            "the file {file} beside the references cannot be read, so a replace ref it may name cannot be ruled out",
            file = common_dir.join("packed-refs").display()
        )),
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
    ///   A caller that answers with Git anyway needs no words,
    ///  and this is that caller's spelling of `open_or_reason`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// openObjectDatabase({ commonDir, objectsDir, format }): ObjectDatabase | undefined;
    /// ```
    pub fn open(common_dir: &Path, objects_dir: Option<&Path>, format: ObjectFormat) -> Option<ObjectDatabase> {
        return ObjectDatabase::open_or_reason(common_dir, objects_dir, format).ok();
    }

    /// What:
    ///  The database under `common_dir/objects` (or `objects_dir` when given),
    ///  or the reason a store in it is one this reader cannot trust,
    ///  in words that name the store and what makes it plain again.
    /// Why:
    ///   A person meets this reason when a removal's own cleanliness proof gives up and the removal refuses,
    ///  so it names which store is in the way and what to run,
    ///  and it is the store,
    ///  never the person's work,
    ///  that a reader of these words changes.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const opened = openOrReason({ commonDir, objectsDir, format }); // { database } | { reason }
    /// ```
    pub fn open_or_reason(common_dir: &Path, objects_dir: Option<&Path>, format: ObjectFormat) -> Result<ObjectDatabase, String> {
        if let Some(reason) = what_replaces_names(common_dir) {
            return Err(reason);
        }
        let objects: PathBuf = match objects_dir {
            Some(directory) => directory.to_path_buf(),
            None => common_dir.join("objects"),
        };
        let own: PathBuf = match std::fs::canonicalize(objects.as_path()) {
            Ok(own) => own,
            Err(error) => return Err(format!("the object directory {directory} cannot be resolved ({error}); `git fsck --full` names what is wrong with it", directory = objects.display())),
        };
        let mut roots: Vec<PathBuf> = vec![own.clone()];
        alternates_of(own.as_path(), 0, &mut roots)?;
        let mut packs: Vec<Pack> = Vec::new();
        for root in &roots {
            let listing = match std::fs::read_dir(root.join("pack")) {
                Ok(listing) => listing,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => return Err(format!("the pack directory {directory} cannot be listed ({error})", directory = root.join("pack").display())),
            };
            for found in listing {
                let Ok(directory_entry) = found else {
                    return Err(format!("the pack directory {directory} cannot be listed", directory = root.join("pack").display()));
                };
                let path: PathBuf = directory_entry.path();
                if path.extension() != Some(std::ffi::OsStr::new("idx")) {
                    continue;
                }
                match Pack::open(path.as_path(), name_length(format)) {
                    Some(pack) => packs.push(pack),
                    None => {
                        return Err(format!(
                            "the pack index {index} is not a version 1 or version 2 index of the pack beside it, so this program does not read the objects of that pack; `git index-pack {pack}` writes the index again from the pack, which Git reads meanwhile",
                            index = path.display(),
                            pack = path.with_extension("pack").display()
                        ));
                    }
                }
            }
        }
        return Ok(ObjectDatabase { roots, packs, format });
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
        // Git searches the packs first and the loose files after, and so does this, for the same reason a read does:
        //  a loose path that is not there is a system call for every packed object (measured on 2026-11-09:
        //  1433 ref objects, all packed, spent about 4.3 ms of a 13.7 ms answer on these stats and gained nothing).
        for pack in &self.packs {
            if pack.find(name).is_some() {
                return true;
            }
        }
        let hex: String = to_hex(name);
        for root in &self.roots {
            note_loose_look();
            if root.join(&hex[..2]).join(&hex[2..]).is_file() {
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
                if let Some(found) = pack.read(offset, &by_name) {
                    // The pack is trusted only when the bytes it holds are the object its index names.
                    if object_name(found.kind, found.bytes.as_slice(), self.format) == name {
                        return Some(found);
                    }
                    return None;
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
            let object: RawObject = parse_loose(std::fs::read(&path).ok()?.as_slice())?;
            // The file is trusted only when the bytes it holds are the object its path names.
            return match object_name(object.kind, object.bytes.as_slice(), self.format) == name {
                true => Some(object),
                false => None,
            };
        }
        return None;
    }
}

#[cfg(test)]
#[path = "repo_read_odb_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "repo_read_odb_refusals_tests.rs"]
mod refusals_tests;
