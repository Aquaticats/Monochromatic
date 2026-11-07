//! What:
//!  The proof, from files alone, that no configuration scope Git reads for repository discovery sets `core.bare` or
//!  `core.worktree`.
//! Why:
//!  `git rev-parse` classifies a repository from the discovered directory plus those two keys of any scope;
//!  the in-process location reader may only answer when none of the files it can see mentions either key,
//!  so the check follows every `include` and `includeIf` path without evaluating its condition (a
//!  conditional include that mentions a key is enough to leave the question to Git).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const safe: boolean | undefined = noScopeSetsBareOrWorktree({ home, env }); // undefined: ask Git
//! ```

use crate::repo_read_config_text::{CONFIG_TEXT_LIMIT, ConfigEntry, parse_bool, parse_config_text};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How many nested includes are followed before the reader gives up.
const INCLUDE_DEPTH_LIMIT: usize = 10;

/// The keys whose presence in any file makes the location reader decline.
const DISCOVERY_KEYS: [&str; 2] = ["core.bare", "core.worktree"];

/// The keys whose presence in any file makes the content readers decline (listings, cached diffs, `show`).
const CONTENT_KEYS: [&str; 11] = [
    "diff.relative",
    "core.ignorecase",
    "core.precomposeunicode",
    "core.sparsecheckout",
    "core.splitindex",
    "index.sparse",
    "diff.orderfile",
    "diff.renames",
    "diff.ignoresubmodules",
    "log.showsignature",
    "i18n.logoutputencoding",
];

/// What:
///  The text of a configuration file, following links, or `Some(None)` when the file does not exist.
/// Why:
///   Dotfile managers link `~/.gitconfig`; Git follows the link, so does this read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = await readFile(path).catch(notFoundIsUndefined); // other failures: ask Git
/// ```
pub fn read_following(path: &Path) -> Option<Option<Vec<u8>>> {
    let metadata = match std::fs::metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(None),
        Err(_) => return None,
    };
    if !metadata.is_file() || metadata.len() > CONFIG_TEXT_LIMIT as u64 {
        return None;
    }
    return std::fs::read(path).ok().map(Some);
}

/// The path an include value names: `~/x` below `home`, an absolute path as is, a relative path beside the including file.
pub fn include_target(value: &[u8], including: &Path, home: Option<&Path>) -> Option<PathBuf> {
    let text: &str = std::str::from_utf8(value).ok()?;
    if let Some(rest) = text.strip_prefix("~/") {
        return Some(home?.join(rest));
    }
    if text.starts_with('~') || text.is_empty() {
        return None;
    }
    let path: &Path = Path::new(text);
    if path.is_absolute() {
        return Some(path.to_path_buf());
    }
    return Some(including.parent()?.join(path));
}

/// Whether an entry key names an include path (`include.path` or `includeif.<condition>.path`).
pub fn is_include_key(key: &str) -> bool {
    return key == "include.path" || (key.starts_with("includeif.") && key.ends_with(".path"));
}

/// Whether a key decides how Git discovers the repository.
pub fn discovery_key(key: &str) -> bool {
    return DISCOVERY_KEYS.contains(&key);
}

/// Whether a key can change what the content queries (listings, cached diffs, `show`) print.
pub fn content_key(key: &str) -> bool {
    return CONTENT_KEYS.contains(&key) || (key.starts_with("submodule.") && key.ends_with(".ignore"));
}

/// What:
///  Whether the file at `path`, and every file it includes, avoids every key `declines` names;
///  `None` when any file cannot be read or parsed.
/// Why:
///   A missing file is an empty file to Git, so it passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function avoidsKeys(path: string, declines: (key: string) => boolean, depth = 0): boolean | undefined;
/// ```
pub fn file_avoids_keys(path: &Path, home: Option<&Path>, depth: usize, declines: &dyn Fn(&str) -> bool) -> Option<bool> {
    if depth > INCLUDE_DEPTH_LIMIT {
        return None;
    }
    let Some(text) = read_following(path)? else {
        return Some(true);
    };
    let entries: Vec<ConfigEntry> = parse_config_text(text.as_slice())?;
    return entries_avoid_keys(entries.as_slice(), path, home, depth, declines);
}

/// The same proof for entries already parsed from `path` (so the repository's own file is read once).
pub fn entries_avoid_keys(
    entries: &[ConfigEntry],
    path: &Path,
    home: Option<&Path>,
    depth: usize,
    declines: &dyn Fn(&str) -> bool,
) -> Option<bool> {
    for entry in entries {
        if declines(entry.key.as_str()) {
            return Some(false);
        }
        if is_include_key(entry.key.as_str()) {
            let target: PathBuf = include_target(entry.value.as_deref()?, path, home)?;
            if !file_avoids_keys(target.as_path(), home, depth + 1, declines)? {
                return Some(false);
            }
        }
    }
    return Some(true);
}

/// The discovery proof for one file.
pub fn file_avoids_discovery_keys(path: &Path, home: Option<&Path>, depth: usize) -> Option<bool> {
    return file_avoids_keys(path, home, depth, &discovery_key);
}

/// What:
///  The environment facts that decide which system and global configuration files Git reads.
/// Why:
///   `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_SYSTEM` and `GIT_CONFIG_GLOBAL` move or remove the files, and the XDG file is
///  read next to `~/.gitconfig` unless `GIT_CONFIG_GLOBAL` replaces both; a value type lets a control inject them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScopeEnvironment = { home?: string; xdg?: string; noSystem: boolean; system?: string; global?: string };
/// ```
#[derive(Clone, Debug, Default)]
pub struct ScopeEnvironment {
    /// `HOME`, when set and not empty.
    pub home: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`, when set and not empty.
    pub xdg: Option<PathBuf>,
    /// Whether `GIT_CONFIG_NOSYSTEM` is set.
    pub no_system: bool,
    /// `GIT_CONFIG_SYSTEM`.
    pub system: Option<PathBuf>,
    /// The path the real Git compiles in, read when `GIT_CONFIG_SYSTEM` is not set; learned from that Git
    ///  (`git_system_config`), and `None` when it is not known, which makes the scope unresolvable.
    pub system_default: Option<PathBuf>,
    /// `GIT_CONFIG_GLOBAL`.
    pub global: Option<PathBuf>,
    /// Whether some fact cannot be settled from the variables alone: a relative path (Git resolves it against the
    /// directory it runs in) or a `GIT_CONFIG_NOSYSTEM` that is not a boolean. Readers that need the files decline.
    pub unresolvable: bool,
}

/// The value of the last variable named `name`, or nothing; later entries override earlier ones, as in a child's environment.
pub fn last_variable(variables: &[(OsString, OsString)], name: &str) -> Option<OsString> {
    for (candidate, value) in variables.iter().rev() {
        if candidate == name {
            return Some(value.clone());
        }
    }
    return None;
}

/// The last variable named `name` as a path, or nothing when it is unset.
fn path_variable(variables: &[(OsString, OsString)], name: &str) -> Option<PathBuf> {
    if let Some(value) = last_variable(variables, name) {
        return Some(PathBuf::from(value));
    }
    return None;
}

/// The last variable named `name` as a path, or nothing when it is unset or empty.
fn nonempty_variable(variables: &[(OsString, OsString)], name: &str) -> Option<PathBuf> {
    if let Some(value) = last_variable(variables, name)
        && !value.is_empty()
    {
        return Some(PathBuf::from(value));
    }
    return None;
}

impl ScopeEnvironment {
    /// The facts of an environment as a child would see it: the process variables with a run's additions over them, which
    ///  include the forward-target variable naming the real Git; the wrapper's own environment does not carry it, so there is
    ///  deliberately no constructor that reads the process environment alone.
    pub fn from_variables(variables: &[(OsString, OsString)]) -> ScopeEnvironment {
        // `GIT_CONFIG_NOSYSTEM` is a boolean: absent or false reads the system file, an empty value is false.
        let (no_system, boolean_ok): (bool, bool) = match last_variable(variables, "GIT_CONFIG_NOSYSTEM") {
            None => (false, true),
            Some(value) if value.is_empty() => (false, true),
            Some(value) => match parse_bool(Some(value.as_encoded_bytes())) {
                Some(truth) => (truth, true),
                None => (true, false),
            },
        };
        let mut scope: ScopeEnvironment = ScopeEnvironment {
            home: nonempty_variable(variables, "HOME"),
            xdg: nonempty_variable(variables, "XDG_CONFIG_HOME"),
            no_system,
            system: path_variable(variables, "GIT_CONFIG_SYSTEM"),
            system_default: None,
            global: path_variable(variables, "GIT_CONFIG_GLOBAL"),
            unresolvable: !boolean_ok,
        };
        // Git reads its compiled-in system file unless the environment removes or replaces it.
        if boolean_ok && !scope.no_system && scope.system.is_none() {
            scope.system_default = crate::git_system_config::system_config_for(variables);
            scope.unresolvable = scope.system_default.is_none();
        }
        let mut relative: bool = false;
        for path in [&scope.home, &scope.xdg, &scope.system, &scope.global] {
            if let Some(found) = path.as_ref()
                && !found.is_absolute()
            {
                relative = true;
            }
        }
        scope.unresolvable = scope.unresolvable || relative;
        return scope;
    }

    /// The files with the scope name `config --show-scope` gives each, in Git's order.
    pub fn labelled_files(&self) -> Vec<(&'static str, PathBuf)> {
        let mut labelled: Vec<(&'static str, PathBuf)> = Vec::new();
        let mut files: Vec<PathBuf> = self.files();
        if !self.no_system && (self.system.is_some() || self.system_default.is_some()) {
            labelled.push(("system", files.remove(0)));
        }
        for file in files {
            labelled.push(("global", file));
        }
        return labelled;
    }

    /// The system and global files Git reads in this environment, in its order.
    pub fn files(&self) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = Vec::new();
        if !self.no_system {
            if let Some(system) = self.system.clone() {
                files.push(system);
            } else if let Some(default) = self.system_default.clone() {
                files.push(default);
            }
        }
        if let Some(global) = &self.global {
            files.push(global.clone());
            return files;
        }
        match (&self.xdg, &self.home) {
            (Some(directory), _) => files.push(directory.join("git/config")),
            (None, Some(base)) => files.push(base.join(".config/git/config")),
            (None, None) => {}
        }
        if let Some(base) = &self.home {
            files.push(base.join(".gitconfig"));
        }
        return files;
    }
}

/// What:
///  Whether the system and global configuration files, with everything they include, avoid the discovery keys.
/// Why:
///   The repository-level files are checked by the layout reader; these are the scopes beyond the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scopeFiles(env).every((file) => avoidsDiscoveryKeys(file)); // undefined when unreadable
/// ```
pub fn outer_scopes_avoid_keys(scope: &ScopeEnvironment, declines: &dyn Fn(&str) -> bool) -> Option<bool> {
    if scope.unresolvable {
        return None;
    }
    for file in scope.files() {
        if !file_avoids_keys(file.as_path(), scope.home.as_deref(), 0, declines)? {
            return Some(false);
        }
    }
    return Some(true);
}

#[cfg(test)]
#[path = "repo_read_config_scan_tests.rs"]
mod tests;
