//! Verifies the guard of the manager's bus name:
//! what it decides for an owner that is the manager,
//! one that is not,
//! no owner,
//! and a bus that cannot be asked,
//! that it kills the impostor and not another process,
//! that it uses the one signal an impostor manager cannot handle,
//! and that it asks the manager to re-execute at most once in a few minutes.

/// The guard under test.
use crate::impostor_guard::{Action, Guard, GuardMemory, GuardSteps, Ownership, REEXEC_GAP};
/// Bounded waits.
use std::time::{Duration, Instant};

/// The manager the fixtures report.
const MANAGER_PID: i32 = 4242;
/// The impostor the fixtures report.
const IMPOSTOR_PID: i32 = 777;
/// The watched user's primary group in these fixtures,
/// which differs from its user id,
/// as it does on a host where accounts start at another number.
const OWN_GID: u32 = 1500;
/// Steps the test scripts.
struct ScriptedSteps {
    /// The main process the system manager reports,
    /// absent when it reports none.
    expected: Option<i32>,
    /// What the bus answers.
    ownership: Ownership,
    /// Whether killing fails.
    kill_fails: bool,
    /// What the kill was asked to do:
    /// the unique connection name and the pid.
    killed: Option<(String, i32)>,
    /// The user and the group the re-execution was asked to drop to.
    reexeced: std::cell::Cell<Option<(u32, u32)>>,
}

/// The scripted steps.
impl GuardSteps for ScriptedSteps {
    /// The scripted main process.
    fn expected_pid(&self, _uid: u32) -> Option<i32> {
        return self.expected;
    }

    /// The scripted owner.
    fn ownership(&mut self, _uid: u32) -> Ownership {
        return self.ownership.clone();
    }

    /// No event is scripted here.
    fn wait_for_change(&mut self, _uid: u32, _timeout: Duration) -> bool {
        return false;
    }

    /// Records the kill,
    /// with the unique connection name the guard read,
    /// which is what the real kill proves again.
    fn kill(&mut self, _uid: u32, unique: &str, pid: i32) -> Result<(), String> {
        self.killed = Some((unique.to_string(), pid));
        if self.kill_fails {
            return Err("operation not permitted".to_string());
        }
        return Ok(());
    }

    /// Records the re-execution,
    /// with the user and the group it was asked to drop to.
    fn reexec(&self, uid: u32, gid: u32) -> Result<(), String> {
        self.reexeced.set(Some((uid, gid)));
        return Ok(());
    }
}

/// Builds the scripted steps.
fn steps(expected: Option<i32>, ownership: Ownership) -> ScriptedSteps {
    return ScriptedSteps { expected, ownership, kill_fails: false, killed: None, reexeced: std::cell::Cell::new(None) };
}

/// The unique connection name the fixtures report.
const IMPOSTOR_UNIQUE: &str = ":1.777";

/// An owner the bus reports.
fn owner(pid: i32) -> Ownership {
    return Ownership::Owner { pid, unique: IMPOSTOR_UNIQUE.to_string() };
}

/// A bus answer the proof asks for.
struct ScriptedLookup {
    /// The pid the bus names,
    /// absent when it names none.
    names: Option<i32>,
}

/// The scripted bus.
impl crate::impostor_guard::steps::OwnerLookup for ScriptedLookup {
    /// The pid the bus names for the unique connection name.
    fn pid_of(&mut self, _unique: &str) -> Option<i32> {
        return self.names;
    }
}

/// A signal that records what it was asked to send.
#[derive(Default)]
struct RecordingSignal {
    /// The pids it was asked to signal.
    sent: std::cell::RefCell<Vec<i32>>,
}

/// The signal a test watches.
impl crate::impostor_guard::steps::SignalSend for RecordingSignal {
    /// Records the pid.
    fn send(&self, pid: i32) -> Result<(), String> {
        self.sent.borrow_mut().push(pid);
        return Ok(());
    }
}

/// The manager owning the name needs nothing done.
#[test]
fn the_manager_owning_the_name_needs_nothing() {
    let mut steps = ScriptedSteps { expected: Some(MANAGER_PID), ownership: owner(MANAGER_PID), kill_fails: false, killed: None, reexeced: std::cell::Cell::new(None) };
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP), gid: OWN_GID };
    assert!(guard.check(1000, Instant::now()).is_empty());
}

/// A process that is not the manager is killed,
/// the report names both pids,
/// and the manager is asked to take the name again.
#[test]
fn an_impostor_is_killed_and_the_manager_takes_the_name_again() {
    let mut steps = steps(Some(MANAGER_PID), owner(IMPOSTOR_PID));
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP), gid: OWN_GID };
    let reports = guard.check(1000, Instant::now());
    assert!(reports.iter().any(|report| return report.loud && report.text.contains(&IMPOSTOR_PID.to_string()) && report.text.contains(&MANAGER_PID.to_string())), "{reports:?}");
    assert!(reports.iter().any(|report| return report.text.contains("re-executed")), "{reports:?}");
    assert!(guard.memory.last_reexec.is_some());
}

/// A second impostor inside the gap is killed too,
/// but the manager is not re-executed again:
/// a re-execution is a heavy thing to repeat in a loop.
#[test]
fn a_second_impostor_inside_the_gap_is_killed_without_another_reexecution() {
    let mut memory = GuardMemory::new(REEXEC_GAP);
    let start = Instant::now();
    assert!(matches!(memory.decide(MANAGER_PID, owner(IMPOSTOR_PID), start), Action::KillImpostor { pid: IMPOSTOR_PID, reexec: true, .. }));
    assert!(matches!(
        memory.decide(MANAGER_PID, owner(IMPOSTOR_PID + 1), start + Duration::from_secs(1)),
        Action::KillImpostor { reexec: false, .. }
    ));
    assert!(matches!(memory.decide(MANAGER_PID, owner(IMPOSTOR_PID), start + REEXEC_GAP), Action::KillImpostor { reexec: true, .. }));
}

/// No owner at all makes `systemctl --user` fail everywhere,
/// so the manager is asked to take the name again,
/// once.
#[test]
fn a_name_without_an_owner_asks_the_manager_to_take_it_again_once() {
    let mut memory = GuardMemory::new(REEXEC_GAP);
    let start = Instant::now();
    assert_eq!(memory.decide(MANAGER_PID, Ownership::Nobody, start), Action::Reexec { ask: true });
    assert_eq!(memory.decide(MANAGER_PID, Ownership::Nobody, start + Duration::from_secs(1)), Action::Reexec { ask: false });
}

/// A bus that cannot be asked changes nothing:
/// guessing could kill a process that is the manager.
#[test]
fn an_unknown_owner_changes_nothing() {
    let mut memory = GuardMemory::new(REEXEC_GAP);
    assert_eq!(memory.decide(MANAGER_PID, Ownership::Unknown, Instant::now()), Action::Nothing);
}

/// A system manager that names no main process is reported,
/// and nothing is killed.
#[test]
fn no_main_process_is_reported_and_nothing_is_killed() {
    let mut steps = steps(None, owner(IMPOSTOR_PID));
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP), gid: OWN_GID };
    let reports = guard.check(1000, Instant::now());
    assert_eq!(reports.len(), 1);
    assert!(reports[0].loud && reports[0].text.contains("no main process"));
    assert_eq!(guard.memory.last_reexec, None);
}

/// Killing the impostor uses `SIGKILL`,
/// which an impostor manager cannot handle:
/// `SIGTERM` would make it stop the units it started.
/// The control runs the real signal on a process that ignores `SIGTERM`,
/// so a kill with `SIGTERM` would leave it running.
#[test]
fn the_impostor_is_killed_with_the_signal_it_cannot_handle() {
    let mut child = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg("trap '' TERM; exec /usr/bin/sleep 30")
        .spawn()
        .expect("start a process that ignores SIGTERM");
    let pid = i32::try_from(child.id()).expect("pid fits");
    std::thread::sleep(Duration::from_millis(200));
    let mut lookup = ScriptedLookup { names: Some(pid) };
    crate::impostor_guard::steps::kill_proven(&mut lookup, &crate::impostor_guard::steps::PidfdSignal, IMPOSTOR_UNIQUE, pid).expect("the proven kill was delivered");
    let status = child.wait().expect("the process ended");
    assert!(!status.success(), "the process ended by a signal: {status:?}");
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(status.signal(), Some(libc::SIGKILL), "the signal was SIGKILL and not SIGTERM");
}

/// A kill that the kernel refuses is reported,
/// and the manager is still asked to take the name again.
#[test]
fn a_kill_that_fails_is_reported() {
    let mut steps = ScriptedSteps { expected: Some(MANAGER_PID), ownership: owner(IMPOSTOR_PID), kill_fails: true, killed: None, reexeced: std::cell::Cell::new(None) };
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP), gid: OWN_GID };
    let reports = guard.check(1000, Instant::now());
    assert!(reports.iter().any(|report| return report.loud && report.text.contains("could not be killed")), "{reports:?}");
}

/// A pid the bus no longer names is never signalled,
/// because the identity the guard read can be given to another process by the kernel.
/// The live process here stands in for that other process:
/// a kill that only used the pid would end it.
#[test]
fn a_pid_the_bus_no_longer_names_is_not_signalled() {
    let mut child = std::process::Command::new("/usr/bin/sleep").arg("30").spawn().expect("start the stand-in process");
    let pid = i32::try_from(child.id()).expect("pid fits");
    let mut lookup = ScriptedLookup { names: None };
    let signal = RecordingSignal::default();
    let refusal = crate::impostor_guard::steps::kill_proven(&mut lookup, &signal, IMPOSTOR_UNIQUE, pid).expect_err("the kill is refused");
    assert!(refusal.contains("no longer names"), "{refusal}");
    assert!(signal.sent.borrow().is_empty(), "nothing was signalled");
    assert!(child.try_wait().expect("the process is asked for").is_none(), "the stand-in process still runs");
    let _killed = child.kill();
    let _reaped = child.wait();
}

/// A unique name that now names another pid proves nothing for the pid the guard read,
/// so that pid is not signalled either.
#[test]
fn a_pid_the_bus_names_differently_is_not_signalled() {
    let mut lookup = ScriptedLookup { names: Some(IMPOSTOR_PID + 1) };
    let signal = RecordingSignal::default();
    let refusal = crate::impostor_guard::steps::kill_proven(&mut lookup, &signal, IMPOSTOR_UNIQUE, IMPOSTOR_PID).expect_err("the kill is refused");
    assert!(refusal.contains(&(IMPOSTOR_PID + 1).to_string()), "{refusal}");
    assert!(signal.sent.borrow().is_empty(), "nothing was signalled");
}

/// The pid the bus just named is signalled,
/// once.
#[test]
fn a_pid_the_bus_still_names_is_signalled() {
    let mut lookup = ScriptedLookup { names: Some(IMPOSTOR_PID) };
    let signal = RecordingSignal::default();
    crate::impostor_guard::steps::kill_proven(&mut lookup, &signal, IMPOSTOR_UNIQUE, IMPOSTOR_PID).expect("the proven kill is delivered");
    assert_eq!(*signal.sent.borrow(), vec![IMPOSTOR_PID]);
}

/// The guard passes the unique connection name it read to the kill,
/// so the kill can prove that same connection and not just a number.
#[test]
fn the_guard_kills_the_connection_it_read() {
    let mut steps = steps(Some(MANAGER_PID), owner(IMPOSTOR_PID));
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP), gid: OWN_GID };
    let _reports = guard.check(1000, Instant::now());
    assert_eq!(steps.killed, Some((IMPOSTOR_UNIQUE.to_string(), IMPOSTOR_PID)));
}

/// A guard whose watcher goes away does not keep polling:
/// the watcher holds the guard's standard input,
/// and closing it ends the guard.
#[test]
fn the_guard_ends_when_its_watcher_closes_its_input() {
    let (input, writer) = std::os::unix::net::UnixStream::pair().expect("a pair of streams");
    let setup = crate::impostor_guard::GuardSetup {
        uid: 1000,
        poll_interval: Duration::from_millis(20),
        gid: OWN_GID,
        reexec_gap: REEXEC_GAP,
    };
    let steps = ScriptedSteps {
        expected: Some(MANAGER_PID),
        ownership: owner(MANAGER_PID),
        kill_fails: false,
        killed: None,
        reexeced: std::cell::Cell::new(None),
    };
    let mut watching = Some(std::thread::spawn(move || {
        let mut steps_in_thread = steps;
        let _run = crate::impostor_guard::child::run_child_watching(&setup, &mut steps_in_thread, input);
    }));
    std::thread::sleep(Duration::from_millis(100));
    assert!(watching.as_ref().is_some_and(|thread| return !thread.is_finished()), "the guard runs while its watcher holds the input");
    drop(writer);
    let thread = watching.take().expect("the guard thread");
    let deadline = Instant::now() + Duration::from_secs(5);
    while !thread.is_finished() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(thread.is_finished(), "closing the watcher's end ended the guard");
}

/// The word after one flag in an argument list,
/// or nothing when the flag is absent.
fn after_flag(arguments: &[String], flag: &str) -> Option<String> {
    let at = arguments.iter().position(|argument| return argument == flag)?;
    return arguments.get(at + 1).cloned();
}

/// The re-execution drops to the user's own primary group,
/// and not to the user's id taken as a group id.
///
/// A host whose accounts have a primary group number other than the user id,
/// or no group numbered like the user at all,
/// made `setpriv --regid <uid>` fail:
/// the unprivileged guard cannot switch to a group it does not belong to,
/// so the manager was never asked to take the name again
/// after an impostor had been killed.
#[test]
fn the_reexecution_drops_to_the_users_primary_group() {
    use crate::impostor_guard::steps::setpriv_arguments;
    /// The watched user of this test,
    /// whose primary group is not its id.
    const WATCHED_UID: u32 = 1000;
    let mut scripted = steps(Some(MANAGER_PID), owner(IMPOSTOR_PID));
    let mut guard = Guard { steps: &mut scripted, memory: GuardMemory::new(REEXEC_GAP), gid: OWN_GID };
    let _reports = guard.check(WATCHED_UID, Instant::now());
    assert_eq!(scripted.reexeced.get(), Some((WATCHED_UID, OWN_GID)), "the re-execution kept the wrong group");
    let arguments: Vec<String> = setpriv_arguments(WATCHED_UID, OWN_GID, &["daemon-reexec"])
        .into_iter()
        .map(|argument| return argument.to_string_lossy().into_owned())
        .collect();
    assert_eq!(after_flag(&arguments, "--reuid"), Some(WATCHED_UID.to_string()), "{arguments:?}");
    assert_eq!(after_flag(&arguments, "--regid"), Some(OWN_GID.to_string()), "the user id is used as a group id: {arguments:?}");
}
