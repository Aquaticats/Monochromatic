//! What:
//!  The record of the index update one landed batch still owes:
//!  its shape,
//!  and how a batch's intent becomes one.
//! Why:
//!  Two files read this record and neither owns it:
//! the file that settles and removes it,
//!  and the file that decides which of its paths a commit leaves out.
//!  Held at the bottom of the folder,
//! it is what both name instead of naming each other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const owed = owedOf(intent); // { seq, repository, gitDir, index, ref, old, new, paths, derived }
//! ```

use crate::service_journal::Intent;
use crate::work_item_record::WORK_DIRECTORY;
use serde_json::{
    Map,
    Value,
};
use std::path::{
    Path,
    PathBuf,
};

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
    /// for a message that names it,
    /// in the form a person can type.
    pub reference: String,
    /// The branch before the batch.
    pub old: String,
    /// The commit the batch landed.
    pub new: String,
    /// The paths whose entries the batch updates,
    /// as the bytes Git listed:
    /// a file name that is not text is still a file name.
    pub paths: Vec<Vec<u8>>,
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
        reference: intent.reference.spoken(),
        old: intent.old.clone(),
        new: intent.new.clone(),
        paths: intent.paths.clone(),
        derived: intent.derived,
    };
}

/// The record format version.
pub const OWED_VERSION: u64 = 1;

/// The extension of one record;
/// the stem is the batch number.
pub const OWED_EXTENSION: &str = "owed";

/// The folder of the records below the work directory of the Git directory.
const OWED_FOLDER: &str = "owed";

/// The folder of the owed records of one worktree.
pub fn directory_of(git_dir: &Path) -> PathBuf {
    return git_dir.join(WORK_DIRECTORY).join(OWED_FOLDER);
}

/// The path of one record.
pub fn path_of(
    git_dir: &Path,
    seq: u64,
) -> PathBuf {
    return directory_of(git_dir).join(format!("{seq}.{OWED_EXTENSION}"));
}

/// The text of a record.
pub(crate) fn encode(owed: &Owed) -> String {
    let mut fields: Map<String, Value> = Map::new();
    fields.insert(String::from("v"), Value::from(OWED_VERSION));
    fields.insert(String::from("seq"), Value::from(owed.seq));
    // A path need not be text, so every path field holds the bytes beside the text the older records carry.
    crate::path_bytes_in_records::write_field(&mut fields, "repository", owed.repository.as_path());
    crate::path_bytes_in_records::write_field(&mut fields, "gitDir", owed.git_dir.as_path());
    crate::path_bytes_in_records::write_field(&mut fields, "index", owed.index.as_path());
    fields.insert(String::from("ref"), Value::from(owed.reference.clone()));
    fields.insert(String::from("old"), Value::from(owed.old.clone()));
    fields.insert(String::from("new"), Value::from(owed.new.clone()));
    // A path of the batch need not be text, so the record holds each one's bytes.
    let mut paths_as_bytes: Vec<String> = Vec::with_capacity(owed.paths.len());
    for path in &owed.paths {
        paths_as_bytes.push(crate::path_bytes_in_records::hex_of_bytes(path.as_slice()));
    }
    fields.insert(String::from("pathsHex"), Value::from(paths_as_bytes));
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
    let index: PathBuf = crate::path_bytes_in_records::read_field(&value, "index")?;
    let repository: PathBuf = crate::path_bytes_in_records::read_field(&value, "repository")?;
    let git_dir: PathBuf = crate::path_bytes_in_records::read_field(&value, "gitDir")?;
    let mut paths: Vec<Vec<u8>> = Vec::new();
    for entry in value
        .get("pathsHex")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(digits) = entry.as_str() else {
            return Err(String::from("the record holds a path that is not text"));
        };
        let Some(path) = crate::path_bytes_in_records::path_of_hex(digits) else {
            return Err(String::from(
                "the record holds a path whose bytes do not decode",
            ));
        };
        paths.push(crate::path_bytes::path_bytes_of(path.as_path()).to_vec());
    }
    return Ok(Owed {
        seq,
        repository,
        git_dir,
        index,
        reference: field_text(&value, "ref").unwrap_or_default(),
        old: field_text(&value, "old").unwrap_or_default(),
        new,
        paths,
        derived: value
            .get("derived")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    });
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
    return records_and_unreadable(git_dir).0;
}

/// The extension of a record whose obligation the landing finished.
/// It holds nothing,
/// and a command that meets one removes it:
/// the rename to it is what says the obligation ended,
/// so a kill between the installation of the index and the
///  rename leaves a record that is still owed,
/// never one that is owed and already settled.
pub const SETTLED_EXTENSION: &str = "settled";

/// What:
///  The owed records of one worktree,
/// and how many records are there that cannot be read.
/// Why:
///   A record that exists and cannot be read is still an obligation:
/// a caller that took its absence from the
///  list for "nothing is owed" would leave the entries it names unprotected.
/// The count is how the caller says it could
///  not tell instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordsAndUnreadable(gitDir) // [records, unreadable]
/// ```
pub fn records_and_unreadable(git_dir: &Path) -> (Vec<Owed>, usize) {
    let Ok(listing) = std::fs::read_dir(directory_of(git_dir)) else {
        return (Vec::new(), 0);
    };
    let mut unreadable: usize = 0;
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
        match crate::plain_file::read_plain_text(path.as_path()).map(decode_text) {
            Ok(Ok(owed)) => found.push(owed),
            Ok(Err(reason)) => {
                crate::service_error!(
                    "the record of an owed index update at {} holds none and is kept: {reason}",
                    path.display()
                );
                unreadable += 1;
            }
            Err(error) => {
                crate::service_error!(
                    "the record of an owed index update at {} cannot be read ({error}) and is kept",
                    path.display()
                );
                unreadable += 1;
            }
        }
    }
    found.sort_by_key(batch_number_of);
    return (found, unreadable);
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
fn field_text(
    value: &Value,
    key: &str,
) -> Option<String> {
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
