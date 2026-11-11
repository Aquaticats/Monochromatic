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
/// Steps the test scripts.
struct ScriptedSteps {
    /// The main process the system manager reports,
    /// absent when it reports none.
    expected: Option<i32>,
    /// What the bus answers.
    ownership: Ownership,
    /// Whether killing fails.
    kill_fails: bool,
}

/// The scripted steps.
impl GuardSteps for ScriptedSteps {
    /// The scripted main process.
    fn expected_pid(&self, _uid: u32) -> Option<i32> {
        return self.expected;
    }

    /// The scripted owner.
    fn ownership(&mut self, _uid: u32) -> Ownership {
        return self.ownership;
    }

    /// No event is scripted here.
    fn wait_for_change(&mut self, _uid: u32, _timeout: Duration) -> bool {
        return false;
    }

    /// Records the kill.
    fn kill(&self, _pid: i32) -> Result<(), String> {
        if self.kill_fails {
            return Err("operation not permitted".to_string());
        }
        return Ok(());
    }

    /// Records the re-execution.
    fn reexec(&self, _uid: u32) -> Result<(), String> {
        return Ok(());
    }
}

/// Builds the scripted steps.
fn steps(expected: Option<i32>, ownership: Ownership) -> ScriptedSteps {
    return ScriptedSteps { expected, ownership, kill_fails: false };
}

/// The manager owning the name needs nothing done.
#[test]
fn the_manager_owning_the_name_needs_nothing() {
    let mut steps = ScriptedSteps { expected: Some(MANAGER_PID), ownership: Ownership::Owner(MANAGER_PID), kill_fails: false };
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP) };
    assert!(guard.check(1000, Instant::now()).is_empty());
}

/// A process that is not the manager is killed,
/// the report names both pids,
/// and the manager is asked to take the name again.
#[test]
fn an_impostor_is_killed_and_the_manager_takes_the_name_again() {
    let mut steps = steps(Some(MANAGER_PID), Ownership::Owner(IMPOSTOR_PID));
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP) };
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
    assert!(matches!(memory.decide(MANAGER_PID, Ownership::Owner(IMPOSTOR_PID), start), Action::KillImpostor { pid: IMPOSTOR_PID, reexec: true }));
    assert!(matches!(
        memory.decide(MANAGER_PID, Ownership::Owner(IMPOSTOR_PID + 1), start + Duration::from_secs(1)),
        Action::KillImpostor { reexec: false, .. }
    ));
    assert!(matches!(memory.decide(MANAGER_PID, Ownership::Owner(IMPOSTOR_PID), start + REEXEC_GAP), Action::KillImpostor { reexec: true, .. }));
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
    let mut steps = steps(None, Ownership::Owner(IMPOSTOR_PID));
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP) };
    let reports = guard.check(1000, Instant::now());
    assert_eq!(reports.len(), 1);
    assert!(reports[0].loud && reports[0].text.contains("no main process"));
    assert_eq!(guard.memory.last_reexec, None);
}

/// Killing the impostor uses `SIGKILL`,
/// which an impostor manager cannot handle:
/// `SIGTERM` would make it stop the units it started.
/// The control runs the real kill on a process that ignores `SIGTERM`,
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
    let steps = crate::impostor_guard::RealSteps::new(Duration::from_secs(2));
    steps.kill(pid).expect("the kill was delivered");
    let status = child.wait().expect("the process ended");
    assert!(!status.success(), "the process ended by a signal: {status:?}");
    use std::os::unix::process::ExitStatusExt;
    assert_eq!(status.signal(), Some(libc::SIGKILL), "the signal was SIGKILL and not SIGTERM");
}

/// A kill that the kernel refuses is reported,
/// and the manager is still asked to take the name again.
#[test]
fn a_kill_that_fails_is_reported() {
    let mut steps = ScriptedSteps { expected: Some(MANAGER_PID), ownership: Ownership::Owner(IMPOSTOR_PID), kill_fails: true };
    let mut guard = Guard { steps: &mut steps, memory: GuardMemory::new(REEXEC_GAP) };
    let reports = guard.check(1000, Instant::now());
    assert!(reports.iter().any(|report| return report.loud && report.text.contains("could not be killed")), "{reports:?}");
}
