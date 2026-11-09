//! What:
//! Where things are:
//! the host's platform id,
//! the install directory of one tool version,
//! the directories each
//!           route puts its binaries in,
//! and the bounded search for an executable inside them.
//! Why:
//! `list`,
//! `install`,
//! the exposure farm and `bin` must agree on where a tool lives;
//! they all ask here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // layout.ts
//! export function hostPlatformId(): string
//! export function toolInstallDir(root: string, id: string, version: string): string  // throws on an unsafe name
//! export function binSearchDirs(tool: LockedTool, scope: Scope, env: EnvMap, extra: string[]): string[]
//! export function findToolBin(tool: LockedTool, name: string, scope: Scope, env: EnvMap, extra: string[]): string | undefined
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `winning_root` from `super::across_roots`.
use super::across_roots::winning_root;
/// Imports `cargo_bin_dir_of` from `super::cargo_installs`.
use super::cargo_installs::cargo_bin_dir_of;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, OsFamily, Route};
/// Imports what this module uses from `super::published_archive`.
use super::published_archive::{current_link_of, is_published_archive};
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope};
/// Imports the name checks from `super::segment`.
use super::segment::{check_tool_id, check_version};

/// How many directory levels below a search root `find_bin_under` descends.
const MAX_BIN_SEARCH_DEPTH: usize = 3;

/// What:
/// The platform id the lock's `artifacts` keys use,
/// from the operating system,
/// the CPU and the C library:
///           `linux-x64`,
/// `linux-arm64-musl`,
/// `macos-arm64`,
/// `windows-x64`.
/// Why:
/// Pure so tests can ask for every combination;
/// `current_platform_id` supplies the running host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function platformId(os: string, arch: string, musl: boolean): string
/// ```
pub fn platform_id(os: &str, arch: &str, musl: bool) -> String {
    let cpu: &str = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };
    let family: &str = match os {
        "linux" => "linux",
        "macos" => "macos",
        "windows" => "windows",
        other => other,
    };
    if family == "linux" && musl {
        return format!("linux-{cpu}-musl");
    }
    return format!("{family}-{cpu}");
}

/// The platform id of the running host (a musl build runs on a musl host,
/// so the build's C library is the host's).
pub fn current_platform_id() -> String {
    return platform_id(std::env::consts::OS, std::env::consts::ARCH, cfg!(target_env = "musl"));
}

/// The OS family of the running host,
/// or `None` for a family the plan cannot name.
pub fn current_family() -> Option<OsFamily> {
    return OsFamily::parse(std::env::consts::OS);
}

/// Whether a tool with this OS filter installs on `family` (no filter means every family).
pub fn applies_to_family(os: &Option<Vec<OsFamily>>, family: Option<OsFamily>) -> bool {
    let Some(allowed) = os else {
        return true;
    };
    let Some(current) = family else {
        return false;
    };
    return allowed.contains(&current);
}

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
        let kept: bool = character.is_ascii_alphanumeric() || character == '.' || character == '_' || character == '-';
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
pub fn tool_directory(toolchain_root: &Path, id: &str) -> Result<PathBuf, ToolchainError> {
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
pub fn tool_install_dir(toolchain_root: &Path, id: &str, version: &str) -> Result<PathBuf, ToolchainError> {
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
pub fn go_bin_dir_of(toolchain_root: &Path, id: &str, version: &str) -> Result<PathBuf, ToolchainError> {
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
pub fn tool_downloads_dir(toolchain_root: &Path, id: &str, version: &str) -> Result<PathBuf, ToolchainError> {
    let downloads: PathBuf = toolchain_root.join("downloads");
    let tool_dir: PathBuf = tool_directory(&downloads, id)?;
    if let Err(message) = check_version(version) {
        return Err(ToolchainError::failure(message));
    }
    return Ok(tool_dir.join(version));
}

/// `$CARGO_HOME`,
/// default `~/.cargo`.
pub fn cargo_home_dir(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("CARGO_HOME") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".cargo"),
    };
}

/// The directory below `$CARGO_HOME` (default `~/.cargo`) cargo installs binaries into.
pub fn cargo_bin_dir(env: &EnvMap, home: &Path) -> PathBuf {
    return cargo_home_dir(env, home).join("bin");
}

/// `$UV_TOOL_BIN_DIR`,
/// default `~/.local/bin`.
fn uv_tool_bin_dir(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("UV_TOOL_BIN_DIR") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".local").join("bin"),
    };
}

/// `$PNPM_HOME`,
/// default `~/.local/share/pnpm`.
fn pnpm_home(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("PNPM_HOME") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".local").join("share").join("pnpm"),
    };
}

/// What:
/// The pnpm home spelled by the directories' physical paths:
/// the longest existing ancestor of `pnpm_home` resolved through its symbolic links,
/// with the part that does not exist yet appended unchanged.
/// Why:
/// pnpm writes the relative links of a global package from the spelling of its home,
/// while the filesystem follows every symbolic link of that spelling when it reads them back.
/// Where `$HOME` is a link (`/home` to `/var/home`) and the store is spelled physically,
/// `pnpm add --global` exits 0 and writes links that dangle,
/// and `pnpm list --global` then shows the package absent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function physicalPnpmHome(env: EnvMap, home: string): string
/// ```
pub fn physical_pnpm_home(env: &EnvMap, home: &Path) -> PathBuf {
    let spelled: PathBuf = pnpm_home(env, home);
    let mut missing_tail: Vec<std::ffi::OsString> = Vec::new();
    let mut existing: &Path = &spelled;
    // Walk up until an ancestor resolves; the names walked past are re-appended in their original order.
    loop {
        if let Ok(resolved) = std::fs::canonicalize(existing) {
            let mut physical: PathBuf = resolved;
            physical.extend(missing_tail.iter().rev());
            return physical;
        }
        match (existing.file_name(), existing.parent()) {
            (Some(name), Some(parent)) => {
                missing_tail.push(name.to_os_string());
                existing = parent;
            }
            // Nothing on the way up exists (a relative or empty spelling): the spelling stands.
            _ => return spelled,
        }
    }
}

/// What:
/// The bin directories of the package managers the routes install through,
/// preceded by the scope's exposure
///           and toolchain bin directories:
/// the directories a child process needs on `PATH`.
/// Why:
/// `bin` reports the ones that exist;
/// install commands put all of them on `PATH`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function managerNativeBinDirs(scope: Scope, env: EnvMap): string[]
/// ```
pub fn manager_native_bin_dirs(scope: &Scope, env: &EnvMap) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = vec![
        scope.exposure_dir.clone(),
        scope.toolchain_bin_dir.clone(),
        cargo_bin_dir(env, &scope.home),
        uv_tool_bin_dir(env, &scope.home),
        pnpm_home(env, &scope.home).join("bin"),
    ];
    if let Some(prefix) = env.get("npm_config_prefix") {
        dirs.push(PathBuf::from(prefix).join("bin"));
    }
    return dirs;
}

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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function binSearchDirs(tool: LockedTool, scope: Scope, env: EnvMap, extra: string[]): string[]
/// ```
pub fn bin_search_dirs(tool: &LockedTool, scope: &Scope, env: &EnvMap, extra_dirs: &[PathBuf]) -> Vec<PathBuf> {
    // A name that is not one plain directory name is refused when the plan and the lock are decoded, so the error
    // arm is unreachable for a decoded lock; for any other value "nowhere to look" is the safe answer.
    let install_dir: Option<PathBuf> = tool_install_dir(&scope.toolchain_root, &tool.declaration.id, &tool.version).ok();
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
        Route::RustupToolchain => {
            dirs.push(cargo_bin_dir(env, &scope.home));
            let rustup_home: PathBuf = match env.get("RUSTUP_HOME") {
                Some(directory) => PathBuf::from(directory),
                None => scope.home.join(".rustup"),
            };
            dirs.push(rustup_home.join("toolchains"));
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
pub fn find_bin_under(root: &Path, name: &str) -> Option<PathBuf> {
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
/// the farm) resolves binaries through this one search.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function findToolBin(tool, name, scope, env, extra): Promise<string | undefined>
/// ```
pub fn find_tool_bin(tool: &LockedTool, name: &str, scope: &Scope, env: &EnvMap, extra_dirs: &[PathBuf]) -> Option<PathBuf> {
    for directory in bin_search_dirs(tool, scope, env, extra_dirs) {
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
