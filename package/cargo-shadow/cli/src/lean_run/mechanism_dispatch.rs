//! What:
//! The dispatch of one mechanism of a run:
//! the arm each mechanism has,
//! and the report it answers with.
//! Why:
//! The run walks the order once here,
//! so the order has one place to change,
//! and the two long passes start after the position the last run recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function runMechanism(mechanism: Mechanism, context: RunContext): MechanismReport
//! ```
/// Imports the mechanism this dispatch switches on.
use super::mechanisms_and_options::Mechanism;
/// Imports what a mechanism reports and what it is given.
use super::report_and_context::{
    MechanismReport,
    RunContext,
};
/// Imports the listing of the profile directories the units pass walks.
use super::unit_profile_listing::unit_profiles;
/// Imports the tree a reported position is named by.
use crate::lean_build_trees::Tree;
/// Imports the long pass and the position the last run stopped at.
use crate::lean_progress::{
    LongPass,
    resume_index,
};
/// Imports the name this file uses from `crate::lean_run_discovery`.
use crate::lean_run_discovery::{
    target_directories,
    verification_directories,
};
/// Imports the steps of the mechanisms this file dispatches.
use crate::lean_run_steps::{
    UnitsPass,
    run_podman,
    run_records,
    run_toolchains,
    sweep_and_cap,
};
/// Imports the tree listing and the report of the idle-trees pass.
use crate::lean_run_trees::{
    tree_report,
    trees_in_order,
};
/// Imports the owned path type the recorded positions are keys of.
use std::path::PathBuf;
/// What:
/// `pub(crate) fn run_mechanism(mechanism: Mechanism, context: &RunContext) -> MechanismReport` runs one mechanism and returns
/// its summary lines and where the next run continues.
/// Why:
/// `run_lean` walks `RUN_ORDER` and each mechanism is one arm here,
/// so the order has one place to change,
/// and the two long passes start after the position the last run recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runMechanism(mechanism: Mechanism, context: RunContext): MechanismReport
/// ```
pub(crate) fn run_mechanism(
    mechanism: Mechanism,
    context: &RunContext,
) -> MechanismReport {
    return match mechanism {
        Mechanism::Trees => match context.base {
            Some(found) => {
                let trees: Vec<Tree> = trees_in_order(found);
                let mut keys: Vec<PathBuf> = Vec::with_capacity(trees.len());
                for tree in &trees {
                    keys.push(tree.path.clone());
                }
                let start: usize = resume_index(&keys, context.progress.after(LongPass::Trees));
                tree_report(
                    &trees,
                    start,
                    context.options,
                    &context.deadline,
                    context.environment,
                    found,
                    context.pinned,
                )
            }
            None => MechanismReport {
                lines: vec![String::from(
                    "idle trees: no build.build-dir with {workspace-path-hash} in the Cargo configuration, so there is no host build base",
                )],
                resume_after: None,
            },
        },
        Mechanism::Units => {
            // Listing the profile directories reads every tree and every target directory,
            // and the deadline only starts to apply once a profile directory is judged.
            let listing_began: std::time::Instant = std::time::Instant::now();
            let profiles: Vec<(PathBuf, u64)> = unit_profiles(context.base, context.checkouts);
            let listing_seconds: u64 = listing_began.elapsed().as_secs();
            let mut keys: Vec<PathBuf> = Vec::with_capacity(profiles.len());
            for (profile, _cap) in &profiles {
                keys.push(profile.clone());
            }
            let start: usize = resume_index(&keys, context.progress.after(LongPass::Units));
            let chosen: &[(PathBuf, u64)] = &profiles[start..];
            let UnitsPass {
                mut lines,
                judged,
            } = sweep_and_cap(
                chosen,
                context.options.dry_run,
                &context.deadline,
                context.environment,
            );
            let last_judged: Option<&(PathBuf, u64)> = if judged > 0 {
                Some(&chosen[judged - 1])
            } else {
                None
            };
            let resume_after: Option<PathBuf> = match (judged < chosen.len(), last_judged) {
                (true, Some(last)) => Some(last.0.clone()),
                (true, None) if start > 0 => Some(profiles[start - 1].0.clone()),
                _ => None,
            };
            lines.insert(
                1,
                format!(
                    "  listed {} profile directories in {} s",
                    profiles.len(),
                    listing_seconds
                ),
            );
            // The position notes come before the notes of what was skipped or left alone.
            if let (true, Some(last)) = (judged < chosen.len(), last_judged) {
                lines.insert(1, format!("  the deadline passed after {judged} of {} profile directories, at {}; the next run continues after it", chosen.len(), last.0.display()));
            }
            if start > 0 {
                lines.insert(1, format!("  this pass started after {}, the profile directory the last run stopped at", profiles[start - 1].0.display()));
            }
            MechanismReport {
                lines,
                resume_after,
            }
        }
        Mechanism::Records => {
            let mut targets: Vec<PathBuf> = Vec::new();
            for checkout in context.checkouts {
                targets.extend(target_directories(checkout));
            }
            MechanismReport {
                lines: run_records(
                    &verification_directories(&targets),
                    context.options.dry_run,
                    &context.deadline,
                    context.environment,
                ),
                resume_after: None,
            }
        }
        Mechanism::Toolchains => MechanismReport {
            lines: run_toolchains(
                &context.rustup_home.join("toolchains"),
                context.pinned,
                context.options.dry_run,
                &context.deadline,
                context.environment,
            ),
            resume_after: None,
        },
        Mechanism::Podman => {
            let main: PathBuf = match crate::lean_checkouts::main_checkout(&context.options.start) {
                Some(found) => found,
                None => match context.checkouts.first() {
                    Some(first) => first.clone(),
                    None => {
                        return MechanismReport {
                            lines: vec![String::from(
                                "podman: no checkout was found, so nothing was judged",
                            )],
                            resume_after: None,
                        };
                    }
                },
            };
            MechanismReport {
                lines: run_podman(&main, context.options.dry_run, &context.deadline),
                resume_after: None,
            }
        }
    };
}
