//! What:
//! Two lookups with their own formats:
//! the Android command-line tools from Google's `repository2-3.xml`,
//! and a conda-forge package from the `api.anaconda.org` file listing.
//! Why:
//! Google publishes no checksum for the command-line tools,
//! so the lock records the hash of the update's own download.
//! Anaconda lists a SHA-256 for every file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-android-conda.ts
//! export function resolveAndroid(ctx, declaration, current?, wanted?): Resolution
//! export function resolveConda(ctx, declaration, current?, wanted?): Resolution
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the model types the resolvers build.
use super::model::Artifact;
/// Imports the shared resolver helpers.
use super::update_resolve::{LINUX_PLATFORMS, Resolution, ResolveError, get_json, get_text, hash_download, is_arm64, text_at};
/// Imports the shared named helpers.
use super::util::all_ascii_digits;

/// One `cmdline-tools` package of the repository listing:
/// its revision and the build number in its archive name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CmdlinePackage {
    /// The revision's major number.
    pub major: u64,
    /// The revision's minor number.
    pub minor: u64,
    /// The build number in `commandlinetools-linux-<build>_latest.zip`.
    pub build: u64,
}

/// The digits between `<tag>` and `</tag>` in `block`,
/// as a number.
fn number_in_tag(block: &str, tag: &str) -> Option<u64> {
    let open: String = format!("<{tag}>");
    let close: String = format!("</{tag}>");
    let after: &str = &block[block.find(&open)? + open.len()..];
    let digits: &str = after[..after.find(&close)?].trim();
    if digits.is_empty() || !all_ascii_digits(digits) {
        return None;
    }
    return digits.parse().ok();
}

/// What:
/// Reads every Linux `cmdline-tools` package out of the repository XML:
/// the text is cut at each `<remotePackage`,
/// and a block that names a `commandlinetools-linux-<build>_latest.zip` archive and a numeric revision is kept.
/// Why:
/// The listing is a large XML document,
/// and only these three numbers per package are needed;
/// a block that does not read cleanly is skipped rather than guessed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function cmdlinePackages(xml: string): CmdlinePackage[]
/// ```
pub fn cmdline_packages(xml: &str) -> Vec<CmdlinePackage> {
    let mut packages: Vec<CmdlinePackage> = Vec::new();
    for block in xml.split("<remotePackage") {
        let open: &str = "<url>commandlinetools-linux-";
        let Some(start) = block.find(open) else {
            continue;
        };
        let after: &str = &block[start + open.len()..];
        let Some(end) = after.find("_latest.zip</url>") else {
            continue;
        };
        let Ok(build) = after[..end].parse::<u64>() else {
            continue;
        };
        let Some(revision) = block.find("<revision>") else {
            continue;
        };
        let revision_text: &str = &block[revision..];
        let (Some(major), Some(minor)) = (number_in_tag(revision_text, "major"), number_in_tag(revision_text, "minor")) else {
            continue;
        };
        packages.push(CmdlinePackage { major, minor, build });
    }
    return packages;
}

/// What:
/// Resolves the Android command-line tools:
/// the newest package in `repository2-3.xml` (or the one with the wanted `major.minor` revision),
/// its archive URL,
/// and the SHA-256 of that archive from a download.
/// Why:
/// The archive is the same file on every Linux platform,
/// so it is downloaded once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveAndroid(ctx, declaration, current?, wanted?): Resolution
/// ```
pub fn resolve_android(ctx: &Context, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    let url: &str = "https://dl.google.com/android/repository/repository2-3.xml";
    let xml: String = get_text(ctx, url, &[])?;
    let mut chosen: Option<CmdlinePackage> = None;
    for package in cmdline_packages(&xml) {
        let label: String = format!("{}.{}", package.major, package.minor);
        let eligible: bool = match wanted {
            Some(version) => label == version,
            None => true,
        };
        let better: bool = match chosen {
            Some(best) => (package.major, package.minor, package.build) > (best.major, best.minor, best.build),
            None => true,
        };
        if eligible && better {
            chosen = Some(package);
        }
    }
    let Some(package) = chosen else {
        return Err(ResolveError::Failed(format!("{url}: the Android repository lists no matching cmdline-tools archive")));
    };
    let version: String = format!("{}.{}", package.major, package.minor);
    let archive: String = format!("https://dl.google.com/android/repository/commandlinetools-linux-{}_latest.zip", package.build);
    let sha256: String = hash_download(ctx, &archive)?;
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        artifacts.push((platform.to_string(), Artifact { url: archive.clone(), sha256: sha256.clone() }));
    }
    return Ok(Resolution { version, artifacts });
}

/// The build number of a conda file listing (0 when it carries none).
fn build_number_of(file: &Value) -> i64 {
    return match file.get("attrs") {
        Some(attrs) => attrs.get("build_number").and_then(Value::as_i64).unwrap_or(0),
        None => 0,
    };
}

/// Whether a character may appear in a conda package name.
fn is_conda_character(character: char) -> bool {
    return character.is_ascii_alphanumeric() || ".-_".contains(character);
}

/// The conda file of `version` for `subdir` with the highest build number:
/// its base name (which includes the subdir) and SHA-256.
fn conda_file<'a>(files: &'a [Value], version: &str, subdir: &str) -> Option<(&'a str, &'a str)> {
    let mut best: Option<(&str, &str)> = None;
    let mut best_build: i64 = -1;
    for file in files {
        if text_at(file, &["version"]) != Some(version) || text_at(file, &["attrs", "subdir"]) != Some(subdir) {
            continue;
        }
        let build: i64 = build_number_of(file);
        if build > best_build {
            best = Some((text_at(file, &["basename"])?, text_at(file, &["sha256"])?));
            best_build = build;
        }
    }
    return best;
}

/// What:
/// Resolves a conda-forge package:
/// `latest_version` (or the wanted version) from `https://api.anaconda.org/package/conda-forge/<name>`,
/// and for each Linux platform the file of that version in its subdir (`linux-64`,
/// `linux-aarch64`).
/// Why:
/// The listing gives each file's SHA-256,
/// so nothing is downloaded to hash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveConda(ctx, declaration, current?, wanted?): Resolution
/// ```
pub fn resolve_conda(ctx: &Context, spec: &str, wanted: Option<&str>) -> Result<Resolution, ResolveError> {
    if spec.is_empty() || !spec.chars().all(is_conda_character) {
        return Err(ResolveError::Failed(format!("{spec:?} is not a conda package name")));
    }
    let url: String = format!("https://api.anaconda.org/package/conda-forge/{spec}");
    let document: Value = get_json(ctx, &url, &[])?;
    let version: String = match wanted {
        Some(version) => version.to_string(),
        None => match text_at(&document, &["latest_version"]) {
            Some(latest) => latest.to_string(),
            None => return Err(ResolveError::Failed(format!("{url}: Anaconda reports no version for {spec}"))),
        },
    };
    let files: &[Value] = document.get("files").and_then(Value::as_array).map(Vec::as_slice).unwrap_or(&[]);
    let mut artifacts: Vec<(String, Artifact)> = Vec::new();
    for platform in LINUX_PLATFORMS {
        let subdir: &str = if is_arm64(platform) { "linux-aarch64" } else { "linux-64" };
        let Some((basename, sha256)) = conda_file(files, &version, subdir) else {
            return Err(ResolveError::Failed(format!("{url}: Anaconda lists no {version} file for {subdir}")));
        };
        artifacts.push((platform.to_string(), Artifact { url: format!("https://conda.anaconda.org/conda-forge/{basename}"), sha256: sha256.to_string() }));
    }
    return Ok(Resolution { version, artifacts });
}
