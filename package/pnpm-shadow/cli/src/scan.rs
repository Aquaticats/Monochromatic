//! What:
//! Finds the root of the pnpm workspace that contains the current directory and checks every manifest under it.
//! Why:
//! The shadow warns before each pnpm run, so it needs the whole tree's findings in one pass. The rule itself lives
//! in `monochromatic-catalog-specifier`; this file only walks and reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // scan.ts
//! export function findWorkspaceRoot(start: string): string | undefined { ... }
//! export function scanWorkspace(root: string): ScanReport { ... }
//! ```

/// Imports the walker that honors `.gitignore`, and its entry type.
use ignore::{DirEntry, WalkBuilder};

/// Imports the rule: one function for any manifest name, one for a name alone, and the finding type.
use monochromatic_catalog_specifier::{Finding, check_manifest, check_manifest_path};

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// Imports the fixed values.
use crate::constants::{MANIFEST_NAMES, READ_MANIFEST_NAME, SKIPPED_DIRECTORIES, WORKSPACE_MARKER};

/// What:
/// `pub struct ScanReport { ... }` is a record type with three fields, all owned by the record: a count
/// (`usize`, the unsigned integer sized for counts and indexes; siblings `u32`, `u64`), a list of findings
/// (`Vec<Finding>`, a growable list; siblings `&[Finding]`, `[Finding; N]`) and a list of notes (`Vec<String>`,
/// owned text; sibling `&str`).
/// Why:
/// The caller prints findings and notes alike, and the count says how many `package.json` files were really read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type ScanReport = { manifestsChecked: number; findings: Finding[]; notes: string[] };
/// ```
pub struct ScanReport {
    /// How many `package.json` files were read and checked.
    pub manifests_checked: usize,
    /// Every finding, in walk order (sorted by path).
    pub findings: Vec<Finding>,
    /// Problems that kept a file from being checked at all (an unreadable file or directory).
    pub notes: Vec<String>,
}

/// What:
/// `pub fn find_workspace_root(start: &Path) -> Option<PathBuf>` walks from `start` up to the filesystem root and
/// returns the first directory that holds `pnpm-workspace.yaml`.
/// Why:
/// pnpm itself finds its workspace this way, and the shadow checks the tree pnpm would use. Only the file's
/// presence is tested: the YAML is never read.
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
/// `.git` and `node_modules` are never tracked sources; skipping them by name keeps the walk fast even when a
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
/// `fn relative_path_text(root: &Path, path: &Path) -> String` makes the path under `root` as text, with `/`
/// between segments.
/// Why:
/// The rule reports a path as text and reads the file name from its last segment; a path relative to the workspace
/// root is what a person recognizes in the message. Unix only, so `/` is already the separator.
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
/// `pub fn scan_workspace(root: &Path) -> ScanReport` walks the tree under `root` and checks every manifest.
/// Why:
/// One pass returns everything the warning needs. A file that cannot be read becomes a note, never a failure:
/// the shadow's job is to advise, and a scan problem must not stop pnpm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function scanWorkspace(root: string): ScanReport {
///   const report = { manifestsChecked: 0, findings: [], notes: [] };
///   for (const entry of walk(root, { honorGitignore: true, skip: ['.git', 'node_modules'] })) { ... }
///   return report;
/// }
/// ```
pub fn scan_workspace(root: &Path) -> ScanReport {
    let mut report: ScanReport = ScanReport { manifests_checked: 0, findings: Vec::new(), notes: Vec::new() };

    // The builder starts at `root`. `hidden(false)` also visits dot-directories (a tracked `.config/package.json`
    // is still a manifest). `require_git(false)` honors `.gitignore` even when the workspace is not a git checkout.
    // `git_global(false)` ignores the user's own global ignore file, so two people see the same tree.
    // Sorting by path makes the report order the same on every run.
    let mut builder: WalkBuilder = WalkBuilder::new(root);
    builder.hidden(false);
    builder.require_git(false);
    builder.git_global(false);
    builder.sort_by_file_path(|left, right| { return left.cmp(right); });
    builder.filter_entry(|entry| { return !is_skipped_directory(entry); });

    for result in builder.build() {
        let entry: DirEntry = match result {
            Ok(found) => found,
            Err(error) => {
                report.notes.push(format!("could not read part of the tree: {error}"));
                continue;
            }
        };
        let is_file: bool = match entry.file_type() {
            Some(file_type) => file_type.is_file(),
            None => false,
        };
        if !is_file {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !MANIFEST_NAMES.contains(&name.as_str()) {
            continue;
        }

        let relative: String = relative_path_text(root, entry.path());

        // Only `package.json` is read. The three other names are refused by name alone, and their bytes are never
        // opened (the rule's contract).
        if name != READ_MANIFEST_NAME {
            if let Some(finding) = check_manifest_path(&relative) {
                report.findings.push(finding);
            }
            continue;
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
    }
    return report;
}
