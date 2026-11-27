//! What:
//! The installed version of a tool whose record is a file the installer left:
//! cargo's own `.crates.toml` in the crate's version directory,
//! the Android command-line tools' directory,
//! the version directory an archive or `git` install left,
//! and the executables a Go install wrote.
//! Why:
//! A version directory proves nothing where a program is the record:
//! `install` creates the directory before `go` runs,
//! so reading it would report a tool with no binaries as installed,
//! and the executables in the route's `bin` directory are the whole record.
//! A directory cargo never recorded is an unfinished install and counts as absent.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function observeCargoBinstall(ctx: Context, tool: LockedTool): string | undefined
//! export function observeAndroidSdk(ctx: Context, tool: LockedTool): string | undefined
//! export function observeVersionDirectory(ctx: Context, tool: LockedTool): string | undefined
//! export function observeGoInstall(ctx: Context, tool: LockedTool): string | undefined
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

/// Imports `observed_cargo_version` from `super::super::cargo_installs`.
use super::super::cargo_installs::observed_cargo_version;
/// Imports `Context` from `super::super::context`.
use super::super::context::Context;
/// Imports what this module uses from `super::super::layout`.
use super::super::layout::{
    go_bin_dir_of,
    is_executable,
    tool_directory,
    tool_install_dir,
};
/// Imports `LockedTool` from `super::super::model`.
use super::super::model::LockedTool;

/// The cargo install root of the crate's version directory,
/// for the cargo-binstall route (never `cargo binstall --list` or `$CARGO_HOME`,
/// which describe the shared install).
pub(super) fn observe_cargo_binstall(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    return observed_cargo_version(ctx.scope, tool);
}

/// The Android command-line tools directory for the locked version,
/// for the android-sdk route.
pub(super) fn observe_android_sdk(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    // A name that is not one plain directory name is refused where the lock is decoded; for any other value the
    // answer "not installed" is the safe one.
    let install: PathBuf = tool_install_dir(
        &ctx.scope.toolchain_root,
        &tool.declaration.id,
        &tool.version,
    )
    .ok()?;
    if install.join("cmdline-tools").join(&tool.version).exists() {
        return Some(tool.version.clone());
    }
    return None;
}

/// The names of the version directories directly inside one tool's directory,
/// sorted (a missing or unreadable directory has none).
fn version_directory_names(tool_root: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(tool_root) else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() && !name.starts_with('.') {
            names.push(name);
        }
    }
    names.sort();
    return names;
}

/// What:
/// The tool's own install directory,
/// for routes that leave no record elsewhere (archive,
/// cargo-git,
/// rustup-components):
/// the locked version when its directory exists,
/// otherwise the first other version directory,
/// otherwise `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function observeVersionDirectory(ctx: Context, tool: LockedTool): string | undefined
/// ```
pub(super) fn observe_version_directory(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    let tool_root: PathBuf =
        tool_directory(&ctx.scope.toolchain_root, &tool.declaration.id).ok()?;
    let names: Vec<String> = version_directory_names(&tool_root);
    if names.contains(&tool.version) {
        return Some(tool.version.clone());
    }
    return names.into_iter().next();
}

/// Whether the `bin` directory of one Go-installed version holds what `go install` writes there:
/// every executable the declaration names,
/// or any executable file when it names none.
fn go_bins_present(
    bin_dir: &Path,
    tool: &LockedTool,
) -> bool {
    if tool.declaration.bins.is_empty() {
        let Ok(entries) = fs::read_dir(bin_dir) else {
            return false;
        };
        for entry in entries.flatten() {
            if is_executable(&entry.path()) {
                return true;
            }
        }
        return false;
    }
    for name in &tool.declaration.bins {
        if !is_executable(&bin_dir.join(name)) {
            return false;
        }
    }
    return true;
}

/// What:
/// The version a Go-installed tool is at,
/// for the go-install route:
/// the locked version when the `bin` directory this route gives `go install` as `GOBIN` holds the executables it was
/// to write,
/// otherwise the first other version whose `bin` directory does,
/// otherwise `None`.
/// Why:
/// A Go install writes no record anywhere:
/// its executables are the whole record,
/// in the same way cargo's record is the whole record of a cargo install.
/// The version directory is not a record at all,
/// because the installer creates it before `go` runs,
/// so reading it would report a tool with no binaries as installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function observeGoInstall(ctx: Context, tool: LockedTool): string | undefined
/// ```
pub(super) fn observe_go_install(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    let mut candidates: Vec<String> = vec![tool.version.clone()];
    let Ok(tool_root) = tool_directory(&ctx.scope.toolchain_root, &tool.declaration.id) else {
        return None;
    };
    for name in version_directory_names(&tool_root) {
        if name != tool.version {
            candidates.push(name);
        }
    }
    for name in candidates {
        let Ok(bin_dir) = go_bin_dir_of(&ctx.scope.toolchain_root, &tool.declaration.id, &name)
        else {
            continue;
        };
        if go_bins_present(&bin_dir, tool) {
            return Some(name);
        }
    }
    return None;
}
