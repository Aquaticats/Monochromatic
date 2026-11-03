//! What:
//! The recovery of a run that was killed: take the lock of its journal, decide per root whether the leftover lockfile is kept or the
//! snapshot put back, and put back the manifests of a killed `--breaking` run.
//! Why:
//! `journal.rs` holds the recording side;
//! the recovery side was split out to keep each file under the line limit.
//! A journal is recovered only when its owner's lock can be taken (the owner is gone) and only after the same lock file is
//! confirmed to be the one at the journal's path.
//! A root that a live transaction covers, a root whose lockfile could not be settled and a root with no decision are never
//! restored, so a lockfile and its manifests are never split.
//! Nothing here waits: a held lock is an answer, and the journal stays for a later run.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-recover.ts
//! export function recover(root: string, judge: Judge): string[]
//! ```

use std::fs::{self, File};
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::constants::LOCKFILE_NAME;
use crate::crates::{LockSnapshot, read_locks};
use crate::file_owner::observe;
use crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, KeepAs, LOCK_FILE, MANIFEST_INDEX_FILE, READY_FILE, REFUSED_SUFFIX, ReplaceOutcome, decode_path, file_name_text, no_hook, replace_keeping_with_hook, sync_directory};
use crate::journal_transaction::{mark_done_and_remove, try_lock};

/// What:
/// `pub(crate) fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>>` reads the lockfile index and the snapshots.
/// Why:
/// The index is versioned and ends with a count, so a truncated or empty index is an error and never a shorter list of roots.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readJournal(directory: string): LockSnapshot[]  // throws on a damaged journal
/// ```
pub(crate) fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>> {
    let corrupt = |why: &str| -> io::Error { return io::Error::new(io::ErrorKind::InvalidData, format!("the lockfile index is corrupt: {why}")); };
    let index: String = fs::read_to_string(directory.join(INDEX_FILE))?;
    let mut lines = index.lines();
    if lines.next() != Some("v2") {
        return Err(corrupt("no v2 header"));
    }
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    let mut ended: Option<usize> = None;
    for line in lines {
        if let Some(count) = line.strip_prefix("end ") {
            ended = Some(count.trim().parse().map_err(|_| { return corrupt("the end count is not a number"); })?);
            continue;
        }
        if ended.is_some() {
            return Err(corrupt("text after the end line"));
        }
        let source: String = fs::read_to_string(directory.join(format!("{}.lock", snapshots.len())))?;
        snapshots.push(LockSnapshot { crate_dir: line.to_string(), source });
    }
    if ended != Some(snapshots.len()) {
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
/// lists is present with exactly the listed length; `Ok(None)` means there is no ready record (the journal was never completed).
/// Why:
/// Review finding (3b 7): a missing or short file of a ready journal is damage, and damage keeps the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReady(directory: string): Ready | undefined  // throws on a damaged journal
/// ```
pub(crate) fn read_ready(directory: &Path) -> io::Result<Option<Ready>> {
    let corrupt = |why: &str| -> io::Error { return io::Error::new(io::ErrorKind::InvalidData, format!("the ready record is corrupt: {why}")); };
    let text: String = match fs::read_to_string(directory.join(READY_FILE)) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut lines = text.lines();
    if lines.next() != Some("v1") {
        return Err(corrupt("no v1 header"));
    }
    let manifests: bool = match lines.next() {
        Some("manifests 1") => true,
        Some("manifests 0") => false,
        _ => return Err(corrupt("no manifests line")),
    };
    let mut listed: usize = 0;
    let mut ended: Option<usize> = None;
    for line in lines {
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
        listed += 1;
    }
    if ended != Some(listed) {
        return Err(corrupt("the end count does not match the lines"));
    }
    return Ok(Some(Ready { manifests }));
}

/// One manifest recovered from a journal.
pub(crate) struct JournaledManifest {
    /// The crate root the manifest belongs to.
    crate_dir: String,
    /// The manifest's path,
    /// byte for byte.
    path: PathBuf,
    /// The manifest's text before the run.
    before: String,
}

/// What:
/// `fn read_manifest_index(directory: &Path) -> io::Result<Vec<JournaledManifest>>` reads the manifest records of a journal.
/// `Ok` with an empty list means the transaction journaled no manifests (the file does not exist);
/// every other failure,
/// including a missing `end` line or a bad count,
/// is an error.
/// Why:
/// An unreadable index must keep the journal:
/// it is the only copy of the original manifests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readManifestIndex(directory: string): JournaledManifest[]  // throws unless the record is complete
/// ```
pub(crate) fn read_manifest_index(directory: &Path) -> io::Result<Vec<JournaledManifest>> {
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `text: String`.
    let text: String = match fs::read_to_string(directory.join(MANIFEST_INDEX_FILE)) {
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => return Err(error),
    };
    // Macro `format!`: builds or prints text (or a list) from a template.
    let corrupt = |why: &str| -> io::Error { return io::Error::new(io::ErrorKind::InvalidData, format!("the manifest index is corrupt: {why}")); };
    // `let mut`: a binding that may change.
    let mut lines = text.lines();
    // `Some`: the present or the absent value of an `Option`.
    if lines.next() != Some("v1") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("no v1 header"));
    }
    // `let mut`: a binding that may change.
    // Typed binding: `found: Vec<JournaledManifest>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut found: Vec<JournaledManifest> = Vec::new();
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
            // What:     `.map_err(f)` converts the error type of a `Result` with the function `f` and leaves a success untouched.
            // Why:      The function's own error type differs from the callee's, and `?` needs them to agree.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // try { ... } catch (error) { throw new MyError(String(error)); }
            // ```
            ended = Some(count.trim().parse().map_err(|_| { return corrupt("the end count is not a number"); })?);
            continue;
        }
        if ended.is_some() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            return Err(corrupt("text after the end line"));
        }
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
        let (crate_dir, encoded) = line.split_once('\t').ok_or_else(|| { return corrupt("a line has no tab"); })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
        // Typed binding: `path: PathBuf`.
        let path: PathBuf = decode_path(encoded).ok_or_else(|| { return corrupt("a path is not hexadecimal"); })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `before: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let before: String = fs::read_to_string(directory.join(format!("m{}.toml", found.len())))?;
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        found.push(JournaledManifest { crate_dir: crate_dir.to_string(), path, before });
    }
    // `Some`: the present or the absent value of an `Option`.
    if ended != Some(found.len()) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("the end count does not match the lines"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(found);
}

/// What one root's recovery decided.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootDecision {
    /// The leftover lockfile passed the policy and was kept; the root's manifest edits stay.
    Kept,
    /// The snapshot is live (or nothing differed); the root's manifests go back.
    Restored,
    /// Another writer's file is live; the root's manifests are left alone too.
    LeftAlone,
    /// The root could not be settled (an error, a live transaction); nothing of the root is touched and the journal is kept.
    Failed,
}

/// What:
/// `pub(crate) fn recover_manifests(manifests, decisions, between, messages) -> bool` puts back the manifests of the roots whose
/// lockfile was restored.
/// Why:
/// A root with no decision, or a failed one, is never restored: the pair lockfile and manifest is not split.
/// Returns whether every manifest was settled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverManifests(manifests: JournaledManifest[], decisions: Map<string, RootDecision>, messages: string[]): boolean
/// ```
pub(crate) fn recover_manifests(manifests: &[JournaledManifest], decisions: &[(String, RootDecision)], between: &dyn Fn(&Path), messages: &mut Vec<String>) -> bool {
    let mut all_settled: bool = true;
    for manifest in manifests {
        let decision: RootDecision = decisions.iter().find(|(crate_dir, _)| { return *crate_dir == manifest.crate_dir; }).map(|(_, found)| { return *found; }).unwrap_or(RootDecision::Failed);
        if decision == RootDecision::Failed {
            all_settled = false;
            continue;
        }
        if decision != RootDecision::Restored {
            continue;
        }
        let shown: String = manifest.path.display().to_string();
        let live: String = match observe(&manifest.path) {
            Ok(Some(found)) => found.text,
            Ok(None) => {
                messages.push(format!("{shown} is missing; the manifest of a killed run was not put back"));
                continue;
            }
            Err(error) => {
                all_settled = false;
                messages.push(format!("{shown} could not be read ({error}); its journal was kept"));
                continue;
            }
        };
        if live == manifest.before {
            continue;
        }
        let keep: KeepAs = KeepAs::next_to_target(REFUSED_SUFFIX);
        match replace_keeping_with_hook(&manifest.path, &live, &manifest.before, &keep, between) {
            Ok(ReplaceOutcome::Replaced(copy)) => messages.push(format!(
                "{shown} was rewritten by a run that was killed and its lockfile was not kept; the earlier manifest was put back and the rewritten text is kept as {}",
                match copy {
                    Some(found) => found.display().to_string(),
                    None => "nothing (it had vanished)".to_string(),
                }
            )),
            Ok(ReplaceOutcome::LeftAlone) => messages.push(format!("{shown} was changed by another process while a killed run was being recovered; it was left as that process wrote it")),
            Ok(ReplaceOutcome::Displaced(kept)) => messages.push(format!("{shown} was put back, but the file it displaced was not the one the killed run wrote; that file is kept as {}", kept.display())),
            Ok(ReplaceOutcome::Undone(kept)) => messages.push(format!("{shown} was changed twice by other processes while a killed run was being recovered; it was left as the last one wrote it, and the snapshot copy is kept as {}", kept.display())),
            Err(error) => {
                all_settled = false;
                messages.push(format!("{shown}: could not recover a killed run: {error}; its journal was kept"));
            }
        }
    }
    return all_settled;
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
    return recover_with(root, judge, &no_hook);
}

/// A journal directory whose lock this process holds for the length of one recovery.
struct HeldJournal {
    /// The journal directory.
    directory: PathBuf,
    /// The locked file.
    _lock: File,
}

/// What a try for a journal's lock found.
enum Probe {
    /// The owner is gone; the lock is ours now.
    Free(HeldJournal),
    /// Someone holds the lock: the owner lives, or another recovery is at work.
    Held,
    /// The journal has no lock file yet (its owner is still preparing it, or died doing so); it is left alone.
    NoLock,
}

/// What:
/// `fn probe(directory: &Path) -> io::Result<Probe>` tries for the lock of a journal without waiting, and confirms that the locked
/// file is the one at the journal's path.
/// Why:
/// A lock taken on a file that was removed and replaced meanwhile proves nothing about the directory now at that path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probe(directory: string): Probe  // flock LOCK_EX|LOCK_NB, then compare ino of the fd and of the path
/// ```
fn probe(directory: &Path) -> io::Result<Probe> {
    let path: PathBuf = directory.join(LOCK_FILE);
    let lock: File = match File::open(&path) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Probe::NoLock),
        Err(error) => return Err(error),
    };
    if !try_lock(&lock)? {
        return Ok(Probe::Held);
    }
    let held = lock.metadata()?;
    let named = fs::metadata(&path)?;
    if held.dev() != named.dev() || held.ino() != named.ino() {
        return Ok(Probe::Held);
    }
    return Ok(Probe::Free(HeldJournal { directory: directory.to_path_buf(), _lock: lock }));
}

/// What:
/// `fn roots_of_live_journals(journals: &Path, except: &Path) -> Vec<String>` lists the crate roots that other journals with a held
/// lock cover.
/// Why:
/// Recovering a dead run must not restore the inputs of a root a live transaction is using.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsOfLiveJournals(journals: string, except: string): string[]
/// ```
fn roots_of_live_journals(journals: &Path, except: &Path) -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();
    let Ok(listing) = fs::read_dir(journals) else {
        return roots;
    };
    for entry in listing.flatten() {
        let other: PathBuf = entry.path();
        if other == except || other.to_string_lossy().ends_with(DONE_SUFFIX) {
            continue;
        }
        if let Ok(Probe::Held) = probe(&other)
            && let Ok(text) = fs::read_to_string(other.join(INDEX_FILE))
        {
            for line in text.lines().skip(1) {
                if !line.starts_with("end ") {
                    roots.push(line.to_string());
                }
            }
        }
    }
    return roots;
}

/// What:
/// `fn make_durable(path: &Path) -> io::Result<()>` syncs a file and the directory that lists it.
/// Why:
/// Review finding (3b 6): a kept output must be on disk before the journal that could undo it is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function makeDurable(path: string): void  // fsync the file, then its directory
/// ```
fn make_durable(path: &Path) -> io::Result<()> {
    match File::open(path) {
        Ok(file) => file.sync_all()?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    return match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => sync_directory(parent),
        _ => Ok(()),
    };
}

/// What:
/// `pub fn recover_with(root, judge, between) -> Vec<String>` is `recover` with a hook that runs before each replacement.
/// Why:
/// Tests publish a file in the gap a replacement leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverWith(root: string, judge: Judge, between: (path: string) => void): string[]
/// ```
pub fn recover_with(root: &Path, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path)) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let journals: PathBuf = root.join(JOURNAL_DIR);
    let Ok(listing) = fs::read_dir(&journals) else {
        return messages;
    };
    let mut entries: Vec<PathBuf> = listing.flatten().map(|entry| { return entry.path(); }).collect();
    entries.sort();
    for directory in entries {
        if directory.to_string_lossy().ends_with(DONE_SUFFIX) {
            if let Err(error) = mark_done_and_remove(&directory) {
                messages.push(format!("{}: a finished journal could not be removed ({error}); it will be tried again", directory.display()));
            }
            continue;
        }
        if !directory.is_dir() {
            continue;
        }
        let held: HeldJournal = match probe(&directory) {
            Ok(Probe::Free(found)) => found,
            Ok(Probe::Held | Probe::NoLock) => continue,
            Err(error) => {
                messages.push(format!("{}: the lock of a journal could not be tried ({error}); it was kept", directory.display()));
                continue;
            }
        };
        recover_one(root, &journals, &held, judge, between, &mut messages);
    }
    return messages;
}

/// What:
/// `fn recover_one(root, journals, held, judge, between, messages)` recovers one journal whose lock this process holds.
/// Why:
/// An unready journal changed nothing; a damaged or unsettled one is kept; a settled one is completed and removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverOne(root: string, journals: string, held: HeldJournal, judge: Judge, messages: string[]): void
/// ```
fn recover_one(root: &Path, journals: &Path, held: &HeldJournal, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), messages: &mut Vec<String>) {
    let directory: &Path = &held.directory;
    let ready: Ready = match read_ready(directory) {
        Ok(Some(found)) => found,
        Ok(None) => {
            // The owner died before the run began, so nothing was changed; removing the copies loses nothing.
            if let Err(error) = mark_done_and_remove(directory) {
                messages.push(format!("{}: a journal that never became ready could not be removed ({error}); it will be tried again", directory.display()));
            }
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
    let busy: Vec<String> = roots_of_live_journals(journals, directory);
    let mut settled: bool = true;
    let mut decisions: Vec<(String, RootDecision)> = Vec::new();
    for (old, now) in before.iter().zip(live.iter()) {
        let decision: RootDecision = recover_root(root, old, now, &busy, judge, between, messages);
        if decision == RootDecision::Failed {
            settled = false;
        }
        decisions.push((old.crate_dir.clone(), decision));
    }
    if !recover_manifests(&manifests, &decisions, between, messages) {
        settled = false;
    }
    if settled {
        for (crate_dir, decision) in &decisions {
            if let Err(error) = make_durable(&root.join(crate_dir).join(LOCKFILE_NAME)) {
                messages.push(format!("{crate_dir}/{LOCKFILE_NAME} could not be synced ({error}); the journal was kept"));
                return;
            }
            let _ = decision;
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
/// `fn recover_root(root, old, now, busy, judge, between, messages) -> RootDecision` settles one root's lockfile.
/// Why:
/// The decision is recorded for every root, so its manifests follow it and an error never reads as a restoration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverRoot(root: string, old: LockSnapshot, now: LockSnapshot, busy: string[], judge: Judge, messages: string[]): RootDecision
/// ```
fn recover_root(root: &Path, old: &LockSnapshot, now: &LockSnapshot, busy: &[String], judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), messages: &mut Vec<String>) -> RootDecision {
    if busy.contains(&old.crate_dir) {
        messages.push(format!("{} is covered by a live transaction of another run; the journal of a killed run was kept and a later run recovers it", old.crate_dir));
        return RootDecision::Failed;
    }
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
                "{}/{LOCKFILE_NAME} was left by a run that was killed; the earlier lockfile was put back, but the file it displaced was not the one that run wrote; that file is kept as {}",
                old.crate_dir,
                kept.display()
            ));
            RootDecision::Restored
        }
        Ok(ReplaceOutcome::Undone(kept)) => {
            messages.push(format!(
                "{}/{LOCKFILE_NAME} was changed twice by other processes while a killed run was being recovered; it was left as the last one wrote it, and the snapshot copy is kept as {}",
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
