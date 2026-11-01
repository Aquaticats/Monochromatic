//! What: Plan patch bumps for publishable dependents of packages whose version differs from
//!       the base, with the exact manifest bytes before and after each bump.
//! Why: This is the one planner behind both entry points: the commit-time policy and the
//!      release workflow's ripple, which will call the native wrapper's direct fix. It is
//!      the incumbent's `planWorkspaceBumps` (`dependent-bump-workflow.ts:280-388`) over the
//!      content seam: it starts no Git process and reads no file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const plan = await planWorkspaceBumps(reader);
//! ```

/// What: `use` brings names from sibling files into this file.
/// Why:  Each step of the plan has its own module.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readManifestStates, bumpedNames } from './dependent_version_states.ts';
/// ```
use super::dependent_version_bundled::bundled_development_edges;
/// The content seam and its failures.
use super::dependent_version_content::{PlanError, TrackedMode, TrackedPath, WorkspaceContent};
/// The dependency walk and the bump plan.
use super::dependent_version_graph::{PlannedBump, WorkspaceNode, plan_dependent_bumps};
/// Strict decoding of the registry configuration.
use super::dependent_version_manifest::strict_text;
/// The registry configuration's path.
use super::dependent_version_paths::PNPR_CONFIG_PATH;
/// The publish set from the registry configuration.
use super::dependent_version_publishable::read_publishable_names;
/// A dependent whose version cannot be bumped automatically.
use super::dependent_version_release::UnsupportedVersion;
/// Manifest states, raised names and duplicate names.
use super::dependent_version_states::{
    ManifestState, bumped_names, duplicate_name, read_manifest_states,
};
/// The byte-preserving version rewrite.
use super::dependent_version_text::replace_manifest_version;

/// What: One dependent bump with the manifest change that performs it. `Vec<u8>` holds
///       exact file bytes.
/// Why:  The policy turns it into a full-content patch of a tracked file; the release path
///       writes `replacement` after checking the file still holds `original`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManifestBump = PlannedBump & { path: string; text: string; replacement: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestBump {
    /// The dependent, its directory and the version change.
    pub planned: PlannedBump,
    /// The manifest path.
    pub path: Vec<u8>,
    /// Its mode in the candidate state.
    pub mode: TrackedMode,
    /// Its exact current bytes.
    pub original: Vec<u8>,
    /// The same bytes with only the version value changed; a leading byte-order mark is
    /// kept.
    pub replacement: Vec<u8>,
}

/// What: A completed plan.
/// Why:  The finding message names the raised packages; the bumps are sorted by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkspaceBumpPlan = { bumpedNames: string[]; bumps: ManifestBump[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceBumpPlan {
    /// Packages whose version differs from the base, in manifest order.
    pub bumped_names: Vec<String>,
    /// Dependent bumps, sorted by name.
    pub bumps: Vec<ManifestBump>,
}

/// What: How planning ended when nothing failed.
/// Why:  A dependent that needs a bump but holds no plain release stops the plan; the
///       policy reports it for a bump by hand, the release path fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlanOutcome = { kind: 'planned'; plan: WorkspaceBumpPlan } | { kind: 'unsupported'; error: UnsupportedVersionError };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlanOutcome {
    /// Every needed bump was planned; the list may be empty.
    Planned(
        /// The plan.
        WorkspaceBumpPlan,
    ),
    /// The first dependent, in manifest order, whose version cannot be bumped.
    Unsupported(
        /// The dependent and its version.
        UnsupportedVersion,
    ),
}

/// What: The publishable names from the registry configuration, or nothing when it is not
///       tracked.
/// Why:  Without the configuration nothing is publishable, so nothing is bumped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [configText] = await reader.pnprConfigText();
/// ```
fn publishable_names(
    content: &mut dyn WorkspaceContent,
    tracked: &[TrackedPath],
) -> Result<Option<Vec<String>>, PlanError> {
    let Some(config) = tracked
        .iter()
        .find(|entry: &&TrackedPath| return entry.path == PNPR_CONFIG_PATH)
    else {
        return Ok(None);
    };
    let bytes: Vec<u8> = content.candidate_bytes(&config.path)?;
    return Ok(Some(read_publishable_names(strict_text(
        &config.path,
        &bytes,
    )?)));
}

/// What: The manifest change for one planned bump.
/// Why:  The edit rewrites the parsed text; the original bytes before it (a byte-order
///       mark) are kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const replacement = replaceManifestVersion({ path, text, from, to });
/// ```
fn manifest_bump(planned: PlannedBump, state: &ManifestState) -> Result<ManifestBump, PlanError> {
    let edited: String = replace_manifest_version(
        &state.path,
        &state.text,
        &planned.bump.from,
        &planned.bump.to,
    )?;
    let prefix: &[u8] = &state.bytes[..state.bytes.len() - state.text.len()];
    return Ok(ManifestBump {
        planned,
        path: state.path.clone(),
        mode: state.mode,
        original: state.bytes.clone(),
        replacement: [prefix, edited.as_bytes()].concat(),
    });
}

/// What: Plan patch bumps for publishable dependents of raised packages.
/// Why:  Reads every manifest at both states; the registry configuration only when a
///       version was raised; source files only of dependents whose development edges could
///       carry a raised version.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function planWorkspaceBumps(reader: WorkspaceFileReader): Promise<WorkspaceBumpPlan>;
/// ```
pub fn plan_workspace_bumps(content: &mut dyn WorkspaceContent) -> Result<PlanOutcome, PlanError> {
    let tracked: Vec<TrackedPath> = content.tracked_paths()?;
    let states: Vec<ManifestState> = read_manifest_states(content, &tracked)?;
    let bumped: Vec<String> = bumped_names(&states);
    let empty = |bumped_names: Vec<String>| {
        return Ok(PlanOutcome::Planned(WorkspaceBumpPlan {
            bumped_names,
            bumps: Vec::new(),
        }));
    };
    if bumped.is_empty() {
        return empty(bumped);
    }
    let Some(publishable) = publishable_names(content, &tracked)? else {
        return empty(bumped);
    };
    // `if let Some(...)` returns the first duplicate as the failure.
    if let Some(duplicate) = duplicate_name(&states) {
        return Err(PlanError::PolicyIncomplete(duplicate));
    }
    let bundled: Vec<Vec<String>> = bundled_development_edges(content, &tracked, &states, &bumped)?;
    // `.zip` pairs each state with its bundled edges.
    let nodes: Vec<WorkspaceNode> = states
        .iter()
        .zip(bundled)
        .map(|(state, edges): (&ManifestState, Vec<String>)| {
            return WorkspaceNode {
                name: state.facts.name.clone(),
                directory: state.directory.clone(),
                version: state.facts.version.clone(),
                edge_names: [state.facts.runtime_dependency_names.clone(), edges].concat(),
            };
        })
        .collect();
    let planned: Vec<PlannedBump> = match plan_dependent_bumps(&nodes, &bumped, &publishable) {
        Ok(found) => found,
        Err(unsupported) => return Ok(PlanOutcome::Unsupported(unsupported)),
    };
    let mut bumps: Vec<ManifestBump> = Vec::new();
    for bump in planned {
        // Names are unique once the duplicate check passed, so this finds the one state.
        if let Some(state) = states
            .iter()
            .find(|state: &&ManifestState| return state.facts.name == bump.name)
        {
            bumps.push(manifest_bump(bump, state)?);
        }
    }
    return Ok(PlanOutcome::Planned(WorkspaceBumpPlan {
        bumped_names: bumped,
        bumps,
    }));
}

/// The plan over scripted workspaces, including the incumbent's worktree ripple cases.
#[cfg(test)]
#[path = "dependent_version_plan_tests.rs"]
mod tests;
