//! What:
//! The `rustup-toolchain` route (`rustup toolchain install --no-self-update <channel>`,
//! then components and
//!           targets) and the `android-sdk` route (command-line tools from a verified zip,
//! then `sdkmanager` packages).
//! Why:
//! Rust channels install directly from the lock:
//! mpm honours a rustup version only because the package id is
//!           the channel,
//! and the prototype showed the direct call produces the same toolchain list.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install-rust-android.ts
//! export function installRustToolchain(ctx: Context, tool: LockedTool): void
//! export function installRustComponents(ctx: Context, tool: LockedTool): void
//! export function installAndroidSdk(ctx: Context, tool: LockedTool): void
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `safe_file_name` from `super::install_archive`.
use super::install_archive::safe_file_name;
/// Imports what this module uses from `super::install_support`.
use super::install_support::{artifact_for_host, download_verified, extract_archive, run_checked};
/// Imports `tool_install_dir` from `super::layout`.
use super::layout::tool_install_dir;
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports `RunRequest` from `super::runner`.
use super::runner::RunRequest;

/// `rustup toolchain install --no-self-update <package id, or the locked version>`.
pub fn install_rust_toolchain(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let channel: String = match &tool.declaration.package_id {
        Some(package_id) => package_id.clone(),
        None => tool.version.clone(),
    };
    let request: RunRequest = ctx.request("rustup", &["toolchain", "install", "--no-self-update", &channel]);
    run_checked(ctx, &request, "rustup toolchain install", &tool.declaration.id)?;
    return Ok(());
}

/// What:
/// `rustup component add <name> --toolchain <version>` for each declared component,
/// then `rustup target add
///           <triple> --toolchain <version>` for each declared target.
/// Why:
/// The toolchain install above brings the default profile;
/// the plan names the extras.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installRustComponents(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_rust_components(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    for component in tool.declaration.options.rust_components.clone().unwrap_or_default() {
        let request: RunRequest = ctx.request("rustup", &["component", "add", &component, "--toolchain", &tool.version]);
        run_checked(ctx, &request, &format!("rustup component add ({component})"), &tool.declaration.id)?;
    }
    for target in tool.declaration.options.rust_targets.clone().unwrap_or_default() {
        let request: RunRequest = ctx.request("rustup", &["target", "add", &target, "--toolchain", &tool.version]);
        run_checked(ctx, &request, &format!("rustup target add ({target})"), &tool.declaration.id)?;
    }
    return Ok(());
}

/// What:
/// Installs the Android command-line tools from the locked zip into `<install dir>/cmdline-tools/<version>`
///           and runs `android sdk install <packages>` with `ANDROID_HOME` and `ANDROID_SDK_ROOT` set.
/// Why:
/// The zip unpacks a top-level `cmdline-tools` directory;
/// `sdkmanager` insists on finding itself at
///           `cmdline-tools/<version>/bin`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installAndroidSdk(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_android_sdk(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    let artifact = artifact_for_host(tool, &ctx.host.platform)?;
    let download_path: PathBuf = ctx
        .scope
        .toolchain_root
        .join("downloads")
        .join(format!("android-sdk-{}-{}", tool.version, safe_file_name(&artifact.url)));
    download_verified(ctx, &artifact.url, &download_path, &artifact.sha256)?;
    let sdk_root: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version);
    let staging: PathBuf = sdk_root.join(".cmdline-tools-staging");
    let destination: PathBuf = sdk_root.join("cmdline-tools").join(&tool.version);
    // Best effort: a staging directory left by a killed run is cleared; absence is the normal case.
    let _ = fs::remove_dir_all(&staging);
    extract_archive(ctx, &download_path, &staging)?;
    if let Err(error) = fs::create_dir_all(sdk_root.join("cmdline-tools")) {
        return Err(ToolchainError::failure(format!("cannot create {}: {error}", sdk_root.join("cmdline-tools").display())));
    }
    let _ = fs::remove_dir_all(&destination);
    if let Err(error) = fs::rename(staging.join("cmdline-tools"), &destination) {
        return Err(ToolchainError::failure(format!(
            "the Android command-line tools archive did not hold a cmdline-tools directory to move into {}: {error}",
            destination.display()
        )));
    }
    let _ = fs::remove_dir_all(&staging);
    let android_cli: PathBuf = destination.join("bin").join("android");
    let mut args: Vec<String> = vec!["sdk".to_string(), "install".to_string()];
    args.extend(tool.declaration.options.android_packages.clone().unwrap_or_default());
    let mut env = ctx.path_env();
    env.insert("ANDROID_HOME".to_string(), sdk_root.to_string_lossy().to_string());
    env.insert("ANDROID_SDK_ROOT".to_string(), sdk_root.to_string_lossy().to_string());
    let request: RunRequest = RunRequest {
        command: android_cli.to_string_lossy().to_string(),
        args,
        env: Some(env),
        cwd: None,
    };
    run_checked(ctx, &request, "android sdk install", &tool.declaration.id)?;
    return Ok(());
}
