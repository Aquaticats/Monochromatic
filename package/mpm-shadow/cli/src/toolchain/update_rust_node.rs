//! What:     Two native lookups: the Rust channel manifests on `static.rust-lang.org` (the nightly date, or a stable
//!           channel's version) and the Node.js release index and checksum files on `nodejs.org`.
//! Why:      Neither goes through a package manager: Rust channels install by their dated name, and Node's archives
//!           are locked by the SHA-256 rows of its `SHASUMS256.txt`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-rust-node.ts
//! export function resolveRust(ctx: Context, declaration: Declaration): Resolution
//! export function resolveNode(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
//! export function parseShasums(text: string): Map<string, string>
//! ```

use serde_json::Value;
use std::collections::BTreeMap;

use super::context::Context;
use super::model::{Artifact, Declaration, LockedTool, Request, is_sha256_hex};
use super::update_resolve::{Resolution, ResolveError, get_json, get_text, not_implemented, text_at};

/// The value of the first `key = "value"` line in a TOML text, or `None`.
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
        if let Some(inner) = value.strip_prefix('"').and_then(|quoted| return quoted.split('"').next())
            && !inner.is_empty()
        {
            return Some(inner.to_string());
        }
    }
    return None;
}

/// The `version` assigned on the line after the `[pkg.rust]` header, or `None`.
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

/// What:     The Rust channel's current release: `nightly-<date>` for the nightly channel, the version's first word
///           (`1.90.0`) for any other channel.
/// Why:      Nightlies install by their dated name, so the lock holds the date, not a version number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveRust(ctx: Context, declaration: Declaration): Resolution
/// ```
pub fn resolve_rust(ctx: &Context, declaration: &Declaration) -> Result<Resolution, ResolveError> {
    let channel: String = match &declaration.request {
        Request::Channel(name) => name.clone(),
        _ => "nightly".to_string(),
    };
    if channel.is_empty() || !channel.chars().all(|character| return character.is_ascii_alphanumeric() || ".-_".contains(character)) {
        return Err(ResolveError::Failed(format!("{channel:?} is not a Rust channel name")));
    }
    let url: String = format!("https://static.rust-lang.org/dist/channel-rust-{channel}.toml");
    let text: String = get_text(ctx, &url, &[])?;
    if channel == "nightly" {
        return match first_assignment(&text, "date") {
            Some(date) => Ok(Resolution { version: format!("nightly-{date}"), artifacts: Vec::new() }),
            None => Err(ResolveError::Failed(format!("{url}: the nightly manifest carries no date"))),
        };
    }
    return match pkg_rust_version(&text) {
        Some(version) => {
            let first_word: String = version.split_whitespace().next().unwrap_or("").to_string();
            Ok(Resolution { version: first_word, artifacts: Vec::new() })
        }
        None => Err(ResolveError::Failed(format!("{url}: the {channel} manifest carries no [pkg.rust] version"))),
    };
}

/// What:     Reads a `SHASUMS256.txt`: lines of `<64 hex>  <file name>` (a `*` before the name marks binary mode) into
///           file name to digest.
/// Why:      Lines that are not a digest followed by a name are ignored rather than trusted.
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
    return parts.len() == 3 && parts.iter().all(|part| return !part.is_empty() && part.chars().all(|character| return character.is_ascii_digit()));
}

/// The hosts whose `SHASUMS256.txt` cover the official and the musl builds.
const SUMS_HOSTS: [&str; 2] = ["https://nodejs.org/dist", "https://unofficial-builds.nodejs.org/download/release"];

/// What:     The newest stable Node.js release and its archives: the platforms and file-name shapes come from the old
///           lock rows (with the old version replaced), the digests from the release's `SHASUMS256.txt` files.
/// Why:      Nothing here is guessed: an archive with no checksum row fails the lookup, naming the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveNode(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
/// ```
pub fn resolve_node(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>) -> Result<Resolution, ResolveError> {
    let Some(locked) = current.filter(|tool| return !tool.artifacts.is_empty()) else {
        return Err(not_implemented(declaration, "resolving Node.js archives for platforms the lock does not yet list"));
    };
    let index: Value = get_json(ctx, "https://nodejs.org/dist/index.json", &[])?;
    let Some(releases) = index.as_array() else {
        return Err(ResolveError::Failed("https://nodejs.org/dist/index.json: the index is not a list".to_string()));
    };
    let newest: Option<&str> = releases.iter().filter_map(|release| return text_at(release, &["version"])).find(|tag| return is_stable_tag(tag));
    let Some(new_tag) = newest else {
        return Err(ResolveError::Failed("https://nodejs.org/dist/index.json: no stable release is listed".to_string()));
    };
    let new_version: String = new_tag.trim_start_matches('v').to_string();
    let mut sums: BTreeMap<String, String> = BTreeMap::new();
    for host in SUMS_HOSTS {
        let url: String = format!("{host}/v{new_version}/SHASUMS256.txt");
        if let Ok(response) = ctx.http.get(&url, &[], super::http::DEFAULT_MAX_GET_BYTES)
            && (200..300).contains(&response.status)
        {
            sums.extend(parse_shasums(&response.text()));
        }
    }
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for (platform, row) in &locked.artifacts {
        let old_name: &str = row.url.rsplit('/').next().unwrap_or("");
        let new_name: String = old_name.replace(&locked.version, &new_version);
        let Some(digest) = sums.get(&new_name) else {
            return Err(ResolveError::Failed(format!("SHASUMS256.txt of Node.js {new_version} has no row for {new_name}")));
        };
        artifacts.push((platform.clone(), Artifact { url: row.url.replace(&locked.version, &new_version), sha256: digest.clone() }));
    }
    return Ok(Resolution { version: new_version, artifacts });
}
