//! What:
//! Finding a tool's version and artifacts:
//! the shared types and fetch helpers,
//! and the one switch from a tool's id,
//! request and route to the resolver that answers it.
//! Why:
//! Every npm and PyPI version,
//! first or later,
//! comes from `registry.npmjs.org` and `pypi.org` directly.
//! Crates come from `crates.io`,
//! Rust channels and rustup from `static.rust-lang.org`,
//! Node from `nodejs.org`,
//! Go from `go.dev` and `proxy.golang.org`,
//! Zig from `ziglang.org`,
//! Java from `api.adoptium.net`,
//! the Android command-line tools from `dl.google.com`,
//! conda packages from `api.anaconda.org`,
//! and GitHub-hosted tools from the GitHub API.
//! Every resolver takes an optional wanted version:
//! `None` asks for the newest,
//! `Some` resolves exactly that version (a newly pinned one).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-resolve.ts
//! export type Resolution = { version: string; artifacts: [string, Artifact][] };
//! export type ResolveError = { kind: 'notImplemented' | 'failed'; message: string };
//! export function resolveTool(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Value` from `serde_json`.
use serde_json::Value;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports the download size bound from `super::http`.
use super::http::{DEFAULT_MAX_DOWNLOAD_BYTES, DEFAULT_MAX_GET_BYTES};
/// Imports the model types the resolvers build.
use super::model::{Artifact, Declaration, LockedTool, Origin};

/// What a resolver found:
/// the version to lock and the artifact rows (empty for tools installed by name and version).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    /// The resolved version.
    pub version: String,
    /// Platform id to artifact,
    /// in the order they are written.
    pub artifacts: Vec<(String, Artifact)>,
}

/// What:
/// Why a resolver gave no answer:
/// the lookup was made and went wrong.
/// Why:
/// A failed lookup stops the whole update before anything is written,
/// and its text names the URL or the field.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResolveError = { message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveError {
    /// The lookup failed;
    /// the text names the URL or the field.
    Failed(
        /// What went wrong,
        /// naming the URL or the field.
        String,
    ),
}

/// The methods of `ResolveError`.
impl ResolveError {
    /// The message to show.
    pub fn message(&self) -> &str {
        match self {
            ResolveError::Failed(text) => return text,
        }
    }
}

/// GETs `url` and returns the body text,
/// or a `Failed` error naming the URL (and the status for a non-2xx answer).
pub fn get_text(ctx: &Context, url: &str, headers: &[(String, String)]) -> Result<String, ResolveError> {
    let response = match ctx.http.get(url, headers, DEFAULT_MAX_GET_BYTES) {
        Ok(answered) => answered,
        Err(error) => return Err(ResolveError::Failed(error.to_string())),
    };
    if !(200..300).contains(&response.status) {
        return Err(ResolveError::Failed(format!("{url}: HTTP {}", response.status)));
    }
    return Ok(response.text());
}

/// GETs `url` and parses the body as JSON,
/// or a `Failed` error naming the URL.
pub fn get_json(ctx: &Context, url: &str, headers: &[(String, String)]) -> Result<Value, ResolveError> {
    let text: String = get_text(ctx, url, headers)?;
    return match serde_json::from_str(&text) {
        Ok(value) => Ok(value),
        Err(error) => Err(ResolveError::Failed(format!("{url}: the answer is not JSON: {error}"))),
    };
}

/// The string at `path` inside `value` (`["dist-tags", "latest"]`),
/// or `None`.
pub fn text_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor: &Value = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    return match cursor.as_str() {
        Some(text) if !text.is_empty() => Some(text),
        _ => None,
    };
}

/// What:
/// The SHA-256 of the file behind `url`,
/// by downloading it to a scratch file and deleting the file afterwards.
/// Why:
/// Some sources (the Android archive,
/// the AWS CLI installer) publish no checksum the update can read,
/// so the lock records the hash of what the update itself downloaded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function hashDownload(ctx: Context, url: string): string
/// ```
pub fn hash_download(ctx: &Context, url: &str) -> Result<String, ResolveError> {
    let scratch: PathBuf = ctx.scope.toolchain_root.join("downloads").join(format!(".update-hash-{}", std::process::id()));
    let result = ctx.http.download(url, &scratch, DEFAULT_MAX_DOWNLOAD_BYTES);
    // Best effort: the file only existed to be hashed.
    let _ = fs::remove_file(&scratch);
    return match result {
        Ok(hash) => Ok(hash),
        Err(error) => Err(ResolveError::Failed(format!("cannot download {url} to hash it: {error}"))),
    };
}

/// The platforms every downloaded tool is recorded for:
/// Linux on x64 and arm64,
/// glibc and musl (the repository's Linux-only stopgap).
pub const LINUX_PLATFORMS: [&str; 4] = ["linux-arm64", "linux-arm64-musl", "linux-x64", "linux-x64-musl"];

/// Whether a platform id names an ARM 64-bit machine.
pub fn is_arm64(platform: &str) -> bool {
    return platform.contains("arm64");
}

/// Whether a platform id names a musl libc machine.
pub fn is_musl(platform: &str) -> bool {
    return platform.ends_with("-musl");
}

/// A resolution that keeps what the lock already holds.
pub fn keep(locked: &LockedTool) -> Resolution {
    return Resolution { version: locked.version.clone(), artifacts: locked.artifacts.clone() };
}

/// A resolution for a tool installed by name and version,
/// with no artifact rows.
pub fn by_name(version: &str) -> Resolution {
    return Resolution { version: version.to_string(), artifacts: Vec::new() };
}

/// What:
/// Resolves one tool from its request alone.
/// Why:
/// The lock is derived from the plan,
/// so nothing here reads a previous lock row to learn a repository,
/// a tag shape,
/// an asset or a platform list.
/// The one exception is the shortcut for an exact version:
/// when the lock already holds that very request,
/// its entry is kept without a lookup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveTool(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
/// ```
pub fn resolve_tool(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>) -> Result<Resolution, ResolveError> {
    if declaration.request.is_exact()
        && let Some(locked) = current
        && locked.declaration.request == declaration.request
    {
        return Ok(keep(locked));
    }
    let range: &str = &declaration.request.range;
    let wanted: Option<&str> = if range == "latest" { None } else { Some(range) };
    match declaration.request.origin() {
        Origin::Github(repo) => return super::update_github::resolve_github(ctx, declaration, &repo, wanted),
        Origin::Npm(name) => return super::update_registry::resolve_npm(ctx, &name, wanted),
        Origin::Cargo(name) => return super::update_registry::resolve_crate(ctx, &name, wanted),
        Origin::Pipx(name) => return super::update_registry::resolve_pypi(ctx, declaration, &name, wanted),
        Origin::Conda(name) => return super::update_android_conda::resolve_conda(ctx, &name, wanted),
        Origin::GoModule(module) => return super::update_go_zig::resolve_go_module(ctx, &module, wanted),
        Origin::Git(_) => return Ok(by_name(range)),
        Origin::Named(name) => match name.as_str() {
            "node" => return super::update_rust_node::resolve_node(ctx, wanted),
            "rustup" => return super::update_rust_node::resolve_rustup(ctx, wanted),
            "rust" => return super::update_rust_node::resolve_rust(ctx, range),
            "go" => return super::update_go_zig::resolve_go_runtime(ctx, wanted),
            "zig" => return super::update_go_zig::resolve_zig(ctx, wanted),
            "java:temurin" => return super::update_java_aws::resolve_java(ctx, range),
            "aws" => return super::update_java_aws::resolve_aws(ctx, wanted),
            "android-sdk" => return super::update_android_conda::resolve_android(ctx, wanted),
            other => return Err(ResolveError::Failed(format!("{other:?} is not an origin with a resolver"))),
        },
    }
}
