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
use std::io;
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
    return fs::rename(&temp, target).inspect_err(|_| { let _ = fs::remove_file(&temp); });
}

/// What:
/// `pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>>`
/// puts each snapshot back only when the live lockfile still holds what this run produced.
/// Why:
/// A live lockfile that differs from `produced` was changed by another process after cargo finished;
/// replacing it would discard
/// that change,
/// so it is left alone and named in a warning.
/// One that already equals the snapshot needs nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function restoreIfUnchanged({ root, snapshots, produced }): string[]
/// ```
pub fn restore_if_unchanged(root: &Path, snapshots: &[LockSnapshot], produced: &[LockSnapshot]) -> io::Result<Vec<String>> {
    let mut warnings: Vec<String> = Vec::new();
    for snapshot in snapshots {
        let path: PathBuf = root.join(&snapshot.crate_dir).join(LOCKFILE_NAME);
        let live: String = fs::read_to_string(&path).unwrap_or_default();
        if live == snapshot.source {
            continue;
        }
        let expected: Option<&LockSnapshot> = produced.iter().find(|candidate| { return candidate.crate_dir == snapshot.crate_dir; });
        if expected.map(|found| { return found.source != live; }).unwrap_or(false) {
            warnings.push(format!(
                "{}/{LOCKFILE_NAME} was changed by another process while the step ran; it was left as that process wrote it, not restored",
                snapshot.crate_dir
            ));
            continue;
        }
        atomic_write(&path, &snapshot.source)?;
    }
    return Ok(warnings);
}

/// Where this process's journal lives.
fn journal_path(root: &Path, pid: u32) -> PathBuf {
    return root.join(JOURNAL_DIR).join(pid.to_string());
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

/// Whether a process with this id exists (Linux `/proc`;
/// a reused id reads as alive,
/// which only delays a recovery).
pub fn process_exists(pid: u32) -> bool {
    return Path::new("/proc").join(pid.to_string()).exists();
}

/// What:
/// `pub fn recover(root, is_alive, judge) -> Vec<String>` finishes the journals of processes that no longer exist.
/// Why:
/// For each such journal:
/// a live lock equal to its snapshot needs nothing;
/// a live lock the `judge` finds acceptable (no policy
/// findings for the versions it added) is kept;
/// otherwise the live lock is saved as `Cargo.lock.refused` and the snapshot goes
/// back by atomic rename.
/// Either way the journal is removed and a message says what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recover({ root, isAlive, judge }): string[]
/// ```
pub fn recover(root: &Path, is_alive: &dyn Fn(u32) -> bool, judge: &dyn Fn(&[LockSnapshot], &[LockSnapshot]) -> Vec<String>) -> Vec<String> {
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
        let before: Vec<LockSnapshot> = match read_journal(&directory) {
            Ok(found) => found,
            Err(_) => {
                let _ = fs::remove_dir_all(&directory);
                continue;
            }
        };
        let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
        let live: Vec<LockSnapshot> = match read_locks(root, &dirs) {
            Ok(found) => found,
            Err(_) => {
                let _ = fs::remove_dir_all(&directory);
                continue;
            }
        };
        for (old, now) in before.iter().zip(live.iter()) {
            if old.source == now.source {
                continue;
            }
            let problems: Vec<String> = judge(std::slice::from_ref(old), std::slice::from_ref(now));
            let path: PathBuf = root.join(&old.crate_dir).join(LOCKFILE_NAME);
            if problems.is_empty() {
                messages.push(format!("{}/{LOCKFILE_NAME} was left by a run that was killed; its new versions pass the policy, so it was kept", old.crate_dir));
                continue;
            }
            let refused: PathBuf = path.with_file_name(format!("{LOCKFILE_NAME}{REFUSED_SUFFIX}"));
            let saved: io::Result<()> = atomic_write(&refused, &now.source).and_then(|()| { return atomic_write(&path, &old.source); });
            match saved {
                Ok(()) => messages.push(format!(
                    "{}/{LOCKFILE_NAME} was left by a run that was killed and holds versions the policy refuses ({}); it was saved as {LOCKFILE_NAME}{REFUSED_SUFFIX} and the earlier lockfile was put back",
                    old.crate_dir,
                    problems.join("; ")
                )),
                Err(error) => messages.push(format!("{}/{LOCKFILE_NAME}: could not recover a killed run: {error}", old.crate_dir)),
            }
        }
        let _ = fs::remove_dir_all(&directory);
    }
    return messages;
}
