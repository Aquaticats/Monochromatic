//! Looking a server program up in the directories the toolchain provides for a project,
//!  and building the `PATH` the confined server gets.
//!
//! The toolchain's directories come first,
//!  then the application's own `PATH`.
//!  A leftover shim of a retired tool manager is never accepted in a project the
//!  toolchain serves:
//!  a shim is the tool manager itself,
//!  which fails inside the sandbox with "No version is set for shim".

/// A program must be a regular file with an execute bit.
use crate::language::launch::{is_executable, resolve_executable};
/// What:
///  `OsString` is text that may not be valid Unicode (sibling:
///  `String`);
///  `Path` and `PathBuf` are borrowed and owned filesystem paths.
/// Why:
///  `PATH` entries are file names,
///  which are not guaranteed to be Unicode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// What:
///  Why a program was not found,
///  when something more specific than "not on PATH" is known.
///  `Option<PathBuf>` is "a path,
///  or nothing".
/// Why:
///  A shim found in place of a real program is a different cause with a different remedy than a
///  program that is absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Missed = { shim?: string };
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Missed {
    /// The first leftover `mise` shim that carries the program's name,
    ///  skipped because it cannot run inside the sandbox.
    pub shim: Option<PathBuf>,
}

/// What:
///  True for a symbolic link that resolves to the `mise` program itself.
///  `canonicalize` follows every link and returns the final path.
/// Why:
///  On Linux a shim is a link named like the tool whose target is the `mise` executable;
///  the file name of the final target is the whole test,
///  so no shim directory is assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isShim = (path: string) => lstatSync(path).isSymbolicLink() && basename(realpathSync(path)) === 'mise';
/// ```
pub fn is_shim(path: &Path) -> bool {
    let is_link = match std::fs::symlink_metadata(path) {
        Ok(found) => found.is_symlink(),
        Err(_) => false,
    };
    if !is_link {
        return false;
    }
    if let Ok(real) = std::fs::canonicalize(path)
        && let Some(name) = real.file_name()
    {
        return name == "mise";
    }
    return false;
}

/// What:
///  Find a server program for a project the toolchain serves:
///  a command with a path separator is taken relative to the project root,
///  a bare name is searched in `directories`,
///  then in the application's `PATH`.
///  `Result<PathBuf, Missed>` is the program or the reason it was not found.
/// Why:
///  The project's own tools win over the stale `PATH` of the desktop session,
///  and a shim is skipped (and remembered) because it cannot run confined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findProgram(command: string, root: string, directories: string[]): string // throws Missed
/// ```
pub fn find_program(
    command: &str,
    root: &Path,
    directories: &[PathBuf],
) -> Result<PathBuf, Missed> {
    let inherited = std::env::var_os("PATH").unwrap_or_default();
    return find_program_in(
        command,
        root,
        directories,
        &std::env::split_paths(&inherited).collect::<Vec<PathBuf>>(),
    );
}

/// What:
///  `find_program` with the application's `PATH` given as a list of directories.
/// Why:
///  A test passes its own directories instead of changing the process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findProgramIn(command: string, root: string, directories: string[], inherited: string[]): string // throws Missed
/// ```
pub fn find_program_in(
    command: &str,
    root: &Path,
    directories: &[PathBuf],
    inherited: &[PathBuf],
) -> Result<PathBuf, Missed> {
    if command.contains('/') {
        return resolve_executable(command, root).ok_or_else(Missed::default);
    }
    let mut missed = Missed::default();
    // A relative entry would depend on the working directory; only absolute ones are trusted.
    for directory in directories.iter().chain(inherited.iter()) {
        if !directory.is_absolute() {
            continue;
        }
        let candidate = directory.join(command);
        if !is_executable(&candidate) {
            continue;
        }
        if is_shim(&candidate) {
            tracing::debug!(candidate = %candidate.display(), "skipping a leftover mise shim: it cannot run inside the sandbox");
            missed.shim.get_or_insert(candidate);
            continue;
        }
        return Ok(candidate);
    }
    return Err(missed);
}

/// What:
///  The `PATH` value a confined server gets:
///  the toolchain's directories first,
///  then `inherited`.
///  A directory whose name contains the separator `:` cannot stand in a `PATH` and is left out,
///  as is one that is not valid Unicode (Helix stores variables as text).
/// Why:
///  A Node server's launcher starts with `#!/usr/bin/env node`,
///  which needs the project's `node` on `PATH` inside the sandbox.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function searchPath(directories: string[], inherited?: string): string
/// ```
pub fn search_path(directories: &[PathBuf], inherited: Option<OsString>) -> Option<String> {
    let mut entries: Vec<PathBuf> = Vec::new();
    for directory in directories {
        let usable = match directory.to_str() {
            Some(text) => !text.contains(':'),
            None => false,
        };
        if usable {
            entries.push(directory.clone());
        } else {
            tracing::warn!(directory = %directory.display(), "a toolchain directory cannot be part of PATH and is left out");
        }
    }
    if let Some(application) = inherited {
        entries.extend(std::env::split_paths(&application));
    }
    // `join_paths` fails only for an entry containing the separator, which inherited entries cannot.
    let joined = std::env::join_paths(entries).ok()?;
    return joined.into_string().ok();
}

/// Lookup order,
///  shim refusal,
///  and `PATH` construction on disposable directories.
#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
