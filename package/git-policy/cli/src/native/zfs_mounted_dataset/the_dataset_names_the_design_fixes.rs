//! What:
//!  The three dataset names the design fixes:
//!  the leaf name of the checkout dataset,
//!  the container the design places beside it,
//!  and the probe dataset under the pool that holds the checkout.
//! Why:
//!  The lookups that find a dataset by these names and the words of a refusal
//!  that found none both name them,
//!  so they sit in one file both read rather than in either one,
//!  and neither of those two files has to use the other
//!  (`AGENTS.md` rule LYF;
//!  `doc/planning/cli-git-source-layout-target-tree.md`,
//!  "Loops between files,
//!  and how each is broken (2026-11-26)").
//!  The names are the design's,
//!  not a caller's:
//!  with no configuration file the checkout is the one mounted ZFS dataset
//!  whose last name component is
//!  `monochromatic-git-checkout`,
//!  the container is the sibling
//!  `monochromatic-git-worktree-clone-datasets`,
//!  and the probe dataset is `nosnap/git-worktree-probe` in the pool that holds
//!  the checkout
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Dataset and path layout";
//!  `doc/planning/cli-git-zfs-start-probe-design.md`,
//!  "The source:
//! the probe dataset";
//!  and the project's answers of 2026-11-22 in
//!  `doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const DESIGN_CHECKOUT_DATASET_NAME = 'monochromatic-git-checkout';
//! const DESIGN_CONTAINER_DATASET_NAME = 'monochromatic-git-worktree-clone-datasets';
//! const DESIGN_PROBE_DATASET_UNDER_THE_POOL = 'nosnap/git-worktree-probe';
//! ```

/// The leaf name the design gives the checkout dataset.
pub const DESIGN_CHECKOUT_DATASET_NAME: &str = "monochromatic-git-checkout";

/// The leaf name the design gives the container of managed clones.
pub const DESIGN_CONTAINER_DATASET_NAME: &str = "monochromatic-git-worktree-clone-datasets";

/// The probe dataset's name inside the pool that holds the checkout,
/// which the start probe's design fixes.
pub const DESIGN_PROBE_DATASET_UNDER_THE_POOL: &str = "nosnap/git-worktree-probe";
