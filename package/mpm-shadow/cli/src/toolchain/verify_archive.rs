//! What:
//! The `verify-archive` command:
//! hash a file that was downloaded outside mpm-shadow (a prebuilt `mpm-shadow` archive,
//! a new `uv`)
//! and compare it with the SHA-256 the lock records for a tool on a platform.
//! Why:
//! A fresh machine or a container fetches one prebuilt archive with `curl` before any mpm-shadow can run `sync`.
//! This is the Rust-side check of that file against the lock,
//! so the bootstrap never runs an archive the lock does not vouch for.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // verify-archive.ts
//! export function runVerifyArchive(scope: Scope, platform: string, toolId: string, file: string): string  // throws on mismatch
//! ```

/// Imports `Path`.
use std::path::Path;

/// Imports what this module uses from `super::agree`.
use super::agree::{compare, refusal};
/// Imports what this module uses from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::hash`.
use super::hash::sha256_file;
/// Imports what this module uses from `super::load`.
use super::load::{Loaded, load};
/// Imports what this module uses from `super::model`.
use super::model::{Artifact, LockedTool};
/// Imports what this module uses from `super::scope`.
use super::scope::Scope;

/// What:
/// Checks `file` against the lock's artifact row of `tool_id` for `platform`.
/// Why:
/// The lock's digest must verify first,
/// so a hand-edited hash cannot vouch for a file;
/// plan and lock disagreement is not a reason to refuse,
/// because the bootstrap runs before anyone has edited a plan.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function runVerifyArchive(scope: Scope, platform: string, toolId: string, file: string): string
/// ```
pub fn run_verify_archive(scope: &Scope, platform: &str, tool_id: &str, file: &Path) -> Result<String, ToolchainError> {
    let loaded: Loaded = load(scope)?;
    let Some(parsed) = &loaded.lock else {
        return Err(ToolchainError::drift(format!(
            "{} does not exist, so there is no recorded SHA-256 to check {} against",
            scope.lock_file.display(),
            file.display()
        )));
    };
    let verdict = compare(&loaded.plan, parsed);
    if verdict.digest_problem.is_some() {
        return Err(refusal(&verdict));
    }
    let Some(tool): Option<&LockedTool> = parsed.lock.tool(tool_id) else {
        return Err(ToolchainError::config(format!("the lock has no tool {tool_id:?}")));
    };
    let mut found: Option<&Artifact> = None;
    for (row_platform, row) in &tool.artifacts {
        if row_platform == platform {
            found = Some(row);
            break;
        }
    }
    let Some(artifact) = found else {
        let mut known: Vec<&str> = Vec::new();
        for (row_platform, _) in &tool.artifacts {
            known.push(row_platform.as_str());
        }
        return Err(ToolchainError::config(format!(
            "the lock has no artifact for {tool_id} on {platform}; it records: {}",
            if known.is_empty() { "none".to_string() } else { known.join(", ") }
        )));
    };
    let observed: String = match sha256_file(file) {
        Ok(hash) => hash,
        Err(message) => return Err(ToolchainError::failure(message)),
    };
    if observed != artifact.sha256 {
        return Err(ToolchainError::failure(format!(
            "SHA-256 mismatch for {}: the lock expects {} for {tool_id} on {platform} ({}), the file is {observed}; do not run it",
            file.display(),
            artifact.sha256,
            artifact.url
        )));
    }
    return Ok(format!("verified {}: SHA-256 {observed} matches the lock for {tool_id} {} on {platform}", file.display(), tool.version));
}
