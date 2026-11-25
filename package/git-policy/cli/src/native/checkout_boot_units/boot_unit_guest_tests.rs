//! What:
//!  The units' guest arm:
//!  the plan a setup run writes is written at the paths systemd reads,
//!  systemd parses every file,
//!  and the loaded units report the ordering,
//! the bounds and the drop-ins the
//!  owner's answers fix.
//! Why:
//!  a unit's text can be right as a string and still be refused by systemd or
//!  carry an order the manager derives differently,
//!  so the plan is proved where a manager exists and nowhere else
//!  (rule `MHX`:
//! a service manager runs in a guest,
//! never on this host).
//!  The gate is only loaded and read,
//! never started:
//!  starting it would run `boot-reconcile` against a machine's own store,
//!  which is the step's own runbook arm and not this text's proof.
//!  Every child runs as a process group of its own with a deadline,
//!  and the files this arm writes are removed on its last line.
//!  On any other machine the arm prints `SKIPPED` and returns,
//!  which is the shape step 8's guest arms use:
//! a harness that skipped an
//!  `#[ignore]`d test would run it nowhere.
//!
//! The driver sets:
//!  `CLI_GIT_ZFS_STEP19_GUEST` to the guest's name,
//!  `CLI_GIT_ZFS_STEP19_CHECKOUT` to the checkout path its pool serves,
//!  `CLI_GIT_ZFS_STEP19_NESTED` to a dataset whose mountpoint lies under the
//!  checkout,
//!  `CLI_GIT_ZFS_STEP19_NESTED_MOUNTPOINT` to that mountpoint,
//!  and `CLI_GIT_ZFS_STEP19_NESTED_SLUG` to the short word its unit's name
//!  carries.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of installPlan(facts, [nested])) writeFileSync(file.path, file.text);
//! expect(await systemctlShow('TimeoutStartUSec')).toBe('5min');
//! ```

use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, InstalledUnitFile, NestedDatasetMount};
use crate::checkout_boot_units::boot_unit_install_plan::install_plan;
use crate::checkout_boot_units::boot_unit_names::BOOT_GATE_UNIT_NAME;
use crate::test_support::spawn_owned;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The variable that says this run is the step 19 guest.
pub const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_GUEST";

/// The checkout the guest's pool serves.
pub const CHECKOUT_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_CHECKOUT";

/// The nested dataset whose mountpoint lies under the checkout.
pub const NESTED_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_NESTED";

/// That dataset's own mountpoint.
pub const NESTED_MOUNTPOINT_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_NESTED_MOUNTPOINT";

/// The short word that dataset's unit name carries.
pub const NESTED_SLUG_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_NESTED_SLUG";

/// The program the units' dependency lines name,
/// which the guest's own packaging installs.
pub const SYSTEMD_ANALYZE: &str = "/usr/bin/systemd-analyze";

/// The manager the guest's own service manager is reached through.
pub const SYSTEMCTL: &str = "/usr/bin/systemctl";

/// How long one child may take before the arm ends its group.
/// Value:
///  120 s,
/// because `systemd-analyze verify` of a handful of units answers in
///  under a second and the manager's own reload in a few.
const COMMAND_DEADLINE: Duration = Duration::from_secs(120);

/// The output one bounded child wrote.
struct Answer {
    /// The bytes it wrote to standard output and error,
    /// joined.
    said: String,
    /// Whether it exited zero.
    succeeded: bool,
}

/// Run one command as a group of its own and wait for it at most `COMMAND_DEADLINE`.
///
/// Why:
///  a hung `systemd-analyze` or manager call would hold the whole test run,
/// so
///  the wait ends and the group is ended with it.
fn run_bounded(program: &str, arguments: &[&str]) -> Answer {
    let mut command: Command = Command::new(program);
    command.args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child: std::process::Child = spawn_owned(&mut command).unwrap_or_else(|error| panic!("{program} starts: {error}"));
    let began: Instant = Instant::now();
    loop {
        match child.try_wait().unwrap_or_else(|error| panic!("{program} answers whether it ended: {error}")) {
            Some(status) => {
                let output = child.wait_with_output().unwrap_or_else(|error| panic!("{program}'s output reads: {error}"));
                let mut said: String = String::from_utf8_lossy(output.stdout.as_slice()).to_string();
                said.push_str(String::from_utf8_lossy(output.stderr.as_slice()).as_ref());
                return Answer { said, succeeded: status.success() };
            }
            None if began.elapsed() >= COMMAND_DEADLINE => {
                crate::child_ended_with_its_whole_group::end_with_its_group(child);
                panic!("{program} {} did not answer within {COMMAND_DEADLINE:?}", arguments.join(" "));
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// The facts the guest's plan is built from.
fn guest_facts() -> CheckoutBootFacts {
    let checkout: String = std::env::var(CHECKOUT_VARIABLE).unwrap_or_else(|_| panic!("{CHECKOUT_VARIABLE} names the guest's checkout"));
    return CheckoutBootFacts {
        checkout: PathBuf::from(checkout),
        root_file: PathBuf::from(crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH),
        key_unit: None,
    };
}

/// The nested dataset the guest's plan carries.
fn guest_nested() -> NestedDatasetMount {
    let dataset: String = std::env::var(NESTED_VARIABLE).unwrap_or_else(|_| panic!("{NESTED_VARIABLE} names the guest's nested dataset"));
    let mountpoint: String = std::env::var(NESTED_MOUNTPOINT_VARIABLE).unwrap_or_else(|_| panic!("{NESTED_MOUNTPOINT_VARIABLE} names its mountpoint"));
    let slug: String = std::env::var(NESTED_SLUG_VARIABLE).unwrap_or_else(|_| panic!("{NESTED_SLUG_VARIABLE} names its unit's short word"));
    return NestedDatasetMount { dataset, mountpoint: PathBuf::from(mountpoint), slug };
}

/// Write one file of the plan at its own path,
/// making the drop-in folders it needs,
/// and give it the mode and owner root writes.
fn write_plan_file(file: &InstalledUnitFile) {
    if let Some(folder) = file.path.parent() {
        std::fs::create_dir_all(folder).unwrap_or_else(|error| panic!("{} is made: {error}", folder.display()));
    }
    std::fs::write(file.path.as_path(), file.text.as_bytes()).unwrap_or_else(|error| panic!("{} writes: {error}", file.path.display()));
    std::fs::set_permissions(file.path.as_path(), std::os::unix::fs::PermissionsExt::from_mode(0o644))
        .unwrap_or_else(|error| panic!("{} takes its mode: {error}", file.path.display()));
}

/// Whether one loaded unit's property carries the wanted text.
fn property_holds(unit: &str, property: &str, wanted: &str) {
    let answer: Answer = run_bounded(SYSTEMCTL, &["show", "--property", property, unit]);
    assert!(answer.succeeded, "`systemctl show --property {property} {unit}` answers: {}", answer.said);
    assert!(
        answer.said.contains(wanted),
        "`{unit}` does not report `{wanted}` in `{property}`; the manager read: {}",
        answer.said
    );
}

#[test]
fn the_plan_parses_and_reports_the_owner_s_order_in_a_guest() {
    if std::env::var_os(GUEST_VARIABLE).is_none() {
        println!("SKIPPED the units' guest arm: {GUEST_VARIABLE} is unset, so this is not the step 19 guest");
        return;
    }
    let facts: CheckoutBootFacts = guest_facts();
    let nested: NestedDatasetMount = guest_nested();
    let plan: Vec<InstalledUnitFile> = install_plan(&facts, &[nested]).unwrap_or_else(|refusal| panic!("{}", refusal.message()));
    for file in &plan {
        write_plan_file(file);
    }
    for file in &plan {
        let path: &str = file.path.to_str().unwrap_or_else(|| panic!("a plan path is text"));
        let answer: Answer = run_bounded(SYSTEMD_ANALYZE, &["verify", path]);
        assert!(answer.succeeded, "systemd refuses `{path}`: {}\n{}", answer.said, file.text);
        println!("STEP19-GUEST verified={path}");
    }
    let reload: Answer = run_bounded(SYSTEMCTL, &["daemon-reload"]);
    assert!(reload.succeeded, "the manager reloads the units: {}", reload.said);

    property_holds(BOOT_GATE_UNIT_NAME, "TimeoutStartUSec", "5min");
    property_holds(BOOT_GATE_UNIT_NAME, "TimeoutStopUSec", "1min");
    property_holds(BOOT_GATE_UNIT_NAME, "Wants", "zfs-mount.service");
    property_holds(BOOT_GATE_UNIT_NAME, "After", "zfs-mount.service");
    property_holds(BOOT_GATE_UNIT_NAME, "After", "zfs-import.target");
    property_holds(BOOT_GATE_UNIT_NAME, "After", "-.mount");
    property_holds(BOOT_GATE_UNIT_NAME, "Before", "umount.target");
    property_holds(BOOT_GATE_UNIT_NAME, "ExecReload", "boot-reconcile");
    property_holds(BOOT_GATE_UNIT_NAME, "ExecStopPost", "shutdown-unmount");
    property_holds(BOOT_GATE_UNIT_NAME, "Type", "oneshot");
    property_holds(BOOT_GATE_UNIT_NAME, "RemainAfterExit", "yes");
    let before: Answer = run_bounded(SYSTEMCTL, &["show", "--property", "Before", BOOT_GATE_UNIT_NAME]);
    assert!(!before.said.contains("systemd-user-sessions"), "no login waits for the gate (V8a): {}", before.said);
    let drop_ins: Answer = run_bounded(SYSTEMCTL, &["show", "--property", "DropInPaths", BOOT_GATE_UNIT_NAME]);
    assert!(drop_ins.said.contains("zz-checkout-stop-failure-mode.conf"), "the stop-failure drop-in is read: {}", drop_ins.said);
    println!("STEP19-GUEST gate-orders-and-bounds-read");

    let slug: String = std::env::var(NESTED_SLUG_VARIABLE).unwrap_or_else(|_| panic!("{NESTED_SLUG_VARIABLE} names its unit's short word"));
    let nested_unit: String = crate::checkout_boot_units::boot_unit_names::nested_unit_name(slug.as_str());
    property_holds(nested_unit.as_str(), "Requires", BOOT_GATE_UNIT_NAME);
    property_holds(nested_unit.as_str(), "After", BOOT_GATE_UNIT_NAME);
    let condition: Answer = run_bounded(SYSTEMCTL, &["show", "--property", "ConditionPathIsMountPoint", nested_unit.as_str()]);
    assert!(condition.succeeded, "the nested unit's condition reads: {}", condition.said);
    assert!(condition.said.contains("ConditionPathIsMountPoint=!"), "the condition skips an already-mounted dataset: {}", condition.said);
    println!("STEP19-GUEST nested-unit-orders-after-the-gate");

    for file in plan.iter().rev() {
        std::fs::remove_file(file.path.as_path()).unwrap_or_else(|error| panic!("{} removes: {error}", file.path.display()));
        if let Some(folder) = file.path.parent() {
            let _ = std::fs::remove_dir(folder);
        }
    }
    let again: Answer = run_bounded(SYSTEMCTL, &["daemon-reload"]);
    assert!(again.succeeded, "the manager reloads without the units: {}", again.said);
    println!("STEP19-GUEST plan-written-parsed-and-removed");
}
