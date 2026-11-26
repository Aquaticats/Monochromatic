//! What:
//!  Package-manager and toolchain repair after a worktree is created.
//! Why:
//!  Repair runs after worktree creation and has its own report.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `package_repair_cargo_concurrent.rs`:
//!   the runner that does one job for every item of a list on several threads.
//! - `package_repair_cargo_lock.rs`:
//!   the reading of `Cargo.lock` before a clean and its restoring afterwards.
//! - `package_repair_prewarm.rs`:
//!   the hand-over of a new worktree's Cargo build directories to the commit service,
//!   so the add does not wait for the clone that fills them.
//! - `package_repair_prewarm_job_names.rs`:
//!   the names one filling of a workspace works in,
//! - `package_repair_prewarm_job_staged_state.rs`:
//!   the answers package repair prewarm job staged state holds.
//! - `package_repair_relink.rs`:
//!   the hand-over of a new worktree's copied package state to the commit service,
//!   so the add does not wait for pnpm's relink or the toolchain install.
//! - `package_repair_relink_failure.rs`:
//!   why a relink's last try failed,
//!   kept until the record settles or the notice quotes it.
//! - `package_repair_relink_job_worktree_proof.rs`:
//!   what a delayed relink proves about the worktree its record names,
//!   before that record's first tool and before every tool after it.
//! - `package_repair_start.rs`:
//!   the bounded start of one package-manager program for a worktree repair,
//!   run off the runtime's workers,
//!   and the stop of a child that appears after the bound.
//! - `package_repair_command.rs`:
//!   starting one package-manager program for a worktree repair:
//!   finding it,
//!   running it with a limit,
//!   saying how it ended.
//! - `package_repair_cargo_workspaces.rs`:
//!   the Cargo workspaces of a new worktree and the manifests that lead to them.
//! - `package_repair_cargo_jobs.rs`:
//!   the answers package repair cargo jobs holds.
//! - `package_repair_report.rs`:
//!   what a worktree repair prints:
//!   one line for success,
//!   one plain line per repair that could not finish.
//! - `package_repair_request.rs`:
//!   what a repair reads and is asked:
//!   the person's switches,
//!   what the process offers,
//!   and the request that holds both.
//! - `package_repair_cargo_hand_over.rs`:
//!   the hand-over of the filling of a new worktree's build directories to the commit service,
//!   and the one line the add answers for it.
//! - `package_repair_prewarm_job.rs`:
//!   the service's filling of those build directories:
//!   clone into a staging directory,
//!   clean the copy,
//!   publish it unless a build got there first.
//! - `package_repair_toolchain_names_and_bounds.rs`:
//!   the words and size bounds the toolchain repair reads,
//! - `package_repair_toolchain_object_identity.rs`:
//!   the object a name holds,
//!   as device,
//!   inode,
//!   size and write time,
//! - `package_repair_toolchain_launcher_records.rs`:
//!   the launcher records of a worktree's launcher directory,
//! - `package_repair_toolchain.rs`:
//!   the toolchain repair of a new worktree:
//!   rebuild the links and launchers that name another checkout's lock.
//! - `package_repair_toolchain_copy_removal.rs`:
//!   the removal of the launcher records,
//!   and the launchers beside them,
//! - `package_repair_toolchain_install.rs`:
//!   the answers package repair toolchain install holds.
//! - `package_repair_touches_only_its_own_worktree.rs`:
//!   whether a directory a repair deletes from or writes in is inside the worktree it serves,
//!   once its links are followed.
//! - `package_repair_cargo.rs`:
//!   the Cargo repair of a new worktree:
//!   clone external build state,
//!   then clean the worktree's own crates.
//! - `package_repair_pnpm.rs`:
//!   the pnpm repair of a new worktree:
//!   relink the copied `node_modules` offline against the frozen lockfile.
//! - `package_repair.rs`:
//!   the repair of a new worktree's package-manager state after a rung filled it and the ignored files were copied.
//! - `package_repair_relink_job.rs`:
//!   the service's relinking of that state:
//!   each tool's own frozen install in the worktree,
//!   run after the add returned.
//! - `package_repair_relink_notice.rs`:
//!   the notice a relink given up on leaves for the person,
//!   and the lines a later command in that worktree shows of it.
//! - `package_repair_tree_clone.rs`:
//!   cloning one directory tree with its modes,
//!   times and links,
//!   for the build state Cargo keeps outside a worktree.
//! - `package_repair_prewarm_job_filling.rs`:
//!   the answers package repair prewarm job filling holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // repair/index.ts: one `export * from './file'` per module below.
//! ```

/// The bounded start of one package-manager program for a worktree repair,
/// run off the runtime's workers,
/// and the stop of a child that appears after the bound.
#[cfg(target_os = "linux")]
pub mod package_repair_start;

/// Starting one package-manager program for a worktree repair:
/// finding it,
/// running it with a limit,
/// saying how it ended.
#[cfg(target_os = "linux")]
pub mod package_repair_command;

/// What a worktree repair prints:
/// one line for success,
/// one plain line per repair that could not finish.
#[cfg(target_os = "linux")]
pub mod package_repair_report;

/// Cloning one directory tree with its modes,
/// times and links,
/// for the build state Cargo keeps outside a worktree.
#[cfg(target_os = "linux")]
pub mod package_repair_tree_clone;

/// The pnpm repair of a new worktree:
/// relink the copied `node_modules` offline against the frozen lockfile.
#[cfg(target_os = "linux")]
pub mod package_repair_pnpm;

/// The Cargo repair of a new worktree:
/// clone external build state,
/// then clean the worktree's own crates.
#[cfg(target_os = "linux")]
pub mod package_repair_cargo;

/// The hand-over of a new worktree's Cargo build directories to the commit service,
/// so the add does not wait for the clone that fills them.
#[cfg(target_os = "linux")]
pub mod package_repair_prewarm;

/// The service's filling of those build directories:
/// clone into a staging directory,
/// clean the copy,
/// publish it unless a build got there first.
#[cfg(target_os = "linux")]
pub mod package_repair_prewarm_job;

/// The hand-over of a new worktree's copied package state to the commit service,
/// so the add does not wait for pnpm's relink or the toolchain install.
#[cfg(target_os = "linux")]
pub mod package_repair_relink;

/// The service's relinking of that state:
/// each tool's own frozen install in the worktree,
/// run after the add returned.
#[cfg(target_os = "linux")]
pub mod package_repair_relink_job;

/// What a delayed relink proves about the worktree its record names,
/// before that record's first tool and before every tool after it.
#[cfg(target_os = "linux")]
pub mod package_repair_relink_job_worktree_proof;

/// Why a relink's last try failed,
/// kept until the record settles or the notice quotes it.
pub mod package_repair_relink_failure;

/// The notice a relink given up on leaves for the person,
/// and the lines a later command in that worktree shows of it.
pub mod package_repair_relink_notice;

/// The runner that does one job for every item of a list on several threads.
pub mod package_repair_cargo_concurrent;

/// The reading of `Cargo.lock` before a clean and its restoring afterwards.
pub mod package_repair_cargo_lock;

/// Whether a directory a repair deletes from or writes in is inside the worktree it serves,
/// once its links are followed.
pub mod package_repair_touches_only_its_own_worktree;

/// The Cargo workspaces of a new worktree and the manifests that lead to them.
pub mod package_repair_cargo_workspaces;

/// The toolchain repair of a new worktree:
/// rebuild the links and launchers that name another checkout's lock.
#[cfg(target_os = "linux")]
pub mod package_repair_toolchain;

/// The repair of a new worktree's package-manager state after a rung filled it and the ignored files were copied.
#[cfg(target_os = "linux")]
pub mod package_repair;

/// What a repair reads and is asked:
///  the person's switches,
/// what the process offers,
/// and the request that holds both.
pub mod package_repair_request;
