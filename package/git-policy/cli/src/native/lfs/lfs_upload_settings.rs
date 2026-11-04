//! What:
//!  The configuration the Git LFS upload plan reads, and the answers derived from it: the default remote, the branch
//!  `HEAD` names, and whether a value reads as true.
//! Why:
//!  The plan and the manual-push probe must agree on the destination of a bare push, so both read it here. The entries
//!  come from a snapshot the caller already holds or from one `config --get-regexp`, never one start per key.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const settings = readSettings(query); const remote = defaultRemote(branch, settings);
//! ```

/// The configuration snapshot the caller may already hold.
use crate::config_snapshot::ConfigSnapshot;
/// Owned OS text for arguments.
use std::ffi::OsString;

/// A read-only Git query: its trimmed standard output when it finished (status zero, or status one with no output, which
///  is how `config --get-regexp` and `symbolic-ref --quiet` say "nothing"), or what went wrong.
pub type Query<'a> = &'a mut dyn FnMut(&[&str]) -> Result<String, String>;

/// Configuration entries the plan reads: `(key, value)` with Git's normalised key.
pub type Settings = Vec<(String, String)>;

/// Why the plan could not be worked out; the push is refused with this text rather than run without its upload.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PlanError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanError(pub String);

/// The `config --get-regexp` pattern that selects exactly the keys `wanted_key` accepts.
pub const SETTINGS_REGEXP: &str =
    "^(branch\\..*\\.(pushremote|remote)|remote\\.pushdefault|remote\\..*\\.(push|pushurl|mirror|url)|push\\.default|url\\..*\\.pushinsteadof)$";

/// Whether the plan reads this configuration key (Git's normalised spelling).
pub(crate) fn wanted_key(key: &str) -> bool {
    let remote_variable = |suffixes: &[&str]| -> bool {
        return key.strip_prefix("remote.").is_some_and(|rest| {
            return rest.rsplit_once('.').is_some_and(|(_name, variable)| {
                return suffixes.contains(&variable);
            });
        });
    };
    let branch_variable: bool = key.strip_prefix("branch.").is_some_and(|rest| {
        return rest.rsplit_once('.').is_some_and(|(_name, variable)| {
            return variable == "pushremote" || variable == "remote";
        });
    });
    let push_rewrite: bool = key.strip_prefix("url.").is_some_and(|rest| {
        return rest.ends_with(".pushinsteadof");
    });
    return branch_variable
        || push_rewrite
        || key == "remote.pushdefault"
        || key == "push.default"
        || remote_variable(&["push", "pushurl", "mirror", "url"]);
}

/// The settings the plan reads, out of a snapshot the caller already holds, so no Git start is needed for them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.entries.filter(wantedKey).map(({ key, value }) => [key, value])
/// ```
pub fn settings_from_snapshot(snapshot: &ConfigSnapshot) -> Settings {
    return snapshot
        .entries()
        .iter()
        .filter(|entry| {
            return wanted_key(entry.key.as_str());
        })
        .map(|entry| {
            let value: String = entry
                .value
                .as_deref()
                .map(|bytes| {
                    return String::from_utf8_lossy(bytes).into_owned();
                })
                .unwrap_or_default();
            return (entry.key.clone(), value);
        })
        .collect();
}

/// The settings the plan reads, from one `config --get-regexp` start.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await git(['config', '--get-regexp', SETTINGS_REGEXP])
/// ```
pub fn read_settings(query: Query<'_>) -> Result<Settings, PlanError> {
    let listing: String = query(&["config", "--get-regexp", SETTINGS_REGEXP])
        .map_err(|why| {
            return PlanError(format!("git config --get-regexp: {why}"));
        })?;
    return Ok(listing
        .lines()
        .map(|line| {
            // A key written without a value is listed alone.
            let (key, value) = line.split_once(' ').unwrap_or((line, ""));
            return (key.to_owned(), value.trim().to_owned());
        })
        .collect());
}

/// The last value of a setting, the last one written winning as in Git.
pub(crate) fn setting<'a>(all: &'a [(String, String)], key: &str) -> Option<&'a str> {
    return all
        .iter()
        .rev()
        .find(|(name, _)| {
            return name == key;
        })
        .map(|(_, value)| {
            return value.as_str();
        });
}

/// Every value of a setting, in the order written.
pub(crate) fn values<'a>(all: &'a [(String, String)], key: &str) -> Vec<&'a str> {
    return all
        .iter()
        .filter(|(name, _)| {
            return name == key;
        })
        .map(|(_, value)| {
            return value.as_str();
        })
        .collect();
}

/// The branch `HEAD` names, from the full ref so that a tag with the same name cannot change the answer.
pub(crate) fn current_branch(query: Query<'_>) -> Result<Option<String>, PlanError> {
    let full: String = query(&["symbolic-ref", "--quiet", "HEAD"]).map_err(|why| {
        return PlanError(format!("git symbolic-ref HEAD: {why}"));
    })?;
    return Ok(full.strip_prefix("refs/heads/").map(str::to_owned));
}

/// The remote Git pushes to when the caller names none: the branch's push remote, the default push remote, the
///  branch's remote, then `origin`.
pub(crate) fn default_remote(branch: Option<&str>, all: &[(String, String)]) -> String {
    let named = |variable: &str| -> Option<String> {
        return branch.and_then(|name| {
            return setting(all, &format!("branch.{name}.{variable}")).map(str::to_owned);
        });
    };
    return named("pushremote")
        .or_else(|| {
            return setting(all, "remote.pushdefault").map(str::to_owned);
        })
        .or_else(|| {
            return named("remote");
        })
        .unwrap_or_else(|| {
            return String::from("origin");
        });
}

/// What:
///  The remote a push goes to: the one the caller named, or the one Git picks for a bare push.
/// Why:
///   The manual-push probe and the upload plan must agree on the destination, so both read it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named ?? defaultRemote(branch, settings)
/// ```
pub fn resolve_push_remote(named: Option<OsString>, query: Query<'_>) -> Result<OsString, PlanError> {
    if let Some(found) = named {
        return Ok(found);
    }
    let branch: Option<String> = current_branch(query)?;
    let all: Settings = read_settings(query)?;
    return Ok(OsString::from(default_remote(branch.as_deref(), all.as_slice())));
}

/// Whether Git reads this configuration text as true.
pub(crate) fn reads_as_true(value: &str) -> bool {
    return ["true", "yes", "on", "1", ""].iter().any(|spelling| {
        return value.eq_ignore_ascii_case(spelling);
    });
}
