//! What:
//! Three native lookups:
//! the Go runtime from the `go.dev` download index,
//! a Go module's newest version from `proxy.golang.org`,
//! and Zig from the `ziglang.org` download index.
//! Why:
//! Each publishes its own checksums,
//! so the lock records the publisher's SHA-256 and nothing is downloaded to hash.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-go-zig.ts
//! export function resolveGoRuntime(ctx, declaration, current?, wanted?): Resolution
//! export function resolveGoModule(ctx, declaration): Resolution
//! export function resolveZig(ctx, declaration, current?, wanted?): Resolution
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the model types the resolvers build.
use super::model::Artifact;
/// Imports the shared resolver helpers.
use super::update_resolve::{LINUX_PLATFORMS, Resolution, ResolveError, by_name, get_json, is_arm64, text_at};

/// One Go release's Linux archive for an architecture (`amd64` or `arm64`):
/// the file name and its SHA-256.
fn go_archive<'a>(release: &'a Value, arch: &str) -> Option<(&'a str, &'a str)> {
    let files: &[Value] = release.get("files").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    for file in files {
        if text_at(file, &["kind"]) == Some("archive") && text_at(file, &["os"]) == Some("linux") && text_at(file, &["arch"]) == Some(arch) {
            return Some((text_at(file, &["filename"])?, text_at(file, &["sha256"])?));
        }
    }
    return None;
}

/// What:
/// Resolves the Go runtime:
/// the newest stable release in `https://go.dev/dl/?mode=json&include=all` (or the wanted version),
/// with its Linux archives and the SHA-256 the index lists for each.
/// Why:
/// The musl platforms reuse the glibc archive,
/// which is statically linked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveGoRuntime(ctx, declaration, current?, wanted?): Resolution
/// ```
pub fn resolve_go_runtime(ctx: &Context, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let url: &str = "https://go.dev/dl/?mode=json&include=all";
    let index: Value = get_json(ctx, url, &[])?;
    let releases: &[Value] = index.as_array().map(Vec::as_slice).unwrap_or(&[]);
    let mut chosen: Option<&Value> = None;
    for release in releases {
        let name: &str = text_at(release, &["version"]).unwrap_or("");
        let matches: bool = match wanted {
            Some(version) => name.strip_prefix("go") == Some(version),
            None => release.get("stable").and_then(Value::as_bool) == Some(true),
        };
        if matches {
            chosen = Some(release);
            break;
        }
    }
    let Some(release) = chosen else {
        return Err(ResolveError::Failed(format!("{url}: the index lists no matching Go release")));
    };
    let version: String = text_at(release, &["version"]).unwrap_or("").trim_start_matches("go").to_string();
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        let arch: &str = if is_arm64(platform) { "arm64" } else { "amd64" };
        let Some((filename, sha256)) = go_archive(release, arch) else {
            return Err(ResolveError::Failed(format!("{url}: go{version} lists no linux {arch} archive")));
        };
        artifacts.push((platform.to_string(), Artifact { url: format!("https://go.dev/dl/{filename}"), sha256: sha256.to_string() }));
    }
    return Ok(Resolution { version, artifacts });
}

/// The module path as the Go proxy wants it:
/// every capital letter becomes `!` and its lowercase.
fn escape_module(module: &str) -> Result<String, ResolveError> {
    let mut escaped: String = String::new();
    for character in module.chars() {
        if character.is_ascii_uppercase() {
            escaped.push('!');
            escaped.push(character.to_ascii_lowercase());
        } else if character.is_ascii_alphanumeric() || ".-_/~".contains(character) {
            escaped.push(character);
        } else {
            return Err(ResolveError::Failed(format!("{module:?} is not a Go module path")));
        }
    }
    if module.is_empty() || module.contains("..") {
        return Err(ResolveError::Failed(format!("{module:?} is not a Go module path")));
    }
    return Ok(escaped);
}

/// `GET https://proxy.golang.org/<module>/@latest`,
/// then `Version` without its `v`.
pub fn resolve_go_module(ctx: &Context, module: &str, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    if let Some(version) = wanted {
        return Ok(by_name(version));
    }
    let url: String = format!("https://proxy.golang.org/{}/@latest", escape_module(module)?);
    let document: Value = get_json(ctx, &url, &[])?;
    return match text_at(&document, &["Version"]) {
        Some(tag) => Ok(by_name(tag.trim_start_matches('v'))),
        None => Err(ResolveError::Failed(format!("{url}: the proxy reports no version for {module}"))),
    };
}

/// A version segment as a number (0 when it is not one).
fn number_or_zero(part: &str) -> u64 {
    return part.parse::<u64>().unwrap_or(0);
}

/// Compares two dotted version keys numerically,
/// segment by segment (a missing segment counts as 0).
fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let left_parts: Vec<u64> = left.split('.').map(number_or_zero).collect();
    let right_parts: Vec<u64> = right.split('.').map(number_or_zero).collect();
    let length: usize = left_parts.len().max(right_parts.len());
    for at in 0..length {
        let one: u64 = left_parts.get(at).copied().unwrap_or(0);
        let other: u64 = right_parts.get(at).copied().unwrap_or(0);
        if one != other {
            return one.cmp(&other);
        }
    }
    return std::cmp::Ordering::Equal;
}

/// The newest stable key (not `master`, no `-` suffix) of the Zig download index.
fn newest_zig_version(index: &Value) -> Option<String> {
    let mut newest: Option<&String> = None;
    for key in index.as_object()?.keys() {
        if key == "master" || key.contains('-') {
            continue;
        }
        let newer: bool = match newest {
            Some(best) => compare_versions(key, best) == std::cmp::Ordering::Greater,
            None => true,
        };
        if newer {
            newest = Some(key);
        }
    }
    return newest.cloned();
}

/// What:
/// Resolves Zig:
/// the newest stable release in `https://ziglang.org/download/index.json` (or the wanted version),
/// with the tarball and `shasum` the index lists for each architecture.
/// Why:
/// The TypeScript resolver reported the new version but kept the old artifact rows;
/// this one records the new tarballs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveZig(ctx, declaration, current?, wanted?): Resolution
/// ```
pub fn resolve_zig(ctx: &Context, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let url: &str = "https://ziglang.org/download/index.json";
    let index: Value = get_json(ctx, url, &[])?;
    let version: String = match wanted {
        Some(version) => version.to_string(),
        None => match newest_zig_version(&index) {
            Some(newest) => newest,
            None => return Err(ResolveError::Failed(format!("{url}: the index lists no stable release"))),
        },
    };
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        let key: &str = if is_arm64(platform) { "aarch64-linux" } else { "x86_64-linux" };
        let tarball: Option<&str> = index.get(&version).and_then(|release| return text_at(release, &[key, "tarball"]));
        let shasum: Option<&str> = index.get(&version).and_then(|release| return text_at(release, &[key, "shasum"]));
        let (Some(tarball_url), Some(sha256)) = (tarball, shasum) else {
            return Err(ResolveError::Failed(format!("{url}: Zig {version} lists no {key} tarball")));
        };
        artifacts.push((platform.to_string(), Artifact { url: tarball_url.to_string(), sha256: sha256.to_string() }));
    }
    return Ok(Resolution { version, artifacts });
}
