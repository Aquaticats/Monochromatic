//! What:
//!  Exact removal of retired hk hook registrations from one explicit Git configuration scope:
//!  list every scoped key with `git config --list --show-scope`, keep the sorted unique keys
//!  of that scope that start with `hook.hk-`, and unset each with `--unset-all`.
//! Why:
//!  The retired hk installer wrote its hooks into Git configuration;
//!  this repository's migration removes only that exact namespace and leaves every other key alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await cleanupHkGitConfig({ gitPath, scope: 'local', cwd });
//! ```

/// `Display` gives the failure text the caller prints.
use std::fmt;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
/// Real Git's captured output and the context that starts it.
use super::transaction_git::{GitContext, GitOutput, GitRequest, run_git};

/// The exact key prefix the retired hk installer owns.
const HK_HOOK_PREFIX: &str = "hook.hk-";

/// What:
///  The Git configuration scope eligible for cleanup.
/// Why:
///   Only the explicitly chosen scope is read and written; the other scope is never touched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HkConfigScope = 'global' | 'local';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HkConfigScope {
    /// The user's global configuration file.
    Global,
    /// The repository's configuration file.
    Local,
}

/// Methods over the scope.
impl HkConfigScope {
    /// What:
    ///  The name Git prints for this scope in `--show-scope` output.
    /// Why:
    ///   The listing is matched against that name.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// scope // 'global' or 'local'
    /// ```
    fn listing_name(self) -> &'static str {
        match self {
            Self::Global => return "global",
            Self::Local => return "local",
        }
    }

    /// What:
    ///  The `git config` flag that selects this scope when writing.
    /// Why:
    ///   Writes must name the same scope the listing read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `--${scope}`
    /// ```
    fn flag(self) -> &'static str {
        match self {
            Self::Global => return "--global",
            Self::Local => return "--local",
        }
    }
}

/// What:
///  Why the cleanup could not finish.
/// Why:
///   Each variant names the step that failed, so the caller can report it without re-deriving the cause.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HkConfigCleanupError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HkConfigCleanupError {
    /// Real Git could not be started; the text names the command and the operating-system reason.
    Start(String),
    /// The listing ended a scope without a key, so its fields cannot be paired.
    MalformedListing,
    /// The listing exited with a status other than zero.
    ListingFailed(i32),
    /// Removing one key exited with a status other than zero.
    Failed {
        /// The key whose removal failed.
        key: String,
        /// Git's exit status; a signal counts as `1`.
        code: i32,
    },
}

/// Display text for a failed cleanup.
impl fmt::Display for HkConfigCleanupError {
    /// What:
    ///  Human-readable text for the variant.
    /// Why:
    ///   The caller prints the error directly.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start(reason) => return formatter.write_str(reason.as_str()),
            Self::MalformedListing => {
                return formatter.write_str("git config --show-scope listing has a scope without a key");
            }
            Self::ListingFailed(code) => {
                return write!(formatter, "git config --list --show-scope exited {code}");
            }
            Self::Failed { key, code } => {
                return write!(formatter, "git config --unset-all {key} exited {code}");
            }
        }
    }
}

/// The error trait lets callers box this error.
impl std::error::Error for HkConfigCleanupError {}

/// What:
///  The keys removed from one scope, in sorted order.
/// Why:
///   An empty list proves that a repeated cleanup changed nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HkConfigCleanupResult = { scope: HkConfigScope; removedKeys: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HkConfigCleanupResult {
    /// The scope inspected.
    pub scope: HkConfigScope,
    /// Sorted unique hk-owned keys removed from that scope.
    pub removed_keys: Vec<String>,
}

/// What:
///  The hk-owned keys of one scope from `git config --null --show-scope --name-only --list` output.
/// Why:
///   Git prints alternating scope and key fields; a trailing empty field is the terminator and is dropped,
///  while an empty field elsewhere stays a field.
///  Keys sort by UTF-16 code units, as JavaScript's default sort does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hkKeysInScope(listing: string, scope: string): string[];
/// ```
pub fn hk_keys_in_scope(
    listing: &[u8],
    scope: HkConfigScope,
) -> Result<Vec<String>, HkConfigCleanupError> {
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(listing);
    // `mut` lets the terminator be dropped.
    let mut fields: Vec<&str> = text.split('\0').collect();
    if fields.last() == Some(&"") {
        fields.pop();
    }
    if fields.len() % 2 != 0 {
        return Err(HkConfigCleanupError::MalformedListing);
    }
    // `mut` lets the keys be sorted and deduplicated in place.
    let mut keys: Vec<String> = Vec::new();
    for pair in fields.chunks(2) {
        match pair {
            [scope_field, key] if *scope_field == scope.listing_name() && key.starts_with(HK_HOOK_PREFIX) => {
                keys.push(String::from(*key));
            }
            _ => {}
        }
    }
    keys.sort_by(|left, right| return left.encode_utf16().cmp(right.encode_utf16()));
    keys.dedup();
    return Ok(keys);
}

/// What:
///  Remove every hk-owned key from one scope, leaving every other key in place.
/// Why:
///   The migration runs once per machine or repository; a repeated run finds nothing and changes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await cleanupHkGitConfig({ gitPath, scope, cwd });
/// ```
pub fn cleanup_hk_git_config(
    context: &GitContext,
    scope: HkConfigScope,
    cwd: &Path,
) -> Result<HkConfigCleanupResult, HkConfigCleanupError> {
    let listing: GitOutput = run_git(
        context,
        &GitRequest::new(cwd, &["config", "--null", "--show-scope", "--name-only", "--list"]),
    )
    .map_err(|error| return HkConfigCleanupError::Start(format!("git config --list could not start: {error}")))?;
    let listing_code: i32 = listing.code.unwrap_or(1);
    if listing_code != 0 {
        return Err(HkConfigCleanupError::ListingFailed(listing_code));
    }
    let keys: Vec<String> = hk_keys_in_scope(listing.stdout.as_slice(), scope)?;
    for key in &keys {
        let removal: GitOutput = run_git(
            context,
            &GitRequest::new(cwd, &["config", scope.flag(), "--unset-all", key.as_str()]),
        )
        .map_err(|error| return HkConfigCleanupError::Start(format!("git config --unset-all could not start: {error}")))?;
        let removal_code: i32 = removal.code.unwrap_or(1);
        if removal_code != 0 {
            return Err(HkConfigCleanupError::Failed {
                key: key.clone(),
                code: removal_code,
            });
        }
    }
    return Ok(HkConfigCleanupResult {
        scope,
        removed_keys: keys,
    });
}

/// Cleanup controls live beside the cleanup.
#[cfg(test)]
#[path = "maintenance_hk_config_tests.rs"]
mod tests;
