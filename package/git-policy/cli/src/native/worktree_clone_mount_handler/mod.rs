//! What:
//!  The root-run clone-mount handler of the ZFS worktree route:
//!  the second executable `mount-git-worktree-clone-datasets`,
//!  its mode grammar,
//!  the two paths it is installed at,
//!  the modes whose plan steps have not landed,
//!  and the administrator's parent adoption,
//!  which is the one mode this build performs.
//! Why:
//!  The design keeps the clone mount out of the Git wrapper:
//!  a root-installed program the ZED event and the boot and shutdown units start
//!  owns attachment,
//!  unmount and the recorded folder identities,
//!  and it shares the binding and the compatibility gate with the wrapper without
//!  becoming a package of its own
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Binding crate and compatibility",
//!  "ZED reconciliation and mount safety";
//!  the build order is `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 5).
//!  Each mode refuses until the plan step that builds it lands,
//!  except the administrator's adoption:
//!  its decision table is step 4's and its end-to-end arm is this step's.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `handler_install_paths.rs`:
//!   the two installed paths,
//!   the configuration's path and the program's own name.
//! - `handler_mode_arguments.rs`:
//!   the mode words,
//!   the adoption's three options,
//!   the refusals and the usage and version lines.
//! - `mode_awaiting_its_step.rs`:
//!   the refusal of a mode whose plan step has not landed.
//! - `attaching_sweep_mode.rs`:
//!   the default invocation's sweep,
//!   which step 8 builds.
//! - `boot_reconcile_mode.rs`:
//!   the boot unit's reconciliation,
//!   which step 6 begins.
//! - `shutdown_unmount_mode.rs`:
//!   the shutdown unit's verified unmount,
//!   which step 9 builds.
//! - `adopt_worktree_parent_mode.rs`:
//!   the administrator's adoption,
//!   decided by step 4's table and recorded here.
//! - `handler_mode_dispatch.rs`:
//!   the command line,
//!   the exit statuses and the lines each mode writes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! process.exit(worktreeCloneMountHandler.run(process.argv.slice(2)));
//! ```

/// The two installed paths,
/// the configuration beside them and the program's own name.
pub mod handler_install_paths;

/// The mode words and the adoption's options.
pub mod handler_mode_arguments;

/// The refusal of a mode whose step has not landed.
pub mod mode_awaiting_its_step;

/// The default invocation:
/// the clone-mount sweep.
pub mod attaching_sweep_mode;

/// The boot unit's reconciliation.
pub mod boot_reconcile_mode;

/// The shutdown unit's verified unmount.
pub mod shutdown_unmount_mode;

/// The administrator's adoption of a replacement parent folder.
pub mod adopt_worktree_parent_mode;

/// The command line and the statuses.
pub mod handler_mode_dispatch;

/// The installed paths and the program's name.
pub use handler_install_paths::{CONFIGURATION_PATH, PROGRAM_NAME, SERVICE_INSTALL_PATH, ZEDLET_INSTALL_PATH};
/// The mode grammar,
/// its refusal and the request an adoption takes.
pub use handler_mode_arguments::{AdoptRequest, ArgumentRefusal, HandlerMode};
/// The refusal of a mode whose step has not landed.
pub use mode_awaiting_its_step::AwaitingItsStep;
/// The administrator's adoption and why it recorded nothing.
pub use adopt_worktree_parent_mode::AdoptFailure;
/// The command line and the two statuses.
pub use handler_mode_dispatch::{EXIT_DONE, EXIT_REFUSED, run};
