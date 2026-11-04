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
use crate::control_group::{Identity, LauncherState, Membership, Record, current_identity, launcher_state, parse_record, populated_at, query, read_identity_text, view_of};

/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot, read_locks};

/// Imports the journal file names and the durability barrier for this file.
use crate::journal::{ACKNOWLEDGED_FILE, ACKNOWLEDGED_HEADER, GROUPS_FILE, INDEX_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, VIEW_FILE, make_durable, sync_directory};

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

#[cfg(test)]
thread_local! {
    /// A hook that runs inside `assess` after every record was queried and before the unsettled scopes are judged,
    /// so a test can make a launcher create its scope in the gap between the two observations (round 10, 9a D1).
    static AFTER_QUERY: std::cell::RefCell<Option<std::rc::Rc<dyn Fn()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
/// `pub(crate) fn after_query_for_test(hook: Option<Rc<dyn Fn()>>)` sets the hook that runs between the queries of a check and its
/// judgement of unsettled scopes.
/// Why:
/// The schedule of 9a D1 puts a launcher's scope creation exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function afterQueryForTest(hook?: () => void): void
/// ```
#[cfg(test)]
pub(crate) fn after_query_for_test(hook: Option<std::rc::Rc<dyn Fn()>>) {
    AFTER_QUERY.with(|slot| { *slot.borrow_mut() = hook; });
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
    let file: PathBuf = directory.join(ACKNOWLEDGED_FILE);
    let Ok(text) = fs::read_to_string(&file) else {
        return false;
    };
    // The whole versioned record: the header, a process id, a time, three lines, each ended by a line feed.
    let lines: Vec<&str> = text.lines().collect();
    let complete: bool = text.ends_with('\n')
        && lines.len() == 3
        && lines[0] == ACKNOWLEDGED_HEADER
        && lines[1].parse::<u32>().is_ok()
        && lines[2].parse::<u64>().is_ok();
    if !complete {
        return false;
    }
    // A name that is visible is not yet an acknowledgement that survives a crash: the reader makes the file and its directory entry
    // durable before it relies on them, and an acknowledgement it cannot make durable does not count (round 9, 8a).
    // The file first, then the directory entry that names it; a failure of either one means the acknowledgement does not count.
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::AcknowledgementFileSync) {
        return false;
    }
    if !fs::File::open(&file).and_then(|opened| { return opened.sync_all(); }).is_ok() {
        return false;
    }
    #[cfg(test)]
    if crate::failpoints::fires(crate::failpoints::Point::AcknowledgementDirectorySync) {
        return false;
    }
    return sync_directory(directory).is_ok();
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
    let view_text: io::Result<String> = fs::read_to_string(directory.join(VIEW_FILE));
    // A reboot of the same machine ended every process of the run (assumption a: the run's writers are inside it, and a boot ends them
    // all). The journal was made by one process in one boot, so its identity covers every record. A different boot id with the same
    // machine id discharges only the question whether the run's processes live; every other check of a recovery still runs.
    let recorded_identity: Option<Identity> = view_text.as_ref().ok().and_then(|recorded| { return read_identity_text(recorded); });
    // Both identities must be well formed (the reader already refuses a malformed or repeated record): an id that is not an id differs
    // from every real one and is no evidence of a reboot (round 10, 9a D2).
    let rebooted: bool = match (&recorded_identity, current_identity()) {
        (Some(recorded), Ok(now)) => recorded.is_well_formed() && now.is_well_formed() && recorded.machine == now.machine && recorded.boot != now.boot,
        _ => false,
    };
    if rebooted {
        // The one contradiction that is kept: a recorded scope (its name carries a random token, so it cannot be a reused name)
        // that has a member after a reboot is not explained by the reboot.
        for line in text.lines() {
            if let Ok(Record::Scope { path: scope_path, .. }) = parse_record(line)
                && populated_at(&scope_path).unwrap_or(false)
            {
                return Some(Finding::Live(format!(
                    "{}: the scope {} of the killed run has a member although the boot id changed since the run; the journal was kept for a later run",
                    directory.display(),
                    scope_path.display()
                )));
            }
        }
        return None;
    }
    // The view of the journal directory: another mount view of the same path cannot be trusted to hold the same scopes.
    match &view_text {
        Ok(recorded) => {
            let expected: String = recorded.lines().next().unwrap_or("").strip_prefix("view ").unwrap_or("").trim().to_string();
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
    // The scopes of runs that are not settled and have no launcher record that shows the launcher gone: the launcher of such a run may
    // not have finished entering its scope.
    let mut unsettled: Vec<PathBuf> = Vec::new();
    // The whole record is read first: the launcher line of a scope comes after the scope line, and a scope is judged by it.
    // A launcher line that the record ends without a line feed may be cut short (a torn write), so it is not understood (11b 2.6).
    let line_total: usize = text.lines().count();
    let ends_whole: bool = text.ends_with('\n');
    let records: Vec<Result<Record, String>> = text
        .lines()
        .enumerate()
        .map(|(index, line)| {
            return if !ends_whole && index + 1 == line_total && line.starts_with("launcher ") {
                Err("the launcher line is not ended by a line feed, so it may be cut short".to_string())
            } else {
                parse_record(line)
            };
        })
        .collect();
    let settled_paths: Vec<&PathBuf> = records.iter().filter_map(|record| { return if let Ok(Record::Settled { path: settled_path }) = record { Some(settled_path) } else { None }; }).collect();
    let scope_paths: Vec<&PathBuf> = records.iter().filter_map(|record| { return if let Ok(Record::Scope { path: listed_path, .. }) = record { Some(listed_path) } else { None }; }).collect();
    for record in &records {
        if let Ok(Record::Launcher { path: noted_path, .. }) = record
            && !scope_paths.contains(&noted_path)
        {
            note_uncertain(format!(
                "{}: the record notes a launcher for the scope {}, which the journal does not list, so what it shows cannot be used; {hint}",
                directory.display(),
                noted_path.display()
            ));
        }
    }
    for record in records.iter().cloned() {
        match record {
            Err(reason) => note_uncertain(format!(
                "{}: the record of the containment of the killed run holds a line that is not understood ({reason}), so a child of that run may still be writing; {hint}",
                directory.display()
            )),
            Ok(Record::Settled { .. } | Record::Launcher { .. }) => {}
            Ok(Record::Scope { namespace, parent_view, path: scope_path }) => {
                // The launcher is asked about before the scope is read (round 11): a launcher that is gone can never join the scope, so
                // what the scope shows afterwards is final; one that is alive keeps blocking, and no acknowledgement overrides it.
                let settled: bool = settled_paths.contains(&&scope_path);
                let launcher: Option<(i32, u64, u64)> = records.iter().rev().find_map(|other| {
                    return if let Ok(Record::Launcher { pid, start, pid_namespace, path: noted_path }) = other
                        && *noted_path == scope_path
                    {
                        Some((*pid, *start, *pid_namespace))
                    } else {
                        None
                    };
                });
                if !settled {
                    match launcher.map(|(pid, start, pid_namespace)| { return (pid, launcher_state(pid, start, pid_namespace)); }) {
                        None => unsettled.push(scope_path.clone()),
                        Some((pid, LauncherState::Alive)) => {
                            return Some(Finding::Live(format!(
                                "{}: the launcher (pid {pid}, which becomes the program) of the scope {} of the killed run is still running; the journal was kept for a later run",
                                directory.display(),
                                scope_path.display()
                            )));
                        }
                        Some((pid, LauncherState::Unknown(reason))) => note_uncertain(format!(
                            "{}: whether the launcher (pid {pid}) of the scope {} is gone cannot be told ({reason}), so it may still join the scope; {hint}",
                            directory.display(),
                            scope_path.display()
                        )),
                        Some((_, LauncherState::Gone)) => {}
                    }
                }
                match query(namespace, parent_view, &scope_path) {
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
                }
            }
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
    #[cfg(test)]
    if let Some(hook) = AFTER_QUERY.with(|slot| { return slot.borrow().clone(); }) {
        hook();
    }
    for open in unsettled {
        // The run did not record its end and no record shows its launcher gone, so nothing shows that the launcher finished entering
        // the scope: an absent scope may not be made yet, and one that exists and is empty may be waiting for the launcher to be
        // attached. A look at the scope, however late, does not turn this into a completed lifecycle (round 10, 9a D1); a positively
        // populated scope has already blocked.
        note_uncertain(format!(
            "{}: the run that was to make the scope {} did not record its end, so its launcher may not have finished entering the scope \
             (whether the scope is absent or empty now) and a program may still start in it; {hint}",
            directory.display(),
            open.display()
        ));
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
