//! What:
//!  Package-manager and toolchain repair after a worktree is created.
//! Why:
//!  Repair runs after worktree creation and has its own report.
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

/// The relink of a worktree a removal takes:
/// the record that had not started is settled,
/// and the tool of one that had stops itself.
#[cfg(target_os = "linux")]
pub mod package_repair_relink_of_a_removed_worktree;

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
