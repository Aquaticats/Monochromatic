//! What: Read every workspace manifest at the candidate state and at the base, and decide
//!       which packages' versions were raised.
//! Why: The plan starts from these facts. The order of failures follows the incumbent's:
//!      every manifest is decoded before any is parsed (its reader decodes while listing,
//!      `dependent-version-bump-policy.ts:82-92`), and each manifest's base text is parsed
//!      before its current text (`dependent-bump-workflow.ts:150-174`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const states = (await reader.manifests()).map(readManifestState);
//! ```

/// What: `use` brings names from sibling files into this file.
/// Why:  Manifests are selected by path, read through the content seam, and parsed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readManifestFacts, strictText } from './dependent_version_manifest.ts';
/// ```
use super::dependent_version_content::{
    PlanError, PolicyIncomplete, TrackedMode, TrackedPath, WorkspaceContent,
};
/// Strict decoding and fact extraction of one manifest.
use super::dependent_version_manifest::{ManifestFacts, read_manifest_facts, strict_text};
/// Which tracked paths are manifests, and their package directories.
use super::dependent_version_paths::{is_workspace_manifest_path, package_directory};

/// What: One workspace manifest with what the plan needs from it.
/// Why:  The patch is built from the exact current bytes; the text after an optional
///       byte-order mark is what was parsed and what the version edit rewrites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ManifestState = { file: WorkspaceManifestFile; directory: string; current: ManifestDependencyFacts; baseVersion?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestState {
    /// The manifest path.
    pub path: Vec<u8>,
    /// Its mode in the candidate state.
    pub mode: TrackedMode,
    /// The package directory.
    pub directory: Vec<u8>,
    /// The exact candidate bytes.
    pub bytes: Vec<u8>,
    /// The decoded candidate text, without a leading byte-order mark.
    pub text: String,
    /// Facts parsed from that text.
    pub facts: ManifestFacts,
    /// The version at the base; nothing for a new manifest or one without a version there.
    pub base_version: Option<Vec<u16>>,
}

/// What: A manifest's bytes before decoding.
/// Why:  Every manifest is read and decoded before any is parsed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawManifest = { tracked: TrackedPath; current: Uint8Array; base?: Uint8Array };
/// ```
struct RawManifest {
    /// The tracked manifest.
    tracked: TrackedPath,
    /// Candidate bytes.
    current: Vec<u8>,
    /// Base bytes, when the base has the manifest.
    base: Option<Vec<u8>>,
}

/// What: Read and decode every workspace manifest, then parse each, base first.
///       `&mut dyn WorkspaceContent` lends any provider for writing; `?` returns a failure
///       to the caller.
/// Why:  A malformed manifest at either state stops the plan, as in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readManifestStates(content, tracked): ManifestState[];
/// ```
pub fn read_manifest_states(
    content: &mut dyn WorkspaceContent,
    tracked: &[TrackedPath],
) -> Result<Vec<ManifestState>, PlanError> {
    let mut raw: Vec<RawManifest> = Vec::new();
    for entry in tracked {
        if is_workspace_manifest_path(&entry.path) {
            raw.push(RawManifest {
                tracked: entry.clone(),
                current: content.candidate_bytes(&entry.path)?,
                base: content.base_bytes(&entry.path)?,
            });
        }
    }
    for manifest in &raw {
        strict_text(&manifest.tracked.path, &manifest.current)?;
        if let Some(base) = &manifest.base {
            strict_text(&manifest.tracked.path, base)?;
        }
    }
    let mut states: Vec<ManifestState> = Vec::new();
    for manifest in raw {
        states.push(manifest_state(manifest)?);
    }
    return Ok(states);
}

/// What: Parse one decoded manifest at both states.
/// Why:  The base contributes only its version; the current text contributes every fact.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readManifestState(file: WorkspaceManifestFile): ManifestState;
/// ```
fn manifest_state(manifest: RawManifest) -> Result<ManifestState, PolicyIncomplete> {
    let path: &[u8] = &manifest.tracked.path;
    let mut base_version: Option<Vec<u16>> = None;
    if let Some(base) = &manifest.base {
        base_version = read_manifest_facts(path, strict_text(path, base)?)?.version;
    }
    let text: String = String::from(strict_text(path, &manifest.current)?);
    let facts: ManifestFacts = read_manifest_facts(path, &text)?;
    return Ok(ManifestState {
        path: path.to_vec(),
        mode: manifest.tracked.mode,
        directory: package_directory(path).to_vec(),
        text,
        facts,
        base_version,
        bytes: manifest.current,
    });
}

/// What: The names of packages whose version differs from the base, in manifest order.
/// Why:  A manifest new at this state, or without a version at the base, raised nothing;
///       a version removed at this state counts as raised, as in the incumbent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// states.filter(s => s.baseVersion !== undefined && s.current.version !== s.baseVersion).map(s => s.current.name);
/// ```
pub fn bumped_names(states: &[ManifestState]) -> Vec<String> {
    return states
        .iter()
        .filter(|state: &&ManifestState| {
            return state.base_version.is_some() && state.facts.version != state.base_version;
        })
        .map(|state: &ManifestState| return state.facts.name.clone())
        .collect();
}

/// What: The first pair of manifests, in order, that declare the same package name.
/// Why:  With two packages of one name, a dependency edge cannot say which it means; the
///       planner refuses rather than choosing one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function duplicateName(states: ManifestState[]): PolicyIncomplete | undefined;
/// ```
pub fn duplicate_name(states: &[ManifestState]) -> Option<PolicyIncomplete> {
    for (index, later) in states.iter().enumerate() {
        // `&states[..index]` borrows every state before this one.
        if let Some(earlier) = states[..index]
            .iter()
            .find(|state: &&ManifestState| return state.facts.name == later.facts.name)
        {
            return Some(PolicyIncomplete::DuplicateName {
                name: later.facts.name.clone(),
                first: earlier.path.clone(),
                second: later.path.clone(),
            });
        }
    }
    return None;
}

/// Manifest reading order, raised-version detection and duplicate names.
#[cfg(test)]
#[path = "dependent_version_states_tests.rs"]
mod tests;
