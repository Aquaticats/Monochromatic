//! What:
//! The removal of dated nightly Rust toolchains that no checkout's lock pins,
//! after the grace period.
//! Why:
//! The repository installs a dated nightly per `toolchain:update` and rustup never removes one
//! (20 toolchains of about 2.7 GiB each on 2026-11-04,
//! `doc/planning/lean-build-dirs.md`).
//! A toolchain is kept when any checkout pins it
//! (`toolchain.lock.json` `.tools.rust.version`,
//! `.cache/toolchain.env.json` `RUSTUP_TOOLCHAIN`,
//! or a `rust-toolchain` file),
//! when rustup's own settings name it as the default or an override,
//! when a process runs a program from it or has it in its environment,
//! or when it is younger than the grace period.
//! Stable releases and the channel names are never touched.
//! The removal is rustup's own (`rustup toolchain uninstall`),
//! after the identity of the directory is checked once more.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function retireNightlies(toolchainsDirectory: string, pinned: Set<string>, ...): Outcome[]
//! ```

/// The streaming log line.
use crate::lean_run_report::emit;
/// Files,
/// sets and paths.
use std::{collections::HashSet, fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// The proof and the identity.
use crate::lean_identity::{Deadline, Identity, identity_of};
/// Imports the names this file uses from `crate::lean_proc_scan`.
use crate::lean_proc_scan::scan_processes;
/// Imports the names this file uses from `crate::lean_sizes`.
use crate::lean_sizes::tree_bytes;
/// Named reads of directory entries.
use crate::lean_text::is_directory;

/// The environment variable that names a toolchain.
const TOOLCHAIN_VARIABLE: &str = "RUSTUP_TOOLCHAIN";

/// One installed toolchain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Installed {
    /// The directory name under `toolchains/` (`nightly-2026-11-07-x86_64-unknown-linux-gnu`).
    pub directory_name: String,
    /// The directory.
    pub path: PathBuf,
}

/// What a retirement did with one toolchain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ToolchainVerdict {
    /// Kept for this reason.
    Keep(
        /// Why the toolchain is kept.
        String,
    ),
    /// Uninstalled (or in a dry run would be) with these disk bytes.
    Retired {
        /// The disk bytes freed or that would be.
        bytes: u64,
    },
    /// Not removed,
    /// for this reason.
    Skip(
        /// Why the toolchain was not removed.
        String,
    ),
}

/// Something that uninstalls a toolchain by name.
pub trait Uninstaller {
    /// Uninstalls the toolchain with this directory name.
    fn uninstall(&self, directory_name: &str) -> Result<(), String>;
}

/// What:
/// `pub fn dated_nightly_name(directory_name: &str) -> Option<String>` is `nightly-YYYY-MM-DD` for a dated nightly's directory name.
/// Why:
/// Only dated nightlies are candidates;
/// the channel `nightly`,
/// `stable` and numbered releases are not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const datedNightlyName = (dir: string): string | undefined => /^(nightly-\d{4}-\d{2}-\d{2})(-|$)/.exec(dir)?.[1];
/// ```
pub fn dated_nightly_name(directory_name: &str) -> Option<String> {
    let rest: &str = directory_name.strip_prefix("nightly-")?;
    let date: &str = rest.get(..10)?;
    let bytes: &[u8] = date.as_bytes();
    let mut shaped: bool = bytes[4] == b'-' && bytes[7] == b'-';
    for (index, character) in date.chars().enumerate() {
        if index != 4 && index != 7 && !character.is_ascii_digit() {
            shaped = false;
        }
    }
    let after_date: &str = &rest[10..];
    if !shaped || !(after_date.is_empty() || after_date.starts_with('-')) {
        return None;
    }
    return Some(format!("nightly-{date}"));
}

/// What:
/// `fn pins_in(checkout: &Path, pinned: &mut HashSet<String>)` adds the toolchain names a checkout pins.
/// Why:
/// A checkout that has not re-run the install still uses the older nightly its environment file names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pinsIn(checkout: string, pinned: Set<string>): void
/// ```
fn pins_in(checkout: &Path, pinned: &mut HashSet<String>) {
    if let Ok(text) = fs::read_to_string(checkout.join("toolchain.lock.json"))
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
        && let Some(version) = value.pointer("/tools/rust/version").and_then(serde_json::Value::as_str)
    {
        pinned.insert(version.to_string());
    }
    if let Ok(text) = fs::read_to_string(checkout.join(".cache").join("toolchain.env.json"))
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
        && let Some(name) = value.get(TOOLCHAIN_VARIABLE).and_then(serde_json::Value::as_str)
    {
        pinned.insert(name.to_string());
    }
    for file in ["rust-toolchain", "rust-toolchain.toml"] {
        if let Ok(text) = fs::read_to_string(checkout.join(file)) {
            for line in text.lines() {
                let trimmed: &str = line.trim().trim_start_matches("channel").trim_start_matches([' ', '=']).trim_matches(['"', ' ']);
                if trimmed.starts_with("nightly-") {
                    pinned.insert(trimmed.to_string());
                }
            }
        }
    }
}

/// What:
/// `pub fn pinned_by(checkouts: &[PathBuf], settings: &Path) -> HashSet<String>` is every toolchain name some checkout pins,
/// plus the default and the overrides in rustup's settings file.
/// Why:
/// All of them are kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pinnedBy(checkouts: string[], settings: string): Set<string>
/// ```
pub fn pinned_by(checkouts: &[PathBuf], settings: &Path) -> HashSet<String> {
    let mut pinned: HashSet<String> = HashSet::new();
    for checkout in checkouts {
        pins_in(checkout, &mut pinned);
    }
    if let Ok(text) = fs::read_to_string(settings) {
        for line in text.lines() {
            if let Some((_, value)) = line.split_once('=') {
                let name: &str = value.trim().trim_matches('"');
                if name.starts_with("nightly-") {
                    pinned.insert(name.to_string());
                }
            }
        }
    }
    return pinned;
}

/// What:
/// `fn is_nul(byte: &u8) -> bool` says whether a byte is the NUL that separates environment entries.
/// Why:
/// `/proc/<pid>/environ` is NUL-separated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isNul = (byte: number): boolean => byte === 0;
/// ```
fn is_nul(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
/// `fn by_directory_name(left: &Installed, right: &Installed) -> std::cmp::Ordering` orders toolchains by directory name.
/// Why:
/// Reports list them in a stable order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byDirectoryName = (a: Installed, b: Installed): number => a.directoryName.localeCompare(b.directoryName);
/// ```
fn by_directory_name(left: &Installed, right: &Installed) -> std::cmp::Ordering {
    return left.directory_name.cmp(&right.directory_name);
}

/// What:
/// `fn named_in_environments(proc_root: &Path) -> HashSet<String>` collects the `RUSTUP_TOOLCHAIN` value of every readable
/// process.
/// Why:
/// A rustup proxy that is about to start the compiler names its toolchain only in its environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedInEnvironments(procRoot: string): Set<string>
/// ```
fn named_in_environments(proc_root: &Path) -> HashSet<String> {
    let mut named: HashSet<String> = HashSet::new();
    let Ok(entries) = fs::read_dir(proc_root) else { return named };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().parse::<i32>().is_err() {
            continue;
        }
        let Ok(bytes) = fs::read(entry.path().join("environ")) else { continue };
        for pair in bytes.split(is_nul) {
            let text: String = String::from_utf8_lossy(pair).to_string();
            if let Some(value) = text.strip_prefix("RUSTUP_TOOLCHAIN=") {
                named.insert(value.to_string());
            }
        }
    }
    return named;
}

/// What:
/// `pub fn installed_toolchains(toolchains: &Path) -> Vec<Installed>` lists the toolchain directories.
/// Why:
/// Candidates come from here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const installedToolchains = (toolchains: string): Installed[]
/// ```
pub fn installed_toolchains(toolchains: &Path) -> Vec<Installed> {
    let mut installed: Vec<Installed> = Vec::new();
    if let Ok(entries) = fs::read_dir(toolchains) {
        for entry in entries.flatten() {
            if is_directory(&entry) {
                installed.push(Installed { directory_name: entry.file_name().to_string_lossy().to_string(), path: entry.path() });
            }
        }
    }
    installed.sort_by(by_directory_name);
    return installed;
}

/// What:
/// `pub fn retire_nightlies(installed: &[Installed], pinned: &HashSet<String>, proc_root: &Path, own_pid: i32, grace_seconds: i64, now: i64, dry_run: bool, uninstaller: &dyn Uninstaller, deadline: &Deadline, log: &mut Vec<String>) -> Vec<(String, ToolchainVerdict)>`
/// retires every dated nightly that nothing holds.
/// Why:
/// It is the owner's answer "Rust toolchains (Recommended)":
/// nightlies no checkout's lock pins,
/// each after the grace period.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireNightlies(installed: Installed[], pinned: Set<string>, procRoot: string, ownPid: number, graceSeconds: number, now: number, dryRun: boolean, uninstaller: Uninstaller, deadline: Deadline, log: string[]): [string, ToolchainVerdict][]
/// ```
pub fn retire_nightlies(
    installed: &[Installed],
    pinned: &HashSet<String>,
    proc_root: &Path,
    own_pid: i32,
    grace_seconds: i64,
    now: i64,
    dry_run: bool,
    uninstaller: &dyn Uninstaller,
    deadline: &Deadline,
    log: &mut Vec<String>,
) -> Vec<(String, ToolchainVerdict)> {
    let mut verdicts: Vec<(String, ToolchainVerdict)> = Vec::new();
    let environment_names: HashSet<String> = named_in_environments(proc_root);
    for toolchain in installed {
        let Some(short) = dated_nightly_name(&toolchain.directory_name) else { continue };
        let verdict: ToolchainVerdict = judge(toolchain, &short, pinned, &environment_names, proc_root, own_pid, grace_seconds, now, dry_run, uninstaller, deadline, log);
        verdicts.push((toolchain.directory_name.clone(), verdict));
    }
    return verdicts;
}

/// What:
/// `fn judge(...) -> ToolchainVerdict` decides and,
/// when nothing holds the toolchain,
/// retires it.
/// Why:
/// One function holds the order of the proofs:
/// pinned,
/// named by a process,
/// young,
/// used by a process,
/// then identity,
/// then removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function judge(...): ToolchainVerdict
/// ```
fn judge(
    toolchain: &Installed,
    short: &str,
    pinned: &HashSet<String>,
    environment_names: &HashSet<String>,
    proc_root: &Path,
    own_pid: i32,
    grace_seconds: i64,
    now: i64,
    dry_run: bool,
    uninstaller: &dyn Uninstaller,
    deadline: &Deadline,
    log: &mut Vec<String>,
) -> ToolchainVerdict {
    for name in [short, toolchain.directory_name.as_str()] {
        if pinned.contains(name) {
            return ToolchainVerdict::Keep(format!("a checkout's lock or rustup's settings pin {name}"));
        }
        if environment_names.contains(name) {
            return ToolchainVerdict::Keep(format!("a process has {TOOLCHAIN_VARIABLE}={name}"));
        }
    }
    let modified: i64 = match fs::symlink_metadata(&toolchain.path) {
        Ok(metadata) => metadata.mtime(),
        Err(_) => now,
    };
    if now - modified <= grace_seconds {
        return ToolchainVerdict::Keep(format!("installed or changed {} s ago, within the grace period of {grace_seconds} s", now - modified));
    }
    let Some(scan) = scan_processes(proc_root, &[toolchain.path.as_path()], &[], own_pid, deadline) else {
        return ToolchainVerdict::Keep(String::from("the deadline passed while scanning the processes"));
    };
    if let Some(used) = scan.blocks(&toolchain.path) {
        return ToolchainVerdict::Keep(format!("pid {} uses {} as {}", used.pid, used.path.display(), used.how));
    }
    let bytes: u64 = tree_bytes(&toolchain.path, deadline).bytes;
    if dry_run {
        emit(log, format!("would retire toolchain {} ({bytes} bytes): no checkout's lock pins it, no process names or uses it", toolchain.directory_name));
        return ToolchainVerdict::Retired { bytes };
    }
    let judged: Identity = match identity_of(&toolchain.path) {
        Ok(found) => found,
        Err(error) => return ToolchainVerdict::Skip(error.to_string()),
    };
    // The identity is read again just before rustup is asked to remove the directory.
    if identity_of(&toolchain.path).ok() != Some(judged) {
        return ToolchainVerdict::Skip(String::from("the directory changed identity before the removal"));
    }
    emit(log, format!(
        "retiring toolchain {} (device {} inode {}, {bytes} bytes): no checkout's lock pins it, no process names or uses it, last changed {} s ago; proof: {} ; pinned names considered: {}",
        toolchain.directory_name,
        judged.device,
        judged.inode,
        now - modified,
        scan.describe(),
        pinned.len()
    ));
    return match uninstaller.uninstall(&toolchain.directory_name) {
        Ok(()) => ToolchainVerdict::Retired { bytes },
        Err(reason) => ToolchainVerdict::Skip(reason),
    };
}
