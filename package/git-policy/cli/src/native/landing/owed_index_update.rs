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
use crate::service_journal::Intent;
use crate::service_protocol::FailureCode;
use crate::transaction_git::{GitContext, GitRequest, run_git};
use crate::work_item::WORK_DIRECTORY;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
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
    let text_of = |key: &str| -> Option<String> { return value.get(key).and_then(Value::as_str).map(str::to_string); };
    let Some(seq) = value.get("seq").and_then(Value::as_u64) else {
        return Err(String::from("the record names no batch"));
    };
    let Some(new) = text_of("new") else {
        return Err(String::from("the record names no landed commit"));
    };
    let Some(index) = text_of("index") else {
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
        repository: PathBuf::from(text_of("repository").unwrap_or_default()),
        git_dir: PathBuf::from(text_of("gitDir").unwrap_or_default()),
        index: PathBuf::from(index),
        reference: text_of("ref").unwrap_or_default(),
        old: text_of("old").unwrap_or_default(),
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
    if std::fs::read_to_string(path.as_path()).is_ok_and(|existing| { return existing == text; }) {
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

/// Removes the record of one batch;
/// a record that cannot be removed stays for the next command to find.
pub fn remove(git_dir: &Path, seq: u64) {
    let _removed: std::io::Result<()> = std::fs::remove_file(path_of(git_dir, seq));
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
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(OWED_EXTENSION) {
            continue;
        }
        match std::fs::read_to_string(path.as_path()).map(|text| { return decode(text.as_str()); }) {
            Ok(Ok(owed)) => found.push(owed),
            Ok(Err(reason)) => crate::service_error!("the record of an owed index update at {} holds none and is kept: {reason}", path.display()),
            Err(error) => crate::service_error!("the record of an owed index update at {} cannot be read ({error}) and is kept", path.display()),
        }
    }
    found.sort_by_key(|owed| { return owed.seq; });
    return found;
}

/// One entry of a tree or an index:
/// its mode and its object.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Entry {
    mode: String,
    object: String,
}

/// What:
///  The entries a tree holds at `names`,
/// each as the mode and object a comparison with an index needs.
/// Why:
///   A repair resets a path only while the index still holds what the batch started from,
/// and that is this value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// treeEntries(tree, names) // `ls-tree -z --full-tree <tree> -- :(top,literal)<name>...`
/// ```
fn tree_entries(git: &GitContext, repository: &Path, tree: &str, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    let specs: Vec<std::ffi::OsString> = names.iter().map(|name| { return literal_spec(name); }).collect();
    let mut arguments: Vec<&std::ffi::OsStr> = vec![std::ffi::OsStr::new("ls-tree"), std::ffi::OsStr::new("-z"), std::ffi::OsStr::new("--full-tree"), std::ffi::OsStr::new(tree), std::ffi::OsStr::new("--")];
    arguments.extend(specs.iter().map(std::ffi::OsString::as_os_str));
    let output = run(git, repository, arguments.as_slice(), None)?;
    return Ok(entries_of(output.stdout.as_slice(), TabShape::Tree));
}

/// The path the record's paths become,
/// which is where the repair is told which entries to look at.
fn literal_spec(name: &[u8]) -> std::ffi::OsString {
    let mut spec: Vec<u8> = b":(top,literal)".to_vec();
    spec.extend_from_slice(name);
    return <std::ffi::OsString as std::os::unix::ffi::OsStringExt>::from_vec(spec);
}

/// Which of the two listings a line belongs to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TabShape {
    /// `<mode> SP <type> SP <object> TAB <path>`,
    /// as `ls-tree -z` writes it.
    Tree,
    /// `<mode> SP <object> SP <stage> TAB <path>`,
    /// as `ls-files -s -z` writes it;
    /// a stage other than zero is an unmerged entry,
    /// which is the person's own.
    Index,
}

/// The entries a NUL-terminated listing holds,
/// split at its words and its tab.
fn entries_of(listing: &[u8], shape: TabShape) -> BTreeMap<Vec<u8>, Entry> {
    let mut found: BTreeMap<Vec<u8>, Entry> = BTreeMap::new();
    for record in listing.split(|byte| { return *byte == 0; }) {
        if record.is_empty() {
            continue;
        }
        let Some(tab) = record.iter().position(|byte| { return *byte == b'\t'; }) else {
            continue;
        };
        let words: Vec<&[u8]> = record[..tab].split(|byte| { return *byte == b' '; }).collect();
        let name: Vec<u8> = record[tab + 1..].to_vec();
        // An unmerged path is not the landing's leftover, whatever else the index holds for it.
        if shape == TabShape::Index && words.last().is_some_and(|stage| { return *stage != b"0"; }) {
            let _dropped = found.remove(name.as_slice());
            continue;
        }
        // A tree line is `<mode> <type> <object>` and an index line is `<mode> <object> <stage>`.
        let object: usize = if shape == TabShape::Tree { 2 } else { 1 };
        found.insert(name, Entry { mode: String::from_utf8_lossy(words.first().copied().unwrap_or_default()).into_owned(), object: String::from_utf8_lossy(words.get(object).copied().unwrap_or_default()).into_owned() });
    }
    return found;
}

/// The entries one index file holds at `names`.
fn index_entries(git: &GitContext, repository: &Path, index: &Path, names: &[Vec<u8>]) -> Result<BTreeMap<Vec<u8>, Entry>, FailureCode> {
    let specs: Vec<std::ffi::OsString> = names.iter().map(|name| { return literal_spec(name); }).collect();
    let mut arguments: Vec<&std::ffi::OsStr> = vec![std::ffi::OsStr::new("ls-files"), std::ffi::OsStr::new("-s"), std::ffi::OsStr::new("-z"), std::ffi::OsStr::new("--")];
    arguments.extend(specs.iter().map(std::ffi::OsString::as_os_str));
    let mut request: GitRequest = GitRequest::new(repository, &arguments);
    request.index = Some(index.to_path_buf());
    let output = match run_git(git, &request) {
        Ok(answer) if answer.succeeded() => answer,
        _ => return Err(FailureCode::Unavailable),
    };
    return Ok(entries_of(output.stdout.as_slice(), TabShape::Index));
}

/// The paths a record's update covers,
/// derived from the two branch values when the batch names none.
fn names_of(git: &GitContext, owed: &Owed) -> Result<Vec<Vec<u8>>, FailureCode> {
    if !owed.derived {
        return Ok(owed.paths.iter().map(|path| { return path.clone().into_bytes(); }).collect());
    }
    let names: Vec<Vec<u8>> = crate::service_land_follow::derived_names(git, owed.repository.as_path(), owed.old.as_str(), owed.new.as_str())?
        .split(|byte| { return *byte == 0; })
        .filter(|name| { return !name.is_empty(); })
        .map(<[u8]>::to_vec)
        .collect();
    return Ok(names);
}

/// What:
///  The paths of a record whose entries in `index` still hold what the batch started from,
/// which are the entries the landing left.
/// Why:
///   Only those may be reset:
/// a path the person staged since the landing holds something else,
/// and resetting it
///  would discard their staging.
/// A path whose old entry does not exist is the batch's own addition,
/// and an index with no entry
///  there matches it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// leftovers(owed, index)
/// ```
pub fn leftovers(git: &GitContext, owed: &Owed, index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let names: Vec<Vec<u8>> = names_of(git, owed)?;
    if names.is_empty() {
        return Ok(names);
    }
    let started: BTreeMap<Vec<u8>, Entry> = tree_entries(git, owed.repository.as_path(), owed.old.as_str(), names.as_slice())?;
    let current: BTreeMap<Vec<u8>, Entry> = index_entries(git, owed.repository.as_path(), index, names.as_slice())?;
    let mut left: Vec<Vec<u8>> = Vec::new();
    for name in names {
        if current.get(name.as_slice()) == started.get(name.as_slice()) {
            left.push(name);
        }
    }
    return Ok(left);
}

/// What:
///  Settles one owed record for one index:
/// the entries the landing left are reset to the commit the batch landed,
/// and the ones a program staged since are left,
/// with the ones reset given back.
/// Why:
///   The reset is the landing's own idempotent step (`service_land_follow.rs`),
/// made here through Git's own lock,
/// so a concurrent
///  staging of a path whose entry is not the landing's is never a candidate,
/// and a concurrent staging of one that is ends
///  in the one thing Git's own `reset` does with a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reset = await settleLeftovers(owed, realIndex); // the paths this settled
/// ```
pub fn settle_leftovers(git: &GitContext, owed: &Owed, index: &Path) -> Result<Vec<Vec<u8>>, FailureCode> {
    let left: Vec<Vec<u8>> = leftovers(git, owed, index)?;
    if !left.is_empty() {
        crate::service_land_follow::reset_entries(git, owed.repository.as_path(), Some(index), owed.new.as_str(), left.as_slice())?;
    }
    return Ok(left);
}

/// Runs one Git step,
/// naming no answer as a failure the caller can classify.
fn run(git: &GitContext, repository: &Path, arguments: &[&std::ffi::OsStr], input: Option<Vec<u8>>) -> Result<crate::transaction_git::GitOutput, FailureCode> {
    let mut request: GitRequest = GitRequest::new(repository, arguments);
    request.input = input;
    return match run_git(git, &request) {
        Ok(output) if output.succeeded() => Ok(output),
        _ => Err(FailureCode::Unavailable),
    };
}
