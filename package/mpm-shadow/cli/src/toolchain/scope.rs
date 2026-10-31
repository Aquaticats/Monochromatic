//! What:
//! The two scopes the toolchain commands work in (this repository's worktree,
//! or the machine-global scope
//!           under the per-user config directory),
//! and every path each scope uses,
//! computed from an injected
//!           environment.
//! Why:
//! Tests pass a disposable home and a fake environment,
//! so no test reads or writes the real `~/.config`,
//!           `~/.local` or `~/.cargo`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // scope.ts
//! export function repoScope(root: string, env: EnvMap): Scope
//! export function globalScope(env: EnvMap): Scope
//! ```

/// Imports the sorted map the environment is held in (sorted so child-process environments are deterministic).
use std::collections::BTreeMap;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;

/// The file name of the plan in both scopes.
pub const PLAN_FILE_NAME: &str = "toolchain.plan.jsonc";

/// The file name of the lock in both scopes.
pub const LOCK_FILE_NAME: &str = "toolchain.lock.json";

/// The exposure directory of the repository scope,
/// relative to the worktree (the IDE launcher parses this value).
pub const EXPOSURE_DIR_RELATIVE: &str = ".cache/toolchain-bin";

/// What:
/// An environment as plain data:
/// names to values,
/// empty values counted as unset.
/// Why:
/// Code that reads `HOME` or `CARGO_HOME` takes this instead of calling `std::env::var`,
/// so a test builds
///           one from literals and the real process environment is read in exactly one place (`from_process`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EnvMap = Readonly<Record<string, string>>;
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnvMap {
    /// Names to values.
    values: BTreeMap<String, String>,
}

/// The methods of `EnvMap`.
impl EnvMap {
    /// The real process environment (variables that are not valid UTF-8 are skipped).
    pub fn from_process() -> EnvMap {
        let mut values: BTreeMap<String, String> = BTreeMap::new();
        for (name, value) in std::env::vars() {
            values.insert(name, value);
        }
        return EnvMap { values };
    }

    /// An environment built from literal pairs (for tests).
    pub fn from_pairs(pairs: &[(&str, &str)]) -> EnvMap {
        let mut values: BTreeMap<String, String> = BTreeMap::new();
        for (name, value) in pairs {
            values.insert((*name).to_string(), (*value).to_string());
        }
        return EnvMap { values };
    }

    /// The value of `name`,
    /// or `None` when it is unset or empty.
    pub fn get(&self, name: &str) -> Option<&str> {
        return match self.values.get(name) {
            Some(value) if !value.is_empty() => Some(value.as_str()),
            _ => None,
        };
    }

    /// A copy with `name` set to `value`.
    pub fn with(&self, name: &str, value: &str) -> EnvMap {
        let mut values: BTreeMap<String, String> = self.values.clone();
        values.insert(name.to_string(), value.to_string());
        return EnvMap { values };
    }

    /// Every variable,
    /// for building a child process environment.
    pub fn all(&self) -> &BTreeMap<String, String> {
        return &self.values;
    }
}

/// Which of the two scopes a command works in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeKind {
    /// This repository's worktree:
    /// `toolchain.plan.jsonc` and `toolchain.lock.json` at the root.
    Repo,
    /// The machine:
    /// the same two files in `${XDG_CONFIG_HOME:-$HOME/.config}/monochromatic-dev/toolchain`.
    Global,
}

/// What:
/// Every path one scope reads or writes.
/// Why:
/// Commands take a `Scope` and never build a path from the environment themselves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Scope = { kind: 'repo' | 'global'; root: string; planFile: string; lockFile: string; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Scope {
    /// Which scope this is.
    pub kind: ScopeKind,
    /// The worktree root (repository scope) or the config directory (global scope):
    /// plan and lock live here.
    pub root: PathBuf,
    /// The plan file people edit.
    pub plan_file: PathBuf,
    /// The lock file only mpm-shadow writes.
    pub lock_file: PathBuf,
    /// The file whose OS lock serializes mutations of the plan's lock in this scope.
    pub guard_file: PathBuf,
    /// Where tool payloads are installed (`.../monochromatic-dev/toolchain` or `toolchain-global`).
    pub toolchain_root: PathBuf,
    /// The directory of links to every declared bin.
    pub exposure_dir: PathBuf,
    /// The exposure directory as the IDE launcher reads it:
    /// relative to the worktree in the repository scope.
    pub exposure_dir_relative: String,
    /// The directory postinstall links go to when the link names none.
    pub link_dir: PathBuf,
    /// The directory the `bin` CLI downloads into.
    pub toolchain_bin_dir: PathBuf,
    /// The `bin` CLI's configuration file.
    pub bin_config_file: PathBuf,
    /// The user's home directory.
    pub home: PathBuf,
}

/// What:
/// The user's home directory from `HOME`,
/// or `USERPROFILE` on Windows.
/// Why:
/// The data and config defaults hang off it;
/// a missing home is a configuration error naming both variables.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function homeDir(env: EnvMap): string  // throws when unset
/// ```
pub fn home_dir(env: &EnvMap) -> Result<PathBuf, ToolchainError> {
    if let Some(home) = env.get("HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Some(profile) = env.get("USERPROFILE") {
        return Ok(PathBuf::from(profile));
    }
    return Err(ToolchainError::config("neither HOME nor USERPROFILE is set, so the user's directories cannot be found"));
}

/// `${XDG_DATA_HOME:-$HOME/.local/share}`.
fn data_home(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("XDG_DATA_HOME") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".local").join("share"),
    };
}

/// `${XDG_CONFIG_HOME:-$HOME/.config}`.
fn config_home(env: &EnvMap, home: &Path) -> PathBuf {
    return match env.get("XDG_CONFIG_HOME") {
        Some(directory) => PathBuf::from(directory),
        None => home.join(".config"),
    };
}

/// What:
/// The scope of the worktree at `root`.
/// Why:
/// Plan and lock sit at the worktree root;
/// the guard file sits in the gitignored `.cache`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function repoScope(root: string, env: EnvMap): Scope
/// ```
pub fn repo_scope(root: &Path, env: &EnvMap) -> Result<Scope, ToolchainError> {
    let home: PathBuf = home_dir(env)?;
    let toolchain_root: PathBuf = data_home(env, &home).join("monochromatic-dev").join("toolchain");
    return Ok(Scope {
        kind: ScopeKind::Repo,
        root: root.to_path_buf(),
        plan_file: root.join(PLAN_FILE_NAME),
        lock_file: root.join(LOCK_FILE_NAME),
        guard_file: root.join(".cache").join("toolchain.guard"),
        exposure_dir: root.join(".cache").join("toolchain-bin"),
        exposure_dir_relative: EXPOSURE_DIR_RELATIVE.to_string(),
        link_dir: root.join(".cache").join("cli-git-bin"),
        toolchain_bin_dir: toolchain_root.join("bin"),
        bin_config_file: toolchain_root.join("bin.config.json"),
        toolchain_root,
        home,
    });
}

/// What:
/// The machine-global scope.
/// Why:
/// Its plan and lock are the user's own files under the config directory;
/// payloads install under the data
///           directory in `toolchain-global`,
/// apart from the repository scope's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function globalScope(env: EnvMap): Scope
/// ```
pub fn global_scope(env: &EnvMap) -> Result<Scope, ToolchainError> {
    let home: PathBuf = home_dir(env)?;
    let config_dir: PathBuf = config_home(env, &home).join("monochromatic-dev").join("toolchain");
    let toolchain_root: PathBuf = data_home(env, &home).join("monochromatic-dev").join("toolchain-global");
    return Ok(Scope {
        kind: ScopeKind::Global,
        root: config_dir.clone(),
        plan_file: config_dir.join(PLAN_FILE_NAME),
        lock_file: config_dir.join(LOCK_FILE_NAME),
        guard_file: config_dir.join(".toolchain.guard"),
        exposure_dir: toolchain_root.join("exposure"),
        exposure_dir_relative: "exposure".to_string(),
        link_dir: home.join(".local").join("bin"),
        toolchain_bin_dir: toolchain_root.join("bin"),
        bin_config_file: toolchain_root.join("bin.config.json"),
        toolchain_root,
        home,
    });
}

/// What:
/// Finds the worktree root:
/// `MONOCHROMATIC_WORKTREE_ROOT` when set,
/// otherwise the nearest directory at or
///           above `start` that holds `toolchain.plan.jsonc`.
/// Why:
/// The IDE launcher and a person at a shell both start somewhere inside the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findRepoRoot(start: string, env: EnvMap): string  // throws when none
/// ```
pub fn find_repo_root(start: &Path, env: &EnvMap) -> Result<PathBuf, ToolchainError> {
    if let Some(root) = env.get("MONOCHROMATIC_WORKTREE_ROOT") {
        return Ok(PathBuf::from(root));
    }
    for directory in start.ancestors() {
        if directory.join(PLAN_FILE_NAME).is_file() {
            return Ok(directory.to_path_buf());
        }
    }
    return Err(ToolchainError::config(format!(
        "no {PLAN_FILE_NAME} in {} or any parent directory; run mpm-shadow inside the repository, \
         set MONOCHROMATIC_WORKTREE_ROOT, or use --global",
        start.display()
    )));
}

/// The scope for a command:
/// the global scope when `global` is set,
/// otherwise the repository scope found from `start`.
pub fn resolve_scope(global: bool, start: &Path, env: &EnvMap) -> Result<Scope, ToolchainError> {
    if global {
        return global_scope(env);
    }
    let root: PathBuf = find_repo_root(start, env)?;
    return repo_scope(&root, env);
}
