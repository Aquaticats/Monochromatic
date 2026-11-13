//! Controls for what the unload of one worktree's units does when the manager,
//! or the service it holds,
//! cannot give
//! it what it asks for within the wait it is given.
//! Why:
//!  The unload is a repair,
//! not a precondition of a commit:
//! a commit whose service is landing a batch,
//! whose manager stops
//!  answering,
//! or whose unit another command starts again inside the wait
//! must land through whatever the manager runs and leave
//!  the record unwritten so the next commit repairs the text,
//! rather than fail with a diagnosis that is not true of the
//!  machine.
//! The manager here is a script,
//! so none of these reach the account's own manager.

use super::*;
use crate::service_instance::{program_link, service_unit_instance};
use std::path::PathBuf;
use crate::service_provision::unit_texts;
use crate::test_support::{fixture, remove};
use std::time::Duration;

/// The instance the fixture worktree has.
const INSTANCE: &str = "fedcba9876543210fedcba9876543210";

/// How many runtime directories this process has made,
/// so two controls never share one.
static RUNTIME_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// One answer a scripted manager gives to a question about a unit.
#[derive(Clone)]
enum Answer {
    /// The unit is in this state.
    State(&'static str),
    /// The manager holds no such unit.
    Gone,
    /// The manager did not answer within the time the call was given.
    Silent,
}

/// What:
///  A manager that answers this worktree's two units from a script and takes every stop it is asked for.
/// Why:
///   The state that fails a commit is a sequence,
/// not a single answer:
/// the questions before the unload are answered,
/// the stop is
///  taken,
/// and only then does the manager go quiet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const manager = { service: ['active', 'inactive', 'silent'], socket: ['active'], stops: [] };
/// ```
struct Scripted {
    service: Vec<Answer>,
    socket: Vec<Answer>,
    /// How many state questions about each unit were answered.
    service_asked: usize,
    socket_asked: usize,
    /// Every stop,
    /// by unit,
    /// in order.
    stops: Vec<String>,
}

impl Scripted {
    /// A manager holding a running service and its socket,
    /// whatever happens after.
    fn new(service: Vec<Answer>, socket: Vec<Answer>) -> Self {
        return Self { service, socket, service_asked: 0, socket_asked: 0, stops: Vec::new() };
    }

    /// The next answer of one script,
    /// the last one repeating so a poll loop runs out inside it.
    fn next(answers: &[Answer], asked: &mut usize) -> Answer {
        let answer: Answer = answers.get(*asked).or_else(|| { return answers.last(); }).cloned().unwrap_or(Answer::Gone);
        *asked += 1;
        return answer;
    }
}

impl Manager for Scripted {
    fn within(&mut self, _wait: Duration) {}

    fn reset_failed(&mut self, _unit: &str) -> Result<(), CallError> {
        return Ok(());
    }

    fn start(&mut self, _unit: &str) -> Result<(), CallError> {
        return Ok(());
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        self.stops.push(unit.to_string());
        return Ok(());
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        self.stops.push(unit.to_string());
        return Ok(());
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        let answer: Answer = if unit == service_unit_instance(INSTANCE) {
            Self::next(self.service.as_slice(), &mut self.service_asked)
        } else if unit == socket_unit_instance(INSTANCE) {
            Self::next(self.socket.as_slice(), &mut self.socket_asked)
        } else {
            // The earlier releases' units of this worktree are not held.
            Answer::Gone
        };
        return match answer {
            Answer::State(text) => Ok(Some(String::from(text))),
            Answer::Gone => Ok(None),
            Answer::Silent => Err(CallError::TimedOut),
        };
    }

    fn listen(&mut self, _unit: &str) -> Result<Option<Vec<String>>, CallError> {
        return Ok(None);
    }
}

/// What a control keeps alive:
/// the fixture root it removes afterwards,
/// and the host the calls are made for.
struct Machine {
    root: PathBuf,
    host: ProvisionHost,
}

/// What:
///  A machine of a control's own:
/// a fixture root,
/// a runtime directory short enough for a socket address,
/// and the waits a
///  control may want short.
/// Why:
///   The unload's waits are the shipped ones,
/// so a control that reads them in real time would take the whole
///  retirement wait;
/// the wait is passed in the host,
/// which is how the crate's other controls read a deadline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const machine = machine('unload-silent-lib', 100); // a 100 ms retirement wait
/// ```
fn machine(name: &str, retirement_wait: Duration) -> Machine {
    let root: PathBuf = fixture(name);
    // What: The runtime directory stands under the agent scratch directory, not under the fixture root.
    // Why:  A socket address is the runtime directory plus 66 bytes and the kernel holds at most 107, so the build
    //       directory's depth cannot be part of it.
    let count: usize = RUNTIME_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let scratch: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    let runtime: PathBuf = scratch.join(format!("unload-{}-{count}", std::process::id()));
    let _removed = std::fs::remove_dir_all(&runtime);
    std::fs::create_dir_all(&runtime).expect("runtime directory");
    crate::test_support::track_for_this_thread(runtime.as_path());
    let host: ProvisionHost = ProvisionHost {
        bus: runtime.join("bus"),
        runtime_dir: runtime,
        user_unit_dir: Some(root.join("config/systemd/user")),
        service_binary: root.join("programs/monochromatic-git-policy-cli"),
        git: PathBuf::from("/usr/bin/git"),
        instance: String::from(INSTANCE),
        git_dir: root.join("repository/.git/worktrees/topic"),
        retirement_wait,
    };
    return Machine { root, host };
}

/// Whether the record of what the manager read says it read these texts for this instance.
fn record_says_read(machine: &Machine) -> bool {
    return crate::service_provision_loaded_unit_texts::remembers(machine.host.runtime_dir.as_path(), INSTANCE, unit_texts(&machine.host).as_slice());
}

/// What:
///  A manager that answered the stop and then stopped answering leaves the commit to go on.
/// Why:
///   The manager was there a moment before,
/// so a line that says this account has no user service manager is false and its
///  remedy cannot help;
/// the unload is a repair the next commit repeats,
/// and nothing was dropped,
/// so the record must stay
///  unwritten and the commit must land.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await provisionWith(host, managerWhichGoesSilentAfterTheStop)).toBe(undefined); expect(recordExists()).toBe(false);
/// ```
#[test]
fn a_manager_that_stops_answering_after_the_stop_leaves_the_commit_to_go_on() {
    let machine: Machine = machine("unload-silent-manager-lib", Duration::from_secs(10));
    let mut manager: Scripted = Scripted::new(
        vec![Answer::State("active"), Answer::State("inactive"), Answer::Silent],
        vec![Answer::State("active"), Answer::Silent],
    );
    let said: Vec<String> = crate::service_timing::capture::entries_of(|| {
        crate::service_provision::provision_with(&machine.host, &mut manager).expect("a manager that answered the stop and then went quiet does not fail the commit");
    })
    .iter()
    .map(|entry| { return entry.message.clone(); })
    .collect();
    // What: No line may name a missing user service manager: one answered the stop a moment before.
    // Why:  That diagnosis sends the person to `loginctl enable-linger`, which cannot make a busy manager answer.
    assert!(!said.iter().any(|line| { return line.contains("no user service manager") || line.contains("loginctl"); }), "{said:?}");
    assert!(said.iter().any(|line| { return line.contains(instance_service().as_str()); }), "a line names this worktree's own unit: {said:?}");
    assert!(!record_says_read(&machine), "nothing was dropped, so the record stays unwritten and the next commit asks again");
    remove(machine.root.as_path());
}

/// What:
///  A service that is still running at the retirement wait is landing a batch,
/// so the commit goes on through it and nothing
///  is stopped.
/// Why:
///   A landing may hold the index for longer than the retirement wait
/// (`wait_ceilings.rs`,
/// `LANDING_LOCK_CEILING`,
/// a minute),
/// and a
///  commit that waited for it would have landed through the very service that is busy;
/// stopping its unit would cut the landing,
///  which is why nothing is stopped here either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await provisionWith(host, managerWithAServiceThatNeverLeaves)).toBe(undefined); expect(manager.stops).toEqual([]);
/// ```
#[test]
fn a_service_that_is_still_landing_leaves_the_commit_to_go_on_and_stops_nothing() {
    let machine: Machine = machine("unload-landing-service-lib", Duration::from_millis(100));
    let mut manager: Scripted = Scripted::new(vec![Answer::State("active")], vec![Answer::State("active")]);
    let said: Vec<String> = crate::service_timing::capture::entries_of(|| {
        crate::service_provision::provision_with(&machine.host, &mut manager).expect("a service that is landing does not fail the commit");
    })
    .iter()
    .map(|entry| { return entry.message.clone(); })
    .collect();
    assert!(manager.stops.is_empty(), "nothing of a service that is landing is stopped: {:?}", manager.stops);
    assert!(said.iter().any(|line| { return line.contains("still running"); }), "one line says what it found: {said:?}");
    assert!(!record_says_read(&machine), "the text was not dropped, so the record stays unwritten and the next commit asks again");
    remove(machine.root.as_path());
}

/// What:
///  A program link that cannot be pointed does not fail the commit:
/// the units are written,
/// one line says what the
///  link names instead,
/// and a start runs whatever that is.
/// Why:
///   The link is how a start finds this release,
/// and a rename that fails (the path is a directory,
/// say) leaves the link
///  naming what it named before,
/// which may still be a program that answers this commit;
/// failing the commit there loses a change
///  the service could have landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await provisionWith(hostWithADirectoryAtTheLinkPath, manager)).toBe(undefined);
/// ```
#[test]
fn a_program_link_that_cannot_be_pointed_leaves_the_commit_to_go_on() {
    let machine: Machine = machine("unload-link-refused-lib", Duration::from_secs(10));
    let link: PathBuf = program_link(machine.host.runtime_dir.as_path());
    std::fs::create_dir_all(&link).expect("occupy the link path with a directory");
    let mut manager: Scripted = Scripted::new(vec![Answer::State("active"), Answer::State("inactive"), Answer::Gone], vec![Answer::State("active"), Answer::Gone]);
    let said: Vec<String> = crate::service_timing::capture::entries_of(|| {
        crate::service_provision::provision_with(&machine.host, &mut manager).expect("a link that cannot be pointed does not fail the commit");
    })
    .iter()
    .map(|entry| { return entry.message.clone(); })
    .collect();
    assert!(said.iter().any(|line| { return line.contains(link.to_string_lossy().as_ref()); }), "one line names the link it could not point: {said:?}");
    remove(machine.root.as_path());
}

/// What:
///  A unit the manager still holds when the wait runs out leaves the commit to go on,
/// and the record of what the
///  manager read is not written.
/// Why:
///   A unit another command started again inside this wait,
/// or one whose collection the manager has not reached,
///  is no
/// reason to end a commit:
/// the manager runs something that answers this commit or it does not,
/// and what it
///  runs is what the start and the client's own wait see.
/// Reaching this wait is the one way a commit could have
///  ended here before,
/// and it does not any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await provisionWith(host, managerWhichKeepsTheUnit)).toBe(undefined);
/// ```
#[test]
fn a_unit_the_manager_keeps_past_the_wait_leaves_the_commit_to_go_on_and_the_record_unwritten() {
    let machine: Machine = machine("unload-unit-kept-lib", Duration::from_secs(10));
    // What: The socket is stopped and gone,
    // and the service is still there at every question after it.
    // Why:  That is what a second command's start, or a collection the manager has not reached, leaves.
    let mut manager: Scripted = Scripted::new(
        vec![Answer::State("active"), Answer::State("inactive"), Answer::State("active")],
        vec![Answer::State("active")],
    );
    let said: Vec<String> = crate::service_timing::capture::entries_of(|| {
        crate::service_provision::provision_with(&machine.host, &mut manager).expect("a unit the manager keeps does not fail the commit");
    })
    .iter()
    .map(|entry| { return entry.message.clone(); })
    .collect();
    assert!(
        said.iter().any(|line| { return line.contains("still holds this worktree's unit") && line.contains(INSTANCE); }),
        "one line says the manager still holds one of this worktree's units, and the commit goes on: {said:?}"
    );
    assert!(!record_says_read(&machine), "the text was not dropped, so the record stays unwritten and the next commit asks again");
    remove(machine.root.as_path());
}

/// The service unit of the fixture's instance.
fn instance_service() -> String {
    return service_unit_instance(INSTANCE);
}
