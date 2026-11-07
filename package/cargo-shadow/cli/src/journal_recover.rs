//! What:
//! The recovery of a run that was killed:
//! take the lock of its journal,
//! decide per root whether the leftover lockfile is kept or the
//! snapshot put back,
//! and put back the manifests of a killed `--breaking` run.
//! Why:
//! `journal.rs` holds the recording side;
//! the recovery side was split out to keep each file under the line limit.
//! A journal is recovered only when its owner's lock can be taken (the owner is gone) and only after the same lock file is
//! confirmed to be the one at the journal's path.
//! A root that a live transaction covers,
//! a root whose lockfile could not be settled and a root with no decision are never
//! restored,
//! so a lockfile and its manifests are never split.
//! Nothing here waits:
//! a held lock is an answer,
//! and the journal stays for a later run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-recover.ts
//! export function recover(root: string, judge: Judge): string[]
//! ```

// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `std::fs::{self, File}` for this file.
use std::fs;
/// Imports `std::io` for this file.
use std::io;
// Imports `std::os::unix::fs::MetadataExt`.
/// Imports `std::os::unix::fs::MetadataExt` for this file.
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

// Imports `crate::constants::LOCKFILE_NAME`.
/// Imports `crate::constants::LOCKFILE_NAME` for this file.
use crate::constants::LOCKFILE_NAME;
/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot, read_locks};
// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, J...`.
/// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, KeepAs, LOCK_FILE, MANIFEST_INDEX_FILE, READY_FILE, REFUSED_SUFFIX, ReplaceOutcome, decode_path, file_name_text, no_hook, replace_keeping_with_hook, sync_directory}` for this file.
use crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, KeepAs, MANIFEST_INDEX_FILE, READY_FILE, REFUSED_SUFFIX, ReplaceOutcome, file_name_text, make_durable, no_hook, replace_keeping_with_hook};
// Imports `crate::journal_transaction::{mark_done_and_...`.
/// Imports `crate::journal_transaction::{finish_journal, take_workspace}` for this file.
use crate::journal_transaction::{finish_journal, take_workspace};
/// Imports the process-group probe and the error code that says a group is empty.
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Re-exports the unready-journal and survivor checks,
/// which live in their own file.
pub(crate) use crate::journal_unready::{recover_unready, survivor_note};

/// Imports the manifest side of a recovery.
use crate::journal_manifests::{JournaledManifest, RootDecision, read_manifest_index, recover_manifests};

/// What:
/// `pub(crate) fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>>` reads the lockfile index and the snapshots.
/// Why:
/// The index is versioned and ends with a count,
/// so a truncated or empty index is an error and never a shorter list of roots.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readJournal(directory: string): LockSnapshot[]  // throws on a damaged journal
/// ```
pub(crate) fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>> {
    // Macro `format!`: builds or prints text (or a list) from a template.
    let corrupt = |why: &str| -> io::Error { return io::Error::new(io::ErrorKind::InvalidData, format!("the lockfile index is corrupt: {why}")); };
    let index: String = fs::read_to_string(directory.join(INDEX_FILE))?;
    // `let mut`: a binding that may change.
    let mut lines = index.lines();
    // `Some`: the present or the absent value of an `Option`.
    if lines.next() != Some("v2") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no v2 header"));
    }
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    // `None`: the present or the absent value of an `Option`.
    // `let mut`: a binding that may change.
    // Typed binding: `ended: Option<usize>`.
    let mut ended: Option<usize> = None;
    for line in lines {
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(count) = line.strip_prefix("end ") {
            // `Some`: the present or the absent value of an `Option`.
            // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
            // Closure `|_|`: an anonymous function, like a TS arrow function.
            // `.map_err(...)`: converts the callee's error into this function's error type.
            ended = Some(count.trim().parse().map_err(|_| { return corrupt("the end count is not a number"); })?);
            continue;
        }
        if ended.is_some() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(corrupt("text after the end line"));
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `source: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let source: String = fs::read_to_string(directory.join(format!("{}.lock", snapshots.len())))?;
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        snapshots.push(LockSnapshot { crate_dir: line.to_string(), source });
    }
    // `Some`: the present or the absent value of an `Option`.
    if ended != Some(snapshots.len()) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("the end count does not match the lines"));
    }
    return Ok(snapshots);
}

/// What the ready record promises.
pub(crate) struct Ready {
    /// Whether the journal carries manifest snapshots.
    pub(crate) manifests: bool,
}

/// What:
/// `pub(crate) fn read_ready(directory: &Path) -> io::Result<Option<Ready>>` reads the ready record and checks that every file it
/// lists is present with exactly the listed length;
/// `Ok(None)` means there is no ready record (the journal was never completed).
/// Why:
/// Review finding (3b 7):
/// a missing or short file of a ready journal is damage,
/// and damage keeps the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReady(directory: string): Ready | undefined  // throws on a damaged journal
/// ```
pub(crate) fn read_ready(directory: &Path) -> io::Result<Option<Ready>> {
    // Macro `format!`: builds or prints text (or a list) from a template.
    let corrupt = |why: &str| -> io::Error { return io::Error::new(io::ErrorKind::InvalidData, format!("the ready record is corrupt: {why}")); };
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `text: String`.
    let text: String = match fs::read_to_string(directory.join(READY_FILE)) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => return Err(error),
    };
    // `let mut`: a binding that may change.
    let mut lines = text.lines();
    // `Some`: the present or the absent value of an `Option`.
    if lines.next() != Some("v1") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no v1 header"));
    }
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `manifests: bool`.
    let manifests: bool = match lines.next() {
        // `Some`: the present or the absent value of an `Option`.
        Some("manifests 1") => true,
        // `Some`: the present or the absent value of an `Option`.
        Some("manifests 0") => false,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        _ => return Err(corrupt("no manifests line")),
    };
    // `let mut`: a binding that may change.
    // Typed binding: `listed: usize`.
    let mut listed: Vec<String> = Vec::new();
    let mut ended: Option<usize> = None;
    for line in lines {
        if ended.is_some() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(corrupt("text after the end line"));
        }
        if let Some(count) = line.strip_prefix("end ") {
            ended = Some(count.trim().parse().map_err(|_| { return corrupt("the end count is not a number"); })?);
            continue;
        }
        let (name, length) = line.split_once('\t').ok_or_else(|| { return corrupt("a line has no tab"); })?;
        let expected: u64 = length.parse().map_err(|_| { return corrupt("a length is not a number"); })?;
        let found: u64 = fs::metadata(directory.join(name))?.len();
        if found != expected {
            return Err(corrupt(&format!("{name} has {found} bytes, the record says {expected}")));
        }
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        listed.push(name.to_string());
    }
    // `Some`: the present or the absent value of an `Option`.
    if ended != Some(listed.len()) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no end line, or its count does not match the lines"));
    }
    // The listed set must be exactly the payload a recovery needs: the index, one snapshot per root, and the manifest index when promised.
    let roots: usize = fs::read_to_string(directory.join(INDEX_FILE))?.lines().filter(|line| { return !line.starts_with("end ") && *line != "v2"; }).count();
    let mut required: Vec<String> = vec![INDEX_FILE.to_string()];
    required.extend((0..roots).map(|position| { return format!("{position}.lock"); }));
    if manifests {
        required.push(MANIFEST_INDEX_FILE.to_string());
        let snapshots: usize = fs::read_to_string(directory.join(MANIFEST_INDEX_FILE))?.lines().filter(|line| { return !line.starts_with("end ") && *line != "v1"; }).count();
        required.extend((0..snapshots).map(|position| { return format!("m{position}.toml"); }));
    }
    // The listed files are exactly the payload the indexes imply: no omission, no duplicate, no extra.
    let mut sorted_listed: Vec<&String> = listed.iter().collect();
    sorted_listed.sort();
    let mut sorted_required: Vec<&String> = required.iter().collect();
    sorted_required.sort();
    if sorted_listed != sorted_required {
        return Err(corrupt(&format!("the listed files {listed:?} differ from the payload the indexes imply {required:?}")));
    }
    return Ok(Some(Ready { manifests }));
}

/// The journals of a workspace.
pub(crate) struct JournalListing {
    /// The journals that have not been marked done,
    /// in name order.
    pub(crate) open: Vec<PathBuf>,
    /// The journals marked done,
    /// whose removal is cleanup.
    pub(crate) done: Vec<PathBuf>,
}

/// What:
/// `pub(crate) fn list_journals(journals: &Path) -> io::Result<Option<JournalListing>>` lists the journal directories,
/// and fails when any entry cannot be listed or examined.
/// Why:
/// A survivor check and a recovery both decide from the whole set;
/// a set that could not be read completely cannot show that no earlier run left a writer
/// (round 7,
/// 6b2 1:
/// `flatten()` and `is_dir()` dropped such entries).
/// Only the opening of the journal directory finding no directory is "no journals" (`None`);
/// every failure after it,
/// `NotFound` of one entry included,
/// is an error that discards the partial listing (round 8,
/// 7b P1-2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function listJournals(journals: string): { open: string[]; done: string[] } | undefined  // throws when any entry fails
/// ```
pub(crate) fn list_journals(journals: &Path) -> io::Result<Option<JournalListing>> {
    #[cfg(test)]
    let open_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingOpen);
    #[cfg(test)]
    let iteration_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingIteration);
    #[cfg(test)]
    let inspection_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingInspection);
    #[cfg(test)]
    let vanished_fault: bool = crate::failpoints::fires(crate::failpoints::Point::ListingEntryNotFound);
    #[cfg(test)]
    if open_fault {
        return Err(io::Error::other("injected failure to open the journal directory"));
    }
    let entries: fs::ReadDir = match fs::read_dir(journals) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut listing: JournalListing = JournalListing { open: Vec::new(), done: Vec::new() };
    #[cfg_attr(not(test), allow(unused_variables))]
    for (index, found) in entries.enumerate() {
        // The second entry is the one a test makes fail, so the first is already in the partial listing.
        #[cfg(test)]
        if index == 1 && iteration_fault {
            crate::bounded_run::trace(|| { return format!("listing fault iteration at entry {index} after {} accumulated", listing.open.len() + listing.done.len()); });
            return Err(io::Error::other("injected failure to read an entry of the journal directory"));
        }
        let entry: fs::DirEntry = found?;
        let path: PathBuf = entry.path();
        #[cfg(test)]
        if index == 1 && inspection_fault {
            crate::bounded_run::trace(|| { return format!("listing fault inspection at entry {index} after {} accumulated", listing.open.len() + listing.done.len()); });
            return Err(io::Error::other("injected failure to examine an entry of the journal directory"));
        }
        #[cfg(test)]
        if index == 1 && vanished_fault {
            crate::bounded_run::trace(|| { return format!("listing fault entry-not-found at entry {index} after {} accumulated", listing.open.len() + listing.done.len()); });
            return Err(io::Error::from(io::ErrorKind::NotFound));
        }
        if path.to_string_lossy().ends_with(DONE_SUFFIX) {
            listing.done.push(path);
        } else if entry.file_type()?.is_dir() {
            listing.open.push(path);
        }
    }
    listing.open.sort();
    listing.done.sort();
    return Ok(Some(listing));
}

/// What:
/// `pub fn recover(root, judge) -> Vec<String>` finishes the journals of runs that no longer exist.
/// Why:
/// It is `recover_with` with no test hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recover(root: string, judge: Judge): string[]
/// ```
pub fn recover(root: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>) -> Vec<String> {
    // Borrow `&no_hook`: lends the value without copying or giving it away.
    return recover_with(root, judge, &no_hook);
}

/// What a recovery says and whether the command that ran it may go on.
#[derive(Debug)]
pub struct RecoveryReport {
    /// What to tell the person,
    /// one warning per entry.
    pub messages: Vec<String>,
    /// Whether a mandatory phase of finishing a recovered journal failed (the output barrier before the marker,
    /// or the durable
    /// completion marker itself):
    /// the journal is not finished,
    /// so the command must stop.
    /// A failed removal of a journal that is
    /// already marked done is not one;
    /// it only warns.
    pub mandatory_failure: bool,
}

/// What:
/// `pub fn recover_report(root, judge) -> RecoveryReport` is `recover` that also says whether a mandatory phase of finishing failed.
/// Why:
/// The callers of a recovery stop on a failed mandatory phase and go on after a cleanup warning (certification,
/// second pass,
/// V5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverReport(root: string, judge: Judge): { messages: string[]; mandatoryFailure: boolean }
/// ```
pub fn recover_report(root: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>) -> RecoveryReport {
    return recover_report_with(root, judge, &no_hook);
}

/// What:
/// `pub fn recover_with(root, judge, between) -> Vec<String>` is `recover` with a hook that runs before each replacement.
/// Why:
/// Tests stop the recovery at the point where another process could interfere.
/// The workspace lock is taken first and the journals are listed only afterwards,
/// so a journal published while this run waited for the lock is seen,
/// and a surviving child of any journal holds back every recovery of the workspace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverWith(root: string, judge: Judge, between: (path: string) => void): string[]
/// ```
pub fn recover_with(root: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path)) -> Vec<String> {
    return recover_report_with(root, judge, between).messages;
}

/// What:
/// `pub fn recover_report_with(root, judge, between) -> RecoveryReport` is `recover_with` that also reports a mandatory failure.
/// Why:
/// It is the one implementation of the recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverReportWith(root: string, judge: Judge, between: (path: string) => void): RecoveryReport
/// ```
pub fn recover_report_with(root: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path)) -> RecoveryReport {
    let mut mandatory: bool = false;
    let messages: Vec<String> = recover_messages(root, judge, between, &mut mandatory);
    return RecoveryReport { messages, mandatory_failure: mandatory };
}

/// What:
/// `fn recover_messages(root, judge, between, mandatory) -> Vec<String>` runs the recovery and sets `mandatory` when a mandatory
/// phase of finishing a journal failed.
/// Why:
/// The early returns of the recovery keep their form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverMessages(root: string, judge: Judge, between: (path: string) => void, mandatory: { value: boolean }): string[]
/// ```
fn recover_messages(root: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), mandatory: &mut bool) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let journals: PathBuf = root.join(JOURNAL_DIR);
    // A metadata failure is not "no journals" (round 7, 6b2 1).
    match fs::metadata(&journals) {
        Ok(found) if found.is_dir() => {}
        Ok(_) => {
            messages.push(format!("{} is not a directory; the journals of earlier runs were left for a later run", journals.display()));
            return messages;
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => return messages,
        Err(error) => {
            messages.push(format!("{} could not be examined ({error}); the journals of earlier runs were left for a later run", journals.display()));
            return messages;
        }
    }
    let _workspace = match take_workspace(root) {
        Ok(Ok(held)) => held,
        Ok(Err(holder)) => {
            messages.push(format!("{holder}; the journals of earlier runs were left for a later run"));
            return messages;
        }
        Err(error) => {
            messages.push(format!("the workspace lock could not be tried ({error}); the journals of earlier runs were left for a later run"));
            return messages;
        }
    };
    // An entry that cannot be listed or examined stops the recovery: the set of journals, and so of surviving children, is unknown
    // (round 7, 6b2 1).
    let listing: JournalListing = match list_journals(&journals) {
        Ok(Some(found)) => found,
        Ok(None) => return messages,
        Err(error) => {
            messages.push(format!("{} could not be listed ({error}); the journals of earlier runs were left for a later run", journals.display()));
            return messages;
        }
    };
    for directory in listing.done {
        // A journal already marked done still has its completion barrier to make: the earlier rename may not be durable, so the parent
        // is synchronized again here, and a failure of that is a mandatory failure, not a cleanup (certification, third pass).
        if let Err(error) = finish_journal(&directory) {
            messages.push(format!("{}: a finished journal could not be completed or removed ({error}); it will be tried again", directory.display()));
            if !error.cleanup_only() {
                *mandatory = true;
            }
        }
    }
    let open: Vec<PathBuf> = listing.open;
    let survivors: Vec<String> = open.iter().filter_map(|directory| { return survivor_note(directory); }).collect();
    if !survivors.is_empty() {
        messages.extend(survivors);
        messages.push("no journal of this workspace was recovered while a child of an earlier run may still be writing".to_string());
        return messages;
    }
    for directory in open {
        recover_one(root, &directory, judge, between, &mut messages, mandatory);
    }
    return messages;
}

/// What:
/// `fn report_leftover_temporaries(root, crate_dirs, messages)` names the replacement temporaries a killed run left in the crate
/// roots.
/// Why:
/// Review finding (4b 3):
/// a run killed between the exchange and the kept link leaves the displaced file only under a hidden
/// `.shadow-*.tmp` name;
/// recovery reports it and never deletes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function reportLeftoverTemporaries(root: string, crateDirs: string[], messages: string[]): void
/// ```
fn report_leftover_temporaries(root: &Path, crate_dirs: &[String], messages: &mut Vec<String>) {
    for crate_dir in crate_dirs {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        let Ok(listing) = fs::read_dir(root.join(crate_dir)) else {
            continue;
        };
        for entry in listing.flatten() {
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // Typed binding: `name: String`.
            let name: String = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') && name.contains(".shadow-") && name.ends_with(".tmp") {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{} may hold a file that a killed run displaced and did not get to keep under another name; it was left as it is", entry.path().display()));
            }
        }
    }
}

/// What:
/// `fn recover_one(root, directory, judge, between, messages)` recovers one journal while this process holds the workspace.
/// Why:
/// An unready journal is settled by `recover_unready`;
/// a damaged or unsettled one is kept;
/// a settled one is made durable,
/// completed and removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverOne(root: string, directory: string, judge: Judge, messages: string[]): void
/// ```
fn recover_one(root: &Path, directory: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), messages: &mut Vec<String>, mandatory: &mut bool) {
    let ready: Ready = match read_ready(directory) {
        Ok(Some(found)) => found,
        Ok(None) => {
            recover_unready(root, directory, messages, mandatory);
            return;
        }
        Err(error) => {
            messages.push(format!("{}: the journal of a killed run is damaged ({error}); it was kept", directory.display()));
            return;
        }
    };
    let before: Vec<LockSnapshot> = match read_journal(directory) {
        Ok(found) => found,
        Err(error) => {
            messages.push(format!("{}: the journal of a killed run could not be read ({error}); it was kept", directory.display()));
            return;
        }
    };
    let manifests: Vec<JournaledManifest> = if ready.manifests {
        match read_manifest_index(directory) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: the manifest record of a killed run could not be read ({error}); its journal was kept", directory.display()));
                return;
            }
        }
    } else {
        Vec::new()
    };
    let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
    let live: Vec<LockSnapshot> = match read_locks(root, &dirs) {
        Ok(found) => found,
        Err(error) => {
            messages.push(format!("{}: a lockfile of a killed run could not be read ({error}); its journal was kept", directory.display()));
            return;
        }
    };
    // Borrow `&dirs`: lends the value without copying or giving it away.
    report_leftover_temporaries(root, &dirs, messages);
    let mut settled: bool = true;
    let mut decisions: Vec<(String, RootDecision)> = Vec::new();
    for (old, now) in before.iter().zip(live.iter()) {
        // Typed binding: `decision: RootDecision`.
        let decision: RootDecision = recover_root(root, old, now, judge, between, messages);
        if decision == RootDecision::Failed {
            settled = false;
        }
        decisions.push((old.crate_dir.clone(), decision));
    }
    if !recover_manifests(&manifests, &decisions, between, messages) {
        settled = false;
    }
    if settled {
        // Borrow `&decisions`: lends the value without copying or giving it away.
        // The output barrier of the finish: a test makes it fail with the recovery's own durability fault.
        #[cfg(test)]
        if crate::failpoints::fires(crate::failpoints::Point::RecoverDurability) {
            messages.push(format!("{}: the restored files could not be synced (injected durability failure); the journal was kept", directory.display()));
            *mandatory = true;
            return;
        }
        for (crate_dir, _decision) in &decisions {
            if let Err(error) = make_durable(&root.join(crate_dir).join(LOCKFILE_NAME)) {
                messages.push(format!("{crate_dir}/{LOCKFILE_NAME} could not be synced ({error}); the journal was kept"));
                *mandatory = true;
                return;
            }
        }
        for manifest in &manifests {
            if let Err(error) = make_durable(&manifest.path) {
                messages.push(format!("{} could not be synced ({error}); the journal was kept", manifest.path.display()));
                *mandatory = true;
                return;
            }
        }
        if let Err(error) = finish_journal(directory) {
            messages.push(format!("{}: a settled journal could not be removed ({error}); it will be tried again", directory.display()));
            if !error.cleanup_only() {
                *mandatory = true;
            }
        }
    }
}

/// What:
/// `fn recover_root(root, old, now, judge, between, messages) -> RootDecision` settles one root's lockfile.
/// Why:
/// The decision is recorded for every root,
/// so its manifests follow it and an error never reads as a restoration.
/// The workspace lock is held,
/// so no transaction covers the root meanwhile.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverRoot(root: string, old: LockSnapshot, now: LockSnapshot, judge: Judge, messages: string[]): RootDecision
/// ```
fn recover_root(root: &Path, old: &LockSnapshot, now: &LockSnapshot, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), messages: &mut Vec<String>) -> RootDecision {
    if old.source == now.source {
        return RootDecision::Restored;
    }
    let problems: Vec<String> = judge(std::slice::from_ref(old), std::slice::from_ref(now));
    if problems.is_empty() {
        messages.push(format!("{}/{LOCKFILE_NAME} was left by a run that was killed; its new versions pass the policy, so it was kept", old.crate_dir));
        return RootDecision::Kept;
    }
    let path: PathBuf = root.join(&old.crate_dir).join(LOCKFILE_NAME);
    let keep: KeepAs = KeepAs::next_to_target(REFUSED_SUFFIX);
    return match replace_keeping_with_hook(&path, &now.source, &old.source, &keep, between) {
        Ok(ReplaceOutcome::Replaced(copy)) => {
            messages.push(format!(
                "{}/{LOCKFILE_NAME} was left by a run that was killed and holds versions the policy refuses ({}); it was saved as {} and the earlier lockfile was put back",
                old.crate_dir,
                problems.join("; "),
                match &copy {
                    Some(found) => file_name_text(found),
                    None => "nothing (it had vanished)".to_string(),
                }
            ));
            RootDecision::Restored
        }
        Ok(ReplaceOutcome::LeftAlone) => {
            messages.push(format!("{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored", old.crate_dir));
            RootDecision::LeftAlone
        }
        Ok(ReplaceOutcome::Displaced(kept)) => {
            messages.push(format!(
                "{}/{LOCKFILE_NAME}: the earlier lockfile was put back; the file it displaced is kept as {} (it may not be the one the killed run wrote)",
                old.crate_dir,
                kept.display()
            ));
            RootDecision::Restored
        }
        Ok(ReplaceOutcome::Undone(kept)) => {
            messages.push(format!(
                "{}/{LOCKFILE_NAME} was changed by other processes during the recovery; the undo put the file it first displaced back, and the file the undo displaced is kept as {}",
                old.crate_dir,
                kept.display()
            ));
            RootDecision::LeftAlone
        }
        Err(error) => {
            messages.push(format!("{}/{LOCKFILE_NAME}: could not recover a killed run: {error}; its journal was kept", old.crate_dir));
            RootDecision::Failed
        }
    };
}
