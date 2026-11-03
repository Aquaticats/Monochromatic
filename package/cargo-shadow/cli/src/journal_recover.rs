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
use std::fs::{self, File};
/// Imports `std::io` for this file.
use std::io;
// Imports `std::os::unix::fs::MetadataExt`.
/// Imports `std::os::unix::fs::MetadataExt` for this file.
use std::os::unix::fs::MetadataExt;
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};

// Imports `crate::constants::LOCKFILE_NAME`.
/// Imports `crate::constants::LOCKFILE_NAME` for this file.
use crate::constants::LOCKFILE_NAME;
/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot, read_locks};
// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, J...`.
/// Imports `crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, KeepAs, LOCK_FILE, MANIFEST_INDEX_FILE, READY_FILE, REFUSED_SUFFIX, ReplaceOutcome, decode_path, file_name_text, no_hook, replace_keeping_with_hook, sync_directory}` for this file.
use crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, KeepAs, LOCK_FILE, READY_FILE, REFUSED_SUFFIX, ReplaceOutcome, file_name_text, no_hook, replace_keeping_with_hook, sync_directory};
// Imports `crate::journal_transaction::{mark_done_and_...`.
/// Imports `crate::journal_transaction::{mark_done_and_remove, try_lock}` for this file.
use crate::journal_transaction::{mark_done_and_remove, try_lock};
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
    let mut listed: usize = 0;
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
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
        let (name, length) = line.split_once('\t').ok_or_else(|| { return corrupt("a line has no tab"); })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|_|`: an anonymous function, like a TS arrow function.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `expected: u64`.
        let expected: u64 = length.parse().map_err(|_| { return corrupt("a length is not a number"); })?;
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Typed binding: `found: u64`.
        let found: u64 = fs::metadata(directory.join(name))?.len();
        if found != expected {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
            // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
            // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
            // ```
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(corrupt(&format!("{name} has {found} bytes, the record says {expected}")));
        }
        listed += 1;
    }
    // `Some`: the present or the absent value of an `Option`.
    if ended != Some(listed) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        return Err(corrupt("the end count does not match the lines"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `Some`: the present or the absent value of an `Option`.
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

/// A journal directory whose lock this process holds for the length of one recovery.
struct HeldJournal {
    /// The journal directory.
    directory: PathBuf,
    /// The locked file.
    _lock: File,
}

/// What a try for a journal's lock found.
enum Probe {
    /// The owner is gone;
    /// the lock is ours now.
    Free(
        /// The journal whose lock this process now holds.
        HeldJournal,
    ),
    /// Someone holds the lock:
    /// the owner lives,
    /// or another recovery is at work.
    Held,
    /// The journal has no lock file yet (its owner is still preparing it,
    /// or died doing so);
    /// it is left alone.
    NoLock,
}

/// What:
/// `fn probe(directory: &Path) -> io::Result<Probe>` tries for the lock of a journal without waiting,
/// and confirms that the locked
/// file is the one at the journal's path.
/// Why:
/// A lock taken on a file that was removed and replaced meanwhile proves nothing about the directory now at that path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function probe(directory: string): Probe  // flock LOCK_EX|LOCK_NB, then compare ino of the fd and of the path
/// ```
fn probe(directory: &Path) -> io::Result<Probe> {
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = directory.join(LOCK_FILE);
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&path`: lends the value without copying or giving it away.
    // Typed binding: `lock: File`.
    let lock: File = match File::open(&path) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Probe::NoLock),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => return Err(error),
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&lock`: lends the value without copying or giving it away.
    if !try_lock(&lock)? {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(Probe::Held);
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    let held = lock.metadata()?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&path`: lends the value without copying or giving it away.
    let named = fs::metadata(&path)?;
    if held.dev() != named.dev() || held.ino() != named.ino() {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        return Ok(Probe::Held);
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
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
    // `let mut`: a binding that may change.
    // Typed binding: `roots: Vec<String>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut roots: Vec<String> = Vec::new();
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    let Ok(listing) = fs::read_dir(journals) else {
        return roots;
    };
    for entry in listing.flatten() {
        // Typed binding: `other: PathBuf`.
        let other: PathBuf = entry.path();
        if other == except || other.to_string_lossy().ends_with(DONE_SUFFIX) {
            continue;
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // Borrow `&other`: lends the value without copying or giving it away.
        if let Ok(Probe::Held) = probe(&other)
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            && let Ok(text) = fs::read_to_string(other.join(INDEX_FILE))
        {
            for line in text.lines().skip(1) {
                if !line.starts_with("end ") {
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
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
/// Review finding (3b 6):
/// a kept output must be on disk before the journal that could undo it is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function makeDurable(path: string): void  // fsync the file, then its directory
/// ```
fn make_durable(path: &Path) -> io::Result<()> {
    // `match`: picks the arm whose pattern fits the value.
    match File::open(path) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        Ok(file) => file.sync_all()?,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => return Err(error),
    }
    // `match`: picks the arm whose pattern fits the value.
    return match path.parent() {
        // `Some`: the present or the absent value of an `Option`.
        Some(parent) if !parent.as_os_str().is_empty() => sync_directory(parent),
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
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
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&journals`: lends the value without copying or giving it away.
    let Ok(listing) = fs::read_dir(&journals) else {
        return messages;
    };
    // Closure `|entry|`: an anonymous function, like a TS arrow function.
    // What:     `.collect()` gathers the items an iterator yields into a collection; the collection type (`Vec`, `String`, `BTreeSet`, ...) comes from the type the result is bound to or named in `::<...>`.
    // Why:      Iterator steps are lazy; `collect` is what actually runs them and builds the result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // Array.from(iterable)  // or [...iterable]
    // ```
    // Adapter `.map(`: transforms or selects items with a closure.
    // `let mut`: a binding that may change.
    // Typed binding: `entries: Vec<PathBuf>`.
    let mut entries: Vec<PathBuf> = listing.flatten().map(|entry| { return entry.path(); }).collect();
    entries.sort();
    for directory in entries {
        if directory.to_string_lossy().ends_with(DONE_SUFFIX) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
            // Borrow `&directory`: lends the value without copying or giving it away.
            if let Err(error) = mark_done_and_remove(&directory) {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{}: a finished journal could not be removed ({error}); it will be tried again", directory.display()));
            }
            continue;
        }
        if !directory.is_dir() {
            continue;
        }
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&directory`: lends the value without copying or giving it away.
        // Typed binding: `held: HeldJournal`.
        let held: HeldJournal = match probe(&directory) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(Probe::Free(found)) => found,
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(Probe::Held | Probe::NoLock) => continue,
            Err(error) => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{}: the lock of a journal could not be tried ({error}); it was kept", directory.display()));
                continue;
            }
        };
        // Borrow `&journals`: lends the value without copying or giving it away.
        recover_one(root, &journals, &held, judge, between, &mut messages);
    }
    return messages;
}

/// What:
/// `fn recover_one(root, journals, held, judge, between, messages)` recovers one journal whose lock this process holds.
/// Why:
/// An unready journal changed nothing;
/// a damaged or unsettled one is kept;
/// a settled one is completed and removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverOne(root: string, journals: string, held: HeldJournal, judge: Judge, messages: string[]): void
/// ```
fn recover_one(root: &Path, journals: &Path, held: &HeldJournal, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), messages: &mut Vec<String>) {
    // Borrow `&held.directory`: lends the value without copying or giving it away.
    // Typed binding: `directory: &Path`.
    let directory: &Path = &held.directory;
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `ready: Ready`.
    let ready: Ready = match read_ready(directory) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        Ok(Some(found)) => found,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Ok(None) => {
            // The owner died before the run began, so nothing was changed; removing the copies loses nothing.
            if let Err(error) = mark_done_and_remove(directory) {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{}: a journal that never became ready could not be removed ({error}); it will be tried again", directory.display()));
            }
            return;
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!("{}: the journal of a killed run is damaged ({error}); it was kept", directory.display()));
            return;
        }
    };
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `before: Vec<LockSnapshot>`.
    let before: Vec<LockSnapshot> = match read_journal(directory) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!("{}: the journal of a killed run could not be read ({error}); it was kept", directory.display()));
            return;
        }
    };
    // Typed binding: `manifests: Vec<JournaledManifest>`.
    let manifests: Vec<JournaledManifest> = if ready.manifests {
        // `match`: picks the arm whose pattern fits the value.
        match read_manifest_index(directory) {
            Ok(found) => found,
            Err(error) => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{}: the manifest record of a killed run could not be read ({error}); its journal was kept", directory.display()));
                return;
            }
        }
    } else {
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        Vec::new()
    };
    // `.collect()`: runs the iterator and gathers the items into the bound collection type.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    // Adapter `.map(`: transforms or selects items with a closure.
    // Typed binding: `dirs: Vec<String>`.
    let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&dirs`: lends the value without copying or giving it away.
    // Typed binding: `live: Vec<LockSnapshot>`.
    let live: Vec<LockSnapshot> = match read_locks(root, &dirs) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!("{}: a lockfile of a killed run could not be read ({error}); its journal was kept", directory.display()));
            return;
        }
    };
    // Typed binding: `busy: Vec<String>`.
    let busy: Vec<String> = roots_of_live_journals(journals, directory);
    // `let mut`: a binding that may change.
    // Typed binding: `settled: bool`.
    let mut settled: bool = true;
    // `let mut`: a binding that may change.
    // Typed binding: `decisions: Vec<(String, RootDecision)>`.
    // Constructor `Vec::new()`: an empty owned collection or a boxed value.
    let mut decisions: Vec<(String, RootDecision)> = Vec::new();
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    for (old, now) in before.iter().zip(live.iter()) {
        // Borrow `&busy`: lends the value without copying or giving it away.
        // Typed binding: `decision: RootDecision`.
        let decision: RootDecision = recover_root(root, old, now, &busy, judge, between, messages);
        if decision == RootDecision::Failed {
            settled = false;
        }
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        decisions.push((old.crate_dir.clone(), decision));
    }
    // Borrow `&manifests`: lends the value without copying or giving it away.
    if !recover_manifests(&manifests, &decisions, between, messages) {
        settled = false;
    }
    if settled {
        // Borrow `&decisions`: lends the value without copying or giving it away.
        for (crate_dir, decision) in &decisions {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
            // Borrow `&root.join`: lends the value without copying or giving it away.
            if let Err(error) = make_durable(&root.join(crate_dir).join(LOCKFILE_NAME)) {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{crate_dir}/{LOCKFILE_NAME} could not be synced ({error}); the journal was kept"));
                return;
            }
            // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
            // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // void expr;   // or try { expr } catch {}
            // ```
            let _ = decision;
        }
        // Borrow `&manifests`: lends the value without copying or giving it away.
        for manifest in &manifests {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
            // Borrow `&manifest.path`: lends the value without copying or giving it away.
            if let Err(error) = make_durable(&manifest.path) {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{} could not be synced ({error}); the journal was kept", manifest.path.display()));
                return;
            }
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Err(error) = mark_done_and_remove(directory) {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!("{}: a settled journal could not be removed ({error}); it will be tried again", directory.display()));
        }
    }
}

/// What:
/// `fn recover_root(root, old, now, busy, judge, between, messages) -> RootDecision` settles one root's lockfile.
/// Why:
/// The decision is recorded for every root,
/// so its manifests follow it and an error never reads as a restoration.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverRoot(root: string, old: LockSnapshot, now: LockSnapshot, busy: string[], judge: Judge, messages: string[]): RootDecision
/// ```
fn recover_root(root: &Path, old: &LockSnapshot, now: &LockSnapshot, busy: &[String], judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path), messages: &mut Vec<String>) -> RootDecision {
    // Borrow `&old.crate_dir`: lends the value without copying or giving it away.
    if busy.contains(&old.crate_dir) {
        // Macro `format!`: builds or prints text (or a list) from a template.
        messages.push(format!("{} is covered by a live transaction of another run; the journal of a killed run was kept and a later run recovers it", old.crate_dir));
        return RootDecision::Failed;
    }
    if old.source == now.source {
        return RootDecision::Restored;
    }
    // Typed binding: `problems: Vec<String>`.
    let problems: Vec<String> = judge(std::slice::from_ref(old), std::slice::from_ref(now));
    if problems.is_empty() {
        // Macro `format!`: builds or prints text (or a list) from a template.
        messages.push(format!("{}/{LOCKFILE_NAME} was left by a run that was killed; its new versions pass the policy, so it was kept", old.crate_dir));
        return RootDecision::Kept;
    }
    // Borrow `&old.crate_dir`: lends the value without copying or giving it away.
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = root.join(&old.crate_dir).join(LOCKFILE_NAME);
    // Typed binding: `keep: KeepAs`.
    let keep: KeepAs = KeepAs::next_to_target(REFUSED_SUFFIX);
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&path`: lends the value without copying or giving it away.
    return match replace_keeping_with_hook(&path, &now.source, &old.source, &keep, between) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(ReplaceOutcome::Replaced(copy)) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!(
                "{}/{LOCKFILE_NAME} was left by a run that was killed and holds versions the policy refuses ({}); it was saved as {} and the earlier lockfile was put back",
                old.crate_dir,
                problems.join("; "),
                // `match`: picks the arm whose pattern fits the value.
                // Borrow `&copy`: lends the value without copying or giving it away.
                match &copy {
                    // `Some`: the present or the absent value of an `Option`.
                    Some(found) => file_name_text(found),
                    // `None`: the present or the absent value of an `Option`.
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    None => "nothing (it had vanished)".to_string(),
                }
            ));
            RootDecision::Restored
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(ReplaceOutcome::LeftAlone) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!("{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored", old.crate_dir));
            RootDecision::LeftAlone
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(ReplaceOutcome::Displaced(kept)) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!(
                "{}/{LOCKFILE_NAME} was left by a run that was killed; the earlier lockfile was put back, but the file it displaced was not the one that run wrote; that file is kept as {}",
                old.crate_dir,
                kept.display()
            ));
            RootDecision::Restored
        }
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(ReplaceOutcome::Undone(kept)) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!(
                "{}/{LOCKFILE_NAME} was changed twice by other processes while a killed run was being recovered; it was left as the last one wrote it, and the snapshot copy is kept as {}",
                old.crate_dir,
                kept.display()
            ));
            RootDecision::LeftAlone
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        Err(error) => {
            // Macro `format!`: builds or prints text (or a list) from a template.
            messages.push(format!("{}/{LOCKFILE_NAME}: could not recover a killed run: {error}; its journal was kept", old.crate_dir));
            RootDecision::Failed
        }
    };
}
