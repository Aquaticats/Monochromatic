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
use crate::journal::{JOURNAL_DIR, KeepAs, REFUSED_SUFFIX, ReplaceOutcome, file_name_text, make_durable, no_hook, replace_keeping_with_hook};
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

/// Reads of a journal directory:
/// index,
/// ready record and listing (split out of the journal recovery module).
pub(crate) use crate::journal_directory_readers::{read_journal, Ready, read_ready, JournalListing, list_journals};

/// What:
/// `pub fn recover(root, judge) -> Vec<String>` finishes the journals of runs that no longer exist.
/// Why:
/// It is `recover_with` with no test hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recover(root: string, judge: Judge): string[]
/// ```
pub fn recover(root: &Path, judge: &dyn Judge) -> Vec<String> {
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
/// `pub trait Judge` is the judgement a recovery asks of the policy about one root:
/// the findings that would refuse a killed run's lockfile.
/// `judge(old, live)` takes the lockfile as the killed run found it and as it stands now,
/// and returns the refusals (empty when it passes).
/// Why:
/// A recovery takes the judgement as a value instead of a closure.
/// Every function with the shape `Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>` is a `Judge` too,
/// so a test passes a closure and the installer passes a `PolicyJudge`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Judge { judge(old: LockSnapshot[], live: LockSnapshot[]): string[]; }
/// ```
pub trait Judge {
    /// What:
    /// `judge(&self, old, live)` returns the refusals for one root.
    /// Why:
    /// An empty list lets the recovery restore or keep the lockfile.
    fn judge(&self, old: &[LockSnapshot], live: &[LockSnapshot]) -> Vec<String>;
}

/// What:
/// The implementation of `Judge` for every function with the judgement's shape.
/// Why:
/// Tests and callers that already hold such a function need no wrapper.
impl<F: Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>> Judge for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the judgement.
    fn judge(&self, old: &[LockSnapshot], live: &[LockSnapshot]) -> Vec<String> {
        return self(old, live);
    }
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
pub fn recover_report(root: &Path, judge: &dyn Judge) -> RecoveryReport {
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
pub fn recover_with(root: &Path, judge: &dyn Judge, between: &dyn Fn(&Path)) -> Vec<String> {
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
pub fn recover_report_with(root: &Path, judge: &dyn Judge, between: &dyn Fn(&Path)) -> RecoveryReport {
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
fn recover_messages(root: &Path, judge: &dyn Judge, between: &dyn Fn(&Path), mandatory: &mut bool) -> Vec<String> {
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
    let mut survivors: Vec<String> = Vec::new();
    for directory in &open {
        if let Some(note) = survivor_note(directory) {
            survivors.push(note);
        }
    }
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
fn recover_one(root: &Path, directory: &Path, judge: &dyn Judge, between: &dyn Fn(&Path), messages: &mut Vec<String>, mandatory: &mut bool) {
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
fn recover_root(root: &Path, old: &LockSnapshot, now: &LockSnapshot, judge: &dyn Judge, between: &dyn Fn(&Path), messages: &mut Vec<String>) -> RootDecision {
    if old.source == now.source {
        return RootDecision::Restored;
    }
    let problems: Vec<String> = judge.judge(std::slice::from_ref(old), std::slice::from_ref(now));
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
