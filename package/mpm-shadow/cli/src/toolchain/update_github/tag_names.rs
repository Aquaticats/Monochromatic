//! What:
//! Which tag answers a request:
//! the range as written and with a `v` in front,
//! the same two after the `tagPrefix` of the program a plan entry names,
//! and the prefix search that takes the highest version among a repository's releases.
//! Why:
//! One repository can tag the releases of several programs,
//! and the release list is ordered by creation rather than by version,
//! so the tag is chosen by rules of its own,
//! apart from the requests that fetch a release object.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function versionAfterPrefix(tag: string, prefix: string): string | undefined
//! function latestReleaseWithPrefix(ctx: Context, repo: string, prefix: string): Release
//! function releaseOfRange(ctx: Context, repo: string, range: string): Release
//! function releaseOfPrefixedRange(ctx: Context, repo: string, prefix: string, range: string): Release
//! ```
/// Imports `Value` from `serde_json`,
/// the release object as the API answers it.
use serde_json::Value;

/// Imports the bundle of seams every resolver receives.
use super::super::context::Context;
/// Imports the shared resolver helpers.
use super::super::update_resolve::{
    ResolveError,
    get_json,
    text_at,
};
/// Imports the headers and the tag lookup the requests are made with.
use super::releases::{
    api_headers,
    release_by_tag,
};
/// Imports the version comparison and the whole-version test.
use crate::version::{
    compare_versions,
    is_whole_version,
};

/// How many releases one page of the release list holds.
pub(crate) const RELEASES_PER_PAGE: usize = 100;

/// How many pages of releases the prefix search reads before it gives up.
pub(crate) const PREFIX_SEARCH_PAGES: u32 = 10;

/// What:
/// The highest version among the releases that are neither a draft nor a prerelease and whose tag is `prefix`
/// followed by a version (see `version_after_prefix`).
/// Why:
/// One repository can tag the releases of several programs.
/// The list is ordered by creation,
/// not by version,
/// so a backport release made after a newer line can sit first;
/// the search therefore reads every page (until a short page or the cap) and keeps the highest version,
/// the earlier one in the list when two are equal.
/// A repository whose last 1000 releases hold none is an error that names the prefix and says the search stopped at
/// its cap when it did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function latestReleaseWithPrefix(ctx: Context, repo: string, prefix: string): Release
/// ```
pub(super) fn latest_release_with_prefix(
    ctx: &Context,
    repo: &str,
    prefix: &str,
) -> Result<Value, ResolveError> {
    let headers: Vec<(String, String)> = api_headers(ctx);
    let mut best: Option<(String, Value)> = None;
    let mut stopped_at_cap: bool = false;
    for page in 1..=PREFIX_SEARCH_PAGES {
        let url: String = format!(
            "https://api.github.com/repos/{repo}/releases?per_page={RELEASES_PER_PAGE}&page={page}"
        );
        let list: Value = get_json(ctx, &url, &headers)?;
        let Some(releases) = list.as_array() else {
            return Err(ResolveError::Failed(format!(
                "{url}: the answer is not a list of releases"
            )));
        };
        for release in releases {
            let skipped: bool = release
                .get("draft")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || release
                    .get("prerelease")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
            let Some(tag) = text_at(release, &["tag_name"]) else {
                continue;
            };
            if skipped {
                continue;
            }
            let Some(version) = version_after_prefix(tag, prefix) else {
                continue;
            };
            let is_higher: bool = match &best {
                Some((kept, _)) => compare_versions(version, kept).is_gt(),
                None => true,
            };
            if is_higher {
                best = Some((version.to_string(), release.clone()));
            }
        }
        if releases.len() < RELEASES_PER_PAGE {
            break;
        }
        stopped_at_cap = page == PREFIX_SEARCH_PAGES;
    }
    if let Some((_, release)) = best {
        return Ok(release);
    }
    let mut message: String = format!(
        "github.com/{repo} has no release (neither draft nor prerelease) whose tag starts with {prefix:?} and continues with a version"
    );
    if stopped_at_cap {
        message.push_str(&format!(
            "; the search stopped at its cap of {PREFIX_SEARCH_PAGES} pages ({} releases), so older releases were not looked at",
            PREFIX_SEARCH_PAGES as usize * RELEASES_PER_PAGE
        ));
    }
    return Err(ResolveError::Failed(message));
}

/// What:
/// The version in `tag` after `prefix`,
/// without a leading `v`,
/// when everything after the prefix is one whole version (`1.2.3`,
/// `1.2.3-rc.1`);
/// a `v` in front of it counts only when the prefix has not spelled the `v` out itself.
/// Why:
/// The prefix must not match a tag of a longer program name that shares it:
/// with the prefix `tool-`,
/// the tag `tool-2-v1.0.0` belongs to a program called `tool-2` and its tail is not a version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function versionAfterPrefix(tag: string, prefix: string): string | undefined
/// ```
pub(super) fn version_after_prefix<'a>(
    tag: &'a str,
    prefix: &str,
) -> Option<&'a str> {
    let rest: &str = tag.strip_prefix(prefix)?;
    let version: &str = if prefix.ends_with('v') {
        rest
    } else {
        rest.strip_prefix('v').unwrap_or(rest)
    };
    if is_whole_version(version) {
        return Some(version);
    }
    return None;
}

/// The release of a wanted version of the program a prefix names:
/// the prefix and the range as written,
/// then the prefix and `v` and the range (unless the prefix already ends in `v`).
pub(super) fn release_of_prefixed_range(
    ctx: &Context,
    repo: &str,
    prefix: &str,
    range: &str,
) -> Result<Value, ResolveError> {
    return match release_by_tag(ctx, repo, &format!("{prefix}{range}")) {
        Ok(release) => Ok(release),
        // A prefix that already ends in `v` has spelled the `v` out.
        Err(first) if prefix.ends_with('v') => Err(first),
        Err(first) => match release_by_tag(ctx, repo, &format!("{prefix}v{range}")) {
            Ok(release) => Ok(release),
            Err(_) => Err(first),
        },
    };
}

/// The release of a wanted tag:
/// the range as written,
/// then with a `v` in front (people write `0.64.3` for the tag `v0.64.3`).
pub(super) fn release_of_range(
    ctx: &Context,
    repo: &str,
    range: &str,
) -> Result<Value, ResolveError> {
    return match release_by_tag(ctx, repo, range) {
        Ok(release) => Ok(release),
        Err(first) => match release_by_tag(ctx, repo, &format!("v{range}")) {
            Ok(release) => Ok(release),
            Err(_) => Err(first),
        },
    };
}
