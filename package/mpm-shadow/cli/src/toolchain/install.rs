//! What:
//! The one switch from a tool's route to the function that installs it.
//! Why:
//! `install` calls `install_tool` for any tool;
//! the match is exhaustive over `Route`,
//! so a new route cannot be
//!           added without deciding how it installs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // install.ts
//! export function installTool(ctx: Context, tool: LockedTool): void
//! ```

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports `install_archive` from `super::install_archive`.
use super::install_archive::install_archive;
/// Imports `install_bin_sourced` from `super::install_bin`.
use super::install_bin::install_bin_sourced;
/// Imports what this module uses from `super::install_cargo`.
use super::install_cargo::{install_cargo_binstall, install_cargo_git};
/// Imports what this module uses from `super::install_managers`.
use super::install_managers::{install_go_install, install_mpm_pnpm, install_mpm_uvx};
/// Imports what this module uses from `super::install_rust_android`.
use super::install_rust_android::{install_android_sdk, install_rust_components, install_rust_toolchain};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Route};

/// What:
/// Installs one tool by its route,
/// then (for a Rust toolchain) its components and targets.
/// Why:
/// The components step follows every route that carries `rustComponents` or `rustTargets`,
/// as the
///           TypeScript sync did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function installTool(ctx: Context, tool: LockedTool): void
/// ```
pub fn install_tool(ctx: &Context, tool: &LockedTool) -> Result<(), ToolchainError> {
    match tool.declaration.route {
        Route::BinSourced => install_bin_sourced(ctx, tool)?,
        Route::Archive => install_archive(ctx, tool)?,
        Route::CargoBinstall => install_cargo_binstall(ctx, tool)?,
        Route::CargoGit => install_cargo_git(ctx, tool)?,
        Route::MpmPnpm => install_mpm_pnpm(ctx, tool)?,
        Route::MpmUvx => install_mpm_uvx(ctx, tool)?,
        Route::GoInstall => install_go_install(ctx, tool)?,
        Route::RustupToolchain => install_rust_toolchain(ctx, tool)?,
        Route::RustupComponents => {}
        Route::AndroidSdk => install_android_sdk(ctx, tool)?,
    }
    let options = &tool.declaration.options;
    if options.rust_components.is_some() || options.rust_targets.is_some() {
        install_rust_components(ctx, tool)?;
    }
    return Ok(());
}
