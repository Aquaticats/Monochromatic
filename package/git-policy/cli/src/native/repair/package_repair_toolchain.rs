//! What:
//!  The toolchain repair:
//! when the launchers copied into a new worktree's `.cache/toolchain-launchers` still name another
//!  checkout's `toolchain.lock.json`,
//! rebuild the worktree's own links and launchers with `mpm-shadow install
//!  --frozen-lockfile`.
//! Why:
//!  mpm-shadow owns `toolchain.plan.jsonc`,
//! `toolchain.lock.json` and the pinned tools.
//! Its exposed links and launcher records
//!  hold absolute paths,
//! so a copy would run the source checkout's tool versions and guard,
//! and a branch that changes the
//!  lock would silently use the source's.
//! The install is lock-constrained:
//! it relinks what is installed and,
//! when this
//!  worktree's lock names tools that are not installed,
//! downloads them.
//! The owner chose that on 2026-11-03 ("Install online
//!  during the add"),
//! knowing an add can then download and can retire tool copies another checkout still links.
//! The summary
//!  line mpm-shadow prints says what was installed.
//! A failed install never fails the add:
//! it prints the reason and
//!  `mise run toolchain:install`.
//! A launcher record whose name the worktree's own lock declares nowhere is a copy of a layout an earlier release
//!  had:
//! no install rewrites it,
//! so the relink removes it instead of reporting a stale record that no install can ever answer,
//! and the success is counted over the records the lock does declare.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairToolchain(destination, environment); // undefined when the launchers already name this worktree
//! ```
//!
//! Files in this module,
//! each using only the files listed before it:
//!
//! - `package_repair_toolchain_names_and_bounds.rs`:
//!   the launcher directory,
//!   the record extension,
//!   the lock file's name and key,
//!   the program and its own link,
//!   the root variable,
//!   the log tag,
//!   the two byte bounds and the controls' slots.
//! - `the_commands_a_repair_prints.rs`:
//!   the command a person runs to finish the repair.
//! - `package_repair_toolchain_object_identity.rs`:
//!   the object a name holds,
//!   and whether a record names another lock or another object.
//! - `package_repair_toolchain_launcher_records.rs`:
//!   the one reading of the launcher directory:
//!   what is stale,
//!   and what no install rewrites.
//! - `package_repair_toolchain_copy_removal.rs`:
//!   the removal that proves each record and launcher it deletes.
//! - `package_repair_toolchain_install.rs`:
//!   the install,
//!   its report and the two faces that wait for it.

/// The removal of the records and launchers no install rewrites.
#[path = "package_repair_toolchain_copy_removal.rs"]
mod copy_removal;
/// The install and the two faces that wait for it.
#[path = "package_repair_toolchain_install.rs"]
mod install;
/// The one reading of a worktree's launcher directory both faces decide from.
#[path = "package_repair_toolchain_launcher_records.rs"]
mod launcher_records;
/// The words and bounds this module reads and prints with.
#[path = "package_repair_toolchain_names_and_bounds.rs"]
mod names_and_bounds;
/// The objects a launcher record names.
#[path = "package_repair_toolchain_object_identity.rs"]
mod object_identity;

/// What the add needs of the toolchain repair.
pub(crate) use self::install::{
    ToolchainNeed,
    need_within_the_budget,
};
/// The toolchain repair,
/// for a caller on its own thread and for one on the runtime.
pub use self::install::{
    repair_toolchain,
    repair_toolchain_awaiting,
};
/// The launcher records of a worktree that name another checkout's lock file.
pub use self::launcher_records::stale_launch_records;
/// The command a person runs to finish the repair,
/// which is the repository's own task,
/// and which this module names for the callers that already read it here.
pub use crate::the_commands_a_repair_prints::MANUAL_COMMAND;

// The slots this module's controls reach through `super::`,
// re-exported so a control sees the whole module;
// a control build is the only build that has them.
#[cfg(test)]
pub(crate) use self::names_and_bounds::{
    KILLED_AFTER_THE_RECORD_IS_ASIDE,
    WHEN_THE_RECORD_IS_OPENED,
    WHEN_THE_REMOVAL_BEGINS,
};

/// Toolchain repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_toolchain_tests.rs"]
mod tests;
