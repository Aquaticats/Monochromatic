//! What:
//! The lockfile transaction's snapshots and its journal:
//! a durable record, written before cargo is started,
//! that lets the next run finish what a killed run left,
//! and the restore that puts snapshots back without destroying anyone's file.
//! Why:
//! Owner rules: "never discard user data" and "use as few locks as possible".
//! So there is no lock around the lockfile.
//! Ownership of a file is decided by the evidence in `file_owner.rs` (the change time of the file against the window of the child's run),
//! never by reading the file after the child returned;
//! a restore replaces a lockfile only when it still holds what this run's child wrote,
//! with the replacement protocol of `journal_replace.rs` (every displaced file keeps a name),
//! and anything else is left alone with a warning.
//! The journal under `.cache/cargo-shadow/journal/<pid>/` is written and synchronized before cargo mutates anything:
//! an owner record (process id and start time, so a reused process id is not mistaken for the killed run),
//! the snapshots, and a complete, versioned index.
//! A recovering run first claims a journal by renaming its directory (only one run wins),
//! decides root by root whether the lockfile the killed run left passes the policy,
//! keeps it if it does and otherwise saves it beside the lockfile as `Cargo.lock.refused` while putting the snapshot back,
//! restores the manifests of the roots whose lockfile was not kept,
//! synchronizes, and only then deletes the journal.
//! `cargo -Zunstable-options update --lockfile-path`,
//! which would resolve into a private file and publish it with one rename,
//! does not exist on the pinned nightly (checked 2026-11-03:
//! `cargo update --help` and `cargo check --help` list no such option).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal.ts
//! export function begin(root: string, snapshots: LockSnapshot[]): void
//! export function restoreIfUnchanged(root: string, snapshots: LockSnapshot[], produced: LockSnapshot[]): string[]
//! export function recover(root: string, isAlive: (pid: number, identity: string) => boolean, judge: Judge): string[]
//! ```

/// What:
/// `use std::cell::RefCell;` imports `RefCell`, a list or value that can be changed through a shared reference.
/// Why:
/// The record of the lockfiles a run wrote grows through one while the transaction body holds a shared reference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS objects are mutable through any reference; no import is needed.
/// ```
use std::cell::RefCell;
/// What:
/// `use std::fs;` imports the standard file-system module.
/// Why:
/// The journal is made of ordinary files and directories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, writeFileSync, mkdirSync, renameSync, rmSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::io::{self, Write};` imports the error module and the trait that adds `write_all` to an open file.
/// Why:
/// Journal files are written through a descriptor and synchronized before they count as written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // errors are thrown in Node
/// ```
use std::io::{self, Write};
/// What:
/// `use std::os::unix::ffi::{OsStrExt, OsStringExt};` imports the traits that turn an operating-system string into bytes and back.
/// Why:
/// A manifest path is stored as the exact bytes of the path, so a path that is not UTF-8 survives a recovery.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Buffer.from(path) / Buffer.toString()
/// ```
use std::os::unix::ffi::{OsStrExt, OsStringExt};
/// What:
/// `use std::path::{Path, PathBuf};` imports the borrowed and owned path types.
/// Why:
/// Paths are lent to the functions and the journal's own paths are returned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `pub use crate::journal_replace::{...};` imports and re-exports the replacement protocol,
/// so callers keep writing `crate::journal::replace_if_holds`.
/// Why:
/// The file was split to stay under the line limit; the callers and the tests are unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { KeepAs, ReplaceOutcome, replaceIfHolds } from './journal-replace.ts';
/// ```
pub use crate::journal_replace::{KeepAs, ReplaceOutcome, file_name_text, no_hook, replace_if_holds, replace_if_holds_with_hooks, replace_keeping, replace_keeping_with_hook, replace_with_swap, sync_directory};

/// What:
/// `use crate::constants::LOCKFILE_NAME;` imports the lockfile's file name.
/// Why:
/// Lockfile paths are built from a crate root and this name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::LOCKFILE_NAME;
/// What:
/// `use crate::crates::{LockSnapshot, read_locks};` imports the lockfile snapshot type and the reader of the lockfiles.
/// Why:
/// Snapshots are what the journal stores; the live lockfiles are read for the recovery decision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LockSnapshot, readLocks } from 'crate/crates';
/// ```
use crate::crates::{LockSnapshot, read_locks};
/// What:
/// `use crate::file_owner::{Claim, Window, claim_output, observe, timed};` imports the ownership evidence.
/// Why:
/// Every lockfile written by a child is attributed with it before it is recorded as this run's output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Claim, Window, claimOutput, observe, timed } from 'crate/file_owner';
/// ```
use crate::file_owner::{Claim, Window, claim_output, observe, timed};
/// What:
/// `use crate::manifest_edit::ManifestEdit;` imports one manifest a `--breaking` run may rewrite.
/// Why:
/// The journal records the manifests with their root and their text before the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ManifestEdit } from 'crate/manifest_edit';
/// ```
use crate::manifest_edit::ManifestEdit;

/// Where journals live, relative to the repository root.
pub const JOURNAL_DIR: &str = ".cache/cargo-shadow/journal";

/// Where files a rollback displaced are kept, relative to the repository root.
pub const QUARANTINE_DIR: &str = ".cache/cargo-shadow/displaced";

/// The suffix appended to a lockfile name when a refused live lock is saved before the snapshot goes back.
pub const REFUSED_SUFFIX: &str = ".refused";

/// The file in a journal directory that names the crate roots,
/// one per line,
/// in the order of the snapshot files;
/// it is written last,
/// so a journal without it was never complete.
const INDEX_FILE: &str = "index.txt";

/// The file in a journal directory that says who wrote it:
/// the version line, the process id and the process start time.
const OWNER_FILE: &str = "owner";

/// The file in a journal directory that names the manifests a `--breaking` run may rewrite.
/// Its first line is `v1`, each following line is `<crate root>`, a tab and the hexadecimal bytes of the path,
/// and its last line is `end <count>`, so a truncated file is recognized.
const MANIFEST_INDEX_FILE: &str = "manifests.txt";

/// What:
/// `fn write_synced(path: &Path, text: &str) -> io::Result<()>` creates a file, writes it and forces it to disk.
/// Why:
/// A journal record counts as written only when its bytes are durable, before cargo mutates the originals.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeSynced(path: string, text: string): void  // openSync, writeSync, fsyncSync
/// ```
fn write_synced(path: &Path, text: &str) -> io::Result<()> {
    let mut file: fs::File = fs::File::create(path)?;
    file.write_all(text.as_bytes())?;
    return file.sync_all();
}

/// What:
/// `pub fn atomic_write(target: &Path, text: &str) -> io::Result<()>` publishes a file whole:
/// written and synchronized under a unique sibling name, renamed over the target, and the directory synchronized.
/// Why:
/// The journal index must appear complete or not at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function atomicWrite(target: string, text: string): void
/// ```
pub fn atomic_write(target: &Path, text: &str) -> io::Result<()> {
    let directory: PathBuf = match target.parent() {
        Some(found) => found.to_path_buf(),
        None => PathBuf::from("."),
    };
    let temp: PathBuf = directory.join(format!(".{}.shadow-{}.tmp", file_name_text(target), crate::file_owner::clock_ns()));
    write_synced(&temp, text)?;
    if let Err(error) = fs::rename(&temp, target) {
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    return sync_directory(&directory);
}

/// What:
/// `fn snapshot_of<'a>(snapshots: &'a [LockSnapshot], crate_dir: &str) -> Option<&'a LockSnapshot>` finds the snapshot of one crate root.
/// Why:
/// The restore pairs each snapshot with the output recorded for the same root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const snapshotOf = (snapshots: LockSnapshot[], crateDir: string) => snapshots.find((s) => s.crateDir === crateDir);
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
/// `pub fn restore_if_unchanged(root, snapshots, produced) -> io::Result<Vec<String>>` puts each snapshot back only when the live
/// lockfile still holds what this run's child wrote.
/// Why:
/// A live lockfile that differs from the recorded output was changed by someone else and is left alone with a warning;
/// a root with no recorded output was never written by this run (unknown ownership never authorizes a restore);
/// a lockfile that cannot be read is an error, never an empty file.
/// The file the restore displaces is kept in the quarantine directory, not deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchanged(root: string, snapshots: LockSnapshot[], produced: LockSnapshot[]): string[]
/// ```
pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>> {
    return restore_if_unchanged_with(root, snapshots, produced, &no_hook);
}

/// What:
/// `pub fn restore_if_unchanged_with(root, snapshots, produced, between)` is `restore_if_unchanged` with a hook that runs after a
/// lockfile was compared and before it is replaced.
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
        let live: String = match observe(&path)? {
            Some(found) => found.text,
            None => {
                warnings.push(format!("{}/{LOCKFILE_NAME} is missing; it was not restored", snapshot.crate_dir));
                continue;
            }
        };
        if live == snapshot.source {
            continue;
        }
        match snapshot_of(produced, &snapshot.crate_dir) {
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
        let keep: KeepAs = KeepAs::in_directory(&root.join(QUARANTINE_DIR), &format!(".{}.displaced", snapshot.crate_dir.replace('/', "__")));
        match replace_keeping_with_hook(&path, &live, &snapshot.source, &keep, between)? {
            ReplaceOutcome::Replaced(_) => {}
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
/// `pub fn run_recording<T>(root, crate_dir, produced, run) -> io::Result<(T, Claim)>` runs a child that may write one lockfile and
/// records the lockfile as this run's output when the evidence says the child wrote it.
/// Why:
/// This is the one place a lockfile is attributed to the run:
/// the live text before the child is the prior state,
/// the child runs inside a timed window,
/// and the file counts as the child's output only if it differs from the prior state and its change time lies in the window.
/// The output is recorded whether the child succeeded or failed,
/// because a child that wrote a lockfile and then exited with an error still left that lockfile behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runRecording<T>(root, crateDir, produced, run: () => T): [T, Claim]
/// ```
pub fn run_recording<T>(root: &Path, crate_dir: &str, produced: &RefCell<Vec<LockSnapshot>>, run: impl FnOnce() -> T) -> io::Result<(T, Claim)> {
    let path: PathBuf = root.join(crate_dir).join(LOCKFILE_NAME);
    let prior: String = match observe(&path)? {
        Some(found) => found.text,
        None => String::new(),
    };
    let (result, window): (T, Window) = timed(run);
    let claim: Claim = claim_output(&path, &prior, window)?;
    if let Claim::Ours(text) = &claim {
        let mut list = produced.borrow_mut();
        let earlier: Vec<LockSnapshot> = std::mem::take(&mut *list);
        for known in earlier {
            if known.crate_dir != crate_dir {
                list.push(known);
            }
        }
        list.push(LockSnapshot { crate_dir: crate_dir.to_string(), source: text.clone() });
    }
    return Ok((result, claim));
}

/// What:
/// `fn journal_path(root: &Path, pid: u32) -> PathBuf` is where a process's journal lives.
/// Why:
/// The directory is named after the process id, so a recovering run can tell whose it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journalPath = (root: string, pid: number): string => join(root, JOURNAL_DIR, String(pid));
/// ```
fn journal_path(root: &Path, pid: u32) -> PathBuf {
    return root.join(JOURNAL_DIR).join(pid.to_string());
}

/// What:
/// `pub fn process_identity(pid: u32) -> Option<String>` is the start time of a process (field 22 of `/proc/<pid>/stat`),
/// or `None` when the process does not exist.
/// Why:
/// A process id can be reused; the pair of process id and start time cannot.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processIdentity(pid: number): string | undefined
/// ```
pub fn process_identity(pid: u32) -> Option<String> {
    let text: String = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The command name is in parentheses and may contain spaces; the fields after the last `)` are plain.
    let after: &str = &text[text.rfind(')')? + 1..];
    let fields: Vec<&str> = after.split_whitespace().collect();
    return fields.get(19).map(|found| { return found.to_string(); });
}

/// What:
/// `pub fn proc_usable() -> bool` says whether `/proc` can be read for process identities.
/// Why:
/// In a chroot or a container `/proc` may be absent; a process that cannot be looked up must be assumed alive,
/// or a running transaction would be "recovered" under its own feet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const procUsable = (): boolean => existsSync('/proc/self/stat');
/// ```
pub fn proc_usable() -> bool {
    return Path::new("/proc/self/stat").exists();
}

/// What:
/// `pub fn process_exists(pid: u32, recorded_identity: &str) -> bool` says whether the process that wrote a journal is still running.
/// Why:
/// Without a usable `/proc` the answer is yes (never recover what cannot be shown dead);
/// with it, the process must exist and, when the journal recorded a start time, have that start time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function processExists(pid: number, recordedIdentity: string): boolean
/// ```
pub fn process_exists(pid: u32, recorded_identity: &str) -> bool {
    if !proc_usable() {
        return true;
    }
    return match process_identity(pid) {
        Some(now) => recorded_identity.is_empty() || now == recorded_identity,
        None => false,
    };
}

/// What:
/// `pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()>` writes this process's journal:
/// the owner record, every snapshot, and last the index,
/// each synchronized,
/// with the journal directories synchronized after.
/// Why:
/// Cargo must not be started until the originals are safely copied;
/// the index is the commit record, so a journal without it is known to be incomplete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[]): void
/// ```
pub fn begin(root: &Path, snapshots: &[LockSnapshot]) -> io::Result<()> {
    let directory: PathBuf = journal_path(root, std::process::id());
    fs::create_dir_all(&directory)?;
    let identity: String = process_identity(std::process::id()).unwrap_or_default();
    write_synced(&directory.join(OWNER_FILE), &format!("v1\n{}\n{identity}\n", std::process::id()))?;
    let mut index: String = String::new();
    for (position, snapshot) in snapshots.iter().enumerate() {
        write_synced(&directory.join(format!("{position}.lock")), &snapshot.source)?;
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    atomic_write(&directory.join(INDEX_FILE), &index)?;
    sync_directory(&directory)?;
    if let Some(parent) = directory.parent() {
        sync_directory(parent)?;
    }
    return Ok(());
}

/// What:
/// `fn encode_path(path: &Path) -> String` writes a path as the hexadecimal text of its bytes.
/// Why:
/// A lossy conversion would journal a different path than the one snapshotted when a directory name is not UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const encodePath = (path: Buffer): string => path.toString('hex');
/// ```
fn encode_path(path: &Path) -> String {
    let mut text: String = String::new();
    for byte in path.as_os_str().as_bytes() {
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// What:
/// `fn decode_path(text: &str) -> Option<PathBuf>` reads back what `encode_path` wrote.
/// Why:
/// Recovery must find the exact manifest that was snapshotted; an odd length or a non-hexadecimal digit is a corrupt record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decodePath = (text: string): Buffer | undefined => /^([0-9a-f]{2})*$/.test(text) ? Buffer.from(text, 'hex') : undefined;
/// ```
fn decode_path(text: &str) -> Option<PathBuf> {
    if text.len() % 2 != 0 {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::new();
    let mut position: usize = 0;
    while position < text.len() {
        let pair: &str = text.get(position..position + 2)?;
        bytes.push(u8::from_str_radix(pair, 16).ok()?);
        position += 2;
    }
    return Some(PathBuf::from(std::ffi::OsString::from_vec(bytes)));
}

/// What:
/// `pub fn add_manifests(root: &Path, edits: &[ManifestEdit]) -> io::Result<()>` records the manifests a `--breaking` run may rewrite
/// in this process's journal, each with its crate root and its text before the run.
/// Why:
/// A killed run must be able to put them back too, root by root.
/// Every snapshot is synchronized before the versioned index is published, and the index ends with a count, so a truncated record is detected.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function addManifests(root: string, edits: ManifestEdit[]): void  // throws when the journal cannot be written
/// ```
pub fn add_manifests(root: &Path, edits: &[ManifestEdit]) -> io::Result<()> {
    let directory: PathBuf = journal_path(root, std::process::id());
    let mut index: String = String::from("v1\n");
    for (position, edit) in edits.iter().enumerate() {
        if edit.crate_dir.contains('\n') || edit.crate_dir.contains('\t') {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the crate root {:?} contains a line feed or tab, which the journal cannot list", edit.crate_dir)));
        }
        write_synced(&directory.join(format!("m{position}.toml")), &edit.before)?;
        index.push_str(&format!("{}\t{}\n", edit.crate_dir, encode_path(&edit.path)));
    }
    index.push_str(&format!("end {}\n", edits.len()));
    atomic_write(&directory.join(MANIFEST_INDEX_FILE), &index)?;
    return sync_directory(&directory);
}

/// What:
/// `pub fn finish(root: &Path)` deletes this process's journal and synchronizes the journal directory.
/// Why:
/// The deletion must be durable too, or a completed journal could reappear after a power loss and be "recovered" over newer work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function finish(root: string): void
/// ```
pub fn finish(root: &Path) {
    let _ = fs::remove_dir_all(journal_path(root, std::process::id()));
    let _ = sync_directory(&root.join(JOURNAL_DIR));
}

/// What:
/// `fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>>` reads a journal's snapshots.
/// Why:
/// Any missing or unreadable part is an error, so the journal is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readJournal(directory: string): LockSnapshot[]  // throws on any missing part
/// ```
fn read_journal(directory: &Path) -> io::Result<Vec<LockSnapshot>> {
    let index: String = fs::read_to_string(directory.join(INDEX_FILE))?;
    let mut snapshots: Vec<LockSnapshot> = Vec::new();
    for (position, crate_dir) in index.lines().enumerate() {
        let source: String = fs::read_to_string(directory.join(format!("{position}.lock")))?;
        snapshots.push(LockSnapshot { crate_dir: crate_dir.to_string(), source });
    }
    return Ok(snapshots);
}

/// One manifest recovered from a journal.
struct JournaledManifest {
    /// The crate root the manifest belongs to.
    crate_dir: String,
    /// The manifest's path, byte for byte.
    path: PathBuf,
    /// The manifest's text before the run.
    before: String,
}

/// What:
/// `fn read_manifest_index(directory: &Path) -> io::Result<Vec<JournaledManifest>>` reads the manifest records of a journal.
/// `Ok` with an empty list means the transaction journaled no manifests (the file does not exist);
/// every other failure, including a missing `end` line or a bad count, is an error.
/// Why:
/// An unreadable index must keep the journal: it is the only copy of the original manifests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readManifestIndex(directory: string): JournaledManifest[]  // throws unless the record is complete
/// ```
fn read_manifest_index(directory: &Path) -> io::Result<Vec<JournaledManifest>> {
    let text: String = match fs::read_to_string(directory.join(MANIFEST_INDEX_FILE)) {
        Ok(found) => found,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let corrupt = |why: &str| -> io::Error { return io::Error::new(io::ErrorKind::InvalidData, format!("the manifest index is corrupt: {why}")); };
    let mut lines = text.lines();
    if lines.next() != Some("v1") {
        return Err(corrupt("no v1 header"));
    }
    let mut found: Vec<JournaledManifest> = Vec::new();
    let mut ended: Option<usize> = None;
    for line in lines {
        if let Some(count) = line.strip_prefix("end ") {
            ended = Some(count.trim().parse().map_err(|_| { return corrupt("the end count is not a number"); })?);
            continue;
        }
        if ended.is_some() {
            return Err(corrupt("text after the end line"));
        }
        let (crate_dir, encoded) = line.split_once('\t').ok_or_else(|| { return corrupt("a line has no tab"); })?;
        let path: PathBuf = decode_path(encoded).ok_or_else(|| { return corrupt("a path is not hexadecimal"); })?;
        let before: String = fs::read_to_string(directory.join(format!("m{}.toml", found.len())))?;
        found.push(JournaledManifest { crate_dir: crate_dir.to_string(), path, before });
    }
    if ended != Some(found.len()) {
        return Err(corrupt("the end count does not match the lines"));
    }
    return Ok(found);
}

/// What a recovery decided about one crate root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RootDecision {
    /// The lockfile the killed run left passed the policy and was kept.
    Kept,
    /// The lockfile was put back, or already equalled the snapshot.
    Restored,
    /// Another writer's lockfile is in place and was left alone.
    LeftAlone,
}

/// What:
/// `fn recover_manifests(manifests, decisions, between, messages) -> bool` puts back the manifests of the roots whose lockfile was
/// put back, saving the text it replaces as `<name>.refused`, and says whether all were settled.
/// Why:
/// Each root is decided on its own: a kept lockfile keeps its manifests, a lockfile left to another writer keeps its manifests too,
/// and only a root whose lockfile went back gets its manifests back.
/// A manifest that cannot be read, or cannot be put back, keeps the journal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverManifests(manifests, decisions, between, messages): boolean
/// ```
fn recover_manifests(manifests: &[JournaledManifest], decisions: &[(String, RootDecision)], between: &dyn Fn(&Path), messages: &mut Vec<String>) -> bool {
    let mut all_settled: bool = true;
    for manifest in manifests {
        let decision: RootDecision = decisions
            .iter()
            .find(|(crate_dir, _)| { return *crate_dir == manifest.crate_dir; })
            .map(|(_, found)| { return *found; })
            .unwrap_or(RootDecision::Restored);
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
            Ok(ReplaceOutcome::Preserved(kept)) => messages.push(format!("{shown} was changed twice by other processes while a killed run was being recovered; the file displaced is kept as {}", kept.display())),
            Err(error) => {
                all_settled = false;
                messages.push(format!("{shown}: could not recover a killed run: {error}; its journal was kept"));
            }
        }
    }
    return all_settled;
}

/// What:
/// `fn read_identity(directory: &Path) -> String` reads the process start time a journal recorded (empty when it recorded none).
/// Why:
/// The journal's directory name carries the process id; the start time tells a reused id from the process that wrote the journal.
/// A journal without an owner record (written by an older version) has an empty identity, which accepts any process with that id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readIdentity(directory: string): string
/// ```
fn read_identity(directory: &Path) -> String {
    let text: String = fs::read_to_string(directory.join(OWNER_FILE)).unwrap_or_default();
    let mut lines = text.lines();
    if lines.next() != Some("v1") {
        return String::new();
    }
    lines.next();
    return lines.next().unwrap_or("").to_string();
}

/// What:
/// `pub fn recover(root, is_alive, judge) -> Vec<String>` finishes the journals of processes that no longer exist.
/// Why:
/// See `recover_with`; this form has no hook.
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
/// `<original>.claim.<this pid>.<time>`; only one of several racing runs succeeds.
/// Why:
/// Two recovering runs must never act on the same journal, and a live transaction must never be recovered by a stale reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claimJournal(directory: string, original: string): string | undefined  // renameSync, undefined when it fails
/// ```
fn claim_journal(directory: &Path, original: &str) -> Option<PathBuf> {
    let parent: &Path = directory.parent()?;
    let claimed: PathBuf = parent.join(format!("{original}.claim.{}.{}", std::process::id(), crate::file_owner::clock_ns()));
    return match fs::rename(directory, &claimed) {
        Ok(()) => Some(claimed),
        Err(_) => None,
    };
}

/// What:
/// `fn release_claim(directory: &Path, journals: &Path, original: &str)` gives a claimed journal back its own name.
/// Why:
/// A recovery that did not finish keeps the journal, and the next run must be able to find and retry it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function releaseClaim(directory: string, journals: string, original: string): void  // renameSync, errors ignored
/// ```
fn release_claim(directory: &Path, journals: &Path, original: &str) {
    let _ = fs::rename(directory, journals.join(original));
}

/// What:
/// `pub fn recover_with(root, is_alive, judge, between) -> Vec<String>` finishes the journals of processes that no longer exist.
/// Why:
/// For each such journal, after claiming it:
/// a live lock equal to its snapshot needs nothing;
/// a live lock the `judge` finds acceptable (no policy findings for the versions it added) is kept;
/// otherwise the live lock is saved beside the lockfile (`Cargo.lock.refused`, the first free `.refused.N`) and the snapshot goes back;
/// the manifests of a root whose lockfile went back are put back too.
/// The journal is deleted (and the deletion synchronized) only when every root was settled;
/// any unreadable part or failed step keeps it, and the claim is released so the next run can retry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverWith({ root, isAlive, judge, between }): string[]
/// ```
pub fn recover_with(root: &Path, is_alive: &dyn Fn(u32, &str) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>, between: &dyn Fn(&Path)) -> Vec<String> {
    let mut messages: Vec<String> = Vec::new();
    let journals: PathBuf = root.join(JOURNAL_DIR);
    let entries = match fs::read_dir(&journals) {
        Ok(found) => found,
        Err(_) => return messages,
    };
    let own_identity: String = process_identity(std::process::id()).unwrap_or_default();
    for listed in entries.flatten() {
        let name: String = listed.file_name().to_string_lossy().to_string();
        // `<pid>` is a journal; `<pid>.claim.<claimer>.<time>` is one that a recovery claimed and did not finish.
        let (original, claimer): (String, Option<u32>) = match name.split_once(".claim.") {
            Some((first, rest)) => (first.to_string(), rest.split('.').next().and_then(|found| { return found.parse().ok(); })),
            None => (name.clone(), None),
        };
        let pid: u32 = match original.parse() {
            Ok(found) => found,
            Err(_) => continue,
        };
        if let Some(other) = claimer
            && process_exists(other, "")
            && other != std::process::id()
        {
            continue;
        }
        let identity: String = read_identity(&listed.path());
        let own: bool = pid == std::process::id() && (identity.is_empty() || identity == own_identity);
        if own || is_alive(pid, &identity) {
            continue;
        }
        let directory: PathBuf = match claim_journal(&listed.path(), &original) {
            Some(found) => found,
            // Another run claimed it first.
            None => continue,
        };
        let before: Vec<LockSnapshot> = match read_journal(&directory) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: the journal of a killed run could not be read ({error}); it was kept", directory.display()));
                release_claim(&directory, &journals, &original);
                continue;
            }
        };
        let manifests: Vec<JournaledManifest> = match read_manifest_index(&directory) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: the manifest record of a killed run could not be read ({error}); its journal was kept", directory.display()));
                release_claim(&directory, &journals, &original);
                continue;
            }
        };
        let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
        let live: Vec<LockSnapshot> = match read_locks(root, &dirs) {
            Ok(found) => found,
            Err(error) => {
                messages.push(format!("{}: a lockfile of a killed run could not be read ({error}); its journal was kept", directory.display()));
                release_claim(&directory, &journals, &original);
                continue;
            }
        };
        let mut settled: bool = true;
        let mut decisions: Vec<(String, RootDecision)> = Vec::new();
        for (old, now) in before.iter().zip(live.iter()) {
            if old.source == now.source {
                decisions.push((old.crate_dir.clone(), RootDecision::Restored));
                continue;
            }
            let problems: Vec<String> = judge(std::slice::from_ref(old), std::slice::from_ref(now));
            let path: PathBuf = root.join(&old.crate_dir).join(LOCKFILE_NAME);
            if problems.is_empty() {
                messages.push(format!("{}/{LOCKFILE_NAME} was left by a run that was killed; its new versions pass the policy, so it was kept", old.crate_dir));
                decisions.push((old.crate_dir.clone(), RootDecision::Kept));
                continue;
            }
            let keep: KeepAs = KeepAs::next_to_target(REFUSED_SUFFIX);
            match replace_keeping_with_hook(&path, &now.source, &old.source, &keep, between) {
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
                    decisions.push((old.crate_dir.clone(), RootDecision::Restored));
                }
                Ok(ReplaceOutcome::LeftAlone) => {
                    messages.push(format!(
                        "{}/{LOCKFILE_NAME} was changed by another process while a killed run was being recovered; it was left as that process wrote it, not restored",
                        old.crate_dir
                    ));
                    decisions.push((old.crate_dir.clone(), RootDecision::LeftAlone));
                }
                Ok(ReplaceOutcome::Preserved(kept)) => {
                    messages.push(format!(
                        "{}/{LOCKFILE_NAME} was changed twice by other processes while a killed run was being recovered; the file displaced is kept as {}",
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
        if !recover_manifests(&manifests, &decisions, between, &mut messages) {
            settled = false;
        }
        if settled {
            let _ = fs::remove_dir_all(&directory);
            let _ = sync_directory(&journals);
        } else {
            // Release the claim so the next run retries.
            release_claim(&directory, &journals, &original);
        }
    }
    return messages;
}
