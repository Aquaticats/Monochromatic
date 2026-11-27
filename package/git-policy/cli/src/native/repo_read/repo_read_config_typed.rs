//! What:
//!  The proof that every configuration value Git types at startup,
//! in every scope the readers look at,
//! is a value
//!  Git accepts without a message.
//! Why:
//!  Git reads many keys for every command (`core.fileMode`,
//! `core.compression`,
//! `advice.*`,
//! `diff.context`,
//! and so
//!  on) and dies (`fatal: bad boolean config value 'maybe' for 'core.filemode'`),
//! or warns,
//! when a value does not
//!  fit the key's type.
//!  The readers never look at most of those keys,
//! so without this proof they would answer where Git fails.
//!  The proof accepts only spellings it can show Git accepts (a conservative subset) and leaves every other
//!  spelling of a typed key to Git,
//! which then words its own failure.
//!  The key table was built from the keys real Git 2.56.0 rejected or warned about when set to malformed values
//!  (`doc/planning/cli-git-in-process-reads.md`,
//! "Typed configuration values"),
//! and
//!  `every_documented_key_with_a_malformed_value_is_declined_or_equals_git` keeps it honest against the Git that
//!  runs the tests.
//!  `index.maxPathLen` joined the table when the read-cache fix that 2.56.1 merged up made every command read it
//!  (`git_default_core_config` in `environment.c`):
//!  a Git with the key set to a malformed value fails every command,
//! so the readers must decline it before they answer.
//!  Which spelling each kind of value takes is read in
//!  `repo_read_config_typed_value_spellings.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ok: boolean | undefined = everyTypedValueIsAccepted({ entries, scope }); // undefined: ask Git
//! ```

use crate::repo_read_config_scan::{
    ScopeEnvironment,
    include_target,
    is_include_key,
    last_variable,
    read_following,
};
use crate::repo_read_config_text::{
    ConfigEntry,
    parse_config_text,
};
use crate::repo_read_config_typed_value_spellings::is_accepted;
#[cfg(test)]
use crate::repo_read_config_typed_value_spellings::number_of;
#[cfg(test)]
use crate::repo_read_config_value_kinds::Kind;
use crate::repo_read_config_value_kinds::kind_of;
use std::ffi::OsString;
use std::path::Path;

/// How many nested includes are followed before the proof gives up;
/// Git's own limit.
const INCLUDE_DEPTH_LIMIT: usize = 10;

/// How many environment-supplied entries are checked before the proof gives up.
const ENV_ENTRY_LIMIT: u64 = 1024;

/// What:
///  Whether Git accepts the value of one entry silently.
/// Why:
///   A key the table does not list is read by no startup code of the commands the readers replace,
/// so it passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const accepted = kindOf(entry.key) === undefined || isAccepted(kindOf(entry.key), entry.value);
/// ```
pub fn entry_is_accepted(
    key: &str,
    value: Option<&[u8]>,
) -> bool {
    return match kind_of(key) {
        Some(kind) => is_accepted(kind, value),
        None => true,
    };
}

/// Whether the entries parsed from `path`,
/// and every file they include,
/// hold only accepted values;
///  `None` when a file cannot be read or parsed.
pub fn entries_are_accepted(
    entries: &[ConfigEntry],
    path: &Path,
    home: Option<&Path>,
    depth: usize,
) -> Option<bool> {
    if depth > INCLUDE_DEPTH_LIMIT {
        return None;
    }
    for entry in entries {
        if !entry_is_accepted(entry.key.as_str(), entry.value.as_deref()) {
            return Some(false);
        }
        if is_include_key(entry.key.as_str()) {
            let target = include_target(entry.value.as_deref()?, path, home)?;
            if !file_is_accepted(target.as_path(), home, depth + 1)? {
                return Some(false);
            }
        }
    }
    return Some(true);
}

/// The same proof for the file at `path`;
/// a missing file is an empty file to Git.
fn file_is_accepted(
    path: &Path,
    home: Option<&Path>,
    depth: usize,
) -> Option<bool> {
    let Some(text) = read_following(path)? else {
        return Some(true);
    };
    let entries: Vec<ConfigEntry> = parse_config_text(text.as_slice())?;
    return entries_are_accepted(entries.as_slice(), path, home, depth);
}

/// What:
///  Whether the system and global files,
/// and everything they include,
/// hold only accepted values.
/// Why:
///   These scopes feed the same startup reads as the repository's own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scopeFiles(env).every((file) => fileIsAccepted(file)); // undefined when unreadable
/// ```
pub fn outer_scopes_are_accepted(scope: &ScopeEnvironment) -> Option<bool> {
    if scope.unresolvable {
        return None;
    }
    for file in scope.files() {
        if !file_is_accepted(file.as_path(), scope.home.as_deref(), 0)? {
            return Some(false);
        }
    }
    return Some(true);
}

/// The key as Git canonicalizes it:
/// lower-case section and name,
/// subsection kept.
fn canonical(key: &str) -> Option<String> {
    let (section, rest) = key.split_once('.')?;
    return Some(match rest.rsplit_once('.') {
        Some((subsection, name)) => format!(
            "{}.{subsection}.{}",
            section.to_ascii_lowercase(),
            name.to_ascii_lowercase()
        ),
        None => format!(
            "{}.{}",
            section.to_ascii_lowercase(),
            rest.to_ascii_lowercase()
        ),
    });
}

/// What:
///  Whether every `GIT_CONFIG_COUNT` entry among `variables` holds an accepted value.
/// Why:
///   Environment entries reach the same startup reads;
/// a malformed set of entries is not accepted either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// envEntries(variables).every(({ key, value }) => entryIsAccepted(key, value)); // false when malformed
/// ```
pub fn environment_is_accepted(variables: &[(OsString, OsString)]) -> bool {
    let Some(count_text) = last_variable(variables, "GIT_CONFIG_COUNT") else {
        return true;
    };
    let Some(count) = crate::child_environment::parse_config_count(count_text.as_os_str()) else {
        return false;
    };
    if count > ENV_ENTRY_LIMIT {
        return false;
    }
    for index in 0..count {
        let (Some(key), Some(value)) = (
            last_variable(variables, format!("GIT_CONFIG_KEY_{index}").as_str()),
            last_variable(variables, format!("GIT_CONFIG_VALUE_{index}").as_str()),
        ) else {
            return false;
        };
        let Some(key_text) = key.to_str() else {
            return false;
        };
        let Some(canonical_key) = canonical(key_text) else {
            return false;
        };
        if !entry_is_accepted(canonical_key.as_str(), Some(value.as_encoded_bytes())) {
            return false;
        }
    }
    return true;
}

#[cfg(test)]
#[path = "repo_read_config_typed_tests.rs"]
mod tests;
