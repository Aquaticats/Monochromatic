//! What:
//! Which tools count as this repository's own:
//! a `cargo-binstall` tool whose crate name is in the repository's crate namespace,
//! and an `archive` tool of this repository's releases whose tag prefix starts the same way.
//! Why:
//! The newest-only layout,
//! the never-compile rule and the older-lock warnings apply to these tools and to no others,
//! and the answer comes from the plan entry alone,
//! so no second list can drift from the crates and the releases.
//! The predicates live here,
//! so the file that installs and deletes versions and the file that prints the sentence both ask one place.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export function isRepositoryPublished(tool: LockedTool): boolean
//! export function isOwnCrateName(name: string): boolean
//! ```

/// Imports `OWN_CRATE_PREFIX` from `crate::constants`.
use crate::constants::OWN_CRATE_PREFIX;

/// Imports what this module uses from `super::super::model`.
use super::super::model::{
    LockedTool,
    Route,
};
/// Imports `package_name` from `super::super::observe`.
use super::super::observe::package_name;
/// Imports `is_published_archive` from `super::super::published_archive`.
use super::super::published_archive::is_published_archive;

/// What:
/// Whether a tool is published by this repository:
/// a `cargo-binstall` tool whose crate name starts with `monochromatic-`,
/// or an `archive` tool of the request `github:Aquaticats/Monochromatic` (in any letter case)
/// whose `options.tagPrefix` starts with it
/// (a release archive of this repository,
/// such as `mpm-shadow` itself).
/// Why:
/// This repository names every crate it publishes that way (`OWN_CRATE_PREFIX`,
/// the same rule that limits the registry prune to its own crates) and tags every release with the same start,
/// so the plan needs no second list that could drift from the crates.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isRepositoryPublished(tool: LockedTool): boolean
/// ```
pub fn is_repository_published(tool: &LockedTool) -> bool {
    if is_published_archive(tool) {
        return true;
    }
    return tool.declaration.route == Route::CargoBinstall
        && is_own_crate_name(&package_name(tool));
}

/// What:
/// Whether a crate name is in this repository's crate namespace:
/// it starts with `OWN_CRATE_PREFIX` once letter case and `_` versus `-` are ignored.
/// Why:
/// crates.io treats `Monochromatic_X` and `monochromatic-x` as one crate,
/// so a spelling variant of a repository crate must not be taken for a third-party one
/// (a third-party crate may fall back to compiling;
/// a repository crate never does).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isOwnCrateName(name: string): boolean
/// ```
pub fn is_own_crate_name(name: &str) -> bool {
    let canonical: String = name.to_ascii_lowercase().replace('_', "-");
    return canonical.starts_with(OWN_CRATE_PREFIX);
}
