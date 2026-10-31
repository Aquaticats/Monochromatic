//! What:
//! What is installed right now,
//! per route:
//! one observer for each way a tool gets installed,
//! each returning
//!           the installed version or `None`,
//! and `status`'s report built from them.
//! Why:
//! `status` compares the observation with the lock;
//! `sync` uses the same observation to skip tools that
//!           already match and `update` uses it to find npm and PyPI tools that are not installed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // observe.ts
//! export function observeTool(ctx: Context, tool: LockedTool): string | undefined
//! export function observeAll(ctx: Context, tools: LockedTool[]): Observation[]
//! ```

/// Imports `Value` from `serde_json`.
use serde_json::Value;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports what this module uses from `super::layout`.
use super::layout::{applies_to_family, tool_install_dir};
/// Imports what this module uses from `super::model`.
use super::model::{LockedTool, Route};
/// Imports what this module uses from `super::parse_output`.
use super::parse_output::{
    global_pnpm_versions, parse_bin_list, parse_cargo_binstall_list, parse_uv_tool_list, version_matches,
};
/// Imports what this module uses from `super::runner`.
use super::runner::{RunOutcome, RunRequest};

/// The first word of a line (up to the first whitespace),
/// or the empty text.
fn first_word(line: &str) -> &str {
    return line.split_whitespace().next().unwrap_or("");
}

/// What:
/// The package key under `uv tool list`:
/// the package id (or tool id) with `-` turned into `_`.
/// Why:
/// `parse_uv_tool_list` keys its map the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const key = (tool.packageId ?? tool.id).replaceAll('-', '_');
/// ```
fn uv_key(tool: &LockedTool) -> String {
    return tool.declaration.package_id.clone().unwrap_or_else(|| return tool.declaration.id.clone()).replace('-', "_");
}

/// The package or crate name a manager route installs:
/// the package id,
/// or the tool id when none is declared.
pub fn package_name(tool: &LockedTool) -> String {
    return tool.declaration.package_id.clone().unwrap_or_else(|| return tool.declaration.id.clone());
}

/// `bin list`,
/// for the bin-sourced route.
fn observe_bin_sourced(ctx: &Context, tool: &LockedTool) -> Option<String> {
    let cli: PathBuf = ctx.find_bin_of("bin", "bin", &[])?;
    let mut env = ctx.path_env();
    env.insert("BIN_CONFIG".to_string(), ctx.scope.bin_config_file.to_string_lossy().to_string());
    let outcome: RunOutcome = ctx.run(&RunRequest::new(&cli.to_string_lossy(), &["list"]).with_env(env));
    let versions = parse_bin_list(&format!("{}{}", outcome.stdout, outcome.stderr));
    for name in &tool.declaration.bins {
        if let Some(version) = versions.get(name) {
            return Some(version.clone());
        }
    }
    return None;
}

/// `cargo binstall --list`,
/// for the cargo-binstall route.
fn observe_cargo_binstall(ctx: &Context, tool: &LockedTool) -> Option<String> {
    let outcome: RunOutcome = ctx.run(&ctx.request("cargo", &["binstall", "--list"]));
    return parse_cargo_binstall_list(&outcome.stdout).get(&package_name(tool)).cloned();
}

/// `pnpm list --global --depth=0 --json`,
/// for the mpm-pnpm route.
fn observe_mpm_pnpm(ctx: &Context, tool: &LockedTool) -> Option<String> {
    let pnpm: PathBuf = ctx.find_bin_of("pnpm", "pnpm", &[])?;
    let outcome: RunOutcome = ctx.run(&ctx.request(&pnpm.to_string_lossy(), &["list", "--global", "--depth=0", "--json"]));
    let parsed: Value = match serde_json::from_str(&outcome.stdout) {
        Ok(value) => value,
        // Not JSON: pnpm failed or printed something else, so the tool is read as absent.
        Err(_) => return None,
    };
    return global_pnpm_versions(&parsed).get(&package_name(tool)).cloned();
}

/// `uv tool list`,
/// for the mpm-uvx route.
fn observe_mpm_uvx(ctx: &Context, tool: &LockedTool) -> Option<String> {
    let uv: PathBuf = ctx.find_bin_of("uv", "uv", &[])?;
    let outcome: RunOutcome = ctx.run(&ctx.request(&uv.to_string_lossy(), &["tool", "list"]));
    return parse_uv_tool_list(&outcome.stdout).get(&uv_key(tool)).cloned();
}

/// `rustup toolchain list`,
/// for the rustup-toolchain route:
/// the listed toolchain whose name starts with the version.
fn observe_rustup_toolchain(ctx: &Context, tool: &LockedTool) -> Option<String> {
    let outcome: RunOutcome = ctx.run(&ctx.request("rustup", &["toolchain", "list"]));
    for line in outcome.stdout.lines() {
        let name: &str = first_word(line);
        if !name.is_empty() && name.starts_with(&tool.version) {
            return Some(name.to_string());
        }
    }
    return None;
}

/// The Android command-line tools directory for the locked version,
/// for the android-sdk route.
fn observe_android_sdk(ctx: &Context, tool: &LockedTool) -> Option<String> {
    let install: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, &tool.version);
    if install.join("cmdline-tools").join(&tool.version).exists() {
        return Some(tool.version.clone());
    }
    return None;
}

/// The tool's own install directory,
/// for routes that leave no record elsewhere (archive,
/// cargo-git,
/// go-install):
/// the locked version when its directory exists,
/// otherwise the first other version directory,
/// otherwise `None`.
fn observe_version_directory(ctx: &Context, tool: &LockedTool) -> Option<String> {
    let tool_root: PathBuf = tool_install_dir(&ctx.scope.toolchain_root, &tool.declaration.id, "");
    let entries = fs::read_dir(&tool_root).ok()?;
    let mut names: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() && !name.starts_with('.') {
            names.push(name);
        }
    }
    names.sort();
    if names.contains(&tool.version) {
        return Some(tool.version.clone());
    }
    return names.into_iter().next();
}

/// What:
/// The installed version of one tool,
/// by its route,
/// or `None` when it is not installed.
/// Why:
/// Each route leaves its record somewhere different;
/// this is the one switch over them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function observeTool(ctx: Context, tool: LockedTool): string | undefined
/// ```
pub fn observe_tool(ctx: &Context, tool: &LockedTool) -> Option<String> {
    match tool.declaration.route {
        Route::BinSourced => return observe_bin_sourced(ctx, tool),
        Route::CargoBinstall => return observe_cargo_binstall(ctx, tool),
        Route::MpmPnpm => return observe_mpm_pnpm(ctx, tool),
        Route::MpmUvx => return observe_mpm_uvx(ctx, tool),
        Route::RustupToolchain => return observe_rustup_toolchain(ctx, tool),
        Route::AndroidSdk => return observe_android_sdk(ctx, tool),
        Route::Archive | Route::CargoGit | Route::GoInstall | Route::RustupComponents => {
            return observe_version_directory(ctx, tool);
        }
    }
}

/// One line of the status report:
/// the tool,
/// the locked version,
/// the installed version (if any) and whether they match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observation {
    /// Tool id.
    pub id: String,
    /// The version the lock pins.
    pub locked: String,
    /// The installed version,
    /// or `None` when the tool is not installed.
    pub observed: Option<String>,
    /// Whether the installed version matches the locked one.
    pub in_sync: bool,
}

/// What:
/// Observes every tool that applies to this host,
/// in lock order.
/// Why:
/// A tool limited to another OS family would otherwise report permanent drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function observeAll(ctx: Context, tools: LockedTool[]): Observation[]
/// ```
pub fn observe_all(ctx: &Context, tools: &[LockedTool]) -> Vec<Observation> {
    let mut observations: Vec<Observation> = Vec::new();
    for tool in tools {
        if !applies_to_family(&tool.declaration.os, ctx.host.family) {
            continue;
        }
        let observed: Option<String> = observe_tool(ctx, tool);
        let in_sync: bool = observed.as_deref().is_some_and(|version| return version_matches(version, &tool.version));
        observations.push(Observation { id: tool.declaration.id.clone(), locked: tool.version.clone(), observed, in_sync });
    }
    return observations;
}

/// What:
/// The text `status` prints:
/// one line per tool (`ok` or `drift`) and a summary line.
/// Why:
/// Same wording as the TypeScript command,
/// so scripts reading it keep working.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function formatStatus(observations: Observation[]): string
/// ```
pub fn format_status(observations: &[Observation]) -> String {
    let mut text: String = String::new();
    for observation in observations {
        if observation.in_sync {
            text.push_str(&format!("ok      {}  {}\n", observation.id, observation.locked));
        } else {
            text.push_str(&format!(
                "drift   {}  locked={}  observed={}\n",
                observation.id,
                observation.locked,
                observation.observed.as_deref().unwrap_or("(absent)")
            ));
        }
    }
    let drifted: usize = observations.iter().filter(|observation| return !observation.in_sync).count();
    if drifted == 0 {
        text.push_str(&format!("toolchain status: {} tools in sync with toolchain.lock.json\n", observations.len()));
    } else {
        text.push_str(&format!("toolchain status: {drifted} of {} tools drifted from toolchain.lock.json\n", observations.len()));
    }
    return text;
}
