//! What:
//! The recovery of a run that was killed:
//! claim its journal,
//! decide per root whether the leftover lockfile is kept or the
//! snapshot put back,
//! and put back the manifests of a killed `--breaking` run.
//! Why:
//! `journal.rs` holds the recording side (snapshots,
//! the transaction's restore,
//! the journal files);
//! the recovery side was split out to keep each file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-recover.ts
//! export function recover(root: string, isAlive: (pid: number, identity: string) => boolean, judge: Judge): string[]
//! ```

/// Imports `fs` to read and rename the journal files.
use std::fs;
/// Imports `io` for the file-system error type.
use std::io;
/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// Imports the snapshot type.
use crate::crates::{LockSnapshot, read_locks};
/// Imports the ownership evidence reader.
use crate::file_owner::observe;
/// Imports the places a displaced file is kept and the replace functions.
use crate::journal::{INDEX_FILE, JOURNAL_DIR, KeepAs, MANIFEST_INDEX_FILE, OWNER_FILE, REFUSED_SUFFIX, ReplaceOutcome, decode_path, file_name_text, no_hook, process_exists, process_identity, replace_keeping_with_hook, sync_directory};
/// Imports the lockfile name.
use crate::constants::LOCKFILE_NAME;

/// What:
/// `fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>>` reads a journal's snapshots.
/// Why:
/// Any missing or unreadable part is an error,
/// so the journal is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readJournal(directory: string): LockSnapshot[]  // throws on any missing part
/// ```
pub(crate) fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>> {
    let index: String = fs::read_to_string(directory.join(INDEX_FILE))?;
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    for (position, crate_dir) in index.lines().enumerate() {
        let source: String = fs::read_to_string(directory.join(format!("{position}.lock")))?;
        snapshots.push(LockSnapshot { crate_dir: crate_dir.to_string(), source });
    }
    return Ok(snapshots);
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

/// What a recovery decided about one crate root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RootDecision {
    /// The lockfile the killed run left passed the policy and was kept.
    Kept,
    /// The lockfile was put back,
    /// or already equalled the snapshot.
    Restored,
    /// Another writer's lockfile is in place and was left alone.
    LeftAlone,
}

/// What:
/// `fn recover_manifests(manifests, decisions, between, messages) -> bool` puts back the manifests of the roots whose lockfile was
/// put back,
/// saving the text it replaces as `<name>.refused`,
/// and says whether all were settled.
/// Why:
/// Each root is decided on its own:
/// a kept lockfile keeps its manifests,
/// a lockfile left to another writer keeps its manifests too,
/// and only a root whose lockfile went back gets its manifests back.
/// A manifest that cannot be read,
/// or cannot be put back,
/// keeps the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverManifests(manifests, decisions, between, messages): boolean
/// ```
pub(crate) fn recover_manifests(manifests: &[JournaledManifest], decisions: &[(String, RootDecision)], between: &dyn Fn(&Path), messages: &mut Vec<String>) -> bool {
    let mut all_settled: bool = true;
    for manifest in manifests {
        // Typed binding: `decision: RootDecision`.
        let decision: RootDecision = decisions
            // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
            .iter()
            // Closure `|(crate_dir, _)|`: an anonymous function, like a TS arrow function.
            .find(|(crate_dir, _)| { return *crate_dir == manifest.crate_dir; })
            // Closure `|(_, found)|`: an anonymous function, like a TS arrow function.
            // Adapter `.map(`: transforms or selects items with a closure.
            .map(|(_, found)| { return *found; })
            // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
            .unwrap_or(RootDecision::Restored);
        if decision != RootDecision::Restored {
            continue;
        }
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Typed binding: `shown: String`.
        let shown: String = manifest.path.display().to_string();
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&manifest.path`: lends the value without copying or giving it away.
        // Typed binding: `live: String`.
        let live: String = match observe(&manifest.path) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `Some`: the present or the absent value of an `Option`.
            Ok(Some(found)) => found.text,
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            Ok(None) => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{shown} is missing; the manifest of a killed run was not put back"));
                continue;
            }
            Err(error) => {
                all_settled = false;
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{shown} could not be read ({error}); its journal was kept"));
                continue;
            }
        };
        if live == manifest.before {
            continue;
        }
        // Typed binding: `keep: KeepAs`.
        let keep: KeepAs = KeepAs::next_to_target(REFUSED_SUFFIX);
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&manifest.path`: lends the value without copying or giving it away.
        match replace_keeping_with_hook(&manifest.path, &live, &manifest.before, &keep, between) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok(ReplaceOutcome::Replaced(copy)) => messages.push(format!(
                "{shown} was rewritten by a run that was killed and its lockfile was not kept; the earlier manifest was put back and the rewritten text is kept as {}",
                // `match`: picks the arm whose pattern fits the value.
                match copy {
                    // `Some`: the present or the absent value of an `Option`.
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    Some(found) => found.display().to_string(),
                    // `None`: the present or the absent value of an `Option`.
                    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                    None => "nothing (it had vanished)".to_string(),
                }
            )),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok(ReplaceOutcome::LeftAlone) => messages.push(format!("{shown} was changed by another process while a killed run was being recovered; it was left as that process wrote it")),
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Ok(ReplaceOutcome::Displaced(kept)) => messages.push(format!("{shown} was put back, but the file it displaced was not the one the killed run wrote; that file is kept as {}", kept.display())),
            Ok(ReplaceOutcome::Undone(kept)) => messages.push(format!("{shown} was changed twice by other processes while a killed run was being recovered; it was left as the last one wrote it, and the snapshot copy is kept as {}", kept.display())),
            Err(error) => {
                all_settled = false;
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{shown}: could not recover a killed run: {error}; its journal was kept"));
            }
        }
    }
    return all_settled;
}

/// What:
/// `fn read_identity(directory: &Path) -> String` reads the process start time a journal recorded (empty when it recorded none).
/// Why:
/// The journal's directory name carries the process id;
/// the start time tells a reused id from the process that wrote the journal.
/// A journal without an owner record (written by an older version) has an empty identity,
/// which accepts any process with that id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readIdentity(directory: string): string
/// ```
pub(crate) fn read_identity(directory: &Path) -> String {
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `text: String`.
    let text: String = fs::read_to_string(directory.join(OWNER_FILE)).unwrap_or_default();
    // `let mut`: a binding that may change.
    let mut lines = text.lines();
    // `Some`: the present or the absent value of an `Option`.
    if lines.next() != Some("v1") {
        // Constructor `String::new()`: an empty owned collection or a boxed value.
        return String::new();
    }
    lines.next();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    return lines.next().unwrap_or("").to_string();
}

/// What:
/// `pub fn recover(root, is_alive, judge) -> Vec<String>` finishes the journals of processes that no longer exist.
/// Why:
/// See `recover_with`;
/// this form has no hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recover({ root, isAlive, judge }): string[]
/// ```
pub fn recover(root: &Path, is_alive: &dyn Fn(u32, &str) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>) -> Vec<String> {
    return recover_with(root, is_alive, judge, &no_hook);
}

/// What:
/// `fn claim_journal(directory: &Path, original: &str) -> Option<PathBuf>` claims a journal by renaming its directory to
/// `<original>.claim.<this pid>.<time>`;
/// only one of several racing runs succeeds.
/// Why:
/// Two recovering runs must never act on the same journal,
/// and a live transaction must never be recovered by a stale reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claimJournal(directory: string, original: string): string | undefined  // renameSync, undefined when it fails
/// ```
pub(crate) fn claim_journal(directory: &Path, original: &str) -> Option<PathBuf> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Typed binding: `parent: &Path`.
    let parent: &Path = directory.parent()?;
    // Typed binding: `claimed: PathBuf`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let claimed: PathBuf = parent.join(format!("{original}.claim.{}.{}", std::process::id(), crate::file_owner::clock_ns()));
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&claimed`: lends the value without copying or giving it away.
    return match fs::rename(directory, &claimed) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        Ok(()) => Some(claimed),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Err(_) => None,
    };
}

/// What:
/// `fn release_claim(directory: &Path, journals: &Path, original: &str)` gives a claimed journal back its own name.
/// Why:
/// A recovery that did not finish keeps the journal,
/// and the next run must be able to find and retry it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function releaseClaim(directory: string, journals: string, original: string): void  // renameSync, errors ignored
/// ```
pub(crate) fn release_claim(directory: &Path, journals: &Path, original: &str) {
    // `let _ =`: discards the result on purpose.
    let _ = fs::rename(directory, journals.join(original));
}

/// What:
/// `pub fn recover_with(root, is_alive, judge, between) -> Vec<String>` finishes the journals of processes that no longer exist.
/// Why:
/// For each such journal,
/// after claiming it:
/// a live lock equal to its snapshot needs nothing;
/// a live lock the `judge` finds acceptable (no policy findings for the versions it added) is kept;
/// otherwise the live lock is saved beside the lockfile (`Cargo.lock.refused`,
/// the first free `.refused.N`) and the snapshot goes back;
/// the manifests of a root whose lockfile went back are put back too.
/// The journal is deleted (and the deletion synchronized) only when every root was settled;
/// any unreadable part or failed step keeps it,
/// and the claim is released so the next run can retry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverWith({ root, isAlive, judge, between }): string[]
/// ```
pub fn recover_with(root: &Path, is_alive: &dyn Fn(u32, &str) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path)) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    // Typed binding: `journals: PathBuf`.
    let journals: PathBuf = root.join(JOURNAL_DIR);
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&journals`: lends the value without copying or giving it away.
    let entries = match fs::read_dir(&journals) {
        Ok(found) => found,
        Err(_) => return messages,
    };
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // Typed binding: `own_identity: String`.
    let own_identity: String = process_identity(std::process::id()).unwrap_or_default();
    for listed in entries.flatten() {
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        // Typed binding: `name: String`.
        let name: String = listed.file_name().to_string_lossy().to_string();
        // `<pid>` is a journal; `<pid>.claim.<claimer>.<time>` is one that a recovery claimed and did not finish.
        let (original, claimer): (String, Option<u32>) = match name.split_once(".claim.") {
            // `Some`: the present or the absent value of an `Option`.
            // Closure `|found|`: an anonymous function, like a TS arrow function.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
            // Adapter `.and_then(`: transforms or selects items with a closure.
            Some((first, rest)) => (first.to_string(), rest.split('.').next().and_then(|found| { return found.parse().ok(); })),
            // `None`: the present or the absent value of an `Option`.
            // `.clone()`: an independent owned copy.
            None => (name.clone(), None),
        };
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `pid: u32`.
        let pid: u32 = match original.parse() {
            Ok(found) => found,
            Err(_) => continue,
        };
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(other) = claimer
            && process_exists(other, "")
            && other != std::process::id()
        {
            continue;
        }
        // Borrow `&listed.path`: lends the value without copying or giving it away.
        // Typed binding: `identity: String`.
        let identity: String = read_identity(&listed.path());
        // Typed binding: `own: bool`.
        let own: bool = pid == std::process::id() && (identity.is_empty() || identity == own_identity);
        // Borrow `&identity`: lends the value without copying or giving it away.
        if own || is_alive(pid, &identity) {
            continue;
        }
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&listed.path`: lends the value without copying or giving it away.
        // Typed binding: `directory: PathBuf`.
        let directory: PathBuf = match claim_journal(&listed.path(), &original) {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found,
            // Another run claimed it first.
            None => continue,
        };
        let before: Vec<LockSnapshot> = match read_journal(&directory) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: the journal of a killed run could not be read ({error}); it was kept", directory.display()));
                // Borrow `&directory`: lends the value without copying or giving it away.
                release_claim(&directory, &journals, &original);
                continue;
            }
        };
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&directory`: lends the value without copying or giving it away.
        // Typed binding: `manifests: Vec<JournaledManifest>`.
        let manifests: Vec<JournaledManifest> = match read_manifest_index(&directory) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) => {
                // Macro `format!`: builds or prints text (or a list) from a template.
                messages.push(format!("{}: the manifest record of a killed run could not be read ({error}); its journal was kept", directory.display()));
                // Borrow `&directory`: lends the value without copying or giving it away.
                release_claim(&directory, &journals, &original);
                continue;
            }
        };
        let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
        let live: Vec<LockSnapshot> = match read_locks(root, &dirs) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: a lockfile of a killed run could not be read ({error}); its journal was kept", directory.display()));
                // Borrow `&directory`: lends the value without copying or giving it away.
                release_claim(&directory, &journals, &original);
                continue;
            }
        };
        let mut settled: bool = true;
        // `let mut`: a binding that may change.
        // Typed binding: `decisions: Vec<(String, RootDecision)>`.
        // Constructor `Vec::new()`: an empty owned collection or a boxed value.
        let mut decisions: Vec<(String, RootDecision)> = Vec::new();
        for (old, now) in before.iter().zip(live.iter()) {
            if old.source == now.source {
                // `.clone()`: an independent owned copy.
                decisions.push((old.crate_dir.clone(), RootDecision::Restored));
                continue;
            }
            let problems: Vec<String> = judge(std::slice::from_ref(old), std::slice::from_ref(now));
            let path: PathBuf = root.join(&old.crate_dir).join(LOCKFILE_NAME);
            if problems.is_empty() {
                messages.push(format!("{}/{LOCKFILE_NAME} was left by a run that was killed; its new versions pass the policy, so it was kept", old.crate_dir));
                // `.clone()`: an independent owned copy.
                decisions.push((old.crate_dir.clone(), RootDecision::Kept));
                continue;
            }
            // Typed binding: `keep: KeepAs`.
            let keep: KeepAs = KeepAs::next_to_target(REFUSED_SUFFIX);
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&path`: lends the value without copying or giving it away.
            match replace_keeping_with_hook(&path, &now.source, &old.source, &keep, between) {
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
                    // `.clone()`: an independent owned copy.
                    decisions.push((old.crate_dir.clone(), RootDecision::Restored));
                }
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(ReplaceOutcome::LeftAlone) => {
                    // Macro `format!`: builds or prints text (or a list) from a template.
                    messages.push(format!(
                        "{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored",
                        old.crate_dir
                    ));
                    // `.clone()`: an independent owned copy.
                    decisions.push((old.crate_dir.clone(), RootDecision::LeftAlone));
                }
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                Ok(ReplaceOutcome::Displaced(kept)) => {
                    messages.push(format!(
                        "{}/{LOCKFILE_NAME} was left by a run that was killed; the earlier lockfile was put back, but the file it displaced was not the one that run wrote; that file is kept as {}",
                        old.crate_dir,
                        kept.display()
                    ));
                    decisions.push((old.crate_dir.clone(), RootDecision::Restored));
                }
                Ok(ReplaceOutcome::Undone(kept)) => {
                    messages.push(format!(
                        "{}/{LOCKFILE_NAME} was changed twice by other processes while a killed run was being recovered; it was left as the last one wrote it, and the snapshot copy is kept as {}",
                        old.crate_dir,
                        kept.display()
                    ));
                    decisions.push((old.crate_dir.clone(), RootDecision::LeftAlone));
                }
                Err(error) => {
                    settled = false;
                    messages.push(format!("{}/{LOCKFILE_NAME}: could not recover a killed run: {error}; its journal was kept", old.crate_dir));
                }
            }
        }
        // Borrow `&manifests`: lends the value without copying or giving it away.
        if !recover_manifests(&manifests, &decisions, between, &mut messages) {
            settled = false;
        }
        if settled {
            let _ = fs::remove_dir_all(&directory);
            // Borrow `&journals`: lends the value without copying or giving it away.
            // `let _ =`: discards the result on purpose.
            let _ = sync_directory(&journals);
        } else {
            // Release the claim so the next run retries.
            release_claim(&directory, &journals, &original);
        }
    }
    return messages;
}
