//! What:
//! The generic GitHub release resolver for `github:<owner>/<repo>` requests:
//! the newest release (or the release of the wanted tag),
//! and for every Linux platform the asset chosen by the usual OS and architecture naming convention.
//! Why:
//! The request names the repository,
//! so nothing is learned from a previous lock row.
//! A repository whose asset names are ambiguous (two programs in one release) gets an `asset` hint in its plan entry,
//! and the resolver fails naming the candidates rather than guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-github.ts
//! export function resolveGithub(ctx: Context, declaration: Declaration, repo: string, wanted?: string): Resolution
//! export function globMatches(pattern: string, name: string): boolean
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the model types the resolver builds.
use super::model::{Artifact, Declaration};
/// Imports the version normalizer shared with `list`.
use super::parse_output::normalize_version;
/// Imports the asset chooser.
use super::update_github_assets::pick_asset;
/// Imports the shared resolver helpers.
use super::update_resolve::{LINUX_PLATFORMS, Resolution, ResolveError, get_json, hash_download, text_at};

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
pub fn glob_matches(pattern: &str, name: &str) -> bool {
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

/// The headers for api.github.com:
/// JSON,
/// the API version and the token when there is one.
pub fn api_headers(ctx: &Context) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = vec![
        ("accept".to_string(), "application/vnd.github+json".to_string()),
        ("x-github-api-version".to_string(), "2022-11-28".to_string()),
    ];
    if let Some(token) = ctx.github_token() {
        headers.push(("authorization".to_string(), format!("Bearer {token}")));
    }
    return headers;
}

/// The latest release object of a repository:
/// `releases/latest`,
/// or the first of `releases?per_page=1` when the repository publishes only prereleases.
fn latest_release(ctx: &Context, repo: &str) -> Result<Value, ResolveError> {
    let headers: Vec<(String, String)> = api_headers(ctx);
    let latest_url: String = format!("https://api.github.com/repos/{repo}/releases/latest");
    if let Ok(release) = get_json(ctx, &latest_url, &headers) {
        return Ok(release);
    }
    let list_url: String = format!("https://api.github.com/repos/{repo}/releases?per_page=1");
    let list: Value = get_json(ctx, &list_url, &headers)?;
    let first: Option<&Value> = match list.as_array() {
        Some(releases) => releases.first(),
        None => None,
    };
    return match first {
        Some(release) => Ok(release.clone()),
        None => Err(ResolveError::Failed(format!("{list_url}: the repository has no releases"))),
    };
}

/// The asset of `release` with exactly this name,
/// or `None`.
pub fn find_asset<'a>(release: &'a Value, name: &str) -> Option<&'a Value> {
    let assets: &[Value] = release.get("assets").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    for asset in assets {
        if text_at(asset, &["name"]) == Some(name) {
            return Some(asset);
        }
    }
    return None;
}

/// The release object of one tag of a repository.
pub fn release_by_tag(ctx: &Context, repo: &str, tag: &str) -> Result<Value, ResolveError> {
    let url: String = format!("https://api.github.com/repos/{repo}/releases/tags/{}", tag.replace('+', "%2B"));
    return get_json(ctx, &url, &api_headers(ctx));
}

/// SHA-256 of a release asset:
/// the `digest` GitHub reports,
/// or the hash of the downloaded file.
pub fn asset_digest(ctx: &Context, asset: &Value, url: &str) -> Result<String, ResolveError> {
    if let Some(digest) = asset.get("digest").and_then(Value::as_str) {
        return Ok(digest.strip_prefix("sha256:").unwrap_or(digest).to_string());
    }
    return hash_download(ctx, url);
}

/// How many pages of 100 releases the prefix search reads before it gives up.
const PREFIX_SEARCH_PAGES: u32 = 10;

/// What:
/// The newest release that is neither a draft nor a prerelease and whose tag starts with `prefix`
/// and continues with a version (a digit,
/// or `v` and a digit).
/// Why:
/// One repository can tag the releases of several programs;
/// the list is newest first,
/// so the first match is the newest release of the program the prefix names.
/// A repository whose last 1000 releases hold none is an error that names the prefix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function latestReleaseWithPrefix(ctx: Context, repo: string, prefix: string): Release
/// ```
fn latest_release_with_prefix(ctx: &Context, repo: &str, prefix: &str) -> Result<Value, ResolveError> {
    let headers: Vec<(String, String)> = api_headers(ctx);
    for page in 1..=PREFIX_SEARCH_PAGES {
        let url: String = format!("https://api.github.com/repos/{repo}/releases?per_page=100&page={page}");
        let list: Value = get_json(ctx, &url, &headers)?;
        let Some(releases) = list.as_array() else {
            return Err(ResolveError::Failed(format!("{url}: the answer is not a list of releases")));
        };
        for release in releases {
            let skipped: bool = release.get("draft").and_then(Value::as_bool).unwrap_or(false)
                || release.get("prerelease").and_then(Value::as_bool).unwrap_or(false);
            let Some(tag) = text_at(release, &["tag_name"]) else {
                continue;
            };
            if !skipped && version_after_prefix(tag, prefix).is_some() {
                return Ok(release.clone());
            }
        }
        if releases.len() < 100 {
            break;
        }
    }
    return Err(ResolveError::Failed(format!(
        "github.com/{repo} has no release (neither draft nor prerelease) whose tag starts with {prefix:?} and continues with a version"
    )));
}

/// What:
/// The part of `tag` after `prefix` when it starts a version (a digit,
/// or `v` and a digit).
/// Why:
/// The prefix must not match a longer program name that shares it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function versionAfterPrefix(tag: string, prefix: string): string | undefined
/// ```
fn version_after_prefix<'a>(tag: &'a str, prefix: &str) -> Option<&'a str> {
    let rest: &str = tag.strip_prefix(prefix)?;
    let bytes: &[u8] = rest.as_bytes();
    let starts_version: bool = bytes.first().is_some_and(u8::is_ascii_digit)
        || (bytes.first() == Some(&b'v') && bytes.get(1).is_some_and(u8::is_ascii_digit));
    if starts_version {
        return Some(rest);
    }
    return None;
}

/// The release of a wanted version of the program a prefix names:
/// the prefix and the range as written,
/// then the prefix and `v` and the range (unless the prefix already ends in `v`).
fn release_of_prefixed_range(ctx: &Context, repo: &str, prefix: &str, range: &str) -> Result<Value, ResolveError> {
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
fn release_of_range(ctx: &Context, repo: &str, range: &str) -> Result<Value, ResolveError> {
    return match release_by_tag(ctx, repo, range) {
        Ok(release) => Ok(release),
        Err(first) => match release_by_tag(ctx, repo, &format!("v{range}")) {
            Ok(release) => Ok(release),
            Err(_) => Err(first),
        },
    };
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
pub fn resolve_github(ctx: &Context, declaration: &Declaration, repo: &str, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    if repo.split('/').count() != 2 || repo.contains(char::is_whitespace) || repo.contains("..") {
        return Err(ResolveError::Failed(format!("{repo:?} is not a GitHub repository (write github:<owner>/<repo>)")));
    }
    let tag_prefix: Option<&str> = declaration.options.tag_prefix.as_deref();
    let release: Value = match (tag_prefix, wanted) {
        (Some(start), Some(range)) => release_of_prefixed_range(ctx, repo, start, range)?,
        (Some(start), None) => latest_release_with_prefix(ctx, repo, start)?,
        (None, Some(range)) => release_of_range(ctx, repo, range)?,
        (None, None) => latest_release(ctx, repo)?,
    };
    let Some(tag) = text_at(&release, &["tag_name"]) else {
        return Err(ResolveError::Failed(format!("github.com/{repo} reports no release tag")));
    };
    let assets: &[Value] = release.get("assets").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    let mut missing: Vec<String> = Vec::new();
    for platform in LINUX_PLATFORMS {
        match pick_asset(assets, platform, declaration.options.asset.as_deref(), repo, tag)? {
            Some(asset) => {
                let Some(url) = text_at(asset, &["browser_download_url"]) else {
                    return Err(ResolveError::Failed(format!("a release asset of github.com/{repo} {tag} carries no download URL")));
                };
                artifacts.push((platform.to_string(), Artifact { url: url.to_string(), sha256: asset_digest(ctx, asset, url)? }));
            }
            None => missing.push(platform.to_string()),
        }
    }
    if artifacts.is_empty() {
        return Err(ResolveError::Failed(format!("github.com/{repo} {tag} has no Linux asset (platforms looked for: {})", missing.join(", "))));
    }
    let after_prefix: Option<&str> = match tag_prefix {
        Some(start) => version_after_prefix(tag, start),
        None => None,
    };
    let version: String = match after_prefix {
        Some(rest) => normalize_version(rest),
        None => normalize_version(tag),
    };
    return Ok(Resolution { version, artifacts });
}
