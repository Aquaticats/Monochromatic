//! What:
//!  The `doctor` report and the checks of hooks and configuration it runs.
//! Why:
//!  A report reads many subsystems but is one feature.
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
