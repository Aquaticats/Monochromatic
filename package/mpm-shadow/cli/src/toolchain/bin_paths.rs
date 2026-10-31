//! What:     `mpm-shadow bin-paths`: prints, as JSON, the directories that hold the toolchain's programs and the
//!           exposed link of every declared binary that currently resolves.
//! Why:      The IDE launcher runs it in a sandbox (no network, project read-only, cleared environment) to find the
//!           tools; it reads only the plan and the file system, never the lock, the network or a package manager.
//!
//! The output shape is the one `package/ide/app/src/language/toolchain/parse.rs` pins: `exposureDir` (relative to the
//! worktree), `pathDirs` (absolute, existing directories) and `bins` (name to absolute link path).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bin-paths.ts
//! export function binPathsOutput(plan: Plan, scope: Scope, env: EnvMap): { exposureDir: string; pathDirs: string[]; bins: Record<string, string> }
//! ```

use serde_json::{Map, Value};
use std::path::PathBuf;

use super::error::ToolchainError;
use super::layout::manager_native_bin_dirs;
use super::plan::Plan;
use super::scope::{EnvMap, Scope};

/// What:     Builds the `bin-paths` document.
/// Why:      A binary name declared twice is refused (the link farm could hold only one of them); a declared name
///           appears in `bins` only when its link exists, and a directory appears in `pathDirs` only when it exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function binPathsOutput(plan, scope, env): BinPathsOutput  // throws on a duplicate bin name
/// ```
pub fn bin_paths_output(plan: &Plan, scope: &Scope, env: &EnvMap) -> Result<Value, ToolchainError> {
    let mut names: Vec<&str> = Vec::new();
    for tool in &plan.tools {
        for name in &tool.bins {
            if names.contains(&name.as_str()) {
                return Err(ToolchainError::config(format!(
                    "the toolchain plan declares the binary name {name:?} more than once"
                )));
            }
            names.push(name.as_str());
        }
    }
    let mut path_dirs: Vec<Value> = Vec::new();
    for directory in manager_native_bin_dirs(scope, env) {
        if directory.exists() {
            path_dirs.push(Value::String(directory.to_string_lossy().to_string()));
        }
    }
    let mut bins: Map<String, Value> = Map::new();
    for name in names {
        let link: PathBuf = scope.exposure_dir.join(name);
        if link.exists() {
            bins.insert(name.to_string(), Value::String(link.to_string_lossy().to_string()));
        }
    }
    let mut document: Map<String, Value> = Map::new();
    document.insert("exposureDir".to_string(), Value::String(scope.exposure_dir_relative.clone()));
    document.insert("pathDirs".to_string(), Value::Array(path_dirs));
    document.insert("bins".to_string(), Value::Object(bins));
    return Ok(Value::Object(document));
}

/// The document as pretty JSON with two-space indentation and a trailing newline, as the TypeScript command printed it.
pub fn render_bin_paths(document: &Value) -> String {
    let mut text: String = match serde_json::to_string_pretty(document) {
        Ok(pretty) => pretty,
        Err(error) => format!("{{\"error\": \"{error}\"}}"),
    };
    text.push('\n');
    return text;
}
