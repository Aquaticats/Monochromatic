//! What:
//! The regular files below a folder of this crate's source tree,
//! each one visited once and never read through a symbolic link.
//! Why:
//! A source folder of this repository may hold a `uses/` folder whose entries are links to the folders its code uses
//! (`AGENTS.md` rules `LYD`, `LYL` and `LYF`),
//! so a walk that followed a link would reach the same source once per chain of links above it,
//! and a guard that reads the sources would read one file many times and could read a file that is not part of the tree.
//! Measured for the wrapper package on 2026-11-27 (issue 6580):
//! a recursive read of its `src/native` returned 165,978 entries and 163,498 `.rs` names,
//! where the tree holds 4,234 entries and 3,905 Rust sources.
//! The walk that package reads its sources through is `package/git-policy/cli/bin/source-tree-files.ts`,
//! and this is the same walk for this crate.
//! Gotcha:
//! This crate's `src` holds no link on 2026-11-27 (`find src -type l` found none),
//! so the rule is held by the control `a_folder_link_is_never_entered_and_a_file_link_is_never_read`
//! (`src/source_tree_files_tests.rs`) instead of by anything in the tree.
//! This module is compiled into a test build only (`#[cfg(test)]` in `src/lib.rs`).

/// What:
/// `OsString` is an owned name whose bytes need not be valid text.
/// The sibling a reader might expect is `String`, which cannot name a file on a file system that takes other bytes.
/// Why:
/// The walk hands each entry's own name to the path it builds unchanged.
use std::ffi::OsString;
/// What:
/// `fs` is the file system's own functions.
/// The sibling a reader might expect is `std::path`, which names a path but never reads one.
/// Why:
/// The walk lists a folder and asks each entry what it is.
use std::fs;
/// What:
/// `io` is the error type and the `Result` alias of a file-system call.
/// Why:
/// A folder that cannot be read is the caller's to answer, never silence.
use std::io;
/// What:
/// `Path` is a borrowed file-system path and `PathBuf` an owned one.
/// Why:
/// The folder is lent to this module, and every path it answers is owned here.
use std::path::{
    Path,
    PathBuf,
};

/// What:
/// `fn source_tree_files(folder: &Path) -> io::Result<Vec<PathBuf>>` is every regular file below one folder,
/// named relative to that folder and sorted by name.
/// Why:
/// A guard that reads the sources asks this,
/// so it reads what the tree holds rather than a list a person keeps.
/// `Vec<PathBuf>` and not `Vec<String>`:
/// a name that is not valid text is still a path, and text would lose it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourceTreeFiles(folder: string): string[]  // throws when a folder cannot be read
/// ```
pub(crate) fn source_tree_files(folder: &Path) -> io::Result<Vec<PathBuf>> {
    // What: an owned copy of the borrowed root, joined with the empty path that makes the answers relative to it.
    let start: (PathBuf, PathBuf) = (folder.to_path_buf(), PathBuf::new());
    // What: the folders still to list, each with the path the answers below it will carry.
    // Why: a stack of folders, not a recursive call, so a tree whose depth is not the caller's to choose still ends.
    let mut pending: Vec<(PathBuf, PathBuf)> = vec![start];
    // What: the regular files found so far, as paths relative to the folder the caller named.
    let mut found: Vec<PathBuf> = Vec::new();
    // What: `pending.pop()` takes the last folder pushed, so this is a depth-first walk over the tree.
    while let Some((directory, relative)) = pending.pop() {
        // What: `fs::read_dir(&directory)?` lists one folder.
        // Why: `?` hands a failure to the caller instead of reading an unreadable folder as an empty one.
        let entries: fs::ReadDir = fs::read_dir(&directory)?;
        // What: `listed` is one entry of the folder, in the order the file system stores it.
        for listed in entries {
            // What: `listed?` is the entry's own error, because a listing can fail halfway through.
            let entry: fs::DirEntry = listed?;
            // What: `file_type()` is the entry's own type and not what a link points at, so a link answers `is_symlink`.
            let kind: fs::FileType = entry.file_type()?;
            // What: the entry's name alone, with no folder in it.
            let name: OsString = entry.file_name();
            // What: `relative.join(&name)` is the entry's path under the folder the caller named.
            let path: PathBuf = relative.join(&name);
            if kind.is_dir() {
                // What: `entry.path()` is the real folder's path on disk, which the next round lists.
                pending.push((entry.path(), path));
            } else if kind.is_file() {
                // What: a regular file is what the walk answers.
                found.push(path);
            }
            // A link, a socket, a fifo or a device is not a source file:
            // it is never entered and never answered, so a link to a folder inside the tree reads nothing twice.
        }
    }
    // What: `found.sort()` puts the names in one order.
    // Why: a directory's own order is the file system's choice and differs between machines.
    found.sort();
    // What: `Ok(found)` is the success variant of `Result`, holding the files; with no trailing `;` it is the return value.
    // Why: the caller reads the listing or the error, and never confuses one for the other.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return files;
    // ```
    return Ok(found);
}
