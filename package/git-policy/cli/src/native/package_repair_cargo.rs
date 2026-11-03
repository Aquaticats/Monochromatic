//! What:
//!  The Cargo repair: for each Cargo workspace of a new worktree, clone the source workspace's build state when Cargo keeps
//!  it outside the worktree, then `cargo clean --package <member>` for every member so only the worktree's own crates
//!  rebuild.
//! Why:
//!  Cargo bakes absolute paths into what it builds (`CARGO_MANIFEST_DIR`, build script output), so build state copied from
//!  another location is wrong for the worktree's own crates, but registry and git dependencies hold nothing path-specific
//!  and can be reused. The owner chose this on 2026-11-03: "Reuse deps, rebuild own", done with "Run cargo clean -p".
//!  Where `build-dir` is outside the worktree (the person's `~/.cargo/config.toml` keys it by workspace path) a copy of
//!  ignored files does not carry it, so it is cloned; under Cargo's default layout it is inside `target/`, which is already
//!  copied. `cargo metadata` names both directories, so no configuration is read by the wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const findings = await repairCargo(request); // one per workspace that failed, one for the rest that worked
//! ```

use super::package_repair::RepairRequest;
use super::package_repair_report::Finding;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  One Cargo workspace as `cargo metadata --no-deps` describes it.
/// Why:
///   The root identifies the workspace, the two directories say where its state lives, and the members are what gets cleaned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Workspace = { root: string; buildDirectory: string; targetDirectory: string; members: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workspace {
    /// The directory holding the workspace's root manifest.
    pub root: PathBuf,
    /// Where intermediate build state lives.
    pub build_directory: PathBuf,
    /// Where final outputs live.
    pub target_directory: PathBuf,
    /// The package names of the workspace members.
    pub members: Vec<String>,
}

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
    /// Nothing is cloned or cleaned; the text says why, for the log.
    Nothing(String),
    /// The state is already in the worktree (copied with `target/`): only the members are cleaned.
    CleanOnly,
    /// The source's external build directory is cloned to the worktree's, then the members are cleaned.
    CloneThenClean {
        /// The source workspace's build directory.
        from: PathBuf,
        /// The worktree's build directory, which does not exist yet.
        to: PathBuf,
    },
}

/// What:
///  Facts about the disk that decide a plan.
/// Why:
///   They are read once, so the planner stays pure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StateFacts = { sourceBuild?: string; sourceBuildExists: boolean; destinationBuildExists: boolean; destinationTargetExists: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateFacts {
    /// The build directory of the source's counterpart workspace; none when the source has no such workspace.
    pub source_build: Option<PathBuf>,
    /// Whether that directory exists.
    pub source_build_exists: bool,
    /// Whether the worktree's build directory exists.
    pub destination_build_exists: bool,
    /// Whether the worktree's target directory exists.
    pub destination_target_exists: bool,
}

/// What:
///  Read the workspace out of `cargo metadata --no-deps --format-version 1` output.
/// Why:
///   Only these four facts are used, and any other shape is an error that names what is missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseMetadata(text: string): Workspace; // throws
/// ```
pub fn parse_metadata(_text: &str) -> Result<Workspace, String> {
    return Err(String::from("not written yet"));
}

/// What:
///  The workspaces once each, in root order.
/// Why:
///   Cargo describes a workspace the same way from any of its members' manifests, so several manifests of one workspace
///  must not be repaired several times.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function distinctWorkspaces(found: Workspace[]): Workspace[];
/// ```
pub fn distinct_workspaces(found: Vec<Workspace>) -> Vec<Workspace> {
    return found;
}

/// What:
///  Decide what the worktree's build state needs.
/// Why:
///   Reusing state that is shared with the source, or cleaning where there is none, would do harm or nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planBuildState(destination: Workspace, destinationRoot: string, facts: StateFacts): BuildStatePlan;
/// ```
pub fn plan_build_state(_destination: &Workspace, _destination_root: &Path, _facts: &StateFacts) -> BuildStatePlan {
    return BuildStatePlan::Nothing(String::from("not written yet"));
}

/// What:
///  The arguments of the clean that removes exactly the members' own artifacts.
/// Why:
///   `--locked --offline` keeps the clean from changing `Cargo.lock` or reaching the network; one `--package` per member
///  keeps registry and git dependencies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanArguments(members: string[]): string[];
/// ```
pub fn clean_arguments(_members: &[String]) -> Vec<OsString> {
    return Vec::new();
}

/// What:
///  Repair every Cargo workspace of the new worktree.
/// Why:
///   Workspaces are independent (each has its own build directory), so they are repaired side by side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairCargo(request: RepairRequest): Promise<Finding[]>;
/// ```
pub fn repair_cargo(_request: &RepairRequest<'_>) -> Vec<Finding> {
    return Vec::new();
}

/// Cargo repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_cargo_tests.rs"]
mod tests;
