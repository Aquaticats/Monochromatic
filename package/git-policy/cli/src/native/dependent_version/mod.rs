//! What:
//!  The dependent-version policy:
//!  manifest and import facts,
//!  the release plan and the policy over them.
//! Why:
//!  One policy with many small helpers is easier to read as a folder than as a prefix.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // dependent_version/index.ts: one `export * from './file'` per module below.
//! ```

/// The repository content the dependent-version planner reads,
/// and its two failure kinds.
pub mod dependent_version_content;

/// Which paths the dependent-version planner reads:
/// manifests,
/// configuration,
/// source.
pub mod dependent_version_paths;

/// Strict JSON reading of a manifest's name,
/// version and dependency names.
pub mod dependent_version_manifest;

/// Patch bumps of plain release versions and `JSON.stringify` quoting.
pub mod dependent_version_release;

/// The byte-preserving rewrite of a manifest's top-level version.
pub mod dependent_version_text;

/// Publishable package names from the generated registry configuration.
pub mod dependent_version_publishable;

/// The block YAML subset the registry configuration is read with:
/// a mapping,
/// a sequence and a scalar.
pub mod dependent_version_yaml;

/// One line of that YAML:
/// its indentation,
/// its key,
/// and its scalar.
pub mod dependent_version_yaml_lines;

/// Whether source text imports a package.
pub mod dependent_version_imports;

/// The dependency walk and the plan of dependent bumps.
pub mod dependent_version_graph;

/// Every workspace manifest at both states,
/// and which versions were raised.
pub mod dependent_version_states;

/// Which development dependencies each dependent bundles.
pub mod dependent_version_bundled;

/// The dependent-version plan with exact manifest bytes before and after.
pub mod dependent_version_plan;

/// The decision and findings of `mono/dependent-version-bump`.
pub mod dependent_version_policy;

/// The dependent-version planner's view of a prepared candidate state.
pub mod dependent_version_lifecycle;

/// Whether a commit or a direct fix may change a tracked file it did not select.
pub mod policy_added_path;

/// The commit path's notes,
///  real-index listing and corrections.
pub mod policy_dependent_version_commit;

/// The `mono/dependent-version-bump` check over a lifecycle's candidate state.
pub mod policy_dependent_version;

/// An in-memory workspace that records reads;
/// never in the release build.
#[cfg(test)]
pub(crate) mod dependent_version_test_support;

/// Differential fixture reading and canonical results;
/// never in the release build.
#[cfg(test)]
pub(crate) mod dependent_version_fixture_json;

/// Canonical renderings of the native planner's results;
/// never in the release build.
#[cfg(test)]
pub(crate) mod dependent_version_fixture_output;

/// One differential case evaluated by the native planner;
/// never in the release build.
#[cfg(test)]
pub(crate) mod dependent_version_fixture_cases;

/// The native planner against the incumbent's recorded results;
/// never in the release build.
#[cfg(test)]
mod dependent_version_differential_tests;
