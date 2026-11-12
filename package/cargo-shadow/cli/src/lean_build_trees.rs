//! What:
//! The trees of a host Cargo build base:
//! how to list them in both layouts,
//! find the profile directories inside one,
//! tell when anything in one was last written,
//! and tell which workspace it was built for.
//! Why:
//! Mechanism C collects whole trees whose workspace is gone and that have been idle,
//! and each of those facts comes from here.
//! A tree has a ledger file when the shadow keyed it (`lean_keyed_folder.rs`).
//! A tree from before that has none,
//! so a lead is read from the first dependency-info file that names a source outside the registry,
//! git and toolchain directories:
//! a lead is not ownership,
//! and such a tree is judged on idleness as well.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function listTrees(base: string): Tree[]
//! ```

/// Files and directories.
use std::{fs, os::unix::fs::MetadataExt, path::{Path, PathBuf}};

/// The ledger and the keyed directory name.
use crate::lean_keyed_folder::{KEYED_DIRECTORY, read_ledger};
/// The deadline of walks and the lock file name.
use crate::lean_identity::Deadline;
/// Imports the names this file uses from `crate::lean_lock_probe`.
use crate::lean_lock_probe::LOCK_FILE_NAMES;
/// Named reads of paths and text.
use crate::lean_text::{all_hex, extension_is, is_directory, name_of, read_small_bytes};

/// Directory names inside a profile directory,
/// never entered when looking for profile directories.
const INSIDE_PROFILE: [&str; 5] = ["build", "deps", "incremental", ".fingerprint", "examples"];

/// How deep below a tree its profile directories can be (`<triple>/debug`,
/// `dylint/debug`).
const PROFILE_DEPTH: usize = 3;

/// How many dependency-info files are read for a lead.
const LEAD_FILES: usize = 300;

/// How many bytes of a dependency-info file are read.
const LEAD_BYTES: usize = 4_096;

/// Path fragments of sources that are not a workspace.
const NOT_A_WORKSPACE: [&str; 5] = ["/.cargo/registry/", "/.cargo/git/", "/rustlib/", "/.rustup/", "/toolchains/"];

/// Which layout a tree is in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeKind {
    /// `<base>/<xx>/<hex>`,
    /// made by Cargo's own template.
    Legacy,
    /// `<base>/toolchain/<toolchain>/<xx>/<hex>`,
    /// made by the shadow.
    Keyed(
        /// The toolchain name.
        String,
    ),
}

/// One tree of the base.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tree {
    /// Where it is.
    pub path: PathBuf,
    /// Its layout.
    pub kind: TreeKind,
}

/// The workspace a tree was built for,
/// as far as is known.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Workspace {
    /// Named by the ledger the shadow left.
    Known(
        /// The workspace root.
        PathBuf,
    ),
    /// A source file the tree's dependency information names,
    /// which is a lead and not proof.
    Lead(
        /// The source file the dependency information names.
        PathBuf,
    ),
    /// Nothing identifies it.
    Unidentified,
}

/// What:
/// `fn directories_of(path: &Path) -> Vec<PathBuf>` lists the subdirectories of a directory,
/// sorted.
/// Why:
/// The listing of both layouts uses it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const directoriesOf = (path: string): string[] => readdirSync(path, { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => join(path, e.name)).sort();
/// ```
fn directories_of(path: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if is_directory(&entry) {
                found.push(entry.path());
            }
        }
    }
    found.sort();
    return found;
}

/// What:
/// `pub fn list_trees(base: &Path, deadline: &Deadline) -> Option<Vec<Tree>>` lists every tree under a build base in both
/// layouts,
/// and answers nothing when the deadline passed before the listing finished.
/// Why:
/// The shard directories (two characters) and the keyed directory are the only children it reads;
/// the trash directory and anything else is not a tree.
/// A listing of a loaded disk is not free:
/// it reads a directory per shard and one per shard's children,
/// and the deadline has to reach between them,
/// or a pass spends its bound finding out what it would have judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function listTrees(base: string, deadline: Deadline): Tree[] | undefined
/// ```
pub fn list_trees(base: &Path, deadline: &Deadline) -> Option<Vec<Tree>> {
    let mut trees: Vec<Tree> = Vec::new();
    for child in directories_of(base) {
        if deadline.passed() {
            return None;
        }
        let name: String = name_of(&child);
        if name == KEYED_DIRECTORY {
            for toolchain in directories_of(&child) {
                let toolchain_name: String = name_of(&toolchain);
                for shard in directories_of(&toolchain) {
                    for tree in directories_of(&shard) {
                        if deadline.passed() {
                            return None;
                        }
                        trees.push(Tree { path: tree, kind: TreeKind::Keyed(toolchain_name.clone()) });
                    }
                }
            }
        } else if name.len() == 2 && all_hex(&name) {
            for tree in directories_of(&child) {
                if deadline.passed() {
                    return None;
                }
                trees.push(Tree { path: tree, kind: TreeKind::Legacy });
            }
        }
    }
    return Some(trees);
}

/// What:
/// `pub fn profile_dirs_of(tree: &Path, deadline: &Deadline) -> Option<Vec<PathBuf>>` finds the profile directories of a
/// tree:
/// the directories that hold a Cargo lock file,
/// and answers nothing when the deadline passed before the walk finished.
/// Why:
/// They are what the locks are taken on.
/// The walk stops at a profile directory and never enters its working directories.
/// A tree reaches four levels deep with a directory per triple and per build script,
/// so the walk is a chain of reads under the one read they share,
/// and a pass that cannot finish the walk of one tree has to leave that tree for the next pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function profileDirsOf(tree: string, deadline: Deadline): string[] | undefined
/// ```
pub fn profile_dirs_of(tree: &Path, deadline: &Deadline) -> Option<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(tree.to_path_buf(), 0)];
    while let Some((directory, depth)) = stack.pop() {
        if deadline.passed() {
            return None;
        }
        let mut has_lock: bool = false;
        for lock_name in LOCK_FILE_NAMES {
            if directory.join(lock_name).exists() {
                has_lock = true;
            }
        }
        if has_lock && depth > 0 {
            found.push(directory);
            continue;
        }
        if depth >= PROFILE_DEPTH {
            continue;
        }
        for child in directories_of(&directory) {
            let name: String = name_of(&child);
            if depth > 0 && INSIDE_PROFILE.contains(&name.as_str()) {
                continue;
            }
            stack.push((child, depth + 1));
        }
    }
    found.sort();
    return Some(found);
}

/// What:
/// `fn newest_in(directory: &Path, newest: &mut i64)` raises `newest` to the modification time of the directory and of every
/// entry one level down.
/// Why:
/// A unit's `invoked.timestamp` and a new unit's directory carry the time of the last build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newestIn(directory: string, newest: { value: number }): void
/// ```
fn newest_in(directory: &Path, newest: &mut i64) {
    if let Ok(metadata) = fs::symlink_metadata(directory) {
        *newest = (*newest).max(metadata.mtime());
    }
    if let Ok(entries) = fs::read_dir(directory) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                *newest = (*newest).max(metadata.mtime());
            }
        }
    }
}

/// What:
/// `pub fn last_activity(tree: &Path, deadline: &Deadline) -> Option<i64>` is the newest write time found in the tree:
/// its directories,
/// its profile directories' working directories,
/// and the `invoked.timestamp` of every unit.
/// Why:
/// Idleness is "no write in any profile directory".
/// `None` means the deadline passed before the answer was known.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lastActivity(tree: string, deadline: Deadline): number | undefined
/// ```
pub fn last_activity(tree: &Path, deadline: &Deadline) -> Option<i64> {
    let mut newest: i64 = 0;
    newest_in(tree, &mut newest);
    for profile in profile_dirs_of(tree, deadline)? {
        newest_in(&profile, &mut newest);
        for name in INSIDE_PROFILE {
            newest_in(&profile.join(name), &mut newest);
        }
        for fingerprints in [profile.join(".fingerprint")] {
            for unit in directories_of(&fingerprints) {
                if deadline.passed() {
                    return None;
                }
                newest_in(&unit, &mut newest);
            }
        }
        for package in directories_of(&profile.join("build")) {
            if deadline.passed() {
                return None;
            }
            for unit in directories_of(&package) {
                newest_in(&unit.join("fingerprint"), &mut newest);
            }
        }
    }
    return Some(newest);
}

/// What:
/// `fn lead_in(tree: &Path, dependency_file: &Path) -> Option<PathBuf>` is the first source path a dependency-info file names
/// outside the tree,
/// the registry,
/// git and the toolchains.
/// Why:
/// It is a lead to the workspace the tree was built for,
/// and the read of the file waits on nothing:
/// a dependency-information file is a name inside a build tree,
/// so a pipe or a device at it must not hold the pass,
/// and the first bytes are all a lead is read from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function leadIn(tree: string, dependencyFile: string): string | undefined
/// ```
fn lead_in(tree: &Path, dependency_file: &Path) -> Option<PathBuf> {
    let buffer: Vec<u8> = read_small_bytes(dependency_file, LEAD_BYTES).ok()?;
    let count: usize = buffer.len();
    let read: String = String::from_utf8_lossy(&buffer).to_string();
    // A read that filled the buffer may have cut the last token in two:
    // a cut path names nothing and would look gone,
    // so the text is taken only up to the last whitespace.
    let text: &str = if count == LEAD_BYTES {
        match read.rfind(char::is_whitespace) {
            Some(end) => &read[..end],
            None => "",
        }
    } else {
        &read
    };
    for token in text.split_whitespace().skip(1) {
        if !token.starts_with('/') || Path::new(token).starts_with(tree) {
            continue;
        }
        let mut excluded: bool = false;
        for fragment in NOT_A_WORKSPACE {
            if token.contains(fragment) {
                excluded = true;
            }
        }
        if excluded {
            continue;
        }
        return Some(PathBuf::from(token));
    }
    return None;
}

/// What:
/// `pub fn workspace_of(tree: &Path, deadline: &Deadline) -> Option<Workspace>` says which workspace a tree was built for,
/// and answers nothing when the deadline passed before the answer was known.
/// Why:
/// The ledger is authoritative;
/// the dependency information is a lead;
/// otherwise nothing identifies the tree.
/// The walk for a lead is hundreds of reads in a build tree,
/// so a pass whose bound is spent spends it here,
/// and a tree it did not finish reading is a tree the next pass judges first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function workspaceOf(tree: string, deadline: Deadline): Workspace | undefined
/// ```
pub fn workspace_of(tree: &Path, deadline: &Deadline) -> Option<Workspace> {
    if let Some(root) = read_ledger(tree) {
        return Some(Workspace::Known(root));
    }
    let mut files: Vec<PathBuf> = Vec::new();
    for profile in profile_dirs_of(tree, deadline)? {
        if deadline.passed() {
            return None;
        }
        for package in directories_of(&profile.join("build")) {
            for unit in directories_of(&package) {
                if let Ok(entries) = fs::read_dir(unit.join("out")) {
                    for entry in entries.flatten() {
                        if extension_is(&entry.path(), "d") {
                            files.push(entry.path());
                        }
                    }
                }
            }
            if files.len() >= LEAD_FILES {
                break;
            }
        }
        if let Ok(entries) = fs::read_dir(profile.join("deps")) {
            for entry in entries.flatten().take(LEAD_FILES) {
                if extension_is(&entry.path(), "d") {
                    files.push(entry.path());
                }
            }
        }
    }
    for file in files.iter().take(LEAD_FILES) {
        if deadline.passed() {
            return None;
        }
        if let Some(lead) = lead_in(tree, file) {
            return Some(Workspace::Lead(lead));
        }
    }
    return Some(Workspace::Unidentified);
}
