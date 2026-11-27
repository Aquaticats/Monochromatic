//! What:
//! The recovery of one journal:
//! which of its roots keep the leftover lockfile and which get the snapshot back,
//! the temporaries a killed run left beside them,
//! and the end of the journal once every root is settled.
//! Why:
//! `journal_recover.rs` walks the journals of a workspace and decides when a recovery may run at all;
//! the work on one journal was split out of it to keep each file under the code-line limit.
//! `recover_one` is called by that walk and imports its `Judge` from it,
//! and the other two items are private to this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-recover-one.ts
//! function recoverOne(root: string, directory: string, judge: Judge, between: (path: string) => void, messages: string[]): void
//! function recoverRoot(root: string, old: LockSnapshot, now: LockSnapshot, judge: Judge, messages: string[]): RootDecision
//! ```

/// Imports `fs` to list the crate roots a killed run left temporaries in.
use std::fs;
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the lockfile name a restored root puts back.
use crate::constants::LOCKFILE_NAME;
/// Imports the lockfile snapshot reader.
use crate::crates::{
    LockSnapshot,
    read_locks,
};
/// Imports the journal file name, the kept-file naming and the replace outcome.
use crate::journal::{
    KeepAs,
    REFUSED_SUFFIX,
    ReplaceOutcome,
    file_name_text,
    make_durable,
    replace_keeping_with_hook,
};
/// Imports the reads of a journal directory the recovery settles from.
use crate::journal_directory_readers::{
    Ready,
    read_journal,
    read_ready,
};
/// Imports the manifest side of a recovery.
use crate::journal_manifests::{
    JournaledManifest,
    RootDecision,
    read_manifest_index,
    recover_manifests,
};
/// Imports the judgement the walk holds and hands to every root.
use crate::journal_recover::Judge;
/// Imports the completion of a journal.
use crate::journal_transaction::finish_journal;
/// Imports the recovery of a journal that never became ready.
use crate::journal_unready::recover_unready;
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
fn report_leftover_temporaries(
    root: &Path,
    crate_dirs: &[String],
    messages: &mut Vec<String>,
) {
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
pub(crate) fn recover_one(
    root: &Path,
    directory: &Path,
    judge: &dyn Judge,
    between: &dyn Fn(&Path),
    messages: &mut Vec<String>,
    mandatory: &mut bool,
) {
    let ready: Ready = match read_ready(directory) {
        Ok(Some(found)) => found,
        Ok(None) => {
            recover_unready(root, directory, messages, mandatory);
            return;
        }
        Err(error) => {
            messages.push(format!(
                "{}: the journal of a killed run is damaged ({error}); it was kept",
                directory.display()
            ));
            return;
        }
    };
    let before: Vec<LockSnapshot> = match read_journal(directory) {
        Ok(found) => found,
        Err(error) => {
            messages.push(format!(
                "{}: the journal of a killed run could not be read ({error}); it was kept",
                directory.display()
            ));
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
            messages.push(format!(
                "{}: a lockfile of a killed run could not be read ({error}); its journal was kept",
                directory.display()
            ));
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
                messages.push(format!(
                    "{} could not be synced ({error}); the journal was kept",
                    manifest.path.display()
                ));
                *mandatory = true;
                return;
            }
        }
        if let Err(error) = finish_journal(directory) {
            messages.push(format!(
                "{}: a settled journal could not be removed ({error}); it will be tried again",
                directory.display()
            ));
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
fn recover_root(
    root: &Path,
    old: &LockSnapshot,
    now: &LockSnapshot,
    judge: &dyn Judge,
    between: &dyn Fn(&Path),
    messages: &mut Vec<String>,
) -> RootDecision {
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
            messages.push(format!(
                "{}/{LOCKFILE_NAME}: could not recover a killed run: {error}; its journal was kept",
                old.crate_dir
            ));
            RootDecision::Failed
        }
    };
}
