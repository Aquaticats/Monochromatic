//! What:
//!  The `doctor` report and the checks of hooks and configuration it runs.
//! Why:
//!  A report reads many subsystems but is one feature.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `checkout_layout.rs`:
//!   how the checkout the report describes is laid out,
//!   and the command that converts each layout.
//! - `doctor_filesystem.rs`:
//!   the filesystem section of the report:
//!   btrfs subvolume,
//!   overlayfs and ZFS,
//!   the removal mount option and its commands.
//! - `doctor_fsmonitor.rs`:
//!   what the report says about Git's own fsmonitor.
//! - `doctor_hook_entries.rs`:
//!   the entries of one hooks directory,
//!   and the bytes the report spells them as.
//! - `external_hooks.rs`:
//!   the configured external hooks,
//!   read from the configuration snapshot.
//! - `traditional_hook_warning.rs`:
//!   the non-blocking warning for a configured `core.hooksPath`,
//!   decided from the configuration snapshot alone.
//! - `lfs_shims.rs`:
//!   the Git LFS shims a repository may hold,
//!   recognized byte for byte.
//! - `hooks_path_values.rs`:
//!   pathnames as Git prints them,
//!   and every value of `core.hooksPath` with its scope.
//! - `doctor_config.rs`:
//!   what the report reads from the configuration:
//!   configured hooks and the retired `hooks` key.
//! - `doctor.rs`:
//!   the report itself:
//!   the traditional hooks a repository holds,
//!   and the exit status its findings give.
//!
//! The child folders `doctor_zfs_route/` and `sanoid_coverage/` hold the ZFS route's and
//! the Sanoid coverage's own checks,
//! each with a folder of its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // health/index.ts: one `export * from './file'` per module below.
//! ```

/// What `git monochromatic-git-policy-cli doctor` reports from the configuration:
/// configured hooks and the retired `hooks` key.
pub mod doctor_config;

/// What `git monochromatic-git-policy-cli doctor` says about Git's own fsmonitor:
/// off by the owner's decision,
/// or the setting that turns it on.
pub mod doctor_fsmonitor;

/// Pathnames as Git prints them,
/// and every value of `core.hooksPath` with its scope.
pub mod hooks_path_values;

/// The configured external hooks,
/// read from the snapshot,
/// and the loud report that none runs.
pub mod external_hooks;

/// How the checkout the filesystem section reports on is laid out,
/// and the command that converts each layout.
pub mod checkout_layout;

/// The non-blocking warning for a configured `core.hooksPath`,
/// decided from the configuration snapshot alone.
pub mod traditional_hook_warning;

/// The Git LFS shims a repository may hold,
/// recognized byte for byte,
/// and the explanation the report gives for them.
pub mod lfs_shims;

/// `git monochromatic-git-policy-cli doctor`:
/// the traditional hooks a repository holds,
/// which are unsupported.
pub mod doctor;

/// The entries of one hooks directory:
/// which of them Git would run as a traditional hook,
/// how the directory is listed,
/// and the bytes the report spells them as.
pub mod doctor_hook_entries;

/// The filesystem section of `git monochromatic-git-policy-cli doctor`:
/// btrfs subvolume,
/// overlayfs,
/// the removal mount option and its commands.
#[cfg(target_os = "linux")]
pub mod doctor_filesystem;

/// The ZFS route's checks plan step 18 adds to `doctor`:
/// the device,
/// ABI and dataset facts,
/// the checkout's mount and propagation,
/// the recorded identities,
/// the store's listing and the two staging forms it removes,
/// the delegation,
/// the handler and its records,
/// the staging folder,
/// the holders,
/// and the checks later steps owe.
/// Nothing reaches it from a command path until step 18 is wired.
#[cfg(target_os = "linux")]
pub mod doctor_zfs_route;

/// Plan step 20's Sanoid coverage,
/// built ahead and unwired:
/// the generated `sanoid.conf` sections,
/// the `com.sun:auto-snapshot=false` markers,
/// `doctor`'s comparison of the measured dataset tree and store against
/// recursive coverage with its printed correction,
/// and the report of the space guard's actual scope.
/// Nothing reaches it from a command path until step 20 is wired.
#[cfg(target_os = "linux")]
pub mod sanoid_coverage;
