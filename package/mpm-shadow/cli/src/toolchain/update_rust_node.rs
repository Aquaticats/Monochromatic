//! What:
//! Three native lookups:
//! the Rust channel manifests on `static.rust-lang.org` (the nightly date,
//! or a stable channel's version),
//! the rustup release and its `rustup-init` downloads,
//! and the Node.js release index and checksum files on `nodejs.org`.
//! Why:
//! None goes through a package manager.
//! Rust channels install by their dated name.
//! Node's and rustup's archives are locked by the SHA-256 the publisher lists.
//! Every row is built from the platform and the version alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-rust-node.ts
//! export function resolveRust(ctx: Context, range: string): Resolution
//! export function resolveRustup(ctx: Context, wanted?: string): Resolution
//! export function resolveNode(ctx: Context, wanted?: string): Resolution
//! export function parseShasums(text: string): Map<string, string>
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `BTreeMap` from `std::collections`.
use std::collections::BTreeMap;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the model types the resolvers build.
use super::model::{Artifact, is_sha256_hex};
/// Imports the shared resolver helpers.
use super::update_resolve::{LINUX_PLATFORMS, Resolution, ResolveError, by_name, get_json, get_text, is_arm64, is_musl, text_at};
/// Imports the shared named helpers.
use super::util::all_ascii_digits;

/// The value of the first `key = "value"` (or `'value'`) line in a TOML text,
/// or `None`.
fn first_assignment(text: &str, key: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed: &str = line.trim();
        let Some(rest) = trimmed.strip_prefix(key) else {
            continue;
        };
        let Some(after_equals) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let value: &str = after_equals.trim();
        for quote in ['"', '\''] {
            if let Some(quoted) = value.strip_prefix(quote)
                && let Some(inner) = quoted.split(quote).next()
                && !inner.is_empty()
            {
                return Some(inner.to_string());
            }
        }
    }
    return None;
}

/// The `version` assigned on the line after the `[pkg.rust]` header,
/// or `None`.
fn pkg_rust_version(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() == "[pkg.rust]"
            && let Some(next) = lines.get(index + 1)
            && let Some(version) = first_assignment(next, "version")
        {
            return Some(version);
        }
    }
    return None;
}

/// What:
/// The Rust channel's current release:
/// `nightly-<date>` for the nightly channel,
/// the version's first word (`1.90.0`) for `stable` or `beta`;
/// any other range is already an exact release name (`nightly-2026-10-29`,
/// `1.99.0`) and needs no lookup.
/// Why:
/// Nightlies install by their dated name,
/// so the lock holds the date,
/// not a version number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveRust(ctx: Context, range: string): Resolution
/// ```
pub fn resolve_rust(ctx: &Context, range: &str) -> Result<Resolution, ResolveError> {
    if !matches!(range, "nightly" | "beta" | "stable") {
        return Ok(by_name(range));
    }
    let url: String = format!("https://static.rust-lang.org/dist/channel-rust-{range}.toml");
    let text: String = get_text(ctx, &url, &[])?;
    if range == "nightly" {
        return match first_assignment(&text, "date") {
            Some(date) => Ok(by_name(&format!("nightly-{date}"))),
            None => Err(ResolveError::Failed(format!("{url}: the nightly manifest carries no date"))),
        };
    }
    return match pkg_rust_version(&text) {
        Some(version) => Ok(by_name(version.split_whitespace().next().unwrap_or(""))),
        None => Err(ResolveError::Failed(format!("{url}: the {range} manifest carries no [pkg.rust] version"))),
    };
}

/// What:
/// Reads a `SHASUMS256.txt`:
/// lines of `<64 hex>  <file name>` (a `*` before the name marks binary mode) into file name to digest.
/// Why:
/// Lines that are not a digest followed by a name are ignored rather than trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseShasums(text: string): Map<string, string>
/// ```
pub fn parse_shasums(text: &str) -> BTreeMap<String, String> {
    let mut sums: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        if line.len() <= 64 || !line.is_char_boundary(64) {
            continue;
        }
        let (digest, rest) = line.split_at(64);
        let named: &str = rest.trim_start();
        if named.len() == rest.len() || !is_sha256_hex(digest) {
            continue;
        }
        let name: &str = named.strip_prefix('*').unwrap_or(named).trim_end();
        if !name.is_empty() {
            sums.insert(name.to_string(), digest.to_string());
        }
    }
    return sums;
}

/// Whether a tag is `v<number>.<number>.<number>`.
fn is_stable_tag(tag: &str) -> bool {
    let Some(numbers) = tag.strip_prefix('v') else {
        return false;
    };
    let parts: Vec<&str> = numbers.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    for part in parts {
        if part.is_empty() || !all_ascii_digits(part) {
            return false;
        }
    }
    return true;
}

/// The newest stable Node.js version listed in `https://nodejs.org/dist/index.json`.
fn newest_node_version(ctx: &Context) -> Result<String, ResolveError> {
    let url: &str = "https://nodejs.org/dist/index.json";
    let index: Value = get_json(ctx, url, &[])?;
    let Some(releases) = index.as_array() else {
        return Err(ResolveError::Failed(format!("{url}: the index is not a list")));
    };
    for release in releases {
        if let Some(tag) = text_at(release, &["version"])
            && is_stable_tag(tag)
        {
            return Ok(tag.trim_start_matches('v').to_string());
        }
    }
    return Err(ResolveError::Failed(format!("{url}: no stable release is listed")));
}

/// What:
/// The newest stable Node.js release (or the wanted one) and its archives for the four Linux platforms:
/// official builds for glibc,
/// the unofficial musl builds for musl,
/// each with the digest from the release's `SHASUMS256.txt`.
/// Why:
/// Nothing is guessed:
/// an archive with no checksum row fails the lookup,
/// naming the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveNode(ctx: Context, wanted?: string): Resolution
/// ```
pub fn resolve_node(ctx: &Context, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let version: String = match wanted {
        Some(exact) => exact.to_string(),
        None => newest_node_version(ctx)?,
    };
    let hosts: [&str; 2] = ["https://nodejs.org/dist", "https://unofficial-builds.nodejs.org/download/release"];
    let mut sums: BTreeMap<String, String> = BTreeMap::new();
    for host in hosts {
        let url: String = format!("{host}/v{version}/SHASUMS256.txt");
        if let Ok(response) = ctx.http.get(&url, &[], super::http::DEFAULT_MAX_GET_BYTES)
            && (200..300).contains(&response.status)
        {
            sums.extend(parse_shasums(&response.text()));
        }
    }
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        let arch: &str = if is_arm64(platform) { "arm64" } else { "x64" };
        let (host, suffix): (&str, &str) = if is_musl(platform) { (hosts[1], "-musl") } else { (hosts[0], "") };
        let name: String = format!("node-v{version}-linux-{arch}{suffix}.tar.gz");
        let Some(digest) = sums.get(&name) else {
            return Err(ResolveError::Failed(format!("SHASUMS256.txt of Node.js {version} has no row for {name}")));
        };
        artifacts.push((platform.to_string(), Artifact { url: format!("{host}/v{version}/{name}"), sha256: digest.clone() }));
    }
    return Ok(Resolution { version, artifacts });
}

/// The rustup target triple of a platform id.
fn rustup_triple(platform: &str) -> String {
    let arch: &str = if is_arm64(platform) { "aarch64" } else { "x86_64" };
    let libc: &str = if is_musl(platform) { "musl" } else { "gnu" };
    return format!("{arch}-unknown-linux-{libc}");
}

/// What:
/// Resolves rustup itself:
/// the version in `release-stable.toml` (or the wanted one),
/// and for each Linux platform the `rustup-init` URL with the hash from the `.sha256` file beside it.
/// Why:
/// The `rustup` tool of the machine-global lock is an archive of `rustup-init` per target.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveRustup(ctx: Context, wanted?: string): Resolution
/// ```
pub fn resolve_rustup(ctx: &Context, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let version: String = match wanted {
        Some(exact) => exact.to_string(),
        None => {
            let url: &str = "https://static.rust-lang.org/rustup/release-stable.toml";
            let text: String = get_text(ctx, url, &[])?;
            match first_assignment(&text, "version") {
                Some(newest) => newest,
                None => return Err(ResolveError::Failed(format!("{url}: the rustup release manifest carries no version"))),
            }
        }
    };
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        let triple: String = rustup_triple(platform);
        let url: String = format!("https://static.rust-lang.org/rustup/archive/{version}/{triple}/rustup-init");
        let checksum: String = get_text(ctx, &format!("{url}.sha256"), &[])?;
        let digest: &str = checksum.split_whitespace().next().unwrap_or("");
        if !is_sha256_hex(digest) {
            return Err(ResolveError::Failed(format!("{url}.sha256: the checksum for {triple} is not a SHA-256 line")));
        }
        artifacts.push((platform.to_string(), Artifact { url, sha256: digest.to_string() }));
    }
    return Ok(Resolution { version, artifacts });
}
