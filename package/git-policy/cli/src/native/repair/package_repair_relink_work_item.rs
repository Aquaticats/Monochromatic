//! What:
//!  The work item a relink is queued as:
//! its object name,
//! the entry with every relative directory of its search path made absolute against the add's own
//! directory,
//! and the item the queue stores.
//! Why:
//!   A work item's id is its idempotency key,
//! so the path alone would let an earlier record answer a worktree removed and added again at the same path;
//! the moment tells two hand-overs of one path apart.
//! The service is a resident process of another session,
//! so a record carrying `bin` would make it search its own `bin`:
//! the add's directory is what a relative entry means,
//! and an entry it cannot read is refused rather than recorded as another set of directories.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const objectNameOf = (destination: string, queuedAt: string) => string;
//! const itemOf = ({ commonDir, destination, entry }) => WorkItem;
//! ```

/// Import the moment a record is stamped with.
use crate::iso_time::now_iso;
/// Import the record's entry,
/// the tool names it carries,
/// and the text it is stored as.
use crate::package_repair_relink_record::{
    RELINK_REFERENCE,
    RelinkEntry,
    SearchPathOfTheAdd,
    encode,
};
/// Import the queue's item and the kind of work a relink is.
use crate::work_item::{
    WorkItem,
    WorkKind,
};
/// Import the owned,
/// platform-native string a joined search path is.
use std::ffi::OsString;
/// Import the byte view of a path's own text.
use std::os::unix::ffi::OsStrExt as _;
/// Import the borrowed and owned filesystem paths a record names.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  The record's object name for one hand-over:
/// a digest of the worktree's path and the moment the add asked for the relink.
/// Why:
///   A work item's id is its idempotency key,
/// and the queue answers an id it already knows as known in every state.
/// With the path alone,
/// a worktree removed and added again at the same path would be answered by the earlier record,
/// and its copied links would stay exactly as the copy left them while the add reported a relink
/// that can never arrive.
/// A previous failure would keep every later add at that path from ever asking again.
/// The moment tells two hand-overs of one path apart,
/// and the path keeps two worktrees apart.
/// Forty hexadecimal digits is what a work item's object name must have,
/// and a SHA-256 slice is cheap to compute.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectNameOf = (destination, queuedAt) => sha256(utf8(`${destination}\n${queuedAt}`)).toString('hex').slice(0, 40);
/// ```
pub fn object_name_of(
    destination: &Path,
    queued_at: &str,
) -> String {
    let mut text: Vec<u8> = Vec::new();
    text.extend_from_slice(destination.as_os_str().as_bytes());
    text.push(b'\n');
    text.extend_from_slice(queued_at.as_bytes());
    let digest: [u8; 32] = crate::subvolume_sha::sha256(text.as_slice());
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(&digest[..20]);
}

/// What:
///  The entry with every relative directory of its search path made absolute against the add's directory.
/// Why:
///   `PATH=bin` names the add's own `bin`,
/// and the service is a resident process of another session,
/// started from another directory,
/// so a record carrying `bin` would make it search its own `bin`,
/// and the program the add resolved may never run.
/// The tools' own children read the same path from the same record,
/// so they resolve on the add's directories too.
/// An empty entry is left as it is:
/// it means the current directory of whoever runs,
/// which no record can name for another process.
///
/// # Errors
///
/// A working directory this process cannot read,
/// so no record could say what `bin` meant,
/// and a search path that cannot be written back.
fn entry_with_an_absolute_search_path(
    entry: &RelinkEntry,
    start: &crate::start_directory::StartDirectory,
) -> Result<RelinkEntry, String> {
    let SearchPathOfTheAdd::Set(search_path) = &entry.search_path else {
        return Ok(entry.clone());
    };
    if !names_a_relative_directory(search_path) {
        return Ok(entry.clone());
    }
    let Some(add) = start.path() else {
        return Err(no_working_directory(start.reason()));
    };
    let mut absolute: Vec<PathBuf> = Vec::new();
    for directory in std::env::split_paths(search_path) {
        if directory.as_os_str().is_empty() || directory.is_absolute() {
            absolute.push(directory);
            continue;
        }
        absolute.push(add.join(directory));
    }
    let joined: OsString = std::env::join_paths(absolute).map_err(unrecordable_search_path)?;
    return Ok(RelinkEntry {
        search_path: SearchPathOfTheAdd::Set(joined),
        ..entry.clone()
    });
}

/// What:
///  Whether any entry of `search_path` is relative to whatever directory runs.
/// Why:
///   Only a relative entry needs the add's directory,
/// and a path of only absolute entries and empty entries is recorded as the add held it,
/// which is the same text the person wrote.
/// An empty entry is not relative:
/// a shell reads it as the current directory of whatever runs,
/// which no record can name for another process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const relative = splitPath(searchPath).some((directory) => directory !== '' && !isAbsolute(directory));
/// ```
fn names_a_relative_directory(search_path: &std::ffi::OsStr) -> bool {
    for directory in std::env::split_paths(search_path) {
        if !directory.as_os_str().is_empty() && !directory.is_absolute() {
            return true;
        }
    }
    return false;
}

/// What:
///  The words a working directory that cannot be read is reported with.
/// Why:
///   `bin` means the add's own `bin`,
/// so a command whose own directory cannot be read cannot say what its search path named,
/// and no record could carry an answer it did not have.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noWorkingDirectory = (error) => `the directory this command runs in cannot be read (${error}), so a relative search path names no program it could resolve`;
/// ```
fn no_working_directory(reason: Option<&str>) -> String {
    return format!(
        "the directory this command runs in cannot be read ({}), so a relative search path names no program it could resolve",
        reason.unwrap_or("the caller named none")
    );
}

/// What:
///  The words a search path that cannot be written back are reported with.
/// Why:
///   A joined path holds a separator between entries,
/// so an entry holding that separator cannot be recorded,
/// and the add says so rather than writing a path that means another set of directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unrecordableSearchPath = (error) => `the search path of this command cannot be recorded (${error})`;
/// ```
fn unrecordable_search_path(error: std::env::JoinPathsError) -> String {
    return format!("the search path of this command cannot be recorded ({error})");
}

/// What:
///  The record a worktree's relink is queued as.
/// Why:
///   The producer and its controls both write exactly this,
/// so a control can compare a record without reaching a service.
/// The stamp is read once and is both the record's `created` and part of its object name,
/// so a control can tell which record it is looking at from the record alone,
/// and two adds at one path are two records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const itemOf = ({ commonDir, destination, entry }) => { const created = new Date().toISOString(); return { kind: 'relink', repository: destination, gitDir: commonDir, ref: RELINK_REFERENCE, oid: objectNameOf(destination, created), parent: encode(entry), created, tries: 0 }; };
/// ```
pub fn item_of(
    common_dir: &Path,
    destination: &Path,
    entry: &RelinkEntry,
    start: &crate::start_directory::StartDirectory,
) -> Result<WorkItem, String> {
    let created: String = now_iso();
    let recorded: RelinkEntry = entry_with_an_absolute_search_path(entry, start)?;
    return Ok(WorkItem {
        kind: WorkKind::Relink,
        repository: destination.to_path_buf(),
        git_dir: common_dir.to_path_buf(),
        branch_ref: String::from(RELINK_REFERENCE),
        oid: object_name_of(destination, created.as_str()),
        parent: encode(&recorded)?,
        created,
        tries: 0,
        transport: false,
    });
}
