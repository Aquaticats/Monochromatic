//! What:
//!  One unit per dataset whose mountpoint lies inside the checkout:
//!  the unit that mounts it only after the gate proved the checkout,
//!  and skips itself when the dataset is already mounted.
//! Why:
//!  `zfs mount -a` mounts a path descendant before the checkout or under a
//!  shared checkout, and a later checkout attach hides it while the mount row
//!  and any folder it made stay.
//!  The project's answer sets such a dataset `canmount=noauto` and gives it this
//!  unit, ordered after the gate, until the wrapper's own
//!  `mount-path-descendant` mode is the thing that mounts it (V4 and V9,
//!  `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`).
//!  The mode exists in this build, so the `ExecStart` line here runs it;
//!  the design's earlier explicit `zfs mount` shape is what it replaces, and the
//!  `[Unit]` block is the same under both.
//!  The condition is load-bearing:
//!  a re-run of `zfs mount` on a mounted dataset prints `filesystem already
//!  mounted` and exits one, while a skipped unit counts as success for a
//!  consumer that requires it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = nestedDatasetUnit({ dataset, mountpoint, slug }, facts);
//! ```

use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, NestedDatasetMount, UnitTextRefusal, check_word, path_text};
use crate::checkout_boot_units::unit_file_text::{file_text, path_comment};
use crate::checkout_boot_units::boot_unit_names::{BOOT_GATE_UNIT_NAME, nested_unit_name, root_words};
use std::path::PathBuf;

/// The mode word that mounts one nested dataset under the recorded checkout.
pub const MOUNT_PATH_DESCENDANT_MODE_WORD: &str = "mount-path-descendant";

/// The option that names the nested dataset the mode mounts.
pub const DATASET_OPTION: &str = "--dataset";

/// What:
///  The unit that mounts one nested dataset under the verified checkout.
/// Why:
///  the condition keeps the unit green where the provider already mounted the
///  dataset,
///  and the requirement on the gate is what makes the mount wait for a checkout
///  the gate proved rather than for whichever mount was there first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = nestedDatasetUnit({ dataset: '…/vm-builder-output', mountpoint: '/home/…/output', slug: 'output' }, facts);
/// ```
///
/// # Errors
/// A dataset name,
/// mountpoint,
/// slug or root file a systemd line cannot carry.
pub fn nested_dataset_unit(nested: &NestedDatasetMount, facts: &CheckoutBootFacts) -> Result<String, UnitTextRefusal> {
    facts.checked()?;
    nested.checked()?;
    check_word("the nested dataset's name", nested.dataset.as_str())?;
    let name: String = nested_unit_name(nested.slug.as_str());
    let path: PathBuf = crate::checkout_boot_units::boot_unit_names::unit_path(name.as_str());
    let mountpoint: &str = path_text("the nested dataset's mountpoint", nested.mountpoint.as_path())?;
    let root: &str = path_text("the root-owned file's path", facts.root_file.as_path())?;
    let words: String = root_words();
    let command: String = format!("{root} {words} {MOUNT_PATH_DESCENDANT_MODE_WORD} {DATASET_OPTION} {}", nested.dataset);
    return Ok(file_text(vec![
        path_comment(path.as_path()),
        String::from("[Unit]"),
        format!("Description=Mount {} under the verified checkout", nested.dataset),
        format!("Requires={BOOT_GATE_UNIT_NAME}"),
        format!("After={BOOT_GATE_UNIT_NAME}"),
        format!("ConditionPathIsMountPoint=!{mountpoint}"),
        String::new(),
        String::from("[Service]"),
        String::from("Type=oneshot"),
        String::from("RemainAfterExit=yes"),
        format!("ExecStart={command}"),
        String::new(),
        String::from("[Install]"),
        String::from("WantedBy=multi-user.target"),
    ]));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "nested_dataset_unit_text_tests.rs"]
mod tests;
