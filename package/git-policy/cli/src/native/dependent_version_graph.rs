//! What: Walk the workspace dependency graph from the packages whose version was raised and
//!       plan a patch bump for every publishable package that reaches one of them.
//! Why: A dependent's published contents change when a dependency it ships with changes,
//!      so its version must move too. This is the incumbent's closure and filter
//!      (`dependent-version-bump.ts:183-350`): edges to names outside the workspace and
//!      self edges are ignored, the walk passes through unpublished packages and cycles,
//!      and only publishable, versioned, not-yet-raised packages are bumped.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // planDependentBumps({ manifests, bumpedNames, publishableNames });
//! ```

/// What: `use` brings names from sibling files and the standard library into this file.
///       `HashMap`/`HashSet` are hash-based map and set types.
/// Why:  The plan bumps release versions; edges and visits are looked up by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { patchBumpVersion } from './dependent_version_release.ts';
/// ```
use super::dependent_version_release::{ReleaseBump, UnsupportedVersion, patch_bump_version};
/// Hash-based maps and sets for edges and visits.
use std::collections::{HashMap, HashSet};

/// What: One workspace package as the walk sees it. `Option<Vec<u16>>` is the version as
///       JavaScript code units, or nothing.
/// Why:  Edges are the names this package reaches through runtime fields or bundled
///       development imports; which ones count is decided before the walk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorkspaceManifest = { name: string; directory: string; version?: string; edgeNames: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceNode {
    /// The package name.
    pub name: String,
    /// The package directory, repository-relative, without a trailing slash.
    pub directory: Vec<u8>,
    /// The current version.
    pub version: Option<Vec<u16>>,
    /// Names this package depends on.
    pub edge_names: Vec<String>,
}

/// What: One planned dependent bump.
/// Why:  The plan turns each into a manifest text edit and a finding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PlannedBump = { name: string; directory: string; from: string; to: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedBump {
    /// The dependent's package name.
    pub name: String,
    /// Its directory.
    pub directory: Vec<u8>,
    /// The version before and after.
    pub bump: ReleaseBump,
}

/// What: Every package that transitively depends on one of `bumped_names`.
///       `&[String]` borrows a list of names; the result owns its names.
/// Why:  The walk keeps a work stack of names whose dependents still need visiting, so it
///       ends on cycles and never recurses. A raised package reached from another raised
///       package is in the result; the caller decides what that means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function transitiveDependentNames({ manifests, bumpedNames }): Set<string>;
/// ```
pub fn transitive_dependent_names(
    nodes: &[WorkspaceNode],
    bumped_names: &[String],
) -> HashSet<String> {
    let workspace: HashSet<&str> = nodes
        .iter()
        .map(|node: &WorkspaceNode| return node.name.as_str())
        .collect();
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for node in nodes {
        for edge in &node.edge_names {
            if workspace.contains(edge.as_str()) && *edge != node.name {
                // `.entry(...).or_default()` finds the list for a key, creating an empty one.
                dependents
                    .entry(edge.as_str())
                    .or_default()
                    .push(node.name.as_str());
            }
        }
    }
    let mut pending: Vec<&str> = bumped_names.iter().map(String::as_str).collect();
    let mut reached: HashSet<String> = HashSet::new();
    // `while let Some(...) = pending.pop()` takes names until the stack is empty.
    while let Some(current) = pending.pop() {
        for dependent in dependents.get(current).map_or(&[][..], Vec::as_slice) {
            // `.insert` answers whether the name was new, so each name is expanded once.
            if reached.insert(String::from(*dependent)) {
                pending.push(dependent);
            }
        }
    }
    return reached;
}

/// What: Order names as JavaScript's `<` does, by UTF-16 code units.
///       `std::cmp::Ordering` is less, equal or greater.
/// Why:  Rust compares `String`s by UTF-8 bytes, which orders characters above U+FFFF
///       differently from characters between U+E000 and U+FFFF.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byName = (left: string, right: string) => (left < right ? -1 : left > right ? 1 : 0);
/// ```
pub fn utf16_order(left: &str, right: &str) -> std::cmp::Ordering {
    return left.encode_utf16().cmp(right.encode_utf16());
}

/// What: The patch bumps for publishable dependents of the raised packages, sorted by name.
///       `Result<T, UnsupportedVersion>` is the plan, or the first dependent (in node order)
///       whose version cannot be bumped automatically.
/// Why:  Raised packages already moved; a package without a version has nothing to bump;
///       an unpublished package is never released, though the walk passes through it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function planDependentBumps({ manifests, bumpedNames, publishableNames }): PlannedBump[];
/// ```
pub fn plan_dependent_bumps(
    nodes: &[WorkspaceNode],
    bumped_names: &[String],
    publishable_names: &[String],
) -> Result<Vec<PlannedBump>, UnsupportedVersion> {
    let reached: HashSet<String> = transitive_dependent_names(nodes, bumped_names);
    let mut bumps: Vec<PlannedBump> = Vec::new();
    for node in nodes {
        let needs_bump: bool = reached.contains(&node.name)
            && !bumped_names.contains(&node.name)
            && publishable_names.contains(&node.name);
        // `if let (true, Some(...))` runs only for a node that needs a bump and has a version.
        if let (true, Some(version)) = (needs_bump, &node.version) {
            bumps.push(PlannedBump {
                name: node.name.clone(),
                directory: node.directory.clone(),
                bump: patch_bump_version(&node.name, version)?,
            });
        }
    }
    // `.sort_by` is stable, as `toSorted` is.
    bumps.sort_by(|left: &PlannedBump, right: &PlannedBump| {
        return utf16_order(&left.name, &right.name);
    });
    return Ok(bumps);
}

/// The closure and plan, including every case of the incumbent's unit tests.
#[cfg(test)]
#[path = "dependent_version_graph_tests.rs"]
mod tests;
