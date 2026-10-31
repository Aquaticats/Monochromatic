//! What:
//! Two lookups for tools that live outside the usual sources:
//! the Temurin JDK (the Adoptium API names the newest release,
//! the GitHub release of that tag holds the archives)
//! and the AWS CLI (the newest stable 2.x tag of `aws/aws-cli`,
//! whose installers publish no checksum).
//! Why:
//! The JDK's archives are ordinary GitHub assets with a digest.
//! The AWS CLI installers are hashed from the download the update makes itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-java-aws.ts
//! export function resolveJava(ctx, declaration, current?, wanted?): Resolution
//! export function resolveAws(ctx, declaration, current?, wanted?): Resolution
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the model types the resolvers build.
use super::model::Artifact;
/// Imports the GitHub release helpers.
use super::update_github::{asset_digest, find_asset, release_by_tag};
/// Imports the shared resolver helpers.
use super::update_resolve::{LINUX_PLATFORMS, Resolution, ResolveError, get_json, hash_download, is_arm64, is_musl, text_at};
/// Imports the shared named helpers.
use super::util::{all_ascii_digits, number_or_zero};

/// The `release_name` of the first record of an Adoptium answer.
fn first_release_name(records: &Value) -> Option<&str> {
    let list: &Vec<Value> = records.as_array()?;
    let first: &Value = list.first()?;
    return text_at(first, &["release_name"]);
}

/// The Adoptium asset name of one platform's JDK archive for a major version and a release core (`21.0.13+9`).
fn jdk_asset_name(major: &str, core: &str, platform: &str) -> String {
    let arch: &str = if is_arm64(platform) { "aarch64" } else { "x64" };
    let os: &str = if is_musl(platform) { "alpine-linux" } else { "linux" };
    return format!("OpenJDK{major}U-jdk_{arch}_{os}_hotspot_{}.tar.gz", core.replace('+', "_"));
}

/// What:
/// Resolves the Temurin JDK for a major line (`java:temurin@21`) or for one release (`java:temurin@21.0.13+9`):
/// the newest release the Adoptium API lists for the major version,
/// then the GitHub release of that tag in `adoptium/temurin<major>-binaries`,
/// with the archive of each Linux platform picked by name and its digest taken from the asset.
/// Why:
/// The lock's version is `temurin-<release>.0.LTS`,
/// and glibc and musl platforms use the `linux` and `alpine-linux` builds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveJava(ctx: Context, range: string): Resolution
/// ```
pub fn resolve_java(ctx: &Context, range: &str) -> Result<Resolution, ResolveError> {
    let major: String = range.chars().take_while(char::is_ascii_digit).collect();
    if major.is_empty() {
        return Err(ResolveError::Failed(format!("{range:?} is not a Java major version or release (write java:temurin@21)")));
    }
    let core: String = if all_ascii_digits(range) {
        let url: String = format!("https://api.adoptium.net/v3/assets/latest/{major}/hotspot?architecture=x64&image_type=jdk&os=linux");
        let records: Value = get_json(ctx, &url, &[])?;
        let release_name: Option<&str> = first_release_name(&records);
        let Some(name) = release_name else {
            return Err(ResolveError::Failed(format!("{url}: Adoptium reports no JDK release")));
        };
        name.strip_prefix("jdk-").unwrap_or(name).to_string()
    } else {
        range.to_string()
    };
    let repo: String = format!("adoptium/temurin{major}-binaries");
    let release: Value = release_by_tag(ctx, &repo, &format!("jdk-{core}"))?;
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        let name: String = jdk_asset_name(&major, &core, platform);
        let Some(asset) = find_asset(&release, &name) else {
            return Err(ResolveError::Failed(format!("github.com/{repo} release jdk-{core} has no asset {name}")));
        };
        let Some(url) = text_at(asset, &["browser_download_url"]) else {
            return Err(ResolveError::Failed(format!("release asset {name} carries no download URL")));
        };
        artifacts.push((platform.to_string(), Artifact { url: url.to_string(), sha256: asset_digest(ctx, asset, url)? }));
    }
    return Ok(Resolution { version: format!("temurin-{core}.0.LTS"), artifacts });
}

/// Whether a tag is `2.<number>.<number>`.
fn is_stable_v2_tag(tag: &str) -> bool {
    let parts: Vec<&str> = tag.split('.').collect();
    if parts.len() != 3 || parts[0] != "2" {
        return false;
    }
    for part in parts {
        if part.is_empty() || !all_ascii_digits(part) {
            return false;
        }
    }
    return true;
}

/// The three numbers of a tag that passed `is_stable_v2_tag`.
fn release_numbers(tag: &str) -> (u64, u64, u64) {
    let mut numbers = tag.split('.').map(number_or_zero);
    return (numbers.next().unwrap_or(0), numbers.next().unwrap_or(0), numbers.next().unwrap_or(0));
}

/// What:
/// Resolves the AWS CLI:
/// the newest stable `2.x.y` tag of `aws/aws-cli` (or the wanted version),
/// the installer URL of each Linux architecture,
/// and a SHA-256 computed from the download.
/// Why:
/// The installers come with no checksum file,
/// and the glibc and musl platforms share one URL per architecture,
/// so each distinct URL is downloaded once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveAws(ctx: Context, wanted?: string): Resolution
/// ```
pub fn resolve_aws(ctx: &Context, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let version: String = match wanted {
        Some(exact) => exact.to_string(),
        None => newest_aws_version(ctx)?,
    };
    let mut hashed: Vec<(String, String)> = Vec::new();
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        let arch: &str = if is_arm64(platform) { "aarch64" } else { "x86_64" };
        let url: String = format!("https://awscli.amazonaws.com/awscli-exe-linux-{arch}-{version}.zip");
        let mut known: Option<String> = None;
        for (hashed_url, hash) in &hashed {
            if *hashed_url == url {
                known = Some(hash.clone());
            }
        }
        let sha256: String = match known {
            Some(hash) => hash,
            None => {
                let computed: String = hash_download(ctx, &url)?;
                hashed.push((url.clone(), computed.clone()));
                computed
            }
        };
        artifacts.push((platform.to_string(), Artifact { url, sha256 }));
    }
    return Ok(Resolution { version, artifacts });
}

/// The newest stable 2.x tag in the first hundred tags of `aws/aws-cli`.
fn newest_aws_version(ctx: &Context) -> Result<String, ResolveError> {
    let url: &str = "https://api.github.com/repos/aws/aws-cli/tags?per_page=100";
    let tags: Value = get_json(ctx, url, &super::update_github::api_headers(ctx))?;
    let mut newest: Option<&str> = None;
    for tag in tags.as_array().map(Vec::as_slice).unwrap_or(&[]) {
        let Some(name) = text_at(tag, &["name"]) else {
            continue;
        };
        if !is_stable_v2_tag(name) {
            continue;
        }
        let newer: bool = match newest {
            Some(best) => release_numbers(name) > release_numbers(best),
            None => true,
        };
        if newer {
            newest = Some(name);
        }
    }
    return match newest {
        Some(version) => Ok(version.to_string()),
        None => Err(ResolveError::Failed(format!("{url}: no stable 2.x release is listed"))),
    };
}
