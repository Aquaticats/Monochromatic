//! What:
//! The three package registries:
//! npm (`registry.npmjs.org`),
//! PyPI (`pypi.org`)
//! and crates.io,
//! each asked directly over HTTPS for a package's newest version or for the release of a wanted one.
//! Why:
//! The decision record fixes npm and PyPI lookups to go to their registries directly,
//! never through `mpm outdated` (probe 3: mpm drops failed managers with exit 0).
//! The package name is checked before it is put in a URL,
//! so a plan entry cannot redirect the lookup.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-registry.ts
//! export function resolveNpm(ctx: Context, name: string, wanted?: string): Resolution
//! export function resolvePypi(ctx: Context, declaration: Declaration, name: string, wanted?: string): Resolution
//! export function resolveCrate(ctx: Context, name: string, wanted?: string): Resolution
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the model types the resolvers build.
use super::model::{Artifact, Declaration};
/// Imports the shared resolver helpers.
use super::update_resolve::{LINUX_PLATFORMS, Resolution, ResolveError, by_name, get_json, text_at};

/// What:
/// A package name made safe to put in a registry URL path:
/// letters, digits and `. _ - @ /` only,
/// with `/` (npm scopes) written `%2F`;
/// anything else is refused.
/// Why:
/// A `?`, `#`, space or `..` in a name would change what the request asks for.
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

/// Whether a range names a version (it starts with a digit) rather than a tag or channel.
fn is_version(range: &str) -> bool {
    return range.starts_with(|character: char| return character.is_ascii_digit());
}

/// What:
/// npm:
/// `GET https://registry.npmjs.org/<name>` with the abbreviated-document `Accept` header,
/// then `dist-tags.<tag>`,
/// where the tag is the range (`latest` when none).
/// Why:
/// An exact version needs no lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveNpm(ctx: Context, name: string, wanted?: string): Resolution
/// ```
pub fn resolve_npm(ctx: &Context, name: &str, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let tag: &str = wanted.unwrap_or("latest");
    if is_version(tag) {
        return Ok(by_name(tag));
    }
    let url: String = format!("https://registry.npmjs.org/{}", registry_path_name(name)?);
    let headers: Vec<(String, String)> = vec![("accept".to_string(), "application/vnd.npm.install-v1+json".to_string())];
    let document: Value = get_json(ctx, &url, &headers)?;
    return match text_at(&document, &["dist-tags", tag]) {
        Some(version) => Ok(by_name(version)),
        None => Err(ResolveError::Failed(format!("{url}: the registry reports no {tag} dist-tag for {name}"))),
    };
}

/// crates.io:
/// `GET https://crates.io/api/v1/crates/<name>`,
/// then `crate.max_stable_version` (an exact range needs no lookup).
pub fn resolve_crate(ctx: &Context, name: &str, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    if let Some(version) = wanted {
        return Ok(by_name(version));
    }
    let url: String = format!("https://crates.io/api/v1/crates/{}", registry_path_name(name)?);
    let document: Value = get_json(ctx, &url, &[])?;
    return match text_at(&document, &["crate", "max_stable_version"]) {
        Some(version) => Ok(by_name(version)),
        None => Err(ResolveError::Failed(format!("{url}: crates.io reports no stable version for {name}"))),
    };
}

/// What:
/// PyPI:
/// `GET https://pypi.org/pypi/<name>/json` for the newest version (`info.version`),
/// or `.../<name>/<version>/json` for a wanted one.
/// Why:
/// A wheel bootstrap also records the wheel's URL and SHA-256 from the same document's `urls`,
/// for the four Linux platforms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolvePypi(ctx: Context, declaration: Declaration, name: string, wanted?: string): Resolution
/// ```
pub fn resolve_pypi(ctx: &Context, declaration: &Declaration, name: &str, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let is_wheel: bool = declaration.options.bootstrap.as_ref().is_some_and(|bootstrap| return bootstrap.distribution == "wheel");
    if let Some(version) = wanted
        && !is_wheel
    {
        return Ok(by_name(version));
    }
    let url: String = match wanted {
        Some(version) => format!("https://pypi.org/pypi/{}/{}/json", registry_path_name(name)?, registry_path_name(version)?),
        None => format!("https://pypi.org/pypi/{}/json", registry_path_name(name)?),
    };
    let document: Value = get_json(ctx, &url, &[])?;
    let version: String = match wanted {
        Some(exact) => exact.to_string(),
        None => match text_at(&document, &["info", "version"]) {
            Some(newest) => newest.to_string(),
            None => return Err(ResolveError::Failed(format!("{url}: PyPI reports no version for {name}"))),
        },
    };
    if !is_wheel {
        return Ok(by_name(&version));
    }
    return Ok(Resolution { version, artifacts: wheel_rows(&document, &url, &declaration.id)? });
}

/// The wheel artifact rows of a release document:
/// its pure-Python wheel,
/// recorded for the four Linux platforms.
fn wheel_rows(document: &Value, url: &str, id: &str) -> Result<Vec<(String, Artifact)>, ResolveError> {
    let files: &[Value] = document.get("urls").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    for file in files {
        if text_at(file, &["packagetype"]) != Some("bdist_wheel") {
            continue;
        }
        let (Some(wheel_url), Some(digest)) = (text_at(file, &["url"]), text_at(file, &["digests", "sha256"])) else {
            continue;
        };
        if !wheel_url.ends_with("-py3-none-any.whl") {
            continue;
        }
        let mut rows: Vec<(String, Artifact)> = Vec::new();
        for platform in LINUX_PLATFORMS {
            rows.push((platform.to_string(), Artifact { url: wheel_url.to_string(), sha256: digest.to_string() }));
        }
        return Ok(rows);
    }
    return Err(ResolveError::Failed(format!("{url}: no pure-Python wheel (py3-none-any) is published for {id}")));
}
