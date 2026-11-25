//! The controls of `fixture_root_taking_any_name_of_unit_tests.rs`.
//!
//! What:
//!  One control that reads the capability sets of a program started from this run's fixture root,
//!  inside the namespace where that root takes a name that is not UTF-8.
//! Why:
//!  The program a fixture starts there is Git,
//!  and a capability Git holds is refused by the confinement that answers for it,
//!  which is logged and costs a denial for a hardlink the kernel lets through anyway
//!  (`doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`,
//!  the `dac_override` section).
//!  A control that reads the child's own `/proc/self/status` sees the state the kernel starts it in,
//!  whatever the started program does afterwards.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('a program started from the any-name root holds no capability', () => {
//!   if (!inAFixtureRootTakingAnyName(self)) return;
//!   assert.equal(capabilitiesOf('sh', ['-c', 'grep ^Cap /proc/self/status']).everyZero(), true);
//! });
//! ```

use std::process::Command;

use super::in_a_fixture_root_taking_any_name;

/// The capability sets a program starts with,
/// as the status file prints them.
/// `CapBnd` is left out on purpose:
/// the bounding set says which capabilities a program could ever be given,
/// and this control asks which ones it holds.
const REPORTED_SETS: [&str; 4] = ["CapInh", "CapPrm", "CapEff", "CapAmb"];

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
///  and this control then reads an ordinary account's child,
///  which holds none either.
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
            held,
            NO_CAPABILITIES_HELD,
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
fn capability_set_from(status: &str, set: &str) -> u64 {
    let printed: &str = status
        .lines()
        .find_map(|line| {
            let (name, word) = line.split_once(':')?;
            return if name == set { Some(word.trim()) } else { None };
        })
        .unwrap_or_else(|| { return panic!("{set} is not in the status file of the program that was started:\n{status}"); });
    return u64::from_str_radix(printed, 16).unwrap_or_else(|error| {
        return panic!("the status file prints {set} as {printed}, which is not a hexadecimal capability word: {error}");
    });
}
