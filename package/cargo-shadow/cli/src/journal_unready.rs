//! What:
//! The recovery of journals that never became ready,
//! and the check for children of a killed run that may still be writing.
//! Why:
//! Round 6 (fifth review 5b2 1 to 4):
//! a survivor holds back every recovery and every new transaction,
//! an unreadable record is
//! uncertainty,
//! and a journal without a ready record is removed only when every payload is redundant and the live files are durable.
//! Split out of `journal_recover.rs`,
//! which holds the recovery of ready journals.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function survivorNote(directory: string): string | undefined
//! export function recoverUnready(root: string, directory: string, messages: string[]): void
//! ```

/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

/// Imports `rustix::io::Errno` for this file.
use rustix::io::Errno;

/// Imports `rustix::process::{Pid, test_kill_process_group}` for this file.
use rustix::process::{Pid, test_kill_process_group};

/// Imports `crate::constants::LOCKFILE_NAME` for this file.
use crate::constants::LOCKFILE_NAME;

/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot, read_locks};

/// Imports the journal file names and the durability barrier for this file.
use crate::journal::{GROUPS_FILE, INDEX_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, READY_FILE, make_durable};

/// Imports `crate::journal_manifests::read_manifest_index` for this file.
use crate::journal_manifests::read_manifest_index;

/// Imports `crate::journal_recover::read_journal` for this file.
use crate::journal_recover::read_journal;

/// Imports `crate::journal_transaction::mark_done_and_remove` for this file.
use crate::journal_transaction::mark_done_and_remove;

/// What:
/// `fn unready_verdict(root: &Path, directory: &Path) -> Result<Vec<String>, String>` decides whether a journal without a ready
/// record holds nothing that the live files do not already hold,
/// and returns the live files to make durable before it is removed;
/// `Err` says why it must be kept.
/// Why:
/// Round 6 (5b2 3):
/// equal lockfiles do not make a manifest snapshot or an unknown file redundant,
/// and a missing index is no proof that the directory is empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreadyVerdict(root: string, directory: string): string[]  // throws the reason to keep it
/// ```
fn unready_verdict(root: &Path, directory: &Path) -> Result<Vec<String>, String> {
    let names: Vec<String> = fs::read_dir(directory)
        .map_err(|error| { return format!("it could not be listed ({error})"); })?
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().to_string(); })
        .collect();
    let is_lock = |name: &str| -> bool { return name.strip_suffix(".lock").is_some_and(|stem| { return !stem.is_empty() && stem.bytes().all(|byte| { return byte.is_ascii_digit(); }); }) };
    let is_snapshot = |name: &str| -> bool { return name.strip_prefix('m').and_then(|rest| { return rest.strip_suffix(".toml"); }).is_some_and(|stem| { return !stem.is_empty() && stem.bytes().all(|byte| { return byte.is_ascii_digit(); }); }) };
    if let Some(unknown) = names.iter().find(|name| { return ![OWNER_FILE, GROUPS_FILE, INDEX_FILE, MANIFEST_INDEX_FILE].contains(&name.as_str()) && !is_lock(name) && !is_snapshot(name) && !name.ends_with(".tmp"); }) {
        return Err(format!("it holds {unknown}, which this version does not know"));
    }
    let locks: usize = names.iter().filter(|name| { return is_lock(name); }).count();
    let snapshots: usize = names.iter().filter(|name| { return is_snapshot(name); }).count();
    let mut live_files: Vec<String> = Vec::new();
    match read_journal(directory) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if locks > 0 {
                return Err("its index is missing but lockfile snapshots remain".to_string());
            }
        }
        Err(error) => return Err(format!("its index is damaged ({error})")),
        Ok(before) => {
            if locks != before.len() {
                return Err("it holds lockfile snapshots that its index does not list".to_string());
            }
            let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
            let live = read_locks(root, &dirs).map_err(|error| { return format!("a live lockfile could not be read ({error})"); })?;
            if !before.iter().zip(live.iter()).all(|(old, now)| { return old.source == now.source; }) {
                return Err("it holds the only copy of a lockfile that differs from the live one".to_string());
            }
            live_files.extend(dirs.iter().map(|crate_dir| { return root.join(crate_dir).join(LOCKFILE_NAME).to_string_lossy().to_string(); }));
        }
    }
    if names.iter().any(|name| { return name == MANIFEST_INDEX_FILE; }) {
        let manifests = read_manifest_index(directory).map_err(|error| { return format!("its manifest index is damaged ({error})"); })?;
        if snapshots != manifests.len() {
            return Err("it holds manifest snapshots that its manifest index does not list".to_string());
        }
        for manifest in &manifests {
            match fs::read_to_string(&manifest.path) {
                Ok(live) if live == manifest.before => live_files.push(manifest.path.to_string_lossy().to_string()),
                Ok(_) => return Err(format!("it holds the only copy of {}, which differs from the live file", manifest.path.display())),
                Err(error) => return Err(format!("{} could not be read ({error})", manifest.path.display())),
            }
        }
    } else if snapshots > 0 {
        return Err("manifest snapshots remain without a manifest index".to_string());
    }
    return Ok(live_files);
}

/// What:
/// `fn recover_unready(root: &Path, directory: &Path, messages: &mut Vec<String>)` removes a journal that never became ready
/// when every payload in it is redundant and the live files that make it redundant are durable,
/// and keeps it otherwise.
/// Why:
/// Deleting a durable copy while its live equal is still in the page cache could lose both in a power cut (round 6,
/// 5b2 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverUnready(root: string, directory: string, messages: string[]): void
/// ```
pub(crate) fn recover_unready(root: &Path, directory: &Path, messages: &mut Vec<String>) {
    let live_files: Vec<String> = match unready_verdict(root, directory) {
        Ok(found) => found,
        Err(reason) => {
            messages.push(format!("{}: a journal without a ready record was kept: {reason}", directory.display()));
            return;
        }
    };
    for live in &live_files {
        #[cfg(test)]
        let outcome: io::Result<()> = if crate::failpoints::fires(crate::failpoints::Point::RecoverDurability) {
            Err(io::Error::other("injected recovery durability failure"))
        } else {
            make_durable(Path::new(live))
        };
        #[cfg(not(test))]
        let outcome: io::Result<()> = make_durable(Path::new(live));
        if let Err(error) = outcome {
            messages.push(format!("{}: a journal without a ready record was kept because {live} could not be made durable ({error})", directory.display()));
            return;
        }
    }
    if let Err(error) = mark_done_and_remove(directory) {
        messages.push(format!("{}: a journal that never became ready could not be removed ({error}); it will be tried again", directory.display()));
    }
}

/// What:
/// `pub(crate) fn survivor_note(directory: &Path) -> Option<String>` says why a journal's recorded children may still be running:
/// a recorded process group that still has a member,
/// or a group record that cannot be read or understood.
/// Why:
/// The end of the wrapper's lock does not say whether its cargo ended.
/// A missing,
/// unreadable or malformed record is uncertainty,
/// never "no children" (round 6,
/// 5b2 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function survivorNote(directory: string): string | undefined
/// ```
pub(crate) fn survivor_note(directory: &Path) -> Option<String> {
    let path: PathBuf = directory.join(GROUPS_FILE);
    let text: String = match fs::read_to_string(&path) {
        Ok(found) => found,
        // Without a ready record the transaction never got as far as starting a child.
        Err(error) if error.kind() == io::ErrorKind::NotFound && !directory.join(READY_FILE).exists() => return None,
        Err(error) => {
            return Some(format!(
                "{}: the record of the process groups of the killed run could not be read ({error}), so a child of that run may still be writing; the journal was kept. If you are sure it is not running, remove {}",
                directory.display(),
                path.display()
            ));
        }
    };
    for line in text.lines() {
        let Some(group) = line.trim().parse::<i32>().ok().and_then(Pid::from_raw) else {
            return Some(format!(
                "{}: the record of the process groups of the killed run holds a line that is not a process group ({line:?}), so a child of that run may still be writing; the journal was kept. If you are sure it is not running, remove {}",
                directory.display(),
                path.display()
            ));
        };
        let alive: bool = match test_kill_process_group(group) {
            Ok(()) => true,
            Err(code) => code != Errno::SRCH,
        };
        if alive {
            return Some(format!(
                "{}: a process group of the killed run (pid {}) is still running; the journal was kept for a later run",
                directory.display(),
                group.as_raw_nonzero()
            ));
        }
    }
    return None;
}
