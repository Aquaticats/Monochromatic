//! What:
//!  The journal of a subvolume migration:
//! a small JSON file beside the checkout that records which steps finished,
//!  written atomically after each step.
//! Why:
//!  A rerun after a kill at any instant resumes from it:
//! a step marked `done` is skipped,
//! a step marked `started` is
//!  repeated (every step is idempotent),
//! and a step with no entry has not begun.
//! The file lives in the checkout's
//!  parent directory (`.cli-git-migrate-<name>.journal.json`),
//! outside the tree that is copied and exchanged,
//! so it
//!  survives the exchange.
//! It is replaced by writing a sibling file,
//! created exclusively under a name no other file has
//!  (so nothing planted or left at a fixed name is truncated or followed),
//! and renaming it,
//! so a kill never leaves half a
//!  journal.
//! The plain files a killed run left under the writer's own temporary names are removed by the next write that
//!  lands.
//! `serde_json` is already a dependency of the crate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const journal = await loadJournal(path); await journal.mark('copy', 'started'); ... await journal.mark('copy', 'done');
//! ```

use crate::iso_time::{format_iso_milliseconds, now_iso};
use crate::plain_file::read_plain_text;
use crate::subvolume_error::MigrateError;
use crate::subvolume_file_write_steps::{sync_directory_to_disk, write_all_then_sync};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// The journal format version this code writes and reads.
const SCHEMA: u64 = 1;

/// The permissions of the journal:
/// its owner reads and writes it.
const PRIVATE_FILE_MODE: u32 = 0o600;

/// What follows the journal's file name in the name of a temporary file:
/// releases up to 0.5.20 wrote exactly this suffix,
/// and
///  now a process number and a clock reading follow it.
const TEMPORARY_SUFFIX: &str = ".tmp";

/// What:
///  Whether `candidate` is a name under which a writer of the journal `journal_name` leaves a temporary file:
/// the journal's
///  name,
/// `.tmp`,
/// and either nothing (the name of releases up to 0.5.20) or a dot,
/// a process number,
/// a dot and a clock reading.
/// Why:
///   Any other name beside the journal is not this writer's to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftover = new RegExp(`^${escape(journalName)}\\.tmp(\\.\\d+\\.\\d+)?$`).test(candidate);
/// ```
fn is_leftover_name(candidate: &[u8], journal_name: &[u8]) -> bool {
    let Some(after_name) = candidate.strip_prefix(journal_name) else { return false };
    let Some(rest) = after_name.strip_prefix(TEMPORARY_SUFFIX.as_bytes()) else { return false };
    if rest.is_empty() {
        return true;
    }
    let Some(numbers) = rest.strip_prefix(b".") else { return false };
    let mut parts = numbers.split(is_dot);
    let process: Option<&[u8]> = parts.next();
    let clock: Option<&[u8]> = parts.next();
    return is_number(process) && is_number(clock) && parts.next().is_none();
}

/// Whether `byte` is a dot,
/// the separator inside the numbers of a temporary name.
fn is_dot(byte: &u8) -> bool {
    return *byte == b'.';
}

/// Whether the part exists and is a non-empty run of ASCII digits.
fn is_number(part: Option<&[u8]>) -> bool {
    return match part {
        Some(digits) => !digits.is_empty() && digits.iter().all(u8::is_ascii_digit),
        None => false,
    };
}

/// The error for a journal file that cannot be understood:
/// it says what is wrong with the file at `path`.
fn broken_journal(path: &Path, why: &str) -> MigrateError {
    return MigrateError::Journal(format!("the journal {} {why}; read it, and delete it only if no migration is in progress", path.display()));
}

/// Write `bytes` to a new file at `sibling`,
/// rename it over `target`,
/// and make the rename durable with the directory that holds the new name.
fn write_and_rename(sibling: &Path, target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    write_exclusively(sibling, bytes)?;
    if let Err(error) = std::fs::rename(sibling, target) {
        let _removed: std::io::Result<()> = std::fs::remove_file(sibling);
        return Err(error);
    }
    // The rename is made durable with the directory that holds the new name.
    return match target.parent() {
        Some(directory) => sync_directory_to_disk(directory),
        None => Ok(()),
    };
}

/// What:
///  Write `bytes` to a file made for them alone at `path`:
/// created exclusively,
/// so a path that already exists (a plain file,
///  a link,
/// a dangling link) is refused and never truncated or followed;
/// synchronized;
/// removed again when the write fails.
/// Why:
///   A fixed temporary name that an earlier run or another user left or planted must not redirect the write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, bytes, { flag: 'wx', mode: 0o600 }); // rejects with EEXIST on anything there, a link included
/// ```
fn write_exclusively(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(PRIVATE_FILE_MODE).open(path)?;
    let written: std::io::Result<()> = write_all_then_sync(&mut file, bytes);
    if let Err(error) = written {
        let _removed: std::io::Result<()> = std::fs::remove_file(path);
        return Err(error);
    }
    return Ok(());
}

/// What:
///  The records of one migration.
///  `BTreeMap<String, String>` is a text-to-text dictionary kept in key order (sibling `HashMap` has no order).
/// Why:
///   A sorted map writes the same file text for the same records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Journal = { path: string; started: string; steps: Record<string, { state: string; at: string }> };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Journal {
    /// Where the file is (or will be).
    path: PathBuf,
    /// When the migration began,
    /// as `toISOString` text.
    started: String,
    /// Step name to state (`started`,
    /// `done` or `rolled-back`).
    states: BTreeMap<String, String>,
    /// Step name to the time in milliseconds after the epoch at which it got its current state.
    times: BTreeMap<String, i64>,
    /// The identity (device,
    /// inode and,
    /// where the filesystem gives one,
    /// birth time) of the subvolume this migration
    ///  made at the stage's name;
    /// absent in a journal written by an earlier release,
    /// which recorded none.
    stage_identity: Option<crate::fs_beneath::ObjectId>,
    /// Whether the file exists on disk.
    persisted: bool,
}

impl Journal {
    /// What:
    ///  Load the journal at `path`,
    /// or start an empty one when there is no file.
    /// Why:
    ///   A file that exists but is not a journal of this format is an error naming the file:
    /// guessing would resume
    ///  or skip steps on a false record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const journal = await Journal.load(path);
    /// ```
    pub fn load(path: &Path) -> Result<Journal, MigrateError> {
        let text: String = match read_plain_text(path) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Journal { path: path.to_path_buf(), started: now_iso(), states: BTreeMap::new(), times: BTreeMap::new(), stage_identity: None, persisted: false });
            }
            Err(error) => return Err(MigrateError::io("read the journal", path, &error)),
        };
        let value: Value = match serde_json::from_str(text.as_str()) {
            Ok(parsed) => parsed,
            Err(error) => return Err(broken_journal(path, format!("is not valid JSON ({error})").as_str())),
        };
        if value.get("schema").and_then(Value::as_u64) != Some(SCHEMA) {
            return Err(broken_journal(path, "has an unknown schema version"));
        }
        let Some(started_text) = value.get("started").and_then(Value::as_str) else {
            return Err(broken_journal(path, "has no start time"));
        };
        let started: String = started_text.to_string();
        let Some(steps) = value.get("steps").and_then(Value::as_object) else {
            return Err(broken_journal(path, "has no steps"));
        };
        // A journal this release wrote names the stage it made; one an earlier release wrote names none, and the rollback
        // then has only the stage's type to judge it by.
        let stage_identity: Option<crate::fs_beneath::ObjectId> = match value.get("stage_identity").and_then(Value::as_str) {
            None => None,
            Some(encoded) => match crate::fs_beneath::ObjectId::decode(encoded) {
                Some(identity) => Some(identity),
                None => return Err(broken_journal(path, "has a stage identity that is not one")),
            },
        };
        let mut states: BTreeMap<String, String> = BTreeMap::new();
        let mut times: BTreeMap<String, i64> = BTreeMap::new();
        for (name, record) in steps {
            let Some(state) = record.get("state").and_then(Value::as_str) else {
                return Err(broken_journal(path, "has a step without a state"));
            };
            states.insert(name.clone(), state.to_string());
            if let Some(milliseconds) = record.get("epoch_ms").and_then(Value::as_i64) {
                times.insert(name.clone(), milliseconds);
            }
        }
        return Ok(Journal { path: path.to_path_buf(), started, states, times, stage_identity, persisted: true });
    }

    /// When the migration began,
    /// as `toISOString` text;
    /// unique to the migration and kept by a resumed run.
    pub fn started(&self) -> &str {
        return self.started.as_str();
    }

    /// The journal file's path.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }

    /// Whether any step has been recorded on disk.
    pub fn is_persisted(&self) -> bool {
        return self.persisted;
    }

    /// The time,
    /// in whole seconds after the epoch,
    /// at which the step got its current state;
    /// nothing for a step with no
    ///  record or a record written without a time.
    pub fn seconds_of(&self, step: &str) -> Option<i64> {
        let milliseconds: &i64 = self.times.get(step)?;
        return Some(milliseconds.div_euclid(1000));
    }

    /// The recorded state of a step,
    /// if any.
    pub fn state(&self, step: &str) -> Option<&str> {
        return self.states.get(step).map(String::as_str);
    }

    /// Whether the step finished.
    pub fn is_done(&self, step: &str) -> bool {
        return self.state(step) == Some("done");
    }

    /// Whether the step began (finished or not).
    pub fn has_begun(&self, step: &str) -> bool {
        return matches!(self.state(step), Some("started" | "done"));
    }

    /// What:
    ///  Record a step's state and write the file.
    /// Why:
    ///   The file is written to a sibling created exclusively (under a name of its own,
    /// never one an earlier run or another
    ///  user could have planted a link at) and renamed over the journal,
    /// with the data synced first,
    /// so a kill leaves either
    ///  the old or the new journal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await journal.mark('copy', 'done');
    /// ```
    pub fn mark(&mut self, step: &str, state: &str) -> Result<(), MigrateError> {
        self.states.insert(step.to_string(), state.to_string());
        let now: i64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX),
            Err(_) => 0,
        };
        self.times.insert(step.to_string(), now);
        return self.persist();
    }

    /// What:
    ///  Record the identity of the subvolume this migration made at the stage's name,
    /// and write the journal.
    /// Why:
    ///   The rollback deletes the stage,
    /// and a subvolume another process puts at that name is not the one this
    ///  migration made;
    /// the identity recorded before anything is copied into the stage is what tells them apart.
    pub fn record_stage_identity(&mut self, identity: crate::fs_beneath::ObjectId) -> Result<(), MigrateError> {
        self.stage_identity = Some(identity);
        return self.persist();
    }

    /// The identity this migration recorded for the stage,
    /// if any.
    pub fn stage_identity(&self) -> Option<crate::fs_beneath::ObjectId> {
        return self.stage_identity;
    }

    /// Write the journal as it stands,
    /// through a sibling created exclusively and renamed over it.
    fn persist(&mut self) -> Result<(), MigrateError> {
        let now: i64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX),
            Err(_) => 0,
        };
        let mut steps: Map<String, Value> = Map::new();
        for (name, recorded) in &self.states {
            let milliseconds: i64 = self.times.get(name).copied().unwrap_or(now);
            let mut record: Map<String, Value> = Map::new();
            record.insert(String::from("state"), Value::String(recorded.clone()));
            record.insert(String::from("at"), Value::String(format_iso_milliseconds(milliseconds)));
            record.insert(String::from("epoch_ms"), Value::from(milliseconds));
            steps.insert(name.clone(), Value::Object(record));
        }
        let mut document: Map<String, Value> = Map::new();
        document.insert(String::from("schema"), Value::from(SCHEMA));
        document.insert(String::from("started"), Value::String(self.started.clone()));
        document.insert(String::from("steps"), Value::Object(steps));
        if let Some(identity) = &self.stage_identity {
            document.insert(String::from("stage_identity"), Value::String(identity.encode()));
        }
        let text: String = serde_json::to_string_pretty(&Value::Object(document)).unwrap_or_default();
        let sibling_path: PathBuf = self.temporary_path();
        let written: std::io::Result<()> = write_and_rename(sibling_path.as_path(), self.path.as_path(), text.as_bytes());
        MigrateError::from_io_result(written, "write the journal", self.path.as_path())?;
        self.persisted = true;
        self.remove_leftovers();
        return Ok(());
    }

    /// A name for this write's temporary file that no other file has:
    /// this process and the clock make it,
    /// and the
    ///  creation is exclusive.
    fn temporary_path(&self) -> PathBuf {
        let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => elapsed.as_nanos(),
            Err(_) => 0,
        };
        let mut name: std::ffi::OsString = self.path.as_os_str().to_os_string();
        name.push(format!("{TEMPORARY_SUFFIX}.{}.{nanos}", std::process::id()));
        return PathBuf::from(name);
    }

    /// What:
    ///  Remove the plain files a killed run left beside the journal under the writer's own temporary names.
    /// Why:
    ///   Only the writer's own shapes are removed,
    /// and only plain files as the directory entry itself says (a link,
    /// a
    ///  directory or any other name is somebody else's and stays),
    /// so nothing is followed and nothing foreign is discarded.
    ///  One command at a time works on a checkout (a numbered claim file),
    /// so no live writer's temporary file is among them.
    ///  A failure to list or remove is not an error:
    /// a leftover costs only space.
    fn remove_leftovers(&self) {
        let (Some(directory), Some(journal_name)) = (self.path.parent(), self.path.file_name()) else { return };
        let Ok(listing) = std::fs::read_dir(directory) else { return };
        for item in listing.flatten() {
            let name: std::ffi::OsString = item.file_name();
            if !is_leftover_name(name.as_encoded_bytes(), journal_name.as_encoded_bytes()) {
                continue;
            }
            let is_plain_file: bool = match item.file_type() {
                Ok(kind) => kind.is_file(),
                Err(_) => false,
            };
            if is_plain_file {
                let _removed: std::io::Result<()> = std::fs::remove_file(item.path());
            }
        }
    }

    /// What:
    ///  Delete the journal file.
    /// Why:
    ///   A rolled-back or fully removed migration leaves no record,
    /// so the next migration starts clean.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await rm(journal.path, { force: true });
    /// ```
    pub fn delete(&mut self) -> Result<(), MigrateError> {
        match std::fs::remove_file(&self.path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(MigrateError::io("delete the journal", self.path.as_path(), &error)),
        }
        self.states.clear();
        self.times.clear();
        self.persisted = false;
        return Ok(());
    }
}

/// Journal controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_journal_tests.rs"]
mod tests;
