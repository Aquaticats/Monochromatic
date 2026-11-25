//! What:
//!  The boot and shutdown units of the ZFS route,
//! as the setup writes them:
//!  the gate that reconciles clone mounts after the checkout is ready,
//!  its stop-failure drop-in,
//!  the provider drop-in that asks the gate to reconcile again,
//!  the propagation service and the mount drop-in that start it,
//!  one unit per dataset whose mountpoint lies inside the checkout,
//!  and the list of files a setup run writes.
//! Why:
//!  ZED alone cannot cover boot and shutdown,
//! so the route installs a
//!  `Type=oneshot` unit whose `ExecStart` reconciles and whose `ExecStopPost`
//!  unmounts what this boot verified
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//!  The owner's answers of 2026-11-24 fix what that unit requires and orders
//!  after,
//! which drop-ins it needs beside it,
//! and the two time bounds
//!  (`doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
//!  V1 to V9 and V8b);
//!  the propagation pair comes from the accepted propagation record
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "Which triggers the route adds").
//!  The texts live in the release code rather than only in a runbook so the
//!  host can parse and assert every ordering line,
//! which is this step's host
//!  half of the plan's split
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  "The measurements the guest carries and the host keeps").
//!  Nothing here starts,
//! stops or reads a service manager:
//!  the module builds text,
//! and a setup run or a guest's own manager acts on it.
//!
//! Files in this folder,
//! each using only the files listed before it:
//!
//! - `boot_unit_names.rs`:
//!   the unit and drop-in names,
//!   the install paths under `/etc/systemd/system`,
//!   the owner's two bounds,
//!   and the mount unit name a checkout path escapes to.
//! - `boot_unit_facts.rs`:
//!   the values a setup fills into the texts,
//!   the refusal that names an unusable one,
//!   and one file of the install plan.
//! - `unit_file_text.rs`:
//!   the one final newline and the comment naming a file's own path.
//! - `boot_gate_unit_text.rs`:
//!   the gate unit and the two drop-ins beside it.
//! - `propagation_unit_text.rs`:
//!   the propagation service and the mount drop-in that starts it.
//! - `nested_dataset_unit_text.rs`:
//!   one nested dataset's mount unit.
//! - `boot_unit_install_plan.rs`:
//!   every file the setup writes,
//!   in the order it writes them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = installPlan(facts); // [{ path: '/etc/systemd/system/…', text }]
//! ```

pub mod boot_gate_unit_text;
pub mod boot_unit_facts;
pub mod boot_unit_install_plan;
pub mod boot_unit_names;
pub mod nested_dataset_unit_text;
pub mod unit_file_text;
pub mod propagation_unit_text;

pub use boot_gate_unit_text::{boot_gate_unit, provider_restart_drop_in, stop_failure_drop_in};
pub use boot_unit_facts::{CheckoutBootFacts, InstalledUnitFile, NestedDatasetMount, UnitTextRefusal};
pub use boot_unit_install_plan::install_plan;
pub use boot_unit_names::{
    BOOT_GATE_UNIT_NAME, BOOT_GATE_UNIT_PATH, BOOT_RECONCILE_MODE_WORD, EXECUTABLE_SYSTEMCTL, GATE_START_BOUND, GATE_STOP_BOUND, NESTED_UNIT_NAME_PREFIX, NESTED_UNIT_NAME_SUFFIX,
    PROPAGATION_MOUNT_DROP_IN_NAME, PROPAGATION_SERVICE_NAME, PROVIDER_DROP_IN_NAME, PROVIDER_UNIT_NAME, SHUTDOWN_UNMOUNT_MODE_WORD, START_PROBE_MODE_WORD, STOP_FAILURE_DROP_IN_NAME,
    STOP_FAILURE_MODE_WORD, SYSTEM_UNIT_DIRECTORY, drop_in_path, mount_unit_name, nested_unit_name, propagation_service_path, root_words, unit_path,
};
pub use nested_dataset_unit_text::{DATASET_OPTION, MOUNT_PATH_DESCENDANT_MODE_WORD, nested_dataset_unit};
pub use unit_file_text::{file_text, path_comment};
pub use propagation_unit_text::{propagation_mount_drop_in, propagation_service};

/// The units' arm in a throwaway guest:
/// every file written at the path systemd reads,
/// parsed by `systemd-analyze verify`,
/// and read back from the manager.
/// Never part of the release executable,
/// and it prints `SKIPPED` anywhere but the step 19 guest.
#[cfg(test)]
#[path = "boot_unit_guest_tests.rs"]
mod guest_tests;

/// The reading every unit text's control shares:
/// the lines of one INI section,
/// so an assertion names the section it read.
/// Never part of the release executable.
#[cfg(test)]
#[path = "unit_text_sections_tests.rs"]
pub(crate) mod unit_text_sections;
