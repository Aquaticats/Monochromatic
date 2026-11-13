//! What:
//!  Where a record of an interrupted copy that cannot be read is kept,
//!  and how it gets there:
//!  one rename into a folder of its own under the journal root,
//!  never deleting it and never over another record.
//! Why:
//!  The record is the only trace of what the interrupted copy was doing,
//!  and the folder is what keeps it out of every reader's way
//!  without a person removing anything.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const moved = await moveAside({ commonDir, record, movedName }); // undefined when the record vanished
//! ```

use crate::json_record::quote;
use crate::random_id::random_uuid;
use crate::rename_that_never_replaces::{rename_noreplace, rename_unsupported};
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use crate::worktree_copy_journal::{ensure_journal_root, ensure_private_directory, journal_root};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// What:
///  The folder under the journal root that keeps the records this release cannot read.
/// Why:
///   A record is the only trace of what an interrupted copy was doing,
/// so it is moved rather than deleted,
/// and one name keeps every one of them out of the readers' way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNREADABLE_FOLDER = 'unreadable';
/// ```
pub const UNREADABLE_FOLDER: &str = "unreadable";

/// How many names one move aside tries before it gives up on a folder that keeps colliding.
/// Exempt from a citation under `LQM` (class:
/// retry bound):
/// each try appends a fresh random identifier,
/// and a folder holding a thousand records of one name is not a state any
/// run produces;
/// the bound keeps a broken random source from looping forever.
const MOVE_ASIDE_TRIES: usize = 16;

/// What:
///  The folder that keeps the records this release cannot read,
/// created when it is not there.
/// Why:
///   The folder is private to this account like every other item of a copy,
/// and creating it on the first move keeps the readers off the cost of one that was never needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await ensureUnreadableRecordsFolder(commonDir);
/// ```
fn unreadable_folder(common_dir: &Path) -> Result<PathBuf, WorktreeCopyError> {
    let root: PathBuf = ensure_journal_root(common_dir)?;
    let folder: PathBuf = root.join(UNREADABLE_FOLDER);
    ensure_private_directory(folder.as_path())?;
    return Ok(folder);
}

/// What:
///  Moves one record that cannot be read into the folder beside the records,
/// under `moved_name`,
/// and returns where it went,
/// or nothing when the record was settled by another command first.
/// Why:
///   The record is the only trace of what the interrupted copy was doing,
/// so it is renamed,
/// never deleted,
/// and never over another record:
/// a name that is taken gets a fresh random suffix rather than replacing what an earlier command kept.
/// The rename is the whole move,
/// so two commands that find the same record settle it once and the other finds it gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const moved = await moveAside({ commonDir, record, movedName }); // undefined when the record vanished
/// ```
pub fn move_aside(common_dir: &Path, record: &Path, moved_name: &OsStr) -> Result<Option<PathBuf>, WorktreeCopyError> {
    let folder: PathBuf = unreadable_folder(common_dir)?;
    let mut target: PathBuf = folder.join(moved_name);
    for attempt in 0..MOVE_ASIDE_TRIES {
        // `match` on the one rename: a vanished record is another command's work, and a taken name keeps both.
        match rename_noreplace(record, target.as_path()) {
            Ok(()) => return Ok(Some(target)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists || (rename_unsupported(&error) && target.exists()) => {
                target = folder.join(suffixed(moved_name, attempt));
            }
            // A filesystem that cannot rename without replacing is served by one checked rename: the folder is private,
            // so no other command puts a name there between the check and the rename.
            Err(error) if rename_unsupported(&error) => {
                std::fs::rename(record, target.as_path()).map_err(|refusal| {
                    return failure_with_cause(
                        format!(
                            "monochromatic-git-policy-cli: the worktree-copy record {} could not be moved to {}.",
                            quote(record.to_string_lossy().as_ref()),
                            quote(target.to_string_lossy().as_ref())
                        )
                        .as_str(),
                        Some(&refusal),
                    );
                })?;
                return Ok(Some(target));
            }
            Err(error) => {
                return Err(failure_with_cause(
                    format!(
                        "monochromatic-git-policy-cli: the worktree-copy record {} could not be moved to {}.",
                        quote(record.to_string_lossy().as_ref()),
                        quote(target.to_string_lossy().as_ref())
                    )
                    .as_str(),
                    Some(&error),
                ));
            }
        }
    }
    return Err(failure_with_cause(
        format!(
            "monochromatic-git-policy-cli: the worktree-copy record {} could not be moved aside: every name it was given in {} is taken.",
            quote(record.to_string_lossy().as_ref()),
            quote(folder.to_string_lossy().as_ref())
        )
        .as_str(),
        None,
    ));
}

/// What:
///  The name `attempt` gives a moved record:
/// its own name the first time,
/// and then its own name with a fresh identifier.
/// Why:
///   A moved record keeps the name that says what it was,
/// and a second record of the same name keeps its own bytes rather than replacing the first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const suffixed = (name, attempt) => attempt === 0 ? name : `${name}.${randomUUID()}`;
/// ```
fn suffixed(moved_name: &OsStr, attempt: usize) -> PathBuf {
    if attempt == 0 {
        return PathBuf::from(moved_name);
    }
    // A random source that fails still gives this try a name of its own, so a run never loops on one taken name.
    let unique: String = match random_uuid() {
        Ok(made) => made,
        Err(_unmade) => format!("{attempt}"),
    };
    let mut name: OsString = moved_name.to_owned();
    name.push(format!(".{unique}"));
    return PathBuf::from(name);
}

/// What:
///  The name of the file one record is moved under,
/// which is the record's own name in every case but a stage's registration record.
/// Why:
///   A moved record keeps the name that says what it was,
/// so the person who opens the folder reads the record the notice named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const movedName = (record) => basename(record);
/// ```
pub fn moved_name_of(record: &Path) -> OsString {
    return record.file_name().map_or_else(OsString::new, OsStr::to_os_string);
}

/// What:
///  The path one moved record would be moved to,
/// which no caller needs but a control that checks the folder.
/// Why:
///   The folder's name and the record's own name are what a person looks for,
/// and a control names the same pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(journalRoot(commonDir), UNREADABLE_FOLDER, movedName)
/// ```
pub fn moved_record_path(common_dir: &Path, moved_name: &OsStr) -> PathBuf {
    return journal_root(common_dir).join(UNREADABLE_FOLDER).join(moved_name);
}

/// What:
///  Whether the folder for records that cannot be read holds anything.
/// Why:
///   A control checks what became of a record this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readdir(journalRoot(commonDir)/UNREADABLE_FOLDER).length > 0
/// ```
pub fn unreadable_records_stand(common_dir: &Path) -> bool {
    return match std::fs::read_dir(journal_root(common_dir).join(UNREADABLE_FOLDER).as_path()) {
        Ok(mut listing) => listing.next().is_some(),
        Err(_absent) => false,
    };
}

/// What:
///  The names of the records the folder keeps,
/// in name order.
/// Why:
///   A control reads which record was moved where without depending on the directory's order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(folder)).sort();
/// ```
pub fn unreadable_record_names(common_dir: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    let folder: PathBuf = journal_root(common_dir).join(UNREADABLE_FOLDER);
    let listing = match std::fs::read_dir(folder.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        names.push(listed.map_err(|error| { return raw_failure(&error); })?.file_name().to_string_lossy().into_owned());
    }
    names.sort_by(crate::worktree_copy_selection::by_js_order);
    return Ok(names);
}
