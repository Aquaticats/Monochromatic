//! What:
//! `mpm-shadow bin`
//! (the command that replaced `bin-paths`):
//! prints,
//! as JSON,
//! the directories that hold the toolchain's programs and the
//!           exposed link of every declared binary that currently resolves.
//! Why:
//! The IDE launcher runs it in a sandbox (no network,
//! project read-only,
//! cleared environment) to find the
//!           tools;
//! it reads only the plan and the file system,
//! never the lock,
//! the network or a package manager.
//!
//! The output shape is the one `package/ide/app/src/language/toolchain/parse.rs` pins:
//! `exposureDir` (relative to the
//! worktree),
//! `pathDirs` (absolute,
//! existing directories) and `bins` (name to absolute link path).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bin-paths.ts
//! export function binPathsOutput(plan: Plan, scope: Scope, env: EnvMap): { exposureDir: string; pathDirs: string[]; bins: Record<string, string> }
//! ```

/// Imports what this module uses from `serde_json`.
use serde_json::{Map, Value};
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `manager_native_bin_dirs` from `super::layout`.
use super::layout::manager_native_bin_dirs;
/// Imports the one pretty-JSON renderer the lock and `bin` share.
use super::model_encode::render_pretty_json;
/// Imports `Plan` from `super::plan`.
use super::plan::Plan;
/// Imports what this module uses from `super::scope`.
use super::scope::{EnvMap, Scope};

/// What:
/// The `bin --json` document shape:
/// each key,
/// in render order,
/// with the value it carries
/// (`exposureDir` relative to the worktree,
/// `pathDirs` the absolute existing program directories,
/// `bins` the name-to-absolute-link map).
/// Why:
/// The writer builds the document from this one declaration,
/// and the test reads the same declaration,
/// so a fourth key cannot be written without the guard covering it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function documentFields(exposureDir: string, pathDirs: string[], bins: Record<string, string>): [string, unknown][]
/// ```
pub(super) fn document_fields(
    exposure_dir: &str,
    path_dirs: Vec<Value>,
    bins: Map<String, Value>,
) -> Vec<(&'static str, Value)> {
    return vec![
        ("exposureDir", Value::String(exposure_dir.to_string())),
        ("pathDirs", Value::Array(path_dirs)),
        ("bins", Value::Object(bins)),
    ];
}

/// What:
/// Builds the `bin` document.
/// Why:
/// A binary name declared twice is refused (the link farm could hold only one of them);
/// a declared name
///           appears in `bins` only when its link exists,
/// and a directory appears in `pathDirs` only when it exists.
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
    for (key, value) in document_fields(&scope.exposure_dir_relative, path_dirs, bins) {
        document.insert(key.to_string(), value);
    }
    return Ok(Value::Object(document));
}

/// The document as pretty JSON with two-space indentation and a trailing newline,
/// as the TypeScript command printed it.
pub fn render_bin_paths(document: &Value) -> String {
    let mut text: String = render_pretty_json(document);
    text.push('\n');
    return text;
}
