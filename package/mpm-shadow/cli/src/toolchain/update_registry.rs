//! What:     The three package registries: npm (`registry.npmjs.org`), PyPI (`pypi.org`) and crates.io, each asked
//!           directly over HTTPS for a package's newest version.
//! Why:      The decision record fixes npm and PyPI lookups to go to their registries directly, never through
//!           `mpm outdated` (probe 3: mpm drops failed managers with exit 0). The package name is checked before it is
//!           put in a URL, so a plan entry cannot redirect the lookup.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-registry.ts
//! export function resolveNpm(ctx: Context, declaration: Declaration): Resolution
//! export function resolvePypi(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
//! export function resolveCrate(ctx: Context, declaration: Declaration): Resolution
//! ```

use serde_json::Value;

use super::context::Context;
use super::model::{Artifact, Declaration, LockedTool, Request};
use super::update_resolve::{Resolution, ResolveError, get_json, text_at};

/// The platforms a wheel is recorded for when the lock holds no row yet (the Linux-only stopgap of the repository).
const WHEEL_PLATFORMS: [&str; 4] = ["linux-arm64", "linux-arm64-musl", "linux-x64", "linux-x64-musl"];

/// What:     A package name made safe to put in a registry URL path: letters, digits and `. _ - @ /` only, with `/`
///           (npm scopes) written `%2F`; anything else is refused.
/// Why:      A `?`, `#`, space or `..` in a name would change what the request asks for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function registryPathName(name: string): string  // throws
/// ```
pub fn registry_path_name(name: &str) -> Result<String, ResolveError> {
    let allowed = |character: char| return character.is_ascii_alphanumeric() || ".-_@/".contains(character);
    if name.is_empty() || name.contains("..") || !name.chars().all(allowed) {
        return Err(ResolveError::Failed(format!("{name:?} is not a package name the registries accept")));
    }
    return Ok(name.replace('/', "%2F"));
}

/// npm: `GET https://registry.npmjs.org/<name>` with the abbreviated-document `Accept` header, then `dist-tags.<tag>`
/// where the tag is the request's channel or `latest`.
pub fn resolve_npm(ctx: &Context, declaration: &Declaration) -> Result<Resolution, ResolveError> {
    let name: String = package_name_of(declaration, "npm:");
    let tag: String = match &declaration.request {
        Request::Channel(channel) => channel.clone(),
        _ => "latest".to_string(),
    };
    let url: String = format!("https://registry.npmjs.org/{}", registry_path_name(&name)?);
    let headers: Vec<(String, String)> = vec![("accept".to_string(), "application/vnd.npm.install-v1+json".to_string())];
    let document: Value = get_json(ctx, &url, &headers)?;
    return match text_at(&document, &["dist-tags", &tag]) {
        Some(version) => Ok(Resolution { version: version.to_string(), artifacts: Vec::new() }),
        None => Err(ResolveError::Failed(format!("{url}: the registry reports no {tag} dist-tag for {name}"))),
    };
}

/// crates.io: `GET https://crates.io/api/v1/crates/<name>`, then `crate.max_stable_version`.
pub fn resolve_crate(ctx: &Context, declaration: &Declaration) -> Result<Resolution, ResolveError> {
    let name: String = package_name_of(declaration, "cargo:");
    let url: String = format!("https://crates.io/api/v1/crates/{}", registry_path_name(&name)?);
    let document: Value = get_json(ctx, &url, &[])?;
    return match text_at(&document, &["crate", "max_stable_version"]) {
        Some(version) => Ok(Resolution { version: version.to_string(), artifacts: Vec::new() }),
        None => Err(ResolveError::Failed(format!("{url}: crates.io reports no stable version for {name}"))),
    };
}

/// PyPI: `GET https://pypi.org/pypi/<name>/json`, then `info.version`; a wheel bootstrap also records the wheel's URL
/// and SHA-256 from the same document's `urls`.
pub fn resolve_pypi(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>) -> Result<Resolution, ResolveError> {
    let name: String = package_name_of(declaration, "pipx:");
    let url: String = format!("https://pypi.org/pypi/{}/json", registry_path_name(&name)?);
    let document: Value = get_json(ctx, &url, &[])?;
    let Some(version) = text_at(&document, &["info", "version"]) else {
        return Err(ResolveError::Failed(format!("{url}: PyPI reports no version for {name}")));
    };
    let artifacts: Vec<(String, Artifact)> = wheel_rows(declaration, current, &document, &url)?;
    return Ok(Resolution { version: version.to_string(), artifacts });
}

/// PyPI at one exact version: `GET https://pypi.org/pypi/<name>/<version>/json`, for a newly pinned PyPI tool.
pub fn resolve_pypi_version(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>, version: &str) -> Result<Resolution, ResolveError> {
    let name: String = package_name_of(declaration, "pipx:");
    let url: String = format!("https://pypi.org/pypi/{}/{}/json", registry_path_name(&name)?, registry_path_name(version)?);
    let document: Value = get_json(ctx, &url, &[])?;
    let artifacts: Vec<(String, Artifact)> = wheel_rows(declaration, current, &document, &url)?;
    return Ok(Resolution { version: version.to_string(), artifacts });
}

/// The wheel artifact rows for a wheel bootstrap (none for any other PyPI tool): the release's pure-Python wheel,
/// recorded for the platforms the old lock row names (or the Linux platforms when there is none).
fn wheel_rows(declaration: &Declaration, current: Option<&LockedTool>, document: &Value, url: &str) -> Result<Vec<(String, Artifact)>, ResolveError> {
    let is_wheel: bool = declaration.options.bootstrap.as_ref().is_some_and(|bootstrap| return bootstrap.distribution == "wheel");
    if !is_wheel {
        return Ok(Vec::new());
    }
    let files: &[Value] = document.get("urls").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    for file in files {
        let kind: Option<&str> = file.get("packagetype").and_then(Value::as_str);
        let wheel_url: Option<&str> = file.get("url").and_then(Value::as_str);
        let digest: Option<&str> = file.get("digests").and_then(|digests| return digests.get("sha256")).and_then(Value::as_str);
        if kind != Some("bdist_wheel") {
            continue;
        }
        let (Some(wheel_url_text), Some(digest_text)) = (wheel_url, digest) else {
            continue;
        };
        if !wheel_url_text.ends_with("-py3-none-any.whl") {
            continue;
        }
        let platforms: Vec<String> = match current {
            Some(locked) if !locked.artifacts.is_empty() => locked.artifacts.iter().map(|(platform, _)| return platform.clone()).collect(),
            _ => WHEEL_PLATFORMS.iter().map(|platform| return (*platform).to_string()).collect(),
        };
        return Ok(platforms
            .into_iter()
            .map(|platform| return (platform, Artifact { url: wheel_url_text.to_string(), sha256: digest_text.to_string() }))
            .collect());
    }
    return Err(ResolveError::Failed(format!("{url}: no pure-Python wheel (py3-none-any) is published for {}", declaration.id)));
}

/// The package name of a manager route: the declared package id, or the tool id without its `prefix`.
fn package_name_of(declaration: &Declaration, prefix: &str) -> String {
    if let Some(package_id) = &declaration.package_id {
        return package_id.clone();
    }
    let bare: String = declaration.id.strip_prefix(prefix).unwrap_or(&declaration.id).to_string();
    return bare;
}
