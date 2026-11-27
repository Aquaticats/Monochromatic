//! What:
//! The directories one tool version lives in:
//! the install directory,
//! the `bin` directory a Go-installed tool writes into,
//! and the directory its downloaded archives are kept in,
//! each built from the toolchain root,
//! the tool id and the version.
//! Why:
//! `install`,
//! `list`,
//! the exposure farm and `prune` must name the same path for the same tool,
//! so every path is built here,
//! and every name that becomes a path segment is checked here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function installDirName(id: string): string
//! export function toolDirectory(root: string, id: string): string        // throws on an unsafe id
//! export function toolInstallDir(root: string, id: string, version: string): string  // throws
//! export function goBinDirOf(root: string, id: string, version: string): string      // throws
//! export function toolDownloadsDir(root: string, id: string, version: string): string  // throws
//! ```
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the error a refused name is reported as.
use super::super::error::ToolchainError;
/// Imports the name checks that keep every segment one plain directory name.
use super::super::segment::{
    check_tool_id,
    check_version,
};

/// What:
/// A tool id as a directory name:
/// letters,
/// digits,
/// `.`,
/// `_` and `-` kept,
/// every run of other characters
///           (`:`,
/// `/`,
/// `@`) turned into one `-`.
/// Why:
/// `npm:@actions/languageserver` becomes `npm-actions-languageserver`,
/// a legal single path segment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installDirName(id: string): string
/// ```
pub fn install_dir_name(id: &str) -> String {
    let mut name: String = String::new();
    let mut previous_was_replaced: bool = false;
    for character in id.chars() {
        let kept: bool = character.is_ascii_alphanumeric()
            || character == '.'
            || character == '_'
            || character == '-';
        if kept {
            name.push(character);
            previous_was_replaced = false;
        } else if !previous_was_replaced {
            name.push('-');
            previous_was_replaced = true;
        }
    }
    return name;
}

/// What:
/// The directory that holds every installed version of a tool:
/// `<toolchain root>/<install dir name>`.
/// Why:
/// Refuses an id whose directory name is not one plain name or is a directory the toolchain root keeps for something
/// else,
/// so a crafted id never makes a path outside the toolchain root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function toolDirectory(toolchainRoot: string, id: string): string  // throws
/// ```
pub fn tool_directory(
    toolchain_root: &Path,
    id: &str,
) -> Result<PathBuf, ToolchainError> {
    if let Err(message) = check_tool_id(id) {
        return Err(ToolchainError::failure(message));
    }
    return Ok(toolchain_root.join(install_dir_name(id)));
}

/// What:
/// The directory one tool version installs into:
/// `<toolchain root>/<install dir name>/<version>`.
/// Why:
/// Refuses an id (see `tool_directory`) or a version that is not one plain name
/// (empty,
/// `.`,
/// `..`,
/// a separator,
/// NUL,
/// a leading dot,
/// `current`),
/// so a crafted lock never makes an install or a deletion reach outside the tool's directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function toolInstallDir(toolchainRoot: string, id: string, version: string): string  // throws
/// ```
pub fn tool_install_dir(
    toolchain_root: &Path,
    id: &str,
    version: &str,
) -> Result<PathBuf, ToolchainError> {
    let tool_dir: PathBuf = tool_directory(toolchain_root, id)?;
    if let Err(message) = check_version(version) {
        return Err(ToolchainError::failure(message));
    }
    return Ok(tool_dir.join(version));
}

/// The directory name below a Go-installed tool's version directory that `go install` writes its executables into.
pub const GO_BIN_DIRECTORY_NAME: &str = "bin";

/// What:
/// The `bin` directory below a Go-installed tool's version directory:
/// the `GOBIN` the `go-install` route gives `go install`,
/// and the place the observation of that route reads.
/// Why:
/// The installer creates it before `go` runs and the observer reads it afterwards,
/// so both must name the same path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function goBinDirOf(toolchainRoot: string, id: string, version: string): string  // throws on an unsafe name
/// ```
pub fn go_bin_dir_of(
    toolchain_root: &Path,
    id: &str,
    version: &str,
) -> Result<PathBuf, ToolchainError> {
    return Ok(tool_install_dir(toolchain_root, id, version)?.join(GO_BIN_DIRECTORY_NAME));
}

/// What:
/// The directory a version's downloaded archives are kept in:
/// `<toolchain root>/downloads/<install dir name>/<version>`.
/// Why:
/// One directory per tool and version,
/// so removing the downloads of one version can never touch another's
/// (a name prefix such as `a-1-` would match both `a` at version `1-x` and `a-1`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function toolDownloadsDir(toolchainRoot: string, id: string, version: string): string  // throws
/// ```
pub fn tool_downloads_dir(
    toolchain_root: &Path,
    id: &str,
    version: &str,
) -> Result<PathBuf, ToolchainError> {
    let downloads: PathBuf = toolchain_root.join("downloads");
    let tool_dir: PathBuf = tool_directory(&downloads, id)?;
    if let Err(message) = check_version(version) {
        return Err(ToolchainError::failure(message));
    }
    return Ok(tool_dir.join(version));
}
