//! What:
//!  The units' guest arm:
//!  the plan a setup run writes is written at the paths systemd reads,
//!  every unit file is parsed by `systemd-analyze verify`,
//!  and the loaded units report the ordering,
//! the bounds and the drop-ins the owner's answers fix.
//! Why:
//!  a unit's text can be right as a string and still be refused by systemd or
//!  carry an order the manager derives differently,
//!  so the plan is proved where a manager exists and nowhere else
//!  (rule `MHX`:
//! a service manager runs in a guest,
//! never on this host).
//!  Only unit files go to the verifier:
//!  `systemd-analyze verify` refuses any drop-in path by name
//!  (`Failed to prepare filename .../x.service.d/y.conf: Invalid argument`,
//!  measured in the step 19 guest of 2026-11-27),
//! so a drop-in is proved
//!  through the manager's own view of the unit it belongs to.
//!  A unit whose `BindsTo=` names the checkout's mount unit cannot be verified
//!  offline either,
//!  because the manager learns that unit from the mount table and writes no file
//!  (measured the same day:
//! `Failed to create checkout-propagation.service/start:
//!  Unit home-....mount not found`),
//!  so the pair is proved through the manager's `Upholds=` and `UpheldBy=`.
//!  The gate is only loaded and read,
//! never started:
//!  starting it would run `boot-reconcile` against a machine's own store,
//!  which is the reboot arm's own work and not this text's proof.
//!  Every child runs with a deadline,
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

use crate::checkout_boot_units::boot_unit_facts::{
    CheckoutBootFacts,
    InstalledUnitFile,
    NestedDatasetMount,
};
use crate::checkout_boot_units::boot_unit_install_plan::install_plan;
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME,
    PROPAGATION_SERVICE_NAME,
    PROVIDER_UNIT_NAME,
    mount_unit_name,
    nested_unit_name,
};
use crate::checkout_boot_units::unit_text_sections::unit_files_of;
use crate::test_support::spawn_owned;
use std::path::PathBuf;
use std::process::{
    Command,
    Stdio,
};
use std::time::{
    Duration,
    Instant,
};

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

/// The key provider the machine has,
/// where it has one,
/// which the gate requires and orders after.
pub const KEY_UNIT_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_KEY_UNIT";

/// The program the units' dependency lines name,
/// which the guest's own packaging installs.
pub const SYSTEMD_ANALYZE: &str = "/usr/bin/systemd-analyze";

/// The manager the guest's own service manager is reached through.
pub const SYSTEMCTL: &str = "/usr/bin/systemctl";

/// How long one child may take before the arm ends it.
/// Value:
///  120 s,
/// because `systemd-analyze verify` of a handful of units answers in
///  under a second and the manager's own reload in a few.
const COMMAND_DEADLINE: Duration = Duration::from_secs(120);

/// The output one bounded child wrote.
pub(crate) struct Answer {
    /// The bytes it wrote to standard output and error,
    /// joined.
    pub said: String,
    /// Whether it exited zero.
    pub succeeded: bool,
}

/// Run one command and wait for it at most `COMMAND_DEADLINE`.
///
/// Why:
///  a hung `systemd-analyze` or manager call would hold the whole test run,
/// so
///  the wait ends and the child is ended with it.
pub(crate) fn run_bounded(
    program: &str,
    arguments: &[&str],
) -> Answer {
    let mut command: Command = Command::new(program);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child: std::process::Child =
        spawn_owned(&mut command).unwrap_or_else(|error| panic!("{program} starts: {error}"));
    let began: Instant = Instant::now();
    loop {
        match child
            .try_wait()
            .unwrap_or_else(|error| panic!("{program} answers whether it ended: {error}"))
        {
            Some(status) => {
                let output = child
                    .wait_with_output()
                    .unwrap_or_else(|error| panic!("{program}'s output reads: {error}"));
                let mut said: String =
                    String::from_utf8_lossy(output.stdout.as_slice()).to_string();
                said.push_str(String::from_utf8_lossy(output.stderr.as_slice()).as_ref());
                return Answer {
                    said,
                    succeeded: status.success(),
                };
            }
            None if began.elapsed() >= COMMAND_DEADLINE => {
                crate::child_ended_with_its_whole_group::end_with_its_group(child);
                panic!(
                    "{program} {} did not answer within {COMMAND_DEADLINE:?}",
                    arguments.join(" ")
                );
            }
            None => std::thread::sleep(Duration::from_millis(10)),
        }
    }
}

/// The facts the guest's plan is built from.
fn guest_facts() -> CheckoutBootFacts {
    let checkout: String = std::env::var(CHECKOUT_VARIABLE)
        .unwrap_or_else(|_| panic!("{CHECKOUT_VARIABLE} names the guest's checkout"));
    return CheckoutBootFacts {
        checkout: PathBuf::from(checkout),
        root_file: PathBuf::from(
            crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH,
        ),
        key_unit: std::env::var(KEY_UNIT_VARIABLE).ok(),
    };
}

/// The nested dataset the guest's plan carries.
fn guest_nested() -> NestedDatasetMount {
    let dataset: String = std::env::var(NESTED_VARIABLE)
        .unwrap_or_else(|_| panic!("{NESTED_VARIABLE} names the guest's nested dataset"));
    let mountpoint: String = std::env::var(NESTED_MOUNTPOINT_VARIABLE)
        .unwrap_or_else(|_| panic!("{NESTED_MOUNTPOINT_VARIABLE} names its mountpoint"));
    let slug: String = std::env::var(NESTED_SLUG_VARIABLE)
        .unwrap_or_else(|_| panic!("{NESTED_SLUG_VARIABLE} names its unit's short word"));
    return NestedDatasetMount {
        dataset,
        mountpoint: PathBuf::from(mountpoint),
        slug,
    };
}

/// Write one file of the plan at its own path,
/// making the drop-in folders it needs,
/// and give it the mode and owner root writes.
fn write_plan_file(file: &InstalledUnitFile) {
    if let Some(folder) = file.path.parent() {
        std::fs::create_dir_all(folder)
            .unwrap_or_else(|error| panic!("{} is made: {error}", folder.display()));
    }
    std::fs::write(file.path.as_path(), file.text.as_bytes())
        .unwrap_or_else(|error| panic!("{} writes: {error}", file.path.display()));
    std::fs::set_permissions(
        file.path.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o644),
    )
    .unwrap_or_else(|error| panic!("{} takes its mode: {error}", file.path.display()));
}

/// What:
///  One property line the manager printed,
///  with the quoting `systemctl show` adds undone.
/// Why:
///  a property value is printed with C-style escaping,
///  so a unit name `systemd-escape`
///  wrote with backslashes answers with each of them doubled:
///  the step 19 guest of
///  2026-11-27 read `UpheldBy="home-zfs\\x2dstep19\\x2dfixture..."` for the
///  checkout's own mount unit,
///  and a comparison against the escaped name would fail on the manager's
///  spelling rather than on the dependency.
fn unquoted(said: &str) -> String {
    return said.replace("\\\\", "\\");
}

/// Ask the manager for one unit's property and require it to carry a text.
fn property_holds(
    unit: &str,
    property: &str,
    wanted: &str,
) {
    let answer: Answer = run_bounded(SYSTEMCTL, &["show", "--property", property, unit]);
    assert!(
        answer.succeeded,
        "`systemctl show --property {property} {unit}` answers: {}",
        answer.said
    );
    assert!(
        unquoted(answer.said.as_str()).contains(wanted),
        "`{unit}` does not report `{wanted}` in `{property}`; the manager read: {}",
        answer.said
    );
}

/// What:
///  Parse every unit file of the plan the way a machine's systemd parses it.
/// Why:
///  the verifier takes unit files and nothing else,
///  so the plan's drop-ins are proved by the manager's own reading of the unit
///  they belong to and never here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const path of unitFilesOf(plan)) expect(analyzeVerify(path).status).toBe(0);
/// ```
fn verify_the_unit_files(plan: &[InstalledUnitFile]) {
    let checkout: PathBuf = guest_facts().checkout;
    let mount_unit: String = mount_unit_name(checkout.as_path());
    for path in unit_files_of(plan) {
        let text: &str = path
            .to_str()
            .unwrap_or_else(|| panic!("a plan path is text"));
        let answer: Answer = run_bounded(SYSTEMD_ANALYZE, &["verify", text]);
        if path.ends_with(PROPAGATION_SERVICE_NAME) {
            // The manager learns the checkout's mount unit from the mount table,
            // and the offline verifier reads files alone, so its one line names
            // the unit the running manager does resolve.
            for line in answer.said.lines().filter(|line| return !line.is_empty()) {
                assert!(
                    line.contains(mount_unit.as_str()),
                    "`systemd-analyze verify {text}` said `{line}`, which names no missing mount unit of the pair"
                );
            }
            println!(
                "STEP19-GUEST propagation-pair-offline-limit={}",
                answer.said.trim()
            );
            continue;
        }
        assert!(
            answer.succeeded,
            "systemd refuses `{text}`: {}",
            answer.said
        );
        println!("STEP19-GUEST verified={text}");
    }
}

/// What:
///  Read the drop-ins back through the manager,
///  which is the only reader that resolves a unit it learned from the mount
///  table.
/// Why:
///  the verifier refuses a drop-in path,
///  so a drop-in proved nowhere would be a file nothing ever parsed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await show('DropInPaths', gate)).toContain('zz-checkout-stop-failure-mode.conf');
/// ```
fn read_the_drop_ins_back() {
    let checkout: PathBuf = guest_facts().checkout;
    let mount_unit: String = mount_unit_name(checkout.as_path());
    property_holds(
        BOOT_GATE_UNIT_NAME,
        "DropInPaths",
        "zz-checkout-stop-failure-mode.conf",
    );
    property_holds(
        PROVIDER_UNIT_NAME,
        "DropInPaths",
        "reconcile-git-worktree-clone-mounts-after-checkout.conf",
    );
    property_holds(PROVIDER_UNIT_NAME, "ExecStartPost", "reload-or-restart");
    property_holds(PROVIDER_UNIT_NAME, "ExecStartPost", BOOT_GATE_UNIT_NAME);
    property_holds(
        mount_unit.as_str(),
        "DropInPaths",
        "10-upholds-propagation.conf",
    );
    property_holds(mount_unit.as_str(), "Upholds", PROPAGATION_SERVICE_NAME);
    property_holds(PROPAGATION_SERVICE_NAME, "UpheldBy", mount_unit.as_str());
    property_holds(PROPAGATION_SERVICE_NAME, "BindsTo", mount_unit.as_str());
    println!("STEP19-GUEST drop-ins-read");
}

#[test]
fn the_plan_parses_and_reports_the_owner_s_order_in_a_guest() {
    if std::env::var_os(GUEST_VARIABLE).is_none() {
        println!(
            "SKIPPED the units' guest arm: {GUEST_VARIABLE} is unset, so this is not the step 19 guest"
        );
        return;
    }
    let facts: CheckoutBootFacts = guest_facts();
    let nested: NestedDatasetMount = guest_nested();
    let plan: Vec<InstalledUnitFile> =
        install_plan(&facts, &[nested]).unwrap_or_else(|refusal| panic!("{}", refusal.message()));
    for file in &plan {
        write_plan_file(file);
    }
    verify_the_unit_files(&plan);
    let reload: Answer = run_bounded(SYSTEMCTL, &["daemon-reload"]);
    assert!(
        reload.succeeded,
        "the manager reloads the units: {}",
        reload.said
    );

    property_holds(BOOT_GATE_UNIT_NAME, "TimeoutStartUSec", "5min");
    property_holds(BOOT_GATE_UNIT_NAME, "TimeoutStopUSec", "1min");
    property_holds(BOOT_GATE_UNIT_NAME, "Wants", PROVIDER_UNIT_NAME);
    property_holds(BOOT_GATE_UNIT_NAME, "After", PROVIDER_UNIT_NAME);
    property_holds(BOOT_GATE_UNIT_NAME, "After", "zfs-import.target");
    property_holds(BOOT_GATE_UNIT_NAME, "After", "-.mount");
    property_holds(BOOT_GATE_UNIT_NAME, "Before", "umount.target");
    property_holds(BOOT_GATE_UNIT_NAME, "ExecReload", "boot-reconcile");
    property_holds(BOOT_GATE_UNIT_NAME, "ExecStopPost", "shutdown-unmount");
    property_holds(BOOT_GATE_UNIT_NAME, "Type", "oneshot");
    property_holds(BOOT_GATE_UNIT_NAME, "RemainAfterExit", "yes");
    let before: Answer = run_bounded(
        SYSTEMCTL,
        &["show", "--property", "Before", BOOT_GATE_UNIT_NAME],
    );
    assert!(
        !before.said.contains("systemd-user-sessions"),
        "no login waits for the gate (V8a): {}",
        before.said
    );
    println!("STEP19-GUEST gate-orders-and-bounds-read");
    read_the_drop_ins_back();

    let slug: String = std::env::var(NESTED_SLUG_VARIABLE)
        .unwrap_or_else(|_| panic!("{NESTED_SLUG_VARIABLE} names its unit's short word"));
    let nested_unit: String = nested_unit_name(slug.as_str());
    property_holds(nested_unit.as_str(), "Requires", BOOT_GATE_UNIT_NAME);
    property_holds(nested_unit.as_str(), "After", BOOT_GATE_UNIT_NAME);
    // The manager prints neither `ConditionPathIsMountPoint=` nor `Conditions=`:
    // `systemctl show --property Conditions` answers `Conditions=[unprintable]` on
    // this guest's systemd 255,
    // measured 2026-11-27,
    // and the verifier has already parsed the line that holds the condition.
    // What the manager can prove here is that it loaded the unit at all;
    // the condition's own effect is the boot arm's,
    // where the nested dataset is mounted only after the gate and never on a
    // boot whose checkout did not come up.
    property_holds(nested_unit.as_str(), "LoadState", "loaded");
    println!("STEP19-GUEST nested-unit-orders-after-the-gate");

    for file in plan.iter().rev() {
        std::fs::remove_file(file.path.as_path())
            .unwrap_or_else(|error| panic!("{} removes: {error}", file.path.display()));
        if let Some(folder) = file.path.parent() {
            let _ = std::fs::remove_dir(folder);
        }
    }
    let again: Answer = run_bounded(SYSTEMCTL, &["daemon-reload"]);
    assert!(
        again.succeeded,
        "the manager reloads without the units: {}",
        again.said
    );
    println!("STEP19-GUEST plan-written-parsed-and-removed");
}

/// The boot arm:
/// one plan written where systemd reads it,
/// the guest's own
/// reboot between two readings of it.
/// It shares this file's bounded child and its variables,
/// because a second copy of either drifts from the first.
#[cfg(test)]
#[path = "boot_unit_reboot_guest_tests.rs"]
mod reboot_tests;
