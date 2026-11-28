//! What:
//!  The facts plan steps 19 and 20's checks read,
//!  at the state a healthy machine shows:
//!  a protected covered entry,
//!  every boot file installed unchanged beside a `canmount=noauto` dataset,
//!  and a Sanoid comparison that found nothing.
//! Why:
//!  every check takes its facts as a value,
//!  so one healthy set lets each control change exactly the field it judges,
//!  and the values are the ones this machine's design states rather than a
//!  made-up route.
//!  Never part of the release executable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = healthyRouteFacts(); facts.bootSetup.nested[0].canmount = 1;
//! const line = route_checks(facts).find((one) => one.check === 'service-unit');
//! ```

use super::boot_protection_checks::BootProtectionFacts;
use super::cancel_command_checks::CancelCommandFacts;
use super::check_outcome::{
    CheckOutcome,
    pass,
};
use super::route_test_facts_names::{
    ACCOUNT,
    CHECKOUT,
};
use super::sanoid_report::SanoidReport;
use super::zfs_checks_limits::{
    CANMOUNT_NOAUTO,
    CANMOUNT_PROPERTY,
};
use crate::checkout_boot_units::boot_unit_facts::{
    CheckoutBootFacts,
    InstalledUnitFile,
    NestedDatasetMount,
};
use crate::checkout_boot_units::boot_unit_install_plan::install_plan;
use crate::checkout_boot_units::boot_unit_names::BOOT_GATE_UNIT_PATH;
use crate::checkout_underlay_entry::underlay_entry_read::CoveredEntryFacts;
use crate::health::doctor_zfs_route::boot_setup_facts::{
    BootSetupFacts,
    FileAtInstallPath,
    NestedDatasetSetting,
    PlannedFileReading,
};
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
use crate::worktree_clone_mount_handler::sweep_dataset_facts::PropertyValue;
use crate::worktree_clone_removal_cancel::cancel_arguments::{
    CANCEL_COMMAND_WORD,
    usage,
};
use std::path::PathBuf;

/// The dataset this machine keeps inside the checkout,
/// as the owner's answer V4 names it.
const NESTED_DATASET: &str = "zpcachyos/nosnap/vm-builder-output";

/// The mountpoint that dataset is mounted at.
const NESTED_MOUNTPOINT: &str = "/home/user/Monochromatic/package/vm-builder/cli/output";

/// The covered entry of an unmounted,
/// immutable,
/// account-owned checkout.
pub(crate) fn healthy_boot_protection() -> BootProtectionFacts {
    return BootProtectionFacts {
        entry: Ok(CoveredEntryFacts {
            device: 42,
            inode: 1001,
            owner: ACCOUNT,
            mode: 0o040755,
            directory: true,
            immutable: true,
            attributes_reported: true,
            same_device_as_parent: true,
        }),
        wanted_owner: ACCOUNT,
    };
}

/// The values the setup's unit texts are built from,
/// as the design states them on this machine.
fn boot_facts() -> CheckoutBootFacts {
    return CheckoutBootFacts {
        checkout: PathBuf::from(CHECKOUT),
        root_file: PathBuf::from(ROOT_MODE_INSTALL_PATH),
        key_unit: None,
    };
}

/// The one dataset inside the checkout,
/// as the plan is given it.
fn nested_mount() -> NestedDatasetMount {
    return NestedDatasetMount {
        dataset: String::from(NESTED_DATASET),
        mountpoint: PathBuf::from(NESTED_MOUNTPOINT),
        slug: String::from("output"),
    };
}

/// The one dataset inside the checkout,
/// set the way V4 decides.
fn nested() -> NestedDatasetSetting {
    return NestedDatasetSetting {
        dataset: String::from(NESTED_DATASET),
        canmount: PropertyValue::Number(CANMOUNT_NOAUTO),
    };
}

/// The healthy boot setup:
/// every file this build's plan writes installed unchanged,
/// and the dataset under the checkout `canmount=noauto`.
pub(crate) fn healthy_boot_setup() -> BootSetupFacts {
    let plan: Vec<InstalledUnitFile> =
        install_plan(&boot_facts(), &[nested_mount()]).expect("the route's own plan");
    let gate_path: PathBuf = PathBuf::from(BOOT_GATE_UNIT_PATH);
    let mut gate: Option<PlannedFileReading> = None;
    let mut others: Vec<PlannedFileReading> = Vec::new();
    for file in plan {
        let reading: PlannedFileReading = PlannedFileReading {
            at_the_path: FileAtInstallPath::Content(file.text.clone()),
            path: file.path,
            planned_text: file.text,
        };
        if reading.path == gate_path {
            gate = Some(reading);
        } else {
            others.push(reading);
        }
    }
    return BootSetupFacts {
        gate: gate.expect("the plan writes the gate"),
        others,
        key_unit: None,
        nested: vec![nested()],
    };
}

/// The cancel command's facts,
/// read from the command's own grammar,
/// which is the copy the wiring run also reads.
pub(crate) fn healthy_cancel_command() -> CancelCommandFacts {
    let text: String = usage();
    let first_line: String = String::from(text.lines().next().expect("the usage has a line"));
    return CancelCommandFacts {
        word: String::from(CANCEL_COMMAND_WORD),
        usage_line: first_line,
    };
}

/// The Sanoid comparison of a machine whose configuration and tree agree,
/// with no drift and no correction below the line.
pub(crate) fn healthy_sanoid() -> SanoidReport {
    let outcome: CheckOutcome = pass(format!(
        "every dataset under the recursive ancestor is covered, the dataset the coverage names as excluded carries `{CANMOUNT_PROPERTY}` beside its section, and the store lies under a dataset the coverage reaches"
    ));
    return SanoidReport::new(outcome, Vec::new());
}
