//! What:
//! The verdict on a journal that has no ready record:
//! whether it holds anything the live files do not already hold.
//! Why:
//! Split out of `journal_unready.rs`,
//! which assesses the journals of killed runs,
//! to keep that file under the code-line limit.
//! The items are re-exported from `journal_unready`,
//! so their paths there are unchanged.

/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::ExplainedError;
/// Imports `std::fs` for this file.
use std::fs;

/// Imports `std::io` for this file.
use std::io;

/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path};



/// Imports `crate::constants::LOCKFILE_NAME` for this file.
use crate::constants::LOCKFILE_NAME;

/// Imports the control group records and queries for this file.
use crate::control_group::{LauncherState, launcher_state};

/// Imports `crate::crates::{LockSnapshot, read_locks}` for this file.
use crate::crates::{LockSnapshot, read_locks};

/// Imports the journal file names and the durability barrier for this file.
use crate::journal::{ACKNOWLEDGED_FILE, GROUPS_FILE, INDEX_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, VIEW_FILE};

/// Imports `crate::journal_manifests::read_manifest_index` for this file.
use crate::journal_manifests::read_manifest_index;

/// Imports `crate::journal_recover::read_journal` for this file.
use crate::journal_recover::read_journal;


/// What:
/// `fn is_digit_run(stem: &str) -> bool` says whether text is a non-empty run of decimal digits.
/// Why:
/// The numbered files of a journal (`7.lock`,
/// `m7.toml`) are recognized by their stems.
pub(crate) fn is_digit_run(stem: &str) -> bool {
    return !stem.is_empty() && stem.bytes().all(is_digit_byte);
}

/// What:
/// `fn is_digit_byte(byte: u8) -> bool` says whether a byte is `0` to `9`.
/// Why:
/// `bytes().all` takes it by path instead of a closure.
pub(crate) fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
/// `fn is_lock(name: &str) -> bool` says whether a file name is a lockfile snapshot (`<number>.lock`).
/// Why:
/// A function replaces the closure that `unready_verdict` called for each name.
pub(crate) fn is_lock(name: &str) -> bool {
    return match name.strip_suffix(".lock") {
        Some(stem) => is_digit_run(stem),
        None => false,
    };
}

/// What:
/// `fn is_snapshot(name: &str) -> bool` says whether a file name is a manifest snapshot (`m<number>.toml`).
/// Why:
/// A function replaces the closure that `unready_verdict` called for each name.
pub(crate) fn is_snapshot(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('m') else {
        return false;
    };
    return match rest.strip_suffix(".toml") {
        Some(stem) => is_digit_run(stem),
        None => false,
    };
}

/// What:
/// `fn is_unknown_file(name: &str) -> bool` says whether a file in a journal directory is none that this version writes.
/// Why:
/// An unknown file may be the only copy of something,
/// so the journal is kept.
pub(crate) fn is_unknown_file(name: &str) -> bool {
    return ![OWNER_FILE, GROUPS_FILE, VIEW_FILE, ACKNOWLEDGED_FILE, INDEX_FILE, MANIFEST_INDEX_FILE].contains(&name) && !is_lock(name) && !is_snapshot(name) && !name.ends_with(".tmp");
}

/// What:
/// `fn is_allocatable_process_id(pid: u32) -> bool` says whether a number is a process id that Linux can allocate.
/// Why:
/// `Result::is_ok_and` takes it by path instead of a closure.
pub(crate) fn is_allocatable_process_id(pid: u32) -> bool {
    return (1..FIRST_PROCESS_ID_LINUX_NEVER_ALLOCATES).contains(&pid);
}

/// What:
/// `fn sync_opened(opened: fs::File) -> io::Result<()>` forces an open file to disk.
/// Why:
/// `Result::and_then` takes it by path instead of a closure.
pub(crate) fn sync_opened(opened: fs::File) -> io::Result<()> {
    return opened.sync_all();
}

/// What:
/// `fn launcher_verdict_of(noted: (i32, u64, u64)) -> (i32, LauncherState)` asks what became of a noted launcher:
/// its process id,
/// start time,
/// and process-id namespace go in,
/// the process id and its state come out.
/// Why:
/// `Option::map` takes it by path instead of a closure.
pub(crate) fn launcher_verdict_of(noted: (i32, u64, u64)) -> (i32, LauncherState) {
    let (pid, start, pid_namespace) = noted;
    return (pid, launcher_state(pid, start, pid_namespace));
}

/// What:
/// `fn note_uncertain(uncertain: &mut Option<String>, message: String)` keeps the first reason a verdict is uncertain.
/// Why:
/// A function replaces the closure that captured the `uncertain` variable.
pub(crate) fn note_uncertain(uncertain: &mut Option<String>, message: String) {
    if uncertain.is_none() {
        *uncertain = Some(message);
    }
}

/// What:
/// `fn unready_verdict(root: &Path, directory: &Path) -> Result<Vec<String>, String>` decides whether a journal without a ready
/// record holds nothing that the live files do not already hold,
/// and returns the live files to make durable before it is removed;
/// `Err` says why it must be kept.
/// Why:
/// Round 6 (5b2 3):
/// equal lockfiles do not make a manifest snapshot or an unknown file redundant,
/// and a missing index is no proof that the directory is empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unreadyVerdict(root: string, directory: string): string[]  // throws the reason to keep it
/// ```
pub(crate) fn unready_verdict(root: &Path, directory: &Path) -> Result<Vec<String>, String> {
    let listing: fs::ReadDir = fs::read_dir(directory).explained(format_args!("it could not be listed"))?;
    let mut names: Vec<String> = Vec::new();
    for entry in listing.flatten() {
        names.push(entry.file_name().to_string_lossy().to_string());
    }
    let mut unknown_name: Option<&String> = None;
    for name in &names {
        if is_unknown_file(name) {
            unknown_name = Some(name);
            break;
        }
    }
    if let Some(unknown) = unknown_name {
        return Err(format!("it holds {unknown}, which this version does not know"));
    }
    let mut locks: usize = 0;
    let mut snapshots: usize = 0;
    for name in &names {
        if is_lock(name) {
            locks += 1;
        }
        if is_snapshot(name) {
            snapshots += 1;
        }
    }
    let mut live_files: Vec<String> = Vec::new();
    match read_journal(directory) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if locks > 0 {
                return Err("its index is missing but lockfile snapshots remain".to_string());
            }
        }
        Err(error) => return Err(format!("its index is damaged ({error})")),
        Ok(before) => {
            if locks != before.len() {
                return Err("it holds lockfile snapshots that its index does not list".to_string());
            }
            let dirs: Vec<String> = before.iter().map(LockSnapshot::crate_dir_text).collect();
            let live = read_locks(root, &dirs).explained(format_args!("a live lockfile could not be read"))?;
            for (old, now) in before.iter().zip(live.iter()) {
                if old.source != now.source {
                    return Err("it holds the only copy of a lockfile that differs from the live one".to_string());
                }
            }
            for crate_dir in &dirs {
                live_files.push(root.join(crate_dir).join(LOCKFILE_NAME).to_string_lossy().to_string());
            }
        }
    }
    let mut has_manifest_index: bool = false;
    for name in &names {
        if name == MANIFEST_INDEX_FILE {
            has_manifest_index = true;
            break;
        }
    }
    if has_manifest_index {
        let manifests = read_manifest_index(directory).explained(format_args!("its manifest index is damaged"))?;
        if snapshots != manifests.len() {
            return Err("it holds manifest snapshots that its manifest index does not list".to_string());
        }
        for manifest in &manifests {
            match fs::read_to_string(&manifest.path) {
                Ok(live) if live == manifest.before => live_files.push(manifest.path.to_string_lossy().to_string()),
                Ok(_) => return Err(format!("it holds the only copy of {}, which differs from the live file", manifest.path.display())),
                Err(error) => return Err(format!("{} could not be read ({error})", manifest.path.display())),
            }
        }
    } else if snapshots > 0 {
        return Err("manifest snapshots remain without a manifest index".to_string());
    }
    return Ok(live_files);
}

/// The first process id Linux never allocates:
/// `PID_MAX_LIMIT` for 64-bit kernels (`include/linux/threads.h`),
/// and the value of
/// `/proc/sys/kernel/pid_max` is "one greater than the maximum PID" (`proc_sys_kernel(5)`).
/// An acknowledgement naming zero,
/// this number
/// or a larger one was not written by a process.
pub(crate) const FIRST_PROCESS_ID_LINUX_NEVER_ALLOCATES: u32 = 4_194_304;
