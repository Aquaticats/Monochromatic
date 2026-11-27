//! What:
//! What is installed right now,
//! per route:
//! one observer for each way a tool gets installed,
//! each returning
//!           the installed version or `None`,
//! and `list`'s report built from them.
//! Why:
//! `list` compares the observation with the lock;
//! `install` uses the same observation to skip tools that
//!           already match and `update` uses it to find npm and PyPI tools that are not installed.
//! The observers are files of their own,
//! named for the record each route leaves:
//! what a program prints (`what_a_program_prints`)
//! and what the file system holds (`what_the_file_system_holds`).
//! This file keeps the name a row installs as,
//! the switch over the routes,
//! the comparison with the lock and the report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // observe.ts
//! export function packageName(tool: LockedTool): string
//! export function observeTool(ctx: Context, tool: LockedTool): string | undefined
//! export function observeAll(ctx: Context, tools: LockedTool[]): Observation[]
//! export * from './observe/what_a_program_prints.ts';
//! export * from './observe/what_the_file_system_holds.ts';
//! ```

/// Imports `winning_version` from `super::across_roots`.
use super::across_roots::winning_version;
/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::is_repository_published;
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `applies_to_family` from `super::layout`.
use super::layout::applies_to_family;
/// Imports what this module uses from `super::model`.
use super::model::{
    Declaration,
    LockedTool,
    Origin,
    Route,
};
/// Imports `version_matches` from `super::parse_output`.
use super::parse_output::version_matches;
/// Imports `is_published_archive` from `super::published_archive`.
use super::published_archive::is_published_archive;
/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;
/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;

/// What a program prints about the tools it installed.
mod what_a_program_prints;
/// What the file system holds for the routes that leave a record there.
mod what_the_file_system_holds;

/// Imports the observers that read a program's output.
use self::what_a_program_prints::{
    observe_bin_sourced,
    observe_mpm_pnpm,
    observe_mpm_uvx,
    observe_rustup_toolchain,
};
/// Imports the observers that read a file the installer left.
use self::what_the_file_system_holds::{
    observe_android_sdk,
    observe_cargo_binstall,
    observe_go_install,
    observe_version_directory,
};

/// What:
/// The package or crate name a manager route installs:
/// the declared package id,
/// else the name in the request's origin (`npm:wrangler` gives `wrangler`),
/// else the tool id.
/// Why:
/// The request names the package once,
/// so a plan entry need not repeat it in `packageId`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function packageName(tool: LockedTool): string
/// ```
pub fn package_name(tool: &LockedTool) -> String {
    return declared_package_name(&tool.declaration);
}

/// The package a declaration installs,
/// from its declaration alone (the plan decoder asks this before there is a lock entry):
/// `packageId`,
/// else the name in the request,
/// else the tool id.
pub fn declared_package_name(declaration: &Declaration) -> String {
    if let Some(package_id) = &declaration.package_id {
        return package_id.clone();
    }
    return match declaration.request.origin() {
        Origin::Npm(name)
        | Origin::Cargo(name)
        | Origin::Pipx(name)
        | Origin::Conda(name)
        | Origin::GoModule(name) => name,
        _ => declaration.id.clone(),
    };
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
pub fn observe_tool(
    ctx: &Context,
    tool: &LockedTool,
) -> Option<String> {
    match tool.declaration.route {
        Route::BinSourced => return observe_bin_sourced(ctx, tool),
        Route::CargoBinstall => return observe_cargo_binstall(ctx, tool),
        Route::MpmPnpm => return observe_mpm_pnpm(ctx, tool),
        Route::MpmUvx => return observe_mpm_uvx(ctx, tool),
        Route::GoInstall => return observe_go_install(ctx, tool),
        Route::RustupToolchain => return observe_rustup_toolchain(ctx, tool),
        Route::AndroidSdk => return observe_android_sdk(ctx, tool),
        // A release archive of this repository: the newest finished version, as for a `monochromatic-` crate.
        Route::Archive if is_published_archive(tool) => {
            return winning_version(ctx.scope, tool);
        }
        Route::Archive | Route::CargoGit | Route::RustupComponents => {
            return observe_version_directory(ctx, tool);
        }
    }
}

/// What:
/// The text an observation of `tool` is compared against:
/// the rustup channel a `rustup-toolchain` row names,
/// else the locked version.
/// Why:
/// `rustup` reports its toolchains by channel plus host triple,
/// while every other route reports the version the lock pins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function comparedWithLock(tool: LockedTool): string
/// ```
pub fn compared_with_lock(tool: &LockedTool) -> &str {
    return match tool.declaration.route {
        Route::RustupToolchain => tool.rustup_channel(),
        _ => &tool.version,
    };
}

/// What:
/// Whether an installed version satisfies the lock:
/// equal for most tools,
/// equal or newer for a repository-published tool.
/// Why:
/// Older repository-published tools are never kept,
/// so a tree whose lock pins an older one runs the newer install and that is not drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function satisfiesLock(tool: LockedTool, observed: string): boolean
/// ```
pub fn satisfies_lock(
    tool: &LockedTool,
    observed: &str,
) -> bool {
    if is_repository_published(tool) {
        return compare_versions(observed, &tool.version) != Ordering::Less;
    }
    return version_matches(observed, compared_with_lock(tool));
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
pub fn observe_all(
    ctx: &Context,
    tools: &[LockedTool],
) -> Vec<Observation> {
    let mut observations: Vec<Observation> = Vec::new();
    for tool in tools {
        if !applies_to_family(&tool.declaration.os, ctx.host.family) {
            continue;
        }
        let observed: Option<String> = observe_tool(ctx, tool);
        let in_sync: bool = match observed.as_deref() {
            Some(version) => satisfies_lock(tool, version),
            None => false,
        };
        observations.push(Observation {
            id: tool.declaration.id.clone(),
            locked: tool.version.clone(),
            observed,
            in_sync,
        });
    }
    return observations;
}

/// What:
/// The text `list` prints:
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
            text.push_str(&format!(
                "ok      {}  {}\n",
                observation.id, observation.locked
            ));
        } else {
            text.push_str(&format!(
                "drift   {}  locked={}  observed={}\n",
                observation.id,
                observation.locked,
                observation.observed.as_deref().unwrap_or("(absent)")
            ));
        }
    }
    let mut drifted: usize = 0;
    for observation in observations {
        if !observation.in_sync {
            drifted += 1;
        }
    }
    if drifted == 0 {
        text.push_str(&format!(
            "toolchain list: {} tools in sync with toolchain.lock.json\n",
            observations.len()
        ));
    } else {
        text.push_str(&format!(
            "toolchain list: {drifted} of {} tools drifted from toolchain.lock.json\n",
            observations.len()
        ));
    }
    return text;
}
