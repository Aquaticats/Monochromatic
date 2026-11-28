//! What:
//!  The private-propagation step's guest arm:
//!  a real checkout dataset mounted at the checkout path,
//!  made shared,
//!  made shared and slave,
//!  left private,
//!  left slave and unbindable,
//!  and replaced by another dataset,
//!  each driven through the step as root,
//!  and then the trigger measurement the accepted design asks for,
//!  whose two rules decide whether the systemd trigger stays.
//! Why:
//!  The step changes a mount and reads the kernel's own answer about it,
//!  which no control on a person's machine may do,
//!  so the states are proved here on a file-backed pool inside a throwaway
//!  guest
//!  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
//!  "Measurements owed",
//!  M4 and M10,
//!  and `doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
//!  "Which triggers the route adds").
//!  No root mode calls the step in this release,
//!  so the arm calls the step itself,
//!  and the reconcile the trigger asks for is a stand-in gate whose
//!  `ExecReload=` appends a line:
//!  what the trigger half reads is whether the mount unit's activation starts
//!  the propagation service once and the gate once,
//!  and whether either loops.
//!  The step is then run on the same instance the replacement left,
//!  so the two halves are read on one checkout and the sentence "the
//!  reconcile made it private" is never claimed.
//!  A run on this machine prints one `SKIPPED` line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const state of ['shared', 'shared+slave', 'private', 'slave', 'unbindable']) expect(phase(state)).toMatch(/outcome=/);
//! for (const replacement of replacements(10)) expect(await seen(replacement)).toBe(true);
//! expect(loops(quiet)).toBe(0);
//! ```

use super::guest_fixture::Fixture;
use super::propagation_calls::TheKernelsCalls;
use super::propagation_record::PropagationRecord;
use super::propagation_step::run_the_step;
use super::propagation_step_names::{
    CheckoutResolution,
    PropagationDone,
    PropagationPlaces,
    PropagationRefusal,
};
use super::propagation_words::{
    PropagationOutcome,
    PropagationStep,
};
use crate::checkout_boot_units::{
    CheckoutBootFacts,
    propagation_mount_drop_in,
    propagation_service,
    propagation_service_path,
};
use crate::subvolume::fs_beneath_mount_facts::unique_mount_number_of;
use std::path::{
    Path,
    PathBuf,
};

/// The dataset the checkout's mount serves,
/// which the arm makes in the guest's pool.
const CHECKOUT_DATASET_LEAF: &str = "checkout";

/// The second dataset the refusal phase mounts at the checkout path.
const OTHER_DATASET_LEAF: &str = "other";

/// The stand-in gate's unit file,
/// whose `ExecStart` does nothing and whose `ExecReload` appends one line,
/// because this release's gate does not call the step yet.
const GATE_UNIT_PATH: &str = "/etc/systemd/system/reconcile-git-worktree-clone-mounts.service";

/// The file the stand-in gate's `ExecReload` appends one line to.
const GATE_LOG_PATH: &str = "/root/propagation-gate.log";

/// The log this arm writes its own status lines to inside the guest.
const ARM_LOG_PATH: &str = "/root/propagation-arm.log";

/// How many replacements typed at a person's pace the trigger half makes,
/// which is the count the design's own example states and this run fixes
/// before it starts.
const PACED_REPLACEMENTS: u32 = 10;

/// How many replacements inside one second the limits reading makes,
/// which is the burst the three limits are read under.
const RAPID_REPLACEMENTS: u32 = 8;

/// The milliseconds one replacement's trigger has to show itself,
/// which is the reading window this run states.
const SEEN_WINDOW_MILLISECONDS: u64 = 10_000;

/// The pause between an unmount and its mount in one paced replacement,
/// which is long enough for a person to type the second command.
const TYPED_PAUSE_MILLISECONDS: u64 = 2_000;

/// The quiet interval after the last replacement,
/// over which a loop would show itself.
const QUIET_MILLISECONDS: u64 = 20_000;

/// What:
///  One line the arm prints and the recipe greps.
/// Why:
///  A phase that returned without its line would read as a passing arm,
///  so every phase says one line whether it passed or refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const say = (line: string): void => { console.log(`PROPAGATION ${line}`); appendArmLog(line); };
/// ```
fn say(line: &str) {
    let text: String = format!("PROPAGATION {line}");
    println!("{text}");
    let _appended: std::io::Result<()> = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ARM_LOG_PATH)
        .and_then(|mut file| {
            return std::io::Write::write_all(&mut file, format!("{text}\n").as_bytes());
        });
}

#[test]
#[ignore = "live: needs the mvm guest fixture; run it through the package's e2e recipe"]
fn the_propagation_guest_arm_tests() {
    let Some(fixture) = Fixture::from_environment("the_propagation_guest_arm_tests") else {
        return;
    };
    make_the_pool_and_the_datasets(&fixture);
    for state in ["shared", "shared+slave", "private", "slave", "unbindable"] {
        in_a_private_namespace(&|| {
            return one_state(&fixture, state);
        });
    }
    in_a_private_namespace(&|| {
        return a_mount_that_is_not_the_checkout(&fixture);
    });
    let trigger: String = the_trigger_measurement(&fixture);
    say(trigger.as_str());
    say("done");
}

/// What:
///  The guest's file-backed pool and the two datasets the arm mounts.
/// Why:
///  A ZFS mount is what the step's type and source checks are about,
///  and a file-backed vdev keeps everything inside the guest,
///  so the arm makes one pool of its own and never touches the guest's root
///  filesystem.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// zpool_create(`/root/${pool}.img`, pool); zfs_create(`${pool}/checkout`); zfs_create(`${pool}/other`);
/// ```
fn make_the_pool_and_the_datasets(fixture: &Fixture) {
    let image: PathBuf = PathBuf::from(format!("/root/{}-pool.img", fixture.pool));
    let (vdev, vdev_said) = fixture.ask(
        "truncate",
        &["--size=512M", image.to_str().expect("a text path")],
    );
    assert!(vdev, "the pool's vdev could not be made: {vdev_said}");
    let (pool, pool_said) = fixture.ask(
        "zpool",
        &[
            "create",
            // This OpenZFS takes short options only: `--force` answers `invalid option '-'` (measured in this round).
            "-f",
            fixture.pool.as_str(),
            image.to_str().expect("a text path"),
        ],
    );
    assert!(pool, "the pool could not be made: {pool_said}");
    let checkout: &Path = fixture.checkout.as_path();
    std::fs::create_dir_all(checkout).expect("the checkout's folder");
    let mountpoint: String = format!("mountpoint={}", checkout.display());
    for leaf in [CHECKOUT_DATASET_LEAF, OTHER_DATASET_LEAF] {
        let dataset: String = format!("{}/{leaf}", fixture.pool);
        // Both datasets mount at the checkout path and neither mounts on its own, so a phase mounts the one it is about
        // and inherits no mount from another.
        let (created, created_said) = fixture.ask(
            "zfs",
            &[
                "create",
                "-o",
                mountpoint.as_str(),
                "-o",
                "canmount=noauto",
                dataset.as_str(),
            ],
        );
        assert!(
            created,
            "the dataset {dataset} could not be made: {created_said}"
        );
    }
    say(format!(
        "fixture guest={} pool={} checkout={}",
        fixture.guest,
        fixture.pool,
        checkout.display()
    )
    .as_str());
}

/// What:
///  One state the kernel can hand the step,
///  made in this process's own private mount namespace,
///  driven through the step,
///  and read back from the record and the kernel.
/// Why:
///  A mount made in a private namespace dies with the namespace,
///  so each state is made fresh and leaves nothing for the next,
///  and the same path carries a different instance in each phase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mount(); makeShared(); const done = runTheStep(places, at(), kernel); expect(record.before).toBe('shared');
/// ```
fn one_state(
    fixture: &Fixture,
    state: &str,
) -> String {
    let dataset: String = format!("{}/{CHECKOUT_DATASET_LEAF}", fixture.pool);
    let checkout: &Path = fixture.checkout.as_path();
    // This namespace starts as a peer of the guest's own table, so a mount made here would be delivered there. Each
    // state fixes the propagation it is about before it mounts anything, so the guest's table keeps what it had.
    match state {
        "shared" => {
            make_the_tree_private(fixture);
            make_the_root_shared(fixture);
            mount_the_checkout(fixture, dataset.as_str());
        }
        "private" => {
            // Under a private parent a mount is private as soon as it exists, which is the state the step reports.
            make_the_tree_private(fixture);
            mount_the_checkout(fixture, dataset.as_str());
        }
        "unbindable" => {
            make_the_tree_private(fixture);
            mount_the_checkout(fixture, dataset.as_str());
            change_the_propagation(checkout, libc::MS_UNBINDABLE);
        }
        "slave" => {
            make_the_parent_a_slave(fixture);
            mount_the_checkout(fixture, dataset.as_str());
        }
        "shared+slave" => {
            // A mount made under a slave parent receives that parent's master, and making it shared again keeps the
            // master: that is the set a machine inside a container has.
            make_the_parent_a_slave(fixture);
            mount_the_checkout(fixture, dataset.as_str());
            change_the_propagation(checkout, libc::MS_SHARED);
        }
        other => panic!("a state this arm does not know: {other}"),
    }
    let before: String = propagation_of(fixture, checkout);
    let instance_before: u64 = instance_of(checkout);
    let places: PropagationPlaces = a_place();
    let answer: Result<PropagationDone, PropagationRefusal> =
        run_the_step(&places, &resolution(fixture), &TheKernelsCalls);
    let after: String = propagation_of(fixture, checkout);
    let instance_after: u64 = instance_of(checkout);
    let record: PropagationRecord = read_the_record(&places);
    let expected: (&str, &str) = match state {
        "shared" => ("applied", "private"),
        "shared+slave" => ("applied", "private"),
        "private" => ("already-private", "private"),
        "slave" => ("left", "slave"),
        _ => ("left", "unbindable"),
    };
    assert_eq!(
        record.outcome.word(),
        expected.0,
        "the step's answer for a {state} mount: {:?}",
        answer.as_ref().map(|done| return done.line.as_str())
    );
    assert_eq!(
        after, expected.1,
        "the mount's own propagation after the step"
    );
    assert_eq!(
        instance_before, instance_after,
        "the set changed the instance it verified and no other"
    );
    assert_eq!(
        record.after.map(|set| return set.word()),
        Some(expected.1),
        "the record's set after the step is the kernel's own answer"
    );
    assert_eq!(
        record.before.map(|set| return set.word()),
        Some(before.as_str()),
        "the record's set before the step is the kernel's own answer"
    );
    if state == "shared+slave" {
        assert!(
            record.master.is_some(),
            "a shared and slave mount's master is recorded, because setting it private detaches that master"
        );
    }
    return format!(
        "state={state} before={before} outcome={} after={after} instance={instance_before}",
        record.outcome.word()
    );
}

/// What:
///  A checkout path carrying another dataset,
///  which the step must refuse without changing anything.
/// Why:
///  The source check is the one that tells the checkout's own dataset from a
///  bind or another dataset at the same path,
///  and a refusal files no receipt,
///  so the arm reads the receipts folder as well as the record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mount(`${pool}/other`, checkout); expect(runTheStep(...)).toMatch(/refused/); expect(receipts()).toBe(0);
/// ```
fn a_mount_that_is_not_the_checkout(fixture: &Fixture) -> String {
    let other: String = format!("{}/{OTHER_DATASET_LEAF}", fixture.pool);
    let checkout: &Path = fixture.checkout.as_path();
    make_the_tree_private(fixture);
    let before: String = propagation_of(fixture, checkout);
    let (mounted, mounted_said) = fixture.ask("zfs", &["mount", other.as_str()]);
    assert!(
        mounted,
        "the other dataset could not be mounted at the checkout: {mounted_said}"
    );
    let receipts_before: usize = count_receipts();
    let places: PropagationPlaces = a_place();
    let refusal: PropagationRefusal = run_the_step(&places, &resolution(fixture), &TheKernelsCalls)
        .expect_err("a mount of another dataset");
    assert_eq!(refusal.step, PropagationStep::Source);
    let record: PropagationRecord = read_the_record(&places);
    assert_eq!(record.outcome, PropagationOutcome::Refused);
    assert_eq!(
        count_receipts(),
        receipts_before,
        "a refusal files no receipt, so the receipts folder holds what it held"
    );
    assert_eq!(
        propagation_of(fixture, checkout),
        before,
        "the refusal changed nothing about the mount"
    );
    return format!(
        "refused=source receipts={receipts_before} outcome={}",
        record.outcome.word()
    );
}

/// What:
///  The accepted design's trigger measurement:
///  the mount unit's transition starts the propagation service once,
///  the gate reloads once,
///  nothing runs again over the quiet interval,
///  and the number of starts a burst of replacements is held to.
/// Why:
///  The trigger is provisional until a guest shows it sees a replacement
///  typed by hand and does not loop,
///  and the three limits say what a burst costs without being a pass or a
///  fail
///  (`doc/planning/cli-git-zfs-private-propagation-after-replacement-mount-design.md`,
///  "The trigger the route adds,
///  and what it must show").
///  The gate is a stand-in because no root mode calls the step in this
///  release;
///  the step is run on the same instance the replacement left,
/// so the
///  repair is read on the checkout the trigger fired for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let i = 0; i < 10; i++) { replace(); expect(await startedOnce()).toBe(true); expect(await reloadedOnce()).toBe(true); }
/// const quiet = await watch(20_000); expect(quiet.starts).toBe(0);
/// ```
fn the_trigger_measurement(fixture: &Fixture) -> String {
    let dataset: String = format!("{}/{CHECKOUT_DATASET_LEAF}", fixture.pool);
    let checkout: &Path = fixture.checkout.as_path();
    mount_the_checkout(fixture, dataset.as_str());
    write_the_gate_and_the_pair(fixture);
    let (reloaded, reload_said) = fixture.ask("systemctl", &["daemon-reload"]);
    assert!(
        reloaded,
        "the manager could not reload its units: {reload_said}"
    );
    // The stand-in gate carries no `[Install]` section, so it is started and not enabled: nothing here survives a boot.
    let (started, said) = fixture.ask(
        "systemctl",
        &["start", "reconcile-git-worktree-clone-mounts.service"],
    );
    assert!(started, "the gate could not be started: {said}");
    // The mount unit's own activation is what arms the propagation service, so the first replacement is the arming one.
    let arming: String = replace_and_wait(fixture, dataset.as_str(), "arming");
    let mut seen: u32 = 0;
    for round in 1..=PACED_REPLACEMENTS {
        let line: String =
            replace_and_wait(fixture, dataset.as_str(), format!("paced-{round}").as_str());
        assert!(
            line.contains("seen=true"),
            "a replacement typed at a person's pace was not seen by the trigger: {line}"
        );
        seen += 1;
        let places: PropagationPlaces = a_place();
        let answer: Result<PropagationDone, PropagationRefusal> =
            run_the_step(&places, &resolution(fixture), &TheKernelsCalls);
        assert!(
            answer.is_ok(),
            "the step the reconcile would have run did not repair the instance the replacement left: {answer:?}"
        );
        assert_eq!(
            propagation_of(fixture, checkout),
            "private",
            "the checkout reads private once the reconcile's step has run"
        );
    }
    let quiet: String = watch_the_quiet_interval(fixture);
    assert!(
        quiet.contains("starts=0") && quiet.contains("reloads=0"),
        "nothing ran over the quiet interval after the last replacement: {quiet}"
    );
    let burst: String = a_burst_of_replacements(fixture, dataset.as_str());
    return format!(
        "trigger arming=[{arming}] seen={seen}/{PACED_REPLACEMENTS} quiet=[{quiet}] burst=[{burst}]"
    );
}

/// What:
///  One replacement typed at a person's pace,
///  and the readings the two rules ask for.
/// Why:
///  The rule's "seen" is one start of the propagation service and one
///  completed reload of the gate inside the reading window,
///  and its "loops" is a second start for the same replacement,
///  so both counts are read around the replacement itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const before = counts(); unmount(); wait(2000); mount(); expect(await waitFor(both, 10_000)).toBe(true);
/// ```
fn replace_and_wait(
    fixture: &Fixture,
    dataset: &str,
    label: &str,
) -> String {
    let starts_before: u32 = count_service_starts(fixture);
    let reloads_before: u32 = count_gate_reloads();
    let (unmounted, said) = fixture.ask("zfs", &["unmount", dataset]);
    assert!(
        unmounted,
        "the checkout could not be unmounted ({label}): {said}"
    );
    std::thread::sleep(std::time::Duration::from_millis(TYPED_PAUSE_MILLISECONDS));
    mount_the_checkout(fixture, dataset);
    let started: u32 = wait_for_counts(fixture, starts_before, reloads_before);
    let starts: u32 = count_service_starts(fixture);
    let reloads: u32 = count_gate_reloads();
    let checkout_propagation: String = propagation_of(fixture, fixture.checkout.as_path());
    return format!(
        "replacement={label} seen={} starts={} reloads={} propagation={checkout_propagation} waited={started}",
        starts - starts_before == 1 && reloads - reloads_before == 1,
        starts - starts_before,
        reloads - reloads_before
    );
}

/// What:
///  Wait,
/// bounded by the reading window,
/// until the propagation service started
///  once and the gate reloaded once since the counts the caller took.
/// Why:
///  Both are what "seen" means,
///  and a wait that returned at the first of them would read a replacement as
///  seen when only half of the trigger had fired.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const waitForCounts = (starts: number, reloads: number): number => { const until = Date.now() + 10_000; while (Date.now() < until) { if (counts().starts > starts && counts().reloads > reloads) return Date.now() - until + 10_000; sleep(100); } return -1; };
/// ```
fn wait_for_counts(
    fixture: &Fixture,
    starts: u32,
    reloads: u32,
) -> u32 {
    let began: std::time::Instant = std::time::Instant::now();
    loop {
        let elapsed: u64 = u64::try_from(began.elapsed().as_millis()).unwrap_or(u64::MAX);
        if count_service_starts(fixture) > starts && count_gate_reloads() > reloads {
            return u32::try_from(elapsed).unwrap_or(u32::MAX);
        }
        if elapsed >= SEEN_WINDOW_MILLISECONDS {
            return u32::MAX;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

/// What:
///  The quiet interval's counts:
///  no start of the propagation service and no reload of the gate.
/// Why:
///  A loop is one of the three things the design names,
///  and every one of them shows as a count that keeps rising while nothing
///  changes,
///  so the interval is read rather than assumed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const watch = (ms: number): string => { const s = counts(); sleep(ms); return `${counts().starts - s.starts} ${counts().reloads - s.reloads}`; };
/// ```
fn watch_the_quiet_interval(fixture: &Fixture) -> String {
    let starts: u32 = count_service_starts(fixture);
    let reloads: u32 = count_gate_reloads();
    std::thread::sleep(std::time::Duration::from_millis(QUIET_MILLISECONDS));
    return format!(
        "starts={} reloads={}",
        count_service_starts(fixture) - starts,
        count_gate_reloads() - reloads
    );
}

/// What:
///  A burst of replacements inside a second,
///  and how many starts the propagation service was held to.
/// Why:
///  The three limits sit where they sit,
///  and the reading describes the cost of a flap without being a pass or a
///  fail,
///  so the arm prints the number of replaces,
///  the starts and the reloads and lets the report compare them with the
///  source's own limits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const burst = (n: number): string => { const s = counts(); for (...) replace(); sleep(2000); return `${n} ${counts().starts - s.starts}`; };
/// ```
fn a_burst_of_replacements(
    fixture: &Fixture,
    dataset: &str,
) -> String {
    let starts: u32 = count_service_starts(fixture);
    let reloads: u32 = count_gate_reloads();
    for _round in 0..RAPID_REPLACEMENTS {
        let _unmounted: (bool, String) = fixture.ask("zfs", &["unmount", dataset]);
        let _mounted: (bool, String) = fixture.ask("zfs", &["mount", dataset]);
    }
    std::thread::sleep(std::time::Duration::from_millis(TYPED_PAUSE_MILLISECONDS));
    return format!(
        "replacements={RAPID_REPLACEMENTS} starts={} reloads={} limits=upholds-16-per-10s,service-5-per-10s,table-5-per-1s",
        count_service_starts(fixture) - starts,
        count_gate_reloads() - reloads
    );
}

/// What:
///  The stand-in gate and the propagation pair the release code builds.
/// Why:
///  The pair is the trigger the project accepted,
///  so its text comes from the release's own builder rather than from this
///  arm,
///  and the gate's `ExecStart` is a stand-in because this release's gate does
///  not call the step:
///  its `ExecReload` appends one line,
/// which is how the arm counts the reloads
///  the request answers with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// write('/etc/systemd/system/reconcile-….service', standIn); write(propagationServicePath(), propagationService(facts)); write(dropIn, propagationMountDropIn(facts));
/// ```
fn write_the_gate_and_the_pair(fixture: &Fixture) {
    let facts: CheckoutBootFacts = CheckoutBootFacts {
        checkout: fixture.checkout.clone(),
        root_file: PathBuf::from(
            "/usr/libexec/monochromatic-git-policy-cli/history_event-mount_git_worktree_clone_datasets",
        ),
        key_unit: None,
    };
    // The gate a boot would have, with a reload that says it ran and a start that does nothing.
    std::fs::write(
        GATE_UNIT_PATH,
        format!(
            "[Unit]\nDescription=Stand-in for the route's gate, whose start this release does not wire\n\n[Service]\nType=oneshot\nRemainAfterExit=yes\nExecStart=/bin/true\nExecReload=/bin/sh -c 'echo reload >> {GATE_LOG_PATH}'\n"
        ),
    )
    .expect("the stand-in gate");
    let service: String = propagation_service(&facts).expect("the propagation service's text");
    let service_path: PathBuf = propagation_service_path();
    std::fs::create_dir_all(service_path.parent().expect("the unit folder"))
        .expect("the unit folder");
    std::fs::write(service_path.as_path(), service.as_str()).expect("the propagation service");
    let drop_in: String = propagation_mount_drop_in(&facts).expect("the mount drop-in's text");
    let drop_in_path: PathBuf = crate::checkout_boot_units::drop_in_path(
        crate::checkout_boot_units::mount_unit_name(fixture.checkout.as_path()).as_str(),
        crate::checkout_boot_units::PROPAGATION_MOUNT_DROP_IN_NAME,
    );
    std::fs::create_dir_all(drop_in_path.parent().expect("the drop-in folder"))
        .expect("the drop-in folder");
    std::fs::write(drop_in_path.as_path(), drop_in.as_str()).expect("the mount drop-in");
}

/// What:
///  How many times the propagation service has started.
/// Why:
///  A start is the unit's own transition,
///  and the journal names it once per start,
///  so the count is the reading the design's two rules are written in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const starts = (): number => journalctl('-u', 'checkout-propagation.service').split('\n').filter(l => l.includes('Starting Ask the boot unit')).length;
/// ```
fn count_service_starts(fixture: &Fixture) -> u32 {
    let (read, said) = fixture.ask(
        "journalctl",
        &[
            "--unit",
            "checkout-propagation.service",
            "--output=cat",
            "--no-pager",
            "--quiet",
        ],
    );
    if !read {
        // A boot whose journal has no line for the unit yet reads no start at all.
        return 0;
    }
    return u32::try_from(
        said.lines()
            .filter(|line| return line.contains("Starting Ask the boot unit"))
            .count(),
    )
    .unwrap_or(0);
}

/// How many times the stand-in gate's reload ran,
/// which is the other half of the rule's "seen".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reloads = (): number => existsSync(GATE_LOG) ? readFileSync(GATE_LOG).split('\n').length - 1 : 0;
/// ```
fn count_gate_reloads() -> u32 {
    let text: String = std::fs::read_to_string(GATE_LOG_PATH).unwrap_or_default();
    return u32::try_from(text.lines().count()).unwrap_or(0);
}

/// How many receipts the route filed,
/// which a refusal must leave as it found it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const receipts = (): number => existsSync('/run/git-worktree-clone-mount-results') ? readdirSync(dir).length : 0;
/// ```
fn count_receipts() -> usize {
    let folder: std::fs::ReadDir = match std::fs::read_dir("/run/git-worktree-clone-mount-results")
    {
        Ok(folder) => folder,
        Err(_absent) => return 0,
    };
    return folder.count();
}

/// What:
///  One row's propagation as `findmnt` prints it,
///  which is the kernel's own answer through the tool a person reads.
/// Why:
///  The record's set is the step's own reading,
///  and the arm must compare it with a second reader's answer about the same
///  mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const propagation = (): string => exec(`findmnt --noheadings --output PROPAGATION --target ${checkout}`).trim();
/// ```
fn propagation_of(
    fixture: &Fixture,
    path: &Path,
) -> String {
    let (read, read_said) = fixture.ask(
        "findmnt",
        &[
            "--noheadings",
            "--output",
            "PROPAGATION",
            "--target",
            path.to_str().expect("a text path"),
        ],
    );
    assert!(
        read,
        "the mount's propagation could not be read: {read_said}"
    );
    let word: &str = read_said.trim();
    // `findmnt` lists the flags a mount carries, comma-separated, and a private mount carries none; the record names
    // the same set joined by `+`, so one spelling compares with the other.
    return match word {
        "" => String::from("private"),
        other => String::from(other).replace(',', "+"),
    };
}

/// The unique number of the mount the checkout path is on,
/// which decides whether two reads name one instance.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const instanceOf = (path: string): number => statx(open(path), '', AT_EMPTY_PATH, STATX_MNT_ID_UNIQUE).stx_mnt_id;
/// ```
fn instance_of(path: &Path) -> u64 {
    let handle: std::fs::File = std::fs::File::open(path).expect("the checkout's descriptor");
    return unique_mount_number_of(&handle).expect("the descriptor's unique mount number");
}

/// What:
///  Change one mount's propagation through the kernel's own call.
/// Why:
///  `mount --make-slave` on this guest answers `mount failed: Unknown error 5005` when util-linux hands the change to a
///  filesystem helper, and the kernel's call names the mount the answer is about,
///  so the arm asks the kernel directly, as the start probe does for its own namespace
///  (`start_probe/probe_mount_calls.rs`, `make_mounts_private`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mount(null, path, null, MS_REC | MS_PRIVATE, null);
/// ```
fn change_the_propagation(
    path: &Path,
    flags: libc::c_ulong,
) {
    let text: std::ffi::CString =
        std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).expect("no NUL in a path");
    // SAFETY: the path is a NUL-terminated string that outlives the call, no source, filesystem or data is named, and
    // the flags ask only for a propagation change.
    let answered: libc::c_int = unsafe {
        libc::mount(
            std::ptr::null(),
            text.as_ptr(),
            std::ptr::null(),
            flags,
            std::ptr::null(),
        )
    };
    assert_eq!(
        answered,
        0,
        "the propagation of {} could not be changed to {flags}: {}",
        path.display(),
        std::io::Error::last_os_error()
    );
}

/// Make this namespace's whole tree private, so nothing mounted here is delivered to the guest's own table.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mount(null, '/', null, MS_REC | MS_PRIVATE, null);
/// ```
fn make_the_tree_private(_fixture: &Fixture) {
    change_the_propagation(Path::new("/"), libc::MS_REC | libc::MS_PRIVATE);
}

/// Make this namespace's root a shared mount of its own, so a mount made on it comes up shared and stays here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exec('mount --make-shared /');
/// ```
fn make_the_root_shared(_fixture: &Fixture) {
    change_the_propagation(Path::new("/"), libc::MS_SHARED);
}

/// Make the folder the checkout is mounted in a slave of the namespace the arm came from,
/// which is what makes a mount beneath it a slave rather than a shared mount.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exec('mount --make-slave /root');
/// ```
fn make_the_parent_a_slave(_fixture: &Fixture) {
    // This namespace's root is a peer of the guest's own, so the kernel makes it a slave of that group, and a mount
    // made under it receives that master and passes nothing on.
    change_the_propagation(Path::new("/"), libc::MS_SLAVE);
}

/// Mount the checkout's dataset at the checkout path,
/// through the `zfs` program the route's own setup uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exec(`zfs mount <dataset>`); // the dataset's mountpoint is the checkout path
/// ```
fn mount_the_checkout(
    fixture: &Fixture,
    dataset: &str,
) {
    let (mounted, mounted_said) = fixture.ask("zfs", &["mount", dataset]);
    if mounted {
        return;
    }
    // A mount namespace that ended leaves OpenZFS still naming its dataset mounted, so the state a phase takes over is
    // unmounted here before this phase's own mount.
    assert!(
        mounted_said.contains("already mounted"),
        "the dataset {dataset} could not be mounted: {mounted_said}"
    );
    let _unmounted: (bool, String) = fixture.ask("zfs", &["unmount", dataset]);
    let (again, again_said) = fixture.ask("zfs", &["mount", dataset]);
    assert!(
        again,
        "the dataset {dataset} could not be mounted after it was unmounted: {again_said}"
    );
}

/// The step's places for one phase:
/// a record under `/run` inside the guest and this boot's identifier.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const places = (): PropagationPlaces => ({ record: PropagationFile.live(), boot: readFile('/proc/sys/kernel/random/boot_id').trim() });
/// ```
fn a_place() -> PropagationPlaces {
    // The release place is the one the record must live in, and every phase overwrites the one record: one checkout has
    // one current answer.
    return PropagationPlaces::live().expect("this guest's own places");
}

/// The checkout and the dataset its mount must serve,
/// as the arm's own configuration would.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const resolution = (): CheckoutResolution => ({ kind: 'at', checkout, dataset: `${pool}/checkout` });
/// ```
fn resolution(fixture: &Fixture) -> CheckoutResolution {
    return CheckoutResolution::At {
        checkout: fixture.checkout.clone(),
        dataset: format!("{}/{CHECKOUT_DATASET_LEAF}", fixture.pool),
    };
}

/// The record the step filed,
/// read through the release's own reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = (places: PropagationPlaces): PropagationRecord => places.record.read().unwrap().unwrap();
/// ```
fn read_the_record(places: &PropagationPlaces) -> PropagationRecord {
    return places
        .record
        .read()
        .expect("a readable record")
        .expect("the record the step filed");
}

/// What:
///  Run one state's body in a mount namespace of its own.
/// Why:
///  A mount made here must die with the phase,
///  and a namespace of this process's own is what makes that true,
///  so the arm forks,
///  unshares the mount namespace in the child,
///  and lets the child say its one line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inNamespace = (body: () => string): void => { const child = fork(() => { unshare(CLONE_NEWNS); console.log(body()); _exit(0); }); waitFor(child); };
/// ```
fn in_a_private_namespace(body: &dyn Fn() -> String) {
    // The child's own fork, so this process's mount namespace is untouched.
    let child: libc::pid_t = unsafe { libc::fork() };
    assert!(
        child >= 0,
        "the phase's child: {}",
        std::io::Error::last_os_error()
    );
    if child == 0 {
        // SAFETY: the child unshares its own mount namespace and touches no memory of the parent's.
        let unshared: libc::c_int = unsafe { libc::unshare(libc::CLONE_NEWNS) };
        if unshared != 0 {
            say("refused=unshare");
            std::process::exit(1);
        }
        let line: String = body();
        say(line.as_str());
        std::process::exit(0);
    }
    let mut status: libc::c_int = 0;
    // SAFETY: the number is this phase's own child and the status word is this call's own.
    let _waited: libc::pid_t = unsafe { libc::waitpid(child, &mut status, 0) };
    assert_eq!(
        status, 0,
        "the phase's own child ended well: {status} (its line is above)"
    );
}
