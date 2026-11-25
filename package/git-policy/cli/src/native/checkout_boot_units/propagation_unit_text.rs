//! What:
//!  The propagation service and the mount drop-in that starts it:
//!  two files whose pair asks the gate to reconcile whenever the checkout's
//!  mount comes up again.
//! Why:
//!  A mount that replaces the checkout's mount later comes up shared again,
//!  and a `Type=oneshot` gate with `RemainAfterExit=yes` does not fire on a later
//!  mount.
//!  The project chose "Unit on the checkout's mount":
//!  a drop-in on the mount unit and one one-shot service that asks the gate to
//!  reconcile by a reload, taken provisionally until a guest measurement shows
//!  whether it misses a hand-typed replacement or loops
//!  (`doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "Which triggers the route adds",
//!  "What the design is accepted").
//!  The service names the mount unit the checkout path escapes to,
//!  because a drop-in on a unit systemd learned from the mount table is read
//!  only there, and the request is a reload because a restart would run the
//!  gate's stop pass and unmount every attached clone.
//!  Neither file is installed without the gate's `ExecReload=`, and
//!  `install_plan` builds them together, so the pair cannot be written beside a
//!  gate that cannot reload.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! write('/etc/systemd/system/checkout-propagation.service', propagationService(facts));
//! write('/etc/systemd/system/home-user-Monochromatic.mount.d/10-upholds-propagation.conf', propagationMountDropIn(facts));
//! ```

use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, UnitTextRefusal, check_path};
use crate::checkout_boot_units::unit_file_text::{file_text, path_comment};
use std::path::PathBuf;
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME, EXECUTABLE_SYSTEMCTL, PROPAGATION_MOUNT_DROP_IN_NAME, PROPAGATION_SERVICE_NAME, drop_in_path, mount_unit_name,
};

/// What:
///  The service that asks the gate to re-verify the checkout's private
///  propagation.
/// Why:
///  it is bound to the checkout's mount unit, so it stops when that mount goes
///  and is armed again when it next activates;
///  it stays active after its request because of `RemainAfterExit=`,
///  and it carries no `[Install]` section because an enabled copy would run at a
///  boot whose checkout may not be mounted at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = propagationService({ checkout, rootFile, keyUnit: null });
/// ```
///
/// # Errors
/// The checkout path is not absolute or not one word.
pub fn propagation_service(facts: &CheckoutBootFacts) -> Result<String, UnitTextRefusal> {
    check_path("the checkout path", facts.checkout.as_path())?;
    let mount_unit: String = mount_unit_name(facts.checkout.as_path());
    let request: String = format!("{EXECUTABLE_SYSTEMCTL} --no-block --no-ask-password reload-or-restart {BOOT_GATE_UNIT_NAME}");
    let path: PathBuf = crate::checkout_boot_units::boot_unit_names::unit_path(PROPAGATION_SERVICE_NAME);
    return Ok(file_text(vec![
        path_comment(path.as_path()),
        String::from("[Unit]"),
        String::from("Description=Ask the boot unit to re-verify the checkout's private propagation"),
        format!("BindsTo={mount_unit}"),
        format!("After={mount_unit}"),
        String::new(),
        String::from("[Service]"),
        String::from("Type=oneshot"),
        String::from("RemainAfterExit=yes"),
        format!("ExecStart={request}"),
    ]));
}

/// What:
///  The drop-in on the checkout's mount unit that upholds the propagation
///  service.
/// Why:
///  `Upholds=` starts the service whenever the mount unit comes up and the
///  service is not already active, which is the trigger the project chose
///  without a timer and without a watcher.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = propagationMountDropIn({ checkout }); // '[Unit]\nUpholds=checkout-propagation.service\n'
/// ```
///
/// # Errors
/// The checkout path is not absolute or not one word.
pub fn propagation_mount_drop_in(facts: &CheckoutBootFacts) -> Result<String, UnitTextRefusal> {
    check_path("the checkout path", facts.checkout.as_path())?;
    let mount_unit: String = mount_unit_name(facts.checkout.as_path());
    let path: PathBuf = drop_in_path(mount_unit.as_str(), PROPAGATION_MOUNT_DROP_IN_NAME);
    return Ok(file_text(vec![path_comment(path.as_path()), String::from("[Unit]"), format!("Upholds={PROPAGATION_SERVICE_NAME}")]));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "propagation_unit_text_tests.rs"]
mod tests;
