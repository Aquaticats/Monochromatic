//! What:
//! The routes that install through another package manager:
//! `mpm-pnpm` (`pnpm add --global`),
//! `mpm-uvx`
//!           (`uv tool install`,
//! from a verified wheel for the `mpm` bootstrap) and `go-install` (`go install` into a
//!           scoped `GOBIN`).
//! Why:
//! Despite the `mpm-` in two route names,
//! nothing here runs mpm:
//! mpm 8.2.0 drops the pinned version for pnpm
//!           (probe 1b),
//! so each manager is called directly with the locked version.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-managers.ts
//! export function installMpmPnpm(ctx: Context, tool: LockedTool): void
//! export function installMpmUvx(ctx: Context, tool: LockedTool): void
//! export function installGoInstall(ctx: Context, tool: LockedTool): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::install_support`.
use super::install_support::{artifact_for_host, download_verified, run_checked};
/// Imports `tool_install_dir` from `super::layout`.
use super::layout::tool_install_dir;
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports `package_name` from `super::observe`.
use super::observe::package_name;
/// Imports `RunRequest` from `super::runner`.
use super::runner::RunRequest;

/// The directories added to the search for the prerequisite tool (`pnpm`,
/// `uv`,
/// `go`):
/// the scope's bin directory.
fn extra_dirs(ctx: &Context) -> Vec<PathBuf> {
    return vec![ctx.scope.toolchain_bin_dir.clone()];
}

/// The path of a prerequisite tool's program,
/// or an error saying which stage must run first.
fn prerequisite(ctx: &Context, tool_id: &str) -> Result<PathBuf, ToolchainError> {
    return match ctx.find_bin_of(tool_id, tool_id, &extra_dirs(ctx)) {
        Some(found) => Ok(found),
        None => Err(ToolchainError::failure(format!(
            "{tool_id} is not installed in the toolchain; the {tool_id} stage must run first"
        ))),
    };
}

/// `pnpm add --global [--allow-build <package>]... <package>@<locked version>`.
pub fn install_mpm_pnpm(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let pnpm: PathBuf = prerequisite(ctx, "pnpm")?;
    let mut args: Vec<String> = vec!["add".to_string(), "--global".to_string()];
    for allowed in tool.declaration.options.allow_builds.clone().unwrap_or_default() {
        args.push("--allow-build".to_string());
        args.push(allowed);
    }
    args.push(format!("{}@{}", package_name(tool), tool.version));
    let request: RunRequest = RunRequest {
        command: pnpm.to_string_lossy().to_string(),
        args,
        env: Some(ctx.path_env()),
        cwd: None,
    };
    run_checked(ctx, &request, "pnpm add --global", &tool.declaration.id)?;
    return Ok(());
}

/// What:
/// `uv tool install --force` of a verified wheel (when the plan marks the tool a wheel bootstrap) or of
///           `<package>==<locked version>`.
/// Why:
/// The wheel route is how `mpm` itself is installed from the lock's SHA-256 before anything else can verify.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installMpmUvx(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_mpm_uvx(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let uv: PathBuf = prerequisite(ctx, "uv")?;
    let uv_text: String = uv.to_string_lossy().to_string();
    let is_wheel: bool = tool.declaration.options.is_wheel();
    if is_wheel {
        let artifact = artifact_for_host(tool, &ctx.host.platform)?;
        let file_name: String = super::install_archive::safe_file_name(&artifact.url);
        let wheel_path: PathBuf = ctx.scope.toolchain_root.join("downloads").join(file_name);
        download_verified(ctx, &artifact.url, &wheel_path, &artifact.sha256)?;
        let wheel_text: String = wheel_path.to_string_lossy().to_string();
        let request: RunRequest = ctx.request(&uv_text, &["tool", "install", "--force", &wheel_text]);
        run_checked(ctx, &request, "uv tool install", &tool.declaration.id)?;
        return Ok(());
    }
    let requirement: String = format!("{}=={}", package_name(tool), tool.version);
    let request: RunRequest = ctx.request(&uv_text, &["tool", "install", "--force", &requirement]);
    run_checked(ctx, &request, "uv tool install", &tool.declaration.id)?;
    return Ok(());
}

/// `go install <module>@v<locked version>` with `GOBIN` set to a `bin` directory under the tool's install directory.
pub fn install_go_install(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let go: PathBuf = prerequisite(ctx, "go")?;
    let install_dir: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version);
    let gobin: PathBuf = install_dir.join("bin");
    if let Err(error) = fs::create_dir_all(&gobin) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", gobin.display())));
    }
    let module_version: String =
        if tool.version.starts_with('v') { tool.version.clone() } else { format!("v{}", tool.version) };
    let target: String = format!("{}@{module_version}", package_name(tool));
    let mut env = ctx.path_env();
    env.insert("GOBIN".to_string(), gobin.to_string_lossy().to_string());
    let request: RunRequest = RunRequest {
        command: go.to_string_lossy().to_string(),
        args: vec!["install".to_string(), target],
        env: Some(env),
        cwd: None,
    };
    run_checked(ctx, &request, "go install", &tool.declaration.id)?;
    return Ok(());
}
