//! What:
//! The `link` subcommand's work:
//! for every shadowed name whose REAL tool exists on PATH,
//! put a symlink with that name in one directory pointing at the `search-shadow` executable;
//! remove a shadow link whose real tool has gone;
//! skip the rest,
//! and leave an entry that is not a shadow link where it is.
//! Why:
//! Starting the binary under a tool's name is what switches it into shadow mode.
//! Linking only names that have a real tool keeps `command -v ag` honest:
//! a link for a tool that is not installed would make scripts believe it is
//! and then fail with "not found" when they run it.
//! The directory is any directory the user names
//! (`~/.local/bin` as readily as the suggested `.cache/search-shadow-bin`),
//! so an entry there that is not this program's own link is somebody else's file
//! and is never deleted.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // link.ts
//! export function linkShadows(directory: string, executable: string, pathVariable: string): LinkReport { ... }
//! ```

/// Imports the file-system module
/// (create directories / remove files / resolve symlinks).
use std::fs;

/// Imports the borrowed operating-system text type,
/// which names a file whose bytes need not be valid UTF-8
/// (its sibling `String` only holds valid UTF-8).
use std::ffi::OsStr;

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

/// Imports the identity check that recognizes a shadow of any version behind a link.
use crate::shadow_identity::shadow_behind;

/// What:
/// `pub struct LinkReport { ... }` declares a record with four lists:
/// the links made,
/// the names skipped because no real tool exists,
/// the stale links removed,
/// and the entries left where they were because they are not shadow links.
/// `Vec<PathBuf>` is an owned growable list of owned paths.
/// `#[derive(Debug, PartialEq)]` generates printing and `==` for the tests.
/// Why:
/// The `link` subcommand prints what happened and the tests check it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LinkReport = { linked: string[]; skipped: string[]; removed: string[]; leftAlone: string[] };
/// ```
#[derive(Debug, PartialEq)]
pub struct LinkReport {
    /// The links created or refreshed.
    pub linked: Vec<PathBuf>,
    /// The names with no real tool on PATH and no link to remove.
    pub skipped: Vec<String>,
    /// The links removed because their real tool is gone.
    pub removed: Vec<PathBuf>,
    /// The entries of a shadowed name that stay in place because they are not shadow links
    /// (somebody's own file, directory or link).
    pub left_alone: Vec<PathBuf>,
}

/// What:
/// `fn is_a_shadow_link(link_path: &Path, own_real_path: &Path) -> bool` declares a private function that answers
/// whether the entry at `link_path` is a link a shadow may replace:
/// one that resolves to `own_real_path`, this program's own resolved path, or to a shadow of any version.
/// Why:
/// The link directory is any directory the user names,
/// so an entry of a shadowed name may be somebody's own file or link,
/// and deleting it would destroy their work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isAShadowLink = (linkPath: string, ownRealPath: string): boolean => {
///   if (!lstatSync(linkPath).isSymbolicLink()) return false;
///   return realpathSync(linkPath) === ownRealPath || shadowBehind(linkPath) !== undefined;
/// };
/// ```
fn is_a_shadow_link(link_path: &Path, own_real_path: &Path) -> bool {
    // `symlink_metadata` describes the link itself rather than the file it points at, so a broken link still
    // answers; `.is_err()` means nothing is there at all.
    let Ok(metadata) = link_path.symlink_metadata() else {
        return false;
    };
    // `.file_type().is_symlink()` is true only for a symbolic link, not for a regular file or a directory.
    if !metadata.file_type().is_symlink() {
        return false;
    }
    // A link to this very program is ours; `fs::canonicalize` resolves the link so a relative spelling still matches.
    if let Ok(link_target) = fs::canonicalize(link_path)
        && link_target == own_real_path
    {
        return true;
    }
    // What:     `fs::read_link` gives the target as it is spelled in the link, which answers even when the target
    // itself no longer exists; the comparison asks whether that target is named `search-shadow`.
    // Why:      A prune removes the version directory an older release's link named, leaving a dangling link that no
    // file check can recognize, and the upgrade still has to take it over rather than leave it behind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const target = readlinkSync(linkPath);
    // if (path.basename(target) === 'search-shadow') return true;
    // ```
    if let Ok(target) = fs::read_link(link_path)
        && target.file_name() == Some(OsStr::new("search-shadow"))
    {
        return true;
    }
    // A link to a shadow of another version is the link an upgrade replaces, and a launcher record is followed to
    // the shadow it leads to; any other target is somebody else's.
    return shadow_behind(link_path).is_some();
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
        left_alone: Vec::new(),
    };
    for name in SHADOWED_NAMES {
        let link_path: PathBuf = directory.join(name);
        let has_real_tool: bool = find_in_path(name, path_variable, &skip).is_some();

        // `symlink_metadata` looks at the link itself, not its target, so a dangling link still counts as present.
        let link_exists: bool = link_path.symlink_metadata().is_ok();

        // What:     `if link_exists && !is_a_shadow_link(...)` runs for an entry that is there but is not a link a
        // shadow made.
        // Why:      That entry is somebody else's file, directory or link, and this subcommand only owns its own links.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (linkExists && !isAShadowLink(linkPath, executable)) { report.leftAlone.push(linkPath); continue; }
        // ```
        if link_exists && !is_a_shadow_link(&link_path, &skip) {
            report.left_alone.push(link_path);
            continue;
        }

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
