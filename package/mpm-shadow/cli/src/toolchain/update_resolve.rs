//! What:     Finding a tool's version and artifacts: the shared types and fetch helpers, and the one switch from a tool's
//!           request and route to the resolver that answers it.
//! Why:      Every npm and PyPI version, first or later, comes from `registry.npmjs.org` and `pypi.org` directly; crates
//!           come from `crates.io`, Rust channels from `static.rust-lang.org`, Node from `nodejs.org`, GitHub-hosted
//!           tools from the GitHub API. A resolver that is not written yet says so by tool and route instead of
//!           guessing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // update-resolve.ts
//! export type Resolution = { version: string; artifacts: [string, Artifact][] };
//! export type ResolveError = { kind: 'notImplemented' | 'failed'; message: string };
//! export function resolveTool(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
//! ```

use serde_json::Value;

use super::context::Context;
use super::model::{Artifact, Declaration, LockedTool, Request, Route};

/// What a resolver found: the version to lock and the artifact rows (empty for tools installed by name and version).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    /// The resolved version.
    pub version: String,
    /// Platform id to artifact, in the order they are written.
    pub artifacts: Vec<(String, Artifact)>,
}

/// What:     Why a resolver gave no answer: `NotImplemented` (no resolver exists for this tool yet) or `Failed` (the
///           lookup was made and went wrong).
/// Why:      `update` keeps the old lock entry for a not-implemented tool that already has one and reports it, but a
///           failed lookup stops the whole update before anything is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResolveError = { kind: 'notImplemented'; message: string } | { kind: 'failed'; message: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolveError {
    /// No resolver exists for this tool; the text names the tool and route.
    NotImplemented(String),
    /// The lookup failed; the text names the URL or the field.
    Failed(String),
}

impl ResolveError {
    /// The message to show.
    pub fn message(&self) -> &str {
        match self {
            ResolveError::NotImplemented(text) | ResolveError::Failed(text) => return text,
        }
    }
}

/// GETs `url` and returns the body text, or a `Failed` error naming the URL (and the status for a non-2xx answer).
pub fn get_text(ctx: &Context, url: &str, headers: &[(String, String)]) -> Result<String, ResolveError> {
    let response = match ctx.http.get(url, headers, super::http::DEFAULT_MAX_GET_BYTES) {
        Ok(answered) => answered,
        Err(error) => return Err(ResolveError::Failed(error.to_string())),
    };
    if !(200..300).contains(&response.status) {
        return Err(ResolveError::Failed(format!("{url}: HTTP {}", response.status)));
    }
    return Ok(response.text());
}

/// GETs `url` and parses the body as JSON, or a `Failed` error naming the URL.
pub fn get_json(ctx: &Context, url: &str, headers: &[(String, String)]) -> Result<Value, ResolveError> {
    let text: String = get_text(ctx, url, headers)?;
    return match serde_json::from_str(&text) {
        Ok(value) => Ok(value),
        Err(error) => Err(ResolveError::Failed(format!("{url}: the answer is not JSON: {error}"))),
    };
}

/// The string at `path` inside `value` (`["dist-tags", "latest"]`), or `None`.
pub fn text_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut cursor: &Value = value;
    for key in path {
        cursor = cursor.get(*key)?;
    }
    return cursor.as_str().filter(|text| return !text.is_empty());
}

/// The "no resolver yet" error for a tool, naming its id and route.
pub fn not_implemented(declaration: &Declaration, what: &str) -> ResolveError {
    return ResolveError::NotImplemented(format!(
        "{} (route {}): {what} is not implemented in mpm-shadow update yet",
        declaration.id,
        declaration.route.as_str()
    ));
}

/// What:     Resolves one tool.
/// Why:      The request decides the shape: a pinned request resolves exactly that version (and needs no lookup when the
///           lock already holds it), anything else asks the tool's source for the newest version or channel head.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveTool(ctx: Context, declaration: Declaration, current?: LockedTool): Resolution
/// ```
pub fn resolve_tool(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>) -> Result<Resolution, ResolveError> {
    if let Request::Pinned(pinned) = &declaration.request {
        return resolve_pinned(ctx, declaration, current, pinned);
    }
    if declaration.id == "node" {
        return super::update_rust_node::resolve_node(ctx, declaration, current);
    }
    match declaration.route {
        Route::MpmPnpm => return super::update_registry::resolve_npm(ctx, declaration),
        Route::MpmUvx => return super::update_registry::resolve_pypi(ctx, declaration, current),
        Route::CargoBinstall => return super::update_registry::resolve_crate(ctx, declaration),
        Route::RustupToolchain => return super::update_rust_node::resolve_rust(ctx, declaration),
        Route::CargoGit => {
            return match current {
                Some(locked) => Ok(Resolution { version: locked.version.clone(), artifacts: locked.artifacts.clone() }),
                None => Err(not_implemented(declaration, "resolving a cargo-git tool that is not in the lock")),
            };
        }
        Route::BinSourced | Route::Archive => {
            if declaration.id.starts_with("conda:") {
                return Err(not_implemented(declaration, "the conda channel lookup"));
            }
            return super::update_github::resolve_github(ctx, declaration, current);
        }
        Route::GoInstall => return Err(not_implemented(declaration, "the Go module proxy lookup")),
        Route::AndroidSdk => return Err(not_implemented(declaration, "the Android repository lookup")),
        Route::RustupComponents => return Err(not_implemented(declaration, "a rustup-components entry")),
    }
}

/// What:     Resolves a pinned request: the lock's own entry when it already holds that pin, otherwise the pinned
///           version with the artifacts its source gives.
/// Why:      A pin never moves by itself; only a changed pin in the plan needs resolving, and only for routes that
///           install by name and version (npm, PyPI, crates, Rust channels) or from a wheel.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolvePinned(ctx, declaration, current, pinned): Resolution
/// ```
fn resolve_pinned(ctx: &Context, declaration: &Declaration, current: Option<&LockedTool>, pinned: &str) -> Result<Resolution, ResolveError> {
    if let Some(locked) = current
        && locked.declaration.request == declaration.request
    {
        return Ok(Resolution { version: locked.version.clone(), artifacts: locked.artifacts.clone() });
    }
    match declaration.route {
        Route::MpmPnpm | Route::CargoBinstall | Route::RustupToolchain => {
            return Ok(Resolution { version: pinned.to_string(), artifacts: Vec::new() });
        }
        Route::MpmUvx => return super::update_registry::resolve_pypi_version(ctx, declaration, current, pinned),
        Route::CargoGit => return Ok(Resolution { version: pinned.to_string(), artifacts: Vec::new() }),
        Route::BinSourced | Route::Archive | Route::GoInstall | Route::AndroidSdk | Route::RustupComponents => {
            return Err(not_implemented(declaration, "resolving the artifacts of a newly pinned version"));
        }
    }
}
