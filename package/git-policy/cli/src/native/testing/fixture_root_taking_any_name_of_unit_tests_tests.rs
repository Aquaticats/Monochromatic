//! The controls of `fixture_root_taking_any_name_of_unit_tests.rs`.
//!
//! What:
//!  One control that reads the capability sets of a program started from this run's fixture root,
//!  inside the namespace where that root takes a name that is not UTF-8,
//!  and one that the window a control states for its child is the window that child gets.
//! Why:
//!  The program a fixture starts there is Git,
//!  and a capability Git holds is refused by the confinement that answers for it,
//!  which is logged and costs a denial for a hardlink the kernel lets through anyway
//!  (`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
//!  the `dac_override` section).
//!  A control that reads the child's own `/proc/self/status` sees the state the kernel starts it in,
//!  whatever the started program does afterwards.
//!  The second control covers the window that child is given:
//!  every control here runs its whole body in that child,
//!  so a window that is not the one the control stated fails a body the host only made slow
//!  (issue 6546).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a program started from the any-name root holds no capability', () => {
//!   if (!inAFixtureRootTakingAnyName(self)) return;
//!   assert.equal(capabilitiesOf('sh', ['-c', 'grep ^Cap /proc/self/status']).everyZero(), true);
//! });
//! ```

use std::process::Command;
use std::time::Duration;

use super::in_a_fixture_root_taking_any_name;
use crate::call_catching_panics::call_catching_panics;
use crate::root_namespace_test_support::in_a_root_namespace_within;

/// The variable that says which half of the deadline control a run is,
/// set in that control's child alone.
const THE_CHILD_HALF: &str = "CLI_GIT_TEST_WINDOW_STATED_CONTROL";

/// How long the deadline control's body holds inside its child.
const THE_BODY_HOLDS: Duration = Duration::from_secs(3);

/// The window the deadline control states for its child,
///  which is shorter than the body it runs.
const THE_WINDOW: Duration = Duration::from_secs(1);

/// The capability sets a program starts with,
/// as the status file prints them.
/// `CapBnd` is left out on purpose:
/// the bounding set says which capabilities a program could ever be given,
/// and this control asks which ones it holds.
const REPORTED_SETS: &[&str] = &["CapInh", "CapPrm", "CapEff", "CapAmb"];

/// What a set of held capabilities reads as when the program holds none.
const NO_CAPABILITIES_HELD: u64 = 0;

/// The uid the root of a user namespace has inside it.
const NAMESPACE_ROOT_UID: libc::uid_t = 0;

/// What:
///  A program started from the fixture root, where that root takes a name that is not UTF-8,
///  holds no capability.
/// Why:
///   The root of a user namespace is given every capability of that namespace when it starts a program,
///  unless the process carries `SECURE_NOROOT`
///  (`security/commoncap.c`, `handle_privileged_root`);
///  the fixture sets that bit once its tmpfs is mounted,
///  so a fixture's Git asks the kernel for no capability it would then hold,
///  and the answer it gets is the one an ordinary account's Git is given.
///  A run whose fixture root already takes such a name starts no program in a namespace of its own,
///  and it must run as an ordinary account,
///  whose programs gain no capability across exec;
///  a run there as root holds every capability in its child and fails this control,
///  which is the state the fixture refuses.
#[test]
fn a_program_started_from_the_any_name_root_holds_no_capability() {
    if !in_a_fixture_root_taking_any_name(
        "testing::fixture_root_taking_any_name_of_unit_tests::tests::a_program_started_from_the_any_name_root_holds_no_capability",
    ) {
        return;
    }
    // SAFETY: `geteuid` reads this process's own effective uid and touches no memory of ours.
    let effective_uid: libc::uid_t = unsafe { libc::geteuid() };
    eprintln!(
        "the fixture root started this control's program as uid {effective_uid}: {}",
        if effective_uid == NAMESPACE_ROOT_UID {
            "the namespace's root, whose programs the kernel would otherwise give every capability across exec"
        } else {
            "an ordinary account, whose programs gain no capability across exec anyway"
        }
    );
    let status: String = capabilities_of_a_started_program();
    for set in REPORTED_SETS {
        let held: u64 = capability_set_from(status.as_str(), set);
        assert_eq!(
            held, NO_CAPABILITIES_HELD,
            "the program started from the fixture root holds {set}={held:#018x}, and a program a fixture starts must hold \
             none of them, as an ordinary account's Git does when it commits in such a fixture\n{status}"
        );
    }
}

/// What:
///  Every capability set the program printed, read from the status file of the program itself.
/// Why:
///   The sets are read from inside the program,
///  so nothing of this process's own state stands in for them,
///  and the whole report is returned as well as parsed
///  so a failure names every set and not only the one it asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function capabilitiesOfAStartedProgram(): string {
///   const run = spawnSync('sh', ['-c', 'grep --extended-regexp ^Cap /proc/self/status']);
///   if (run.status !== 0) throw new Error(run.stderr);
///   return run.stdout.toString();
/// }
/// ```
fn capabilities_of_a_started_program() -> String {
    let started: std::process::Output = Command::new("sh")
        .args(["-c", "grep --extended-regexp ^Cap /proc/self/status"])
        .output()
        .expect("the shell that reports its own capabilities runs");
    assert!(
        started.status.success(),
        "the shell that reports its own capabilities succeeds: {}",
        String::from_utf8_lossy(started.stderr.as_slice())
    );
    return String::from_utf8(started.stdout).expect("a status file is text");
}

/// What:
///  The hexadecimal word the status file prints for one capability set.
/// Why:
///   The file is the kernel's own report,
///  and a set it does not name is a control that read nothing,
///  which must fail rather than count as a set of none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function capabilitySetFrom(status: string, set: string): bigint {
///   const line = status.split('\n').find((each) => each.startsWith(`${set}:`));
///   if (line === undefined) throw new Error(`${set} is not in the status file:\n${status}`);
///   return BigInt(`0x${line.slice(set.length + 1).trim()}`);
/// }
/// ```
fn capability_set_from(
    status: &str,
    set: &str,
) -> u64 {
    let printed: &str = status
        .lines()
        .find_map(|line| {
            let (name, word) = line.split_once(':')?;
            return if name == set { Some(word.trim()) } else { None };
        })
        .unwrap_or_else(|| {
            panic!("{set} is not in the status file of the program that was started:\n{status}")
        });
    return u64::from_str_radix(printed, 16)
        .unwrap_or_else(|error| panic!("the status file prints {set} as {printed}, which is not a hexadecimal capability word: {error}"));
}

/// What:
///  The window a control states is the window the child that runs its body is given:
///  a body that holds longer than that window is ended at it,
///  and the failure names the figure the control stated.
/// Why:
///  A control whose body does more than a handful of files' work states its own window
///  (`service_landing_state::awkward_change_lists_land_as_native_git_commits_them`,
///  issue 6546),
///  and a window that never reaches the child leaves such a control failing on the host's load instead;
///  the default answers `CHILD_TEST_DEADLINE`,
///  whose 120 s this control's own message would name.
///  The child is this test binary on this test,
///  so the body that holds is written here as the half that reads `THE_CHILD_HALF`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('a stated window is the childs window', () => {
///   if (process.env[THE_CHILD_HALF]) { sleep(THE_BODY_HOLDS); return; }
///   const warned = catchingPanics(() => inARootNamespaceWithin(THE_CHILD_HALF, self, THE_WINDOW));
///   assert(warned.message.includes(`did not end within ${secondsOf(THE_WINDOW)} s`));
/// });
/// ```
#[test]
fn the_window_a_control_states_is_the_window_its_child_gets() {
    if std::env::var_os(THE_CHILD_HALF).is_some() {
        std::thread::sleep(THE_BODY_HOLDS);
        return;
    }
    let caught: std::thread::Result<bool> = call_catching_panics((), |()| {
        return in_a_root_namespace_within(
            THE_CHILD_HALF,
            "testing::fixture_root_taking_any_name_of_unit_tests::tests::the_window_a_control_states_is_the_window_its_child_gets",
            THE_WINDOW,
        );
    });
    let Err(payload) = caught else {
        panic!(
            "the child that holds {THE_BODY_HOLDS:?} was not ended by the {:?} window the control stated for it",
            THE_WINDOW
        );
    };
    let words: Option<&str> = payload.downcast_ref::<&str>().copied();
    let said: String = match payload.downcast_ref::<String>() {
        Some(text) => text.clone(),
        None => words.unwrap_or_default().to_string(),
    };
    let owed: String = format!("did not end within {} s", THE_WINDOW.as_secs_f64());
    assert!(
        said.contains(owed.as_str()),
        "the failure names no {owed:?}, so the window the control stated did not reach its child: {said}"
    );
}
