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

/// Imports the control group records and queries for this file.
use crate::control_group::{Membership, Record, parse_record, query, reclaim};

/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot, read_locks};

/// Imports the journal file names and the durability barrier for this file.
use crate::journal::{GROUPS_FILE, INDEX_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, make_durable};

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
/// a recorded control group with a member (the kernel's own answer),
/// a recorded process group of the weaker path that still has a member,
/// or a record that is missing, cannot be read or cannot be understood.
/// Why:
/// The end of the wrapper's lock does not say whether its cargo ended.
/// A record is written before every child starts,
/// so an empty record, or a directory with no files at all (a begin cut off right after its creation),
/// shows that no child started;
/// a directory that holds other files but lost its record is uncertainty, never "no children"
/// (round 7, 6b2 2: neither the ready record nor the absence of one stands in for it).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function survivorNote(directory: string): string | undefined
/// ```
pub(crate) fn survivor_note(directory: &Path) -> Option<String> {
    let path: PathBuf = directory.join(GROUPS_FILE);
    let remedy: String = format!(
        "the journal was kept. When you are sure no cargo of that run is running, put the lockfiles in {} back by hand and remove that directory",
        directory.display()
    );
    let text: String = match fs::read_to_string(&path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return match fs::read_dir(directory) {
                Ok(mut entries) => match entries.next() {
                    None => None,
                    Some(Ok(_)) => Some(format!(
                        "{}: the record of the control groups of the killed run is missing, so a child of that run may still be writing; {remedy}",
                        directory.display()
                    )),
                    Some(Err(listing_error)) => Some(format!("{}: the journal could not be listed ({listing_error}), so a child of the killed run may still be writing; {remedy}", directory.display())),
                },
                Err(listing_error) => Some(format!("{}: the journal could not be listed ({listing_error}), so a child of the killed run may still be writing; {remedy}", directory.display())),
            };
        }
        Err(error) => {
            return Some(format!(
                "{}: the record of the control groups of the killed run could not be read ({error}), so a child of that run may still be writing; {remedy}",
                directory.display()
            ));
        }
    };
    // A "starting" line of the weaker path that no process group line follows: a child may run that no record names.
    let mut announced_without_group: bool = false;
    for line in text.lines() {
        match parse_record(line) {
            Err(reason) => {
                return Some(format!(
                    "{}: the record of the control groups of the killed run holds a line that is not understood ({reason}), so a child of that run may still be writing; {remedy}",
                    directory.display()
                ));
            }
            Ok(Record::Group { id, namespace, path: group_path }) => match query(id, namespace, &group_path) {
                Membership::Gone => {}
                Membership::Alive => {
                    return Some(format!(
                        "{}: a process in the control group {} of the killed run is still running; the journal was kept for a later run",
                        directory.display(),
                        group_path.display()
                    ));
                }
                Membership::Unknown(reason) => {
                    return Some(format!(
                        "{}: whether the control group of the killed run still has a member cannot be told ({reason}), so a child of that run may still be writing; {remedy}",
                        directory.display()
                    ));
                }
            },
            Ok(Record::ProcessGroupStarting) => announced_without_group = true,
            Ok(Record::ProcessGroup(leader)) => {
                announced_without_group = false;
                // The weaker path: only the recorded process group can be asked; a descendant that left it cannot be found.
                let alive: bool = match Pid::from_raw(leader).map(test_kill_process_group) {
                    Some(Ok(())) => true,
                    Some(Err(code)) => code != Errno::SRCH,
                    None => true,
                };
                if alive {
                    return Some(format!(
                        "{}: a process group of the killed run (pid {leader}) is still running; the journal was kept for a later run",
                        directory.display()
                    ));
                }
            }
        }
    }
    if announced_without_group {
        return Some(format!(
            "{}: the killed run announced a child (the weaker path, no control group) and never recorded its process group, so that child may still be writing; {remedy}",
            directory.display()
        ));
    }
    return None;
}

/// What:
/// `pub(crate) fn reclaim_recorded_groups(directory: &Path)` removes the control groups a journal recorded that are empty.
/// Why:
/// A run that was killed leaves its empty group directory behind;
/// the recovery that settles the journal tidies it, and an occupied group refuses the removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function reclaimRecordedGroups(directory: string): void
/// ```
pub(crate) fn reclaim_recorded_groups(directory: &Path) {
    let Ok(text) = fs::read_to_string(directory.join(GROUPS_FILE)) else {
        return;
    };
    for line in text.lines() {
        if let Ok(Record::Group { id, path, .. }) = parse_record(line) {
            reclaim(id, &path);
        }
    }
}
