//! What:
//!  Every file a setup run writes for the boot and shutdown route,
//!  in the order it writes them.
//! Why:
//!  The gate must carry `ExecReload=` before the propagation pair is installed,
//!  because without that line the pair's request is a restart, which stops the
//!  gate, runs its `ExecStopPost=shutdown-unmount` and unmounts every attached
//!  clone
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "What must be applied to the machine, and where that is recorded").
//!  Building the list in one place means the pair cannot be written beside a
//!  gate that cannot reload it,
//!  and a control writes exactly what a person's setup would write.
//!  The nested datasets' units come last, because each requires the gate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const files = installPlan(facts, nested); // gate, stop drop-in, provider drop-in, propagation pair, nested units
//! ```

use crate::checkout_boot_units::boot_gate_unit_text::{boot_gate_unit, provider_restart_drop_in, stop_failure_drop_in};
use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, InstalledUnitFile, NestedDatasetMount, UnitTextRefusal};
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME, BOOT_GATE_UNIT_PATH, PROPAGATION_MOUNT_DROP_IN_NAME, PROVIDER_DROP_IN_NAME, PROVIDER_UNIT_NAME, STOP_FAILURE_DROP_IN_NAME, drop_in_path, mount_unit_name,
    nested_unit_name, propagation_service_path, unit_path,
};
use crate::checkout_boot_units::nested_dataset_unit_text::nested_dataset_unit;
use crate::checkout_boot_units::propagation_unit_text::{propagation_mount_drop_in, propagation_service};
use std::path::PathBuf;

/// What:
///  Every file the setup writes,
///  with each text built from the same facts,
///  or the refusal that stopped the plan.
/// Why:
///  a plan that built some texts and refused later ones would leave a machine
///  half set up,
///  so every value is checked before the list is returned and the caller writes
///  nothing on a refusal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = installPlan(facts, [{ dataset, mountpoint, slug }]);
/// ```
///
/// # Errors
/// Any fact a unit text cannot carry,
/// which the first refused text names.
pub fn install_plan(facts: &CheckoutBootFacts, nested: &[NestedDatasetMount]) -> Result<Vec<InstalledUnitFile>, UnitTextRefusal> {
    facts.checked()?;
    let gate: String = boot_gate_unit(facts)?;
    if !gate.lines().any(|line| return line.starts_with("ExecReload=")) {
        return Err(UnitTextRefusal::MissingLine { line: "ExecReload=", unit: BOOT_GATE_UNIT_NAME });
    }
    let mount_unit: String = mount_unit_name(facts.checkout.as_path());
    let mut files: Vec<InstalledUnitFile> = vec![
        InstalledUnitFile { path: PathBuf::from(BOOT_GATE_UNIT_PATH), text: gate },
        InstalledUnitFile { path: drop_in_path(BOOT_GATE_UNIT_NAME, STOP_FAILURE_DROP_IN_NAME), text: stop_failure_drop_in() },
        InstalledUnitFile { path: drop_in_path(PROVIDER_UNIT_NAME, PROVIDER_DROP_IN_NAME), text: provider_restart_drop_in() },
        InstalledUnitFile { path: propagation_service_path(), text: propagation_service(facts)? },
        InstalledUnitFile { path: drop_in_path(mount_unit.as_str(), PROPAGATION_MOUNT_DROP_IN_NAME), text: propagation_mount_drop_in(facts)? },
    ];
    for one in nested {
        let name: String = nested_unit_name(one.slug.as_str());
        files.push(InstalledUnitFile { path: unit_path(name.as_str()), text: nested_dataset_unit(one, facts)? });
    }
    return Ok(files);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_unit_install_plan_tests.rs"]
mod tests;
