//! What:
//! The install itself:
//! every selected tool of the lock,
//! in installation order,
//! with the components and targets a Rust toolchain that is already at its version still gets,
//! and then the postinstall links,
//! the exposure directory,
//! the tool environment and the postinstall commands.
//! Why:
//! This is the one place a tool is installed,
//! so the probe before installing,
//! the warning for a replaced version,
//! and the report that names every failure live together.
//! A caller imports from `sync` and never from this file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function syncTools(ctx: Context, only?: string[]): SyncReport;
//! ```

/// Imports the root that holds the winning copy of a repository-published tool.
use super::super::across_roots::winning_root;
/// Imports the checks a repository-published tool's link is settled with,
/// and its older-lock warnings.
use super::super::cargo_installs::{
    is_repository_published,
    older_lock_warnings,
    point_current_at_newest_of,
};
/// Imports the bundle of seams the installers receive.
use super::super::context::Context;
/// Imports `ToolchainError` from `super::super::error`.
use super::super::error::ToolchainError;
/// Imports the exposure directory,
/// the postinstall commands and the bin check that follow the installs.
use super::super::exposure::{
    rebuild_exposure,
    run_postinstall_commands,
    unresolvable_bins,
};
/// Imports `install_tool` from `super::super::install`.
use super::super::install::install_tool;
/// Imports `reconcile_rust_parts` from `super::super::install_rust_android`.
use super::super::install_rust_android::reconcile_rust_parts;
/// Imports `applies_to_family` from `super::super::layout`.
use super::super::layout::applies_to_family;
/// Imports the locked tool the sync installs,
/// and the route that is never replaced by an older version.
use super::super::model::{
    LockedTool,
    Route,
};
/// Imports the probe of what is installed now.
use super::super::observe::{
    compared_with_lock,
    observe_tool,
    satisfies_lock,
};
/// Imports `sync_postinstall_links` from `super::super::postinstall_links`.
use super::super::postinstall_links::sync_postinstall_links;
/// Imports `write_tool_env` from `super::super::tool_env`.
use super::super::tool_env::write_tool_env;

/// Imports the order the tools are installed in.
use super::the_install_order::sync_order;
/// Imports the report a run answers with,
/// and the warning for a replaced version.
use super::what_the_sync_did::{
    SyncReport,
    replacement_warning,
};
/// What:
/// Syncs the tools of `ctx.tools` (all of them,
/// or only those whose ids are in `only`),
/// in install order.
/// Why:
/// `update` calls it with `only` set to install just the npm and PyPI tools its lookups need;
/// `install` calls it
///           with `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function syncTools(ctx: Context, only?: string[]): SyncReport
/// ```
pub fn sync_tools(
    ctx: &Context,
    only: Option<&[String]>,
) -> SyncReport {
    let mut report: SyncReport = SyncReport::default();
    let mut entries: Vec<&LockedTool> = Vec::new();
    for tool in sync_order(ctx.tools) {
        if !applies_to_family(&tool.declaration.os, ctx.host.family) {
            continue;
        }
        let selected: bool = match only {
            Some(ids) => ids.contains(&tool.declaration.id),
            None => true,
        };
        if selected {
            entries.push(tool);
        }
    }
    for tool in &entries {
        let observed: Option<String> = observe_tool(ctx, tool);
        // A cargo-binstall install lives in a version directory of its own, so nothing shared is replaced;
        // a repository-published tool is never replaced by an older one (it keeps the newest and warns instead).
        if tool.declaration.route != Route::CargoBinstall
            && !is_repository_published(tool)
            && let Some(warning) =
                replacement_warning(&tool.declaration.id, &observed, compared_with_lock(tool))
        {
            report.warnings.push(warning);
        }
        let current: bool = match observed.as_deref() {
            Some(found) => satisfies_lock(tool, found),
            None => false,
        };
        if current {
            // The root that holds the winning copy: a `current` left stale there (a run killed between the completion marker
            // and the link) would make every scope link an older copy.
            if is_repository_published(tool)
                && let Err(error) = point_current_at_newest_of(&winning_root(ctx.scope, tool), tool)
            {
                report
                    .failed
                    .push((tool.declaration.id.clone(), error.message));
            }
            // A toolchain another program installed is already at its version, so the install path (the only place
            // that applied the lock's component and target rows) never runs; the rows are reconciled here instead,
            // which asks `rustup` what it holds and adds nothing on a machine that already has every part.
            let mut added: Vec<String> = Vec::new();
            let reconciled: Result<(), ToolchainError> =
                reconcile_rust_parts(ctx, tool, &mut added);
            let added_anything: bool = !added.is_empty();
            // Parts added before a later failure still happened, so their sentences stay in the report.
            report.reconciled.extend(added);
            match reconciled {
                Ok(()) if added_anything => report.installed.push(tool.declaration.id.clone()),
                Ok(()) => report.unchanged.push(tool.declaration.id.clone()),
                Err(error) => report
                    .failed
                    .push((tool.declaration.id.clone(), error.message)),
            }
            continue;
        }
        match install_tool(ctx, tool) {
            Ok(()) => report.installed.push(tool.declaration.id.clone()),
            Err(error) => report
                .failed
                .push((tool.declaration.id.clone(), error.message)),
        }
    }
    match sync_postinstall_links(ctx, &entries) {
        Ok(notices) => {
            report.retired_links = notices.retired;
            report.warnings.extend(notices.warnings);
        }
        Err(error) => report
            .failed
            .push(("postinstall links".to_string(), error.message)),
    }
    if let Err(error) = rebuild_exposure(ctx, &entries) {
        report
            .failed
            .push(("exposure directory".to_string(), error.message));
    }
    // Only a full sync writes the tool environment: it is built from the whole lock, and `update`'s partial sync
    // (which installs just the tools its lookups need) must not rewrite it from a subset.
    if only.is_none() {
        match write_tool_env(ctx) {
            Ok(outcome) => report.exported_env = outcome.names,
            Err(error) => report
                .failed
                .push(("tool environment".to_string(), error.message)),
        }
    }
    let commands = run_postinstall_commands(ctx, &entries);
    report.failed.extend(commands.failed);
    report.missing_bins = unresolvable_bins(ctx.scope, &entries);
    report
        .warnings
        .extend(older_lock_warnings(ctx.scope, ctx.tools, ctx.host.family));
    return report;
}
