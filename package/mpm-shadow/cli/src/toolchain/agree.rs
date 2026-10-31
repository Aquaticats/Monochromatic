//! What:
//! The one validator for "do the plan and the lock agree":
//! it compares the plan's declarations with the
//!           lock's,
//! sorts every difference into what offline `lock` may fix and what needs `update`,
//! and checks the
//!           lock's digest.
//! Why:
//! `lock`,
//! `lock --check`,
//! `sync`,
//! `status` and `update` all need the same verdict,
//! so they share this
//!           module;
//! `sync` and `status` refuse on any disagreement and name the command that resolves it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // agree.ts
//! export function compare(plan: Plan, parsed: ParsedLock): Disagreement { ... }
//! export function project(plan: Plan, lock: Lock): Lock { ... }   // offline, resolution kept
//! ```

/// Imports `ToolchainError` from `super::error`.
use super::error::ToolchainError;
/// Imports what this module uses from `super::lock`.
use super::lock::{Lock, ParsedLock, lock_digest};
/// Imports what this module uses from `super::model`.
use super::model::{Declaration, LockedTool, expand_version};
/// Imports `Plan` from `super::plan`.
use super::plan::Plan;

/// What:
/// Everything that differs between a plan and a lock.
/// Why:
/// Each list is a different remedy:
/// `added` and `resolution_edits` need `update`;
/// `removed`,
///           `projection_edits` and `order_differs` are fixed offline by `lock`;
/// `digest_problem` needs the file
///           restored or deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Disagreement = { added: string[]; removed: string[]; resolutionEdits: [string, string[]][];
///   projectionEdits: [string, string[]][]; orderDiffers: boolean; digestProblem?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Disagreement {
    /// Tool ids the plan declares and the lock lacks.
    pub added: Vec<String>,
    /// Tool ids the lock holds and the plan no longer declares.
    pub removed: Vec<String>,
    /// Tool id and the changed fields that affect which version or artifact is resolved.
    pub resolution_edits: Vec<(String, Vec<String>)>,
    /// Tool id and the changed fields that only shape installs and links.
    pub projection_edits: Vec<(String, Vec<String>)>,
    /// Whether the lock lists its tools in a different order than the plan.
    pub order_differs: bool,
    /// Why the lock's digest does not match its contents,
    /// or `None` when it does.
    pub digest_problem: Option<String>,
}

/// The methods of `Disagreement`.
impl Disagreement {
    /// Whether the plan and the lock agree completely.
    pub fn is_empty(&self) -> bool {
        return self.added.is_empty()
            && self.removed.is_empty()
            && self.resolution_edits.is_empty()
            && self.projection_edits.is_empty()
            && !self.order_differs
            && self.digest_problem.is_none();
    }

    /// Whether only `update` (the network) can resolve the difference.
    pub fn needs_update(&self) -> bool {
        return !self.added.is_empty() || !self.resolution_edits.is_empty();
    }

    /// The ids only `update` can resolve:
    /// the tools the lock lacks and the tools with a resolution-affecting edit,
    /// in plan order of discovery.
    pub fn ids_needing_update(&self) -> Vec<String> {
        let mut ids: Vec<String> = self.added.clone();
        for (id, _) in &self.resolution_edits {
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        return ids;
    }

    /// What:
    /// One sentence per difference,
    /// each naming the tool and the field.
    /// Why:
    /// The text goes straight to the person,
    /// who must know which file to open.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// describe(): string[]
    /// ```
    pub fn describe(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        if let Some(problem) = &self.digest_problem {
            lines.push(problem.clone());
        }
        for id in &self.added {
            lines.push(format!("{id} is in the plan but not in the lock"));
        }
        for id in &self.removed {
            lines.push(format!("{id} is in the lock but no longer in the plan"));
        }
        for (id, fields) in &self.resolution_edits {
            lines.push(format!("{id} changed in the plan in a way that moves its version or artifacts: {}", fields.join(", ")));
        }
        for (id, fields) in &self.projection_edits {
            lines.push(format!("{id} differs between the plan and the lock in: {}", fields.join(", ")));
        }
        if self.order_differs {
            lines.push("the lock lists its tools in a different order than the plan".to_string());
        }
        return lines;
    }
}

/// What:
/// The names of the declaration fields that differ between `plan` and `locked`,
/// split into the ones that
///           change which version or artifacts are resolved (`resolution`) and the ones that only shape installs.
/// Why:
/// The split is the rule "offline `lock` rejects additions and resolution-affecting edits (pins,
/// package id,
///           route,
/// artifact selection,
/// platforms)":
/// request,
/// route,
/// manager,
/// packageId,
/// os,
/// assetPattern,
/// cargoGit
///           and bootstrap are resolution-affecting;
/// bins,
/// env and the remaining options are not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function changedFields(plan: Declaration, locked: Declaration): { resolution: string[]; projection: string[] }
/// ```
fn changed_fields(plan: &Declaration, locked: &Declaration) -> (Vec<String>, Vec<String>) {
    let mut resolution: Vec<String> = Vec::new();
    let mut projection: Vec<String> = Vec::new();
    if plan.request != locked.request {
        resolution.push("request".to_string());
    }
    if plan.route != locked.route {
        resolution.push("route".to_string());
    }
    if plan.manager != locked.manager {
        resolution.push("manager".to_string());
    }
    if plan.package_id != locked.package_id {
        resolution.push("packageId".to_string());
    }
    if plan.os != locked.os {
        resolution.push("os".to_string());
    }
    if plan.options.asset != locked.options.asset {
        resolution.push("options.asset".to_string());
    }
    if plan.options.bootstrap != locked.options.bootstrap {
        resolution.push("options.bootstrap".to_string());
    }
    if plan.bins != locked.bins {
        projection.push("bins".to_string());
    }
    if plan.env != locked.env {
        projection.push("env".to_string());
    }
    if plan.options.rust_components != locked.options.rust_components {
        projection.push("options.rustComponents".to_string());
    }
    if plan.options.rust_targets != locked.options.rust_targets {
        projection.push("options.rustTargets".to_string());
    }
    if plan.options.cargo_features != locked.options.cargo_features {
        projection.push("options.cargoFeatures".to_string());
    }
    if plan.options.allow_builds != locked.options.allow_builds {
        projection.push("options.allowBuilds".to_string());
    }
    if plan.options.android_packages != locked.options.android_packages {
        projection.push("options.androidPackages".to_string());
    }
    if plan.options.postinstall != locked.options.postinstall {
        projection.push("options.postinstall".to_string());
    }
    if plan.options.bin_aliases != locked.options.bin_aliases {
        projection.push("options.binAliases".to_string());
    }
    return (resolution, projection);
}

/// Whether the plan declares a tool with this id.
fn plan_has(plan: &Plan, id: &str) -> bool {
    for declaration in &plan.tools {
        if declaration.id == id {
            return true;
        }
    }
    return false;
}

/// What:
/// Compares a plan with a parsed lock and returns every difference,
/// including a digest that does not match
///           the lock's contents.
/// Why:
/// This is the single verdict behind `lock --check`,
/// `lock`,
/// `sync`,
/// `status` and `update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function compare(plan: Plan, parsed: ParsedLock): Disagreement
/// ```
pub fn compare(plan: &Plan, parsed: &ParsedLock) -> Disagreement {
    let lock: &Lock = &parsed.lock;
    let mut result: Disagreement = Disagreement::default();
    match &parsed.recorded_digest {
        None => {
            result.digest_problem = Some(
                "toolchain.lock.json has no digest, so it was not written by mpm-shadow".to_string(),
            );
        }
        Some(recorded) => {
            let expected: String = lock_digest(lock);
            if *recorded != expected {
                result.digest_problem = Some(format!(
                    "toolchain.lock.json does not match its digest (recorded {recorded}, contents hash to {expected}); \
                     it was edited by hand or by another tool"
                ));
            }
        }
    }
    for declaration in &plan.tools {
        let Some(locked) = lock.tool(&declaration.id) else {
            result.added.push(declaration.id.clone());
            continue;
        };
        let expanded: Declaration = expand_version(declaration, &locked.version);
        let (resolution, projection) = changed_fields(&expanded, &locked.declaration);
        if !resolution.is_empty() {
            result.resolution_edits.push((declaration.id.clone(), resolution));
        }
        if !projection.is_empty() {
            result.projection_edits.push((declaration.id.clone(), projection));
        }
    }
    for locked in &lock.tools {
        if !plan_has(plan, &locked.declaration.id) {
            result.removed.push(locked.declaration.id.clone());
        }
    }
    let mut plan_order: Vec<&str> = Vec::new();
    for declaration in &plan.tools {
        if lock.tool(&declaration.id).is_some() {
            plan_order.push(declaration.id.as_str());
        }
    }
    let mut lock_order: Vec<&str> = Vec::new();
    for locked in &lock.tools {
        if plan_has(plan, &locked.declaration.id) {
            lock_order.push(locked.declaration.id.as_str());
        }
    }
    result.order_differs = plan_order != lock_order;
    return result;
}

/// What:
/// Builds the lock the plan implies without touching the network:
/// every plan tool in plan order,
/// with the
///           plan's declaration and the version and artifacts the old lock already holds.
/// Why:
/// This is offline `lock`.
/// It is only defined when nothing needs resolving,
/// so a tool the lock lacks or a
///           resolution-affecting edit is an error that names `update`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function project(plan: Plan, lock: Lock): Lock  // throws when update is needed
/// ```
pub fn project(plan: &Plan, lock: &Lock) -> Result<Lock, ToolchainError> {
    let mut tools: Vec<LockedTool> = Vec::new();
    for declaration in &plan.tools {
        let Some(locked) = lock.tool(&declaration.id) else {
            return Err(ToolchainError::drift(format!(
                "{} is in the plan but not in the lock; run `mpm-shadow update {}` to resolve just that tool \
                 (every other row keeps its version)",
                declaration.id,
                declaration.id
            )));
        };
        let expanded: Declaration = expand_version(declaration, &locked.version);
        let (resolution, _) = changed_fields(&expanded, &locked.declaration);
        if !resolution.is_empty() {
            return Err(ToolchainError::drift(format!(
                "{} changed in the plan in a way that moves its version or artifacts ({}); \
                 run `mpm-shadow update {}` to resolve just that tool (every other row keeps its version)",
                declaration.id,
                resolution.join(", "),
                declaration.id
            )));
        }
        tools.push(LockedTool {
            declaration: expanded,
            version: locked.version.clone(),
            artifacts: locked.artifacts.clone(),
        });
    }
    return Ok(Lock { tools });
}

/// What:
/// The error `sync` and `status` return when the plan and lock disagree:
/// every difference,
/// and the command
///           that resolves it.
/// Why:
/// Acting on a lock that does not describe the plan would install the wrong tools,
/// so these commands refuse
///           and say which of `lock` or `update` fixes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function refusal(disagreement: Disagreement): ToolchainError
/// ```
pub fn refusal(disagreement: &Disagreement) -> ToolchainError {
    let remedy: String = if disagreement.digest_problem.is_some() {
        "restore toolchain.lock.json from version control, or delete it and run `mpm-shadow update`".to_string()
    } else if disagreement.needs_update() {
        format!(
            "run `mpm-shadow update {}` to resolve just those tools (it needs the network; every other row keeps its \
             version), or `mpm-shadow update` to resolve every tool",
            disagreement.ids_needing_update().join(" ")
        )
    } else {
        "run `mpm-shadow lock` (it works offline)".to_string()
    };
    let mut message: String = String::from("the toolchain plan and lock disagree:\n");
    for line in disagreement.describe() {
        message.push_str("  ");
        message.push_str(&line);
        message.push('\n');
    }
    message.push_str(&remedy);
    return ToolchainError::drift(message);
}
