//! Find the project folders whose changes the language servers hear about,
//!  with the search's ignore rules.
//!
//! ripgrep lists the files the search would see (`src/search_process.rs`):
//!  it honours `.gitignore`,
//!  `.ignore`,
//! `.rgignore`,
//!  and skips hidden names.
//!  Every folder holding such a file is a source folder.
//!  `node_modules`,
//! `target`,
//!  and `.git` are never watched,
//!  even where no ignore file names them.
//!
//! Two properties of ripgrep decide how folders are rescanned (probed with ripgrep 15.2.0):
//!  a directory
//! given on the command line is listed even when an ignore rule names it,
//!  but ignore files of its parents
//! still apply to everything below it.
//!  So a new folder is classified by scanning its parent again,
//!  never by
//! scanning the folder itself.
//!  A folder with no entries at all cannot be classified by a file list;
//!  it is
//! watched while empty (provisionally),
//!  and its first change asks for its parent to be scanned again.

/// The ripgrep settings the search uses,
///  so both apply the same ignore rules.
use crate::search_process::RIPGREP_SETTINGS;
/// What:
///  `BTreeSet` is an ordered set of owned paths;
///  `OsStr` is a native filename slice;
///  `OsStrExt` turns
///       raw bytes into one on Unix;
///  `Command`/`Stdio` start a child process with chosen pipes.
/// Why:
///  ripgrep separates paths with NUL bytes,
///  and paths keep their native bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawnSync } from 'node:child_process';
/// ```
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// Folder names never watched for the servers:
///  dependencies,
///  build output,
///  and version-control data.
pub(super) const PRUNED: [&str; 3] = ["node_modules", "target", ".git"];

/// What one scan of a folder found.
#[derive(Debug, Default)]
pub(super) struct Scan {
    /// The folder scanned;
    ///  everything the scan reports lies inside it.
    pub(super) base: PathBuf,
    /// Folders holding a file ripgrep lists,
    ///  the base included.
    pub(super) directories: BTreeSet<PathBuf>,
    /// Folders with no entries,
    ///  inside a source folder,
    ///  not hidden and not pruned.
    pub(super) provisional: BTreeSet<PathBuf>,
    /// The files ripgrep listed,
    ///  for the creations a newly watched folder may have missed.
    pub(super) files: Vec<PathBuf>,
    /// The scan that starts watching for the servers:
    ///  its files existed before,
    ///  so none is reported.
    pub(super) initial: bool,
    /// Which period of watching for the servers asked for it;
    ///  a scan from before the feed was cleared is dropped.
    pub(super) generation: u64,
}

/// True when the directory entry is a folder itself;
///  the file type does not follow symbolic links.
fn is_directory_entry(entry: &fs::DirEntry) -> bool {
    return match entry.file_type() {
        Ok(kind) => kind.is_dir(),
        Err(_) => false,
    };
}

/// True for a name the search skips (hidden) or that is never watched.
fn skipped_name(name: &OsStr) -> bool {
    // What: `as_bytes` views the native name as raw bytes; `first` is its first byte, if any.
    // Why: ripgrep's hidden rule is a leading dot, which is a byte test on Unix.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return name.startsWith('.') || PRUNED.includes(name);
    // ```
    let hidden = name.as_bytes().first() == Some(&b'.');
    let mut pruned = false;
    for candidate in PRUNED.iter() {
        if name == *candidate {
            pruned = true;
            break;
        }
    }
    return hidden || pruned;
}

/// True when some folder between `base` and `file` is never watched,
///  so the file does not count.
fn under_pruned(base: &Path, file: &Path) -> bool {
    // What: `strip_prefix` gives the path relative to `base`, or an error when it is not inside it.
    // Why: Only the folders below the base decide; the base itself was chosen as a source folder.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const relative = path.relative(base, file);
    // ```
    let Ok(relative) = file.strip_prefix(base) else {
        return true;
    };
    // The last component is the file name itself; a file named `target` still counts.
    let Some(parent) = relative.parent() else {
        return false;
    };
    for component in parent.components() {
        for pruned in PRUNED.iter() {
            if component.as_os_str() == *pruned {
                return true;
            }
        }
    }
    return false;
}

/// What:
///  List the files below `base` exactly as the search would,
///  NUL-separated,
///  or `None` when ripgrep
///       could not run.
///  `Option<Vec<PathBuf>>` is a list or nothing.
/// Why:
///  Running the same program with the same settings is the only way to agree with the search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listFiles(root: string, base: string): string[] | undefined
/// ```
fn list_files(root: &Path, base: &Path) -> Option<Vec<PathBuf>> {
    let mut command = Command::new("rg");
    command
        .current_dir(root)
        .env_remove("RIPGREP_CONFIG_PATH")
        .args(RIPGREP_SETTINGS)
        .args(["--files", "--null", "--"])
        .arg(base)
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    // What: `output()` runs the program to its end and collects standard output; `match` separates a
    //       program that ran from one that could not start.
    // Why: Exit status 1 means "no files" and 2 means "some paths were unreadable"; both still list
    //      every readable file, so only a failure to start is a failure here.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let output; try { output = spawnSync('rg', args, { cwd: root }); } catch (error) { return undefined; }
    // ```
    let output = match command.output() {
        Ok(output) => output,
        Err(error) => {
            tracing::warn!(%error, "cannot run ripgrep to list the project for the language servers' file watching");
            return None;
        }
    };
    tracing::debug!(base = %base.display(), status = ?output.status, bytes = output.stdout.len(), "ripgrep listed a folder for the language servers");
    // What: `split` cuts the bytes at every NUL; `filter` drops the empty piece after the last one;
    //       `OsStr::from_bytes` keeps each path's native bytes; `collect` gathers the paths into a list.
    // Why: Paths are not necessarily UTF-8.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return output.stdout.split('\0').filter(Boolean);
    // ```
    let mut files: Vec<PathBuf> = Vec::new();
    let mut start = 0;
    for (index, byte) in output.stdout.iter().enumerate() {
        if *byte == 0 {
            if index > start {
                files.push(PathBuf::from(OsStr::from_bytes(
                    &output.stdout[start..index],
                )));
            }
            start = index + 1;
        }
    }
    if start < output.stdout.len() {
        files.push(PathBuf::from(OsStr::from_bytes(&output.stdout[start..])));
    }
    return Some(files);
}

/// Every folder below `base` that is neither hidden nor pruned,
///  as the source folders when ripgrep
/// cannot run:
///  no ignore file is honoured then,
///  which may watch more than the search shows.
fn walk_all(base: &Path) -> BTreeSet<PathBuf> {
    let mut found = BTreeSet::new();
    // What: `vec![...]` makes a list used as a stack of folders still to read.
    // Why: An explicit stack walks deep trees without recursion.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pending = [base];
    // ```
    let mut pending = vec![base.to_path_buf()];
    while let Some(directory) = pending.pop() {
        // `read_dir` fails for a folder that vanished or cannot be read; it is then skipped.
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        found.insert(directory);
        // `flatten` skips entries that could not be read.
        for entry in entries.flatten() {
            // `file_type` does not follow symbolic links, so a link to a folder is not entered.
            let is_directory = is_directory_entry(&entry);
            if is_directory && !skipped_name(&entry.file_name()) {
                pending.push(entry.path());
            }
        }
    }
    return found;
}

/// True when `directory` exists and holds no entries at all.
fn is_empty_directory(directory: &Path) -> bool {
    // What: `next()` reads the first entry; `is_none` is true when there is none.
    // Why: One entry is enough to know the folder is not empty.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return readdirSync(directory).length === 0;
    // ```
    return match fs::read_dir(directory) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    };
}

/// Empty folders directly inside the source folders:
///  no file list can classify them,
///  so they are
/// watched until their first change.
fn empty_children(directories: &BTreeSet<PathBuf>) -> BTreeSet<PathBuf> {
    let mut empty = BTreeSet::new();
    for directory in directories {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_directory = is_directory_entry(&entry);
            if !is_directory || skipped_name(&entry.file_name()) || directories.contains(&path) {
                continue;
            }
            if is_empty_directory(&path) {
                empty.insert(path);
            }
        }
    }
    return empty;
}

/// What:
///  Scan one folder:
///  its source folders,
///  its empty folders,
///  and its files.
///  `initial` marks the
///       scan that starts watching,
///  whose files are not reported as created.
/// Why:
///  The watch thread replaces everything it knew below `base` with this result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function scan(root: string, base: string, initial: boolean): Scan
/// ```
pub(super) fn scan(root: &Path, base: &Path, initial: bool) -> Scan {
    let mut result = Scan {
        base: base.to_path_buf(),
        initial,
        ..Scan::default()
    };
    if !base.is_dir() {
        tracing::debug!(base = %base.display(), "a folder to scan for the language servers is gone");
        return result;
    }
    // `Some(files)` is the listing; `None` falls back to a walk without ignore rules.
    if let Some(files) = list_files(root, base) {
        result.directories.insert(base.to_path_buf());
        for file in files {
            if under_pruned(base, &file) {
                continue;
            }
            // Every folder from the file's own up to the base holds a listed file.
            let mut folder = file.parent();
            while let Some(directory) = folder {
                if !directory.starts_with(base) || result.directories.contains(directory) {
                    break;
                }
                result.directories.insert(directory.to_path_buf());
                folder = directory.parent();
            }
            result.files.push(file);
        }
    } else {
        result.directories = walk_all(base);
    }
    result.provisional = empty_children(&result.directories);
    tracing::debug!(
        base = %base.display(),
        directories = result.directories.len(),
        provisional = result.provisional.len(),
        files = result.files.len(),
        "scanned a folder for the language servers"
    );
    return result;
}

/// What:
///  The folders to scan for a batch of requests:
///  each path that is now a folder (not hidden,
///  not
///       pruned,
///  not a symbolic link) asks for its parent,
///  and each explicit rescan asks for itself.
///       Folders inside another requested folder are dropped,
///  since that scan covers them.
/// Why:
///  A new folder is classified by its parent's ignore rules,
///  which a scan of the folder itself skips.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bases(root: string, candidates: Set<string>, rescans: Set<string>): string[]
/// ```
pub(super) fn bases(
    root: &Path,
    candidates: &BTreeSet<PathBuf>,
    rescans: &BTreeSet<PathBuf>,
) -> Vec<PathBuf> {
    let mut wanted = BTreeSet::new();
    for candidate in candidates {
        // `symlink_metadata` describes the link itself, so a link to a folder is not followed.
        let is_directory = match fs::symlink_metadata(candidate) {
            Ok(metadata) => metadata.is_dir(),
            Err(_) => false,
        };
        let named = match candidate.file_name() {
            Some(name) => !skipped_name(name),
            None => false,
        };
        if is_directory
            && named
            && let Some(parent) = candidate.parent()
            && parent.starts_with(root)
        {
            wanted.insert(parent.to_path_buf());
        }
    }
    for rescan in rescans {
        if rescan.starts_with(root) {
            wanted.insert(rescan.clone());
        }
    }
    // Sorted order puts a folder before everything inside it, so one pass drops the covered ones.
    let mut kept: Vec<PathBuf> = Vec::new();
    for base in wanted {
        let mut covered = false;
        for outer in kept.iter() {
            if base.starts_with(outer) {
                covered = true;
                break;
            }
        }
        if covered {
            continue;
        }
        kept.push(base);
    }
    return kept;
}

/// The folder walk,
///  the classification of new folders,
///  and the ignore rules,
///  on disposable projects.
#[cfg(test)]
#[path = "server_scan_tests.rs"]
mod tests;
