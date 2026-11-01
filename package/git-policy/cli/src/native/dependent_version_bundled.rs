//! What: Decide which development dependencies each dependent bundles, reading source
//!       only where an edge could carry a raised version.
//! Why: A development dependency ships inside a dependent only when its non-test source
//!      imports it. Scanning every package's source on every commit would read the whole
//!      workspace, so, as in the incumbent (`dependent-bump-workflow.ts:176-278`), only
//!      dependents that could reach a raised package if every development dependency were
//!      bundled are scanned, and only for edges that lead toward a raised package.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await bundledDevelopmentEdges({ reader, states, bumpedNames });
//! ```

/// What: `use` brings names from sibling files and the standard library into this file.
/// Why:  The superset walk is the plan's own walk; source paths and imports have their
///       own modules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { transitiveDependentNames } from './dependent_version_graph.ts';
/// ```
use super::dependent_version_content::{ContentUnavailable, TrackedPath, WorkspaceContent};
/// The superset walk over every edge.
use super::dependent_version_graph::{WorkspaceNode, transitive_dependent_names};
/// The specifier scan.
use super::dependent_version_imports::imports_package;
/// Which tracked paths are a package's non-test source.
use super::dependent_version_paths::is_non_test_source_path;
/// One manifest's facts and directory.
use super::dependent_version_states::ManifestState;
/// A hash-based set of relevant names.
use std::collections::HashSet;

/// What: The development dependencies of one dependent that could carry a raised version.
/// Why:  An edge to a package outside the superset and the raised set leads nowhere; a self
///       edge is never an edge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// state.current.devDependencyNames.filter(name => relevant.has(name) && name !== state.current.name);
/// ```
fn candidate_edges(state: &ManifestState, relevant: &HashSet<String>) -> Vec<String> {
    return state
        .facts
        .dev_dependency_names
        .iter()
        .filter(|name: &&String| return relevant.contains(*name) && **name != state.facts.name)
        .cloned()
        .collect();
}

/// What: The texts of a package's non-test source files. `String::from_utf8_lossy`
///       replaces bytes that are not UTF-8 with U+FFFD, as the incumbent's lenient decoder
///       does.
/// Why:  A source file only needs scanning for specifiers, so undecodable bytes are not a
///       failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await Promise.all((await reader.sourceFiles(directory)).filter(isSource).map(file => file.text()));
/// ```
fn source_texts(
    content: &mut dyn WorkspaceContent,
    tracked: &[TrackedPath],
    directory: &[u8],
) -> Result<Vec<String>, ContentUnavailable> {
    let mut texts: Vec<String> = Vec::new();
    for entry in tracked {
        if is_non_test_source_path(directory, &entry.path) {
            let bytes: Vec<u8> = content.candidate_bytes(&entry.path)?;
            texts.push(String::from_utf8_lossy(&bytes).into_owned());
        }
    }
    return Ok(texts);
}

/// What: For each state, in order, the development dependencies its source imports among
///       those that could carry a raised version. `Vec<Vec<String>>` is one list per state.
/// Why:  These become edges of the plan's walk alongside the runtime edges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function bundledDevelopmentEdges({ reader, states, bumpedNames }): Map<string, string[]>;
/// ```
pub fn bundled_development_edges(
    content: &mut dyn WorkspaceContent,
    tracked: &[TrackedPath],
    states: &[ManifestState],
    bumped_names: &[String],
) -> Result<Vec<Vec<String>>, ContentUnavailable> {
    let every_edge: Vec<WorkspaceNode> = states
        .iter()
        .map(|state: &ManifestState| {
            return WorkspaceNode {
                name: state.facts.name.clone(),
                directory: state.directory.clone(),
                version: None,
                edge_names: [
                    state.facts.runtime_dependency_names.as_slice(),
                    state.facts.dev_dependency_names.as_slice(),
                ]
                .concat(),
            };
        })
        .collect();
    let mut relevant: HashSet<String> = transitive_dependent_names(&every_edge, bumped_names);
    relevant.extend(bumped_names.iter().cloned());
    let mut bundled: Vec<Vec<String>> = Vec::new();
    for state in states {
        // The incumbent also skips a dependent outside the superset. That test is implied:
        // an edge to a relevant workspace package puts the dependent in the superset.
        let candidates: Vec<String> = candidate_edges(state, &relevant);
        if candidates.is_empty() {
            bundled.push(Vec::new());
            continue;
        }
        let texts: Vec<String> = source_texts(content, tracked, &state.directory)?;
        bundled.push(
            candidates
                .into_iter()
                .filter(|name: &String| {
                    return texts
                        .iter()
                        .any(|text: &String| return imports_package(text, name));
                })
                .collect(),
        );
    }
    return Ok(bundled);
}

/// Which dependents are scanned, which edges are confirmed, and which files are read.
#[cfg(test)]
#[path = "dependent_version_bundled_tests.rs"]
mod tests;
