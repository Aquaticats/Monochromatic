//! What:
//!  What one workspace's build state needs in the new worktree,
//! decided from what the disk holds:
//! nothing,
//! a move of the state the copy carried,
//! or a filling of it from the source checkout.
//! Why:
//!  The decision is pure,
//! so each layout is tested without Cargo,
//! and it stays apart from the repair that carries it out:
//! a plan a caller reads alone says why a workspace needs nothing.
//! The two types are the answer and the facts it is made from,
//! and `plan_build_state` is the one rule that turns the second into the first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type BuildStatePlan = { kind: 'nothing'; reason: string } | { kind: 'moved-aside'; to: string } | { kind: 'fill-from-the-source'; from: string; to: string };
//! type StateFacts = { sourceHasTheWorkspace: boolean; sourceBuildExists: boolean; destinationBuildExists: boolean; destinationBuildIsInside: boolean; destinationTargetIsInside: boolean };
//! function planBuildState(destination: Workspace, destinationRoot: string, facts: StateFacts): BuildStatePlan;
//! ```

/// Import the Cargo workspace whose build state is planned.
use crate::package_repair_cargo_workspaces::Workspace;
/// Import the borrowed and owned filesystem paths a plan names.
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  What a workspace's build state needs in the new worktree.
/// Why:
///   The decision is pure so each layout is tested without Cargo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BuildStatePlan = { kind: 'nothing'; reason: string } | { kind: 'clean-only' } | { kind: 'clone-then-clean'; from: string; to: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuildStatePlan {
    /// Nothing is cloned or cleaned;
    /// the text says why,
    /// for the log.
    Nothing(String),
    /// The worktree holds build state of its own,
    /// copied with `target/`:
    /// the add moves it aside in one rename and the service cleans it there,
    /// so no build can trust the copy's artifacts and the add asks Cargo nothing.
    MovedAside {
        /// The worktree's build directory,
        /// which the add moves to the filling's staging name.
        to: PathBuf,
    },
    /// The source's external build directory is filled into the worktree's by the commit service,
    /// and the members are cleaned there,
    /// so the add does not wait for the clone.
    FillFromTheSource {
        /// The source workspace's build directory.
        from: PathBuf,
        /// The worktree's build directory,
        /// which does not exist yet.
        to: PathBuf,
    },
}

/// What:
///  Facts about the disk that decide a plan.
/// Why:
///   They are read once,
/// so the planner stays pure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateFacts = { sourceHasTheWorkspace: boolean; sourceBuild?: string; sourceBuildExists: boolean; destinationBuildExists: boolean; destinationBuildIsInside: boolean; destinationTargetIsInside: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateFacts {
    /// Whether the source checkout has a counterpart workspace at all;
    /// one walk of the tracked manifests answers it,
    /// without asking Cargo anything.
    pub source_has_the_workspace: bool,
    /// The build directory of the source's counterpart workspace;
    /// read only where the worktree's own build directory is absent,
    /// because only a fill uses it.
    pub source_build: Option<PathBuf>,
    /// Whether that directory exists.
    pub source_build_exists: bool,
    /// Whether the worktree's build directory exists.
    pub destination_build_exists: bool,
    /// Whether the worktree's build directory is still inside the worktree once its links are followed.
    pub destination_build_is_inside: bool,
    /// Whether the worktree's target directory is still inside the worktree once its links are followed.
    pub destination_target_is_inside: bool,
}

/// What:
///  Decide what the worktree's build state needs.
/// Why:
///   Reusing state that is shared with the source,
/// or cleaning where there is none,
/// would do harm or nothing.
/// A clean runs only where the worktree's own build directory holds state a build could reuse:
/// Cargo keeps the fingerprints that make artifacts fresh inside that directory,
/// so a worktree whose build directory is absent rebuilds its own crates by itself,
/// which was measured on 2026-11-12 with a copy of a built workspace whose `target/` held the
///  source's binary (the copy's first build rebuilt it and named the copy's own path).
/// A copied `target/` alone is therefore no reason to clean,
/// and on the host whose Cargo keeps the build directory outside the worktree it made every one of
///  an add's 34 cleans a full workspace resolution that deleted nothing
/// (`doc/planning/cli-git-worktree-add-package-repair-off-the-waiting-path.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planBuildState(destination: Workspace, destinationRoot: string, facts: StateFacts): BuildStatePlan;
/// ```
pub fn plan_build_state(
    destination: &Workspace,
    destination_root: &Path,
    facts: &StateFacts,
) -> BuildStatePlan {
    if destination.build_directory.starts_with(destination_root) {
        // A copied `target/` can be a link to a directory outside the worktree, and a clean through it would delete the
        //  files of the checkout that link names, which this repair does not serve.
        if !facts.destination_build_is_inside || !facts.destination_target_is_inside {
            return BuildStatePlan::Nothing(String::from(
                "the build state is reached through a link that leaves the worktree, and a clean there would delete another directory's files",
            ));
        }
        // What a clean here would remove is the copy's state for the worktree's own crates, so nothing was carried when the
        //  source has no counterpart workspace.
        if !facts.source_has_the_workspace {
            return BuildStatePlan::Nothing(String::from(
                "the source has no such workspace, so no state was carried",
            ));
        }
        return if facts.destination_build_exists {
            BuildStatePlan::MovedAside {
                to: destination.build_directory.clone(),
            }
        } else {
            BuildStatePlan::Nothing(String::from(
                "the build directory is inside the worktree and was not carried",
            ))
        };
    }
    if facts.destination_build_exists {
        return BuildStatePlan::Nothing(String::from(
            "the worktree's build directory already holds state of its own",
        ));
    }
    let Some(source_build) = facts.source_build.as_ref() else {
        return BuildStatePlan::Nothing(String::from(
            "the source holds no build directory this repair could read, so there is nothing to fill",
        ));
    };
    if *source_build == destination.build_directory {
        return BuildStatePlan::Nothing(String::from(
            "the source and the worktree share one build directory, which a clean would damage",
        ));
    }
    if facts.source_build_exists {
        return BuildStatePlan::FillFromTheSource {
            from: source_build.clone(),
            to: destination.build_directory.clone(),
        };
    }
    return BuildStatePlan::Nothing(String::from(
        "the worktree's build directory does not exist and the source holds none for it, so a clean would reach nothing a build could reuse",
    ));
}
