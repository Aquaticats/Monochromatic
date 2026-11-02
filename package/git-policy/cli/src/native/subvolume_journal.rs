//! What:
//!  The journal of a subvolume migration: a small JSON file beside the checkout that records which steps finished,
//!  written atomically after each step.
//! Why:
//!  A rerun after a kill at any instant resumes from it: a step marked `done` is skipped, a step marked `started` is
//!  repeated (every step is idempotent), and a step with no entry has not begun. The file lives in the checkout's
//!  parent directory (`.cli-git-migrate-<name>.journal.json`), outside the tree that is copied and exchanged, so it
//!  survives the exchange. It is replaced by writing a sibling file and renaming it, so a kill never leaves half a
//!  journal. `serde_json` is already a dependency of the crate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const journal = await loadJournal(path); await journal.mark('copy', 'started'); ... await journal.mark('copy', 'done');
//! ```

use super::iso_time::{format_iso_milliseconds, now_iso};
use super::subvolume_error::MigrateError;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

/// The journal format version this code writes and reads.
const SCHEMA: u64 = 1;

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
    /// When the migration began, as `toISOString` text.
    started: String,
    /// Step name to state (`started`, `done` or `rolled-back`).
    states: BTreeMap<String, String>,
    /// Step name to the time in milliseconds after the epoch at which it got its current state.
    times: BTreeMap<String, i64>,
    /// Whether the file exists on disk.
    persisted: bool,
}

impl Journal {
    /// What:
    ///  Load the journal at `path`, or start an empty one when there is no file.
    /// Why:
    ///   A file that exists but is not a journal of this format is an error naming the file: guessing would resume
    ///  or skip steps on a false record.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const journal = await Journal.load(path);
    /// ```
    pub fn load(path: &Path) -> Result<Journal, MigrateError> {
        let text: String = match std::fs::read_to_string(path) {
            Ok(found) => found,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Journal { path: path.to_path_buf(), started: now_iso(), states: BTreeMap::new(), times: BTreeMap::new(), persisted: false });
            }
            Err(error) => return Err(MigrateError::io("read the journal", path, &error)),
        };
        let broken = |why: &str| { return MigrateError::Journal(format!("the journal {} {why}; read it, and delete it only if no migration is in progress", path.display())); };
        let value: Value = serde_json::from_str(text.as_str()).map_err(|error: serde_json::Error| { return broken(format!("is not valid JSON ({error})").as_str()); })?;
        if value.get("schema").and_then(Value::as_u64) != Some(SCHEMA) {
            return Err(broken("has an unknown schema version"));
        }
        let started: String = value.get("started").and_then(Value::as_str).ok_or_else(|| { return broken("has no start time"); })?.to_string();
        let steps: &Map<String, Value> = value.get("steps").and_then(Value::as_object).ok_or_else(|| { return broken("has no steps"); })?;
        let mut states: BTreeMap<String, String> = BTreeMap::new();
        let mut times: BTreeMap<String, i64> = BTreeMap::new();
        for (name, record) in steps {
            let state: &str = record.get("state").and_then(Value::as_str).ok_or_else(|| { return broken("has a step without a state"); })?;
            states.insert(name.clone(), state.to_string());
            if let Some(milliseconds) = record.get("epoch_ms").and_then(Value::as_i64) {
                times.insert(name.clone(), milliseconds);
            }
        }
        return Ok(Journal { path: path.to_path_buf(), started, states, times, persisted: true });
    }

    /// When the migration began, as `toISOString` text; unique to the migration and kept by a resumed run.
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

    /// The time, in whole seconds after the epoch, at which the step got its current state; nothing for a step with no
    ///  record or a record written without a time.
    pub fn seconds_of(&self, step: &str) -> Option<i64> {
        return self.times.get(step).map(|milliseconds: &i64| { return milliseconds.div_euclid(1000); });
    }

    /// The recorded state of a step, if any.
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
    ///   The file is written to a sibling and renamed over the journal, with the data synced first, so a kill leaves
    ///  either the old or the new journal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// await journal.mark('copy', 'done');
    /// ```
    pub fn mark(&mut self, step: &str, state: &str) -> Result<(), MigrateError> {
        self.states.insert(step.to_string(), state.to_string());
        let now: i64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed: std::time::Duration| { return i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX); });
        self.times.insert(step.to_string(), now);
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
        let text: String = serde_json::to_string_pretty(&Value::Object(document)).unwrap_or_default();
        let mut sibling: std::ffi::OsString = self.path.as_os_str().to_os_string();
        sibling.push(".tmp");
        let sibling_path: PathBuf = PathBuf::from(sibling);
        let write = || -> std::io::Result<()> {
            let mut file: std::fs::File = std::fs::File::create(&sibling_path)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
            return std::fs::rename(&sibling_path, &self.path);
        };
        write().map_err(|error: std::io::Error| { return MigrateError::io("write the journal", self.path.as_path(), &error); })?;
        self.persisted = true;
        return Ok(());
    }

    /// What:
    ///  Delete the journal file.
    /// Why:
    ///   A rolled-back or fully removed migration leaves no record, so the next migration starts clean.
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
