//! What:
//!  The paths of one migration and the refusals that stop it before anything is created:
//! every refusal names its cause
//!  and every remedy that applies.
//! Why:
//!  Converting a checkout swaps its directory under every process and worktree,
//! so the migration starts only when the
//!  filesystem,
//! the checkout and the state left by an earlier run allow it.
//! The conditions are those of the proven
//!  script (`doc/handover/cli-git-subvolume-migration-script.md`,
//! "Refusals") with the rest of the owner's rules:
//!  nothing is spawned,
//! and a refusal is text a person can act on.
//! A condition checks only metadata (the dry run reads
//!  `.git` and `.git/worktrees/*/gitdir`,
//! and lists every directory of the tree once to count its entries for the free-space
//!  check,
//! without opening a file).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusals: { cause: string; remedy: string }[] = preflight({ platform, layout, journal });
//! ```
//!
//! The code of this module is split by role,
//! and this file re-exports every name those files hold,
//! so that a caller writes `subvolume_preflight::migrate_refusals` as it did before:
//! `subvolume_preflight_paths` holds the paths of a migration and the space it needs,
//! `subvolume_preflight_refusal` the refusal itself,
//! `subvolume_preflight_git_state_refusals` what Git's own state forbids,
//! `subvolume_preflight_migration_refusals` the conditions of a fresh or resumed migration,
//! and `subvolume_preflight_rollback_refusals` the rollback and the deletion of the old tree.

pub use crate::subvolume_preflight_git_state_refusals::IN_PROGRESS_NAMES;
pub use crate::subvolume_preflight_migration_refusals::migrate_refusals;
pub use crate::subvolume_preflight_paths::{
    Layout,
    METADATA_BYTES_PER_ENTRY,
    METADATA_COPIES,
    layout_of,
    required_free_bytes,
};
pub use crate::subvolume_preflight_refusal::Refusal;
pub use crate::subvolume_preflight_rollback_refusals::{
    INSTALLED_ROOT,
    installed_identity,
    remove_old_refusals,
    rollback_refusals,
    wrong_root_refusal,
};

/// Preflight controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_preflight_tests.rs"]
mod tests;
