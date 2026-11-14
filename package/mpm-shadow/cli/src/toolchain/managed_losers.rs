//! What:
//! Finds the older managed copies of this repository's tools that the newest copy across both toolchain roots makes redundant,
//! and decides which links move and whether they may go.
//! Why:
//! The owner's decision:
//! this repository's tools are one set across both roots,
//! the newest copy wins wherever it is,
//! and the older managed copy is removed.
//! A copy is removed only after the links that resolve to it moved to the winner,
//! and only while the winner is a working executable,
//! so the older copy is never the only working one.
//! A copy in the other scope's root is removed only when every place that could link to it was searched,
//! because that scope's links are not this run's own.
//! The removal leaves a `current` link in the emptied root that forwards to the winner's `current`,
//! so a launcher record or a link that names the emptied root's `current` keeps running the winner.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // managed-losers.ts
//! export function planLosers(ctx: Context, failed: [string, string][], places: Places | undefined): LoserPlan[]
//! ```

/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;
/// Imports `fs` from `std`.
use std::fs;
/// Imports `PathBuf` from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports what this module uses from `super::across_roots`.
use super::across_roots::{WinningCopy, winning_copy};
/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::{current_link, finished_versions_of, is_repository_published, running_version, version_directory};
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports what this module uses from `super::layout`.
use super::layout::{applies_to_family, find_bin_under, find_tool_bin, tool_install_dir};
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports `package_name` from `super::observe`.
use super::observe::package_name;
/// Imports `find_tool` from `super::util`.
use super::util::find_tool;
/// Imports `Repoint` from `super::old_copies`.
use super::old_copies::Repoint;
/// Imports what this module uses from `super::old_copy_places`.
use super::old_copy_places::{Places, links_to_copy, places_of};
/// Imports `running_version_archive` from `super::published_archive`.
use super::published_archive::{current_link_of, is_published_archive, running_version_archive};
/// Imports the keeps every removal asks from `super::store_keeps`.
use super::store_keeps::InstallKeeps;

/// One older version of a tool in a root that does not hold the newest copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoserCopy {
    /// The version.
    pub version: String,
    /// The version's directory in the root.
    pub directory: PathBuf,
    /// Each declared bin that exists in the version,
    /// with its file.
    pub bins: Vec<(String, PathBuf)>,
}

/// One older version of a tool in a root that this run keeps,
/// and why it stays.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeptCopy {
    /// The version.
    pub version: String,
    /// The version's directory in the root.
    pub directory: PathBuf,
    /// Why nothing may remove it:
    /// the lock that names it,
    /// or the process that uses it.
    pub reason: String,
}

/// What the cleanup decided for the older managed copies of one tool in one root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoserPlan {
    /// The id of the plan entry.
    pub tool_id: String,
    /// The toolchain root that holds the older copies.
    pub root: PathBuf,
    /// The toolchain root that holds the newest copy.
    pub winner_root: PathBuf,
    /// The newest version,
    /// the one that runs.
    pub winner_version: String,
    /// The older versions this run removes,
    /// newest first.
    pub copies: Vec<LoserCopy>,
    /// The older versions this run keeps,
    /// newest first,
    /// each with the lock or the process that keeps it.
    pub kept_copies: Vec<KeptCopy>,
    /// The links to move,
    /// sorted.
    pub repoints: Vec<Repoint>,
    /// Why the older copies stay,
    /// or `None` when they are removed once the links moved.
    pub kept: Option<String>,
    /// The `current` link of the root and the winner's `current` link it will forward to,
    /// used when removing the older copies leaves the root with no finished version of the tool
    /// (`emptied_by` asks again when the plan is applied).
    pub forward: Option<(PathBuf, PathBuf)>,
    /// Whether the tool is a release archive (its downloads go with its versions).
    pub archive: bool,
}

/// The directory of `version` of the tool in `root`.
fn version_dir_in(root: &Path, tool: &LockedTool, version: &str) -> Option<PathBuf> {
    if is_published_archive(tool) {
        return tool_install_dir(root, &tool.declaration.id, version).ok();
    }
    return Some(version_directory(root, &package_name(tool), version));
}

/// The file of `bin` inside one version directory,
/// when the version holds it.
fn bin_in(directory: &Path, tool: &LockedTool, bin: &str) -> Option<PathBuf> {
    if is_published_archive(tool) {
        return find_bin_under(directory, bin);
    }
    let file: PathBuf = directory.join("bin").join(bin);
    if fs::metadata(&file).is_ok() {
        return Some(file);
    }
    return None;
}

/// The older copies of the tool in `root`,
/// newest first,
/// each with its declared bins.
fn older_copies_in(root: &Path, tool: &LockedTool, winner: &WinningCopy) -> Vec<LoserCopy> {
    let mut copies: Vec<LoserCopy> = Vec::new();
    for version in finished_versions_of(root, tool) {
        if compare_versions(&version, &winner.version) != Ordering::Less {
            continue;
        }
        let Some(directory) = version_dir_in(root, tool, &version) else {
            continue;
        };
        let mut bins: Vec<(String, PathBuf)> = Vec::new();
        for bin in &tool.declaration.bins {
            if let Some(file) = bin_in(&directory, tool, bin) {
                bins.push((bin.clone(), file));
            }
        }
        copies.push(LoserCopy { version, directory, bins });
    }
    return copies;
}

/// The `current` link of the tool in `root`.
fn current_in(root: &Path, tool: &LockedTool) -> Option<PathBuf> {
    if is_published_archive(tool) {
        return current_link_of(root, &tool.declaration.id).ok();
    }
    return Some(current_link(root, &package_name(tool)));
}

/// Why the older copies stay before any link is looked at,
/// or `None` when they may go once their links moved.
/// The `copies` here are the ones this plan would remove:
/// a version a lock names is not one of them,
/// so a failure that only endangers a version nobody would remove keeps nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function preconditionReason(ctx: Context, tool: LockedTool, copies: LoserCopy[], failed: [string, string][]): string | undefined
/// ```
fn precondition_reason(ctx: &Context, tool: &LockedTool, copies: &[LoserCopy], failed: &[(String, String)]) -> Option<String> {
    for (id, _) in failed {
        if *id == tool.declaration.id {
            return Some(format!("the managed install of {id} failed in this run, so this may be the only working copy"));
        }
    }
    for copy in copies {
        for (bin, _) in &copy.bins {
            if find_tool_bin(tool, bin, ctx.scope, ctx.env, &[]).is_none() {
                return Some(format!("the newest copy has no working {bin}, so this may be the only working one"));
            }
        }
    }
    return None;
}

/// What:
/// The plan for the older copies of one tool in one root:
/// the links that resolve to them,
/// each with the newest copy's file of the same bin as its target.
/// `kept_copies` are the older versions a lock names or a process uses,
/// which this plan does not remove and does not move a link away from.
/// Why:
/// Reads only.
/// A root that is not the running scope's keeps its older copies while some place could not be searched.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planRoot(ctx: Context, tool: LockedTool, winner: WinningCopy, root: string, copies: LoserCopy[], keptCopies: KeptCopy[], failed: [string, string][], places: Places): LoserPlan
/// ```
fn plan_root(
    ctx: &Context,
    tool: &LockedTool,
    winner: &WinningCopy,
    root: PathBuf,
    copies: Vec<LoserCopy>,
    kept_copies: Vec<KeptCopy>,
    failed: &[(String, String)],
    places: &Places,
) -> LoserPlan {
    let mut repoints: Vec<Repoint> = Vec::new();
    let mut unreadable: Vec<PathBuf> = Vec::new();
    for copy in &copies {
        for (bin, file) in &copy.bins {
            let scan = links_to_copy(places, file);
            unreadable.extend(scan.unreadable);
            let Some(target) = find_tool_bin(tool, bin, ctx.scope, ctx.env, &[]) else {
                continue;
            };
            for found in scan.found {
                repoints.push(Repoint { link: found.path, kind: found.kind, target: target.clone() });
            }
        }
    }
    repoints.sort_by(compare_repoints);
    repoints.dedup();
    let mut reasons: Vec<String> = Vec::new();
    if let Some(reason) = precondition_reason(ctx, tool, &copies, failed) {
        reasons.push(reason);
    }
    if root != ctx.scope.toolchain_root {
        for reason in &places.incomplete {
            reasons.push(format!("another scope's links may point at it and a place could not be searched: {reason}"));
        }
    }
    for directory in unreadable {
        reasons.push(format!("{} could not be listed, so links in it are unseen", directory.display()));
    }
    let kept: Option<String> = if reasons.is_empty() { None } else { Some(reasons.join("; ")) };
    let forward: Option<(PathBuf, PathBuf)> = match (current_in(&root, tool), current_in(&winner.root, tool)) {
        (Some(link), Some(target)) => Some((link, target)),
        _ => None,
    };
    return LoserPlan {
        tool_id: tool.declaration.id.clone(),
        root,
        winner_root: winner.root.clone(),
        winner_version: winner.version.clone(),
        copies,
        kept_copies,
        repoints,
        kept,
        forward,
        archive: is_published_archive(tool),
    };
}

/// Orders two re-points by the link they move.
fn compare_repoints(left: &Repoint, right: &Repoint) -> Ordering {
    return left.link.cmp(&right.link);
}

/// What:
/// Whether removing the planned versions leaves the plan's root with no finished version of the tool,
/// asked again when the plan is applied.
/// Why:
/// Another tree can install a newer version into the root between the plan and its application;
/// that root then owns its `current`,
/// and a forward would overwrite it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function emptiedBy(ctx: Context, plan: LoserPlan): boolean
/// ```
pub fn emptied_by(ctx: &Context, plan: &LoserPlan) -> bool {
    let Some(tool) = find_tool(ctx.tools, &plan.tool_id) else {
        return false;
    };
    for version in finished_versions_of(&plan.root, tool) {
        let mut planned: bool = false;
        for copy in &plan.copies {
            if copy.version == version {
                planned = true;
            }
        }
        if !planned {
            return false;
        }
    }
    return true;
}

/// What:
/// The copies of one root this run removes,
/// and the ones no removal may take away.
/// Why:
/// A version a running process of this user uses outranks the newest-only rule,
/// so it is not part of what the plan removes;
/// the links that resolve to it also stay where they are,
/// because the version they reach is still there.
/// A version that only a lock names is removed here,
/// by the owner's decision that this repository's tools are one set across both roots:
/// a tree whose lock pins an older version runs the newer one and is warned,
/// and the scope keeps no older copy of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function partitionKept(ctx: Context, copies: LoserCopy[]): [LoserCopy[], KeptCopy[]]
/// ```
fn partition_kept(ctx: &Context, copies: Vec<LoserCopy>) -> (Vec<LoserCopy>, Vec<KeptCopy>) {
    // What: the locks and the process table are read once for every copy of this tool in this root.
    // Why: one read judges all of them by the same state, and the read costs about a second.
    let keeps: InstallKeeps = InstallKeeps::of(ctx);
    let mut removable: Vec<LoserCopy> = Vec::new();
    let mut kept: Vec<KeptCopy> = Vec::new();
    for copy in copies {
        match keeps.in_use_reason(&copy.directory) {
            Some(reason) => kept.push(KeptCopy { version: copy.version, directory: copy.directory, reason }),
            None => removable.push(copy),
        }
    }
    return (removable, kept);
}

/// What:
/// What this run does to the loser root's `current` link once its removals are done.
/// Why:
/// The link names the newest finished version of that root,
/// which may be one of the versions this run removes,
/// and a root that keeps an older version must not be left with a link that points at nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CurrentAfterRemoval =
///   | { kind: 'leave' }
///   | { kind: 'point'; link: string; version: string }
///   | { kind: 'forward'; link: string; target: string }
///   | { kind: 'remove'; link: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrentAfterRemoval {
    /// The link does not name a version this run removes, so it is left as it is.
    Leave,
    /// The link names a version this run removes, and this version of the same root stays.
    Point(PathBuf, String),
    /// The root keeps no finished version, so its link forwards to the winner's `current`.
    Forward(PathBuf, PathBuf),
    /// The root keeps no finished version and there is nothing to forward to, so its link goes.
    Remove(PathBuf),
}

/// What:
/// The newest finished version of the plan's tool that stays in the plan's root,
/// or `None` when every finished one there is removed.
/// Why:
/// It is what the root's `current` has to name after the removals,
/// and a root that keeps a version a lock names keeps a working link to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function newestSurvivor(ctx: Context, plan: LoserPlan): string | undefined
/// ```
fn newest_survivor(ctx: &Context, plan: &LoserPlan) -> Option<String> {
    let tool: &LockedTool = find_tool(ctx.tools, &plan.tool_id)?;
    // What: `finished_versions_of` answers newest first, so the first one that is not removed is the newest survivor.
    // Why: the link has to name the newest version that stays, the way an install writes it.
    for version in finished_versions_of(&plan.root, tool) {
        let going: bool = plan.copies.iter().any(|copy| return copy.version == version);
        if !going {
            return Some(version);
        }
    }
    return None;
}

/// What:
/// The version the root's `current` link names,
/// whichever route installed the tool.
/// Why:
/// The plan is about one tool of one root,
/// and an archive tool's link is not the crate link,
/// so the read follows the route the tool's declaration names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runningVersionOf(root: string, tool: LockedTool): string | undefined
/// ```
fn running_version_of(root: &Path, tool: &LockedTool) -> Option<String> {
    if is_published_archive(tool) {
        return running_version_archive(root, &tool.declaration.id);
    }
    return running_version(root, &package_name(tool));
}

/// What:
/// What this run does to the loser root's `current` link,
/// asked when the plan is applied so a version another run installed meanwhile is seen.
/// Why:
/// The link may name a version this run removes while the root keeps an older one,
/// and the two answers that matter are the version that stays and the forward to the winner.
/// A link that names nothing this run removes stays as it is:
/// it may be another run's newer install, and that run owns its link.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function currentAfterRemoval(ctx: Context, plan: LoserPlan): CurrentAfterRemoval
/// ```
pub fn current_after_removal(ctx: &Context, plan: &LoserPlan) -> CurrentAfterRemoval {
    let Some(tool) = find_tool(ctx.tools, &plan.tool_id) else {
        return CurrentAfterRemoval::Leave;
    };
    let Some(link) = current_in(&plan.root, tool) else {
        return CurrentAfterRemoval::Leave;
    };
    let Some(named) = running_version_of(&plan.root, tool) else {
        return CurrentAfterRemoval::Leave;
    };
    let going: bool = plan.copies.iter().any(|copy| return copy.version == named);
    if !going {
        return CurrentAfterRemoval::Leave;
    }
    if let Some(survivor) = newest_survivor(ctx, plan) {
        return CurrentAfterRemoval::Point(link, survivor);
    }
    if let Some((forward_link, target)) = &plan.forward {
        return CurrentAfterRemoval::Forward(forward_link.clone(), target.clone());
    }
    return CurrentAfterRemoval::Remove(link);
}

/// What:
/// The plans for every tool of `ctx.tools` that has older finished copies in a root other than the winner's.
/// `failed` is what the install of this run reported as failed.
/// Why:
/// The places are searched only when some tool has an older copy,
/// so an ordinary run asks git nothing,
/// and the locks and the process table are read only when a copy would otherwise be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planLosers(ctx: Context, failed: [string, string][], places: Places | undefined): LoserPlan[]
/// ```
pub fn plan_losers(ctx: &Context, failed: &[(String, String)], places: &mut Option<Places>) -> Vec<LoserPlan> {
    let mut plans: Vec<LoserPlan> = Vec::new();
    for tool in ctx.tools {
        if !is_repository_published(tool) || !applies_to_family(&tool.declaration.os, ctx.host.family) {
            continue;
        }
        let Some(winner) = winning_copy(ctx.scope, tool) else {
            continue;
        };
        for root in [ctx.scope.toolchain_root.clone(), ctx.scope.other_toolchain_root.clone()] {
            if root == winner.root {
                continue;
            }
            let copies: Vec<LoserCopy> = older_copies_in(&root, tool, &winner);
            if copies.is_empty() {
                continue;
            }
            // What: the versions a lock names or a process uses are set aside before any link is looked at.
            // Why: their links still resolve, so nothing about them dangles, and the plan must not count them as removed.
            let (removable, kept_copies) = partition_kept(ctx, copies);
            let searched: &Places = places_of(places, ctx.scope, ctx.env, ctx.runner);
            plans.push(plan_root(ctx, tool, &winner, root, removable, kept_copies, failed, searched));
        }
    }
    return plans;
}
