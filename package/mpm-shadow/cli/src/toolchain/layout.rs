//! What:     Where things are: the host's platform id, the install directory of one tool version, the directories each
//!           route puts its binaries in, and the bounded search for an executable inside them.
//! Why:      `status`, `sync`, the exposure farm and `bin-paths` must agree on where a tool lives; they all ask here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // layout.ts
//! export function hostPlatformId(): string
//! export function toolInstallDir(root: string, id: string, version: string): string
//! export function binSearchDirs(tool: LockedTool, scope: Scope, env: EnvMap, extra: string[]): string[]
//! export function findToolBin(tool: LockedTool, name: string, scope: Scope, env: EnvMap, extra: string[]): string | undefined
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use super::model::{LockedTool, OsFamily, Route};
use super::scope::{EnvMap, Scope};

/// How many directory levels below a search root `find_bin_under` descends.
const MAX_BIN_SEARCH_DEPTH: usize = 3;

/// What:     The platform id the lock's `artifacts` keys use, from the operating system, the CPU and the C library:
///           `linux-x64`, `linux-arm64-musl`, `macos-arm64`, `windows-x64`.
/// Why:      Pure so tests can ask for every combination; `current_platform_id` supplies the running host.
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

/// The platform id of the running host (a musl build runs on a musl host, so the build's C library is the host's).
pub fn current_platform_id() -> String {
    return platform_id(std::env::consts::OS, std::env::consts::ARCH, cfg!(target_env = "musl"));
}

/// The OS family of the running host, or `None` for a family the plan cannot name.
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

/// What:     A tool id as a directory name: letters, digits, `.`, `_` and `-` kept, every run of other characters
///           (`:`, `/`, `@`) turned into one `-`.
/// Why:      `npm:@actions/languageserver` becomes `npm-actions-languageserver`, a legal single path segment.
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

/// The directory one tool version installs into: `<toolchain root>/<install dir name>/<version>`; an empty `version`
/// gives the tool's own directory.
pub fn tool_install_dir(toolchain_root: &Path, id: &str, version: &str) -> PathBuf {
    let tool_dir: PathBuf = toolchain_root.join(install_dir_name(id));
    if version.is_empty() {
        return tool_dir;
    }
    return tool_dir.join(version);
}

/// The directory below `$CARGO_HOME` (default `~/.cargo`) cargo installs binaries into.
fn cargo_bin_dir(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("CARGO_HOME") {
        Some(directory) => PathBuf::from(directory).join("bin"),
        None => home.join(".cargo").join("bin"),
    };
}

/// `$UV_TOOL_BIN_DIR`, default `~/.local/bin`.
fn uv_tool_bin_dir(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("UV_TOOL_BIN_DIR") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".local").join("bin"),
    };
}

/// `$PNPM_HOME`, default `~/.local/share/pnpm`.
fn pnpm_home(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("PNPM_HOME") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".local").join("share").join("pnpm"),
    };
}

/// What:     The bin directories of the package managers the routes install through, preceded by the scope's exposure
///           and toolchain bin directories: the directories a child process needs on `PATH`.
/// Why:      `bin-paths` reports the ones that exist; install commands put all of them on `PATH`.
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

/// What:     The `sdkmanager` build-tools directory name of an Android tool, from its `androidPackages` entry
///           (`build-tools/37.0.0` or `build-tools;37.0.0`), or the tool's own version when none is declared.
/// Why:      The build-tools binaries sit in `build-tools/<version>`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function buildToolsVersion(tool: LockedTool): string
/// ```
fn build_tools_version(tool: &LockedTool) -> String {
    if let Some(packages) = &tool.declaration.options.android_packages {
        for package in packages {
            if let Some(version) = package.strip_prefix("build-tools;").or_else(|| return package.strip_prefix("build-tools/")) {
                return version.to_string();
            }
        }
    }
    return tool.version.clone();
}

/// What:     The directories where a tool's binaries are looked for, in order, by route.
/// Why:      Each route installs somewhere different (cargo's bin, the pnpm home, a per-tool directory), and the
///           first hit wins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function binSearchDirs(tool: LockedTool, scope: Scope, env: EnvMap, extra: string[]): string[]
/// ```
pub fn bin_search_dirs(tool: &LockedTool, scope: &Scope, env: &EnvMap, extra_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let install_dir: PathBuf = tool_install_dir(&scope.toolchain_root, &tool.declaration.id, &tool.version);
    let mut dirs: Vec<PathBuf> = Vec::new();
    match tool.declaration.route {
        Route::CargoBinstall => dirs.push(cargo_bin_dir(env, &scope.home)),
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
        Route::CargoGit | Route::GoInstall | Route::Archive => dirs.push(install_dir),
        Route::RustupToolchain => {
            dirs.push(cargo_bin_dir(env, &scope.home));
            let rustup_home: PathBuf = match env.get("RUSTUP_HOME") {
                Some(directory) => PathBuf::from(directory),
                None => scope.home.join(".rustup"),
            };
            dirs.push(rustup_home.join("toolchains"));
        }
        Route::AndroidSdk => {
            dirs.push(install_dir.join("cmdline-tools").join(&tool.version).join("bin"));
            dirs.push(install_dir.join("platform-tools"));
            dirs.push(install_dir.join("build-tools").join(build_tools_version(tool)));
        }
        Route::RustupComponents => {}
    }
    return dirs;
}

/// What:     Whether `path` is a regular file the current user can run (any execute bit on Unix; on Windows, any file).
/// Why:      A directory or a data file of the same name must not count as the binary.
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
        use std::os::unix::fs::PermissionsExt;
        return metadata.permissions().mode() & 0o111 != 0;
    }
    #[cfg(not(unix))]
    {
        return true;
    }
}

/// The names of the directories directly inside `directory`, sorted (a missing or unreadable directory has none).
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

/// What:     Looks for an executable called `name` directly in `root`, then in `root`'s subdirectories level by level,
///           down to `MAX_BIN_SEARCH_DEPTH` levels; the first hit in sorted order wins.
/// Why:      Archives unpack to `node-v26.13.0-linux-x64/bin/node`; the farm links the real file without knowing the
///           archive's inner layout. Level by level (not depth first) so a shallow hit beats a deep one.
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

/// What:     The executable `name` of one tool: the first of its search directories that holds it directly or below.
/// Why:      Every consumer (observers, providers, the farm) resolves binaries through this one search.
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
