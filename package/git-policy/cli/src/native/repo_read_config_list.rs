//! What:
//!  The merged configuration listing of `git config --list -z --show-scope`, built in process from the system, global
//!  and repository files, their `include.path` files, and the environment's `GIT_CONFIG_COUNT` entries.
//! Why:
//!  The wrapper reads this listing once per commit and asks several single-key questions Git answers from the same
//!  files.
//!  The reader declines conditional includes (whose condition Git evaluates), a worktree-specific file, command-line
//!  `-c` entries (`GIT_CONFIG_PARAMETERS`), a file it cannot parse and an environment entry it cannot canonicalize.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const entries = listConfig({ commonDir, scope, variables }); // undefined: ask Git
//! ```

use super::repo_read_config_scan::{ScopeEnvironment, include_target, is_include_key, read_following};
use super::repo_read_config_text::{ConfigEntry, last_entry, parse_config_text};
use std::ffi::OsString;
use std::path::Path;

/// How many nested includes are followed; Git's own limit.
const INCLUDE_DEPTH_LIMIT: usize = 10;

/// How many environment-supplied entries are read.
const ENV_ENTRY_LIMIT: u64 = 1024;

/// One listed entry with the scope name `--show-scope` prints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedEntry {
    /// `system`, `global`, `local` or `command`.
    pub scope: &'static str,
    /// The entry.
    pub entry: ConfigEntry,
}

/// Append the entries of the file at `path` and, in place, of every file it includes.
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

/// The canonical spelling of a key given in the environment: lower-case section and variable name, subsection kept.
fn canonical_key(key: &str) -> Option<String> {
    let (section, rest) = key.split_once('.')?;
    let (middle, name) = match rest.rsplit_once('.') {
        Some((subsection, name)) => (Some(subsection), name),
        None => (None, rest),
    };
    let plain = |text: &str| -> bool { return !text.is_empty() && text.bytes().all(|byte| { return byte.is_ascii_alphanumeric() || byte == b'-'; }); };
    if !plain(section) || !plain(name) || !name.as_bytes()[0].is_ascii_alphabetic() || middle.is_some_and(|text| { return text.contains('\n'); }) {
        return None;
    }
    return Some(match middle {
        Some(subsection) => format!("{}.{subsection}.{}", section.to_ascii_lowercase(), name.to_ascii_lowercase()),
        None => format!("{}.{}", section.to_ascii_lowercase(), name.to_ascii_lowercase()),
    });
}

/// Append the `GIT_CONFIG_COUNT` entries of `variables` under the scope `command`.
fn add_environment(variables: &[(OsString, OsString)], listed: &mut Vec<ListedEntry>) -> Option<()> {
    let find = |name: &str| -> Option<OsString> {
        return variables.iter().rev().find(|(candidate, _)| { return candidate == name; }).map(|(_, value)| { return value.clone(); });
    };
    if find("GIT_CONFIG_PARAMETERS").is_some() {
        return None;
    }
    let Some(count_text) = find("GIT_CONFIG_COUNT") else {
        return Some(());
    };
    let count: u64 = super::child_environment::parse_config_count(count_text.as_os_str())?;
    if count > ENV_ENTRY_LIMIT {
        return None;
    }
    for index in 0..count {
        let key: OsString = find(format!("GIT_CONFIG_KEY_{index}").as_str())?;
        let value: OsString = find(format!("GIT_CONFIG_VALUE_{index}").as_str())?;
        let entry: ConfigEntry = ConfigEntry { key: canonical_key(key.to_str()?)?, value: Some(value.as_encoded_bytes().to_vec()) };
        listed.push(ListedEntry { scope: "command", entry });
    }
    return Some(());
}

/// What:
///  Every entry Git would list, in its order, or `None` when Git must be asked.
/// Why:
///   The order is system, global, local, command; included files follow their `include.path` entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function listConfig(commonDir: string, scope: ScopeEnvironment, variables: [string, string][]): ListedEntry[] | undefined;
/// ```
pub fn list_config(common_dir: &Path, scope: &ScopeEnvironment, variables: &[(OsString, OsString)]) -> Option<Vec<ListedEntry>> {
    let mut listed: Vec<ListedEntry> = Vec::new();
    for (label, file) in scope.labelled_files() {
        add_file(file.as_path(), label, scope.home.as_deref(), 0, &mut listed)?;
    }
    let start: usize = listed.len();
    add_file(common_dir.join("config").as_path(), "local", scope.home.as_deref(), 0, &mut listed)?;
    let local: Vec<ConfigEntry> = listed[start..].iter().map(|item| { return item.entry.clone(); }).collect();
    if last_entry(local.as_slice(), "extensions.worktreeconfig").is_some() {
        return None;
    }
    add_environment(variables, &mut listed)?;
    return Some(listed);
}

/// The listing as `git config --list -z --show-scope` prints it: `scope\0key\nvalue\0` for each entry.
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
