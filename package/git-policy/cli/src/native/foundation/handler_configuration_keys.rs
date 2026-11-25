//! What:
//!  The keys the root-owned handler configuration holds,
//!  in the one place the wrapper and the root modes read them from.
//! Why:
//!  The sweep and the start probe read the same file
//!  (`/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc`,
//!  `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety"),
//!  and the project's first answer of 2026-11-22 has the probe read the sweep's
//!  clone-container key as well as its own object
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "Four further answers on the look-up with no configuration file").
//!  A spelling that two modules repeat can drift,
//!  so both name these constants instead,
//!  and a control in each program's own tests compares what it reads with them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const CLONE_MOUNTS_KEY = 'cloneMounts'; export const PARENT_KEY = 'parent';
//! ```

/// The object the sweep's three names sit in.
pub const CLONE_MOUNTS_KEY: &str = "cloneMounts";

/// The key naming the pinned clone parent dataset,
/// which is the container every managed clone is a child of.
pub const PARENT_KEY: &str = "parent";

/// The key naming the checkout the walk starts from.
pub const CHECKOUT_KEY: &str = "checkout";

/// The key naming the checkout dataset,
/// whose snapshots the design's add route clones.
pub const CHECKOUT_DATASET_KEY: &str = "checkoutDataset";

/// The object the start probe's own names sit in,
/// which the sweep reads only to refuse two different checkouts.
pub const START_PROBE_KEY: &str = "startProbe";

/// The key inside `startProbe` naming the probe dataset.
pub const PROBE_DATASET_KEY: &str = "dataset";

/// The key inside `startProbe` naming the checkout the probe opens.
pub const PROBE_CHECKOUT_KEY: &str = "checkout";

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_configuration_keys_tests.rs"]
mod tests;
