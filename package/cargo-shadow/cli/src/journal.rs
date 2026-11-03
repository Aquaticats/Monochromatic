//! What:
//! Compare-before-replace for the lockfile transaction,
//! and a journal that lets the next run finish what a killed run left.
//! Why:
//! The install and the update snapshot every lockfile,
//! let cargo rewrite them,
//! check the new versions against the policy,
//! and
//! put the snapshot back on a refusal.
//! Owner rules:
//! "never discard user data" and "use as few locks as possible".
//! So there is no
//! lock:
//! a restore replaces a lockfile only if it still holds what this run produced (anything else was written by someone
//! else meanwhile and is left alone with a warning),
//! and the replacement is a write to a sibling file and an atomic rename.
//! A process killed between cargo's write and the restore leaves a journal directory with the snapshots;
//! the next run reads it,
//! checks the live lock against the policy,
//! keeps it if it passes,
//! and otherwise saves it beside the lockfile as
//! `Cargo.lock.refused` before putting the snapshot back,
//! so no lockfile content is ever thrown away.
//! `cargo -Zunstable-options update --lockfile-path`,
//! which would resolve into a private file and publish it with one rename,
//! does not exist on the pinned nightly (checked 2026-11-03:
//! `cargo update --help` and `cargo check --help` list no such option).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal.ts
//! export function restoreIfUnchanged({ root, snapshots, produced }): string[]  // warnings
//! export function begin / finish / recover
//! ```

/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `atomic_write`,
/// `restore_if_unchanged`,
/// `begin`,
/// `finish`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::io;` imports `io` from the standard library so this file can name it unqualified.
/// Why:
/// `io` is used by `atomic_write`,
/// `restore_if_unchanged`,
/// `begin`,
/// `read_journal`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { io } from 'std';
/// ```
use std::cell::RefCell;
use std::io;
use std::io::Write;
/// What:
/// `use std::path::{Path, PathBuf};` imports `Path`,
/// `PathBuf` from the standard library so this file can name them unqualified.
/// Why:
/// `Path` is used by `sibling_temp`,
/// `atomic_write`,
/// `restore_if_unchanged`,
/// `journal_path`,
/// and others;
/// `PathBuf` is used by `sibling_temp`,
/// `atomic_write`,
/// `restore_if_unchanged`,
/// `journal_path`,
/// and others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path, PathBuf } from 'std/path';
/// ```
use std::path::{Path, PathBuf};
/// Imports `rustix`'s safe wrapper of `renameat2`: the flags, and the current directory as the base of relative names.
use rustix::fs::{CWD, RenameFlags, renameat_with};

/// What:
/// `use crate::constants::LOCKFILE_NAME;` imports `LOCKFILE_NAME` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LOCKFILE_NAME` is used by `restore_if_unchanged`,
/// `recover`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::LOCKFILE_NAME;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports `LockSnapshot`,
/// `read_locks` from another module of this crate so this file can name them unqualified.
/// Why:
/// `LockSnapshot` is used by `restore_if_unchanged`,
/// `begin`,
/// `read_journal`,
/// `recover`;
/// `read_locks` is used by `recover`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, read_locks } from 'crate/crates';
/// ```
use crate::crates::{LockSnapshot, read_locks};
/// Imports `ManifestEdit`, one manifest a `--breaking` run may rewrite.
use crate::manifest_edit::ManifestEdit;

/// Where journals live,
/// relative to the repository root.
pub const JOURNAL_DIR: &str = ".cache/cargo-shadow/journal";

/// The extension appended to a lockfile name when a refused live lock is saved before the snapshot goes back.
pub const REFUSED_SUFFIX: &str = ".refused";

/// The file in a journal directory that names the crate roots,
/// one per line,
/// in the order of the snapshot files.
const INDEX_FILE: &str = "index.txt";

/// The path of a sibling file used for the write half of an atomic replace.
fn sibling_temp(target: &Path) -> PathBuf {
    let mut name: std::ffi::OsString = target.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    name.push(format!(".cargo-shadow-{}.tmp", std::process::id()));
    return target.with_file_name(name);
}

/// Writes `text` to `target` by writing a sibling file and renaming it over the target.
pub fn atomic_write(target: &Path, text: &str) -> io::Result<()> {
    let temp: PathBuf = sibling_temp(target);
    fs::write(&temp, text)?;
    let renamed: io::Result<()> = fs::rename(&temp, target);
    if renamed.is_err() {
        let _ = fs::remove_file(&temp);
    }
    return renamed;
}

/// What a compare-and-replace did to one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplaceOutcome {
    /// The file held what was expected and now holds the new text.
    Replaced,
    /// The file held something else, or held it only for a moment and was put back:
    /// another writer's file is where it was.
    LeftAlone,
    /// Another writer published twice in the gap, so a file was displaced and could not be put back;
    /// it was kept under this name.
    Preserved(PathBuf),
}

/// What:
/// `fn exchange(first, second) -> io::Result<()>` swaps two paths in one step (`renameat2` with `RENAME_EXCHANGE`,
/// through `rustix`'s safe wrapper).
/// Why:
/// After the call `first` names what `second` named and the other way round, with no moment at which either path is missing,
/// so the file that was displaced can be read and, if it was not the expected one, swapped back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // no Node API; the kernel call is renameat2(RENAME_EXCHANGE)
/// function exchange(first: string, second: string): void
/// ```
fn exchange(first: &Path, second: &Path) -> io::Result<()> {
    return renameat_with(CWD, first, CWD, second, RenameFlags::EXCHANGE).map_err(io::Error::from);
}

/// What:
/// `fn preserve(displaced, target) -> io::Result<PathBuf>` moves a file to the first free `<name>.displaced`,
/// `<name>.displaced.1`, ... next to the target, without replacing anything.
/// Why:
/// A file that could not be put back must not be deleted; a hard link fails when the name exists, so two runs never share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function preserve(displaced: string, target: string): string  // linkSync(displaced, name) per candidate, then unlinkSync(displaced)
/// ```
fn preserve(displaced: &Path, target: &Path) -> io::Result<PathBuf> {
    let base: String = target.file_name().map(|name| { return name.to_string_lossy().to_string(); }).unwrap_or_default();
    let mut number: u32 = 0;
    loop {
        let name: String = if number == 0 { format!("{base}.displaced") } else { format!("{base}.displaced.{number}") };
        let candidate: PathBuf = target.with_file_name(name);
        match fs::hard_link(displaced, &candidate) {
            Ok(()) => {
                fs::remove_file(displaced)?;
                return Ok(candidate);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => number += 1,
            Err(error) => return Err(error),
        }
    }
}

/// What:
/// `pub fn replace_if_holds(path, expected, new_text, between) -> io::Result<ReplaceOutcome>` puts `new_text` in `path` only if
/// `path` holds `expected`, comparing and replacing in one atomic step.
/// Why:
/// Review finding and delegated pick (2026-11-03): reading a file and then renaming over it leaves a gap in which another writer
/// can publish a lockfile that the rename then destroys. The new text is written to a sibling file and exchanged with `path`
/// atomically; the file that came out is read, and when it is not the expected one the exchange is undone, so no writer's
/// file is ever lost and no lock is taken. A filesystem that cannot exchange falls back to a second comparison right before
/// an ordinary rename. `between` runs after the first comparison, so a test can publish a file exactly in the gap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHolds(path: string, expected: string, newText: string, between: (path: string) => void): ReplaceOutcome
/// ```
pub fn replace_if_holds(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    return replace_if_holds_with_hooks(path, expected, new_text, between, &|_path| {});
}

/// What:
/// `pub fn replace_if_holds_with_hooks(path, expected, new_text, between, after_exchange)` is `replace_if_holds` with a second
/// hook that runs right after the exchange, before the displaced file is read.
/// Why:
/// The second gap is where a writer publishes a file between the swap and the swap back; the hook lets a test publish one there,
/// to show that the file displaced twice is kept and not deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function replaceIfHoldsWithHooks(path, expected, newText, between, afterExchange): ReplaceOutcome
/// ```
pub fn replace_if_holds_with_hooks(path: &Path, expected: &str, new_text: &str, between: &dyn Fn(&Path), after_exchange: &dyn Fn(&Path)) -> io::Result<ReplaceOutcome> {
    let first: String = fs::read_to_string(path).unwrap_or_default();
    if first != expected {
        return Ok(ReplaceOutcome::LeftAlone);
    }
    between(path);
    let temp: PathBuf = sibling_temp(path);
    fs::write(&temp, new_text)?;
    if exchange(&temp, path).is_err() {
        // No exchange here (the path vanished, or the filesystem cannot swap): compare again, then an ordinary rename.
        let again: String = fs::read_to_string(path).unwrap_or_default();
        if again != expected {
            let _ = fs::remove_file(&temp);
            return Ok(ReplaceOutcome::LeftAlone);
        }
        fs::rename(&temp, path)?;
        return Ok(ReplaceOutcome::Replaced);
    }
    after_exchange(path);
    // The temporary name now holds the file that was at `path`.
    let displaced: String = fs::read_to_string(&temp).unwrap_or_default();
    if displaced == expected {
        fs::remove_file(&temp)?;
        return Ok(ReplaceOutcome::Replaced);
    }
    // Another writer published in the gap: swap back, so their file is where it was.
    if exchange(&temp, path).is_err() {
        return Ok(ReplaceOutcome::Preserved(preserve(&temp, path)?));
    }
    let after_undo: String = fs::read_to_string(&temp).unwrap_or_default();
    if after_undo == new_text {
        fs::remove_file(&temp)?;
        return Ok(ReplaceOutcome::LeftAlone);
    }
    // Yet another writer published between the swap and the swap back; their file now sits at the temporary name.
    return Ok(ReplaceOutcome::Preserved(preserve(&temp, path)?));
}

/// What:
/// `fn snapshot_of(snapshots, crate_dir) -> Option<&LockSnapshot>` finds the snapshot of one crate root,
/// or `None`.
/// Why:
/// The restore needs the lockfile this run produced for a root,
/// to tell it from one a concurrent writer left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshotOf = (snapshots: Snapshot[], crateDir: string) => snapshots.find((s) => s.crateDir === crateDir);
/// ```
fn snapshot_of<'a>(snapshots: &'a [LockSnapshot], crate_dir: &str) -> Option<&'a LockSnapshot> {
    for candidate in snapshots {
        if candidate.crate_dir == crate_dir {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
/// `pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>>`
/// puts each snapshot back only when the live lockfile still holds what this run produced.
/// Why:
/// A live lockfile that differs from `produced` was changed by another process after cargo finished;
/// replacing it would discard
/// that change,
/// so it is left alone and named in a warning.
/// A root that `produced` does not list was never written by this run, so unknown ownership never authorizes a restore.
/// One that already equals the snapshot needs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchanged({ root, snapshots, produced }): string[]
/// ```
pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>> {
    return restore_if_unchanged_with(root, snapshots, produced, &|_path| {});
}

/// What:
/// `pub fn restore_if_unchanged_with(root, snapshots, produced, between)` is `restore_if_unchanged` with a hook that runs
/// after a lockfile was compared and before it is replaced.
/// Why:
/// The gap between the comparison and the replacement is where another writer can publish a lockfile;
/// the hook lets a test publish one exactly there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchangedWith({ root, snapshots, produced, between }): string[]
/// ```
pub fn restore_if_unchanged_with(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot], between: &dyn Fn(&Path)) -> io::Result<Vec<String>> {
    let mut warnings: Vec<String> = Vec::new();
    for snapshot in snapshots {
        let path: PathBuf = root.join(&snapshot.crate_dir).join(LOCKFILE_NAME);
        let live: String = fs::read_to_string(&path).unwrap_or_default();
        if live == snapshot.source {
            continue;
        }
        // Review finding (unknown ownership): only a lockfile this run is known to have written may be replaced. A root the run
        // never recorded as produced, or one that holds something else than was recorded, belongs to somebody else.
        let expected: Option<&LockSnapshot> = snapshot_of(produced, &snapshot.crate_dir);
        match expected {
            Some(found) if found.source == live => {}
            Some(_) => {
                warnings.push(format!(
                    "{}/{LOCKFILE_NAME} was changed by another process while the step ran; it was left as that process wrote it, not restored",
                    snapshot.crate_dir
                ));
                continue;
            }
            None => {
                warnings.push(format!(
                    "{}/{LOCKFILE_NAME} differs from the snapshot, but this run did not write it, so another process did; it was left as it is, not restored",
                    snapshot.crate_dir
                ));
                continue;
            }
        }
        match replace_if_holds(&path, &live, &snapshot.source, between)? {
            ReplaceOutcome::Replaced => {}
            ReplaceOutcome::LeftAlone => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed by another process while the step ran; it was left as that process wrote it, not restored",
                snapshot.crate_dir
            )),
            ReplaceOutcome::Preserved(copy) => warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed twice by other processes while the step ran; the file displaced is kept as {}",
                snapshot.crate_dir,
                copy.display()
            )),
        }
    }
    return Ok(warnings);
}

/// What:
/// `pub fn note_produced(root, crate_dir, produced)` records the lockfile of one crate root as this run's own output,
/// right after a cargo call of this run succeeded there.
/// Why:
/// The rollback replaces a lockfile only when it still holds what this run produced;
/// recording each write as it happens means a failure halfway never has to guess the content from the file system,
/// where another writer's content would be indistinguishable from the run's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function noteProduced({ root, crateDir, produced }: { root: string; crateDir: string; produced: LockSnapshot[] }): void
/// ```
pub fn note_produced(root: &Path, crate_dir: &str, produced: &RefCell<Vec<LockSnapshot>>) {
    // A lockfile that cannot be read right now is not recorded; an unrecorded root is never restored.
    let source: String = match fs::read_to_string(root.join(crate_dir).join(LOCKFILE_NAME)) {
        Ok(found) => found,
        Err(_) => return,
    };
    let mut list = produced.borrow_mut();
    list.retain(|known| { return known.crate_dir != crate_dir; });
    list.push(LockSnapshot { crate_dir: crate_dir.to_string(), source });
}

/// What:
/// `fn save_aside(target, text) -> io::Result<PathBuf>` writes `text` to a new file next to `target`
/// named `target`, then `target.1`, `target.2`, ... , the first name that does not exist yet, and returns the path.
/// Why:
/// A refused lockfile is evidence the user may need;
/// a later refusal must never replace an earlier saved copy,
/// and `create_new` makes the claim of a name atomic, so two runs never pick the same one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function saveAside(target: string, text: string): string  // writeFileSync(name, text, { flag: 'wx' }) per candidate name
/// ```
fn save_aside(target: &Path, text: &str) -> io::Result<PathBuf> {
    let base: String = target.file_name().map(|name| { return name.to_string_lossy().to_string(); }).unwrap_or_default();
    let mut number: u32 = 0;
    loop {
        let name: String = if number == 0 { base.clone() } else { format!("{base}.{number}") };
        let candidate: PathBuf = target.with_file_name(name);
        // `create_new(true)` fails with `AlreadyExists` instead of truncating, the exclusive-create flag `wx` of Node.
        let opened = fs::OpenOptions::new().write(true).create_new(true).open(&candidate);
        match opened {
            Ok(mut file) => {
                file.write_all(text.as_bytes())?;
                return Ok(candidate);
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => number += 1,
            Err(error) => return Err(error),
        }
    }
}

/// Where this process's journal lives.
fn journal_path(root: &Path, pid: u32) -> PathBuf {
    return root.join(JOURNAL_DIR).join(pid.to_string());
}

/// The file in a journal directory that names the manifests a `--breaking` run may rewrite,
/// one absolute path per line,
/// in the order of the `m<position>.toml` files that hold their text before the run.
const MANIFEST_INDEX_FILE: &str = "manifests.txt";

/// What:
/// `pub fn add_manifests(root, edits) -> io::Result<()>` records the manifests a `--breaking` run may rewrite in this process's
/// journal, with their text before the run.
/// Why:
/// Delegated pick (2026-11-03): cargo's breaking update rewrites manifests as well as the lockfile, so a killed run must be
/// able to put them back too. A path with a line feed cannot be listed one per line, so it stops the run before cargo runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addManifests(root: string, edits: ManifestEdit[]): void  // throws when the journal cannot be written
/// ```
pub fn add_manifests(root: &Path, edits: &[ManifestEdit]) -> io::Result<()> {
    let directory: PathBuf = journal_path(root, std::process::id());
    let mut index: String = String::new();
    for (position, edit) in edits.iter().enumerate() {
        let path_text: String = edit.path.to_string_lossy().to_string();
        if path_text.contains('\n') {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the manifest path {path_text:?} contains a line feed, which the journal cannot list")));
        }
        fs::write(directory.join(format!("m{position}.toml")), &edit.before)?;
        index.push_str(&path_text);
        index.push('\n');
    }
    return atomic_write(&directory.join(MANIFEST_INDEX_FILE), &index);
}

/// What:
/// `pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()>` writes the journal of this process:
/// every snapshot
/// and the list of crate roots.
/// Why:
/// If this process is killed before `finish`,
/// the next run finds the snapshots here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[]): void
/// ```
pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()> {
    let directory: PathBuf = journal_path(root, std::process::id());
    fs::create_dir_all(&directory)?;
    let mut index: String = String::new();
    for (position, snapshot) in snapshots.iter().enumerate() {
        fs::write(directory.join(format!("{position}.lock")), &snapshot.source)?;
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    return atomic_write(&directory.join(INDEX_FILE), &index);
}

/// Removes this process's journal;
/// a missing journal is fine.
pub fn finish(root: &Path) {
    let _ = fs::remove_dir_all(journal_path(root, std::process::id()));
}

/// Reads the snapshots recorded in a journal directory.
fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>> {
    let index: String = fs::read_to_string(directory.join(INDEX_FILE))?;
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    for (position, crate_dir) in index.lines().enumerate() {
        let source: String = fs::read_to_string(directory.join(format!("{position}.lock")))?;
        snapshots.push(LockSnapshot { crate_dir: crate_dir.to_string(), source });
    }
    return Ok(snapshots);
}

/// Whether a process with this id exists.
/// It reads Linux `/proc`, and the crate refuses to compile on other systems (`lib.rs`), so a missing `/proc` cannot make a
/// running process look dead;
/// a reused id reads as alive, which only delays a recovery.
pub fn process_exists(pid: u32) -> bool {
    return Path::new("/proc").join(pid.to_string()).exists();
}

/// What:
/// `fn recover_manifests(directory, between, messages) -> bool` puts back the manifests a killed `--breaking` run journaled,
/// saving the text it replaces as `<name>.refused` (or the first free `<name>.refused.N`), and says whether all were settled.
/// Why:
/// A journaled manifest that differs from its text before the run was rewritten by the killed run (or edited since, which the
/// saved copy keeps), and a manifest that cannot be put back keeps the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverManifests(directory: string, between: (path: string) => void, messages: string[]): boolean
/// ```
fn recover_manifests(directory: &Path, between: &dyn Fn(&Path), messages: &mut Vec<String>) -> bool {
    let index: String = match fs::read_to_string(directory.join(MANIFEST_INDEX_FILE)) {
        Ok(found) => found,
        Err(_) => return true,
    };
    let mut all_settled: bool = true;
    for (position, path_text) in index.lines().enumerate() {
        let path: PathBuf = PathBuf::from(path_text);
        let before: String = match fs::read_to_string(directory.join(format!("m{position}.toml"))) {
            Ok(found) => found,
            Err(error) => {
                all_settled = false;
                messages.push(format!("{path_text}: the journaled text of a manifest could not be read ({error}); its journal was kept"));
                continue;
            }
        };
        let live: String = fs::read_to_string(&path).unwrap_or_default();
        if live == before {
            continue;
        }
        let saved: io::Result<PathBuf> = save_aside(&path.with_file_name(format!("{}{REFUSED_SUFFIX}", path.file_name().map(|name| { return name.to_string_lossy().to_string(); }).unwrap_or_default())), &live);
        let restored: io::Result<(PathBuf, ReplaceOutcome)> = match saved {
            Ok(copy) => replace_if_holds(&path, &live, &before, between).map(|outcome| { return (copy, outcome); }),
            Err(error) => Err(error),
        };
        match restored {
            Ok((copy, ReplaceOutcome::Replaced)) => messages.push(format!(
                "{path_text} was rewritten by a run that was killed and its lockfile was not kept; the earlier manifest was put back and the rewritten text is kept as {}",
                copy.display()
            )),
            Ok((_, ReplaceOutcome::LeftAlone)) => messages.push(format!("{path_text} was changed by another process while a killed run was being recovered; it was left as that process wrote it")),
            Ok((_, ReplaceOutcome::Preserved(kept))) => messages.push(format!("{path_text} was changed twice by other processes while a killed run was being recovered; the file displaced is kept as {}", kept.display())),
            Err(error) => {
                all_settled = false;
                messages.push(format!("{path_text}: could not recover a killed run: {error}; its journal was kept"));
            }
        }
    }
    return all_settled;
}

/// What:
/// `pub fn recover(root, is_alive, judge) -> Vec<String>` finishes the journals of processes that no longer exist.
/// Why:
/// For each such journal:
/// a live lock equal to its snapshot needs nothing;
/// a live lock the `judge` finds acceptable (no policy
/// findings for the versions it added) is kept;
/// otherwise the live lock is saved as `Cargo.lock.refused` (or the first free `Cargo.lock.refused.N`, so an earlier copy
/// is never replaced) and the snapshot goes back by atomic rename,
/// after the live lock was read again and found to be still the judged one.
/// The journal is removed only when every root it lists was settled;
/// an unreadable journal or lockfile and a failed step keep it, because it holds the only copy of the original.
/// A message says what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recover({ root, isAlive, judge }): string[]
/// ```
pub fn recover(root: &Path, is_alive: &dyn Fn(u32) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>) -> Vec<String> {
    return recover_with(root, is_alive, judge, &|_path| {});
}

/// What:
/// `pub fn recover_with(root, is_alive, judge, between) -> Vec<String>` is `recover` with a hook that runs after the last
/// comparison of a lockfile and before it is replaced.
/// Why:
/// The same gap as in `restore_if_unchanged_with`, for the recovery of a killed run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverWith({ root, isAlive, judge, between }): string[]
/// ```
pub fn recover_with(root: &Path, is_alive: &dyn Fn(u32) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path)) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let entries = match fs::read_dir(root.join(JOURNAL_DIR)) {
        Ok(found) => found,
        Err(_) => return messages,
    };
    for listed in entries.flatten() {
        let pid: u32 = match listed.file_name().to_string_lossy().parse() {
            Ok(found) => found,
            Err(_) => continue,
        };
        if pid == std::process::id() || is_alive(pid) {
            continue;
        }
        let directory: PathBuf = listed.path();
        // Review finding (recovery destroys the remaining copies): the journal holds the only copy of the original lockfiles,
        // so it is deleted only when every root it lists was settled. Any unreadable part or failed step keeps it.
        let before: Vec<LockSnapshot> = match read_journal(&directory) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: the journal of a killed run could not be read ({error}); it was kept", directory.display()));
                continue;
            }
        };
        let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
        let live: Vec<LockSnapshot> = match read_locks(root, &dirs) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: a lockfile of a killed run could not be read ({error}); its journal was kept", directory.display()));
                continue;
            }
        };
        let mut settled: bool = true;
        // Whether a lockfile this journal lists was kept because its new versions pass the policy: the manifest edits that go
        // with it are kept too.
        let mut kept_a_lock: bool = false;
        for (old, now) in before.iter().zip(live.iter()) {
            if old.source == now.source {
                continue;
            }
            let problems: Vec<String> = judge(std::slice::from_ref(old), std::slice::from_ref(now));
            let path: PathBuf = root.join(&old.crate_dir).join(LOCKFILE_NAME);
            if problems.is_empty() {
                messages.push(format!("{}/{LOCKFILE_NAME} was left by a run that was killed; its new versions pass the policy, so it was kept", old.crate_dir));
                kept_a_lock = true;
                continue;
            }
            // Review finding (compare-before-replace): the judgement above can take a long time, so the file is read again
            // right before it is replaced, and a lockfile that is no longer the judged one is another writer's and is left.
            let latest: String = fs::read_to_string(&path).unwrap_or_default();
            if latest != now.source {
                messages.push(format!(
                    "{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored",
                    old.crate_dir
                ));
                continue;
            }
            let saved: io::Result<PathBuf> = save_aside(&path.with_file_name(format!("{LOCKFILE_NAME}{REFUSED_SUFFIX}")), &now.source);
            let restored: io::Result<(PathBuf, ReplaceOutcome)> = match saved {
                Ok(copy) => replace_if_holds(&path, &now.source, &old.source, between).map(|outcome| { return (copy, outcome); }),
                Err(error) => Err(error),
            };
            match restored {
                Ok((copy, ReplaceOutcome::Replaced)) => messages.push(format!(
                    "{}/{LOCKFILE_NAME} was left by a run that was killed and holds versions the policy refuses ({}); it was saved as {} and the earlier lockfile was put back",
                    old.crate_dir,
                    problems.join("; "),
                    copy.file_name().map(|name| { return name.to_string_lossy().to_string(); }).unwrap_or_default()
                )),
                Ok((copy, ReplaceOutcome::LeftAlone)) => messages.push(format!(
                    "{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored (the refused text is kept as {})",
                    old.crate_dir,
                    copy.display()
                )),
                Ok((_, ReplaceOutcome::Preserved(kept))) => messages.push(format!(
                    "{}/{LOCKFILE_NAME} was changed twice by other processes while a killed run was being recovered; the file displaced is kept as {}",
                    old.crate_dir,
                    kept.display()
                )),
                Err(error) => {
                    settled = false;
                    messages.push(format!("{}/{LOCKFILE_NAME}: could not recover a killed run: {error}; its journal was kept", old.crate_dir));
                }
            }
        }
        // Delegated pick (2026-11-03): the manifests of a killed `--breaking` run go back unless its lockfile was kept.
        if !kept_a_lock && !recover_manifests(&directory, between, &mut messages) {
            settled = false;
        }
        if settled {
            let _ = fs::remove_dir_all(&directory);
        }
    }
    return messages;
}
