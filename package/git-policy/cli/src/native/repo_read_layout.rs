//! What:
//!  The repository location that `git rev-parse --path-format=absolute --is-bare-repository --git-dir --git-common-dir
//!  --show-toplevel --show-prefix` prints, found from the file system without a Git process.
//! Why:
//!  Every wrapped command asks for it, and it is the only extra Git start of most forwarded commands.
//!  The reader walks up from the directory looking for `.git`, checks what Git checks for the plain layouts
//!  (a valid `HEAD`, the objects and refs directories, files ref storage, a known object format, the owner of every
//!  path, no `core.bare` or `core.worktree` in any scope), and returns `None` for everything else, which sends the
//!  caller to Git: a bare repository, `GIT_*` variables, a symbolic link, another file system, reftable, an unknown
//!  extension, a submodule that sets `core.worktree`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const layout: RepoLayout | undefined = readLayout({ globalPrefix, overlay }); // undefined: ask Git
//! ```

use super::child_environment::{COUNT_VARIABLE, environment_value, parse_config_count};
use super::private_storage::read_regular_file;
use super::repo_read_hook::enabled;
use super::repo_read_config_scan::{ScopeEnvironment, discovery_key, entries_avoid_keys, outer_scopes_avoid_keys};
use super::repo_read_config_text::{CONFIG_TEXT_LIMIT, ConfigEntry, last_entry, parse_bool, parse_config_text};
use super::repo_read_config_typed::{entries_are_accepted, environment_is_accepted, outer_scopes_are_accepted};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The environment variables a plain read tolerates, by prefix; any other `GIT_` variable can move a path or a key.
const PLAIN_ENVIRONMENT_PREFIXES: [&str; 19] = [
    "GIT_AUTHOR_",
    "GIT_COMMITTER_",
    "GIT_TERMINAL_PROMPT",
    "GIT_ASKPASS",
    "GIT_PAGER",
    "GIT_EDITOR",
    "GIT_SEQUENCE_EDITOR",
    "GIT_EXEC_PATH",
    "GIT_SSH",
    "GIT_ALLOW_PROTOCOL",
    "GIT_LFS_",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_KEY_",
    "GIT_CONFIG_VALUE_",
    "GIT_MERGE_AUTOEDIT",
    "GIT_OPTIONAL_LOCKS",
];

/// The longest `HEAD` or `commondir` or gitfile this reader looks at.
const SMALL_FILE_LIMIT: u64 = 4096;

/// How many environment-supplied configuration entries the reader inspects before it declines.
const ENV_CONFIG_ENTRY_LIMIT: u64 = 1024;

/// The hash algorithms of object names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectFormat {
    /// 20-byte names, 40 hexadecimal digits.
    Sha1,
    /// 32-byte names, 64 hexadecimal digits.
    Sha256,
}

impl ObjectFormat {
    /// The name `rev-parse --show-object-format` prints.
    pub fn name(self) -> &'static str {
        return match self {
            ObjectFormat::Sha1 => "sha1",
            ObjectFormat::Sha256 => "sha256",
        };
    }

    /// The number of hexadecimal digits of one object name.
    pub fn hex_length(self) -> usize {
        return match self {
            ObjectFormat::Sha1 => 40,
            ObjectFormat::Sha256 => 64,
        };
    }
}

/// What:
///  Where one worktree and its repository are, as Git reports them with absolute paths.
/// Why:
///   One value answers the location, the Git paths and the object format queries of a command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RepoLayout = { gitDir: string; commonDir: string; worktreeRoot: string; prefix: Uint8Array; objectFormat: 'sha1' | 'sha256' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoLayout {
    /// The real path of the worktree's Git directory.
    pub git_dir: PathBuf,
    /// The real path of the directory that holds the objects and the shared refs.
    pub common_dir: PathBuf,
    /// The real path of the worktree's top level.
    pub worktree_root: PathBuf,
    /// The start directory below the top level, with a trailing `/`, or empty at the top level.
    pub prefix: Vec<u8>,
    /// The object hash of the repository.
    pub object_format: ObjectFormat,
}

/// The keys an environment-supplied configuration entry may not set without the reader declining.
fn env_key_moves_discovery(key: &str) -> bool {
    let lowered: String = key.to_ascii_lowercase();
    return lowered == "core.bare" || lowered == "core.worktree" || lowered.starts_with("include");
}

/// Whether the `GIT_CONFIG_COUNT` entries of `variables` (the wrapper injects some) all leave discovery alone.
fn env_config_is_plain(variables: &[(OsString, OsString)]) -> bool {
    return env_config_avoids(variables, &env_key_moves_discovery);
}

/// What:
///  Whether every environment-supplied configuration entry has a key `declines` does not name;
///  a malformed set of entries is not plain either.
/// Why:
///   The content queries use the same walk with their own key list, so an injected `diff.orderFile` is not missed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// envConfigKeys(variables).every((key) => !declines(key)); // false when malformed
/// ```
pub fn env_config_avoids(variables: &[(OsString, OsString)], declines: &dyn Fn(&str) -> bool) -> bool {
    let Some(count_text) = environment_value(variables, COUNT_VARIABLE) else {
        return true;
    };
    let Some(count) = parse_config_count(count_text.as_os_str()) else {
        return false;
    };
    if count > ENV_CONFIG_ENTRY_LIMIT {
        return false;
    }
    for index in 0..count {
        let Some(key) = environment_value(variables, format!("GIT_CONFIG_KEY_{index}").as_str()) else {
            return false;
        };
        match key.to_str() {
            Some(text) if !declines(text) => {}
            _ => return false,
        }
    }
    return true;
}

/// Whether every `GIT_` variable among `variables` is one that cannot move discovery.
fn environment_is_plain(variables: &[(OsString, OsString)]) -> bool {
    for (name, _) in variables {
        let Some(text) = name.to_str() else {
            continue;
        };
        if text.starts_with("GIT_") && !PLAIN_ENVIRONMENT_PREFIXES.iter().any(|prefix| { return text.starts_with(prefix); }) {
            return false;
        }
    }
    return env_config_is_plain(variables);
}

/// The environment Git would see: the process environment with the child overlay applied over it.
fn effective_environment(overlay: &[(OsString, OsString)]) -> Vec<(OsString, OsString)> {
    let mut variables: Vec<(OsString, OsString)> = std::env::vars_os()
        .filter(|(name, _)| { return !overlay.iter().any(|(added, _)| { return added == name; }); })
        .collect();
    variables.extend(overlay.iter().cloned());
    return variables;
}

/// The directory a command starts in: the process directory moved by leading `-C <path>` options.
///  Any other global option returns `None`.
fn start_directory(global_prefix: &[OsString]) -> Option<PathBuf> {
    let mut directory: PathBuf = std::env::current_dir().ok()?;
    let mut options = global_prefix.iter();
    while let Some(option) = options.next() {
        if option != "-C" {
            return None;
        }
        let path: &OsString = options.next()?;
        if !path.is_empty() {
            directory = directory.join(path);
        }
        // Git changes directory at each `-C` in turn and fails at the first one that is not a directory.
        if !std::fs::metadata(&directory).is_ok_and(|found| { return found.is_dir(); }) {
            return None;
        }
    }
    return std::fs::canonicalize(directory).ok();
}

/// A small regular file's text, refusing links.
fn small_file(path: &Path) -> Option<String> {
    let bytes: Vec<u8> = read_regular_file(path, SMALL_FILE_LIMIT).ok()?;
    return String::from_utf8(bytes).ok();
}

/// Whether `directory` holds the three entries Git takes for a Git directory.
fn has_git_directory_entries(directory: &Path) -> bool {
    return directory.join("HEAD").exists() && directory.join("objects").is_dir() && directory.join("refs").is_dir();
}

/// Whether the `HEAD` text is one of the two spellings Git accepts that this reader knows.
fn head_is_valid(text: &str, format: ObjectFormat) -> bool {
    let Some(line) = text.strip_suffix('\n') else {
        return false;
    };
    if let Some(name) = line.strip_prefix("ref: ") {
        return name.starts_with("refs/") && !name.contains('\n');
    }
    return line.len() == format.hex_length() && line.bytes().all(|byte| { return byte.is_ascii_hexdigit(); });
}

/// Whether `path` is owned by the effective user, as Git's ownership check requires.
#[cfg(unix)]
fn owned_by_current_user(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let Ok(metadata) = std::fs::symlink_metadata(path) else {
        return false;
    };
    // SAFETY: `geteuid` takes no argument, touches no memory and cannot fail.
    return metadata.uid() == unsafe { libc::geteuid() };
}

/// Without user ids, every repository is left to Git.
#[cfg(not(unix))]
fn owned_by_current_user(_path: &Path) -> bool {
    return false;
}

/// The device a directory lives on, so a walk can stop where Git stops.
#[cfg(unix)]
fn device_of(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    return std::fs::metadata(path).ok().map(|metadata| { return metadata.dev(); });
}

/// Without device numbers, every walk is left to Git.
#[cfg(not(unix))]
fn device_of(_path: &Path) -> Option<u64> {
    return None;
}

/// The Git directory a `.git` entry names: the entry itself when it is a directory,
///  the `gitdir:` line of a plain file, `None` for a link or anything else.
fn dot_git_target(worktree: &Path) -> Option<Option<(PathBuf, bool)>> {
    let dot_git: PathBuf = worktree.join(".git");
    let metadata = match std::fs::symlink_metadata(&dot_git) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(None),
        Err(_) => return None,
    };
    if metadata.is_dir() {
        return Some(Some((dot_git, false)));
    }
    if !metadata.is_file() {
        return None;
    }
    let text: String = small_file(dot_git.as_path())?;
    let target: &str = text.strip_prefix("gitdir: ")?.strip_suffix('\n')?;
    if target.is_empty() || target.contains('\n') {
        return None;
    }
    return Some(Some((worktree.join(target), true)));
}

/// What:
///  The repository format keys the layout needs from the repository's own configuration file.
/// Why:
///   The object format decides the length of a name; any other extension may change how the repository is read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const format: ObjectFormat | undefined = readFormat(entries); // undefined: ask Git
/// ```
fn object_format_of(entries: &[ConfigEntry]) -> Option<ObjectFormat> {
    // A bare `core.repositoryformatversion` is an error in Git, not an absent key.
    let version: Option<&[u8]> = match last_entry(entries, "core.repositoryformatversion") {
        Some(entry) => Some(entry.value.as_deref()?),
        None => None,
    };
    let version_one: bool = match version {
        None | Some(b"0") => false,
        Some(b"1") => true,
        Some(_) => return None,
    };
    let mut format: ObjectFormat = ObjectFormat::Sha1;
    for entry in entries.iter().filter(|entry| { return entry.key.starts_with("extensions."); }) {
        if !version_one {
            return None;
        }
        let value: &[u8] = entry.value.as_deref().unwrap_or_default();
        match (entry.key.as_str(), value) {
            ("extensions.noop", _) => {}
            ("extensions.preciousobjects", _) if parse_bool(entry.value.as_deref()).is_some() => {}
            ("extensions.objectformat", b"sha1") => format = ObjectFormat::Sha1,
            ("extensions.objectformat", b"sha256") => format = ObjectFormat::Sha256,
            ("extensions.refstorage", b"files") => {}
            _ => return None,
        }
    }
    return Some(format);
}

/// What:
///  Check the discovered Git directory and build the layout.
/// Why:
///   Everything beyond the plain layout returns `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function finish(start: string, worktree: string, gitDir: string, fromFile: boolean): RepoLayout | undefined;
/// ```
fn finish(start: &Path, worktree: &Path, entry: (PathBuf, bool), scope: &ScopeEnvironment) -> Option<RepoLayout> {
    let (target, from_file) = entry;
    let git_dir: PathBuf = std::fs::canonicalize(target).ok()?;
    let dot_git: PathBuf = worktree.join(".git");
    let owners_ok: bool = owned_by_current_user(worktree)
        && owned_by_current_user(git_dir.as_path())
        && (!from_file || owned_by_current_user(dot_git.as_path()));
    if !owners_ok {
        return None;
    }
    let common_dir: PathBuf = match small_file(git_dir.join("commondir").as_path()) {
        Some(text) => std::fs::canonicalize(git_dir.join(text.strip_suffix('\n')?)).ok()?,
        None if git_dir.join("commondir").exists() => return None,
        None => git_dir.clone(),
    };
    if git_dir.join("reftable").exists() || common_dir.join("reftable").exists() || !has_git_directory_entries_split(&git_dir, &common_dir) {
        return None;
    }
    let config_bytes: Vec<u8> = read_regular_file(common_dir.join("config").as_path(), CONFIG_TEXT_LIMIT as u64).ok()?;
    let entries: Vec<ConfigEntry> = parse_config_text(config_bytes.as_slice())?;
    let object_format: ObjectFormat = object_format_of(entries.as_slice())?;

    // The repository's own file carries `bare = false` after `init`; that spelling is the plain layout.
    let discoverable: Vec<ConfigEntry> = entries
        .iter()
        .filter(|candidate| { return !(candidate.key == "core.bare" && parse_bool(candidate.value.as_deref()) == Some(false)); })
        .cloned()
        .collect();
    let local_ok: bool = entries_avoid_keys(discoverable.as_slice(), common_dir.join("config").as_path(), scope.home.as_deref(), 0, &discovery_key)?;
    let worktree_config: bool = last_entry(entries.as_slice(), "extensions.worktreeconfig").is_some();
    if !local_ok || worktree_config || !outer_scopes_avoid_keys(scope, &discovery_key)? {
        return None;
    }
    // Git types many keys for every command and fails, or warns, on a value that does not fit; the readers leave
    // such a repository to Git, which words its own failure.
    let typed_ok: bool = entries_are_accepted(entries.as_slice(), common_dir.join("config").as_path(), scope.home.as_deref(), 0)? && outer_scopes_are_accepted(scope)?;
    if !typed_ok {
        return None;
    }
    let head: String = small_file(git_dir.join("HEAD").as_path())?;
    if !head_is_valid(head.as_str(), object_format) {
        return None;
    }
    let relative: &Path = start.strip_prefix(worktree).ok()?;
    let mut prefix: Vec<u8> = Vec::new();
    for component in relative.components() {
        prefix.extend_from_slice(component.as_os_str().as_encoded_bytes());
        prefix.push(b'/');
    }
    return Some(RepoLayout { git_dir, common_dir, worktree_root: worktree.to_path_buf(), prefix, object_format });
}

/// The three-entry check, with `HEAD` in the worktree's directory and the rest in the common directory.
fn has_git_directory_entries_split(git_dir: &Path, common_dir: &Path) -> bool {
    return git_dir.join("HEAD").exists() && common_dir.join("objects").is_dir() && common_dir.join("refs").is_dir();
}

/// What:
///  The layout of the repository a command run with `global_prefix` and `overlay` starts in,
///  or `None` when Git must be asked.
/// Why:
///   This is the one entry point the callers use; it never errors, because "cannot tell" and "not a repository" both
///  mean "ask Git", which words the failure itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayout(globalPrefix, overlay); // undefined: ask Git
/// ```
pub fn read_layout(global_prefix: &[OsString], overlay: &[(OsString, OsString)]) -> Option<RepoLayout> {
    if !enabled(overlay) {
        return None;
    }
    let variables: Vec<(OsString, OsString)> = effective_environment(overlay);
    if !environment_is_plain(variables.as_slice()) || !environment_is_accepted(variables.as_slice()) {
        return None;
    }
    let start: PathBuf = start_directory(global_prefix)?;
    return walk_up(start.as_path(), &ScopeEnvironment::from_variables(variables.as_slice()));
}

/// What:
///  The read with its environment injected: the variables Git would see and the configuration scope facts.
/// Why:
///   The controls run in a process whose own environment and home they must not depend on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutWith(start, variables, scope); // undefined: ask Git
/// ```
pub fn read_layout_with(start: &Path, variables: &[(OsString, OsString)], scope: &ScopeEnvironment) -> Option<RepoLayout> {
    if !environment_is_plain(variables) || !environment_is_accepted(variables) {
        return None;
    }
    return walk_up(std::fs::canonicalize(start).ok()?.as_path(), scope);
}

/// What:
///  The same read for a command run in `directory` without global options.
/// Why:
///   The transaction Git requests name their directory and leave the invocation's global options out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const layout = readLayoutIn(directory, overlay); // undefined: ask Git
/// ```
pub fn read_layout_in(directory: &Path, overlay: &[(OsString, OsString)]) -> Option<RepoLayout> {
    if !enabled(overlay) {
        return None;
    }
    let variables: Vec<(OsString, OsString)> = effective_environment(overlay);
    return read_layout_with(directory, variables.as_slice(), &ScopeEnvironment::from_variables(variables.as_slice()));
}

/// Walk from `start` to the root looking for the first `.git`, as Git does, and build the layout there.
fn walk_up(start: &Path, scope: &ScopeEnvironment) -> Option<RepoLayout> {
    let mut directory: PathBuf = start.to_path_buf();
    loop {
        if let Some(entry) = dot_git_target(directory.as_path())? {
            return finish(start, directory.as_path(), entry, scope);
        }
        if has_git_directory_entries(directory.as_path()) {
            return None;
        }
        let parent: PathBuf = directory.parent()?.to_path_buf();
        if device_of(parent.as_path())? != device_of(directory.as_path())? {
            return None;
        }
        directory = parent;
    }
}

/// What:
///  The absolute path `git rev-parse --path-format=absolute --git-path <name>` prints for the names this reader knows
///  in a repository whose worktree owns its Git directory.
/// Why:
///   A linked worktree splits names between its own directory and the common one by a table in Git;
///  those answers stay with Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitPath = (layout, name) => layout.gitDir === layout.commonDir && KNOWN.has(name) ? join(layout.gitDir, name) : undefined;
/// ```
pub fn git_path(layout: &RepoLayout, name: &str) -> Option<PathBuf> {
    const KNOWN: [&str; 9] = [
        "index",
        "objects",
        "HEAD",
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "cli-git-transactions",
        "packed-refs",
        "config",
    ];
    if layout.git_dir != layout.common_dir || !KNOWN.contains(&name) {
        return None;
    }
    let path: PathBuf = layout.git_dir.join(name);
    // With `--path-format=absolute` Git prints the target of a link, so a linked name is left to Git.
    if std::fs::symlink_metadata(path.as_path()).is_ok_and(|metadata| { return metadata.file_type().is_symlink(); }) {
        return None;
    }
    return Some(path);
}

#[cfg(test)]
#[path = "repo_read_layout_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "repo_read_layout_worktree_tests.rs"]
mod worktree_tests;
