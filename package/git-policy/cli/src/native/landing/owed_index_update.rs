//! What:
//!  The record of an index update a landed batch still owes,
//! written in the worktree's own Git directory where both the service and
//!  the person's next command read it,
//! and the repair that repeats the update for the entries the landing itself left.
//! Why:
//!   A batch whose branch moved and whose index update did not finish is finished by a restart,
//! and the journal's intent names the paths,
//!  the index and the commit.
//! The intent is one file of many fields,
//! and a build that cannot read it finishes nothing:
//! a person's
//!  next commit then records a file the landed commit added as a staged deletion,
//! because the index still describes the branch before
//!  the landing (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
//! finding 1 and finding 2).
//! This record is the
//!  obligation on its own:
//! small,
//! fixed in shape,
//! and beside the worktree the update belongs to,
//! so it outlives the intent and is read by the command
//!  that would record the deletion.
//!   The repair resets only the entries that still hold what the batch started from,
//! so staging that is the person's
//!  own is kept,
//! and a path the person staged since the landing is named and left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeOwed({ gitDir, index, old, new, paths }); const left = await leftovers(owed, realIndex); await reset(left);
//! ```

use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
pub use crate::owed_index_repair::{exclude_what_a_landed_commit_left, excluded_notice};
use crate::service_journal::Intent;
use crate::work_item::WORK_DIRECTORY;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// The record format version.
pub const OWED_VERSION: u64 = 1;

/// The extension of one record;
/// the stem is the batch number.
pub const OWED_EXTENSION: &str = "owed";

/// The folder of the records below the work directory of the Git directory.
const OWED_FOLDER: &str = "owed";

/// What:
///  The index update one landed batch still owes.
/// Why:
///   It is an intent's index obligations alone:
/// which index,
///  from which branch value to which commit,
/// and which paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Owed = { seq; repository; gitDir; index; ref; old; new; paths; derived };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Owed {
    /// The batch whose landing owes the update.
    pub seq: u64,
    /// The worktree root whose index the batch updates.
    pub repository: PathBuf,
    /// The worktree's own Git directory,
    /// which holds this record.
    pub git_dir: PathBuf,
    /// The index file the update belongs to.
    pub index: PathBuf,
    /// The branch,
    /// for a message that names it.
    pub reference: String,
    /// The branch before the batch.
    pub old: String,
    /// The commit the batch landed.
    pub new: String,
    /// The paths whose entries the batch updates.
    pub paths: Vec<String>,
    /// Whether the paths are every path that differs between `old` and `new`.
    pub derived: bool,
}

/// What:
///  The record of an intent's owed index update.
/// Why:
///   The landing writes it from the intent it already has;
/// the fields are the ones a repair reads,
/// and nothing else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// owedOf(intent)
/// ```
pub fn of_intent(intent: &Intent) -> Owed {
    return Owed {
        seq: intent.seq,
        repository: intent.repository.clone(),
        git_dir: intent.git_dir.clone(),
        index: intent.index.clone(),
        reference: intent.reference.clone(),
        old: intent.old.clone(),
        new: intent.new.clone(),
        paths: intent.paths.clone(),
        derived: intent.derived,
    };
}

/// The folder of the owed records of one worktree.
pub fn directory_of(git_dir: &Path) -> PathBuf {
    return git_dir.join(WORK_DIRECTORY).join(OWED_FOLDER);
}

/// The path of one record.
pub fn path_of(git_dir: &Path, seq: u64) -> PathBuf {
    return directory_of(git_dir).join(format!("{seq}.{OWED_EXTENSION}"));
}

/// The text of a record.
fn encode(owed: &Owed) -> String {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("v"), Value::from(OWED_VERSION));
    fields.insert(String::from("seq"), Value::from(owed.seq));
    fields.insert(String::from("repository"), Value::from(owed.repository.display().to_string()));
    fields.insert(String::from("gitDir"), Value::from(owed.git_dir.display().to_string()));
    fields.insert(String::from("index"), Value::from(owed.index.display().to_string()));
    fields.insert(String::from("ref"), Value::from(owed.reference.clone()));
    fields.insert(String::from("old"), Value::from(owed.old.clone()));
    fields.insert(String::from("new"), Value::from(owed.new.clone()));
    fields.insert(String::from("paths"), Value::from(owed.paths.clone()));
    fields.insert(String::from("derived"), Value::from(owed.derived));
    return format!("{}\n", Value::Object(fields));
}

/// The record a file's text holds,
/// or why it holds none.
fn decode(text: &str) -> Result<Owed, String> {
    let value: Value = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => return Err(format!("the record is not JSON: {error}")),
    };
    if value.get("v").and_then(Value::as_u64) != Some(OWED_VERSION) {
        return Err(String::from("the record has an unsupported version"));
    }
    let Some(seq) = value.get("seq").and_then(Value::as_u64) else {
        return Err(String::from("the record names no batch"));
    };
    let Some(new) = field_text(&value, "new") else {
        return Err(String::from("the record names no landed commit"));
    };
    let Some(index) = field_text(&value, "index") else {
        return Err(String::from("the record names no index file"));
    };
    let mut paths: Vec<String> = Vec::new();
    for entry in value.get("paths").and_then(Value::as_array).into_iter().flatten() {
        let Some(path) = entry.as_str() else {
            return Err(String::from("the record holds a path that is not text"));
        };
        paths.push(path.to_string());
    }
    return Ok(Owed {
        seq,
        repository: PathBuf::from(field_text(&value, "repository").unwrap_or_default()),
        git_dir: PathBuf::from(field_text(&value, "gitDir").unwrap_or_default()),
        index: PathBuf::from(index),
        reference: field_text(&value, "ref").unwrap_or_default(),
        old: field_text(&value, "old").unwrap_or_default(),
        new,
        paths,
        derived: value.get("derived").and_then(Value::as_bool).unwrap_or(false),
    });
}

/// What:
///  Records that a batch's index update is owed,
/// before anything tries to install it.
/// Why:
///   A kill,
/// a failing step or a restart that declines to replay leaves the obligation here,
/// where the person's own next
///  command reads it.
/// The write is not flushed:
/// what a power loss loses is the intent's own record too,
/// and a record that is
///  lost costs the notice,
/// never the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeOwed(owed) // the path, or nothing when the folder cannot be made
/// ```
pub fn write(owed: &Owed) -> Option<PathBuf> {
    let path: PathBuf = path_of(owed.git_dir.as_path(), owed.seq);
    let folder: PathBuf = directory_of(owed.git_dir.as_path());
    if let Err(error) = create_private_directory_all(folder.as_path()) {
        crate::service_error!("batch {}: the record of its owed index update cannot be made in {} ({error})", owed.seq, folder.display());
        return None;
    }
    // A replay writes what the first landing wrote, and a batch built again under its own number replaces the commit it names: an
    //  identical record is left as it is and anything else goes before the new one is published.
    let text: String = encode(owed);
    let unchanged: bool = match std::fs::read_to_string(path.as_path()) {
        Ok(existing) => existing == text,
        Err(_) => false,
    };
    if unchanged {
        return Some(path);
    }
    let _gone: std::io::Result<()> = std::fs::remove_file(path.as_path());
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Some(path),
        Err(error) => {
            crate::service_error!("batch {}: the record of its owed index update cannot be written in {} ({error})", owed.seq, path.display());
            None
        }
    };
}

/// The extension of a record whose obligation the landing finished.
/// It holds nothing,
/// and a command that meets one removes it:
/// the rename to it is what says the obligation ended,
/// so a kill between the installation of the index and the
///  rename leaves a record that is still owed,
/// never one that is owed and already settled.
pub const SETTLED_EXTENSION: &str = "settled";

/// The path that says one batch's obligation is over.
pub fn settled_path_of(git_dir: &Path, seq: u64) -> PathBuf {
    return directory_of(git_dir).join(format!("{seq}.{SETTLED_EXTENSION}"));
}

/// What:
///  Says that a batch's owed index update is settled.
/// Why:
///   The rename is atomic,
/// so a command that finds the owed name still sees an obligation,
/// and one that finds the
///  settled name sees that there is none.
/// The settled file is removed at once;
/// one a kill leaves is removed by the next command,
/// and it
///  carries no obligation either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settle(gitDir, seq) // rename `.owed` to `.settled`, then unlink
/// ```
pub fn settle(git_dir: &Path, seq: u64) {
    let settled: PathBuf = settled_path_of(git_dir, seq);
    if std::fs::rename(path_of(git_dir, seq), settled.as_path()).is_ok() {
        let _removed: std::io::Result<()> = std::fs::remove_file(settled.as_path());
    }
}

/// Removes the record of one batch,
/// settled or not;
/// a record that cannot be removed stays for the next command to find.
pub fn remove(git_dir: &Path, seq: u64) {
    let _removed: std::io::Result<()> = std::fs::remove_file(path_of(git_dir, seq));
    let _settled: std::io::Result<()> = std::fs::remove_file(settled_path_of(git_dir, seq));
}

/// What:
///  Every owed record of one worktree,
/// in batch order,
/// with the ones that cannot be read said and left.
/// Why:
///   A record that cannot be read is no proof that nothing is owed,
/// so it is kept and named,
/// and the repair of what can be read goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordsOf(gitDir)
/// ```
pub fn records_of(git_dir: &Path) -> Vec<Owed> {
    let Ok(listing) = std::fs::read_dir(directory_of(git_dir)) else {
        return Vec::new();
    };
    let mut found: Vec<Owed> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        match path.extension().and_then(std::ffi::OsStr::to_str) {
            Some(OWED_EXTENSION) => {}
            // The remains of a settled obligation: nothing is owed, and it goes.
            Some(SETTLED_EXTENSION) => {
                let _removed: std::io::Result<()> = std::fs::remove_file(path.as_path());
                continue;
            }
            _ => continue,
        }
        match std::fs::read_to_string(path.as_path()).map(decode_text) {
            Ok(Ok(owed)) => found.push(owed),
            Ok(Err(reason)) => crate::service_error!("the record of an owed index update at {} holds none and is kept: {reason}", path.display()),
            Err(error) => crate::service_error!("the record of an owed index update at {} cannot be read ({error}) and is kept", path.display()),
        }
    }
    found.sort_by_key(batch_number_of);
    return found;
}

/// What:
///  The text of one field of a record,
/// nothing when it holds no text there.
/// Why:
///  `decode` reads most of its fields by name through this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldText = (record, key) => typeof record[key] === 'string' ? record[key] : undefined;
/// ```
fn field_text(value: &Value, key: &str) -> Option<String> {
    return value.get(key).and_then(Value::as_str).map(str::to_string);
}

/// What a record's file holds,
/// decoded.
fn decode_text(text: String) -> Result<Owed, String> {
    return decode(text.as_str());
}

/// The batch number a record is kept under,
/// which orders the records of one worktree.
fn batch_number_of(owed: &Owed) -> u64 {
    return owed.seq;
}

/// The batch number a journal file names,
/// read from its own name.
pub fn batch_number_of_path(path: &Path) -> String {
    return match path.file_stem() {
        Some(stem) => stem.to_string_lossy().into_owned(),
        None => String::new(),
    };
}
