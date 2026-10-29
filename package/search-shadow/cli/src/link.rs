//! What:     The `link` subcommand's work: put a symlink named `rg`, `grep` and `find` in one directory, each pointing at
//!           the `search-shadow` executable.
//! Why:      Starting the binary under those names is what switches it into shadow mode; a directory of such links,
//!           placed first on PATH, shadows the real tools.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // link.ts
//! export function linkShadows(directory: string, executable: string): string[] { ... }
//! ```

/// Imports the file-system module (create directories, remove files).
use std::fs;

/// What:     `use std::io;` imports the standard input/output module, for `io::Result`.
/// Why:      The function reports failures with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Errors are thrown in TS.
/// ```
use std::io;

/// What:     `use std::os::unix::fs::symlink;` imports the function that creates a symbolic link.
/// Why:      The shadow links are symlinks to one binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { symlinkSync } from 'node:fs';
/// ```
use std::os::unix::fs::symlink;

/// Imports `Path` (borrowed) and `PathBuf` (owned) path types.
use std::path::{Path, PathBuf};

use crate::constants::SHADOWED_NAMES;

/// What:     `pub fn link_shadows(directory: &Path, executable: &Path) -> io::Result<Vec<PathBuf>>` declares a public
///           function. `io::Result<Vec<PathBuf>>` is a `Result` that is either `Ok(list of created links)` or
///           `Err(an I/O error)`; `Vec<PathBuf>` is an owned growable list (sibling: `&[PathBuf]` is a borrowed view).
/// Why:      The caller prints what was created. `Vec` and not a borrow because the list is built here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function linkShadows(directory: string, executable: string): string[] { // throws on I/O errors
/// ```
pub fn link_shadows(directory: &Path, executable: &Path) -> io::Result<Vec<PathBuf>> {
    // What:     `fs::create_dir_all(directory)?;` makes the directory and any missing parents. The trailing `?` means
    //           "if this returned an error, stop and return that error to my caller; otherwise continue".
    // Why:      The links need somewhere to live.
    // Gotcha:   `?` is an early `return Err(...)`, not an exception: control leaves the function at that line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fs.mkdirSync(directory, { recursive: true }); // throws on failure
    // ```
    fs::create_dir_all(directory)?;

    let mut created: Vec<PathBuf> = Vec::new();
    for name in SHADOWED_NAMES {
        let link_path: PathBuf = directory.join(name);

        // `symlink_metadata` looks at the link itself, not its target, so a dangling link still counts as present.
        if link_path.symlink_metadata().is_ok() {
            fs::remove_file(&link_path)?;
        }
        symlink(executable, &link_path)?;
        created.push(link_path);
    }

    // What:     `Ok(created)` wraps the list in the success case of `Result`.
    // Why:      Hand the created links to the caller and signal "no error".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return created;
    // ```
    return Ok(created);
}
