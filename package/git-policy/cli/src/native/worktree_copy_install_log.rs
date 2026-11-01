//! What:
//!  The append-only install log inside one transaction's private stage:
//!  one synced JSON line per batch of intents and one per batch of proven creations.
//! Why:
//!  The journal header changes only with the phase, so the per-batch records live here;
//!  bytes written then grow with the entry count instead of with the square of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await using opened = await openInstallLog(stageContainer);
//! await opened.log.appendIntents(paths);
//! ```

use super::json_record::quote;
use super::private_storage::sync_directory;
use super::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure};
use super::worktree_copy_install_entry::InstalledPath;
use super::worktree_copy_journal::{
    Role, assert_private_path, encode_installed_entry, installed_entry, string_array,
};
use super::worktree_copy_selection::assert_safe_repository_path;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// What:
///  The install-log file name inside a stage container.
/// Why:
///   Recovery finds the log by this name, so both wrappers must agree on it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const INSTALL_LOG_NAME = 'install-log.jsonl';
/// ```
pub const INSTALL_LOG_NAME: &str = "install-log.jsonl";

/// What:
///  The records an existing log holds.
/// Why:
///   An interrupted installation resumes from exactly these claims and creations.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallLogRecords = { intendedEntries: string[]; createdEntries: InstalledWorktreePath[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InstallLogRecords {
    /// Selected paths claimed before destination mutation, in claim order.
    pub intended: Vec<String>,
    /// Paths proven created, in creation order.
    pub created: Vec<InstalledPath>,
}

/// What:
///  The append handle of one log.
/// Why:
///   Each append is synced before the caller mutates the destination, so a crash leaves the claim on disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallLog = { appendIntents(paths): Promise<void>; appendCreations(entries): Promise<void> };
/// ```
#[derive(Debug)]
pub struct InstallLog {
    /// The log file, opened for appending without following links.
    file: std::fs::File,
    /// The log's path, named in every diagnostic.
    path: PathBuf,
}

/// What:
///  An open log together with what it already held.
/// Why:
///   The caller seeds its claim and creation sets from the recorded part before it installs anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { log: InstallLog; recorded: InstallLogRecords }
/// ```
#[derive(Debug)]
pub struct OpenedLog {
    /// The append handle.
    pub log: InstallLog,
    /// The records already in the log.
    pub recorded: InstallLogRecords,
}

/// What:
///  One line's records.
/// Why:
///   A line carries either intents or creations, never both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseLine({ line, path }): InstallLogRecords;
/// ```
fn parse_line(line: &str, path: &Path) -> Result<InstallLogRecords, WorktreeCopyError> {
    let corrupt = || {
        return failure_with_cause(
            format!(
                "cli-git: worktree-copy install log is corrupt: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        );
    };
    let value: serde_json::Value = serde_json::from_str(line).map_err(|_| return corrupt())?;
    let object = value.as_object().ok_or_else(corrupt)?;
    if let Some(intended) = object.get("intended").and_then(serde_json::Value::as_array) {
        let names: Vec<String> = intended
            .iter()
            .map(|item| return item.as_str().map(str::to_string))
            .collect::<Option<Vec<String>>>()
            .ok_or_else(corrupt)?;
        for name in &names {
            assert_safe_repository_path(name.as_str())?;
        }
        return Ok(InstallLogRecords {
            intended: names,
            created: Vec::new(),
        });
    }
    if let Some(created) = object.get("created").and_then(serde_json::Value::as_array) {
        let entries: Vec<InstalledPath> = created
            .iter()
            .map(installed_entry)
            .collect::<Option<Vec<InstalledPath>>>()
            .ok_or_else(corrupt)?;
        for entry in &entries {
            assert_safe_repository_path(entry.relative_path.as_str())?;
        }
        return Ok(InstallLogRecords {
            intended: Vec::new(),
            created: entries,
        });
    }
    return Err(corrupt());
}

/// What:
///  The records of every complete line, and the byte length those lines span.
/// Why:
///   A trailing fragment without its line feed is an append the owner never finished; it is not a record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readCompleteRecords(path): Promise<{ records; completeLength; totalLength }>;
/// ```
fn read_complete_records(path: &Path) -> Result<(InstallLogRecords, u64, u64), WorktreeCopyError> {
    assert_private_path(path, Role::JournalFile)?;
    let content: Vec<u8> = std::fs::read(path).map_err(|error| return raw_failure(&error))?;
    let complete_length: usize = content
        .iter()
        .rposition(|byte| return *byte == b'\n')
        .map_or(0, |index| return index + 1);
    let text: String = String::from_utf8_lossy(&content[..complete_length]).into_owned();
    let mut records: InstallLogRecords = InstallLogRecords::default();
    for line in text.split('\n').filter(|line| return !line.is_empty()) {
        let parsed: InstallLogRecords = parse_line(line, path)?;
        records.intended.extend(parsed.intended);
        records.created.extend(parsed.created);
    }
    return Ok((records, complete_length as u64, content.len() as u64));
}

/// What:
///  Opens the log of one stage, returning what it already records and truncating any unfinished append.
/// Why:
///   Resumption must see every claim and creation, and new records must start on a fresh line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await using opened = await openInstallLog(stageContainer);
/// ```
pub fn open_install_log(stage_container: &Path) -> Result<OpenedLog, WorktreeCopyError> {
    let path: PathBuf = stage_container.join(INSTALL_LOG_NAME);
    let existed: bool = std::fs::symlink_metadata(path.as_path()).is_ok();
    let (recorded, complete_length, total_length) = if existed {
        read_complete_records(path.as_path())?
    } else {
        (InstallLogRecords::default(), 0, 0)
    };
    let opened: std::io::Result<std::fs::File> = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .custom_flags(libc::O_NOFOLLOW)
        .mode(0o600)
        .open(path.as_path());
    let file: std::fs::File = match opened {
        Ok(file) => file,
        Err(error) => {
            let message: String = format!(
                "cli-git: could not open worktree-copy install log {}.",
                quote(path.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    if complete_length < total_length {
        file.set_len(complete_length)
            .map_err(|error| return raw_failure(&error))?;
        file.sync_all()
            .map_err(|error| return raw_failure(&error))?;
    }
    if !existed {
        sync_directory(stage_container).map_err(|error| return raw_failure(&error))?;
    }
    return Ok(OpenedLog {
        log: InstallLog { file, path },
        recorded,
    });
}

impl InstallLog {
    /// Appends one line and waits until it is durable; an empty batch writes nothing.
    fn append_line(&mut self, line: &str) -> Result<(), WorktreeCopyError> {
        let written: std::io::Result<()> = self
            .file
            .write_all(line.as_bytes())
            .and_then(|()| return self.file.sync_all());
        return written.map_err(|error| {
            let message: String = format!(
                "cli-git: could not persist worktree-copy install log {}.",
                quote(self.path.to_string_lossy().as_ref())
            );
            return failure_with_cause(message.as_str(), Some(&error));
        });
    }

    /// Durably appends the selected paths one batch is about to create.
    pub fn append_intents(&mut self, relative_paths: &[String]) -> Result<(), WorktreeCopyError> {
        if relative_paths.is_empty() {
            return Ok(());
        }
        let line: String = format!("{{\"intended\":{}}}\n", string_array(relative_paths));
        return self.append_line(line.as_str());
    }

    /// Durably appends the identities of the paths one batch created.
    pub fn append_creations(&mut self, entries: &[InstalledPath]) -> Result<(), WorktreeCopyError> {
        if entries.is_empty() {
            return Ok(());
        }
        let encoded: Vec<String> = entries.iter().map(encode_installed_entry).collect();
        let line: String = format!("{{\"created\":[{}]}}\n", encoded.join(","));
        return self.append_line(line.as_str());
    }
}

/// Install-log controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_install_log_tests.rs"]
mod tests;
