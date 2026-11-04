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
use crate::journal::{DONE_SUFFIX, GROUPS_FILE, INDEX_FILE, JOURNAL_DIR, KeepAs, MANIFEST_INDEX_FILE, READY_FILE, REFUSED_SUFFIX, ReplaceOutcome, file_name_text, make_durable, no_hook, replace_keeping_with_hook};
// Imports `crate::journal_transaction::{mark_done_and_...`.
/// Imports `crate::journal_transaction::{mark_done_and_remove, try_lock}` for this file.
use crate::journal_transaction::{mark_done_and_remove, take_workspace};
/// Imports the process-group probe and the error code that says a group is empty.
use rustix::io::Errno;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
/// Imports `rustix::process::{Pid, test_kill_process_group}` for this file.
use rustix::process::{Pid, test_kill_process_group};
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
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `let mut`: a binding that may change.
    // Typed binding: `required: Vec<String>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let mut required: Vec<String> = vec![INDEX_FILE.to_string()];
    // Closure `|position|`: an anonymous function, like a TS arrow function.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Macro `format!`: builds or prints text (or a list) from a template.
    required.extend((0..roots).map(|position| { return format!("{position}.lock"); }));
    if manifests {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        required.push(MANIFEST_INDEX_FILE.to_string());
    }
    // Borrow `&required`: lends the value without copying or giving it away.
    for name in &required {
        if !listed.contains(name) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Borrow `&format`: lends the value without copying or giving it away.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(corrupt(&format!("{name} is required and not listed")));
        }
    }
    return Ok(Some(Ready { manifests }));
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
    let mut messages: Vec<String> = Vec::new();
    let journals: PathBuf = root.join(JOURNAL_DIR);
    if !journals.is_dir() {
        return messages;
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
    let mut entries: Vec<PathBuf> = match fs::read_dir(&journals) {
        Ok(listing) => listing.flatten().map(|entry| { return entry.path(); }).collect(),
        Err(error) => {
            messages.push(format!("{} could not be listed ({error}); the journals of earlier runs were left for a later run", journals.display()));
            return messages;
        }
    };
    entries.sort();
    let mut open: Vec<PathBuf> = Vec::new();
    for directory in entries {
        if directory.to_string_lossy().ends_with(DONE_SUFFIX) {
            if let Err(error) = mark_done_and_remove(&directory) {
                messages.push(format!("{}: a finished journal could not be removed ({error}); it will be tried again", directory.display()));
            }
        } else if directory.is_dir() {
            open.push(directory);
        }
    }
    let survivors: Vec<String> = open.iter().filter_map(|directory| { return survivor_note(directory); }).collect();
    if !survivors.is_empty() {
        messages.extend(survivors);
        messages.push("no journal of this workspace was recovered while a child of an earlier run may still be writing".to_string());
        return messages;
    }
    for directory in open {
        recover_one(root, &directory, judge, between, &mut messages);
    }
    return messages;
}

/// What:
/// `pub(crate) fn survivor_note(directory: &Path) -> Option<String>` says why a journal's recorded children may still be running:
/// a recorded process group that still has a member, or a group record that cannot be read or understood.
/// Why:
/// The end of the wrapper's lock does not say whether its cargo ended.
/// A missing, unreadable or malformed record is uncertainty, never "no children" (round 6, 5b2 2).
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

/// What:
/// `fn recover_unready(root, directory, messages)` settles a journal that has no ready record.
/// Why:
/// Review finding (4b2 6):
/// no ready record is normally a run killed while it prepared,
/// before cargo ran,
/// and then the copies lose
/// nothing;
/// but a ready record that was lost to damage looks the same.
/// So the copies are removed only when no lockfile differs from its snapshot (the copies are then redundant);
/// otherwise the journal is kept and named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverUnready(root: string, directory: string, messages: string[]): void
/// ```
fn recover_unready(root: &Path, directory: &Path, messages: &mut Vec<String>) {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `redundant: bool`.
    let redundant: bool = match read_journal(directory) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == io::ErrorKind::NotFound => true,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(_) => false,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(before) => {
            // `.collect()`: runs the iterator and gathers the items into the bound collection type.
            // What:     `.iter()` walks a collection by reference without consuming it; `.into_iter()` consumes it and yields owned items; `.cloned()`/`.copied()` turn the borrowed items into owned copies.
            // Why:      The collection stays usable after the walk and no item is copied until it must be.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // for (const item of list) { ... }  // or list.map(...)
            // ```
            // Adapter `.map(`: transforms or selects items with a closure.
            // Typed binding: `dirs: Vec<String>`.
            let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&dirs`: lends the value without copying or giving it away.
            match read_locks(root, &dirs) {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // Closure `|(old, now)|`: an anonymous function, like a TS arrow function.
                // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
                Ok(live) => before.iter().zip(live.iter()).all(|(old, now)| { return old.source == now.source; }),
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                Err(_) => false,
            }
        }
    };
    if !redundant {
        // Macro `format!`: builds or prints text (or a list) from a template.
        messages.push(format!("{}: a journal without a ready record holds the only copy of a lockfile that differs from the live one; it was kept", directory.display()));
        return;
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let Err(error) = mark_done_and_remove(directory) {
        // Macro `format!`: builds or prints text (or a list) from a template.
        messages.push(format!("{}: a journal that never became ready could not be removed ({error}); it will be tried again", directory.display()));
    }
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
fn recover_one(root: &Path, directory: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), messages: &mut Vec<String>) {
    let ready: Ready = match read_ready(directory) {
        Ok(Some(found)) => found,
        Ok(None) => {
            recover_unready(root, directory, messages);
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
        for (crate_dir, _decision) in &decisions {
            if let Err(error) = make_durable(&root.join(crate_dir).join(LOCKFILE_NAME)) {
                messages.push(format!("{crate_dir}/{LOCKFILE_NAME} could not be synced ({error}); the journal was kept"));
                return;
            }
        }
        for manifest in &manifests {
            if let Err(error) = make_durable(&manifest.path) {
                messages.push(format!("{} could not be synced ({error}); the journal was kept", manifest.path.display()));
                return;
            }
        }
        if let Err(error) = mark_done_and_remove(directory) {
            messages.push(format!("{}: a settled journal could not be removed ({error}); it will be tried again", directory.display()));
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
