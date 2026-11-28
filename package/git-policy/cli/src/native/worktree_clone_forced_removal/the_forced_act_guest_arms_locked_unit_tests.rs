//! What:
//!  The guest arm's phase for a locked holder that is a service unit of the
//!  guest account's own user manager:
//!  the kernel's two refusals before the stop,
//!  the manager's stop,
//!  and the removal's own calls after it.
//! Why:
//!  The owner's answer of 2026-11-23 stops that unit through the person's own
//!  manager and lets the removal go on,
//!  and no arm had watched that stop
//!  (`doc/decision/cli-git-zfs-step-17-forced-and-cancelled-removal-designs-accepted.md`,
//!  "A locked holder whose copy is locked (V10)").
//!  The unit's own namespace is a copy of the guest's root namespace,
//!  so the clone is mounted there and not in this phase's private one:
//!  a unit the manager starts descends from process 1 and can never see a mount
//!  this phase made for itself.
//!  Everything else is the route's own code:
//!  `read_the_owner`,
//!  `the_unit_of_the_accounts_manager`,
//!  both stages of the act,
//!  and `stop_the_unit` with the harness's provisional wait,
//!  because the release's own wait has no value.
//!  Never part of the release executable.

use super::the_forced_act_guest_arms::{
    AHolder,
    a_job_of_the_callers_session,
    mount_the_checkout,
};
use super::the_forced_act_guest_fixture::Fixture;
use super::the_forced_act_guest_fixture::{
    SLOT,
    holds_the_copy_at,
    phase_line,
    the_table_of_the_process,
};
use crate::process::a_locked_holders_own_unit::{
    UNIFIED_HIERARCHY_PREFIX,
    the_unit_of_the_accounts_manager,
};
use crate::process::a_service_unit_stopped_through_the_accounts_manager::{
    StoppedUnit,
    TheUnitsStop,
    UnitNotStopped,
    stop_the_unit,
};
use crate::worktree_clone_forced_removal::descriptor_calls::open_directory;
use crate::worktree_clone_forced_removal::forced_arm_numbers::the_stop_wait;
use crate::worktree_clone_forced_removal::holder_preflight::Route;
use crate::worktree_clone_forced_removal::namespace_owner::{
    NamespaceOwner,
    read_the_owner,
};
use crate::worktree_clone_forced_removal::recorded_handle::{
    RecordedHandle,
    name_the_handle,
};
use crate::worktree_clone_forced_removal::the_copy_taken_out_of_a_namespace::{
    NamespaceCopy,
    make_the_way_ready,
    take_the_copy_out,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// The account's own runtime directory in the guest,
/// which the manager's bus listens in.
const THE_ACCOUNTS_RUNTIME_DIRECTORY: &str = "/run/user/0";

/// The account's own session bus,
/// as its address spells it.
const THE_ACCOUNTS_SESSION_BUS: &str = "unix:path=/run/user/0/bus";

/// The transient unit these arms start,
/// which no unit file on the guest names.
const THE_UNITS_NAME: &str = "forced-act-holder";

/// The wait a run of this arm gives a stop,
/// which is the harness's provisional value and not a shipped one.
const A_HARNESSES_STOP_WAIT: Duration = Duration::from_secs(10);

/// The reserved name way 7's directory carries here,
/// which the release derives from the clone's GUID.
const THE_RESERVED_NAME: &str = ".git-worktree-clone-removal-0";

/// What:
///  One command run inside the guest's root mount namespace,
///  which is the namespace the account's own user manager descends from.
/// Why:
///  A mount this phase made for itself is invisible to a unit the manager
///  starts,
///  so the clone is mounted where the manager's tree can copy it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = inTheRootNamespace(fixture, 'mount', ['-t', 'zfs', dataset, path]);
/// ```
fn in_the_root_namespace(
    fixture: &Fixture,
    program: &str,
    arguments: &[&str],
) -> (bool, String) {
    // What: the words `nsenter` takes before the program, which enter process 1's mount namespace.
    // Why:  that namespace is the one the manager and its units stand on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const words = ['--target', '1', '--mount', '--', program, ...arguments];
    // ```
    let mut words: Vec<&str> = vec!["--target", "1", "--mount", "--", program];
    words.extend_from_slice(arguments);
    return fixture.ask("nsenter", words.as_slice());
}

/// What:
///  One command that asks the account's own user manager something,
///  given the runtime directory and the bus address the manager listens on.
/// Why:
///  The guest's account is never logged in,
///  so nothing in this process's own environment names that manager,
///  and a child is handed the two places instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = aSessionCommand(fixture, 'systemctl', ['--user', 'is-system-running']);
/// ```
fn a_session_command(
    fixture: &Fixture,
    program: &str,
    arguments: &[&str],
) -> (bool, String) {
    // What: the child's own answer, whichever way it went.
    // Why:  a refused call and a call that answered are both a phase's evidence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const answer = spawnSync(program, arguments, { env: { PATH, XDG_RUNTIME_DIR, DBUS_SESSION_BUS_ADDRESS } });
    // ```
    let answer: Result<std::process::Output, std::io::Error> = std::process::Command::new(program)
        .args(arguments)
        .env("PATH", fixture.search_path())
        .env("XDG_RUNTIME_DIR", THE_ACCOUNTS_RUNTIME_DIRECTORY)
        .env("DBUS_SESSION_BUS_ADDRESS", THE_ACCOUNTS_SESSION_BUS)
        .output();
    return match answer {
        Ok(output) => (
            output.status.success(),
            format!(
                "{}{}",
                String::from_utf8_lossy(output.stdout.as_slice()).trim(),
                String::from_utf8_lossy(output.stderr.as_slice()).trim()
            ),
        ),
        Err(error) => (false, format!("`{program}` could not start: {error}")),
    };
}

/// What:
///  Start the transient unit that holds the slot's copy,
///  and answer the number of the task it runs.
/// Why:
///  The unit is the shape the owner's answer stops:
///  a service unit of the account's own manager with `PrivateMounts=yes`,
///  whose mount namespace the kernel then locks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pid = startTheUnit(fixture);  // the unit's MainPID
/// ```
fn start_the_unit(fixture: &Fixture) -> i32 {
    let (started, said) = a_session_command(
        fixture,
        "systemd-run",
        &[
            "--user",
            "--unit",
            THE_UNITS_NAME,
            "--property=PrivateMounts=yes",
            "/usr/bin/sleep",
            "infinity",
        ],
    );
    assert!(started, "the manager starts the unit: {said}");
    // What: the number of the task the unit runs, which every read of the holder is about.
    // Why:  the kernel's namespace, control group and lock are the task's, not the unit name's.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let pid = 0; while (!pid) pid = Number(showProperty('MainPID'));
    // ```
    let mut pid: i32 = 0;
    let deadline: std::time::Instant = std::time::Instant::now() + Duration::from_secs(30);
    while std::time::Instant::now() < deadline {
        let (read, text) = a_session_command(
            fixture,
            "systemctl",
            &[
                "--user",
                "show",
                "--property=MainPID",
                "--value",
                THE_UNITS_NAME,
            ],
        );
        if read {
            pid = text.trim().parse::<i32>().unwrap_or(0);
        }
        if pid > 0 {
            return pid;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("the manager started the unit and named no task of it: {said}");
}

/// What:
///  The act's own view of the copy in the unit's namespace,
///  from the descriptor this value owns.
/// Why:
///  The two stages take the copy by the descriptor and the dataset it must
///  match,
///  so one value holds both and a caller cannot pair two namespaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copy = { namespace, identity, checkout, slot, dataset, handle, directory };
/// ```
struct TheUnitsCopy {
    namespace: std::fs::File,
    identity: String,
    dataset: String,
}

impl TheUnitsCopy {
    /// The act's own value for this copy,
    /// borrowed from the descriptor this value owns.
    fn named<'a>(
        &'a self,
        checkout: &'a Path,
        slot: &'a Path,
        handle: &'a RecordedHandle,
    ) -> NamespaceCopy<'a> {
        return NamespaceCopy {
            namespace: &self.namespace,
            identity: self.identity.as_str(),
            checkout,
            slot,
            dataset: self.dataset.as_str(),
            handle,
            directory: THE_RESERVED_NAME,
        };
    }
}

/// What:
///  What one way of the act answered for this copy,
///  its word and the words that go with it.
/// Why:
///  A locked namespace can refuse a way at its structural stage rather than at
///  its unmount,
///  and the phase line has to say which stage refused and why.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [word, said] = theWayAnswers(Route.InPlaceByTheHandle, copy);
/// ```
fn the_way_answers(
    route: Route,
    copy: &NamespaceCopy<'_>,
) -> (String, String) {
    return match make_the_way_ready(route, copy) {
        Ok(ready) => match take_the_copy_out(&ready, copy) {
            Ok(_taken) => (String::from("accepted"), String::new()),
            Err(refusal) => (String::from(refusal.word.word()), String::from(refusal.said)),
        },
        Err(refusal) => (String::from(refusal.word.word()), String::from(refusal.said)),
    };
}

/// What:
///  The locked holder's phase:
///  the kernel refuses both ways,
///  the manager's stop ends the unit,
///  its control group and its namespace,
///  and the host's unmount and the head destroy follow.
/// Why:
///  This is the one arm the decision record said was owed for V10's stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = phaseLockedUnit(fixture);
/// ```
pub(crate) fn the_locked_holders_unit_phase(fixture: &Fixture) -> String {
    let mut fields: Vec<(&str, &str)> = Vec::new();
    let caller_job: AHolder = a_job_of_the_callers_session();
    fixture.a_fresh_clone();
    // This phase's own view of the checkout, which the underlay's handle is taken through.
    let checkout: PathBuf = mount_the_checkout(fixture);
    let slot: PathBuf = checkout.join(SLOT);
    std::fs::create_dir_all(slot.as_path()).expect("the slot's underlay");
    let underlay: std::fs::File = open_directory(slot.as_path()).expect("the underlay is opened");
    let handle: RecordedHandle = name_the_handle(&underlay)
        .expect("the underlay's handle is taken before the clone is mounted");
    // The checkout and the clone in the namespace the manager's own tree descends from, where a unit can copy them.
    let (checkout_mounted, checkout_said) = in_the_root_namespace(
        fixture,
        "mount",
        &[
            "-t",
            "zfs",
            fixture.checkout().as_str(),
            checkout.to_str().expect("the checkout path is text"),
        ],
    );
    assert!(
        checkout_mounted,
        "the root namespace mounts the checkout: {checkout_said}"
    );
    let (clone_mounted, clone_said) = in_the_root_namespace(
        fixture,
        "mount",
        &[
            "-t",
            "zfs",
            fixture.clone().as_str(),
            slot.to_str().expect("the slot path is text"),
        ],
    );
    assert!(
        clone_mounted,
        "the root namespace mounts the clone at the slot: {clone_said}"
    );
    let (manager_runs, manager_said) =
        a_session_command(fixture, "systemctl", &["--user", "is-system-running"]);
    fields.push((
        "manager",
        if manager_runs {
            manager_said.as_str()
        } else {
            "absent"
        },
    ));
    let pid: i32 = start_the_unit(fixture);
    let owner: NamespaceOwner = read_the_owner(Path::new(format!("/proc/{pid}/ns/mnt").as_str()))
        .expect("the holder's namespace owner is read");
    fields.push(("owner", if owner.locked() { "another" } else { "ours" }));
    let cgroup: String = std::fs::read_to_string(format!("/proc/{pid}/cgroup"))
        .expect("the holder's control-group line is read");
    let unit: String = the_unit_of_the_accounts_manager(cgroup.as_str(), 0)
        .expect("the holder runs in a service unit of this account's own manager");
    fields.push(("unit", unit.as_str()));
    let held: TheUnitsCopy = TheUnitsCopy {
        namespace: std::fs::File::open(format!("/proc/{pid}/ns/mnt"))
            .expect("the holder's namespace is opened"),
        identity: owner.namespace.clone(),
        dataset: fixture.clone(),
    };
    let copy: NamespaceCopy<'_> = held.named(checkout.as_path(), slot.as_path(), &handle);
    // What: what each way answered, its word and the sentence that goes with it.
    // Why:  a way that refuses before its unmount says a different word, and the line has to say which.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const way8 = theWayAnswers(Route.InPlaceByTheHandle, copy);   // { word, said }
    // ```
    let way8: (String, String) = the_way_answers(Route::InPlaceByTheHandle, &copy);
    let way7: (String, String) = the_way_answers(Route::SlaveMoveAndUnmount, &copy);
    fields.push(("way8", way8.0.as_str()));
    fields.push(("way8-said", way8.1.as_str()));
    fields.push(("way7", way7.0.as_str()));
    fields.push(("way7-said", way7.1.as_str()));
    fields.push((
        "holders-copy",
        if holds_the_copy_at(
            the_table_of_the_process(pid).rows.as_slice(),
            slot.as_path(),
            fixture.clone().as_str(),
        ) {
            "stays"
        } else {
            "gone"
        },
    ));
    // The release's own wait has no value, so a stop cannot be armed from it yet: the phase says so and then hands
    // the harness's own wait, which ships by nothing.
    let refused_wait: String = match the_stop_wait() {
        Ok(_value) => String::from("a-value"),
        Err(refusal) => format!("refused:{}", refusal.constant()),
    };
    fields.push(("stop-wait", refused_wait.as_str()));
    let tasks_cgroup: String = format!("/proc/{pid}/cgroup");
    let namespace: String = format!("/proc/{pid}/ns/mnt");
    let the: TheUnitsStop<'_> = TheUnitsStop {
        unit: unit.as_str(),
        cgroup: cgroup
            .lines()
            .find(|line| return line.starts_with(UNIFIED_HIERARCHY_PREFIX))
            .map(|line| return line.trim_start_matches(UNIFIED_HIERARCHY_PREFIX))
            .unwrap_or("/"),
        tasks_cgroup: Path::new(tasks_cgroup.as_str()),
        namespace: Path::new(namespace.as_str()),
        identity: owner.namespace.as_str(),
    };
    let stopped: Result<StoppedUnit, UnitNotStopped> =
        stop_the_unit(Path::new("/run/user/0/bus"), &the, A_HARNESSES_STOP_WAIT);
    match &stopped {
        Ok(answer) => {
            fields.push(("stopped", answer.unit.as_str()));
            fields.push(("stopped-namespace", answer.namespace.as_str()));
        }
        Err(refusal) => fields.push(("stopped", refusal.word())),
    }
    fields.push((
        "namespace-after",
        if Path::new(namespace.as_str()).exists() {
            "there"
        } else {
            "gone"
        },
    ));
    fields.push((
        "tasks-after",
        if Path::new(tasks_cgroup.as_str()).exists() {
            "there"
        } else {
            "gone"
        },
    ));
    // The act's own descriptor is a reference to the holder's namespace, so it is dropped before the namespace goes or
    // the namespace would outlive the unit.
    drop(held);
    fields.push((
        "hosts-copy",
        if holds_the_copy_at(
            the_table_of_the_process(1).rows.as_slice(),
            slot.as_path(),
            fixture.clone().as_str(),
        ) {
            "stays"
        } else {
            "gone"
        },
    ));
    let (unmounted, unmount_said) = in_the_root_namespace(
        fixture,
        "umount",
        &[slot.to_str().expect("the slot path is text")],
    );
    fields.push((
        "unmount-after",
        if unmounted {
            "accepted"
        } else {
            unmount_said.as_str()
        },
    ));
    let (destroyed, _destroy_said) =
        fixture.ask("zfs", &["destroy", "-r", fixture.clone().as_str()]);
    fields.push(("destroy-after", if destroyed { "gone" } else { "refused" }));
    let alive: bool = caller_job.alive();
    fields.push(("caller-job", if alive { "alive" } else { "gone" }));
    caller_job.end();
    assert!(
        owner.locked(),
        "the holder's namespace is owned by a user namespace of its own"
    );
    assert!(
        stopped.is_ok(),
        "the manager's stop ended the unit: {fields:?}"
    );
    return phase_line("locked-holder-stopped-through-its-unit", fields.as_slice());
}
