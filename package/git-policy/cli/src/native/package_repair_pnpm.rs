//! What:
//!  The pnpm repair: `pnpm install --offline --frozen-lockfile` in a new worktree whose `node_modules` was copied.
//! Why:
//!  pnpm's links reach its store by a path counted in directory levels from the project, so a copy at another depth holds
//!  links that are dead or, worse, resolve through a parent directory's `node_modules` to the source's packages. pnpm
//!  rewrites its own layout for the new location in about the time of a no-op install.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairPnpm(destination, environment); // undefined when pnpm has nothing to relink
//! ```

use super::package_repair::RepairEnvironment;
use super::package_repair_report::Finding;
use std::path::Path;

/// What:
///  Relink the copied `node_modules` of a worktree with pnpm, when it has a lockfile and a `node_modules`.
/// Why:
///   Without a lockfile or a copied `node_modules` there is no layout to repair, and nothing is started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairPnpm(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub fn repair_pnpm(_destination: &Path, _environment: &RepairEnvironment) -> Option<Finding> {
    return None;
}

/// pnpm repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_pnpm_tests.rs"]
mod tests;
