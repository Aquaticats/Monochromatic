//! What:
//!  The merged configuration listing of `git config --list -z --show-scope`,
//! built in process from the system,
//! global
//!  and repository files,
//! their `include.path` files,
//! and the environment's `GIT_CONFIG_COUNT` entries.
//! Why:
//!  The wrapper reads this listing once per commit and asks several single-key questions Git answers from the same
//!  files.
//!  The per-worktree `config.worktree` is listed under the scope `worktree` after the repository's own file when
//!  `extensions.worktreeConfig` is on.
//!  The reader declines conditional includes (whose condition Git evaluates),
//! command-line `-c` entries (`GIT_CONFIG_PARAMETERS`),
//! a file it cannot parse and an environment entry it cannot canonicalize.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = listConfig({ commonDir, gitDir, scope, variables }); // undefined: ask Git
//! ```

use crate::repo_read_config_scan::{ScopeEnvironment, include_target, is_include_key, last_variable, read_following};
use crate::repo_read_config_text::{ConfigEntry, last_entry, parse_bool, parse_config_text};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// How many nested includes are followed;
/// Git's own limit.
const INCLUDE_DEPTH_LIMIT: usize = 10;

/// How many environment-supplied entries are read.
const ENV_ENTRY_LIMIT: u64 = 1024;

/// One listed entry with the scope name `--show-scope` prints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedEntry {
    /// `system`,
    /// `global`,
    /// `local`,
    /// `worktree` or `command`.
    pub scope: &'static str,
    /// The entry.
    pub entry: ConfigEntry,
}

/// Append the entries of the file at `path` and,
/// in place,
/// of every file it includes.
fn add_file(path: &Path, scope: &'static str, home: Option<&Path>, depth: usize, listed: &mut Vec<ListedEntry>) -> Option<()> {
    if depth > INCLUDE_DEPTH_LIMIT {
        return None;
    }
    let Some(text) = read_following(path)? else {
        return Some(());
    };
    for entry in parse_config_text(text.as_slice())? {
        let include: Option<std::path::PathBuf> = if is_include_key(entry.key.as_str()) {
            if entry.key != "include.path" {
                return None;
            }
            Some(include_target(entry.value.as_deref()?, path, home)?)
        } else {
            None
        };
        listed.push(ListedEntry { scope, entry });
        if let Some(target) = include {
            add_file(target.as_path(), scope, home, depth + 1, listed)?;
        }
    }
    return Some(());
}

/// Whether a section or key name is made only of letters,
/// digits and `-`,
/// and is not empty.
fn is_plain_key_part(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    for byte in text.bytes() {
        if !(byte.is_ascii_alphanumeric() || byte == b'-') {
            return false;
        }
    }
    return true;
}

/// The canonical spelling of a configuration key,
/// in any letter case:
/// lower-case section and variable name,
/// subsection kept.
/// Why:
///   Git's configuration keys are case-insensitive in their letters,
/// so every reader that compares a key Git supplied must compare this spelling,
/// or `Attr.Tree` is not the `attr.tree` Git answers for
/// (cross-vendor review round 2,
/// finding 2).
pub fn canonical_key(key: &str) -> Option<String> {
    let (section, rest) = key.split_once('.')?;
    let (middle, name) = match rest.rsplit_once('.') {
        Some((subsection, name)) => (Some(subsection), name),
        None => (None, rest),
    };
    let middle_has_newline: bool = match middle {
        Some(text) => text.contains('\n'),
        None => false,
    };
    if !is_plain_key_part(section) || !is_plain_key_part(name) || !name.as_bytes()[0].is_ascii_alphabetic() || middle_has_newline {
        return None;
    }
    return Some(match middle {
        Some(subsection) => format!("{}.{subsection}.{}", section.to_ascii_lowercase(), name.to_ascii_lowercase()),
        None => format!("{}.{}", section.to_ascii_lowercase(), name.to_ascii_lowercase()),
    });
}

/// Append the `GIT_CONFIG_COUNT` entries of `variables` under the scope `command`.
fn add_environment(variables: &[(OsString, OsString)], listed: &mut Vec<ListedEntry>) -> Option<()> {
    if last_variable(variables, "GIT_CONFIG_PARAMETERS").is_some() {
        return None;
    }
    let Some(count_text) = last_variable(variables, "GIT_CONFIG_COUNT") else {
        return Some(());
    };
    let count: u64 = crate::child_environment::parse_config_count(count_text.as_os_str())?;
    if count > ENV_ENTRY_LIMIT {
        return None;
    }
    for index in 0..count {
        let key: OsString = last_variable(variables, format!("GIT_CONFIG_KEY_{index}").as_str())?;
        let value: OsString = last_variable(variables, format!("GIT_CONFIG_VALUE_{index}").as_str())?;
        let entry: ConfigEntry = ConfigEntry { key: canonical_key(key.to_str()?)?, value: Some(value.as_encoded_bytes().to_vec()) };
        listed.push(ListedEntry { scope: "command", entry });
    }
    return Some(());
}

/// What:
///  Every entry Git would list,
/// in its order,
/// or `None` when Git must be asked.
/// Why:
///   The order is system,
/// global,
/// local,
/// command;
/// included files follow their `include.path` entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listConfig(commonDir: string, gitDir: string, scope: ScopeEnvironment, variables: [string, string][]): ListedEntry[] | undefined;
/// ```
pub fn list_config(common_dir: &Path, git_dir: &Path, scope: &ScopeEnvironment, variables: &[(OsString, OsString)]) -> Option<Vec<ListedEntry>> {
    if scope.unresolvable {
        return None;
    }
    let mut listed: Vec<ListedEntry> = Vec::new();
    for (label, file) in scope.labelled_files() {
        add_file(file.as_path(), label, scope.home.as_deref(), 0, &mut listed)?;
    }
    let start: usize = listed.len();
    let common_file: PathBuf = common_dir.join("config");
    add_file(common_file.as_path(), "local", scope.home.as_deref(), 0, &mut listed)?;
    let mut local: Vec<ConfigEntry> = Vec::new();
    for item in &listed[start..] {
        local.push(item.entry.clone());
    }
    // Git reads the extension from the common file alone (an included file's entry is listed but not honored), so an
    // entry only an include supplies is left to Git.
    let direct_text: Vec<u8> = read_following(common_file.as_path())??;
    let direct: Vec<ConfigEntry> = parse_config_text(direct_text.as_slice())?;
    let listed_switch: Option<&ConfigEntry> = last_entry(local.as_slice(), "extensions.worktreeconfig");
    let direct_switch: Option<&ConfigEntry> = last_entry(direct.as_slice(), "extensions.worktreeconfig");
    if listed_switch != direct_switch {
        return None;
    }
    // The per-worktree file follows the common one, under its own scope, when the extension is on and the common file
    // names a repository format version: Git clears the whole format when no version was found (`read_repository_format`
    // in `setup.c`), the extension with it.
    let versioned: bool = last_entry(direct.as_slice(), "core.repositoryformatversion").is_some();
    if let Some(switch) = direct_switch
        && parse_bool(switch.value.as_deref())?
        && versioned
    {
        add_file(git_dir.join("config.worktree").as_path(), "worktree", scope.home.as_deref(), 0, &mut listed)?;
    }
    add_environment(variables, &mut listed)?;
    return Some(listed);
}

/// The listing as `git config --list -z --show-scope` prints it:
/// `scope\0key\nvalue\0` for each entry.
pub fn listing_bytes(listed: &[ListedEntry]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    for item in listed {
        out.extend_from_slice(item.scope.as_bytes());
        out.push(0);
        out.extend_from_slice(item.entry.key.as_bytes());
        if let Some(value) = &item.entry.value {
            out.push(b'\n');
            out.extend_from_slice(value.as_slice());
        }
        out.push(0);
    }
    return out;
}

#[cfg(test)]
#[path = "repo_read_config_list_tests.rs"]
mod tests;
