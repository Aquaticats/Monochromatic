//! What:
//! The generic GitHub release resolver for `github:<owner>/<repo>` requests:
//! the newest release (or the release of the wanted tag),
//! and for every Linux platform the asset chosen by the usual OS and architecture naming convention.
//! Why:
//! The request names the repository,
//! so nothing is learned from a previous lock row.
//! A repository whose asset names are ambiguous (two programs in one release) gets an `asset` hint in its plan entry,
//! and the resolver fails naming the candidates rather than guess.
//! The release and tag lookups are one file each,
//! named for what each holds:
//! the release objects asked for at api.github.com (`releases`)
//! and the tags a request names (`tag_names`).
//! This file holds the resolver itself and the `asset` pattern it honours.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-github.ts
//! export function resolveGithub(ctx: Context, declaration: Declaration, repo: string, wanted?: string): Resolution
//! export function globMatches(pattern: string, name: string): boolean
//! export * from './update_github/releases.ts';
//! export * from './update_github/tag_names.ts';
//! ```
/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports the bundle of seams every resolver receives.
use super::context::Context;
/// Imports the model types the resolver builds.
use super::model::{
    Artifact,
    Declaration,
};
/// Imports the version normalizer shared with `list`.
use super::parse_output::normalize_version;
/// Imports the asset chooser.
use super::update_github_assets::pick_asset;
/// Imports the shared resolver helpers.
use super::update_resolve::{
    LINUX_PLATFORMS,
    Resolution,
    ResolveError,
    text_at,
};

/// The release objects,
/// asked for at api.github.com.
mod releases;
/// The tags a request names.
mod tag_names;

/// Imports the release lookups this file chooses between.
use self::releases::latest_release;
/// Re-exports the requests for a repository's latest release,
/// the release of one tag,
/// one asset of a release and that asset's SHA-256.
pub use self::releases::{
    api_headers,
    asset_digest,
    find_asset,
    release_by_tag,
};
/// Re-exports the bounds of the prefix search,
/// which its tests state alone.
#[cfg(test)]
pub(crate) use self::tag_names::{
    PREFIX_SEARCH_PAGES,
    RELEASES_PER_PAGE,
};
/// Imports the tag rules this file chooses between.
use self::tag_names::{
    latest_release_with_prefix,
    release_of_prefixed_range,
    release_of_range,
    version_after_prefix,
};

/// What:
/// Whether `name` matches `pattern`,
/// where `*` matches any run of characters (including none) and every other character matches itself.
/// Why:
/// The `asset` option selects among several assets of one release.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function globMatches(pattern: string, name: string): boolean
/// ```
pub fn glob_matches(
    pattern: &str,
    name: &str,
) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return name == pattern;
    }
    let first: &str = parts[0];
    let last: &str = parts[parts.len() - 1];
    if name.len() < first.len() + last.len() || !name.starts_with(first) || !name.ends_with(last) {
        return false;
    }
    let mut cursor: usize = first.len();
    for middle in &parts[1..parts.len() - 1] {
        match name[cursor..].find(middle) {
            Some(at) => cursor += at + middle.len(),
            None => return false,
        }
    }
    return cursor <= name.len() - last.len();
}

/// What:
/// Resolves a GitHub-hosted tool from its repository alone.
/// Why:
/// The version is the release tag without its `v` or `name-v` prefix;
/// each Linux platform's asset is chosen by name;
/// a platform with no matching asset is left out (and fails at `install` with a message naming it),
/// but a release that matches no platform at all is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveGithub(ctx: Context, declaration: Declaration, repo: string, wanted?: string): Resolution
/// ```
pub fn resolve_github(
    ctx: &Context,
    declaration: &Declaration,
    repo: &str,
    wanted: Option<&str>,
) -> Result<Resolution, ResolveError> {
    if repo.split('/').count() != 2 || repo.contains(char::is_whitespace) || repo.contains("..") {
        return Err(ResolveError::Failed(format!(
            "{repo:?} is not a GitHub repository (write github:<owner>/<repo>)"
        )));
    }
    let tag_prefix: Option<&str> = declaration.options.tag_prefix.as_deref();
    let release: Value = match (tag_prefix, wanted) {
        (Some(start), Some(range)) => release_of_prefixed_range(ctx, repo, start, range)?,
        (Some(start), None) => latest_release_with_prefix(ctx, repo, start)?,
        (None, Some(range)) => release_of_range(ctx, repo, range)?,
        (None, None) => latest_release(ctx, repo)?,
    };
    let Some(tag) = text_at(&release, &["tag_name"]) else {
        return Err(ResolveError::Failed(format!(
            "github.com/{repo} reports no release tag"
        )));
    };
    let assets: &[Value] = release
        .get("assets")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    for platform in LINUX_PLATFORMS {
        match pick_asset(
            assets,
            platform,
            declaration.options.asset.as_deref(),
            repo,
            tag,
        )? {
            Some(asset) => {
                let Some(url) = text_at(asset, &["browser_download_url"]) else {
                    return Err(ResolveError::Failed(format!(
                        "a release asset of github.com/{repo} {tag} carries no download URL"
                    )));
                };
                artifacts.push((
                    platform.to_string(),
                    Artifact {
                        url: url.to_string(),
                        sha256: asset_digest(ctx, asset, url)?,
                    },
                ));
            }
            None => missing.push(platform.to_string()),
        }
    }
    if artifacts.is_empty() {
        return Err(ResolveError::Failed(format!(
            "github.com/{repo} {tag} has no Linux asset (platforms looked for: {})",
            missing.join(", ")
        )));
    }
    let version: String = match tag_prefix {
        Some(start) => match version_after_prefix(tag, start) {
            Some(rest) => normalize_version(rest),
            None => {
                return Err(ResolveError::Failed(format!(
                    "the tag {tag:?} of github.com/{repo} does not continue the tagPrefix {start:?} with a version (such as 1.2.3)"
                )));
            }
        },
        None => normalize_version(tag),
    };
    return Ok(Resolution {
        version,
        artifacts,
    });
}
