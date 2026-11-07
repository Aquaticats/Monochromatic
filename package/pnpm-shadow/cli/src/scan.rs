//! What:
//! Finds the root of the pnpm workspace that contains the current directory and checks every manifest under it.
//! Why:
//! The shadow warns before each pnpm run,
//! so it needs the whole tree's findings in one pass,
//! and that pass is on the
//! path of every pnpm call.
//! The rule itself lives in `monochromatic-catalog-specifier`;
//! this file only walks and reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // scan.ts
//! export function findWorkspaceRoot(start: string): string | undefined { ... }
//! export function scanWorkspace(root: string): ScanReport { ... }
//! ```

/// Imports the walker that honors `.gitignore`,
/// its entry and error types,
/// and the answer a visitor gives after each entry.
use ignore::{DirEntry, Error as WalkError, WalkBuilder, WalkState};

/// Imports the rule:
/// one function for any manifest name,
/// one for a name alone,
/// and the finding type.
use monochromatic_catalog_specifier::{Finding, check_manifest, check_manifest_path};

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// Imports the lock that lets several walker threads add to one report.
use std::sync::Mutex;

/// Imports the fixed values.
use crate::constants::{MANIFEST_NAMES, READ_MANIFEST_NAME, SKIPPED_DIRECTORIES, WORKSPACE_MARKER};

/// What:
/// `pub struct ScanReport { ... }` is a record type with three fields,
/// all owned by the record:
/// a count
/// (`usize`,
/// the unsigned integer sized for counts and indexes;
/// siblings `u32`,
/// `u64`),
/// a list of findings
/// (`Vec<Finding>`,
/// a growable list;
/// siblings `&[Finding]`,
/// `[Finding; N]`) and a list of notes (`Vec<String>`,
/// owned text;
/// sibling `&str`).
/// Why:
/// The caller prints findings and notes alike,
/// and the count says how many `package.json` files were really read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type ScanReport = { manifestsChecked: number; findings: Finding[]; notes: string[] };
/// ```
pub struct ScanReport {
    /// How many `package.json` files were read and checked.
    pub manifests_checked: usize,
    /// Every finding,
    /// sorted by path;
    /// the findings of one file keep the rule's own order.
    pub findings: Vec<Finding>,
    /// Problems that kept a file from being checked at all (an unreadable file or directory),
    /// sorted.
    pub notes: Vec<String>,
}

/// What:
/// `pub fn find_workspace_root(start: &Path) -> Option<PathBuf>` walks from `start` up to the filesystem root and
/// returns the first directory that holds `pnpm-workspace.yaml`.
/// Why:
/// pnpm itself finds its workspace this way,
/// and the shadow checks the tree pnpm would use.
/// Only the file's
/// presence is tested:
/// the YAML is never read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findWorkspaceRoot(start: string): string | undefined {
///   for (let dir = start; ; dir = dirname(dir)) { if (existsSync(join(dir, 'pnpm-workspace.yaml'))) return dir; if (dir === dirname(dir)) return undefined; }
/// }
/// ```
pub fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    // `start.ancestors()` yields `start` first, then each parent up to the root.
    for directory in start.ancestors() {
        if directory.join(WORKSPACE_MARKER).is_file() {
            return Some(directory.to_path_buf());
        }
    }
    return None;
}

/// What:
/// `fn is_skipped_directory(entry: &DirEntry) -> bool` says whether the walk must not enter this entry.
/// Why:
/// `.git` and `node_modules` are never tracked sources;
/// skipping them by name keeps the walk fast even when a
/// checkout has no `.gitignore` for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSkippedDirectory(entry: DirEntry): boolean { return entry.isDirectory() && SKIPPED.includes(entry.name); }
/// ```
fn is_skipped_directory(entry: &DirEntry) -> bool {
    let is_directory: bool = match entry.file_type() {
        Some(file_type) => file_type.is_dir(),
        None => false,
    };
    if !is_directory {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    for skipped in SKIPPED_DIRECTORIES {
        if name == skipped {
            return true;
        }
    }
    return false;
}

/// What:
/// `fn relative_path_text(root: &Path, path: &Path) -> String` makes the path under `root` as text,
/// with `/`
/// between segments.
/// Why:
/// The rule reports a path as text and reads the file name from its last segment;
/// a path relative to the workspace
/// root is what a person recognizes in the message.
/// Unix only,
/// so `/` is already the separator.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function relativePathText(root: string, path: string): string { return relative(root, path); }
/// ```
fn relative_path_text(root: &Path, path: &Path) -> String {
    // `strip_prefix` returns a `Result`; on `Err` (the path is not under the root) the full path is used.
    let relative: &Path = match path.strip_prefix(root) {
        Ok(stripped) => stripped,
        Err(_) => path,
    };
    return relative.to_string_lossy().to_string();
}

/// What:
/// `fn check_walk_result(root: &Path, result: Result<DirEntry, WalkError>) -> ScanReport` checks one walk result and
/// returns what it found as a small report of its own.
/// Why:
/// Each walker thread calls it without touching shared state,
/// so the lock is held only to merge the small result.
/// A file that cannot be read becomes a note,
/// never a failure:
/// the shadow's job is to advise,
/// and a scan problem
/// must not stop pnpm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkWalkResult(root: string, result: DirEntry | Error): ScanReport
/// ```
fn check_walk_result(root: &Path, result: Result<DirEntry, WalkError>) -> ScanReport {
    let mut report: ScanReport = ScanReport { manifests_checked: 0, findings: Vec::new(), notes: Vec::new() };
    let entry: DirEntry = match result {
        Ok(found) => found,
        Err(error) => {
            report.notes.push(format!("could not read part of the tree: {error}"));
            return report;
        }
    };
    let is_file: bool = match entry.file_type() {
        Some(file_type) => file_type.is_file(),
        None => false,
    };
    if !is_file {
        return report;
    }
    let name = entry.file_name().to_string_lossy().to_string();
    if !MANIFEST_NAMES.contains(&name.as_str()) {
        return report;
    }

    let relative: String = relative_path_text(root, entry.path());

    // Only `package.json` is read. The three other names are refused by name alone, and their bytes are never
    // opened (the rule's contract).
    if name != READ_MANIFEST_NAME {
        if let Some(finding) = check_manifest_path(&relative) {
            report.findings.push(finding);
        }
        return report;
    }
    match std::fs::read(entry.path()) {
        Ok(bytes) => {
            report.manifests_checked += 1;
            report.findings.extend(check_manifest(&relative, &bytes));
        }
        Err(error) => {
            report.notes.push(format!("could not read {relative}: {error}"));
        }
    }
    return report;
}

/// What:
/// `pub fn scan_workspace(root: &Path) -> ScanReport` walks the tree under `root` and checks every manifest.
/// Why:
/// One pass returns everything the warning needs.
/// The walk is parallel because it is the whole cost of the scan:
/// measured on this repository (about 22,500 entries,
/// 257 manifests),
/// the serial walk took a median of 79 ms and the
/// parallel walk a median of 10 ms,
/// alternating 30 rounds each (doc/handover/catalog-specifier-policy.md).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function scanWorkspace(root: string): ScanReport {
///   const report = { manifestsChecked: 0, findings: [], notes: [] };
///   await Promise.all(walk(root, { honorGitignore: true, skip: ['.git', 'node_modules'] }).map(entry => merge(report, checkEntry(entry))));
///   report.findings.sort(byPath); report.notes.sort();
///   return report;
/// }
/// ```
pub fn scan_workspace(root: &Path) -> ScanReport {
    // What:     `Mutex<ScanReport>` is a value only one thread may touch at a time (a TS value behind a lock; sibling:
    //           `RwLock`, which lets many readers in). Walker threads add to it by locking it for the short merge.
    // Why:      The walker calls the visitor from several threads, and they all append to one report.
    let shared: Mutex<ScanReport> =
        Mutex::new(ScanReport { manifests_checked: 0, findings: Vec::new(), notes: Vec::new() });

    // The builder starts at `root`. `hidden(false)` also visits dot-directories (a tracked `.config/package.json`
    // is still a manifest). `require_git(false)` honors `.gitignore` even when the workspace is not a git checkout.
    // `git_global(false)` ignores the user's own global ignore file, so two people see the same tree.
    let mut builder: WalkBuilder = WalkBuilder::new(root);
    builder.hidden(false);
    builder.require_git(false);
    builder.git_global(false);
    builder.filter_entry(|entry| { return !is_skipped_directory(entry); });

    // `run` calls the outer closure once per walker thread to make that thread's visitor. The visitor borrows
    // `root` and `shared`, which outlive the call, so no `Arc` is needed.
    builder.build_parallel().run(|| {
        return Box::new(|result| {
            let partial: ScanReport = check_walk_result(root, result);
            // A poisoned lock means another thread panicked while holding it; the report is still the best
            // available, so the guard is taken from the error.
            let mut guard = match shared.lock() {
                Ok(held) => held,
                Err(poisoned) => poisoned.into_inner(),
            };
            guard.manifests_checked += partial.manifests_checked;
            guard.findings.extend(partial.findings);
            guard.notes.extend(partial.notes);
            return WalkState::Continue;
        });
    });

    let mut report: ScanReport = match shared.into_inner() {
        Ok(inner) => inner,
        Err(poisoned) => poisoned.into_inner(),
    };
    // Threads finish in any order, so the report is sorted to be the same on every run. The sort is stable, so the
    // findings of one file keep the rule's own order.
    report.findings.sort_by(|left, right| { return left.path.cmp(&right.path); });
    report.notes.sort();
    return report;
}
