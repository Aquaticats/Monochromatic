//! What:
//! Where a tool's executables are looked for,
//! route by route,
//! and the bounded search that finds one inside such a directory:
//! directly,
//! or level by level below it,
//! a shallow hit before a deep one.
//! Why:
//! Every consumer (observers,
//! providers,
//! the exposure farm) resolves binaries through this one search,
//! so a tool that `install` placed is found by `list` at the same path,
//! and an archive's inner layout never has to be guessed.
//! A `rustup-toolchain` row's directory is not read from the names in `~/.rustup/toolchains`:
//! only `rustup` knows which toolchain a channel is,
//! so the caller that can ask it passes that one directory in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function binSearchDirs(tool: LockedTool, scope: Scope, env: EnvMap, extra: string[], theToolchainsBin: string | undefined): string[]
//! export function isExecutable(path: string): boolean
//! export function findBinUnder(root: string, name: string): string | undefined
//! export function findToolBin(tool: LockedTool, name: string, scope: Scope, env: EnvMap, extra: string[], theToolchainsBin: string | undefined): string | undefined
//! ```
/// Imports the file-system module the search and the executable check read.
use std::fs;
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the newest finished copy of a repository-published tool.
use super::super::across_roots::winning_root;
/// Imports the cargo bin directory of one tool.
use super::super::cargo_installs::cargo_bin_dir_of;
/// Imports the locked tool and the routes its directories depend on.
use super::super::model::{
    LockedTool,
    Route,
};
/// Imports the `current` link and the rule that names a release archive of this repository.
use super::super::published_archive::{
    current_link_of,
    is_published_archive,
};
/// Imports the environment map and the scope the directories are built from.
use super::super::scope::{
    EnvMap,
    Scope,
};
/// Imports the install directory of one tool version.
use super::install_directories::tool_install_dir;
/// Imports the package managers' directories the search also looks in.
use super::manager_homes::{
    cargo_bin_dir,
    pnpm_home,
    uv_tool_bin_dir,
};

/// How many directory levels below a search root `find_bin_under` descends.
const MAX_BIN_SEARCH_DEPTH: usize = 3;

/// What:
/// The `sdkmanager` build-tools directory name of an Android tool,
/// from its `androidPackages` entry
///           (`build-tools/37.0.0` or `build-tools;37.0.0`),
/// or the tool's own version when none is declared.
/// Why:
/// The build-tools binaries sit in `build-tools/<version>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildToolsVersion(tool: LockedTool): string
/// ```
fn build_tools_version(tool: &LockedTool) -> String {
    if let Some(packages) = &tool.declaration.options.android_packages {
        for package in packages {
            let stripped: Option<&str> = match package.strip_prefix("build-tools;") {
                Some(version) => Some(version),
                None => package.strip_prefix("build-tools/"),
            };
            if let Some(version) = stripped {
                return version.to_string();
            }
        }
    }
    return tool.version.clone();
}

/// What:
/// The directories where a tool's binaries are looked for,
/// in order,
/// by route.
/// Why:
/// Each route installs somewhere different (cargo's bin,
/// the pnpm home,
/// a per-tool directory),
/// and the
///           first hit wins.
/// `the_toolchains_bin` is the `bin` directory of the toolchain a `rustup-toolchain` row's channel is,
/// as `rustup` names it (see `Context::the_toolchains_bin_dir`),
/// for this caller has to ask the manager before the search can read that directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function binSearchDirs(tool: LockedTool, scope: Scope, env: EnvMap, extra: string[], theToolchainsBin: string | undefined): string[]
/// ```
pub fn bin_search_dirs(
    tool: &LockedTool,
    scope: &Scope,
    env: &EnvMap,
    extra_dirs: &[PathBuf],
    the_toolchains_bin: Option<&Path>,
) -> Vec<PathBuf> {
    // A name that is not one plain directory name is refused when the plan and the lock are decoded, so the error
    // arm is unreachable for a decoded lock; for any other value "nowhere to look" is the safe answer.
    let install_dir: Option<PathBuf> =
        tool_install_dir(&scope.toolchain_root, &tool.declaration.id, &tool.version).ok();
    let mut dirs: Vec<PathBuf> = Vec::new();
    match tool.declaration.route {
        Route::CargoBinstall => dirs.push(cargo_bin_dir_of(tool, scope)),
        Route::BinSourced => {
            dirs.push(scope.toolchain_bin_dir.clone());
            dirs.push(scope.home.join(".local").join("bin"));
            dirs.extend(extra_dirs.iter().cloned());
        }
        Route::MpmPnpm => {
            dirs.extend(extra_dirs.iter().cloned());
            dirs.push(pnpm_home(env, &scope.home));
            dirs.push(scope.toolchain_root.join("node"));
        }
        Route::MpmUvx => {
            dirs.push(uv_tool_bin_dir(env, &scope.home));
            dirs.extend(extra_dirs.iter().cloned());
        }
        // A release archive of this repository is run from the newest installed version, through its `current` link.
        Route::Archive if is_published_archive(tool) => {
            if let Ok(link) = current_link_of(&winning_root(scope, tool), &tool.declaration.id) {
                dirs.push(link);
            }
        }
        Route::CargoGit | Route::GoInstall | Route::Archive => dirs.extend(install_dir),
        // A `rustup-toolchain` row's programs are the ones the toolchain its channel is holds,
        // and that directory is the manager's to name;
        // the neighbours in `~/.rustup/toolchains` are other toolchains and are never read:
        // a custom linked channel's directory is the link's own name,
        // a second host triple's directory extends the channel exactly as this host's does,
        // and a linked name that extends the channel is a toolchain of its own
        // (issue 6634's read, L-1).
        Route::RustupToolchain => {
            dirs.push(cargo_bin_dir(env, &scope.home));
            if let Some(bin) = the_toolchains_bin {
                dirs.push(bin.to_path_buf());
            }
        }
        Route::AndroidSdk => {
            if let Some(sdk) = install_dir {
                dirs.push(sdk.join("cmdline-tools").join(&tool.version).join("bin"));
                dirs.push(sdk.join("platform-tools"));
                dirs.push(sdk.join("build-tools").join(build_tools_version(tool)));
            }
        }
        Route::RustupComponents => {}
    }
    return dirs;
}

/// What:
/// Whether `path` is a regular file the current user can run (any execute bit on Unix;
/// on Windows,
/// any file).
/// Why:
/// A directory or a data file of the same name must not count as the binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function isExecutable(path: string): Promise<boolean>
/// ```
pub fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        /// Imports `PermissionsExt` from `std::os::unix::fs`.
        use std::os::unix::fs::PermissionsExt;
        return metadata.permissions().mode() & 0o111 != 0;
    }
    #[cfg(not(unix))]
    {
        return true;
    }
}

/// The names of the directories directly inside `directory`,
/// sorted (a missing or unreadable directory has none).
fn sorted_child_directories(directory: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut children: Vec<PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        if path.is_dir() {
            children.push(path);
        }
    }
    children.sort();
    return children;
}

/// What:
/// Looks for an executable called `name` directly in `root`,
/// then in `root`'s subdirectories level by level,
///           down to `MAX_BIN_SEARCH_DEPTH` levels;
/// the first hit in sorted order wins.
/// Why:
/// Archives unpack to `node-v26.13.0-linux-x64/bin/node`;
/// the farm links the real file without knowing the
///           archive's inner layout.
/// Level by level (not depth first) so a shallow hit beats a deep one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function findBinUnder(root: string, name: string): Promise<string | undefined>
/// ```
pub fn find_bin_under(
    root: &Path,
    name: &str,
) -> Option<PathBuf> {
    let mut level: Vec<PathBuf> = vec![root.to_path_buf()];
    for _depth in 0..=MAX_BIN_SEARCH_DEPTH {
        if level.is_empty() {
            return None;
        }
        for directory in &level {
            let direct: PathBuf = directory.join(name);
            if is_executable(&direct) {
                return Some(direct);
            }
        }
        let mut next: Vec<PathBuf> = Vec::new();
        for directory in &level {
            next.extend(sorted_child_directories(directory));
        }
        level = next;
    }
    return None;
}

/// What:
/// The executable `name` of one tool:
/// the first of its search directories that holds it directly or below.
/// Why:
/// Every consumer (observers,
/// providers,
/// the farm) resolves binaries through this one search,
/// and the directory of a `rustup-toolchain` row's toolchain is what the caller asked the manager for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function findToolBin(tool, name, scope, env, extra, theToolchainsBin): Promise<string | undefined>
/// ```
pub fn find_tool_bin(
    tool: &LockedTool,
    name: &str,
    scope: &Scope,
    env: &EnvMap,
    extra_dirs: &[PathBuf],
    the_toolchains_bin: Option<&Path>,
) -> Option<PathBuf> {
    for directory in bin_search_dirs(tool, scope, env, extra_dirs, the_toolchains_bin) {
        let direct: PathBuf = directory.join(name);
        if is_executable(&direct) {
            return Some(direct);
        }
        if let Some(nested) = find_bin_under(&directory, name) {
            return Some(nested);
        }
    }
    return None;
}
