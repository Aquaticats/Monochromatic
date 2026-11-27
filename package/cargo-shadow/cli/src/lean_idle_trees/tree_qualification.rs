//! What:
//! The decision that one tree of the host build base qualifies for removal.
//! Why:
//! The decision is its own file so it can be read and tested alone,
//! apart from the removal it authorizes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function qualifies(tree: Tree, workspace: Workspace, retired: Set<string>, policy: IdlePolicy, now: number, activity: number): { ok: [string, number] } | { no: string }
//! ```
/// Imports the idle periods this decision applies.
use super::idle_policy_and_verdicts::IdlePolicy;
/// Imports the tree to judge,
/// its kind and what identifies the workspace it was built for.
use crate::lean_build_trees::{
    Tree,
    TreeKind,
    Workspace,
};
/// Imports the check that an entry's absence is confirmed before its record counts as gone.
use crate::lean_identity::missing;
/// Imports the set of names the retired toolchains are held in.
use std::collections::HashSet;
/// What:
/// `pub(super) fn qualifies(tree: &Tree, workspace: &Workspace, retired: &HashSet<String>, policy: &IdlePolicy, now: i64, activity: i64) -> Result<(String, i64), String>`
/// decides from the facts whether a tree qualifies:
/// `Ok((reason, idle period that applied))` when it does,
/// `Err(why not)` when it does not.
/// Why:
/// The decision is separate from the removal so it can be read and tested alone,
/// and the workspace it was read once for the whole pass is handed in:
/// reading it again here would repeat the most expensive part of the pass for every tree it judges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function qualifies(tree: Tree, workspace: Workspace, retired: Set<string>, policy: IdlePolicy, now: number, activity: number): { ok: string } | { no: string }
/// ```
pub(super) fn qualifies(
    tree: &Tree,
    workspace: &Workspace,
    retired: &HashSet<String>,
    policy: &IdlePolicy,
    now: i64,
    activity: i64,
) -> Result<(String, i64), String> {
    let idle: i64 = now - activity;
    let toolchain_retired: bool = match &tree.kind {
        TreeKind::Keyed(name) => retired.contains(name),
        TreeKind::Legacy => false,
    };
    let (needed, why): (i64, String) = match workspace {
        _ if toolchain_retired => (
            policy.idle_seconds,
            String::from("its toolchain is retired"),
        ),
        Workspace::Known(root) if !missing(root) => {
            return Err(format!(
                "its workspace {} exists or cannot be shown to be gone",
                root.display()
            ));
        }
        Workspace::Known(root) => (
            policy.idle_seconds,
            format!("its workspace {} is gone", root.display()),
        ),
        Workspace::Lead(source) if !missing(source) => {
            return Err(format!(
                "the source {} that its dependency information names exists or cannot be shown to be gone",
                source.display()
            ));
        }
        Workspace::Lead(source) => (
            policy.idle_seconds,
            format!(
                "the source {} that its dependency information names is gone (a lead, not proof)",
                source.display()
            ),
        ),
        Workspace::Unidentified => (
            policy.unidentified_idle_seconds,
            String::from("nothing identifies its workspace"),
        ),
    };
    if idle <= needed {
        return Err(format!(
            "idle for {idle} s, within the {needed} s period ({why})"
        ));
    }
    return Ok((
        format!("{why}; idle for {idle} s, longer than {needed} s"),
        needed,
    ));
}
