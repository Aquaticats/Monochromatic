//! What:
//! The `link` subcommand's work:
//! for every shadowed name whose REAL tool exists on PATH,
//! put a symlink with that name in one directory pointing at the `search-shadow` executable;
//! remove a link whose real tool has gone;
//! skip the rest.
//! Why:
//! Starting the binary under a tool's name is what switches it into shadow mode.
//! Linking only names that have a real tool keeps `command -v ag` honest:
//! a link for a tool that is not installed would make scripts believe it is
//! and then fail with "not found" when they run it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // link.ts
//! export function linkShadows(directory: string, executable: string, pathVariable: string): LinkReport { ... }
//! ```

/// Imports the file-system module
/// (create directories / remove files / resolve symlinks).
use std::fs;

/// What:
/// `use std::io;` imports the standard input/output module for `io::Result`.
/// Why:
/// The function reports failures with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Errors are thrown in TS.
/// ```
use std::io;

/// What:
/// `use std::os::unix::fs::symlink;` imports the function that creates a symbolic link.
/// Why:
/// The shadow links are symlinks to one binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { symlinkSync } from 'node:fs';
/// ```
use std::os::unix::fs::symlink;

/// Imports `Path` (borrowed) and `PathBuf` (owned) path types.
use std::path::{Path, PathBuf};

/// Imports the list of names that get a shadow link.
use crate::constants::SHADOWED_NAMES;

/// Imports the PATH lookup that finds the real tool.
use crate::real_binary::find_in_path;

/// What:
/// `pub struct LinkReport { ... }` declares a record with three lists:
/// the links made and the names skipped because no real tool exists and the stale links removed.
/// `Vec<PathBuf>` is an owned growable list of owned paths.
/// `#[derive(Debug, PartialEq)]` generates printing and `==` for the tests.
/// Why:
/// The `link` subcommand prints what happened and the tests check it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkReport = { linked: string[]; skipped: string[]; removed: string[] };
/// ```
#[derive(Debug, PartialEq)]
pub struct LinkReport {
    /// The links created or refreshed.
    pub linked: Vec<PathBuf>,
    /// The names with no real tool on PATH and no link to remove.
    pub skipped: Vec<String>,
    /// The links removed because their real tool is gone.
    pub removed: Vec<PathBuf>,
}

/// What:
/// `pub fn link_shadows(...) -> io::Result<LinkReport>` declares a public function.
/// `io::Result<LinkReport>` is a `Result` that is either `Ok(report)` or `Err(an I/O error)`.
/// Why:
/// `path_variable` is the PATH to search for real tools.
/// The shadow links themselves resolve to `executable` and are skipped by that search,
/// so running `link` twice gives the same answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function linkShadows(directory: string, executable: string, pathVariable: string): LinkReport { // throws on I/O errors
/// ```
pub fn link_shadows(directory: &Path, executable: &Path, path_variable: &str) -> io::Result<LinkReport> {
    // What:     `fs::create_dir_all(directory)?;` makes the directory and any missing parents. The trailing `?` means "if this
    // returned an error, stop and return that error to my caller; otherwise continue".
    // Why:      The links need somewhere to live.
    // Gotcha:   `?` is an early `return Err(...)`, not an exception: control leaves the function at that line.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // fs.mkdirSync(directory, { recursive: true }); // throws on failure
    // ```
    fs::create_dir_all(directory)?;

    // The resolved path of the executable is what the PATH search must skip; fall back to the given path if it cannot resolve.
    let mut skip: PathBuf = executable.to_path_buf();
    if let Ok(real) = fs::canonicalize(executable) {
        skip = real;
    }

    let mut report: LinkReport = LinkReport {
        linked: Vec::new(),
        skipped: Vec::new(),
        removed: Vec::new(),
    };
    for name in SHADOWED_NAMES {
        let link_path: PathBuf = directory.join(name);
        let has_real_tool: bool = find_in_path(name, path_variable, &skip).is_some();

        // `symlink_metadata` looks at the link itself, not its target, so a dangling link still counts as present.
        let link_exists: bool = link_path.symlink_metadata().is_ok();

        if has_real_tool {
            if link_exists {
                fs::remove_file(&link_path)?;
            }
            symlink(executable, &link_path)?;
            report.linked.push(link_path);
        } else if link_exists {
            fs::remove_file(&link_path)?;
            report.removed.push(link_path);
        } else {
            report.skipped.push((*name).to_string());
        }
    }

    // What:     `Ok(report)` wraps the record in the success case of `Result`.
    // Why:      Hand what happened to the caller and signal "no error".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return report;
    // ```
    return Ok(report);
}
