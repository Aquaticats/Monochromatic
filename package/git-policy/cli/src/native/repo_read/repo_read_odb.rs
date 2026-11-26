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
//!  history,
//! kept in a ref store this reader does not scan) returns `None`,
//! and the caller asks Git,
//! which words the failure or fetches a promised object itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const odb = openObjectDatabase(commonDir, format); const object = odb.read(name); // undefined: ask Git
//! ```

#![cfg(unix)]

use crate::private_storage::{
    ReadRefusal,
    read_regular_file,
};
use crate::repo_read_config_scan::read_following;
use crate::repo_read_config_text::{
    last_entry,
    parse_config_text,
};
use crate::repo_read_layout::ObjectFormat;
use crate::repo_read_object::{
    OBJECT_SIZE_LIMIT,
    RawObject,
    name_length,
    parse_loose,
    to_hex,
};
use crate::repo_read_pack::{
    BaseByName,
    Pack,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};

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
pub fn alternates_of(
    root: &Path,
    depth: usize,
    found: &mut Vec<PathBuf>,
) -> Option<()> {
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

/// The variable Git reads as the ref storage format of one command:
/// it overrides `extensions.refStorage` for every start
/// (`git(1)`,
/// and `setup.c` reads it before applying the format).
const REFERENCE_BACKEND_ENVIRONMENT: &str = "GIT_REFERENCE_BACKEND";

/// The variables that send a read of one command to an object store or a replacement base this reader does not look
/// at:
/// the object store itself (`GIT_OBJECT_DIRECTORY`),
/// the directories searched beside it (`GIT_ALTERNATE_OBJECT_DIRECTORIES`),
/// and the namespace replacement refs are read from (`GIT_REPLACE_REF_BASE`).
const MOVED_READ_ENVIRONMENTS: &[&str] = &[
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_REPLACE_REF_BASE",
];

/// What:
///  Whether the environment sends an object read where this reader does not look.
/// Why:
///  The service's landing callers read objects in process and their Git children inherit the service's environment,
///  so a variable that moves the read makes the reader answer from the repository's own store where Git reads
///  another one:
///  `GIT_OBJECT_DIRECTORY` and `GIT_ALTERNATE_OBJECT_DIRECTORIES` name the store,
///  `GIT_REPLACE_REF_BASE` names where replacement refs are read,
///  and `GIT_REFERENCE_BACKEND` names the ref store.
///  `GIT_REFERENCE_BACKEND` overrides the repository's configuration for every Git start
/// (`git(1)`,
/// entry `GIT_REFERENCE_BACKEND`;
/// Git 2.54 and later),
///  and any value but exactly `files` puts the store where this reader does not look:
///  a files payload is another directory,
///  and the reftable backend keeps its replace refs in tables,
///  while a value of `files` changes nothing and leaves the repository open.
///  The other three change what a read means whatever their value,
///  even a value that names the store this reader already scans,
///  so their presence declines:
/// Git answers there,
/// at the price of its starts.
fn environment_names_another_read() -> bool {
    if std::env::var_os(REFERENCE_BACKEND_ENVIRONMENT).is_some_and(|value| {
        return value.as_os_str() != OsStr::new("files");
    }) {
        return true;
    }
    return MOVED_READ_ENVIRONMENTS.iter().any(|name| {
        return std::env::var_os(name).is_some();
    });
}

/// What:
///  Whether the repository keeps its references in the reftable backend's tables:
/// a replace ref is then in `reftable/`,
/// not in `refs/replace`
///  or `packed-refs`,
/// so a replacement there would go unseen and the reader would answer the object Git replaces.
/// Why:
///  The folder is not the only place the backend is named:
/// Git 2.56 reads `extensions.refStorage` as `<format>://<payload>`,
///  and the tables of such a store stand outside the common directory,
/// so a store moved that way leaves no `reftable` folder behind.
///  The configuration is where Git reads the backend from,
/// so it is read here as well.
/// Gotcha:
///  That read is what an open of a files-backend repository pays for this:
/// 7.5 microseconds added to a 16.5 microsecond open on the host measured on 2026-11-22,
/// and the note `doc/object-database-and-a-reftable-ref-store.md` records the paired run.
fn ref_store_is_reftable(common_dir: &Path) -> bool {
    if environment_names_another_read() {
        return true;
    }
    return ref_store_is_reftable_in_the_repository(common_dir);
}

/// What:
///  Whether the repository's own files keep its references in the reftable backend's tables,
///  asking nothing of the environment.
/// Why:
///  A proof of *this worktree's own* files reads the repository's own store,
/// as `main`'s reader did,
///  and a command's environment moves what Git reads and not what this repository keeps:
///  `ObjectDatabase::open_of_the_repository_alone` asks this,
///  and `ObjectDatabase::open` asks the environment first and then this.
fn ref_store_is_reftable_in_the_repository(common_dir: &Path) -> bool {
    if common_dir.join("reftable").exists() {
        return true;
    }
    // Git reads an extension from the repository's own file alone and honors no include of it
    //  (`read_repository_format` in `setup.c`), so this read follows none either.
    return match read_following(common_dir.join("config").as_path()) {
        Some(Some(text)) => config_names_another_ref_store(text.as_slice()),
        Some(None) => false,
        None => true,
    };
}

/// What:
///  Whether the repository's configuration names a reference store this reader cannot scan:
/// `extensions.refStorage` with any value but exactly `files`.
/// Why:
///  Git honors the extension in a version 1 repository format and refuses a repository that carries one in any
///  other
///  (`fatal: repo version is 0, but v1-only extension found`),
/// so a key that names another store is declined whatever version the configuration states.
/// Reading the version the way Git reads it is then not needed,
///  and its spellings (`1`,
///  `01`,
///  `+1`) cannot decide whether a store is declined.
/// A file that cannot be parsed is left to Git rather than guessed at.
fn config_names_another_ref_store(text: &[u8]) -> bool {
    let Some(entries) = parse_config_text(text) else {
        return true;
    };
    return match last_entry(entries.as_slice(), "extensions.refstorage") {
        Some(entry) => entry.value.as_deref() != Some(b"files"),
        None => false,
    };
}

/// Whether the repository changes what an object name means:
/// replace refs,
/// a shallow file,
/// or grafts.
///  The ref store is asked apart by `ObjectDatabase::open`,
///  because a caller that read the layout has already proved it.
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
///  The objects of a repository whose ref store the caller has settled:
///  its own objects directory (or the one given) and the directories its `info/alternates` names,
///  with every pack of each.
/// Why:
///  `ObjectDatabase::open` settles the ref store first and a caller that read the layout has already done so,
///  and the listing below is the same work either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// openTheObjects({ commonDir, objectsDir, format }): ObjectDatabase | undefined;
/// ```
fn open_the_objects(
    common_dir: &Path,
    objects_dir: Option<&Path>,
    format: ObjectFormat,
) -> Option<ObjectDatabase> {
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
    return Some(ObjectDatabase {
        roots,
        packs,
        format,
    });
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
    fn base_named(
        &self,
        name: &[u8],
    ) -> Option<RawObject> {
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
    pub fn open(
        common_dir: &Path,
        objects_dir: Option<&Path>,
        format: ObjectFormat,
    ) -> Option<ObjectDatabase> {
        if ref_store_is_reftable(common_dir) {
            return None;
        }
        return open_the_objects(common_dir, objects_dir, format);
    }

    /// What:
    ///  The database of *this worktree's own* files,
    ///  asked by a proof that compares the index with the worktree and never asks what a command reads.
    /// Why:
    ///  A content proof reads the repository's own store,
    /// as `main`'s reader did,
    /// whatever store a command's
    ///  environment names:
    ///  the files it judges are this worktree's own,
    /// and a store the environment moves is read by Git's own answers,
    ///  not by this one.
    ///  Every other decline stays,
    /// so a store the repository itself moved away still refuses.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// openObjectDatabaseOfTheRepositoryAlone({ commonDir, objectsDir, format }): ObjectDatabase | undefined;
    /// ```
    pub fn open_of_the_repository_alone(
        common_dir: &Path,
        objects_dir: Option<&Path>,
        format: ObjectFormat,
    ) -> Option<ObjectDatabase> {
        if ref_store_is_reftable_in_the_repository(common_dir) {
            return None;
        }
        return open_the_objects(common_dir, objects_dir, format);
    }

    /// What:
    ///  The same database for a repository whose layout the caller has already read,
    ///  so its ref store is not read a second time.
    /// Why:
    ///  `RepoLayout` exists only for a repository whose configuration names the files backend,
    ///  whose Git and common directories hold no `reftable` folder,
    ///  and whose environment sets no variable that can move an answer
    ///  (`repo_read_layout.rs`),
    ///  which is the proof `ref_store_is_reftable` makes.
    ///  Reading the configuration again on every open cost about 7.5 microseconds of a 16.5 microsecond open
    ///  on the host measured on 2026-11-22 (`doc/object-database-and-a-reftable-ref-store.md`),
    ///  and the callers that already hold the layout paid it for nothing.
    ///  Replace refs and the shallow and graft files are still read here:
    ///  the layout does not look at them.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// openObjectDatabaseFromProof({ commonDir: layout.commonDir, objectsDir, format: layout.objectFormat }): ObjectDatabase | undefined;
    /// ```
    pub fn open_where_the_layout_proved_the_store(
        layout: &crate::repo_read_layout::RepoLayout,
        objects_dir: Option<&Path>,
    ) -> Option<ObjectDatabase> {
        // The layout proved the ref store when it was read, and a store moved under a command that holds the layout (a
        //  `git refs migrate` inside a forwarded `merge`, say) leaves that proof stale. The folder is asked again, which is
        //  the cheap half of what the proof skipped; the environment and the configuration are not, since the layout refused
        //  them by default-deny when it was read.
        if layout.git_dir.join("reftable").exists() || layout.common_dir.join("reftable").exists() {
            return None;
        }
        return open_the_objects(
            layout.common_dir.as_path(),
            objects_dir,
            layout.object_format,
        );
    }

    /// The object named `name`,
    /// or `None` when it is absent,
    /// unreadable or beyond the size limit.
    pub fn read(
        &self,
        name: &[u8],
    ) -> Option<RawObject> {
        return self.read_at_depth(name, 0);
    }

    /// How many pack entries this database has decoded so far,
    /// counting an entry each time its bytes are rebuilt.
    pub fn decoded_entries(&self) -> u64 {
        return self.packs.iter().map(Pack::decoded_entries).sum();
    }

    /// Whether the object named `name` is stored loose or in a pack,
    /// without reading it.
    pub fn contains(
        &self,
        name: &[u8],
    ) -> bool {
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
    fn read_at_depth(
        &self,
        name: &[u8],
        depth: usize,
    ) -> Option<RawObject> {
        if name.len() != name_length(self.format) || depth > 8 {
            return None;
        }
        // Git searches the packs first and the loose files after, and so does this: nearly every object a commit reads is packed,
        //  and a look at a loose path that is not there is a system call for each of thousands of objects.
        // The pack follows a delta chain through this value, which carries the database and the depth so far.
        let by_name: BaseOneDeeper<'_> = BaseOneDeeper {
            database: self,
            depth,
        };
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
