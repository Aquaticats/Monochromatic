//! What:
//!  The plan's boot arm,
//! in three phases a driver splits across a guest's own
//!  reboots:
//!  write every file of the plan where systemd reads it and enable the gate,
//!  then read what the manager did with them after the guest rebooted,
//!  then take them away again.
//! Why:
//!  A unit text can be right as a string and its ordering still decide
//!  differently once a boot runs it,
//!  so the plan is installed and the guest rebooted,
//! and the reading after the
//!  reboot names the manager's own state rather than the file it parsed
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 19,
//!  "Acceptance tests":
//!  "Perform an actual reboot and shutdown with idle and busy legacy clones",
//!  "Delay or fail import,
//!  key loading and checkout mounting;
//!  assert no nested dataset attaches or creates mountpoint folders in the
//!  home underlay").
//!  The phases are separate runs because no test of this crate reboots the
//!  machine under itself:
//!  the driver starts the reboot between them and waits until the guest answers
//!  again (the shape step 10's own split arms use).
//!  On any other machine every phase prints `SKIPPED` and returns.
//!
//! The driver sets:
//!  `CLI_GIT_ZFS_STEP19_GUEST` to the guest's name,
//!  `CLI_GIT_ZFS_STEP19_PHASE` to `install`,
//! `read` or `remove`,
//!  `CLI_GIT_ZFS_STEP19_BOOT_EXPECT` to `healthy` or `failed` while reading,
//!  `CLI_GIT_ZFS_STEP19_UNDERLAY` to a folder on the home dataset the checkout
//!  path falls through to while it is unmounted,
//!  `CLI_GIT_ZFS_STEP19_NESTED_MOUNTPOINT` to the nested dataset's mountpoint,
//!  and `CLI_GIT_ZFS_STEP19_SLOT` to the slot of one managed clone while a
//!  clone is expected to stand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writePlan(installPlan(facts, [nested])); await exec('systemctl enable --now reconcile-...service');
//! // the driver reboots here
//! await waitForGate(); expect(await gateProperty('ActiveState')).toBe('active');
//! ```

use super::{
    CHECKOUT_VARIABLE,
    GUEST_VARIABLE,
    NESTED_MOUNTPOINT_VARIABLE,
    NESTED_SLUG_VARIABLE,
    NESTED_VARIABLE,
    SYSTEMCTL,
    run_bounded,
};
use crate::checkout_boot_units::boot_unit_facts::{
    CheckoutBootFacts,
    InstalledUnitFile,
    NestedDatasetMount,
};
use crate::checkout_boot_units::boot_unit_install_plan::install_plan;
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME,
    nested_unit_name,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    Instant,
};

/// The phase the driver asks for.
pub const PHASE_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_PHASE";

/// What the reading after a reboot expects of the gate.
pub const EXPECT_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_BOOT_EXPECT";

/// The folder the checkout path falls through to while it is unmounted.
pub const UNDERLAY_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_UNDERLAY";

/// The slot of one managed clone,
/// whose mount a healthy boot must have taken again.
pub const SLOT_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_SLOT";

/// The OpenZFS userspace this phase starts,
/// which the recipe installed outside the guest's own `PATH`.
pub const SBIN_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_SBIN";

/// The checkout dataset the add route snapshots.
pub const CHECKOUT_DATASET_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_CHECKOUT_DATASET";

/// The clone container the add route clones into.
pub const CONTAINER_VARIABLE: &str = "CLI_GIT_ZFS_STEP19_CONTAINER";

/// The readiness record the gate publishes for this boot.
pub const READINESS_RECORD: &str = "/run/git-worktree-clone-mount-readiness/result";

/// The kernel file that names this boot.
pub const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";

/// The kernel's mount table,
/// which is where an attached clone and a mounted nested dataset appear.
pub const MOUNT_TABLE: &str = "/proc/self/mountinfo";

/// How long a reading waits for the gate to settle after a boot.
/// Value:
///  180 s,
/// which is the owner's own start bound of `5min` less the part of it a boot
///  spends before the gate is started,
/// and the wait answers at once in every run measured so far.
const SETTLE_DEADLINE: Duration = Duration::from_secs(180);

/// How often the wait asks the manager.
const SETTLE_PAUSE: Duration = Duration::from_millis(500);

/// The facts the plan is built from,
/// read from the variables the driver set.
fn facts() -> CheckoutBootFacts {
    let checkout: String = std::env::var(CHECKOUT_VARIABLE)
        .unwrap_or_else(|_| panic!("{CHECKOUT_VARIABLE} names the guest's checkout"));
    return CheckoutBootFacts {
        checkout: PathBuf::from(checkout),
        root_file: PathBuf::from(
            crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH,
        ),
        key_unit: std::env::var(super::KEY_UNIT_VARIABLE).ok(),
    };
}

/// The nested dataset the plan carries.
fn nested() -> NestedDatasetMount {
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

/// Write one file of the plan at its path and give it the mode a setup writes.
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

/// Ask the manager for one property of one unit.
fn property_of(
    unit: &str,
    property: &str,
) -> String {
    let answer = run_bounded(SYSTEMCTL, &["show", "--property", property, unit]);
    assert!(
        answer.succeeded,
        "`systemctl show --property {property} {unit}` answers: {}",
        answer.said
    );
    return answer.said.trim().to_string();
}

/// Whether one path is a mount in the kernel's own table.
///
/// Why:
///  the answer comes from the kernel rather than from a `findmnt` that could be
///  reading another namespace,
/// and the guest's own root account reads its own table.
fn is_mounted(path: &Path) -> bool {
    let table: String = std::fs::read_to_string(MOUNT_TABLE)
        .unwrap_or_else(|error| panic!("{MOUNT_TABLE} reads: {error}"));
    let wanted: String = format!(" {} ", path.display());
    return table
        .lines()
        .any(|line| return line.contains(wanted.as_str()));
}

/// The gate's state,
/// waiting for it to leave `activating` and `deactivating`.
///
/// Why:
///  the driver's reboot answers when the guest runs commands again,
/// and the gate may still be starting the sweep at that moment,
/// so a reading taken at once would name a state the boot has not reached.
fn settled_gate_state() -> String {
    let began: Instant = Instant::now();
    loop {
        let state: String = property_of(BOOT_GATE_UNIT_NAME, "ActiveState");
        let word: &str = state
            .strip_prefix("ActiveState=")
            .unwrap_or_else(|| panic!("the manager's answer is a property line: {state}"));
        if word != "activating" && word != "deactivating" {
            return String::from(word);
        }
        assert!(
            began.elapsed() < SETTLE_DEADLINE,
            "the gate is still `{word}` {SETTLE_DEADLINE:?} after the guest answered again"
        );
        std::thread::sleep(SETTLE_PAUSE);
    }
}

/// The readiness record's lines,
/// or nothing when the gate published none.
fn readiness_record() -> Option<String> {
    return std::fs::read_to_string(READINESS_RECORD).ok();
}

/// This boot's own identifier.
fn this_boot() -> String {
    return std::fs::read_to_string(BOOT_ID_PATH)
        .unwrap_or_else(|error| panic!("{BOOT_ID_PATH} reads: {error}"))
        .trim()
        .to_string();
}

/// Write the plan and enable the gate and the nested dataset's unit.
fn install() {
    let plan: Vec<InstalledUnitFile> = install_plan(&facts(), &[nested()])
        .unwrap_or_else(|refusal| panic!("{}", refusal.message()));
    for file in &plan {
        write_plan_file(file);
    }
    let reload = run_bounded(SYSTEMCTL, &["daemon-reload"]);
    assert!(reload.succeeded, "the manager reloads: {}", reload.said);
    let slug: String = std::env::var(NESTED_SLUG_VARIABLE)
        .unwrap_or_else(|_| panic!("{NESTED_SLUG_VARIABLE} names its unit's short word"));
    let nested_unit: String = nested_unit_name(slug.as_str());
    for unit in [BOOT_GATE_UNIT_NAME, nested_unit.as_str()] {
        let enabled = run_bounded(SYSTEMCTL, &["enable", unit]);
        assert!(
            enabled.succeeded,
            "`{unit}` is enabled by its own `[Install]` line: {}",
            enabled.said
        );
        println!("STEP19-BOOT enabled={unit}");
    }
    println!("STEP19-BOOT installed files={}", plan.len());
}

/// Read what this boot did with the plan,
/// and judge it against what the driver said the boot was.
fn read() {
    let expectation: String = std::env::var(EXPECT_VARIABLE)
        .unwrap_or_else(|_| panic!("{EXPECT_VARIABLE} says what this boot expects"));
    let state: String = settled_gate_state();
    let result: String = property_of(BOOT_GATE_UNIT_NAME, "Result");
    let boot: String = this_boot();
    let published: Option<String> = readiness_record();
    let nested_mountpoint: PathBuf = nested().mountpoint;
    let nested_mounted: bool = is_mounted(nested_mountpoint.as_path());
    let slug: String = std::env::var(NESTED_SLUG_VARIABLE)
        .unwrap_or_else(|_| panic!("{NESTED_SLUG_VARIABLE} names its unit's short word"));
    let nested_unit: String = nested_unit_name(slug.as_str());
    let nested_state: String = property_of(nested_unit.as_str(), "ActiveState");
    let underlay: String = std::env::var(UNDERLAY_VARIABLE)
        .unwrap_or_else(|_| panic!("{UNDERLAY_VARIABLE} names the home underlay"));
    let underlay_nested: PathBuf = Path::new(underlay.as_str()).join(
        nested_mountpoint
            .strip_prefix(facts().checkout.as_path())
            .unwrap_or_else(|_| panic!("the nested dataset's mountpoint lies inside the checkout")),
    );
    println!(
        "STEP19-BOOT read expect={expectation} gate={state} {result} nested-unit={nested_state} nested-mounted={nested_mounted} underlay-folder={}",
        underlay_nested.exists()
    );
    if expectation == "healthy" {
        assert_eq!(state, "active", "the gate ran and stayed up: {result}");
        let record: String = published
            .clone()
            .unwrap_or_else(|| panic!("a healthy boot publishes this boot's readiness record"));
        assert!(
            record.contains(format!("boot {boot}").as_str()) || record.contains(boot.as_str()),
            "the readiness record names this boot: {record}"
        );
        assert!(
            record.contains("sweep finished"),
            "the sweep finished: {record}"
        );
        assert!(
            nested_mounted,
            "the nested dataset's unit mounted it under the verified checkout: {}",
            nested_mountpoint.display()
        );
        if let Ok(slot) = std::env::var(SLOT_VARIABLE) {
            let slot_path: PathBuf = facts().checkout.join(".claude/worktrees").join(slot);
            assert!(
                is_mounted(slot_path.as_path()),
                "the gate attached the idle clone at {} after the reboot",
                slot_path.display()
            );
            println!("STEP19-BOOT clone-attached={}", slot_path.display());
        }
        println!("STEP19-BOOT healthy-boot read");
        return;
    }
    // A boot whose import, key load or checkout mount did not happen:
    // the gate refuses, no nested dataset is attached, and nothing created a
    // mountpoint folder in the home underlay.
    assert_ne!(
        state, "active",
        "a boot without its checkout does not run the gate"
    );
    assert!(
        !nested_mounted,
        "no nested dataset attaches without the checkout: {}",
        nested_mountpoint.display()
    );
    assert_eq!(
        nested_state, "ActiveState=inactive",
        "the nested dataset's unit never ran, because the gate it requires did not"
    );
    assert!(
        !underlay_nested.exists(),
        "no unit made the nested mountpoint in the home underlay: {}",
        underlay_nested.display()
    );
    match published {
        Some(record) => {
            assert!(
                !record.contains(boot.as_str()),
                "this boot published no readiness: {record}"
            );
        }
        None => println!("STEP19-BOOT no-readiness-record=true"),
    }
    println!("STEP19-BOOT failed-boot read");
}

/// Take the plan's files away again and stop the units from starting.
fn remove() {
    let plan: Vec<InstalledUnitFile> = install_plan(&facts(), &[nested()])
        .unwrap_or_else(|refusal| panic!("{}", refusal.message()));
    let slug: String = std::env::var(NESTED_SLUG_VARIABLE)
        .unwrap_or_else(|_| panic!("{NESTED_SLUG_VARIABLE} names its unit's short word"));
    let nested_unit: String = nested_unit_name(slug.as_str());
    for unit in [BOOT_GATE_UNIT_NAME, nested_unit.as_str()] {
        let disabled = run_bounded(SYSTEMCTL, &["disable", unit]);
        assert!(
            disabled.succeeded,
            "`{unit}` is disabled: {}",
            disabled.said
        );
    }
    for file in plan.iter().rev() {
        std::fs::remove_file(file.path.as_path())
            .unwrap_or_else(|error| panic!("{} removes: {error}", file.path.display()));
        if let Some(folder) = file.path.parent() {
            let _ = std::fs::remove_dir(folder);
        }
    }
    let reload = run_bounded(SYSTEMCTL, &["daemon-reload"]);
    assert!(reload.succeeded, "the manager reloads: {}", reload.said);
    println!("STEP19-BOOT removed files={}", plan.len());
}

/// Run one `zfs` command and hand back what it printed.
///
/// Why:
///  the phase reaches the guest's OpenZFS by its own path,
/// because the recipe
///  installs it outside the account's `PATH`,
/// and a command that failed must
///  name itself here rather than at the reading after the reboot.
fn zfs(arguments: &[&str]) -> String {
    let sbin: String = std::env::var(SBIN_VARIABLE)
        .unwrap_or_else(|_| panic!("{SBIN_VARIABLE} names the guest's zfs userspace"));
    let program: String = format!("{sbin}/zfs");
    let output = run_bounded(program.as_str(), arguments);
    assert!(
        output.succeeded,
        "`zfs {}` failed: {}",
        arguments.join(" "),
        output.said
    );
    return output.said;
}

/// What:
///  One managed clone with durable intent,
/// made through the release writers the
///  add route uses,
/// so the boot has something to attach again.
/// Why:
///  the setup run that publishes the store's parent record belongs to plan step
///  19's own wiring and reaches no command yet,
///  so this phase writes that record with the same release call the sweep's own
///  arms use,
///  and the clone itself with `zfs clone` of the checkout dataset's snapshot
///  under the add route's own origin prefix.
///  Everything the boot then does is the gate's own work.
/// What:
///  This boot's start-probe record,
///  left as the gate's own probe wrote it where it passed.
/// Why:
///  the gate's `ExecStartPre` runs the start probe,
///  and a record the sweep cannot read refuses the reconciliation before it attaches
///  anything,
///  so a clone arm with no readable record would prove nothing about the boot.
///  The probe writes its own record on a machine whose module and userspace it
///  checks,
///  and where this guest's own probe could not,
///  the record is written here in the mode's own shape from the live
///  fingerprint,
///  which is the fixture plan step 8's guest arm writes for the same
///  reason.
fn ensure_a_passing_probe_record() {
    let path: PathBuf =
        crate::worktree_clone_mount_handler::sweep_probe_record::probe_record_path();
    let standing = crate::worktree_clone_mount_handler::sweep_probe_record::read_probe_standing(
        path.as_path(),
    );
    match standing {
        Ok(crate::worktree_clone_mount_handler::sweep_probe_record::ProbeStanding::Passed) => {
            println!("STEP19-BOOT probe-record=passed");
        }
        other => {
            let live: crate::start_probe::probe_fingerprint::ProbeFingerprint =
                crate::start_probe::probe_fingerprint::ProbeFingerprint::read(
                    &crate::start_probe::probe_fingerprint::FingerprintPaths::live(),
                )
                .unwrap_or_else(|refusal| {
                    panic!("this guest's fingerprint reads: {}", refusal.message())
                });
            let absent: &str = crate::start_probe::probe_record::ABSENT;
            let text: String = format!(
                "{version}\nkind {kind}\nstate finished\nboot {boot}\nkernel {kernel}\nmodule {module}\nsrcversion {srcversion}\nbuild-id {build}\nhandler {handler}\ntime {time}\nverdict passed\nverdict-step none\nskip-reason none\n",
                version = crate::start_probe::probe_record::RECORD_VERSION,
                kind = crate::start_probe::probe_record::RECORD_KIND,
                boot = live.boot_id,
                kernel = live.kernel_release,
                module = live.module_version.as_deref().unwrap_or(absent),
                srcversion = live.module_srcversion.as_deref().unwrap_or(absent),
                build = live.module_build_id.as_deref().unwrap_or(absent),
                handler = live.handler.as_deref().unwrap_or(absent),
                time = crate::iso_time::now_iso()
            );
            if let Some(folder) = path.parent() {
                std::fs::create_dir_all(folder).expect("the probe record's folder is made");
            }
            std::fs::write(path.as_path(), text.as_bytes()).expect("the probe record is written");
            std::fs::set_permissions(
                path.as_path(),
                std::os::unix::fs::PermissionsExt::from_mode(0o644),
            )
            .expect("the probe record holds its mode");
            println!("STEP19-BOOT probe-record=written standing={other:?}");
        }
    }
}

fn clone_one() {
    let slot: String = std::env::var(SLOT_VARIABLE)
        .unwrap_or_else(|_| panic!("{SLOT_VARIABLE} names the clone's slot"));
    let checkout: PathBuf = facts().checkout;
    let checkout_dataset: String = std::env::var(CHECKOUT_DATASET_VARIABLE)
        .unwrap_or_else(|_| panic!("{CHECKOUT_DATASET_VARIABLE} names the checkout dataset"));
    let container: String = std::env::var(CONTAINER_VARIABLE)
        .unwrap_or_else(|_| panic!("{CONTAINER_VARIABLE} names the container"));
    ensure_a_passing_probe_record();
    let worktrees: PathBuf = checkout.join(".claude").join("worktrees");
    std::fs::create_dir_all(worktrees.as_path()).expect("the worktrees parent is made");
    let parent: std::fs::File = std::fs::File::open(worktrees.as_path())
        .expect("the worktrees parent opens for its identity");
    let identity: crate::worktree_clone_mount::CloneFolderIdentity =
        crate::worktree_clone_mount::CloneFolderIdentity::capture(&parent)
            .expect("the parent's identity reads");
    crate::worktree_clone_mount::publish_parent_identity_under_root_control(
        Path::new(crate::worktree_clone_mount::SLOT_IDENTITY_STORE_PATH),
        &identity,
        &identity,
    )
    .expect("the parent record is published under root control");
    let slot_folder: PathBuf = worktrees.join(slot.as_str());
    std::fs::create_dir_all(slot_folder.as_path()).expect("the slot folder is made");
    let pointer: String = format!(
        "gitdir: {}\n",
        checkout
            .join(".git")
            .join("worktrees")
            .join(slot.as_str())
            .display()
    );
    std::fs::write(slot_folder.join(".git"), pointer).expect("the slot's Git pointer is written");
    let snapshot: String = format!("{checkout_dataset}@{ORIGIN_PREFIX}{slot}");
    zfs(&["snapshot", snapshot.as_str()]);
    let clone: String = format!("{container}/{slot}");
    zfs(&["clone", snapshot.as_str(), clone.as_str()]);
    let request: String = format!("v1:{NONCE}:mount");
    zfs(&[
        "set",
        format!("{DESIRED_STATE_PROPERTY}={request}").as_str(),
        clone.as_str(),
    ]);
    println!("STEP19-BOOT clone-made slot={slot} clone={clone} origin={snapshot}");
}

/// The prefix the add route gives its origin snapshots.
pub const ORIGIN_PREFIX: &str = "git-worktree-clone-origin-";

/// The nonce the clone's mount request carries.
/// Value:
///  the sixteen bytes the sweep arms' own request uses,
/// so a receipt this boot files is the shape a person reads.
pub const NONCE: &str = "00112233445566778899aabbccddeeff";

/// The property one clone's durable intent is written in,
/// which is the release's own name for it.
pub const DESIRED_STATE_PROPERTY: &str =
    crate::worktree_clone_mount_protocol::clone_mount_property_names::DESIRED_STATE_PROPERTY;

#[test]
fn the_plan_the_setup_writes_orders_a_real_boot_in_a_guest() {
    if std::env::var_os(GUEST_VARIABLE).is_none() {
        println!(
            "SKIPPED the boot arm: {GUEST_VARIABLE} is unset, so this is not the step 19 guest"
        );
        return;
    }
    let phase: String =
        std::env::var(PHASE_VARIABLE).unwrap_or_else(|_| panic!("{PHASE_VARIABLE} names a phase"));
    match phase.as_str() {
        "install" => install(),
        "clone" => clone_one(),
        "read" => read(),
        "remove" => remove(),
        other => panic!("`{other}` is no phase of the boot arm"),
    }
}
