//! What:
//! Plan step 19's shutdown pass,
//!  built ahead of the unit that runs it:
//!  withdraw this boot's readiness,
//!  close admission to new attaches,
//!  and unmount only the mounts this boot verified,
//!  without changing durable intent.
//! Why:
//!  The export command unmounts datasets before it issues the export,
//!  and the export event is posted too late to order those unmounts,
//!  so the shutdown pass is a systemd unit and not a ZED event
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  whose `ExecStopPost` runs `zfs-root shutdown-unmount`).
//!  The mode that will run this pass refuses in this build,
//!  and no command reaches this folder:
//!  `crate::worktree_clone_mount_handler::shutdown_unmount_mode` is still the
//!  stub that names the unit,
//!  and the round that wires it calls `run_the_pass` from there
//! (`doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`:
//!  "A piece built ahead merges as code no person's command reaches.
//!  Its own step wires it in,
//!  in the plan's order,
//!  and that wiring is where the step's "Depends on" still holds").
//!  So the code is proved two ways and reached by neither:
//!  value tests that decide every branch on values a control states,
//!  and one guest arm that calls `run_the_pass` as root against a real
//!  file-backed pool,
//!  its checkout,
//!  its clone and its mounts.
//!
//!  What the pass reads to know which mounts this boot verified:
//! this boot's readiness record,
//!  which `boot-reconcile` publishes and which
//!  names the boot,
//!  the pool and the checkout it found ready
//!  (`crate::worktree_clone_mount_protocol::clone_mount_readiness`);
//!  the root-owned configuration the boot unit's own `ExecStart` read;
//! this process's mount table,
//!  whose rows name the dataset each mount shows;
//! the route's own record of the `.claude/worktrees` folder,
//!  which says the
//!  slots still hang where the route put them
//!  (`crate::worktree_clone_mount::slot_identity_store`);
//! and,
//!  for each mount,
//!  a fresh walk of its slot and the descriptor that walk
//!  opens.
//! A mount no record of this boot proves is not unmounted,
//!  and the line it
//!  answers says which fact was missing.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `shutdown_unmount_places.rs`:
//!   every place the pass reads and writes,
//!   and this boot's own identifier.
//! - `shutdown_unmount_sources.rs`:
//!   the calls the pass makes,
//!   each a trait with a live implementation.
//! - `shutdown_unmount_request.rs`:
//!   one pass's places and calls as one value.
//! - `shutdown_unmount_answer.rs`:
//!   what one mount's turn did,
//!   what the whole pass did,
//!   and the lines it answers.
//! - `the_boots_readiness.rs`:
//!   the withdrawal of this boot's record and what that record proves.
//! - `unmountable_mounts.rs`:
//!   the mounts the pass may look at.
//! - `one_mounts_checks.rs`:
//!   the comparisons that prove one mount is the clone's own at its slot.
//! - `one_mounts_unmount.rs`:
//!   one mount's turn:
//!   its claim,
//!   its checks,
//!   its unmount and its busy answer.
//! - `shutdown_unmount_turn.rs`:
//!   the whole pass in the order the design gives.
//! - the folder's value tests,
//!   one file beside each module and named for it,
//!   and the two files they share,
//!   `shutdown_unmount_pass_test_fixture_values.rs` and
//!   `shutdown_unmount_pass_test_doubles.rs`,
//!   which are never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = runThePass(request);
//! outcome.lines(); // ['shutdown-unmount in boot …: readiness …', 'shutdown-unmount: unmounted slot …']
//! ```

/// The places the pass reads and writes.
pub mod shutdown_unmount_places;

/// The calls the pass makes,
///  each a seam.
pub mod shutdown_unmount_sources;

/// One pass's places and calls as one value.
pub mod shutdown_unmount_request;

/// What one mount's turn and the whole pass answered.
pub mod shutdown_unmount_answer;

/// This boot's record,
///  its withdrawal,
///  and what it proves.
pub mod the_boots_readiness;

/// The mounts the pass may look at.
pub mod unmountable_mounts;

/// The comparisons that prove one mount.
pub mod one_mounts_checks;

/// One mount's whole turn.
pub mod one_mounts_unmount;

/// The whole pass in the design's order.
pub mod shutdown_unmount_turn;

/// The values the folder's value tests state,
///  never part of the release executable.
#[cfg(test)]
pub(crate) mod shutdown_unmount_pass_test_fixture_values;

/// The doubles the folder's value tests share,
///  never part of the release executable.
#[cfg(test)]
pub(crate) mod shutdown_unmount_pass_test_doubles;

pub use shutdown_unmount_answer::PassOutcome;
pub use shutdown_unmount_places::ShutdownUnmountPlaces;
pub use shutdown_unmount_request::ShutdownUnmountRequest;
pub use shutdown_unmount_sources::{
    HolderScanning,
    LiveDatasetGuids,
    LiveHolderScans,
    LiveSlotLookups,
    LiveWorktreesFolder,
    SlotLookups,
    WorktreesFolder,
};
pub use shutdown_unmount_turn::run_the_pass;
