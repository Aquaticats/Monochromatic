//! What:
//! Which `mpm-shadow` a launcher is a copy of:
//! the newest installed one,
//! found through its `current` link in either toolchain root,
//! and,
//! when the running program belongs to no install (a build folder,
//! a copy fetched to a scratch path),
//! the running program itself.
//! Also the `/proc/self/exe` read that makes a running program whose file was deleted copyable.
//! Why:
//! A launcher copied from whichever `mpm-shadow` ran `install` is as old as that copy.
//! The machine-global `git` launcher stayed a `0.2.7-alpha.2` copy that way,
//! and a launcher older than the follow-`current` rule starts a recorded version folder that a newer install deleted
//! (`doc/troubleshooting/mpm-shadow-launcher-names-a-deleted-release-folder-after-a-newer-install.md`).
//! Copying the newest installed shadow gives every launcher the code that follows `current` and rewrites its record,
//! and it also removes the dependence on the running file:
//! an `mpm-shadow` that installs a newer one has just deleted its own file,
//! which no operating system without `/proc` can hand back.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // launcher-source.ts
//! export function launcherSource(scope: Scope, running: string, procAvailable: boolean): string  // throws
//! export function newestInstalledShadow(scope: Scope, running: string): string | undefined
//! export function usableSource(path: string, procAvailable: boolean): string  // throws
//! ```

/// Imports `OsString` from `std::ffi`.
use std::ffi::OsString;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports `CURRENT_LINK_NAME` from `super::cargo_installs`.
use super::cargo_installs::CURRENT_LINK_NAME;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `version_named_by` from `super::launcher_current_link`.
use super::launcher_current_link::version_named_by;
/// Imports `find_bin_under` from `super::layout`.
use super::layout::find_bin_under;
/// Imports `Scope` from `super::scope`.
use super::scope::Scope;
/// Imports `file_name_text` from `super::util`.
use super::util::file_name_text;

/// The running process's own executable as the Linux kernel exposes it,
/// readable even after the file it was started from was deleted.
pub const PROC_SELF_EXE: &str = "/proc/self/exe";

/// The kernel's note on the path of a running program whose file was removed.
const DELETED_SUFFIX: &str = " (deleted)";

/// What:
/// The running program's path as the operating system reports it.
/// Why:
/// The path itself says whether this shadow is an install,
/// so the search for the newest installed one starts here,
/// not at the readable file `usable_source` may return.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runningProgram(): string  // throws
/// ```
pub fn running_program() -> Result<PathBuf, ToolchainError> {
    return match std::env::current_exe() {
        Ok(found) => Ok(found),
        Err(error) => Err(ToolchainError::failure(format!("cannot find the running mpm-shadow to make a launcher: {error}"))),
    };
}

/// What:
/// `path` when it names a file;
/// otherwise the running process's own executable through `/proc` when `proc_available`;
/// otherwise an error that says the running program's file is gone and what to do.
/// Why:
/// Only Linux has `/proc/self/exe`.
/// On macOS a deleted running program reports its old path and no other way to read it exists,
/// so naming `/proc/self/exe` there would end in a `cannot copy /proc/self/exe` message about a file that never existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function usableSource(path: string, procAvailable: boolean): string  // throws
/// ```
pub fn usable_source(path: PathBuf, proc_available: bool) -> Result<PathBuf, ToolchainError> {
    if path.exists() {
        return Ok(path);
    }
    if proc_available {
        return Ok(PathBuf::from(PROC_SELF_EXE));
    }
    return Err(ToolchainError::failure(format!(
        "the running mpm-shadow, {}, no longer exists: installing a newer mpm-shadow removes the version directory this one was started from, \
         and this operating system has no way to read a running program whose file was deleted, so the launchers could not be made. \
         The newer mpm-shadow is installed; run the same command again with it \
         (its `current` link in the toolchain root) and the launchers are made then",
        path.display()
    )));
}

/// Whether the metadata of the link's own path says it is a symbolic link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isSymbolicLink(metadata: Metadata): boolean
/// ```
fn is_symbolic_link(metadata: fs::Metadata) -> bool {
    return metadata.is_symlink();
}

/// What:
/// Whether the directory holds a link named `current`:
/// the mark of a tool's install directory,
/// whichever route installed it (`<root>/cargo/<crate>/current`,
/// `<root>/<tool id>/current`).
/// Why:
/// The install directory is found from the running program's own path,
/// which does not pass through the link,
/// so the mark is a child of the directory rather than a name on the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holdsCurrentLink(directory: string): boolean
/// ```
fn holds_current_link(directory: &Path) -> bool {
    // A version directory is never called `current` (the version check refuses it), so a link of that name marks the
    // directory above the versions.
    return fs::symlink_metadata(directory.join(CURRENT_LINK_NAME)).is_ok_and(is_symbolic_link);
}

/// What:
/// The install directory `program` belongs to:
/// the nearest ancestor that holds a `current` link.
/// Why:
/// An installed program sits at `<root>/<tool id>/<version>/<archive folder>/<name>`
/// or `<root>/cargo/<crate>/<version>/bin/<name>`,
/// so it does not pass through the link itself and the link is found beside the versions,
/// not above them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installDirectoryAbove(program: string): string | undefined
/// ```
fn install_directory_above(program: &Path) -> Option<&Path> {
    for directory in program.ancestors().skip(1) {
        if holds_current_link(directory) {
            return Some(directory);
        }
    }
    return None;
}

/// What:
/// The install directory `program` belongs to,
/// as the path below the toolchain root that holds it:
/// `<install directory>/current` is the link every launcher record and exposed link resolves through,
/// so the same path names the same tool in both toolchain roots.
/// Why:
/// A tree's launchers and the machine-global ones are the same set,
/// and a program that belongs to neither root (a build folder) belongs to no install.
/// The install directory and each root are compared as files,
/// never as text:
/// on a machine whose home is a symbolic link (`/home` to `/var/home`) the running program's own path is the
/// resolved spelling `/proc/self/exe` reports,
/// while a scope's root keeps the spelling `HOME` gives,
/// so a text comparison found no install on any such machine and every launcher was a copy of whichever program ran
/// `install` instead of the newest installed one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installBelowRoot(scope: Scope, program: string): string | undefined
/// ```
fn install_below_root(scope: &Scope, program: &Path) -> Option<PathBuf> {
    let install: &Path = install_directory_above(program)?;
    let Ok(resolved_install) = fs::canonicalize(install) else {
        return None;
    };
    for root in [&scope.toolchain_root, &scope.other_toolchain_root] {
        // A root that does not exist yet holds no install, and has no resolved spelling to compare with.
        let Ok(resolved_root) = fs::canonicalize(root) else {
            continue;
        };
        if let Ok(relative) = resolved_install.strip_prefix(&resolved_root) {
            return Some(relative.to_path_buf());
        }
    }
    return None;
}

/// What:
/// The name of the program file,
/// without the kernel's ` (deleted)` note on a running program whose file was removed.
/// Why:
/// The search under `current` looks for the same file name the running install had,
/// and the exposure farm links the running `mpm-shadow` under this name,
/// which is the word the sentences it prints tell a person to type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function programName(program: string): string | undefined
/// ```
pub fn program_name(program: &Path) -> Option<OsString> {
    let text: String = file_name_text(program);
    let name: &str = text.strip_suffix(DELETED_SUFFIX).unwrap_or(&text);
    if name.is_empty() {
        return None;
    }
    return Some(OsString::from(name));
}

/// What:
/// The program of the newest installed `mpm-shadow` a launcher should be a copy of,
/// looked for in the same install directory below both toolchain roots,
/// or `None` when the running program belongs to no install.
/// Why:
/// The newest copy wins wherever it is,
/// the rule the repository's own tools already follow across the two roots,
/// and `current` only ever names a finished version,
/// so reading the links costs no check of completion markers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function newestInstalledShadow(scope: Scope, running: string): string | undefined
/// ```
pub fn newest_installed_shadow(scope: &Scope, running: &Path) -> Option<PathBuf> {
    let relative: PathBuf = install_below_root(scope, running)?;
    let name: OsString = program_name(running)?;
    let mut newest: Option<(String, PathBuf)> = None;
    for root in [&scope.toolchain_root, &scope.other_toolchain_root] {
        let link: PathBuf = root.join(&relative).join(CURRENT_LINK_NAME);
        let Some(version) = version_named_by(&link) else {
            continue;
        };
        if let Some((best, _program)) = &newest
            && compare_versions(&version, best).is_le()
        {
            continue;
        }
        if let Some(program) = find_bin_under(&link, &name.to_string_lossy()) {
            newest = Some((version, program));
        }
    }
    let (_version, program) = newest?;
    return Some(program);
}

/// What:
/// The file to make a launcher from:
/// the newest installed `mpm-shadow` when the running program belongs to an install,
/// otherwise the running program itself (through `/proc/self/exe` when its file was deleted).
/// Why:
/// One place decides which binary every launcher is,
/// so no caller can fall back to the installer without saying so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function launcherSource(scope: Scope, running: string, procAvailable: boolean): string  // throws
/// ```
pub fn launcher_source(scope: &Scope, running: PathBuf, proc_available: bool) -> Result<PathBuf, ToolchainError> {
    if let Some(newest) = newest_installed_shadow(scope, &running) {
        return Ok(newest);
    }
    return usable_source(running, proc_available);
}
