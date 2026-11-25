//! What:
//!  The gate's own unit text,
//!  the stop-failure drop-in beside it,
//!  and the provider drop-in that asks the gate to reconcile after the provider
//!  ran.
//! Why:
//!  The owner's answers of 2026-11-24 fix every line:
//!  `Wants=` and `After=` for the provider (V2),
//!  one `RequiresMountsFor=` (V3),
//!  no before-login ordering (V8a),
//!  and the two bounds (V8b).
//!  The key lines are conditional,
//! because a unit that requires a unit the boot
//!  does not install stays `inactive` with `Result=success` and writes no
//!  journal line at all,
//! which is a gate that silently never runs
//!  (`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
//!  "The boot and shutdown unit",
//!  "Why each line";
//!  `doc/decision/cli-git-zfs-step-19-boot-mount-provider-and-ordering-choices.md`,
//!  V1 to V8).
//!  The provider drop-in is what re-runs the gate after a post-key-load checkout
//!  mount,
//! and it is a reload so a provider restart leaves the gate and every
//!  attached clone alone
//!  (`doc/decision/cli-git-zfs-lost-event-triggers-hook-nudge-and-cost-cap.md`,
//!  "Which trigger runs after the checkout becomes ready").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! write('/etc/systemd/system/reconcile-…service', bootGateUnit(facts));
//! ```

use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, UnitTextRefusal, path_text};
use crate::checkout_boot_units::unit_file_text::{file_text, path_comment};
use std::path::{Path, PathBuf};
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME, BOOT_GATE_UNIT_PATH, BOOT_RECONCILE_MODE_WORD, EXECUTABLE_SYSTEMCTL, GATE_START_BOUND, GATE_STOP_BOUND, PROVIDER_DROP_IN_NAME, PROVIDER_UNIT_NAME,
    SHUTDOWN_UNMOUNT_MODE_WORD, START_PROBE_MODE_WORD, STOP_FAILURE_DROP_IN_NAME, STOP_FAILURE_MODE_WORD, drop_in_path, root_words,
};

/// The word before the start probe's line that keeps a failing probe from
/// failing the gate.
/// Value:
///  the dash systemd reads as "ignore this command's failure".
pub const FAILURE_IGNORED_PREFIX: char = '-';

/// What:
///  The gate,
/// whole:
///  the unit that reconciles the managed clone mounts at boot,
///  reloads on the provider's request,
///  and unmounts what this boot verified at shutdown.
/// Why:
///  the plan installs this text and the design's guest measured it booting
///  green with the same order the owner's answers describe,
/// so the text is
///  written here and asserted rather than retyped by each setup run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = bootGateUnit({ checkout, rootFile, keyUnit: null });
/// ```
///
/// # Errors
/// The facts carry a path or a unit name a systemd line cannot take.
pub fn boot_gate_unit(facts: &CheckoutBootFacts) -> Result<String, UnitTextRefusal> {
    facts.checked()?;
    let checkout: &str = path_text("the checkout path", facts.checkout.as_path())?;
    let root: &str = path_text("the root-owned file's path", facts.root_file.as_path())?;
    let words: String = root_words();
    let commands: String = format!("{root} {words}");
    let start_probe: String = format!("{FAILURE_IGNORED_PREFIX}{commands} {START_PROBE_MODE_WORD}");
    let reconcile: String = format!("{commands} {BOOT_RECONCILE_MODE_WORD}");
    let stop: String = format!("{FAILURE_IGNORED_PREFIX}{commands} {SHUTDOWN_UNMOUNT_MODE_WORD}");
    let mut lines: Vec<String> = vec![
        path_comment(Path::new(BOOT_GATE_UNIT_PATH)),
        String::from("[Unit]"),
        String::from("Description=Reconcile the managed worktree clone mounts"),
    ];
    if let Some(key_unit) = &facts.key_unit {
        lines.push(format!("Requires={key_unit}"));
        lines.push(format!("After={key_unit}"));
    }
    lines.extend([
        format!("Wants={PROVIDER_UNIT_NAME}"),
        format!("After={PROVIDER_UNIT_NAME} zfs-import.target"),
        format!("RequiresMountsFor={checkout}"),
        String::from("Before=umount.target"),
        String::from("Conflicts=umount.target"),
        String::new(),
        String::from("[Service]"),
        String::from("Type=oneshot"),
        String::from("RemainAfterExit=yes"),
        format!("TimeoutStartSec={GATE_START_BOUND}"),
        format!("TimeoutStopSec={GATE_STOP_BOUND}"),
        format!("ExecStartPre={start_probe}"),
        format!("ExecStart={reconcile}"),
        format!("ExecReload={reconcile}"),
        format!("ExecStopPost={stop}"),
        String::new(),
        String::from("[Install]"),
        String::from("WantedBy=multi-user.target"),
    ]);
    return Ok(file_text(lines));
}

/// What:
///  The drop-in that decides what a bound does when it fires:
///  `TimeoutStopFailureMode=terminate`.
/// Why:
///  a machine whose packaging sets `abort` sends SIGABRT to an overrunning stop
///  stage and leaves a coredump;
///  this file's name sorts after Fedora's own `10-timeout-abort.conf`,
/// so the
///  value it sets is the one in force.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = stopFailureDropIn(); // '[Service]\nTimeoutStopFailureMode=terminate\n'
/// ```
pub fn stop_failure_drop_in() -> String {
    let path: PathBuf = drop_in_path(BOOT_GATE_UNIT_NAME, STOP_FAILURE_DROP_IN_NAME);
    return file_text(vec![path_comment(path.as_path()), String::from("[Service]"), format!("TimeoutStopFailureMode={STOP_FAILURE_MODE_WORD}")]);
}

/// What:
///  The provider's drop-in:
///  after `zfs mount -a` ran,
/// ask the gate to reconcile by a reload.
/// Why:
///  a key-load event can arrive before the checkout is mounted,
///  so the trigger that must run afterwards belongs to the provider whose
///  command brought the checkout up,
///  including a provider started by something other than the boot.
///  The dash prefix keeps the provider green when the request cannot be queued,
///  and the request is a reload so that no stop pass runs and no attached clone
///  is unmounted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = providerRestartDropIn(); // '[Service]\nExecStartPost=-/usr/bin/systemctl --no-block … reload-or-restart reconcile-…service\n'
/// ```
pub fn provider_restart_drop_in() -> String {
    let request: String = format!("{EXECUTABLE_SYSTEMCTL} --no-block --no-ask-password reload-or-restart {BOOT_GATE_UNIT_NAME}");
    let line: String = format!("{FAILURE_IGNORED_PREFIX}{request}");
    let path: PathBuf = drop_in_path(PROVIDER_UNIT_NAME, PROVIDER_DROP_IN_NAME);
    return file_text(vec![path_comment(path.as_path()), String::from("[Service]"), format!("ExecStartPost={line}")]);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "boot_gate_unit_text_tests.rs"]
mod tests;
