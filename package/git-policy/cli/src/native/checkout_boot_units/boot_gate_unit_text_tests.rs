//! What:
//!  The gate's controls:
//!  what it wants and requires of the provider,
//!  how it orders the checkout,
//!  the stop half,
//!  the three mode lines with their failure prefixes,
//!  the owner's bounds,
//!  the key lines' condition,
//!  the two drop-ins beside it,
//!  and the values it refuses.
//! Why:
//!  the owner's answers of 2026-11-24 are one line each,
//!  and a text that lost one would boot with a weaker promise than the record
//!  states:
//!  `Requires=` on the provider restarts the gate with every attached clone,
//!  a before-login line holds a login on a hung sweep,
//!  and a third key line requires a unit no boot installs.
//!  Each assertion names the section it read, so a line of another section
//!  never satisfies it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesOf(bootGateUnit(facts), '[Unit]')).toContain('Before=umount.target');
//! ```

use crate::checkout_boot_units::boot_gate_unit_text::{boot_gate_unit, provider_restart_drop_in, stop_failure_drop_in};
use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, UnitTextRefusal};
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME, BOOT_RECONCILE_MODE_WORD, GATE_START_BOUND, GATE_STOP_BOUND, SHUTDOWN_UNMOUNT_MODE_WORD, START_PROBE_MODE_WORD, STOP_FAILURE_DROP_IN_NAME, STOP_FAILURE_MODE_WORD,
    root_words,
};
use crate::checkout_boot_units::unit_text_sections::{assert_file_shape, carries, holds, lines_of};
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
use std::path::PathBuf;

/// The facts a setup run of this route fills in,
/// with no key-loading unit,
/// which is this host's shape.
fn facts() -> CheckoutBootFacts {
    return CheckoutBootFacts { checkout: PathBuf::from("/home/user/Monochromatic"), root_file: PathBuf::from(ROOT_MODE_INSTALL_PATH), key_unit: None };
}

/// The commands the gate's three lines begin with,
/// which every root-mode line spells the same way.
fn root_command() -> String {
    return format!("{ROOT_MODE_INSTALL_PATH} {}", root_words());
}

#[test]
fn the_gate_wants_the_provider_and_orders_after_it_without_requiring_it() {
    let text: String = boot_gate_unit(&facts()).expect("the route's facts are text");
    assert!(holds(&text, "[Unit]", "Wants=zfs-mount.service"), "V2 wants the provider: {text}");
    assert!(holds(&text, "[Unit]", "After=zfs-mount.service zfs-import.target"), "V2 orders after the provider and the import: {text}");
    assert!(!carries(&text, "[Unit]", "Requires=zfs-mount.service"), "a provider restart must leave the gate and its clones alone: {text}");
}

#[test]
fn the_gate_names_the_checkout_in_one_requires_mounts_for_line() {
    let text: String = boot_gate_unit(&facts()).expect("the route's facts are text");
    assert!(holds(&text, "[Unit]", "RequiresMountsFor=/home/user/Monochromatic"), "V3's one line: {text}");
    let ordering: Vec<String> = lines_of(&text, "[Unit]").into_iter().filter(|line| return line.starts_with("Before=") || line.starts_with("After=")).collect();
    assert!(!ordering.iter().any(|line| return line.contains("systemd-user-sessions")), "V8a keeps logins out of the gate's order: {ordering:?}");
}

#[test]
fn the_gate_stops_before_umount_target_and_conflicts_with_it() {
    let text: String = boot_gate_unit(&facts()).expect("the route's facts are text");
    assert!(holds(&text, "[Unit]", "Before=umount.target"), "the stop pass runs before umount.target: {text}");
    assert!(holds(&text, "[Unit]", "Conflicts=umount.target"), "a shutdown takes the gate down: {text}");
}

#[test]
fn the_gate_runs_the_three_modes_with_the_prefixes_the_design_measured() {
    let text: String = boot_gate_unit(&facts()).expect("the route's facts are text");
    let command: String = root_command();
    assert!(
        holds(&text, "[Service]", format!("ExecStartPre=-{command} {START_PROBE_MODE_WORD}").as_str()),
        "a failing probe must not fail the gate, and the dash is what buys that: {text}"
    );
    assert!(holds(&text, "[Service]", format!("ExecStart={command} {BOOT_RECONCILE_MODE_WORD}").as_str()), "the pass itself: {text}");
    assert!(holds(&text, "[Service]", format!("ExecReload={command} {BOOT_RECONCILE_MODE_WORD}").as_str()), "the propagation pair's lever: {text}");
    assert!(
        holds(&text, "[Service]", format!("ExecStopPost=-{command} {SHUTDOWN_UNMOUNT_MODE_WORD}").as_str()),
        "the stop pass runs whichever way the start ended, and a refusal in the mode must not fail a shutdown: {text}"
    );
}

#[test]
fn the_gate_is_a_oneshot_that_remains_and_is_wanted_at_multi_user() {
    let text: String = boot_gate_unit(&facts()).expect("the route's facts are text");
    assert!(holds(&text, "[Service]", "Type=oneshot"));
    assert!(holds(&text, "[Service]", "RemainAfterExit=yes"));
    assert!(holds(&text, "[Install]", "WantedBy=multi-user.target"));
}

#[test]
fn the_owner_bounds_stand_until_step_22_measures_the_workload() {
    let text: String = boot_gate_unit(&facts()).expect("the route's facts are text");
    assert_eq!(GATE_START_BOUND, "5min");
    assert_eq!(GATE_STOP_BOUND, "1min");
    assert!(holds(&text, "[Service]", "TimeoutStartSec=5min"), "V8b's start bound: {text}");
    assert!(holds(&text, "[Service]", "TimeoutStopSec=1min"), "V8b's stop bound: {text}");
}

#[test]
fn the_key_lines_appear_only_on_a_machine_that_has_a_key_unit() {
    let without: String = boot_gate_unit(&facts()).expect("the route's facts are text");
    assert!(!carries(&without, "[Unit]", "Requires=zfs-load-key@"), "no key unit means no requirement on one: {without}");

    let key_unit: &str = r"zfs-load-key@zfsstep19-home-monochromatic\x2dgit\x2dcheckout.service";
    let with: CheckoutBootFacts = CheckoutBootFacts { key_unit: Some(String::from(key_unit)), ..facts() };
    let text: String = boot_gate_unit(&with).expect("a generator's escaped unit name is one dependency word");
    assert!(holds(&text, "[Unit]", format!("Requires={key_unit}").as_str()), "the key unit's failure is the checkout's: {text}");
    assert!(holds(&text, "[Unit]", format!("After={key_unit}").as_str()), "and the gate waits for it: {text}");
}

#[test]
fn the_stop_failure_drop_in_sets_terminate_and_sorts_after_the_distributions_own() {
    let text: String = stop_failure_drop_in();
    assert_file_shape(&text);
    assert!(holds(&text, "[Service]", "TimeoutStopFailureMode=terminate"), "an overrun ends in SIGKILL, not a coredump: {text}");
    assert!(STOP_FAILURE_DROP_IN_NAME > "10-timeout-abort.conf", "the file name is what decides the winner: {STOP_FAILURE_DROP_IN_NAME}");
    assert_eq!(STOP_FAILURE_MODE_WORD, "terminate");
}

#[test]
fn the_provider_drop_in_asks_for_a_reload_with_the_failure_prefix() {
    let text: String = provider_restart_drop_in();
    assert_file_shape(&text);
    let request: String = format!(
        "ExecStartPost=-/usr/bin/systemctl --no-block --no-ask-password reload-or-restart {BOOT_GATE_UNIT_NAME}"
    );
    assert!(holds(&text, "[Service]", request.as_str()), "a provider restart reloads the gate instead of restarting it: {text}");
}

#[test]
fn a_checkout_path_a_systemd_line_cannot_carry_is_refused() {
    let spaced: CheckoutBootFacts = CheckoutBootFacts { checkout: PathBuf::from("/home/user/Mono chrome"), ..facts() };
    let refusal: UnitTextRefusal = boot_gate_unit(&spaced).expect_err("a space is two words on an Exec line");
    assert!(refusal.message().contains("Mono chrome"), "the refusal names the path: {}", refusal.message());
    assert!(refusal.message().contains("Nothing was written"), "the refusal says nothing happened: {}", refusal.message());

    let relative: CheckoutBootFacts = CheckoutBootFacts { checkout: PathBuf::from("Monochromatic"), ..facts() };
    assert!(matches!(boot_gate_unit(&relative), Err(UnitTextRefusal::PathNotAbsolute { .. })), "a relative path names another folder for every reader");
}

#[test]
fn a_key_unit_that_is_two_words_is_refused() {
    let facts: CheckoutBootFacts = CheckoutBootFacts { key_unit: Some(String::from("zfs-load-key@one.service zfs-load-key@two.service")), ..facts() };
    let refusal: UnitTextRefusal = boot_gate_unit(&facts).expect_err("two units on one line would require both");
    assert!(refusal.message().contains("the key-loading unit's name"), "the refusal names the value: {}", refusal.message());
}
