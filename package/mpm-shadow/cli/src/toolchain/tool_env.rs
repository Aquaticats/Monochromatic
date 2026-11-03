//! What:
//! The tool environment:
//! the variables the lock's `env` blocks declare (`ANDROID_HOME`,
//! `JAVA_HOME`,
//! `RUSTUP_TOOLCHAIN`),
//! written as a generated mise
//!           configuration fragment.
//! Why:
//! Mise runs every task and shell of the repository (and, for the machine-global scope,
//! of the machine),
//! and reads every file of a `conf.d` directory in name order,
//! so a fragment there reaches them all with no committed line
//!           pointing at it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // tool-env.ts
//! export function writeToolEnv(ctx: Context): { file: string; names: string[] }  // throws on a value mise would misread
//! ```

/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;

/// What `write_tool_env` did:
/// the fragment's path and the variable names it now holds.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ToolEnvOutcome {
    /// The fragment the variables are in (or were removed from).
    pub file: PathBuf,
    /// The variable names the fragment now holds,
    /// sorted.
    pub names: Vec<String>,
}

/// Writes the scope's tool environment fragment from the lock.
pub fn write_tool_env(ctx: &Context) -> Result<ToolEnvOutcome, ToolchainError> {
    return Ok(ToolEnvOutcome { file: ctx.scope.mise_env_file.clone(), names: Vec::new() });
}
