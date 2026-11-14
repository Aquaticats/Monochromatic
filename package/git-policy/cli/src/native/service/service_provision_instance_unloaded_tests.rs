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
use crate::service_server_socket_binding::Bind;
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
    /// by unit and by the mode it was sent in,
    /// in order.
    stops: Vec<String>,
    /// What the service was doing when the last stop was sent,
    /// which is the state no stop may end at a landing.
    service_state_at_the_stop: Option<String>,
    /// How the service answers from the socket's stop onwards,
    /// which is where a control models a concurrent commit starting it inside the window.
    after_the_socket_stop: Option<Answer>,
}

impl Scripted {
    /// A manager holding a running service and its socket,
    /// whatever happens after.
    fn new(service: Vec<Answer>, socket: Vec<Answer>) -> Self {
        return Self { service, socket, service_asked: 0, socket_asked: 0, stops: Vec::new(), service_state_at_the_stop: None, after_the_socket_stop: None };
    }

    /// Makes the service answer `answer` from the socket's stop onwards,
    /// which is a concurrent commit starting it inside the unload's window.
    fn started_again_after_the_socket_stop(mut self, answer: Answer) -> Self {
        self.after_the_socket_stop = Some(answer);
        return self;
    }

    /// What the service does when a stop job runs,
    /// which is after any start a concurrent command made.
    fn state_when_the_stop_runs(&mut self) -> Option<String> {
        if let Some(answer) = self.after_the_socket_stop.clone() {
            return match answer {
                Answer::State(text) => Some(String::from(text)),
                Answer::Gone | Answer::Silent => None,
            };
        }
        return match Self::state_of(self, service_unit_instance(INSTANCE).as_str()) {
            Answer::State(text) => Some(String::from(text)),
            Answer::Gone | Answer::Silent => None,
        };
    }

    /// What the script says the service does now,
    /// which every question after the socket's stop is answered from when a control set that.
    fn state_of(&mut self, unit: &str) -> Answer {
        // What: The service a concurrent commit starts is there from the moment the first stop job is sent, and from then on.
        // Why:  That start is what the look could not see, and the stop's job runs after it.
        let window_opened: bool = !self.stops.is_empty();
        if window_opened
            && let Some(answer) = self.after_the_socket_stop.clone()
            && unit == service_unit_instance(INSTANCE)
        {
            return answer;
        }
        return if unit == service_unit_instance(INSTANCE) {
            Self::next(self.service.as_slice(), &mut self.service_asked)
        } else if unit == socket_unit_instance(INSTANCE) {
            Self::next(self.socket.as_slice(), &mut self.socket_asked)
        } else {
            // The earlier releases' units of this worktree are not held.
            Answer::Gone
        };
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
        // What: A stop in the mode that carries through `Requires=`, which no control here may see for a service.
        // Why:  That mode is what ends a service another command started inside the unload's window, and the repair may not
        //       use it (`service_provision_instance_unloaded.rs`).
        self.service_state_at_the_stop = self.state_when_the_stop_runs();
        self.stops.push(format!("stop {unit}"));
        return Ok(());
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        self.service_state_at_the_stop = self.state_when_the_stop_runs();
        self.stops.push(format!("stop-alone {unit}"));
        return Ok(());
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        let answer: Answer = Self::state_of(self, unit);
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
    let mut manager: Scripted = Scripted::new(vec![Answer::State("active"), Answer::State("inactive")], vec![Answer::State("active")]);
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

/// What:
///  A commit that starts to land between the unload's look and its stop is never stopped:
/// the socket is stopped
///  alone,
/// which cannot carry through `Requires=` to the service,
/// and the service is asked to retire instead.
/// Why:
///   A plain stop of the socket ends the service the unit requires,
/// so a service another command started inside that window was cut in the middle
///  of a landing,
/// with no retire ask made
/// (the review of `575c585373`,
/// its reproduction `unload-stop-race`).
/// That is the
///  one thing this repair must not do:
/// a cut batch is where "not known whether it landed" comes from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(await provisionWith(host, managerWhichStartsTheServiceInTheWindow).stops).toEqual(['stop-alone <socket>']);
/// ```
#[test]
fn a_service_started_inside_the_unloads_window_is_never_stopped() {
    let machine: Machine = machine("unload-stop-race-lib", Duration::from_millis(300));
    // What: The first look finds the service at rest and its socket held,
    // and a concurrent commit starts the service the moment the socket is stopped.
    // Why:  That is the window: the look said nothing was running, and the stop's job runs after the start.
    let mut manager: Scripted = Scripted::new(vec![Answer::State("inactive")], vec![Answer::State("active")])
        .started_again_after_the_socket_stop(Answer::State("active"));
    let said: Vec<String> = crate::service_timing::capture::entries_of(|| {
        crate::service_provision::provision_with(&machine.host, &mut manager).expect("a service that is landing does not fail the commit");
    })
    .iter()
    .map(|entry| { return entry.message.clone(); })
    .collect();
    // What: The only stop the unload sent, and what the service was doing when it was sent.
    // Why:  A stop of the service, or a stop of the socket in the mode that carries through `Requires=`, is what cuts the
    //       landing; the socket stopped alone leaves the service running.
    assert_eq!(manager.stops, vec![format!("stop-alone {}", socket_unit_instance(INSTANCE))], "the only stop names the socket and leaves the service alone");
    assert_eq!(manager.service_state_at_the_stop.as_deref(), Some("active"), "the service was landing when the socket was stopped, and nothing of it was stopped");
    assert!(
        said.iter().any(|line| { return line.contains("started again while its units were being dropped"); }),
        "one line says the service started inside the wait and was left: {said:?}"
    );
    assert!(!record_says_read(&machine), "the units were not watched being dropped, so the record stays unwritten and the next commit asks again");
    remove(machine.root.as_path());
}

/// The service unit of the fixture's instance.
fn instance_service() -> String {
    return service_unit_instance(INSTANCE);
}

/// The socket unit of the fixture's instance.
fn instance_socket() -> String {
    return crate::service_instance::socket_unit_instance(INSTANCE);
}

/// What:
///  A machine whose runtime directory is too long for a socket path,
/// so the worktree's service is listened at under an abstract name,
/// with the waits a control may want short.
/// Why:
///   The unload asks the running service to retire before it stops its unit,
/// and the address it asks at is the one the service holds:
/// a runtime directory of forty-one bytes leaves no room for a socket file,
/// so only a name can carry the address (`service_address_of_a_worktree.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const machine = machineAtAnAbstractName('unload-abstract-lib', 2000); // a 2 s retirement wait
/// ```
#[cfg(target_os = "linux")]
fn machine_at_an_abstract_name(name: &str, retirement_wait: Duration) -> Machine {
    let root: PathBuf = fixture(name);
    let count: usize = RUNTIME_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let scratch: PathBuf = PathBuf::from(std::env::var_os("HOME").expect("a home directory")).join("temp/agent");
    // A runtime directory one byte past the longest a socket path carries, built to the byte.
    let holder: PathBuf = scratch.join(format!("unloadname-{}-{count}", std::process::id()));
    std::fs::create_dir_all(&holder).expect("the holder of the runtime directory");
    crate::test_support::track_for_this_thread(holder.as_path());
    let room: usize = (crate::service_instance::longest_runtime_directory(INSTANCE) + 1).saturating_sub(holder.as_os_str().len() + 1);
    let runtime: PathBuf = holder.join("r".repeat(room.max(1)));
    std::fs::create_dir_all(&runtime).expect("runtime directory");
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

/// What:
///  A stand-in for the service of this worktree at an abstract address:
/// it greets a connection and says it was asked to retire when it reads the ask.
/// Why:
///   The ask is the one fact this control is about:
/// a service that is never asked leaves nothing to see but the retirement wait,
/// so the connection itself is the record of the ask.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = standingAtAnAbstractName(address); // asked.flag is set when the retire line arrives
/// ```
#[cfg(target_os = "linux")]
fn standing_at_an_abstract_name(address: &Bind) -> (std::thread::JoinHandle<bool>, std::sync::Arc<std::sync::atomic::AtomicBool>) {
    use std::os::linux::net::SocketAddrExt;
    let Bind::Abstract(name) = address else { panic!("this control stands a service at an abstract name") };
    let socket_address = std::os::unix::net::SocketAddr::from_abstract_name(name.as_bytes()).expect("the name fits an address");
    let listener: std::os::unix::net::UnixListener = std::os::unix::net::UnixListener::bind_addr(&socket_address).expect("a listener at the abstract name");
    let asked: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag: std::sync::Arc<std::sync::atomic::AtomicBool> = std::sync::Arc::clone(&asked);
    let thread: std::thread::JoinHandle<bool> = std::thread::spawn(move || {
        // The accept itself is bounded: a control whose ask is lost must say so in its assertions, not hold the suite in a
        // thread that waits for a connection nothing will make.
        let Ok(()) = listener.set_nonblocking(true) else { return false };
        let began: std::time::Instant = std::time::Instant::now();
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _where)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_refused) => return false,
            }
            if began.elapsed() > Duration::from_secs(5) {
                return false;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        let _back_to_blocking = stream.set_nonblocking(false);
        let _timeout = stream.set_read_timeout(Some(Duration::from_secs(10)));
        let mut welcome: String = crate::service_protocol_encode_messages_to_lines::encode_service(&crate::service_protocol::ServiceMessage::Welcome {
            version: crate::service_protocol::PROTOCOL_VERSION,
            release: None,
        });
        welcome.push('\n');
        if std::io::Write::write_all(&mut stream, welcome.as_bytes()).is_err() {
            return false;
        }
        let mut line: String = String::new();
        if std::io::BufRead::read_line(&mut std::io::BufReader::new(stream.try_clone().expect("a second handle")), &mut line).is_err() {
            return false;
        }
        let heard: bool = matches!(
            crate::service_protocol_decode_lines_to_messages::decode_client(line.trim_end_matches('\n')),
            Ok(crate::service_protocol::ClientMessage::Retire)
        );
        if heard {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        }
        return heard;
    });
    return (thread, asked);
}

/// What:
///  A manager whose service unit is at rest only once the service was really asked to retire.
/// Why:
///   A manager that says the service left whether or not it was asked cannot tell a delivered ask from a lost one:
/// this one holds the service `active` until the stand-in at the abstract name reads the ask,
/// which is what a service finishing its batch looks like from the manager's side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const manager = { service: asked ? 'gone' : 'active', socket: stopped ? 'gone' : 'active' };
/// ```
#[cfg(target_os = "linux")]
struct UntilAsked {
    service: String,
    socket: String,
    asked: std::sync::Arc<std::sync::atomic::AtomicBool>,
    socket_stopped: bool,
}

#[cfg(target_os = "linux")]
impl Manager for UntilAsked {
    fn within(&mut self, _wait: Duration) {}

    fn reset_failed(&mut self, _unit: &str) -> Result<(), CallError> {
        return Ok(());
    }

    fn start(&mut self, _unit: &str) -> Result<(), CallError> {
        return Ok(());
    }

    fn stop(&mut self, unit: &str) -> Result<(), CallError> {
        if unit == self.socket {
            self.socket_stopped = true;
        }
        return Ok(());
    }

    fn stop_alone(&mut self, unit: &str) -> Result<(), CallError> {
        return self.stop(unit);
    }

    fn active_state(&mut self, unit: &str) -> Result<Option<String>, CallError> {
        if unit == self.service {
            let still_running: bool = !self.asked.load(std::sync::atomic::Ordering::SeqCst);
            return Ok(if still_running { Some(String::from("active")) } else { None });
        }
        if unit == self.socket {
            return Ok(if self.socket_stopped { None } else { Some(String::from("active")) });
        }
        return Ok(None);
    }

    fn listen(&mut self, _unit: &str) -> Result<Option<Vec<String>>, CallError> {
        // A manager that cannot say where a unit listens proves nothing either way, which is no refusal.
        return Ok(None);
    }
}

/// What:
///  A service that listens at an abstract name is asked to retire,
/// and with that ask delivered the repair finishes and the record of what the manager read is written,
/// once for the state a kill leaves and again for a unit text that changed while the service ran.
/// Why:
///   The record is the whole point of the unload:
/// it says the manager read the texts on disk,
/// so
///  every later commit takes the fast path.
/// A repair whose ask never arrives never finishes,
/// and each commit then spends the
///  retirement wait and reports a batch landing that is not happening
/// (measured twice at 10,016 ms with no record and the service alive).
/// Both states are reached without a kill:
/// the first is the unit files as this release writes them with the record
///  absent, which a record write that failed leaves and a kill between the two leaves;
/// the second is a unit text
///  that changed while the service ran,
/// which another resolution of Git writes (the service unit names it), or another spelling of the runtime directory
///  (the service unit names the program link under it).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(asked).toBe(true); expect(recordExists()).toBe(true); // then the same again after the service unit's text changed
/// ```
#[test]
#[cfg(target_os = "linux")]
fn a_service_at_an_abstract_name_is_asked_to_retire_and_the_repair_finishes() {
    let mut machine: Machine = machine_at_an_abstract_name("unload-abstract-lib", Duration::from_secs(2));
    let address: Bind = crate::service_address_of_a_worktree::socket_address_of(machine.host.runtime_dir.as_path(), INSTANCE);
    assert!(matches!(address, Bind::Abstract(_)), "this control's runtime directory is served at an abstract name");
    let unit_directory: PathBuf = crate::service_provision::runtime_unit_dir(machine.host.runtime_dir.as_path());
    std::fs::create_dir_all(&unit_directory).expect("the unit directory");
    // What: The unit files say what this release writes and the record of what the manager read is absent.
    // Why:  A kill between the template's write and the record's leaves exactly that, and so does a record write that failed;
    //       every commit finds it until the repair finishes.
    for (name, text) in unit_texts(&machine.host) {
        std::fs::write(unit_directory.join(name), text).expect("the unit file");
    }
    assert!(!record_says_read(&machine), "the record starts absent");
    let (_, asked) = standing_at_an_abstract_name(&address);
    let mut manager: UntilAsked = UntilAsked {
        service: instance_service(),
        socket: instance_socket(),
        asked: std::sync::Arc::clone(&asked),
        socket_stopped: false,
    };
    let began: std::time::Instant = std::time::Instant::now();
    crate::service_provision::provision_with(&machine.host, &mut manager).expect("a repair that cannot be finished does not fail the commit");
    let took: Duration = began.elapsed();
    assert!(asked.load(std::sync::atomic::Ordering::SeqCst), "the service at the abstract name was never asked to retire");
    assert!(record_says_read(&machine), "the ask arrived, so the repair finished and the record of what the manager read is written");
    assert!(took < machine.host.retirement_wait, "the unload did not spend the retirement wait: {took:?}");
    // What: A unit text that changed while the service ran is repaired by asking the running service to retire again.
    // Why:  The same state, reached by a sequence no kill is needed for: the service unit names the program link and the stop
    //       timeout, so a different Git resolution or another spelling of the runtime directory writes another text, and the
    //       service that runs still holds the text the manager read earlier.
    // The Git the service unit names is one word of its text, so another resolution of Git is another text.
    machine.host.git = PathBuf::from("/usr/bin/git-of-another-resolution");
    assert!(!record_says_read(&machine), "the texts this host writes now are not the ones the record names");
    let (_, asked_again) = standing_at_an_abstract_name(&address);
    let mut manager: UntilAsked = UntilAsked {
        service: instance_service(),
        socket: instance_socket(),
        asked: std::sync::Arc::clone(&asked_again),
        socket_stopped: false,
    };
    let began: std::time::Instant = std::time::Instant::now();
    crate::service_provision::provision_with(&machine.host, &mut manager).expect("a text that changed while the service ran is repaired");
    let took: Duration = began.elapsed();
    assert!(asked_again.load(std::sync::atomic::Ordering::SeqCst), "the service running the text from before was asked to retire");
    assert!(record_says_read(&machine), "and the repair wrote the record of the texts the manager now read");
    assert!(took < machine.host.retirement_wait, "the second repair did not spend the retirement wait either: {took:?}");
    remove(machine.root.as_path());
}
