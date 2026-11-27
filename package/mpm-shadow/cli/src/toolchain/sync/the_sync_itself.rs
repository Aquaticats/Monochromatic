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
use super::super::across_roots::{
    winning_root,
    winning_version,
};
/// Imports the hidden siblings a killed install leaves,
/// the rule that keeps an older repository tool out of the way,
/// and the repair of them.
use super::super::archive_staging::{
    AnOlderVersion,
    Recovered,
    recover_leftovers,
};
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
/// Imports the install directory of one tool version.
use super::super::layout::tool_install_dir;
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
/// Imports the program that shows an install of a tool finished,
/// the removal of the older versions of a tool this repository publishes,
/// and the question that says whether this run is the one that publishes it.
use super::super::published_archive::{
    first_bin_of_tool,
    is_published_archive,
    program_that_finishes_the_install,
    remove_older_archives,
};
/// Imports `write_tool_env` from `super::super::tool_env`.
use super::super::tool_env::write_tool_env;
/// Imports the text of the hidden sibling's own name from `super::super::util`.
use super::super::util::file_name_text;
/// Imports `compare_versions` from `crate::version`,
/// which says which of two version names is the newer one.
use crate::version::compare_versions;
/// Imports the ordered answer `compare_versions` gives.
use std::cmp::Ordering;
/// Imports the owned path type the version directory is built into.
use std::path::PathBuf;

/// Imports the order the tools are installed in.
use super::the_install_order::sync_order;
/// Imports the report a run answers with,
/// the warning for a replaced version,
/// the sentence for a version a killed install left unfinished,
/// and the sentence for an older version the newest-only rule removed.
use super::what_the_sync_did::{
    SyncReport,
    older_versions_removed_sentence,
    recovery_sentence,
    replacement_warning,
};
/// What:
/// The rule that decides whether a recovery may keep a copy of this tool's version:
/// an older repository-published tool is never kept once a newer one is installed,
/// whichever toolchain root holds the newer one.
/// Why:
/// The recovery runs before the check that a newer version satisfies the lock,
/// so this is the only place that keeps an older locked version from appearing beside the newer one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function olderVersionRule(ctx: Context, tool: LockedTool): AnOlderVersion {
///   if (!isRepositoryPublished(tool)) return 'may-stay';
///   const newest = winningVersion(ctx.scope, tool);
///   if (newest === undefined) return 'may-stay';
///   return compareVersions(tool.version, newest) < 0 ? { neverKept: newest } : 'may-stay';
/// }
/// ```
fn older_version_rule(
    ctx: &Context,
    tool: &LockedTool,
) -> AnOlderVersion {
    if !is_repository_published(tool) {
        return AnOlderVersion::MayStay;
    }
    let Some(newest) = winning_version(ctx.scope, tool) else {
        return AnOlderVersion::MayStay;
    };
    // What: `compare_versions` returns `Less`, `Equal` or `Greater`, and `Less` says the locked version is the older one
    //  (TS: a negative number from a comparator).
    // Why: only an older locked version may not be kept; the newest one, and a lock that names it, are kept.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return compareVersions(tool.version, newest) === -1 ? { neverKept: newest } : 'may-stay';
    // ```
    if compare_versions(&tool.version, &newest) == Ordering::Less {
        return AnOlderVersion::NeverKept {
            newer: newest,
        };
    }
    return AnOlderVersion::MayStay;
}

/// What:
/// Finishes the publish of one tool's version that a killed install left unfinished,
/// and removes the hidden directories of installers that no longer run.
/// Why:
/// The copy a killed install left under a hidden name is complete,
/// so the version gets its name back without the archive being downloaded again,
/// which is what an offline machine needs to start that version at all
/// (`archive_staging`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function recoverAKilledInstall(ctx: Context, tool: LockedTool, olderVersion: AnOlderVersion): Recovered
/// ```
fn recover_a_killed_install(
    ctx: &Context,
    tool: &LockedTool,
    older_version: AnOlderVersion,
) -> Result<Recovered, ToolchainError> {
    let version_dir: PathBuf = tool_install_dir(
        &ctx.scope.toolchain_root,
        &tool.declaration.id,
        &tool.version,
    )?;
    return recover_leftovers(
        &version_dir,
        program_that_finishes_the_install(tool),
        older_version,
    );
}

/// What:
/// Settles the rule that an older repository-published tool is never kept after a recovery put a copy at a version's name:
/// every version directory older than the newest finished one is removed,
/// and the report says so.
/// Why:
/// A newer version can land between the recovery's own check and its publish,
/// and a version directory older than the newest one may already be there;
/// the removal is the same one an install makes,
/// run at the same point of the run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleTheNewestOnlyRule(ctx: Context, tool: LockedTool, recovered: Recovered, report: SyncReport): void
/// ```
fn settle_the_newest_only_rule(
    ctx: &Context,
    tool: &LockedTool,
    recovered: &Recovered,
    report: &mut SyncReport,
) {
    if !is_published_archive(tool) || !recovered.put_a_copy_at_the_version_name() {
        return;
    }
    match remove_older_archives(
        &ctx.scope.toolchain_root,
        &tool.declaration.id,
        first_bin_of_tool(tool),
    ) {
        Ok(removed) if removed.is_empty() => {}
        Ok(removed) => {
            // What: `.iter().map(...).collect()` builds the version names out of the directories removed, since the
            //  report names versions and the removal answers paths (TS: `removed.map((path) => basename(path))`).
            // Why: the sentence a person reads names the versions, not the directories they sit in.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const versions = removed.map((directory) => basename(directory));
            // ```
            let versions: Vec<String> = removed
                .iter()
                .map(|directory| return file_name_text(directory))
                .collect();
            report.recovered.push(older_versions_removed_sentence(
                &tool.declaration.id,
                &versions,
            ));
        }
        Err(error) => report
            .failed
            .push((tool.declaration.id.clone(), error.message)),
    }
}

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
        let mut recovered: Recovered = Recovered::default();
        let mut recovery_problem: Option<String> = None;
        if tool.declaration.route == Route::Archive {
            match recover_a_killed_install(ctx, tool, older_version_rule(ctx, tool)) {
                Ok(outcome) => recovered = outcome,
                // The install below may still put the version in place: a recovery that failed must not cost the run the
                // download that can repair it, and a network that is there is what an offline machine does not have.
                Err(error) => recovery_problem = Some(error.message),
            }
        }
        if let Some(sentence) = recovery_sentence(&tool.declaration.id, &tool.version, &recovered) {
            report.recovered.push(sentence);
        }
        settle_the_newest_only_rule(ctx, tool, &recovered, &mut report);
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
                Ok(()) if recovered.installed_here_or_meanwhile() => {
                    report.installed.push(tool.declaration.id.clone());
                }
                Ok(()) => report.unchanged.push(tool.declaration.id.clone()),
                Err(error) => report
                    .failed
                    .push((tool.declaration.id.clone(), error.message)),
            }
            // A recovery that failed is a warning here, not a failure: the version's name holds the finished copy the
            // lock asks for, and what went wrong is a hidden sibling this run left or could not remove.
            if let Some(problem) = recovery_problem {
                report.warnings.push(problem);
            }
            continue;
        }
        match install_tool(ctx, tool) {
            Ok(()) => {
                report.installed.push(tool.declaration.id.clone());
                if let Some(problem) = recovery_problem {
                    report.warnings.push(problem);
                }
            }
            Err(error) => {
                // The tool failed, so both causes are one failure a person reads: the recovery's own message says what
                // disk holds, and the install's says why the version did not come back.
                let message: String = match recovery_problem {
                    Some(problem) => {
                        format!(
                            "{problem}; the install that followed it also failed: {}",
                            error.message
                        )
                    }
                    None => error.message,
                };
                report.failed.push((tool.declaration.id.clone(), message));
            }
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
