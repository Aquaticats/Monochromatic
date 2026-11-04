//! What:
//!  Package-manager and toolchain repair after a worktree is created.
//! Why:
//!  Repair runs after worktree creation and has its own report.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // repair/index.ts: one `export * from './file'` per module below.
//! ```

/// Starting one package-manager program for a worktree repair: finding it, running it with a limit, saying how it ended.
#[cfg(target_os = "linux")]
pub mod package_repair_command;

/// What a worktree repair prints: one line for success, one plain line per repair that could not finish.
#[cfg(target_os = "linux")]
pub mod package_repair_report;

/// Cloning one directory tree with its modes, times and links, for the build state Cargo keeps outside a worktree.
#[cfg(target_os = "linux")]
pub mod package_repair_tree_clone;

/// The pnpm repair of a new worktree: relink the copied `node_modules` offline against the frozen lockfile.
#[cfg(target_os = "linux")]
pub mod package_repair_pnpm;

/// The Cargo repair of a new worktree: clone external build state, then clean the worktree's own crates.
#[cfg(target_os = "linux")]
pub mod package_repair_cargo;

/// The toolchain repair of a new worktree: rebuild the links and launchers that name another checkout's lock.
#[cfg(target_os = "linux")]
pub mod package_repair_toolchain;

/// The repair of a new worktree's package-manager state after a rung filled it and the ignored files were copied.
#[cfg(target_os = "linux")]
pub mod package_repair;
