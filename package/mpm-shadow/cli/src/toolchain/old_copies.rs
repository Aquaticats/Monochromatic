//! What:
//! Finds the old copies of this repository's own tools and decides,
//! for each,
//! which links move to the managed copy and whether the copy may be removed.
//! An old copy is a regular file in the shared cargo bin directory (`$CARGO_HOME/bin`,
//! default `~/.cargo/bin`) named like a bin of a repository-published tool the scope's lock declares,
//! which mpm-shadow did not install.
//! Why:
//! The owner's rule:
//! "No older repo-maintained tools should ever exist when newer is present."
//! The retired TypeScript sync placed such copies there,
//! and `~/.local/bin/git` and every checkout's `.cache/*-bin` links still run them.
//! The decision is a plan over the file system and changes nothing,
//! so `list` shows what `install` would do and `install` applies the same plan.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // old-copies.ts
//! export function planOldCopies(ctx: Context, failed: [string, string][]): { plans: OldCopyPlan[]; places: Places }
//! ```

/// Imports `Ordering` from `std::cmp`.
use std::cmp::Ordering;
/// Imports `fs` from `std`.
use std::fs;
/// Imports what this module uses from `std::path`.
use std::path::{Path, PathBuf};

/// Imports `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::OWN_CRATE_PREFIX;
/// Imports `compare_versions` from `crate::version`.
use crate::version::compare_versions;

/// Imports what this module uses from `super::cargo_installs`.
use super::cargo_installs::{is_repository_published, newest_installed_of};
/// Imports what this module uses from `super::cargo_record`.
use super::cargo_record::{RecordedInstall, install_naming_bin, read_recorded_installs};
/// Imports `Context` from `super::context`.
use super::context::Context;
/// Imports what this module uses from `super::layout`.
use super::layout::{applies_to_family, cargo_bin_dir, cargo_home_dir, find_tool_bin};
/// Imports `LockedTool` from `super::model`.
use super::model::LockedTool;
/// Imports what this module uses from `super::old_copy_places`.
use super::old_copy_places::{LinkScan, PlaceKind, Places, known_places, links_to_copy};
/// Imports what this module uses from `super::scope`.
use super::scope::{ScopeKind, toolchain_roots};

/// A link the cleanup re-points at the managed copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Repoint {
    /// The link.
    pub link: PathBuf,
    /// The kind of place the link sits in.
    pub kind: PlaceKind,
    /// The managed copy it will point at.
    pub target: PathBuf,
}

/// What the cleanup decided for one old copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OldCopyPlan {
    /// The id of the plan entry whose bin this is a copy of.
    pub tool_id: String,
    /// The bin name.
    pub bin: String,
    /// The old copy.
    pub path: PathBuf,
    /// The managed copy of the same bin,
    /// when there is a working one.
    pub managed: Option<PathBuf>,
    /// The links to move,
    /// sorted.
    pub repoints: Vec<Repoint>,
    /// Why the old copy stays,
    /// or `None` when it is removed once the links moved.
    pub kept: Option<String>,
}

/// Every old copy of the run,
/// with the places that were searched for links.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OldCopyPlans {
    /// One plan per old copy,
    /// in lock order and bin order.
    pub plans: Vec<OldCopyPlan>,
    /// The places searched;
    /// empty when no old copy exists,
    /// because then nothing asked git for the checkouts.
    pub places: Places,
}

/// The entry named like a bin in the shared cargo bin directory.
struct Candidate {
    /// The bin name.
    bin: String,
    /// `<cargo home>/bin/<bin>`.
    path: PathBuf,
    /// Whether it is a symbolic link and not a file.
    is_link: bool,
}

/// The path with symbolic links resolved when it exists,
/// otherwise as given.
fn resolved(path: &Path) -> PathBuf {
    return match fs::canonicalize(path) {
        Ok(found) => found,
        Err(_) => path.to_path_buf(),
    };
}

/// Whether `path` lies inside a toolchain root of this user (either scope's),
/// so it is a managed copy and not an old one.
fn inside_a_toolchain_root(ctx: &Context, path: &Path) -> bool {
    let target: PathBuf = resolved(path);
    let mut roots: Vec<PathBuf> = toolchain_roots(ctx.env);
    roots.push(ctx.scope.toolchain_root.clone());
    for root in roots {
        if target.starts_with(resolved(&root)) {
            return true;
        }
    }
    return false;
}

/// The entries of the shared cargo bin directory named like the declared bins of `tool`.
fn candidates_of(ctx: &Context, tool: &LockedTool) -> Vec<Candidate> {
    let directory: PathBuf = cargo_bin_dir(ctx.env, &ctx.scope.home);
    let mut found: Vec<Candidate> = Vec::new();
    for bin in &tool.declaration.bins {
        let path: PathBuf = directory.join(bin);
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.is_dir() || inside_a_toolchain_root(ctx, &path) {
            continue;
        }
        found.push(Candidate { bin: bin.clone(), path, is_link: metadata.file_type().is_symlink() });
    }
    return found;
}

/// The toolchain root of the other scope,
/// or `None` when there is none.
fn other_root(ctx: &Context) -> Option<PathBuf> {
    let own: PathBuf = resolved(&ctx.scope.toolchain_root);
    for root in toolchain_roots(ctx.env) {
        if resolved(&root) != own {
            return Some(root);
        }
    }
    return None;
}

/// The working managed copy of `bin` under `root`:
/// the executable behind the `current` link of the tool there.
fn managed_copy_under(ctx: &Context, tool: &LockedTool, bin: &str, root: &Path) -> Option<PathBuf> {
    let mut scope = ctx.scope.clone();
    scope.toolchain_root = root.to_path_buf();
    return find_tool_bin(tool, bin, &scope, ctx.env, &[]);
}

/// What the cleanup does with the links of one place kind,
/// by which toolchain roots hold a working copy.
enum Target {
    /// Re-point at this copy.
    Use(
        /// The managed copy.
        PathBuf,
    ),
    /// Both scopes hold a copy and the place belongs to neither;
    /// the owner has not decided which one wins.
    Undecided,
}

/// What:
/// The managed copy a link in a place of `kind` moves to.
/// Why:
/// A checkout belongs to the repository scope,
/// so its links go to the repository scope's copy when it has one.
/// Outside any checkout (`~/.local/bin`,
/// the global farm) a single working copy in one root is the only candidate;
/// when both roots hold one,
/// which wins is the owner's decision,
/// so the link is left alone and the old copy stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function targetFor(kind: PlaceKind, repoCopy?: string, globalCopy?: string): Target
/// ```
fn target_for(kind: PlaceKind, repository_copy: Option<&Path>, global_copy: Option<&Path>) -> Target {
    if kind == PlaceKind::Checkout {
        return match repository_copy.or(global_copy) {
            Some(copy) => Target::Use(copy.to_path_buf()),
            None => Target::Undecided,
        };
    }
    return match (repository_copy, global_copy) {
        (Some(only), None) | (None, Some(only)) => Target::Use(only.to_path_buf()),
        _ => Target::Undecided,
    };
}

/// The repository scope's copy and the global scope's copy,
/// given which one this run belongs to.
fn copies_by_scope(ctx: &Context, mine: &Path, other: Option<PathBuf>) -> (Option<PathBuf>, Option<PathBuf>) {
    return match ctx.scope.kind {
        ScopeKind::Repo => (Some(mine.to_path_buf()), other),
        ScopeKind::Global => (other, Some(mine.to_path_buf())),
    };
}

/// A plan that keeps the copy for `reason` and moves no link.
fn kept_plan(tool: &LockedTool, candidate: &Candidate, managed: Option<PathBuf>, reason: String) -> OldCopyPlan {
    return OldCopyPlan {
        tool_id: tool.declaration.id.clone(),
        bin: candidate.bin.clone(),
        path: candidate.path.clone(),
        managed,
        repoints: Vec::new(),
        kept: Some(reason),
    };
}

/// Why a copy is kept before any link is looked at,
/// or `None` when the managed copy is good enough to move links to.
fn precondition_reason(
    tool: &LockedTool,
    candidate: &Candidate,
    recorded: Option<&RecordedInstall>,
    managed_version: Option<&str>,
    failed: &[(String, String)],
) -> Option<String> {
    if candidate.is_link {
        return Some(
            "it is a symbolic link, not a copy of the program, so it is left as it is (remove or re-point it by hand if it should go)"
                .to_string(),
        );
    }
    for (id, _) in failed {
        if *id == tool.declaration.id {
            return Some(format!("the managed install of {id} failed in this run, so this may be the only working copy"));
        }
    }
    let Some(version) = managed_version else {
        return Some("there is no finished managed copy yet, so this may be the only working copy".to_string());
    };
    if let Some(install) = recorded
        && compare_versions(&install.version, version) == Ordering::Greater
    {
        return Some(format!(
            "cargo's record says it is {} {}, newer than the managed {version}; upgrade this tree's lock with `mise run toolchain:update` \
             so the newest copy is the managed one",
            install.crate_name, install.version
        ));
    }
    return None;
}

/// The reasons the copy stays after its links were looked at:
/// places that could not be seen or listed,
/// and links no root may take over yet.
fn scan_reasons(places: &Places, scan: &LinkScan, undecided: &[PathBuf]) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    for reason in &places.incomplete {
        reasons.push(format!("a place could not be searched for links: {reason}"));
    }
    for directory in &scan.unreadable {
        reasons.push(format!("{} could not be listed, so links in it are unseen", directory.display()));
    }
    if !undecided.is_empty() {
        let mut names: Vec<String> = Vec::new();
        for link in undecided {
            names.push(link.display().to_string());
        }
        reasons.push(format!(
            "both toolchain roots hold a managed copy, so which one {} should run is the owner's decision (links left as they are)",
            names.join(", ")
        ));
    }
    return reasons;
}

/// What:
/// The plan for one old copy whose preconditions hold:
/// the links in the known places that resolve to it,
/// each with the managed copy it moves to.
/// Why:
/// Reads only.
/// A copy whose links cannot all move,
/// or whose places could not all be searched,
/// is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planLinks(ctx: Context, tool: LockedTool, candidate: Candidate, mine: string, places: Places): OldCopyPlan
/// ```
fn plan_links(ctx: &Context, tool: &LockedTool, candidate: &Candidate, mine: PathBuf, places: &Places) -> OldCopyPlan {
    let other: Option<PathBuf> = match other_root(ctx) {
        Some(root) => managed_copy_under(ctx, tool, &candidate.bin, &root),
        None => None,
    };
    let (repository_copy, global_copy): (Option<PathBuf>, Option<PathBuf>) = copies_by_scope(ctx, &mine, other);
    let scan: LinkScan = links_to_copy(places, &candidate.path);
    let mut repoints: Vec<Repoint> = Vec::new();
    let mut undecided: Vec<PathBuf> = Vec::new();
    for found in &scan.found {
        match target_for(found.kind, repository_copy.as_deref(), global_copy.as_deref()) {
            Target::Use(target) => repoints.push(Repoint { link: found.path.clone(), kind: found.kind, target }),
            Target::Undecided => undecided.push(found.path.clone()),
        }
    }
    let reasons: Vec<String> = scan_reasons(places, &scan, &undecided);
    let kept: Option<String> = if reasons.is_empty() { None } else { Some(reasons.join("; ")) };
    return OldCopyPlan {
        tool_id: tool.declaration.id.clone(),
        bin: candidate.bin.clone(),
        path: candidate.path.clone(),
        managed: Some(mine),
        repoints,
        kept,
    };
}

/// What:
/// The plan for every old copy of the repository-published tools in `ctx.tools`.
/// `failed` is what the install of this run reported as failed,
/// as tool id and message.
/// Why:
/// A tool whose install failed,
/// or that has no working managed copy,
/// keeps its old copy and says why:
/// "never delete an old copy that is the only working copy".
/// A copy that cargo's record attributes to a crate this repository does not publish is another program and is not
/// judged at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function planOldCopies(ctx: Context, failed: [string, string][]): { plans: OldCopyPlan[]; places: Places }
/// ```
pub fn plan_old_copies(ctx: &Context, failed: &[(String, String)]) -> OldCopyPlans {
    let installs: Vec<RecordedInstall> = read_recorded_installs(&cargo_home_dir(ctx.env, &ctx.scope.home));
    let mut plans: Vec<OldCopyPlan> = Vec::new();
    let mut places: Option<Places> = None;
    for tool in ctx.tools {
        if !is_repository_published(tool) || !applies_to_family(&tool.declaration.os, ctx.host.family) {
            continue;
        }
        for candidate in candidates_of(ctx, tool) {
            let recorded: Option<&RecordedInstall> = install_naming_bin(&installs, &candidate.bin);
            if let Some(install) = recorded
                && !install.crate_name.starts_with(OWN_CRATE_PREFIX)
            {
                continue;
            }
            let managed_version: Option<String> = newest_installed_of(&ctx.scope.toolchain_root, tool);
            let mine: Option<PathBuf> = find_tool_bin(tool, &candidate.bin, ctx.scope, ctx.env, &[]);
            if let Some(reason) = precondition_reason(tool, &candidate, recorded, managed_version.as_deref(), failed) {
                plans.push(kept_plan(tool, &candidate, mine, reason));
                continue;
            }
            let Some(managed) = mine else {
                plans.push(kept_plan(tool, &candidate, None, "the managed copy of this bin is missing or not executable".to_string()));
                continue;
            };
            if places.is_none() {
                places = Some(known_places(ctx.scope, ctx.env, ctx.runner));
            }
            if let Some(searched) = places.as_ref() {
                plans.push(plan_links(ctx, tool, &candidate, managed, searched));
            }
        }
    }
    return OldCopyPlans { plans, places: places.unwrap_or_default() };
}
