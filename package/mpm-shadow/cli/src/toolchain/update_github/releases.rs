//! What:
//! The release objects of a GitHub repository,
//! asked for at api.github.com:
//! the latest release,
//! the release of one tag,
//! one asset of a release by its name,
//! and the SHA-256 a lock records for that asset.
//! Why:
//! The release and tag lookups are the part of the resolver that talks to GitHub,
//! so they sit in one file:
//! the `tagPrefix` search and the resolver above them read release objects
//! and never build a request themselves.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function apiHeaders(ctx: Context): [string, string][]
//! export function releaseByTag(ctx: Context, repo: string, tag: string): Release
//! export function findAsset(release: Release, name: string): Asset | undefined
//! export function assetDigest(ctx: Context, asset: Asset, url: string): string
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
    hash_download,
    text_at,
};

/// The headers for api.github.com:
/// JSON,
/// the API version and the token when there is one.
pub fn api_headers(ctx: &Context) -> Vec<(String, String)> {
    let mut headers: Vec<(String, String)> = vec![
        (
            "accept".to_string(),
            "application/vnd.github+json".to_string(),
        ),
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
pub(super) fn latest_release(
    ctx: &Context,
    repo: &str,
) -> Result<Value, ResolveError> {
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
        None => Err(ResolveError::Failed(format!(
            "{list_url}: the repository has no releases"
        ))),
    };
}

/// The asset of `release` with exactly this name,
/// or `None`.
pub fn find_asset<'a>(
    release: &'a Value,
    name: &str,
) -> Option<&'a Value> {
    let assets: &[Value] = release
        .get("assets")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for asset in assets {
        if text_at(asset, &["name"]) == Some(name) {
            return Some(asset);
        }
    }
    return None;
}

/// One URL path segment:
/// every byte outside the unreserved set is percent-encoded.
/// Why:
/// A git tag may hold `#`, `?`, `%` and `+`;
/// left as they are, they end the path at a fragment,
/// start a query or spell another character,
/// so the request would name a different release or none.
fn encode_path_segment(text: &str) -> String {
    let mut encoded: String = String::new();
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    return encoded;
}

/// The release object of one tag of a repository.
pub fn release_by_tag(
    ctx: &Context,
    repo: &str,
    tag: &str,
) -> Result<Value, ResolveError> {
    let url: String = format!(
        "https://api.github.com/repos/{repo}/releases/tags/{}",
        encode_path_segment(tag)
    );
    return get_json(ctx, &url, &api_headers(ctx));
}

/// SHA-256 of a release asset:
/// the `sha256:` digest GitHub reports,
/// or the hash of the downloaded file.
/// Why:
/// GitHub may report a digest of another algorithm (or none),
/// and recording that text as the SHA-256 would write a lock row that can never verify.
pub fn asset_digest(
    ctx: &Context,
    asset: &Value,
    url: &str,
) -> Result<String, ResolveError> {
    if let Some(sha256) = asset
        .get("digest")
        .and_then(Value::as_str)
        .and_then(|digest| return digest.strip_prefix("sha256:"))
    {
        return Ok(sha256.to_string());
    }
    return hash_download(ctx, url);
}
