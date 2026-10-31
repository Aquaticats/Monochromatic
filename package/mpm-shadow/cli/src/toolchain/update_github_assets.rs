//! What:
//! Choosing the release asset for one platform by the usual naming convention:
//! the file name must say Linux and the platform's CPU,
//! must not be a checksum, signature, installer package or debug build,
//! and prefers the platform's C library and the commonest archive format.
//! Why:
//! The choice must be the same on every run and must never guess:
//! when two assets are equally good the chooser fails and names them,
//! and the plan's `asset` hint (a glob) is how a person settles it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-github-assets.ts
//! export function pickAsset(assets: Asset[], platform: string, hint?: string, repo: string, tag: string): Asset | undefined
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports the glob matcher.
use super::update_github::glob_matches;
/// Imports the shared resolver helpers.
use super::update_resolve::{ResolveError, is_arm64, is_musl, text_at};

/// File-name endings that are never the program:
/// checksums,
/// signatures,
/// installer packages,
/// scripts and metadata.
const EXCLUDED_ENDINGS: [&str; 26] = [
    ".sha256", ".sha256sum", ".sha512", ".sig", ".asc", ".sbom", ".json", ".jsonl", ".txt", ".md", ".pem", ".minisig", ".deb", ".rpm", ".msi", ".apk", ".dmg",
    ".pkg", ".exe", ".sh", ".ps1", ".b3", ".crt", ".bundle", ".whl", ".vsix",
];

/// Name fragments that mark a build the toolchain does not want:
/// debug symbols,
/// baseline CPU builds,
/// sources and documentation.
const EXCLUDED_FRAGMENTS: [&str; 12] = ["baseline", "debug", "symbols", "-src", "_src", "source", "docs", "completion", "provenance", "checksums", "profile", "sbom"];

/// Name fragments of operating systems and CPUs other than the platforms looked for.
const FOREIGN_FRAGMENTS: [&str; 22] = [
    "windows", "win32", "win64", "darwin", "macos", "apple", "freebsd", "netbsd", "openbsd", "android", "i686", "i386", "-386", "armv6", "armv7", "ppc64", "s390x",
    "riscv64", "loongarch", "mips", "wasm", "illumos",
];

/// Whether the name says Linux (`ubuntu` builds count).
fn says_linux(lower: &str) -> bool {
    return lower.contains("linux") || lower.contains("ubuntu");
}

/// Whether the name says the platform's CPU and not the other one.
fn says_cpu(lower: &str, arm64: bool) -> bool {
    let arm_tokens: [&str; 3] = ["aarch64", "arm64", "armv8"];
    let x64_tokens: [&str; 4] = ["x86_64", "amd64", "x64", "x86-64"];
    let (wanted, unwanted): (&[&str], &[&str]) = if arm64 { (&arm_tokens, &x64_tokens) } else { (&x64_tokens, &arm_tokens) };
    let has_wanted: bool = wanted.iter().any(|token| return lower.contains(token));
    let has_unwanted: bool = unwanted.iter().any(|token| return lower.contains(token));
    return has_wanted && !has_unwanted;
}

/// The score of a file format:
/// higher is preferred.
fn format_score(lower: &str) -> u32 {
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        return 5;
    }
    if lower.ends_with(".tar.xz") {
        return 4;
    }
    if lower.ends_with(".zip") {
        return 3;
    }
    if lower.ends_with(".gz") {
        return 2;
    }
    return 1;
}

/// The score of an asset for a platform,
/// or `None` when the name rules it out.
fn score(name: &str, platform: &str) -> Option<u32> {
    let lower: String = name.to_ascii_lowercase();
    if EXCLUDED_ENDINGS.iter().any(|ending| return lower.ends_with(ending)) {
        return None;
    }
    if EXCLUDED_FRAGMENTS.iter().any(|fragment| return lower.contains(fragment)) {
        return None;
    }
    if FOREIGN_FRAGMENTS.iter().any(|fragment| return lower.contains(fragment)) {
        return None;
    }
    if !says_linux(&lower) || !says_cpu(&lower, is_arm64(platform)) {
        return None;
    }
    let libc_matches: bool = lower.contains("musl") == is_musl(platform);
    return Some(if libc_matches { 100 } else { 0 } + format_score(&lower));
}

/// What:
/// The asset of a release that fits `platform`,
/// `Ok(None)` when none does,
/// and an error naming the candidates when several fit equally well.
/// Why:
/// A `hint` glob restricts the candidates first,
/// so a repository that ships several programs in one release can say which one it means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function pickAsset(assets, platform, hint, repo, tag): Asset | undefined  // throws on a tie
/// ```
pub fn pick_asset<'a>(assets: &'a [Value], platform: &str, hint: Option<&str>, repo: &str, tag: &str) -> Result<Option<&'a Value>, ResolveError> {
    let mut best: Vec<&Value> = Vec::new();
    let mut best_score: u32 = 0;
    for asset in assets {
        let Some(name) = text_at(asset, &["name"]) else {
            continue;
        };
        if let Some(pattern) = hint
            && !glob_matches(pattern, name)
        {
            continue;
        }
        let Some(points) = score(name, platform) else {
            continue;
        };
        if points > best_score {
            best_score = points;
            best.clear();
        }
        if points == best_score {
            best.push(asset);
        }
    }
    if best.len() > 1 {
        let mut names: Vec<&str> = Vec::new();
        for asset in &best {
            names.push(text_at(asset, &["name"]).unwrap_or(""));
        }
        return Err(ResolveError::Failed(format!(
            "github.com/{repo} {tag} has {} equally good assets for {platform}: {}; set `options.asset` in the plan entry to a glob that matches exactly one",
            names.len(),
            names.join(", ")
        )));
    }
    return Ok(best.first().copied());
}
