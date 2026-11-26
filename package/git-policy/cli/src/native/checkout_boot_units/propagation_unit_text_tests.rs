//! What:
//!  The propagation pair's controls:
//!  what the service is bound to and what it asks,
//!  the drop-in that upholds it,
//!  the absence of an install section,
//!  the escaped mount unit name in both files,
//!  and the refusals.
//! Why:
//!  the pair is only safe beside a gate that carries `ExecReload=`,
//!  and the service names the mount unit by its escaped path,
//!  which is the one spelling a drop-in on a learned unit is read under.
//!  A control that checked the unescaped path would pass on a machine where
//!  the pair is installed and inert.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesOf(propagationService(facts), '[Unit]')).toContain('BindsTo=home-user-Monochromatic.mount');
//! ```

use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, UnitTextRefusal};
use crate::checkout_boot_units::boot_unit_names::{BOOT_GATE_UNIT_NAME, PROPAGATION_MOUNT_DROP_IN_NAME, PROPAGATION_SERVICE_NAME};
use crate::checkout_boot_units::propagation_unit_text::{propagation_mount_drop_in, propagation_service};
use crate::checkout_boot_units::unit_text_sections::{assert_file_shape, carries, holds, lines_of};
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
use std::path::PathBuf;

/// The route's facts,
/// with this machine's checkout path.
fn facts() -> CheckoutBootFacts {
    return CheckoutBootFacts { checkout: PathBuf::from("/home/user/Monochromatic"), root_file: PathBuf::from(ROOT_MODE_INSTALL_PATH), key_unit: None };
}

#[test]
fn the_service_binds_to_the_mount_unit_the_checkout_path_escapes_to() {
    let text: String = propagation_service(&facts()).expect("the route's facts are text");
    assert_file_shape(&text);
    assert!(holds(&text, "[Unit]", "BindsTo=home-user-Monochromatic.mount"), "the service stops with the mount it serves: {text}");
    assert!(holds(&text, "[Unit]", "After=home-user-Monochromatic.mount"), "and starts after it: {text}");
    assert!(carries(&text, "[Unit]", &format!("Description=Ask the boot unit")), "a person reads what asked: {text}");
}

#[test]
fn the_service_asks_for_a_reload_and_stays_active_without_an_install_section() {
    let text: String = propagation_service(&facts()).expect("the route's facts are text");
    assert!(holds(&text, "[Service]", "Type=oneshot"));
    assert!(holds(&text, "[Service]", "RemainAfterExit=yes"), "a clean run must not be upheld again at once: {text}");
    assert!(
        holds(
            &text,
            "[Service]",
            format!("ExecStart=/usr/bin/systemctl --no-block --no-ask-password reload-or-restart {BOOT_GATE_UNIT_NAME}").as_str()
        ),
        "a restart would run the gate's stop pass and unmount every clone: {text}"
    );
    assert!(lines_of(&text, "[Install]").is_empty(), "an enabled copy would run at a boot with no checkout mounted: {text}");
    assert!(!text.contains("[Install]"), "the file has no install section at all: {text}");
}

#[test]
fn the_mount_drop_in_upholds_the_service() {
    let text: String = propagation_mount_drop_in(&facts()).expect("the route's facts are text");
    assert_file_shape(&text);
    assert!(holds(&text, "[Unit]", format!("Upholds={PROPAGATION_SERVICE_NAME}").as_str()), "the mount unit is what starts the service: {text}");
    assert!(text.contains("home-user-Monochromatic.mount.d"), "the file's own path is the mount unit's drop-in folder: {text}");
    assert_eq!(PROPAGATION_MOUNT_DROP_IN_NAME, "10-upholds-propagation.conf");
}

#[test]
fn a_guest_path_with_dashes_is_escaped_in_both_files() {
    let guest: CheckoutBootFacts = CheckoutBootFacts { checkout: PathBuf::from("/srv/zfs-step-19-fixture/home/account/Monochromatic"), ..facts() };
    let escaped: &str = r"srv-zfs\x2dstep\x2d19\x2dfixture-home-account-Monochromatic.mount";
    let service: String = propagation_service(&guest).expect("a guest path is text");
    assert!(holds(&service, "[Unit]", format!("BindsTo={escaped}").as_str()), "the escaped name is the one systemd knows: {service}");
    let drop_in: String = propagation_mount_drop_in(&guest).expect("a guest path is text");
    assert!(drop_in.contains(format!("{escaped}.d").as_str()), "and it is the folder the drop-in is read from: {drop_in}");
}

#[test]
fn a_checkout_path_a_unit_line_cannot_carry_is_refused_by_both_files() {
    let spaced: CheckoutBootFacts = CheckoutBootFacts { checkout: PathBuf::from("/home/user/Mono chrome"), ..facts() };
    let service: UnitTextRefusal = propagation_service(&spaced).expect_err("a space breaks a dependency line too");
    assert!(service.message().contains("Mono chrome"), "the refusal names the value: {}", service.message());
    assert!(matches!(propagation_mount_drop_in(&spaced), Err(UnitTextRefusal::PathNotOneWord { .. })));
}
