//! What:
//!  The checks plan step 18 runs before anything else:
//!  `/dev/zfs` and the account's open of it,
//!  the loaded module's layout against the request contract,
//!  the pool the checkout dataset names,
//!  the checkout dataset and the clone container with their decided
//!  properties,
//!  each dataset's encryption key,
//!  and every clone's attach properties.
//! Why:
//!  The design's preflight reads these facts before a claim,
//!  slot or clone is
//!  written,
//!  and `doctor` reports the same facts without acting on them
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints" and
//!  "Add and Git registration",
//!  step 1;
//!  `doc/planning/cli-git-zfs-runtime-layout-checker-design.md`,
//!  whose verdict the ABI check reads).
//!  Every value here is a reading,
//!  so a dataset that is not there,
//!  a dataset this account could not read and a dataset whose property moved
//!  are three different answers:
//!  the first two decide between a finding and unknown,
//!  and the third is always a finding naming the property.
//!  Nothing here opens the device:
//!  the caller hands in what it read,
//!  so a control answers without this machine's kernel files.
//!  The checks are split across siblings because this file is over the line
//!  limit:
//!  this one lists one report per dataset,
//!  `zfs_checks_limits.rs` holds the numbers and words the kernel and the
//!  design fix,
//!  `zfs_checks_reading.rs` reads one failed dataset read,
//!  `zfs_checks_machine_prerequisites.rs` judges the device,
//!  the module layout and the pool,
//!  `zfs_checks_dataset_roles.rs` holds a dataset's role and its facts,
//!  `zfs_checks_dataset_properties.rs` compares one dataset against the
//!  properties its role requires,
//!  and `zfs_checks_key_and_clones.rs` judges the keys and the clones.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = { role: DatasetRole.Container, name, reading: Reading.Read(cloneObjectFacts) };
//! const outcome = checkDataset(facts); // { kind: 'pass' | 'finding' | 'unknown' }
//! ```

/// One check's answer,
/// which a control reads through `super::`.
#[cfg(test)]
use super::check_outcome::CheckOutcome;
/// One report per dataset of the route.
use super::check_outcome::CheckReport;
/// One dataset against the properties its role requires.
pub use super::zfs_checks_dataset_properties::check_dataset;
/// One dataset's facts,
/// and the check its role requires.
pub use super::zfs_checks_dataset_roles::{
    DatasetFacts,
    DatasetRole,
};
/// The encryption key of each dataset,
/// and every clone's attach properties.
pub use super::zfs_checks_key_and_clones::{
    check_clone_properties,
    check_key,
};
/// The numbers and words the kernel and the design fix.
pub use super::zfs_checks_limits::{
    CANMOUNT_OFF,
    CANMOUNT_PROPERTY,
    ENCRYPTION_OFF,
    FILESYSTEM_OBJECT_SET,
    FLAG_OFF,
    KEY_UNAVAILABLE,
    LEGACY_MOUNTPOINT,
    NOT_FOUND_ERRNO,
};
/// The device the route reads the pool through,
/// the module layout's verdict and the pool itself.
pub use super::zfs_checks_machine_prerequisites::{
    AbiFacts,
    DeviceFacts,
    DeviceState,
    PoolFacts,
    check_abi,
    check_device,
    check_pool,
};
/// Any value a read of one dataset answers with,
/// and the rule that reads a failed read.
pub use super::zfs_checks_reading::{
    Reading,
    reading_of,
};
/// What:
///  One report per dataset of the route,
///  each named by its role.
/// Why:
///  The wiring run prints one line per dataset,
///  so a person reads the checkout,
///  the container and each clone separately,
///  and a control asserts one of them without building the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reports = datasetReports(datasets); // [{ check: 'checkout-dataset', outcome }]
/// ```
pub fn dataset_reports(datasets: &[DatasetFacts]) -> Vec<CheckReport> {
    let mut reports: Vec<CheckReport> = Vec::new();
    for dataset in datasets {
        reports.push(CheckReport::new(dataset.check(), check_dataset(dataset)));
    }
    return reports;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "zfs_checks_tests.rs"]
mod tests;
