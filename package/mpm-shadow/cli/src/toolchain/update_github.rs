//! What:     The GitHub release lookup for `bin-sourced` and GitHub-hosted `archive` tools: find the newest release of
//!           the repository the lock's artifact URLs name, derive the version from its tag, and map each platform's
//!           asset by replacing the old version in the old file name.
//! Why:      These tools carry no registry of their own; the lock already names their repository, tag shape and file
//!           names, so the update follows that shape and takes each file's SHA-256 from the release asset's `digest`
//!           (hashing the download only when GitHub gives none).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-github.ts
//! export function resolveGithub(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
//! export function versionFromTag(oldTag: string, oldVersion: string, newTag: string): string
//! export function globMatches(pattern: string, name: string): boolean
//! ```

use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use super::context::Context;
use super::http::DEFAULT_MAX_DOWNLOAD_BYTES;
use super::install_bin::percent_decode;
use super::model::{Artifact, Declaration, LockedTool};
use super::update_resolve::{Resolution, ResolveError, get_json, not_implemented, text_at};

/// The repository (`owner/repo`) and the tag of a release-download URL, or `None` for any other URL.
pub fn parse_release_download(url: &str) -> Option<(String, String)> {
    let rest: &str = url.strip_prefix("https://github.com/")?;
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() < 6 || segments[2] != "releases" || segments[3] != "download" || segments[0].is_empty() || segments[1].is_empty() {
        return None;
    }
    return Some((format!("{}/{}", segments[0], segments[1]), percent_decode(segments[4])));
}

/// What:     The version a new tag stands for, given how the old tag related to the old version: the old tag's prefix
///           (`v`, `bun-v`, nothing) is removed from the new tag.
/// Why:      Tags are `v1.2.3` for one repository and `bun-v1.2.3` for another; the old pair shows the shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function versionFromTag(oldTag: string, oldVersion: string, newTag: string): string  // throws
/// ```
pub fn version_from_tag(old_tag: &str, old_version: &str, new_tag: &str) -> Result<String, ResolveError> {
    let prefix: &str = match old_tag.find(old_version) {
        Some(at) => &old_tag[..at],
        None => {
            return Err(ResolveError::Failed(format!("cannot derive a version from tag {new_tag} given {old_tag} -> {old_version}")));
        }
    };
    return match new_tag.strip_prefix(prefix) {
        Some(version) => Ok(version.to_string()),
        None => Err(ResolveError::Failed(format!("cannot derive a version from tag {new_tag} given {old_tag} -> {old_version}"))),
    };
}

/// What:     Whether `name` matches `pattern`, where `*` matches any run of characters (including none) and every other
///           character matches itself.
/// Why:      The `assetPattern` option selects among several assets of one release.
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

/// The headers for api.github.com: JSON, the API version and the token when there is one.
fn api_headers(ctx: &Context) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = vec![
        ("accept".to_string(), "application/vnd.github+json".to_string()),
        ("x-github-api-version".to_string(), "2022-11-28".to_string()),
    ];
    if let Some(token) = ctx.github_token() {
        headers.push(("authorization".to_string(), format!("Bearer {token}")));
    }
    return headers;
}

/// The latest release object of a repository: `releases/latest`, or the first of `releases?per_page=1` when the
/// repository publishes only prereleases.
fn latest_release(ctx: &Context, repo: &str) -> Result<Value, ResolveError> {
    let headers: Vec<(String, String)> = api_headers(ctx);
    let latest_url: String = format!("https://api.github.com/repos/{repo}/releases/latest");
    if let Ok(release) = get_json(ctx, &latest_url, &headers) {
        return Ok(release);
    }
    let list_url: String = format!("https://api.github.com/repos/{repo}/releases?per_page=1");
    let list: Value = get_json(ctx, &list_url, &headers)?;
    return match list.as_array().and_then(|releases| return releases.first()) {
        Some(release) => Ok(release.clone()),
        None => Err(ResolveError::Failed(format!("{list_url}: the repository has no releases"))),
    };
}

/// SHA-256 of a release asset: the `digest` GitHub reports, or the hash of the downloaded file.
fn asset_digest(ctx: &Context, asset: &Value, name: &str, url: &str) -> Result<String, ResolveError> {
    if let Some(digest) = asset.get("digest").and_then(Value::as_str) {
        return Ok(digest.strip_prefix("sha256:").unwrap_or(digest).to_string());
    }
    let scratch: PathBuf = ctx.scope.toolchain_root.join("downloads").join(format!(".update-hash-{}", std::process::id()));
    let result = ctx.http.download(url, &scratch, DEFAULT_MAX_DOWNLOAD_BYTES);
    // Best effort: the file only existed to be hashed.
    let _ = fs::remove_file(&scratch);
    return match result {
        Ok(hash) => Ok(hash),
        Err(error) => Err(ResolveError::Failed(format!("release asset {name} carries no digest and cannot be downloaded to hash it: {error}"))),
    };
}

/// What:     Resolves the newest release of a GitHub-hosted tool.
/// Why:      Needs the old rows: they name the repository, the tag shape and the platforms. A tool with no row yet has
///           nothing to derive them from, which is reported as not implemented rather than guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveGithub(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
/// ```
pub fn resolve_github(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>) -> Result<Resolution, ResolveError> {
    let Some(locked) = current.filter(|tool| return !tool.artifacts.is_empty()) else {
        return Err(not_implemented(declaration, "resolving a GitHub-hosted tool that is not in the lock yet"));
    };
    let Some((repo, old_tag)) = locked.artifacts.iter().find_map(|(_, row)| return parse_release_download(&row.url)) else {
        return Err(not_implemented(declaration, "a source other than GitHub releases"));
    };
    let release: Value = latest_release(ctx, &repo)?;
    let Some(new_tag) = text_at(&release, &["tag_name"]) else {
        return Err(ResolveError::Failed(format!("github.com/{repo} reports no release tag")));
    };
    let new_version: String = version_from_tag(&old_tag, &locked.version, new_tag)?;
    if new_version == locked.version {
        return Ok(Resolution { version: locked.version.clone(), artifacts: locked.artifacts.clone() });
    }
    let assets: Vec<Value> = release.get("assets").and_then(Value::as_array).cloned().unwrap_or_default();
    let pattern_matches: Vec<&Value> = match &declaration.options.asset_pattern {
        Some(pattern) => assets
            .iter()
            .filter(|asset| return text_at(asset, &["name"]).is_some_and(|name| return glob_matches(pattern, name)))
            .collect(),
        None => Vec::new(),
    };
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for (platform, row) in &locked.artifacts {
        let old_name: &str = row.url.rsplit('/').next().unwrap_or("");
        let candidates: [String; 2] = [old_name.replace(&locked.version, &new_version), old_name.to_string()];
        let by_name: Option<&Value> = candidates
            .iter()
            .find_map(|candidate| return assets.iter().find(|asset| return text_at(asset, &["name"]) == Some(candidate.as_str())));
        let chosen: &Value = match (by_name, &declaration.options.asset_pattern) {
            (Some(asset), _) => asset,
            (None, Some(pattern)) if pattern_matches.len() == 1 => pattern_matches[0],
            (None, Some(pattern)) => {
                return Err(ResolveError::Failed(format!(
                    "asset pattern {pattern} matches {} release assets for {platform} in {new_tag}; expected exactly one",
                    pattern_matches.len()
                )));
            }
            (None, None) => {
                return Err(ResolveError::Failed(format!("no release asset in {new_tag} of {repo} matches {old_name}")));
            }
        };
        let (Some(name), Some(url)) = (text_at(chosen, &["name"]), text_at(chosen, &["browser_download_url"])) else {
            return Err(ResolveError::Failed(format!("release asset {old_name} of {repo} carries no download URL")));
        };
        let sha256: String = asset_digest(ctx, chosen, name, url)?;
        artifacts.push((platform.clone(), Artifact { url: url.to_string(), sha256 }));
    }
    return Ok(Resolution { version: new_version, artifacts });
}
