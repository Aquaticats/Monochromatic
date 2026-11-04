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
use crate::control_group::{Membership, Record, parse_record, query, view_of};

/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot, read_locks};

/// Imports the journal file names and the durability barrier for this file.
use crate::journal::{ACKNOWLEDGED_FILE, ACKNOWLEDGED_HEADER, GROUPS_FILE, INDEX_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, VIEW_FILE, make_durable};

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
    if let Some(unknown) = names.iter().find(|name| { return ![OWNER_FILE, GROUPS_FILE, VIEW_FILE, ACKNOWLEDGED_FILE, INDEX_FILE, MANIFEST_INDEX_FILE].contains(&name.as_str()) && !is_lock(name) && !is_snapshot(name) && !name.ends_with(".tmp"); }) {
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

/// What a check of one journal found.
enum Finding {
    /// A recorded scope or process group is positively observed to have a member:
    /// nothing overrides this.
    Live(
        /// The message.
        String,
    ),
    /// The containment context cannot establish that the run's writers stopped:
    /// an acknowledgement overrides this.
    Uncertain(
        /// The message.
        String,
    ),
}

/// What:
/// `fn is_acknowledged(directory: &Path) -> bool` says whether a person acknowledged this journal.
/// Why:
/// An unreadable or partial file does not count:
/// an acknowledgement interrupted while it was written leaves the journal unacknowledged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isAcknowledged(directory: string): boolean
/// ```
pub(crate) fn is_acknowledged(directory: &Path) -> bool {
    return fs::read_to_string(directory.join(ACKNOWLEDGED_FILE)).is_ok_and(|text| { return text.lines().next() == Some(ACKNOWLEDGED_HEADER) && text.ends_with('\n') && text.lines().count() == 3; });
}

/// What:
/// `fn acknowledge_hint(directory: &Path) -> String` is the end of every message that an acknowledgement can resolve.
/// Why:
/// The message names the journal and the command that acknowledges it,
/// and what the person asserts by running it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function acknowledgeHint(directory: string): string
/// ```
fn acknowledge_hint(directory: &Path) -> String {
    return format!(
        "the journal {} was kept. When every process or worker of that run that can modify the workspace has stopped, run: cargo-shadow acknowledge {}",
        directory.display(),
        directory.display()
    );
}

/// What:
/// `fn assess(directory: &Path) -> Option<Finding>` reads the records of one journal and says what they show about writers.
/// Why:
/// A record is written before every child starts,
/// so an empty record, or a directory with no files at all (a begin cut off right after its creation),
/// shows that no child started;
/// a directory that holds other files but lost its record is uncertainty, never "no children"
/// (round 7, 6b2 2: neither the ready record nor the absence of one stands in for it).
/// A journal that holds records is also checked for the mount and inode view it was made in (contract assumption c).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function assess(directory: string): Finding | undefined
/// ```
fn assess(directory: &Path) -> Option<Finding> {
    let path: PathBuf = directory.join(GROUPS_FILE);
    let hint: String = acknowledge_hint(directory);
    let text: String = match fs::read_to_string(&path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return match fs::read_dir(directory) {
                Ok(mut entries) => match entries.next() {
                    None => None,
                    Some(Ok(_)) => Some(Finding::Uncertain(format!(
                        "{}: the record of the containment of the killed run is missing, so a child of that run may still be writing; {hint}",
                        directory.display()
                    ))),
                    Some(Err(listing_error)) => Some(Finding::Uncertain(format!("{}: the journal could not be listed ({listing_error}), so a child of the killed run may still be writing; {hint}", directory.display()))),
                },
                Err(listing_error) => Some(Finding::Uncertain(format!("{}: the journal could not be listed ({listing_error}), so a child of the killed run may still be writing; {hint}", directory.display()))),
            };
        }
        Err(error) => {
            return Some(Finding::Uncertain(format!(
                "{}: the record of the containment of the killed run could not be read ({error}), so a child of that run may still be writing; {hint}",
                directory.display()
            )));
        }
    };
    // An empty record shows that no child ever started.
    let _first_line: &str = text.lines().next()?;
    let mut uncertain: Option<String> = None;
    let mut note_uncertain = |message: String| {
        if uncertain.is_none() {
            uncertain = Some(message);
        }
    };
    // The view of the journal directory: another mount view of the same path cannot be trusted to hold the same scopes.
    match fs::read_to_string(directory.join(VIEW_FILE)) {
        Ok(recorded) => {
            let expected: String = recorded.trim().strip_prefix("view ").unwrap_or("").to_string();
            match view_of(directory) {
                Ok(seen) if seen.to_string() == expected => {}
                Ok(seen) => note_uncertain(format!(
                    "{}: the journal is seen through another mount or path (mount and inode {seen} here, {expected:?} when it was made), so the containment it recorded may not be the one visible here; {hint}",
                    directory.display()
                )),
                Err(error) => note_uncertain(format!("{}: the view of the journal cannot be examined ({error}); {hint}", directory.display())),
            }
        }
        Err(error) => note_uncertain(format!("{}: the record of the journal's mount view could not be read ({error}); {hint}", directory.display())),
    }
    // A "starting" line of the weaker path that no process group line follows: a child may run that no record names.
    let mut announced_without_group: bool = false;
    let mut weaker_path: bool = false;
    for line in text.lines() {
        match parse_record(line) {
            Err(reason) => note_uncertain(format!(
                "{}: the record of the containment of the killed run holds a line that is not understood ({reason}), so a child of that run may still be writing; {hint}",
                directory.display()
            )),
            Ok(Record::Scope { namespace, parent_view, path: scope_path }) => match query(namespace, parent_view, &scope_path) {
                Membership::Gone => {}
                Membership::Alive => {
                    return Some(Finding::Live(format!(
                        "{}: a process in the scope {} of the killed run is still running; the journal was kept for a later run",
                        directory.display(),
                        scope_path.display()
                    )));
                }
                Membership::Unknown(reason) => note_uncertain(format!(
                    "{}: whether the scope of the killed run still has a member cannot be told ({reason}), so a child of that run may still be writing; {hint}",
                    directory.display()
                )),
            },
            Ok(Record::ProcessGroupStarting) => {
                weaker_path = true;
                announced_without_group = true;
            }
            Ok(Record::ProcessGroup(leader)) => {
                weaker_path = true;
                announced_without_group = false;
                let alive: bool = match Pid::from_raw(leader).map(test_kill_process_group) {
                    Some(Ok(())) => true,
                    Some(Err(code)) => code != Errno::SRCH,
                    None => true,
                };
                if alive {
                    return Some(Finding::Live(format!(
                        "{}: a process group of the killed run (pid {leader}) is still running; the journal was kept for a later run",
                        directory.display()
                    )));
                }
            }
        }
    }
    if announced_without_group {
        note_uncertain(format!(
            "{}: the killed run announced a child (the weaker path, no delegated control group) and never recorded its process group, so that child may still be writing; {hint}",
            directory.display()
        ));
    }
    if weaker_path {
        note_uncertain(format!(
            "{}: the killed run had no delegated control group, so a process that left its process group cannot be found and the end of that group does not show that the run's writers stopped; {hint}",
            directory.display()
        ));
    }
    return uncertain.map(Finding::Uncertain);
}

/// What:
/// `pub(crate) fn survivor_note(directory: &Path) -> Option<String>` says why a journal's recorded children may still be running,
/// or `None` when the records show they stopped.
/// Why:
/// The end of the wrapper's lock does not say whether its cargo ended.
/// A live recorded scope or process group always blocks.
/// Any other uncertainty about the containment (a weaker-path run, a missing, unreadable or damaged record, a recorded scope that
/// cannot be examined, another mount view) blocks until a person acknowledges the journal
/// (`cargo-shadow acknowledge`), who thereby asserts that every process or worker of the run has stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function survivorNote(directory: string): string | undefined
/// ```
pub(crate) fn survivor_note(directory: &Path) -> Option<String> {
    return match assess(directory) {
        None => None,
        Some(Finding::Live(message)) => Some(message),
        Some(Finding::Uncertain(message)) => {
            if is_acknowledged(directory) {
                None
            } else {
                Some(message)
            }
        }
    };
}
