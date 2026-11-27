//! What:
//! Where the package managers keep their homes and binaries:
//! `$CARGO_HOME` and its `bin` directory,
//! `$UV_TOOL_BIN_DIR`,
//! `$PNPM_HOME`,
//! the pnpm home spelled by the directories' physical paths,
//! and the bin directories a child process needs on `PATH`.
//! Why:
//! Four routes install through a package manager,
//! so the directory each one writes into is asked for here,
//! once,
//! instead of being spelled again at every call site.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function cargoHomeDir(env: EnvMap, home: string): string
//! export function cargoBinDir(env: EnvMap, home: string): string
//! export function physicalPnpmHome(env: EnvMap, home: string): string
//! export function managerNativeBinDirs(scope: Scope, env: EnvMap): string[]
//! ```
/// Imports the borrowed and owned path types.
use std::path::{
    Path,
    PathBuf,
};

/// Imports the environment map and the scope whose home the managers sit under.
use super::super::scope::{
    EnvMap,
    Scope,
};

/// `$CARGO_HOME`,
/// default `~/.cargo`.
pub fn cargo_home_dir(
    env: &EnvMap,
    home: &Path,
) -> PathBuf {
    return match env.get("CARGO_HOME") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".cargo"),
    };
}

/// The directory below `$CARGO_HOME` (default `~/.cargo`) cargo installs binaries into.
pub fn cargo_bin_dir(
    env: &EnvMap,
    home: &Path,
) -> PathBuf {
    return cargo_home_dir(env, home).join("bin");
}

/// `$UV_TOOL_BIN_DIR`,
/// default `~/.local/bin`.
pub(super) fn uv_tool_bin_dir(
    env: &EnvMap,
    home: &Path,
) -> PathBuf {
    return match env.get("UV_TOOL_BIN_DIR") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".local").join("bin"),
    };
}

/// `$PNPM_HOME`,
/// default `~/.local/share/pnpm`.
pub(super) fn pnpm_home(
    env: &EnvMap,
    home: &Path,
) -> PathBuf {
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
pub fn physical_pnpm_home(
    env: &EnvMap,
    home: &Path,
) -> PathBuf {
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
pub fn manager_native_bin_dirs(
    scope: &Scope,
    env: &EnvMap,
) -> Vec<PathBuf> {
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
