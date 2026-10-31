//! What:
//! The two cargo routes:
//! `cargo-binstall` (`cargo binstall --no-confirm --version <locked> <crate>`) and
//!           `cargo-git` (`cargo install --git <url> --rev <rev> --root <install dir> <package>`).
//! Why:
//! The version always comes from the lock,
//! so a crate is installed at exactly the locked version and never
//!           through mpm,
//! which drops pinned versions for cargo.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-cargo.ts
//! export function installCargoBinstall(ctx: Context, tool: LockedTool): void
//! export function installCargoGit(ctx: Context, tool: LockedTool): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `run_checked` from `super::install_support`.
use super::install_support::run_checked;
/// Imports `tool_install_dir` from `super::layout`.
use super::layout::tool_install_dir;
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Request};
/// Imports `package_name` from `super::observe`.
use super::observe::package_name;
/// Imports `RunRequest` from `super::runner`.
use super::runner::RunRequest;

/// Installs one crate with `cargo binstall`,
/// passing the GitHub token when one is available.
pub fn install_cargo_binstall(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let mut args: Vec<String> = vec!["binstall".to_string(), "--no-confirm".to_string(), "--version".to_string(), tool.version.clone()];
    for feature in tool.declaration.options.cargo_features.clone().unwrap_or_default() {
        args.push(format!("--features={feature}"));
    }
    args.push(package_name(tool));
    let mut env = ctx.path_env();
    if let Some(token) = ctx.github_token() {
        env.insert("GITHUB_TOKEN".to_string(), token);
    }
    let request: RunRequest = RunRequest { command: "cargo".to_string(), args, env: Some(env), cwd: None };
    run_checked(ctx, &request, "cargo binstall", &tool.declaration.id)?;
    return Ok(());
}

/// What:
/// Installs one crate from a git source at a pinned revision into the tool's own install directory.
/// Why:
/// The revision rides in the pinned request as `ref:<rev>` (or bare);
/// without a pinned request the locked
///           version is used as the revision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installCargoGit(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_cargo_git(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let Some(url) = &tool.declaration.options.cargo_git_url else {
        return Err(ToolchainError::failure(format!("toolchain plan entry {} carries no cargoGit source url", tool.declaration.id)));
    };
    let pinned: String = match &tool.declaration.request {
        Request::Pinned(version) => version.clone(),
        _ => tool.version.clone(),
    };
    let revision: String = pinned.strip_prefix("ref:").unwrap_or(&pinned).to_string();
    let install_dir: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version);
    if let Err(error) = fs::create_dir_all(&install_dir) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", install_dir.display())));
    }
    let install_text: String = install_dir.to_string_lossy().to_string();
    let package: String = package_name(tool);
    let request: RunRequest = ctx.request("cargo", &["install", "--git", url, "--rev", &revision, "--root", &install_text, &package]);
    run_checked(ctx, &request, "cargo install --git", &tool.declaration.id)?;
    return Ok(());
}
