//! What:
//!  The timing record a wrapped commit appends to the repository and the lines the commit service writes to its log,
//! run as
//!  the real programs against a real repository.
//! Why:
//!  After a slow commit,
//! its time has to be split by phase from logs that exist without anyone having switched anything on.
//! A
//!  held index lock is made slow on purpose,
//! and a service of another release is put in the way,
//! to show that the record and
//!  the lines attribute the time to the phase that spent it.
//! The services are started by the test at sockets under the test's
//!  own runtime directory,
//! with their standard error in a file the test reads,
//! so nothing touches the user's service manager or
//!  journal;
//! the real `.git/cli-git` of this repository is never used.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::runtime_for;
use super::service_landing_lock::foreign_commit;
use super::service_landing_selection::base_repository;
use super::support::{Fixture, fixture, observe};
use git_policy_cli::commit_timing::TIMING_FILE;
use git_policy_cli::service_client::ask_to_retire;
use git_policy_cli::service_instance::{instance_of, socket_for};
use git_policy_cli::service_protocol::RELEASE;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const SERVICE: &str = env!("CARGO_BIN_EXE_cli-git-service");
const PATIENCE: Duration = Duration::from_secs(60);

/// The seconds a service waits idle in a control that does not test the idle exit,
/// so one left behind by a failed control ends.
const SAFETY_IDLE_SECONDS: &str = "120";

/// The seconds the idle exit is given in its control.
const IDLE_SECONDS: u64 = 1;

/// The seconds the foreign commit keeps the index lock.
const HOLD_SECONDS: u32 = 4;

/// A message no log line may carry:
/// the commit's text is the person's.
const MESSAGE: &str = "timing control message that no log may repeat";

/// The release of a service that is not the one this build starts.
const OTHER_RELEASE: &str = "0.0.0-timing-control";

/// A service process that is stopped when the control ends,
/// whether it passed or panicked.
struct Held(Child);

impl Drop for Held {
    fn drop(&mut self) {
        let _killed = self.0.kill();
        let _waited = self.0.wait();
    }
}

/// Where a service listens and where its standard error goes.
struct Places {
    socket: PathBuf,
    log: PathBuf,
}

/// Starts a service at `places.socket` with its standard error in `places.log`,
/// once it listens.
fn spawn_service(root: &Path, places: &Places, idle_seconds: &str, extra: &[&str]) -> Held {
    std::fs::create_dir_all(places.socket.parent().expect("parent")).expect("socket directory");
    let log = std::fs::OpenOptions::new().create(true).append(true).open(&places.log).expect("service log");
    let child: Child = Command::new(SERVICE)
        .args(["--journal"])
        .arg(root.join("journal"))
        .arg("--path")
        .arg(&places.socket)
        .args(["--idle-seconds", idle_seconds])
        .args(extra)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", root.join("home"))
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("CLI_GIT_LOCK_WAIT_SECONDS", "60")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(log))
        .spawn()
        .expect("the service starts");
    let held: Held = Held(child);
    let began: Instant = Instant::now();
    while std::os::unix::net::UnixStream::connect(&places.socket).is_err() {
        assert!(began.elapsed() < PATIENCE, "the service never listened");
        std::thread::sleep(Duration::from_millis(10));
    }
    return held;
}

/// The places of the service of the worktree `repo`,
/// under a runtime directory named for the control.
fn places_for(fixture: &Fixture, name: &str, repo: &Path) -> Places {
    let runtime: PathBuf = runtime_for(name);
    return Places { socket: socket_for(runtime.as_path(), repo.join(".git").as_path()), log: fixture.root.join(format!("{name}.log")) };
}

/// The instance argument that makes a service serve the worktree `repo` only,
/// as the unit does.
fn instance_argument(repo: &Path) -> String {
    return instance_of(repo.join(".git").as_path());
}

/// Waits for the service to exit by itself,
/// at most `limit`.
fn exit_of(service: &mut Held, limit: Duration) -> std::process::ExitStatus {
    let began: Instant = Instant::now();
    loop {
        if let Ok(Some(status)) = service.0.try_wait() {
            return status;
        }
        assert!(began.elapsed() < limit, "the service did not exit within {limit:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The lines of a service log that begin with the word after the service's prefix.
fn lines_of(log: &Path, word: &str) -> Vec<String> {
    let text: String = std::fs::read_to_string(log).expect("service log");
    let prefix: String = format!("{}{word}", git_policy_cli::service_timing::LOG_PREFIX);
    return text.lines().filter(|line| { return line.starts_with(prefix.as_str()); }).map(str::to_string).collect();
}

/// The whole milliseconds a `key=<digits>` word of a line states.
fn ms_of(line: &str, key: &str) -> u64 {
    let marker: String = format!(" {key}=");
    let from: usize = line.find(marker.as_str()).unwrap_or_else(|| { panic!("no {key} in {line}") }) + marker.len();
    let digits: String = line[from..].chars().take_while(char::is_ascii_digit).collect();
    return digits.parse::<u64>().unwrap_or_else(|_| { panic!("no number for {key} in {line}") });
}

/// The records of the repository's timing file,
/// parsed.
fn records(repo: &Path) -> Vec<serde_json::Value> {
    let text: String = std::fs::read_to_string(repo.join(".git").join(TIMING_FILE)).expect("the timing file");
    return text.lines().map(|line| { return serde_json::from_str(line).expect("a JSON line"); }).collect();
}

/// A number field of a record.
fn number(record: &serde_json::Value, name: &str) -> u64 {
    return record[name].as_u64().unwrap_or_else(|| { panic!("no number {name} in {record}") });
}

/// One wrapped commit of `a.txt` through the service at `runtime`,
/// observed.
fn commit_a(fixture: &Fixture, repo: &Path, runtime: &Path) -> super::support::Observed {
    std::fs::write(repo.join("a.txt"), format!("edit by {}\n", std::process::id())).expect("a.txt");
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "-m", MESSAGE, "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime);
    return observe(&mut command, b"");
}

/// The phases the record counts as disjoint stretches of the command.
const DISJOINT: [&str; 8] = ["prepareMs", "buildMs", "provisionMs", "submitMs", "afterLandingMs", "gateMs", "handoffMs", "otherMs"];

/// An ordinary commit appends one record to the repository and gets one line from the service,
/// and neither the terminal nor the
///  logs carry the commit's text or the committer's address.
/// The record's phases add up to its total,
/// and its instance is the
///  one the service says it serves,
/// so the record names the unit to read the journal of.
#[test]
fn a_wrapped_commit_leaves_one_record_and_the_service_one_line_for_it() {
    let fixture: Fixture = fixture("timing-record");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("timing-record-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let _service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    let identity: String = git_text(&fixture, &repo, &["var", "GIT_COMMITTER_IDENT"]);
    let address: &str = identity.split_once('<').and_then(|(_, rest)| { return rest.split_once('>'); }).map(|(inside, _)| { return inside; }).expect("an address");
    let branch: String = git_text(&fixture, &repo, &["symbolic-ref", "--short", "HEAD"]).trim().to_string();
    let served = commit_a(&fixture, &repo, runtime_for(name.as_str()).as_path());
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    for (stream, bytes) in [("stdout", &served.stdout), ("stderr", &served.stderr)] {
        let text: String = String::from_utf8_lossy(bytes).into_owned();
        assert!(!text.contains(TIMING_FILE) && !text.contains("totalMs") && !text.contains("_ms="), "timing reached the terminal on {stream}: {text}");
    }
    let found: Vec<serde_json::Value> = records(&repo);
    assert_eq!(found.len(), 1, "one record for one commit: {found:?}");
    let record: &serde_json::Value = &found[0];
    println!("TIMING RECORD: {record}");
    assert_eq!(record["outcome"], "landed");
    assert_eq!(record["exitCode"], 0);
    assert_eq!(record["branch"], format!("refs/heads/{branch}"));
    assert_eq!(record["release"], RELEASE);
    assert_eq!(record["instance"], instance.as_str());
    assert_eq!(record["answeredByRelease"], RELEASE);
    assert_eq!(record["attempts"], 1);
    assert_eq!(record["sends"], 1);
    assert!(number(record, "connectTries") >= 1, "{record}");
    let phases: u64 = DISJOINT.iter().map(|field| { return number(record, field); }).sum();
    // Each term is cut to whole milliseconds on its own, so the sum falls short of the total by less than one per term.
    let slack: u64 = u64::try_from(DISJOINT.len()).expect("a small count");
    assert!(phases <= number(record, "totalMs") && number(record, "totalMs") - phases < slack, "the phases and the remainder add up to the total: {record}");
    assert!(number(record, "reachMs") + number(record, "answerWaitMs") <= number(record, "submitMs"), "{record}");
    let started: Vec<String> = lines_of(&places.log, "started");
    println!("SERVICE START: {}", started.join("\n"));
    assert_eq!(started.len(), 1, "{started:?}");
    assert!(started[0].contains(format!("release={RELEASE:?}").as_str()) && started[0].contains(format!("instance={instance}").as_str()), "{}", started[0]);
    assert!(started[0].contains(format!("pid={}", _service.0.id()).as_str()), "{}", started[0]);
    assert!(std::fs::read_to_string(&places.log).expect("log").contains("ready"), "the ready line stays");
    let answered: Vec<String> = lines_of(&places.log, "answered");
    println!("SERVICE ANSWER: {}", answered.join("\n"));
    assert_eq!(answered.len(), 1, "one line for one request: {answered:?}");
    let line: &str = answered[0].as_str();
    assert!(line.contains("outcome=landed") && line.contains(format!("branch=\"refs/heads/{branch}\"").as_str()), "{line}");
    assert!(line.contains(" size=1 position=1 "), "{line}");
    let request_phases: u64 = ["read_ms", "queued_ms", "settle_ms", "landing_ms", "answer_ms"].iter().map(|key| { return ms_of(line, key); }).sum();
    assert!(request_phases <= ms_of(line, "total_ms"), "the phases lie inside the request's total: {line}");
    let landing_keys: [&str; 6] = ["lock_wait_ms", "reads_ms", "objects_ms", "journal_ms", "ref_ms", "sync_ms"];
    let landing_phases: u64 = landing_keys.iter().map(|key| { return ms_of(line, key); }).sum();
    let landing_total: u64 = ms_of(line, "landing_ms");
    assert!(landing_phases <= landing_total && landing_total - landing_phases < u64::try_from(landing_keys.len()).expect("a small count"), "{line}");
    assert!(!line.contains("lock_holder"), "nobody held the index: {line}");
    let everything: String = std::fs::read_to_string(&places.log).expect("log") + &std::fs::read_to_string(repo.join(".git").join(TIMING_FILE)).expect("record");
    assert!(!everything.contains(MESSAGE), "the commit's text is in a log");
    assert!(!everything.contains(address), "the committer's address is in a log");
    remove_settled(&fixture);
}

/// A foreign commit holds the index lock while the wrapped commit lands:
/// the service's line puts the time in the wait for the
///  lock and names the holder,
/// the wait lies inside the time the wrapper's record gives the answer,
/// and no other phase of
///  either comes near it.
#[test]
fn a_commit_that_waits_for_a_foreign_index_holder_is_attributed_to_that_wait() {
    let fixture: Fixture = fixture("timing-lock");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("timing-lock-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let _service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    let holder: Child = foreign_commit(&fixture, &repo, HOLD_SECONDS);
    let served = commit_a(&fixture, &repo, runtime_for(name.as_str()).as_path());
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let answered: Vec<String> = lines_of(&places.log, "answered");
    assert_eq!(answered.len(), 1, "{answered:?}");
    let line: &str = answered[0].as_str();
    println!("SERVICE ANSWER (held index): {line}");
    let lock_wait: u64 = ms_of(line, "lock_wait_ms");
    let hold_ms: u64 = u64::from(HOLD_SECONDS) * 1_000;
    assert!(lock_wait >= hold_ms / 2, "the wait for the lock is most of the hold: {line}");
    assert!(line.contains("lock_holder=\"Process "), "the holder is named: {line}");
    for other in ["read_ms", "queued_ms", "settle_ms", "reads_ms", "objects_ms", "journal_ms", "ref_ms", "sync_ms", "answer_ms"] {
        assert!(ms_of(line, other) < lock_wait, "{other} is not what the time went to: {line}");
    }
    let found: Vec<serde_json::Value> = records(&repo);
    let record: &serde_json::Value = found.last().expect("the wrapped commit's record");
    println!("TIMING RECORD (held index): {record}");
    assert!(number(record, "answerWaitMs") >= lock_wait, "the wrapper waited at least as long as the service did for the lock: {record}");
    for other in ["prepareMs", "buildMs", "provisionMs", "afterLandingMs", "gateMs", "handoffMs", "reachMs"] {
        assert!(number(record, other) < number(record, "answerWaitMs"), "{other} is not what the time went to: {record}");
    }
    assert!(number(record, "waitingNotices") >= 1, "the service told the wrapper it was waiting: {record}");
    remove_settled(&fixture);
}

/// A service of another release answers first:
/// the wrapper asks it to retire,
/// the service says so and exits,
/// the current
///  release is reached,
/// and the record states the retire rounds and both releases in the order they greeted.
#[test]
fn a_commit_that_meets_another_release_records_the_update_and_both_services_log_it() {
    let fixture: Fixture = fixture("timing-update");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("timing-update-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let current_log: PathBuf = fixture.root.join("current.log");
    let instance: String = instance_argument(&repo);
    let retiring: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str(), "--release", OTHER_RELEASE]);
    // What the service manager does on the next connection once the old service has exited: start the current one.
    let successor = {
        let socket: PathBuf = places.socket.clone();
        let root: PathBuf = fixture.root.clone();
        let argument: String = instance.clone();
        let log: PathBuf = current_log.clone();
        std::thread::spawn(move || {
            let mut old: Held = retiring;
            let status = exit_of(&mut old, PATIENCE);
            let next: Held = spawn_service(root.as_path(), &Places { socket, log }, SAFETY_IDLE_SECONDS, &["--instance", argument.as_str()]);
            return (status, next);
        })
    };
    let served = commit_a(&fixture, &repo, runtime_for(name.as_str()).as_path());
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let (old_status, _current): (std::process::ExitStatus, Held) = successor.join().expect("the successor thread");
    assert!(old_status.success(), "the old service exited by itself: {old_status:?}");
    let found: Vec<serde_json::Value> = records(&repo);
    let record: &serde_json::Value = &found[0];
    println!("TIMING RECORD (other release): {record}");
    assert!(number(record, "updateRounds") >= 1, "{record}");
    assert_eq!(record["releasesSeen"], serde_json::Value::from(vec![OTHER_RELEASE, RELEASE]));
    assert_eq!(record["answeredByRelease"], RELEASE);
    assert!(number(record, "connectTries") > number(record, "updateRounds"), "{record}");
    assert!(number(record, "startWaitMs") <= number(record, "reachMs"), "{record}");
    let retired: Vec<String> = lines_of(&places.log, "retire requested");
    println!("OLD SERVICE: {}", std::fs::read_to_string(&places.log).expect("log"));
    // The client sends a retire in every round until the old service exits, and the service logs each one it reads; one it
    //  exited before reading is not logged, so the lines are at least one and at most the rounds the record states.
    let rounds: usize = usize::try_from(number(record, "updateRounds")).expect("a small count");
    assert!(!retired.is_empty() && retired.len() <= rounds, "{retired:?} against {rounds} rounds");
    assert!(retired.iter().all(|line| { return line.contains(format!("{OTHER_RELEASE:?}").as_str()); }), "{retired:?}");
    let exits: Vec<String> = lines_of(&places.log, "exiting");
    assert_eq!(exits.len(), 1, "{exits:?}");
    assert!(exits[0].contains("reason=retire"), "{}", exits[0]);
    assert_eq!(lines_of(&current_log, "answered").len(), 1, "the current release landed it");
    assert_eq!(lines_of(&places.log, "answered").len(), 0, "the old release landed nothing");
    remove_settled(&fixture);
}

/// A service with nothing to do exits after the idle period and says so,
/// with an uptime at least as long as the period.
#[test]
fn a_service_says_it_exits_idle_with_its_uptime() {
    let fixture: Fixture = fixture("timing-idle");
    let name: String = format!("timing-idle-{}", std::process::id());
    let places: Places = Places { socket: runtime_for(name.as_str()).join("cli-git/idle.sock"), log: fixture.root.join("idle.log") };
    let mut service: Held = spawn_service(fixture.root.as_path(), &places, IDLE_SECONDS.to_string().as_str(), &[]);
    let status = exit_of(&mut service, PATIENCE);
    assert!(status.success(), "{status:?}");
    let exits: Vec<String> = lines_of(&places.log, "exiting");
    println!("SERVICE EXIT (idle): {}", exits.join("\n"));
    assert_eq!(exits.len(), 1, "{exits:?}");
    assert!(exits[0].contains("reason=idle"), "{}", exits[0]);
    assert!(ms_of(exits[0].as_str(), "uptime_ms") >= IDLE_SECONDS * 1_000, "{}", exits[0]);
    remove_settled(&fixture);
}

/// A service asked to retire logs the request and its exit.
#[test]
fn a_service_says_it_retires_when_asked() {
    let fixture: Fixture = fixture("timing-retire");
    let name: String = format!("timing-retire-{}", std::process::id());
    let places: Places = Places { socket: runtime_for(name.as_str()).join("cli-git/retire.sock"), log: fixture.root.join("retire.log") };
    let mut service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &[]);
    ask_to_retire(places.socket.as_path());
    let status = exit_of(&mut service, PATIENCE);
    assert!(status.success(), "{status:?}");
    println!("SERVICE RETIRE: {}", std::fs::read_to_string(&places.log).expect("log"));
    assert_eq!(lines_of(&places.log, "retire requested").len(), 1);
    let exits: Vec<String> = lines_of(&places.log, "exiting");
    assert_eq!(exits.len(), 1, "{exits:?}");
    assert!(exits[0].contains("reason=retire"), "{}", exits[0]);
    remove_settled(&fixture);
}

/// The service manager stops a unit with `SIGTERM`:
/// the service writes its last line and still ends from the signal,
/// as it
///  would have with no handler.
#[test]
fn a_service_stopped_by_a_signal_says_so_and_still_dies_of_it() {
    let fixture: Fixture = fixture("timing-signal");
    let name: String = format!("timing-signal-{}", std::process::id());
    let places: Places = Places { socket: runtime_for(name.as_str()).join("cli-git/signal.sock"), log: fixture.root.join("signal.log") };
    let mut service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &[]);
    let pid: libc::pid_t = libc::pid_t::try_from(service.0.id()).expect("a process number");
    // SAFETY: `kill` takes a process number and a signal number and touches no memory of this process.
    let sent: libc::c_int = unsafe { libc::kill(pid, libc::SIGTERM) };
    assert_eq!(sent, 0);
    let status = exit_of(&mut service, PATIENCE);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "{status:?}");
    let exits: Vec<String> = lines_of(&places.log, "exiting");
    println!("SERVICE EXIT (signal): {}", exits.join("\n"));
    assert_eq!(exits.len(), 1, "{exits:?}");
    assert!(exits[0].contains(format!("reason=signal-{}", libc::SIGTERM).as_str()), "{}", exits[0]);
    remove_settled(&fixture);
}
