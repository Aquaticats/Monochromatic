//! What:
//!  The launcher records of a worktree's launcher directory,
//!  read once:
//!  the records that name another checkout's lock,
//!  the launcher names this worktree's lock declares,
//!  and which records are stale work an install rewrites,
//!  against those no install can.
//! Why:
//!  `package_repair_toolchain.rs` would otherwise pass the file-size limit,
//!  and both faces of the repair decide what to run from this one reading,
//!  so the rule that tells an install's work from a copy of an earlier release's layout
//!  lives in a file of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const launchersOf = (destination: string) => ({ stale: [...], unrewritable: [...] });
//! ```

/// Import the log tag.
use super::names_and_bounds::TAG;
/// Import the launcher directory and the record extension.
use super::names_and_bounds::{
    LAUNCHERS,
    RECORD_EXTENSION,
};
/// Import the lock's name and the byte bound the lock is read at.
use super::names_and_bounds::{
    LOCK_FILE,
    LONGEST_LOCK,
};
/// Import the object comparisons a record is judged by.
use super::object_identity::{
    Identity,
    ThisWorktreesLock,
    identity_of,
};
/// Import the one diagnostic log this crate writes.
use crate::diagnostic_log::debug;
/// Import the bounded read of the worktree's own lock through any link of the person's.
use crate::plain_file::read_plain_file_through_links;
/// Import the set of launcher names a lock declares.
use std::collections::HashSet;
/// Import native paths.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  A launcher record and the launcher beside it,
///  as one pass classified them:
///  a copy of an earlier release's layout that no install rewrites.
/// Why:
///   The removal proves what it deletes,
///   and a launcher has no bytes that say whether an install wrote it:
///   the object that was beside the record when the pass read the directory is the only thing to compare,
///   because a launcher another command writes is another object under the same name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Copy = { record: string; launcher: string; name?: string; launcherIdentity?: Identity };
/// ```
pub(super) struct CopyOfAnEarlierLayout {
    /// The record.
    pub(super) record: PathBuf,
    /// The object the record was when the pass judged it:
    /// another command that writes its own record at that name writes another object, which this pass may not take.
    pub(super) record_identity: Option<Identity>,
    /// The launcher beside it, whether or not there was one.
    pub(super) launcher: PathBuf,
    /// The launcher's name the lock is asked about.
    pub(super) name: Option<String>,
    /// The object that was at the launcher's name.
    pub(super) launcher_identity: Option<Identity>,
}

/// What:
///  The launcher names this worktree's lock declares,
///  which are the names an install may rewrite a record for,
///  or nothing when the lock cannot be read as one carrying names.
/// Why:
///   `mpm-shadow install` writes a launcher and its record for each bin the lock declares and touches nothing else,
///  so a record for any other name is a copy of a layout an earlier release had and no install answers for it.
///   A lock this release cannot read leaves every record something an install answers,
///  which keeps the check as strict as it was before the names were read at all.
///  The lock is read the way the record beside it is: only a plain file is opened and only so many bytes of it are read,
///  so a named pipe at that name holds neither the add's thread nor a job of the service's pool,
///  and a runaway file cannot be read into memory whole.
///  A lock behind a link of the person's own is read, because that is the lock the worktree means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const declared = (destination) => { const lock = JSON.parse(readPlainFileThroughLinks(join(destination, 'toolchain.lock.json'), SMALL_FILE_LIMIT)); return new Set(Object.values(lock.tools).flatMap((tool) => tool.bins ?? [])); };
/// ```
pub(super) fn declared_launcher_names(destination: &Path) -> Option<HashSet<String>> {
    let bytes: Vec<u8> =
        read_plain_file_through_links(destination.join(LOCK_FILE).as_path(), LONGEST_LOCK).ok()?;
    let lock: serde_json::Value =
        serde_json::from_slice::<serde_json::Value>(bytes.as_slice()).ok()?;
    let tools: &serde_json::Map<String, serde_json::Value> = lock.get("tools")?.as_object()?;
    let mut names: HashSet<String> = HashSet::new();
    for tool in tools.values() {
        for bin in tool
            .get("bins")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(name) = bin.as_str() {
                names.insert(name.to_owned());
            }
        }
    }
    return (!names.is_empty()).then_some(names);
}

/// What:
///  The launcher name a record belongs to:
///  its own file name without the record's extension.
/// Why:
///   The launcher beside a record is named exactly so,
///  and the lock declares those names,
///  so this is what tells a record an install rewrites from a copy no install rewrites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const launcherName = (record) => basename(record, '.launch');
/// ```
fn launcher_name(record: &Path) -> Option<String> {
    return record.file_stem()?.to_str().map(str::to_owned);
}

/// What:
///  What a worktree's launcher directory holds:
///  the records an install rewrites,
///  and the copies of an earlier release's layout beside them.
/// Why:
///  The two are answered differently:
///  a stale record an install rewrites is a failure when the install leaves it stale,
///  while no install can rewrite a record for a name the lock declares nowhere,
///  so failing over it would report work no tool of this release can do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Launchers = { stale: string[]; unrewritable: string[] };
/// ```
pub(super) struct Launchers {
    /// Records naming another lock that the worktree's own lock declares a launcher for.
    pub(super) stale: Vec<PathBuf>,
    /// Records naming another lock for a name the worktree's own lock declares nowhere, and what they were read beside.
    pub(super) unrewritable: Vec<CopyOfAnEarlierLayout>,
}

/// What:
///  The two kinds of stale launcher record of `destination`,
/// read from the launcher directory and the lock alone.
/// Why:
///   Both faces of the repair decide the install from this,
///  and the count after the install asks it again,
///  so the rule that tells an install's work from a copy lives in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const launchersOf = (destination) => { const declared = declaredLauncherNames(destination); return { stale: namedAnother.filter((r) => declared?.has(launcherName(r)) ?? true), unrewritable: declared ? namedAnother.filter((r) => !declared.has(launcherName(r))) : [] }; };
/// ```
pub(super) fn launchers_of(destination: &Path) -> Launchers {
    let named_another: Vec<PathBuf> = stale_launch_records(destination);
    let Some(declared) = declared_launcher_names(destination) else {
        return Launchers {
            stale: named_another,
            unrewritable: Vec::new(),
        };
    };
    let mut stale: Vec<PathBuf> = Vec::new();
    let mut unrewritable: Vec<CopyOfAnEarlierLayout> = Vec::new();
    for record in named_another {
        let name: Option<String> = launcher_name(record.as_path());
        if name
            .as_ref()
            .is_some_and(|launcher| return declared.contains(launcher))
        {
            stale.push(record);
            continue;
        }
        let launcher: PathBuf = record.with_extension("");
        let record_identity: Option<Identity> = identity_of(record.as_path());
        let launcher_identity: Option<Identity> = identity_of(launcher.as_path());
        unrewritable.push(CopyOfAnEarlierLayout {
            record,
            record_identity,
            launcher,
            name,
            launcher_identity,
        });
    }
    return Launchers {
        stale,
        unrewritable,
    };
}

/// What:
///  The launcher records of a worktree that name a lock file other than the worktree's own.
/// Why:
///   They are what a copy leaves behind;
/// a record naming this worktree's lock is already right and needs no repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleLaunchRecords(destination: string): string[];
/// ```
pub fn stale_launch_records(destination: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(destination.join(LAUNCHERS)) else {
        return Vec::new();
    };
    let own: ThisWorktreesLock = ThisWorktreesLock::of(destination);
    let mut stale: Vec<PathBuf> = Vec::new();
    for item in listing.flatten() {
        let path: PathBuf = item.path();
        let is_record: bool = match path.extension() {
            Some(extension) => extension == RECORD_EXTENSION,
            None => false,
        };
        if !is_record {
            continue;
        }
        match own.names_another_lock(path.as_path()) {
            Some(true) => stale.push(path),
            Some(false) => {}
            None => debug(
                TAG,
                format!("{} names no lock file and is left alone", path.display()).as_str(),
            ),
        }
    }
    stale.sort();
    return stale;
}
