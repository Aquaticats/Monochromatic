//! What:
//! Carries out the plan for the old copies of this repository's own tools,
//! in the owner's order:
//! links that resolve to an old copy move to the managed copy first,
//! then the old copy and its cargo record are removed,
//! each action reported as a line.
//! Why:
//! The chosen option "Re-point,
//! then remove":
//! "After its own newest copy is installed:
//! links in known places ...
//! that target an old copy are re-pointed to the managed
//! copy;
//! then the old copy and its cargo record are removed,
//! each action printed."
//! Nothing is removed while a link that could be seen still resolves to the copy,
//! so the cleanup never leaves a dangling link behind.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old-copy-apply.ts
//! export function retireOldCopies(ctx: Context, failed: [string, string][]): { lines: string[]; warnings: string[]; failed: [string, string][] }
//! export function previewLines(plans: OldCopyPlans): string[]
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `Path` from `std::path`.
use std::path::Path;

/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports `atomic_symlink` from `super::install_support`.
use super::install_support::atomic_symlink;
/// Imports what this module uses from `super::cargo_record`.
use super::cargo_record::{RecordedInstall, install_naming_bin, read_recorded_installs};
/// Imports `cargo_home_dir` and `is_executable` from `super::layout`.
use super::layout::{cargo_home_dir, is_executable};
/// Imports what this module uses from `super::old_copies`.
use super::old_copies::{OldCopyPlan, OldCopyPlans, Repoint, plan_old_copies};
/// Imports what this module uses from `super::managed_losers`.
use super::managed_losers::LoserPlan;
/// Imports `links_to_copy` from `super::old_copy_places`.
use super::old_copy_places::links_to_copy;
/// Imports `remove_downloads` from `super::published_archive`.
use super::published_archive::remove_downloads;

/// What the cleanup did:
/// the action lines,
/// the old copies it kept (loud warnings),
/// and the failures.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OldCopyOutcome {
    /// One line per action taken (a link re-pointed,
    /// a copy removed).
    pub lines: Vec<String>,
    /// One warning per old copy that stays,
    /// with the reason.
    pub warnings: Vec<String>,
    /// A step that failed,
    /// as a label and a message;
    /// `install` counts these as failures.
    pub failed: Vec<(String, String)>,
}

/// The warning for an old copy that stays.
fn kept_warning(plan: &OldCopyPlan, reason: &str) -> String {
    return format!(
        "old copy {} of {} is kept (it is not the one `install` put there): {reason}",
        plan.path.display(),
        plan.tool_id
    );
}

/// The label of a failure about one old copy.
fn failure_label(plan: &OldCopyPlan) -> String {
    return format!("old copy {}", plan.path.display());
}

/// What:
/// Removes the old copy:
/// `cargo uninstall --root <cargo home> --bin <bin> <crate>` when cargo's record names the bin,
/// then the file itself when it is still there.
/// Returns what happened to the record,
/// as text for the report.
/// Why:
/// `cargo uninstall` updates both of cargo's records and removes only the bin named,
/// so a crate with other bins stays recorded for them.
/// It decides from `.crates.toml` alone (measured 2026-11-01),
/// so a bin that file does not name is removed directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removeCopy(ctx: Context, plan: OldCopyPlan): string  // throws
/// ```
fn remove_copy(ctx: &Context, plan: &OldCopyPlan) -> Result<String, String> {
    let cargo_home: String = cargo_home_dir(ctx.env, &ctx.scope.home).to_string_lossy().to_string();
    let installs: Vec<RecordedInstall> = read_recorded_installs(&cargo_home_dir(ctx.env, &ctx.scope.home));
    let mut record_note: String = "cargo has no record of it".to_string();
    if let Some(install) = install_naming_bin(&installs, &plan.bin) {
        let request = ctx.request("cargo", &["uninstall", "--root", &cargo_home, "--bin", &plan.bin, &install.crate_name]);
        let outcome = ctx.run(&request);
        if !outcome.succeeded() {
            return Err(format!(
                "`{}` failed, so the copy stays (its links already moved); fix what cargo reports and run `mpm-shadow install` again: {}",
                request.line(),
                outcome.failure_text()
            ));
        }
        record_note = format!("`cargo uninstall` removed it from cargo's record of {} {}", install.crate_name, install.version);
    }
    if fs::symlink_metadata(&plan.path).is_ok()
        && let Err(error) = fs::remove_file(&plan.path)
    {
        return Err(format!("cannot remove the file: {error}"));
    }
    return Ok(record_note);
}

/// Re-points one link,
/// recording the line or the failure under `label`.
/// A link that could not move still resolves to the copy,
/// and the search before the removal keeps the copy for that reason.
fn repoint(label: &str, link: &Repoint, outcome: &mut OldCopyOutcome) {
    match atomic_symlink(&link.link, &link.target) {
        Ok(()) => outcome.lines.push(format!("re-pointed {} -> {}", link.link.display(), link.target.display())),
        Err(error) => outcome.failed.push((label.to_string(), format!("cannot re-point {}: {error}", link.link.display()))),
    }
}

/// What:
/// Applies one plan:
/// moves its links,
/// then removes the copy when nothing that could be seen still resolves to it.
/// Why:
/// The links are searched for again after they moved,
/// and the managed copy must still be executable,
/// so a link made meanwhile or a managed copy removed meanwhile keeps the old copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyPlan(ctx: Context, plans: OldCopyPlans, plan: OldCopyPlan, outcome: OldCopyOutcome): void
/// ```
fn apply_plan(ctx: &Context, plans: &OldCopyPlans, plan: &OldCopyPlan, outcome: &mut OldCopyOutcome) {
    for link in &plan.repoints {
        repoint(&failure_label(plan), link, outcome);
    }
    if let Some(reason) = &plan.kept {
        outcome.warnings.push(kept_warning(plan, reason));
        return;
    }
    let Some(managed) = &plan.managed else {
        outcome.warnings.push(kept_warning(plan, "there is no managed copy to take over"));
        return;
    };
    if !is_executable(managed) {
        outcome.warnings.push(kept_warning(plan, &format!("the managed copy {} is no longer executable", managed.display())));
        return;
    }
    let remaining = links_to_copy(&plans.places, &plan.path);
    if !remaining.found.is_empty() || !remaining.unreadable.is_empty() {
        outcome.warnings.push(kept_warning(plan, "a link to it still resolves to it, so removing it would leave that link dangling"));
        return;
    }
    match remove_copy(ctx, plan) {
        Ok(record_note) => outcome.lines.push(format!(
            "removed {} (old copy of {}; the managed copy is {}; {record_note})",
            plan.path.display(),
            plan.tool_id,
            managed.display()
        )),
        Err(message) => outcome.failed.push((failure_label(plan), message)),
    }
}

/// The label of a failure about the older copies of one tool in one root.
fn loser_label(plan: &LoserPlan) -> String {
    return format!("older managed copy of {} in {}", plan.tool_id, plan.root.display());
}

/// The warning for older managed copies that stay.
fn loser_kept_warning(plan: &LoserPlan, reason: &str) -> String {
    return format!(
        "older managed copy of {} in {} is kept ({} in {} runs): {reason}",
        plan.tool_id,
        plan.root.display(),
        plan.winner_version,
        plan.winner_root.display()
    );
}

/// Removes one older version's directory
/// (and,
/// for a release archive,
/// its downloads).
fn remove_version(plan: &LoserPlan, directory: &Path, version: &str) -> Result<(), String> {
    if let Err(error) = fs::remove_dir_all(directory) {
        return Err(format!("cannot remove {}: {error}", directory.display()));
    }
    if plan.archive
        && let Err(error) = remove_downloads(&plan.root, &plan.tool_id, version)
    {
        return Err(error.message);
    }
    return Ok(());
}

/// What:
/// Applies the plan for the older managed copies of one tool in one root:
/// moves the links that resolve to them,
/// removes the versions when nothing that could be seen still resolves to one,
/// and leaves the emptied root's `current` forwarding to the winner's.
/// Why:
/// The forwarding link is why a launcher record or a link that names the emptied root's `current` keeps running the winner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyLoser(plans: OldCopyPlans, plan: LoserPlan, outcome: OldCopyOutcome): void
/// ```
fn apply_loser(plans: &OldCopyPlans, plan: &LoserPlan, outcome: &mut OldCopyOutcome) {
    for link in &plan.repoints {
        repoint(&loser_label(plan), link, outcome);
    }
    if let Some(reason) = &plan.kept {
        outcome.warnings.push(loser_kept_warning(plan, reason));
        return;
    }
    for copy in &plan.copies {
        for (_, file) in &copy.bins {
            let remaining = links_to_copy(&plans.places, file);
            if !remaining.found.is_empty() || !remaining.unreadable.is_empty() {
                outcome.warnings.push(loser_kept_warning(plan, "a link to it still resolves to it, so removing it would leave that link dangling"));
                return;
            }
        }
    }
    for copy in &plan.copies {
        match remove_version(plan, &copy.directory, &copy.version) {
            Ok(()) => outcome.lines.push(format!(
                "removed {} (older managed copy of {}; {} in {} runs)",
                copy.directory.display(),
                plan.tool_id,
                plan.winner_version,
                plan.winner_root.display()
            )),
            Err(message) => outcome.failed.push((loser_label(plan), message)),
        }
    }
    if let Some((link, target)) = &plan.forward {
        match atomic_symlink(link, target) {
            Ok(()) => outcome.lines.push(format!("{} now forwards to {}", link.display(), target.display())),
            Err(error) => outcome.failed.push((loser_label(plan), format!("cannot forward {}: {error}", link.display()))),
        }
    }
}

/// What:
/// Carries out every plan in order.
/// Why:
/// Separate from planning so `list` can print the plan without touching anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function applyOldCopies(ctx: Context, plans: OldCopyPlans): OldCopyOutcome
/// ```
pub fn apply_old_copies(ctx: &Context, plans: &OldCopyPlans) -> OldCopyOutcome {
    let mut outcome: OldCopyOutcome = OldCopyOutcome::default();
    for plan in &plans.plans {
        apply_plan(ctx, plans, plan, &mut outcome);
    }
    for loser in &plans.losers {
        apply_loser(plans, loser, &mut outcome);
    }
    return outcome;
}

/// What:
/// Plans and carries out the cleanup of the old copies for the tools of `ctx`.
/// `failed` is what the install of this run reported as failed.
/// Why:
/// `install` calls this after its own installs and links are done,
/// so the managed copies and this tree's own links exist when the old copies are judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireOldCopies(ctx: Context, failed: [string, string][]): OldCopyOutcome
/// ```
pub fn retire_old_copies(ctx: &Context, failed: &[(String, String)]) -> OldCopyOutcome {
    return apply_old_copies(ctx, &plan_old_copies(ctx, failed));
}

/// What:
/// The lines `list` prints for the old copies:
/// what `install` would do with each,
/// or why it would keep it.
/// Why:
/// A person can see the cleanup coming before running it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function previewLines(plans: OldCopyPlans): string[]
/// ```
pub fn preview_lines(plans: &OldCopyPlans) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for plan in &plans.plans {
        let mut links: Vec<String> = Vec::new();
        for link in &plan.repoints {
            links.push(link.link.display().to_string());
        }
        let moves: String = if links.is_empty() {
            "no link to re-point".to_string()
        } else {
            format!("re-point {} link(s) ({})", links.len(), links.join(", "))
        };
        match &plan.kept {
            Some(reason) => lines.push(format!(
                "old copy {} of {} stays: {reason}; `install` would {moves}",
                plan.path.display(),
                plan.tool_id
            )),
            None => lines.push(format!(
                "old copy {} of {} would be removed by `install` after it did: {moves}",
                plan.path.display(),
                plan.tool_id
            )),
        }
    }
    for plan in &plans.losers {
        let mut versions: Vec<String> = Vec::new();
        for copy in &plan.copies {
            versions.push(copy.version.clone());
        }
        let moves: String = format!("re-point {} link(s)", plan.repoints.len());
        match &plan.kept {
            Some(reason) => lines.push(format!(
                "older managed copy of {} in {} ({}) stays: {reason}; `install` would {moves}",
                plan.tool_id,
                plan.root.display(),
                versions.join(", ")
            )),
            None => lines.push(format!(
                "older managed copy of {} in {} ({}) would be removed by `install` after it did: {moves}; {} in {} runs",
                plan.tool_id,
                plan.root.display(),
                versions.join(", "),
                plan.winner_version,
                plan.winner_root.display()
            )),
        }
    }
    return lines;
}
