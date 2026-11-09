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
//! the real `.git/monochromatic-git-policy-cli` of this repository is never used.

use super::commit::{git_text, remove_settled, wrapped_commit};
use super::service_commit::runtime_for;
use super::service_landing_lock::foreign_commit_until_released;
use super::service_landing_selection::base_repository;
use super::strace::{strace_available, strace_program};
use super::support::{Fixture, fixture, observe};
use super::wait_notice::{Watched, watch_notices};
use git_policy_cli::commit_timing::TIMING_FILE;
use git_policy_cli::service_client::ask_to_retire;
use git_policy_cli::service_instance::{instance_of, socket_for};
use git_policy_cli::service_protocol::RELEASE;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const PATIENCE: Duration = Duration::from_secs(60);

/// The seconds a service waits idle in a control that does not test the idle exit,
/// so one left behind by a failed control ends.
const SAFETY_IDLE_SECONDS: &str = "120";

/// The seconds the idle exit is given in its control.
const IDLE_SECONDS: u64 = 1;

/// The seconds the foreign commit keeps the index lock after the wrapped commit has been seen waiting for it.
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
    return spawn_service_with_environment(root, places, idle_seconds, extra, &[]);
}

/// `spawn_service` with more variables in the service's otherwise empty environment.
fn spawn_service_with_environment(root: &Path, places: &Places, idle_seconds: &str, extra: &[&str], environment: &[(&str, String)]) -> Held {
    let log = std::fs::OpenOptions::new().create(true).append(true).open(&places.log).expect("service log");
    return spawn_service_with_stderr(root, places, idle_seconds, extra, environment, Stdio::from(log));
}

/// `spawn_service_with_environment` with the service's standard error given,
/// so a control can make it a pipe.
fn spawn_service_with_stderr(root: &Path, places: &Places, idle_seconds: &str, extra: &[&str], environment: &[(&str, String)], stderr: Stdio) -> Held {
    std::fs::create_dir_all(places.socket.parent().expect("parent")).expect("socket directory");
    let child: Child = super::support::service_command()
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
        .envs(environment.iter().map(|(name, value)| { return (*name, value.as_str()); }))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(stderr)
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

/// The wrapped commit of `a.txt` through the service at `runtime`,
/// not yet run.
fn commit_a_command(fixture: &Fixture, repo: &Path, runtime: &Path) -> Command {
    // A different text each time, so a second commit in one control has something to commit.
    static ROUND: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let round: usize = ROUND.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    std::fs::write(repo.join("a.txt"), format!("edit {round} by {}\n", std::process::id())).expect("a.txt");
    let mut command: Command = wrapped_commit(fixture, repo, &["commit", "-m", MESSAGE, "--", "a.txt"]);
    command.env("XDG_RUNTIME_DIR", runtime);
    return command;
}

/// One wrapped commit of `a.txt` through the service at `runtime`,
/// observed.
fn commit_a(fixture: &Fixture, repo: &Path, runtime: &Path) -> super::support::Observed {
    return observe(&mut commit_a_command(fixture, repo, runtime), b"");
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
    // The commit is of one named path, so its private index is built once, from a copy of the real index: the record says
    // which way that build went, so a slow build is never again left without an answer (an entry per build, in order built).
    let one_kept_build: serde_json::Value = serde_json::from_str(r#"[{"kind":"first","statDataKept":true}]"#).expect("the one build");
    assert_eq!(record["indexBuilds"], one_kept_build, "one entry for the one build, and it kept the real index's stat data: {record}");
    assert!(number(record, "connectTries") >= 1, "{record}");
    let phases: u64 = DISJOINT.iter().map(|field| { return number(record, field); }).sum();
    // Each term is cut to whole milliseconds on its own, so the sum falls short of the total by less than one per term.
    let slack: u64 = u64::try_from(DISJOINT.len()).expect("a small count");
    assert!(phases <= number(record, "totalMs") && number(record, "totalMs") - phases < slack, "the phases and the remainder add up to the total: {record}");
    assert!(number(record, "reachMs") + number(record, "answerWaitMs") <= number(record, "submitMs"), "{record}");
    // The longest quiet gap of every Git step and tool of the command is in the record, by step name; a commit that started
    // none still states the empty object and the zero, so a reader never has to tell a missing field from a quiet command.
    assert!(record["quietMs"].is_object(), "{record}");
    assert!(record["longestQuietMs"].is_number(), "{record}");
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
    // One commit waits for nothing: nothing queued behind its landing.
    assert_eq!(ms_of(line, "queued_behind"), 0, "a lone commit has nothing behind it: {line}");
    let request_phases: u64 = ["read_ms", "queued_ms", "settle_ms", "landing_ms", "answer_ms"].iter().map(|key| { return ms_of(line, key); }).sum();
    assert!(request_phases <= ms_of(line, "total_ms"), "the phases lie inside the request's total: {line}");
    let landing_keys: [&str; 6] = ["lock_wait_ms", "reads_ms", "objects_ms", "journal_ms", "ref_ms", "index_update_ms"];
    let landing_phases: u64 = landing_keys.iter().map(|key| { return ms_of(line, key); }).sum();
    let landing_total: u64 = ms_of(line, "landing_ms");
    assert!(landing_phases <= landing_total && landing_total - landing_phases < u64::try_from(landing_keys.len()).expect("a small count"), "{line}");
    // The residual reads are named one by one, so a slow landing says which read spent the time; each is a part of `reads_ms`.
    let read_names: [&str; 9] = ["reads_config_ms", "reads_branch_ms", "reads_parents_ms", "reads_changed_ms", "reads_content_ms", "reads_witness_ms", "reads_conclusion_ms", "reads_owed_ms", "reads_follow_ms"];
    let named_reads: u64 = read_names.iter().map(|key| { return ms_of(line, key); }).sum();
    assert!(named_reads <= ms_of(line, "reads_ms"), "the named reads are parts of the residual ({} of {}): {line}", named_reads, ms_of(line, "reads_ms"));
    // Every sync of the landing is stated with what it was for: one handed to the background says `queued`, and the landing
    //  says how many it waited for and how long that took.
    assert!(ms_of(line, "syncs_queued") >= 1, "the records of the landing are flushed after the answer: {line}");
    assert!(line.contains("lock-marker=queued"), "the flush of the lock marker is named: {line}");
    assert_eq!(ms_of(line, "syncs_waited"), 0, "an unset `core.fsync` waits for no sync: {line}");
    assert_eq!(ms_of(line, "sync_held_ms"), 0, "nothing was waited for, so nothing was held: {line}");
    assert!(!line.contains("lock_holder"), "nobody held the index: {line}");
    let everything: String = std::fs::read_to_string(&places.log).expect("log") + &std::fs::read_to_string(repo.join(".git").join(TIMING_FILE)).expect("record");
    assert!(!everything.contains(MESSAGE), "the commit's text is in a log");
    assert!(!everything.contains(address), "the committer's address is in a log");
    remove_settled(&fixture);
}

/// How long the stand-in Git sleeps before it runs the commit step,
/// in seconds.
const SLOW_STEP_SECONDS: u64 = 3;

/// How long the ceiling of the scratch commit is set to in the controls that sleep in that step to watch its notices and records,
/// in seconds:
/// far above their sleeps,
/// so the measured production ceiling (16,332 ms) does not decide them.
const CONTROL_CEILING_SECONDS: &str = "600";

/// A directory with a stand-in `git` that sleeps `seconds` before any run whose words include `commit`,
/// then runs the
///  real Git;
/// every other step runs at once.
fn slow_commit_git(fixture: &Fixture, seconds: u64) -> PathBuf {
    let directory: PathBuf = fixture.root.join("slow-commit-bin");
    std::fs::create_dir_all(&directory).expect("directory of the stand-in");
    let program: PathBuf = directory.join("git");
    let source: String = format!(
        "#!/bin/sh\nslow=no\nfor word in \"$@\"; do\n  if [ \"$word\" = cat-file ]; then slow=no; break; fi\n  if [ \"$word\" = commit ]; then slow=yes; fi\ndone\nif [ $slow = yes ]; then /usr/bin/sleep {seconds}; fi\nexec /usr/bin/git \"$@\"\n"
    );
    std::fs::write(&program, source).expect("the stand-in");
    std::fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return directory;
}

/// How long the stand-in Git sleeps in the control of the ceiling,
/// in seconds:
/// far beyond the ceiling,
/// so only the stop ends the step.
const BEYOND_THE_CEILING_SECONDS: u64 = 60;

/// A commit whose message came with the command and whose scratch `git commit` outlasts the ceiling
/// (16,332 ms,
/// owner,
/// 2026-11-08,
/// "About 16 seconds")
/// is stopped,
/// says that nothing was committed and that the same command can be run again,
/// and changes neither the branch nor the person's files.
#[test]
fn a_scratch_commit_that_outlasts_the_ceiling_is_stopped_and_nothing_is_committed() {
    let fixture: Fixture = fixture("timing-ceiling-stop");
    let repo: PathBuf = base_repository(&fixture);
    git_text(&fixture, &repo, &["config", "commit.verbose", "false"]);
    let name: String = format!("timing-ceiling-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let _service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    let before: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]);
    let mut command: Command = commit_a_command(&fixture, &repo, runtime_for(name.as_str()).as_path());
    let edited: Vec<u8> = std::fs::read(repo.join("a.txt")).expect("a.txt");
    let mut path: std::ffi::OsString = slow_commit_git(&fixture, BEYOND_THE_CEILING_SECONDS).into_os_string();
    path.push(":");
    path.push(fixture.root.join("bin"));
    command.env("PATH", path);
    let began: std::time::Instant = std::time::Instant::now();
    let served = observe(&mut command, b"");
    let lasted: Duration = began.elapsed();
    let stderr: String = String::from_utf8_lossy(&served.stderr).into_owned();
    assert_eq!(served.code, Some(1), "{stderr}");
    assert!(lasted < Duration::from_secs(BEYOND_THE_CEILING_SECONDS), "the stop did not end the step: {lasted:?}");
    assert!(stderr.contains("was stopped after") && stderr.contains("nothing was committed"), "{stderr}");
    assert!(stderr.contains("Run the same `git commit` command again"), "{stderr}");
    assert_eq!(git_text(&fixture, &repo, &["rev-parse", "HEAD"]), before, "nothing was committed");
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("a.txt"), edited, "the person's file is as it was");
    remove_settled(&fixture);
}

/// The record names every Git step of the command with when it began and how long it took,
/// so a slow build is attributed to
///  the step that spent the time.
/// Why:
///  On 2026-11-07 a commit spent 241 s in its build phase and the record could not say in which step:
/// the commit Git builds in
///  the scratch repository ran with no notice and no entry of its own.
/// The stand-in Git sleeps in that step,
/// and the step's
///  duration has to be in the record,
/// with a name that carries no path,
/// message or environment.
#[test]
fn a_slow_build_step_is_named_in_the_record_with_its_duration() {
    let fixture: Fixture = fixture("timing-steps");
    let repo: PathBuf = base_repository(&fixture);
    // A configuration key the service does not embed (`commit.verbose`) makes Git build the commit, in the scratch repository.
    git_text(&fixture, &repo, &["config", "commit.verbose", "false"]);
    let name: String = format!("timing-steps-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let _service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    let mut command: Command = commit_a_command(&fixture, &repo, runtime_for(name.as_str()).as_path());
    // Real Git is searched in the usual places first, so the stand-in is found only where the PATH has no `/usr/bin`.
    let mut path: std::ffi::OsString = slow_commit_git(&fixture, SLOW_STEP_SECONDS).into_os_string();
    path.push(":");
    path.push(fixture.root.join("bin"));
    command.env("PATH", path).env(git_policy_cli::native_commit_waiting_notice::SCRATCH_CEILING_VARIABLE, CONTROL_CEILING_SECONDS);
    let served = observe(&mut command, b"");
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let found: Vec<serde_json::Value> = records(&repo);
    assert_eq!(found.len(), 1, "{found:?}");
    let record: &serde_json::Value = &found[0];
    let steps: &Vec<serde_json::Value> = record["steps"].as_array().unwrap_or_else(|| { panic!("the record has no steps list: {record}") });
    let slow: u64 = SLOW_STEP_SECONDS * 1000;
    let built: &serde_json::Value = steps.iter().find(|step| { return step["name"] == "git commit"; }).unwrap_or_else(|| { panic!("no git commit step: {record}") });
    assert!(number(built, "durationMs") >= slow, "the sleeping step's duration is in the record: {record}");
    assert!(number(built, "durationMs") <= number(record, "buildMs"), "the step lies inside the build phase: {record}");
    assert!(number(built, "offsetMs") + number(built, "durationMs") <= number(record, "totalMs"), "{record}");
    for step in steps {
        let step_name: &str = step["name"].as_str().expect("a step name");
        assert!(step_name.starts_with("git ") && !step_name.contains('/') && !step_name.contains(MESSAGE) && !step_name.contains('='), "a step name is the program and the subcommand: {step_name}");
        assert!(step["offsetMs"].is_u64() && step["durationMs"].is_u64(), "{step}");
    }
    assert_eq!(record["stepsDropped"], 0, "{record}");
    // Steps are listed in the order they began.
    let offsets: Vec<u64> = steps.iter().map(|step| { return number(step, "offsetMs"); }).collect();
    assert!(offsets.windows(2).all(|pair| { return pair[0] <= pair[1]; }), "{offsets:?}");
    // What the step's own figure cannot say is in the record beside it: whether a ceiling was armed, how long the notice's own
    // join took (a line that waits for its reader is that figure and not the step's), and which programs ran below the step.
    // The stand-in sleeps before it execs Git, so the sleep is one of them, and its time is the time Git was not running.
    let scratch: &Vec<serde_json::Value> = record["scratchSteps"].as_array().unwrap_or_else(|| { panic!("the record has no scratch step list: {record}") });
    assert_eq!(scratch.len(), 1, "one entry for the one build: {record}");
    let build: &serde_json::Value = &scratch[0];
    assert_eq!(build["armed"], true, "a message on the command line arms the ceiling: {record}");
    assert_eq!(number(build, "ceilingMs"), CONTROL_CEILING_SECONDS.parse::<u64>().expect("the control's ceiling") * 1000, "{record}");
    assert!(build["stoppedAfterMs"].is_null(), "the step ended by itself: {record}");
    assert!(number(build, "looks") >= 1 && number(build, "looksFound") >= 1, "the notice looked and found the step: {record}");
    let below: &Vec<serde_json::Value> = build["below"].as_array().unwrap_or_else(|| { panic!("no programs below the step: {record}") });
    let sleeping: &serde_json::Value = below.iter().find(|program| { return program["name"] == "sleep"; }).unwrap_or_else(|| { panic!("the stand-in's sleep is not named below the step: {record}") });
    let slept: u64 = number(sleeping, "seenMs");
    assert!(slept >= (SLOW_STEP_SECONDS - 1) * 1000, "the sleep ran {SLOW_STEP_SECONDS} s: {record}");
    assert!(slept <= SLOW_STEP_SECONDS * 1000, "the sleep cannot be seen longer than it ran: {record}");
    remove_settled(&fixture);
}

/// Whole milliseconds each term of a sum may fall short by,
/// because every phase is cut to whole milliseconds on its own.
const ROUNDING_MS: u64 = 12;

/// Whether the wait for a foreign holder accounts for the rest of the hold.
/// The hold had lasted `age_ms` when the service first looked at the lock (it measures that itself,
/// from the lock's own
///  age) and ended when the wait ended,
/// so on any host the wait is at least what was left of the hold.
fn wait_covers_the_hold(wait_ms: u64, age_ms: u64, hold_ms: u64) -> bool {
    return wait_ms + age_ms + ROUNDING_MS >= hold_ms;
}

/// Whether a phase lies beside the wait and not inside it:
/// the phase and the wait fit in the whole they are both part of.
fn phase_lies_beside_the_wait(phase_ms: u64, wait_ms: u64, whole_ms: u64) -> bool {
    return phase_ms + wait_ms <= whole_ms + ROUNDING_MS;
}

/// A host that is slow for one phase (an object write of 2.7 s) and that delays the wait's start (a long preparation) breaks
///  neither bound:
/// the wait still covers what is left of the hold,
/// and the slow phase still lies beside it.
#[test]
fn a_slow_phase_and_a_late_wait_do_not_break_the_attribution() {
    let hold: u64 = 4_000;
    let wait: u64 = 2_100;
    let age: u64 = 1_900;
    assert!(wait_covers_the_hold(wait, age, hold), "the wait began {age} ms into a hold of {hold} ms");
    assert!(phase_lies_beside_the_wait(2_681, wait, 2_681 + wait + 50), "a slow phase is not the wait");
}

/// A wait that is shorter than what was left of the hold means the time went to another phase;
/// a phase that overlaps the wait
///  is counted twice.
#[test]
fn a_wait_that_lost_time_or_a_phase_that_overlaps_it_is_caught() {
    assert!(!wait_covers_the_hold(500, 300, 4_000), "3.2 s of the hold went to no wait");
    assert!(!phase_lies_beside_the_wait(3_000, 2_000, 4_000), "3 s and 2 s do not fit in 4 s");
}

/// How many holds the wrapped commit is stalled for after the foreign commit took the lock in the control of a late commit:
/// a loaded host delays the commit for longer than a stand-in that keeps the lock for a fixed time lasts
///  (a failing run at a load average of 38 to 44 spent 6760 ms in one phase against a hold of 4000 ms).
const STALL_HOLDS: u64 = 2;

/// A foreign commit holds the index lock while the wrapped commit lands:
/// the service's line puts the time in the wait for the
///  lock and names the holder,
/// the wait lies inside the time the wrapper's record gives the answer,
/// and no other phase of
///  either comes near it.
/// The foreign commit lets go only after the wrapped commit has said it is waiting for it,
/// so the outcome does not depend on how fast the host runs the wrapped commit.
#[test]
fn a_commit_that_waits_for_a_foreign_index_holder_is_attributed_to_that_wait() {
    attributes_the_wait_for_a_foreign_index_holder("timing-lock", Duration::ZERO);
}

/// The same claim when the host stalls the wrapped commit after the foreign commit took the lock:
/// however late the wrapped commit reaches the lock,
/// the holder is still there and the wait is attributed to it.
#[test]
fn a_commit_that_reaches_a_foreign_index_holder_late_is_attributed_to_that_wait() {
    attributes_the_wait_for_a_foreign_index_holder("timing-lock-late", Duration::from_secs(STALL_HOLDS * u64::from(HOLD_SECONDS)));
}

/// Runs the claim of the two controls:
/// a wrapped commit starts `stall` after the foreign commit took the lock,
/// and the foreign
///  commit lets go `HOLD_SECONDS` after the wrapped commit's first notice that it waits for it.
fn attributes_the_wait_for_a_foreign_index_holder(control: &str, stall: Duration) {
    let fixture: Fixture = fixture(control);
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("{control}-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let _service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    let (holder, release): (Child, PathBuf) = foreign_commit_until_released(&fixture, &repo, HOLD_SECONDS);
    std::thread::sleep(stall);
    // The service's first notice that the commit waits for a holder it proved alive names the holder's process; the foreign
    //  commit lets go only after that line, so it is there for the wait however late the wrapped commit reaches the lock.
    let waiting_for_holder: String = format!("waiting for PID {}", holder.id());
    let served: Watched = watch_notices(
        &mut commit_a_command(&fixture, &repo, runtime_for(name.as_str()).as_path()),
        waiting_for_holder.as_str(),
        1,
        || { std::fs::write(&release, b"").expect("the foreign commit is told to let go"); },
    );
    let held = holder.wait_with_output().expect("the foreign commit ends");
    assert!(held.status.success(), "{}", String::from_utf8_lossy(&held.stderr));
    assert_eq!(served.code, Some(0), "{}", served.stderr);
    let answered: Vec<String> = lines_of(&places.log, "answered");
    assert_eq!(answered.len(), 1, "{answered:?}");
    let line: &str = answered[0].as_str();
    println!("SERVICE ANSWER (held index): {line}");
    let lock_wait: u64 = ms_of(line, "lock_wait_ms");
    let hold_ms: u64 = u64::from(HOLD_SECONDS) * 1_000;
    assert!(line.contains("lock_holder=\"Process "), "the holder is named: {line}");
    let found: Vec<serde_json::Value> = records(&repo);
    let record: &serde_json::Value = found.last().expect("the wrapped commit's record");
    println!("TIMING RECORD (held index): {record}");
    // The hold after the notice lasts `HOLD_SECONDS` and the wait began at or before the notice, so the wait covers the hold;
    //  the lock was already `age_ms` old at the first look, which only loosens the bound.
    let age_at_first_look: u64 = ms_of(line, "age_ms");
    assert!(wait_covers_the_hold(lock_wait, age_at_first_look, hold_ms), "the wait for the lock is what was left of the hold ({hold_ms} ms) after {age_at_first_look} ms: {line}");
    // A phase that is slow on a slow host still lies beside the wait: the two fit in the whole they are parts of.
    let landing_total: u64 = ms_of(line, "landing_ms");
    for other in ["reads_ms", "objects_ms", "journal_ms", "ref_ms", "index_update_ms"] {
        assert!(phase_lies_beside_the_wait(ms_of(line, other), lock_wait, landing_total), "{other} is not what the time went to: {line}");
    }
    for other in ["read_ms", "queued_ms", "settle_ms", "answer_ms"] {
        assert!(phase_lies_beside_the_wait(ms_of(line, other), lock_wait, ms_of(line, "total_ms")), "{other} is not what the time went to: {line}");
    }
    assert!(number(record, "answerWaitMs") >= lock_wait, "the wrapper waited at least as long as the service did for the lock: {record}");
    for other in ["prepareMs", "buildMs", "provisionMs", "afterLandingMs", "gateMs", "handoffMs", "reachMs"] {
        assert!(phase_lies_beside_the_wait(number(record, other), number(record, "answerWaitMs"), number(record, "totalMs")), "{other} is not what the time went to: {record}");
    }
    assert!(number(record, "waitingNotices") >= 1, "the service told the wrapper it was waiting: {record}");
    // The record states the landing's own figures, which the service cannot put anywhere else: the reads it names, the time it
    //  held the index, and each of its syncs with the time of each.
    let landing: &serde_json::Value = &record["landing"];
    assert!(landing.is_object(), "the answer carries the landing's figures into the record: {record}");
    assert!(landing["readsMs"].is_u64() || landing["readsMs"].is_i64() || landing["readsMs"].is_f64(), "{record}");
    assert!(landing["reads"].is_object(), "{record}");
    assert!(landing["lockHoldMs"].is_number(), "{record}");
    assert!(landing["syncs"].is_array(), "{record}");
    assert_eq!(record["lockHolderKind"], "proven-alive", "the record says what kind of holder the wait was for: {record}");
    // The wait is written down when it ends, with what it found: the foreign commit has the lock open.
    let waited: Vec<String> = lines_of(&places.log, "waited for the index lock");
    assert_eq!(waited.len(), 1, "{waited:?}");
    println!("SERVICE WAIT (held index): {}", waited[0]);
    assert!(waited[0].contains(" result=taken ") && ms_of(waited[0].as_str(), "lock_looks") >= 1, "{}", waited[0]);
    // Native `git commit` keeps its lock without a descriptor through its hooks, so the holder is proven by its PID file and
    //  the lock is described as one with no service marker.
    assert!(waited[0].contains("lock_found=\"foreign no-marker"), "the holder is described: {}", waited[0]);
    assert!(wait_covers_the_hold(ms_of(waited[0].as_str(), "lock_place_ms"), ms_of(waited[0].as_str(), "age_ms"), hold_ms), "the wait for the holder is timed apart from the flush: {}", waited[0]);
    assert!(line.contains("lock_found=") && line.contains(" lock_stage_ms="), "the answer carries it too: {line}");
    remove_settled(&fixture);
}

/// Every line the service writes states its own time at its end,
/// so a journal that receives it late does not move the event.
/// The time is the wall clock of the event:
/// the answer to the commit falls between the moments before and after the commit,
/// and every line falls between the start and the moment the log was read,
/// because work the service finishes after it answered
/// (the rules warm on the blocking pool)
/// can end after the commit returned.
#[test]
fn every_service_line_states_the_time_of_its_event() {
    let fixture: Fixture = fixture("timing-stamps");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("timing-stamps-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let before: u128 = git_policy_cli::service_timing::wall_millis();
    let _service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    let served = commit_a(&fixture, &repo, runtime_for(name.as_str()).as_path());
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let after: u128 = git_policy_cli::service_timing::wall_millis();
    let text: String = std::fs::read_to_string(&places.log).expect("service log");
    let read_at: u128 = git_policy_cli::service_timing::wall_millis();
    let lines: Vec<&str> = text.lines().filter(|line| { return line.starts_with(git_policy_cli::service_timing::LOG_PREFIX); }).collect();
    assert!(lines.len() >= 3, "started, ready, answered: {text}");
    let mut answers: usize = 0;
    for line in lines {
        let stamp: u128 = line.rsplit_once(" at_ms=").unwrap_or_else(|| { panic!("no time at the end of {line}") }).1.parse::<u128>().unwrap_or_else(|_| { panic!("no number in {line}") });
        assert!(stamp >= before && stamp <= read_at, "{stamp} is not between {before} and the read at {read_at}: {line}");
        let event: &str = line.strip_prefix(git_policy_cli::service_timing::LOG_PREFIX).unwrap_or(line);
        if event.starts_with("answered ") {
            answers += 1;
            assert!(stamp <= after, "the answer {stamp} is stamped after the commit returned at {after}: {line}");
        }
    }
    assert_eq!(answers, 1, "one answer for one commit: {text}");
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

/// The command `inner` run under `strace`,
/// which writes the calls that create,
/// link,
/// rename or remove a file to `trace`,
/// in the environment and directory of `inner`.
fn traced_for_files(inner: &Command, trace: &Path) -> Command {
    let mut outer: Command = Command::new(strace_program().expect("strace is installed"));
    outer
        .arg("--follow-forks")
        .arg("--quiet")
        .arg("--quiet")
        .arg("--string-limit=4096")
        .arg("--no-abbrev")
        .arg(format!("--output={}", trace.display()))
        .arg("--trace=openat,open,creat,link,linkat,rename,renameat,renameat2,unlink,unlinkat")
        .arg(inner.get_program())
        .args(inner.get_args());
    outer.env_clear();
    for (name, setting) in inner.get_envs() {
        if let Some(text) = setting {
            outer.env(name, text);
        }
    }
    if let Some(directory) = inner.get_current_dir() {
        outer.current_dir(directory);
    }
    return outer;
}

/// The upgrade path a real install takes:
/// a service of release N is retired by a client of release N+1,
/// and the new service
/// lands the commit.
/// The client,
/// with every program it starts,
/// never creates,
/// links,
/// renames or removes the worktree's real `index.lock`,
/// so it holds no lock of its own across the upgrade;
/// the one step that does write that lock is the refresh of a stale index's stat data,
/// and it runs before the request is made and only when the index no longer describes the worktree
/// (`commit_index_stat_staleness.rs`),
/// where this repository's index is one Git has just written;
/// and the service that lands the commit finds the lock free at its first
/// placement (no look at a held lock),
/// so the wait for the lock in an upgrade round is another program's,
/// or the disk's.
/// This is the control for the 5.6 s `lock_wait_ms` of the 0.5.13 upgrade commit.
#[test]
fn a_client_that_updates_the_service_never_takes_the_real_index_lock_and_the_new_service_finds_it_free() {
    if !strace_available() {
        eprintln!("skipped: strace is not installed, so no file call was traced");
        return;
    }
    let fixture: Fixture = fixture("timing-upgrade-lock");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("timing-upgrade-lock-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let current_log: PathBuf = fixture.root.join("current.log");
    let instance: String = instance_argument(&repo);
    let retiring: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str(), "--release", OTHER_RELEASE]);
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
    std::fs::write(repo.join("a.txt"), format!("edit by {}\n", std::process::id())).expect("a.txt");
    let mut inner: Command = wrapped_commit(&fixture, &repo, &["commit", "-m", MESSAGE, "--", "a.txt"]);
    inner.env("XDG_RUNTIME_DIR", runtime_for(name.as_str()));
    let trace: PathBuf = fixture.root.join("client.trace");
    let served = observe(&mut traced_for_files(&inner, trace.as_path()), b"");
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let (_old_status, _current): (std::process::ExitStatus, Held) = successor.join().expect("the successor thread");
    let text: String = std::fs::read_to_string(&trace).expect("the trace");
    let real_lock: String = format!("{}/.git/index.lock\"", repo.display());
    let touching: Vec<&str> = text.lines().filter(|line| { return line.contains(real_lock.as_str()); }).collect();
    // The positive control: the same trace and the same filter see a program that does take the lock.
    let control_trace: PathBuf = fixture.root.join("control.trace");
    let mut taker: Command = Command::new("/bin/sh");
    taker.arg("-c").arg("echo held > \"$0/.git/index.lock\" && rm \"$0/.git/index.lock\"").arg(&repo);
    let control = traced_for_files(&taker, control_trace.as_path()).output().expect("the control runs");
    assert!(control.status.success(), "{}", String::from_utf8_lossy(&control.stderr));
    let control_text: String = std::fs::read_to_string(&control_trace).expect("the control's trace");
    assert!(control_text.lines().any(|line| { return line.contains(real_lock.as_str()); }), "the filter sees a program that takes the lock: {control_text}");
    assert!(touching.is_empty(), "the client touched the real index lock: {touching:?}");
    let answered: Vec<String> = lines_of(&current_log, "answered");
    assert_eq!(answered.len(), 1, "the current release landed it");
    println!("SERVICE ANSWER (upgrade): {}", answered[0]);
    assert!(!answered[0].contains(" lock_looks="), "the new service found the lock held: {}", answered[0]);
    assert!(lines_of(&current_log, "waited for the index lock").len() <= 1);
    remove_settled(&fixture);
}

/// The marker files of the lock that sit in the repository's Git directory.
fn marker_files(repo: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(repo.join(".git"))
        .expect("the git directory")
        .flatten()
        .map(|entry| { return entry.file_name().to_string_lossy().into_owned(); })
        .filter(|name| { return name.starts_with("index.lock.cli-git-marker"); })
        .collect();
    names.sort();
    return names;
}

/// The first commit of a service stages its marker itself and the service then makes the next batch's while it is idle:
/// the second commit's lock is placed from that file (`lock_stage_kind=pre`),
/// one file at most waits in the Git directory between commits,
/// and none is left when the service retires.
#[test]
fn the_second_commit_of_a_service_finds_its_marker_made_ahead_and_nothing_is_left_when_it_retires() {
    let fixture: Fixture = fixture("timing-prestage");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("timing-prestage-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let mut service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    let runtime: PathBuf = runtime_for(name.as_str());
    for round in 0..3 {
        let served = commit_a(&fixture, &repo, runtime.as_path());
        assert_eq!(served.code, Some(0), "round {round}: {}", String::from_utf8_lossy(&served.stderr));
        // The marker of the next batch is made after the answer, while the service is idle.
        let began: Instant = Instant::now();
        while marker_files(&repo).len() != 1 {
            assert!(began.elapsed() < Duration::from_secs(10), "round {round}: {:?}", marker_files(&repo));
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    let answered: Vec<String> = lines_of(&places.log, "answered");
    assert_eq!(answered.len(), 3, "{answered:?}");
    println!("SERVICE ANSWERS (prestage): {answered:#?}");
    assert!(answered[0].contains(" lock_stage_kind=cold "), "the first commit has nothing made ahead: {}", answered[0]);
    assert!(answered[1].contains(" lock_stage_kind=pre ") && answered[2].contains(" lock_stage_kind=pre "), "{answered:?}");
    ask_to_retire(places.socket.as_path());
    let status = exit_of(&mut service, PATIENCE);
    assert!(status.success(), "{status:?}");
    assert!(marker_files(&repo).is_empty(), "the marker made ahead is removed when the service ends: {:?}", marker_files(&repo));
    remove_settled(&fixture);
}

/// A service with nothing to do exits after the idle period and says so,
/// with an uptime at least as long as the period.
#[test]
fn a_service_says_it_exits_idle_with_its_uptime() {
    let fixture: Fixture = fixture("timing-idle");
    let name: String = format!("timing-idle-{}", std::process::id());
    let places: Places = Places { socket: runtime_for(name.as_str()).join("monochromatic-git-policy-cli/idle.sock"), log: fixture.root.join("idle.log") };
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
    let places: Places = Places { socket: runtime_for(name.as_str()).join("monochromatic-git-policy-cli/retire.sock"), log: fixture.root.join("retire.log") };
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
    let places: Places = Places { socket: runtime_for(name.as_str()).join("monochromatic-git-policy-cli/signal.sock"), log: fixture.root.join("signal.log") };
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

/// A receiving socket on a fresh abstract name standing for the journal,
/// and the settings that make a service started with
/// its standard error in `places.log` believe the log is the journal's stream:
/// `$JOURNAL_STREAM` names the log's device and
/// inode,
/// and the test-only variable names the receiver.
fn native_journal(places: &Places, name: &str) -> (std::os::unix::net::UnixDatagram, Vec<(&'static str, String)>) {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::fs::MetadataExt;
    let abstract_name: String = format!("cli-git-test-journal-binary-{}-{name}", std::process::id());
    let address = std::os::unix::net::SocketAddr::from_abstract_name(abstract_name.as_bytes()).expect("an abstract address");
    let receiver = std::os::unix::net::UnixDatagram::bind_addr(&address).expect("bind the journal stand-in");
    receiver.set_read_timeout(Some(Duration::from_millis(200))).expect("a read timeout");
    // The service opens the log in append mode when it starts; the file exists once created here.
    std::fs::OpenOptions::new().create(true).append(true).open(&places.log).expect("the log");
    let log = std::fs::metadata(&places.log).expect("log metadata");
    let environment: Vec<(&'static str, String)> = vec![
        (git_policy_cli::journald_native::STREAM_VARIABLE, format!("{}:{}", log.dev(), log.ino())),
        (git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@{abstract_name}")),
    ];
    return (receiver, environment);
}

/// Every entry the stand-in has received so far,
/// as `NAME=value` maps,
/// waiting up to `patience` for the first and a moment
/// for the rest.
fn entries_received(receiver: &std::os::unix::net::UnixDatagram, until: impl Fn(&[std::collections::BTreeMap<String, String>]) -> bool) -> Vec<std::collections::BTreeMap<String, String>> {
    let began: Instant = Instant::now();
    let mut found: Vec<std::collections::BTreeMap<String, String>> = Vec::new();
    let mut buffer: Vec<u8> = vec![0; 65_536];
    while !until(found.as_slice()) && began.elapsed() < PATIENCE {
        if let Ok(length) = receiver.recv(buffer.as_mut_slice()) {
            let text: String = String::from_utf8_lossy(&buffer[..length]).into_owned();
            found.push(text.lines().filter_map(|line| { return line.split_once('=').map(|(name, value)| { return (name.to_string(), value.to_string()); }); }).collect());
        }
    }
    return found;
}

/// A service whose standard error is the journal sends its entries on the native socket with a priority and fields,
/// writes
/// nothing of them to standard error,
/// and the request identity in the `answered` entry is the one in the client's record,
/// so the two join on a key and not on a time.
#[test]
fn a_service_on_the_journal_stream_sends_native_entries_that_join_the_clients_record() {
    let fixture: Fixture = fixture("timing-native");
    let repo: PathBuf = base_repository(&fixture);
    let name: String = format!("timing-native-{}", std::process::id());
    let places: Places = places_for(&fixture, name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let (receiver, environment) = native_journal(&places, "send");
    let before: u128 = git_policy_cli::service_timing::wall_millis();
    let _service: Held = spawn_service_with_environment(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()], environment.as_slice());
    let served = commit_a(&fixture, &repo, runtime_for(name.as_str()).as_path());
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let after: u128 = git_policy_cli::service_timing::wall_millis();
    let entries = entries_received(&receiver, |found| { return found.iter().any(|entry| { return entry.get("MESSAGE").is_some_and(|message| { return message.starts_with("answered "); }); }); });
    let record: serde_json::Value = records(&repo).remove(0);
    let answered = entries.iter().find(|entry| { return entry.get("MESSAGE").is_some_and(|message| { return message.starts_with("answered "); }); }).expect("an answered entry");
    assert_eq!(Some(answered["CLI_GIT_REQUEST_ID"].as_str()), record["requestId"].as_str(), "the key that joins the client's record to the service's entry");
    assert!(record["requestId"].as_str().is_some_and(|id| { return !id.is_empty(); }), "{record}");
    assert_eq!(answered["PRIORITY"], "6");
    assert_eq!(answered["SYSLOG_IDENTIFIER"], "monochromatic-git-policy-cli-service");
    assert_eq!(answered["CLI_GIT_OUTCOME"], "landed");
    assert_eq!(answered["CLI_GIT_RELEASE"], RELEASE);
    assert_eq!(answered["CLI_GIT_INSTANCE"], instance.as_str());
    assert!(answered["MESSAGE"].contains(format!("request=\"{}\"", answered["CLI_GIT_REQUEST_ID"]).as_str()), "{answered:?}");
    let stamp: u128 = answered["CLI_GIT_AT_MS"].parse().expect("a number");
    assert!(stamp >= before && stamp <= after, "{stamp} is not between {before} and {after}");
    let words: Vec<&str> = entries.iter().filter_map(|entry| { return entry.get("MESSAGE").and_then(|message| { return message.split(' ').next(); }); }).collect();
    assert!(words.contains(&"started") && words.contains(&"ready"), "the life of the service is in the journal: {words:?}");
    assert_eq!(std::fs::read_to_string(&places.log).expect("log"), "", "nothing is also written to standard error");
    remove_settled(&fixture);
}

/// The signal handler dates the exit of a service that is stopped by sending it on the native socket,
/// and the line is not
/// also written to standard error.
#[test]
fn a_service_stopped_by_a_signal_sends_its_exit_on_the_native_socket() {
    let fixture: Fixture = fixture("timing-native-signal");
    let name: String = format!("timing-native-signal-{}", std::process::id());
    let places: Places = Places { socket: runtime_for(name.as_str()).join("monochromatic-git-policy-cli/native-signal.sock"), log: fixture.root.join("native-signal.log") };
    let (receiver, environment) = native_journal(&places, "signal");
    let mut service: Held = spawn_service_with_environment(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &[], environment.as_slice());
    let pid: libc::pid_t = libc::pid_t::try_from(service.0.id()).expect("a process number");
    let before: u128 = git_policy_cli::service_timing::wall_millis();
    // SAFETY: `kill` takes a process number and a signal number and touches no memory of this process.
    assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0);
    let status = exit_of(&mut service, PATIENCE);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "{status:?}");
    let entries = entries_received(&receiver, |found| { return found.iter().any(|entry| { return entry.get("MESSAGE").is_some_and(|message| { return message.starts_with("exiting "); }); }); });
    let exit = entries.iter().find(|entry| { return entry.get("MESSAGE").is_some_and(|message| { return message.starts_with("exiting "); }); }).expect("an exiting entry");
    assert!(exit["MESSAGE"].starts_with(format!("exiting reason=signal-{} ", libc::SIGTERM).as_str()), "{exit:?}");
    assert_eq!(exit["PRIORITY"], "6");
    let stamp: u128 = exit["CLI_GIT_AT_MS"].parse().expect("a number");
    assert!(stamp >= before && stamp <= git_policy_cli::service_timing::wall_millis(), "{stamp} against {before}");
    assert!(!std::fs::read_to_string(&places.log).expect("log").contains("exiting"), "the exit is in the journal once");
    remove_settled(&fixture);
}

/// A journal that is not listening costs no line:
/// the service writes it to standard error,
/// with the priority prefix the
/// journal's stream reads,
/// and it still starts and answers.
#[test]
fn a_journal_that_is_gone_leaves_the_lines_on_standard_error_with_their_priority() {
    let fixture: Fixture = fixture("timing-native-gone");
    let name: String = format!("timing-native-gone-{}", std::process::id());
    let places: Places = Places { socket: runtime_for(name.as_str()).join("monochromatic-git-policy-cli/gone.sock"), log: fixture.root.join("gone.log") };
    let (receiver, mut environment) = native_journal(&places, "gone");
    drop(receiver);
    // The abstract name is free again once the receiver is gone, so the service finds nobody to send to.
    environment.retain(|(variable, _)| { return *variable != git_policy_cli::journald_native::TEST_SOCKET_VARIABLE; });
    environment.push((git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@cli-git-test-journal-nobody-{}", std::process::id())));
    let _service: Held = spawn_service_with_environment(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &[], environment.as_slice());
    let text: String = std::fs::read_to_string(&places.log).expect("log");
    assert!(text.lines().any(|line| { return line.starts_with("<6>monochromatic-git-policy-cli-service: ready"); }), "{text}");
    assert!(text.lines().all(|line| { return line.starts_with("<6>") && line.contains(" at_ms="); }), "{text}");
    remove_settled(&fixture);
}

/// A pipe whose write end the control keeps,
/// so it can fill the pipe after the service has started.
struct FullablePipe {
    /// The end the service gets as standard error.
    service_end: std::os::fd::OwnedFd,
    /// The end the control keeps and fills;
    /// it shares the open file description with the service's end,
    /// so the flags the
    /// control leaves alone stay blocking for the service.
    control_end: std::os::fd::OwnedFd,
    /// The end nobody reads,
    /// kept open so that the pipe stays full instead of broken.
    _reader: std::os::fd::OwnedFd,
}

/// A new pipe,
/// made as two descriptors for each end.
fn fullable_pipe() -> FullablePipe {
    use std::os::fd::{FromRawFd, OwnedFd};
    let mut ends: [libc::c_int; 2] = [0; 2];
    // SAFETY: `ends` is valid for the two descriptors `pipe2` writes.
    assert_eq!(unsafe { libc::pipe2(ends.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
    // SAFETY: `pipe2` returned two new descriptors that nothing else owns.
    let (reader, writer) = unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    let control_end: OwnedFd = writer.try_clone().expect("a second descriptor of the write end");
    return FullablePipe { service_end: writer, control_end, _reader: reader };
}

impl FullablePipe {
    /// Fills the pipe with page-sized writes while a poll shows room,
    /// never changing the flags it shares with the service,
    /// so the next write of the service finds a full pipe that blocks.
    fn fill(&self) {
        use std::os::fd::AsRawFd;
        let page: [u8; 4096] = [b'x'; 4096];
        loop {
            let mut request: libc::pollfd = libc::pollfd { fd: self.control_end.as_raw_fd(), events: libc::POLLOUT, revents: 0 };
            // SAFETY: `request` is valid for the call, which reads and writes one `pollfd`.
            let ready: libc::c_int = unsafe { libc::poll(&raw mut request, 1, 0) };
            if ready <= 0 || request.revents & libc::POLLOUT == 0 {
                return;
            }
            // SAFETY: `page` is valid for its length for the call.
            assert!(unsafe { libc::write(self.control_end.as_raw_fd(), page.as_ptr().cast(), page.len()) } > 0);
        }
    }
}

/// A service stopped by `SIGTERM` while its standard error is a full blocking pipe still exits by the signal,
/// within a bound:
/// the handler may lose its line,
/// never wait for room.
fn stopped_with_a_full_pipe(name: &str, on_stream: bool) {
    use std::os::fd::AsRawFd;
    let fixture: Fixture = fixture(name);
    let unique: String = format!("{name}-{}", std::process::id());
    let places: Places = Places { socket: runtime_for(unique.as_str()).join("monochromatic-git-policy-cli/full-pipe.sock"), log: fixture.root.join("unused.log") };
    let pipe: FullablePipe = fullable_pipe();
    let mut environment: Vec<(&'static str, String)> = Vec::new();
    if on_stream {
        // SAFETY: `stat` is plain data for which all zero bytes is a valid value.
        let mut status: libc::stat = unsafe { std::mem::zeroed() };
        // SAFETY: the descriptor is open and `status` is valid for one `stat`.
        assert_eq!(unsafe { libc::fstat(pipe.control_end.as_raw_fd(), &raw mut status) }, 0);
        environment.push((git_policy_cli::journald_native::STREAM_VARIABLE, format!("{}:{}", status.st_dev, status.st_ino)));
        // Nobody listens there, so the handler's datagram fails and the line falls back to the pipe.
        environment.push((git_policy_cli::journald_native::TEST_SOCKET_VARIABLE, format!("@cli-git-test-journal-nobody-{unique}")));
    }
    let service_end: std::os::fd::OwnedFd = pipe.service_end.try_clone().expect("a descriptor for the service");
    let mut service: Held = spawn_service_with_stderr(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &[], environment.as_slice(), Stdio::from(service_end));
    pipe.fill();
    let pid: libc::pid_t = libc::pid_t::try_from(service.0.id()).expect("a process number");
    // SAFETY: `kill` takes a process number and a signal number and touches no memory of this process.
    assert_eq!(unsafe { libc::kill(pid, libc::SIGTERM) }, 0);
    let status = exit_of(&mut service, FULL_PIPE_BOUND);
    assert_eq!(status.signal(), Some(libc::SIGTERM), "{status:?}");
    remove_settled(&fixture);
}

/// How long a service gets to end after `SIGTERM` when its standard error has no room:
/// far more than it needs,
/// far less than
/// forever.
const FULL_PIPE_BOUND: Duration = Duration::from_secs(10);

/// The handler's fallback to a plain standard error never waits for room in a pipe.
#[test]
fn a_service_stopped_while_standard_error_is_a_full_pipe_still_exits() {
    stopped_with_a_full_pipe("signal-full-pipe", false);
}

/// The same holds when standard error is named as the journal's stream and the datagram cannot be sent.
#[test]
fn a_service_on_a_journal_stream_that_is_a_full_pipe_still_exits() {
    stopped_with_a_full_pipe("signal-full-stream", true);
}

/// The seconds the notice controls make the scratch Git,
/// or its signer,
/// take.
const WAIT_SECONDS: u64 = 3;

/// A signer stand-in for `gpg.program`:
/// it answers Git's status protocol and prints a signature,
/// after `WAIT_SECONDS` seconds.
fn slow_signer(fixture: &Fixture) -> PathBuf {
    let program: PathBuf = fixture.root.join("slow-signer");
    let source: String = format!(
        "#!/bin/sh\n/usr/bin/cat > /dev/null\n/usr/bin/sleep {WAIT_SECONDS}\necho '[GNUPG:] SIG_CREATED D 1 8 00 1 1 ABC' >&2\nprintf -- '-----BEGIN PGP SIGNATURE-----\\n\\nZmFrZQ==\\n-----END PGP SIGNATURE-----\\n'\n"
    );
    std::fs::write(&program, source).expect("the signer stand-in");
    std::fs::set_permissions(&program, std::os::unix::fs::PermissionsExt::from_mode(0o755)).expect("executable");
    return program;
}

/// One wrapped commit of `a.txt` in a repository whose configuration says `configuration`,
/// with a notice interval of one second,
///  extra words after `commit`,
/// and the stand-in Git (when `slow_git`) that sleeps in `commit`;
/// returns what the command wrote to standard error.
fn commit_noticing(name: &str, configuration: &[(&str, String)], extra_words: &[&str], slow_git: bool) -> String {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = base_repository(&fixture);
    // A configuration key the service does not embed makes Git build the commit, in the scratch repository.
    git_text(&fixture, &repo, &["config", "commit.verbose", "false"]);
    for (key, value) in configuration {
        git_text(&fixture, &repo, &["config", key, value.as_str()]);
    }
    let service_name: String = format!("{name}-{}", std::process::id());
    let places: Places = places_for(&fixture, service_name.as_str(), &repo);
    let instance: String = instance_argument(&repo);
    let _service: Held = spawn_service(fixture.root.as_path(), &places, SAFETY_IDLE_SECONDS, &["--instance", instance.as_str()]);
    std::fs::write(repo.join("a.txt"), format!("edit by {}\n", std::process::id())).expect("a.txt");
    let mut arguments: Vec<&str> = vec!["commit"];
    arguments.extend_from_slice(extra_words);
    arguments.extend_from_slice(&["-m", MESSAGE, "--", "a.txt"]);
    let mut command: Command = wrapped_commit(&fixture, &repo, arguments.as_slice());
    command.env("XDG_RUNTIME_DIR", runtime_for(service_name.as_str())).env("CLI_GIT_WAIT_NOTICE_SECONDS", "1").env("GIT_EDITOR", "/usr/bin/true");
    command.env(git_policy_cli::native_commit_waiting_notice::SCRATCH_CEILING_VARIABLE, CONTROL_CEILING_SECONDS);
    if slow_git {
        let mut path: std::ffi::OsString = slow_commit_git(&fixture, SLOW_STEP_SECONDS).into_os_string();
        path.push(":");
        path.push(fixture.root.join("bin"));
        command.env("PATH", path);
    }
    let served = observe(&mut command, b"");
    assert_eq!(served.code, Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let stderr: String = String::from_utf8_lossy(&served.stderr).into_owned();
    remove_settled(&fixture);
    return stderr;
}

/// The commit Git builds in the scratch repository names what the command waits for,
/// like every other wait (issue 1811).
/// Why:
///  That step runs with the terminal attached and no watch,
/// so a commit that waited minutes in it printed nothing at all;
/// the owner
///  asked that it say what it waits for.
/// It is stopped at its ceiling when the message came with the command (owner,
/// 2026-11-07,
/// "limit unless editor"):
/// the line says when.
#[test]
fn a_slow_scratch_commit_says_what_the_command_waits_for() {
    let said: String = commit_noticing("notice-scratch-commit", &[], &[], true);
    let line: &str = said.lines().find(|line| { return line.contains("`git commit`"); }).unwrap_or_else(|| { panic!("a three second wait said nothing about the commit Git builds: {said}") });
    assert!(line.starts_with("monochromatic-git-policy-cli: "), "{line}");
    assert!(line.contains("`git commit`") && line.contains("scratch repository"), "{line}");
    assert!(line.contains("stopped if it lasts"), "the line says when the step is stopped: {line}");
    assert!(!line.contains("no time limit"), "the step has a ceiling now: {line}");
    assert!(!line.contains("signs"), "nothing is signed here: {line}");
    assert!(!said.contains(MESSAGE), "the person's text is not in a notice: {said}");
}

/// With `commit.gpgsign` set the notice says the commit is signed,
/// and names the signing program when it runs.
#[test]
fn a_slow_signing_commit_says_it_waits_for_the_signature() {
    let fixture_for_signer: Fixture = fixture("notice-signer-path");
    let signer: PathBuf = slow_signer(&fixture_for_signer);
    let configuration: Vec<(&str, String)> = vec![("commit.gpgsign", String::from("true")), ("gpg.program", signer.display().to_string()), ("user.signingkey", String::from("ABC"))];
    let said: String = commit_noticing("notice-signing-commit", configuration.as_slice(), &[], false);
    let line: &str = said.lines().find(|line| { return line.contains("`git commit`"); }).unwrap_or_else(|| { panic!("a three second wait for a signer said nothing about the commit: {said}") });
    assert!(line.contains("`git commit`") && line.contains("signs the commit") && line.contains("`commit.gpgsign`"), "{line}");
    assert!(line.contains("slow-signer"), "the signing program is named while it runs: {line}");
    remove_settled(&fixture_for_signer);
}

/// A commit that opens the person's editor says nothing:
/// the person is the wait.
#[test]
fn a_commit_that_opens_an_editor_prints_no_waiting_notice() {
    let said: String = commit_noticing("notice-editor", &[], &["--edit"], true);
    assert!(!said.contains("`git commit`"), "the person at the editor is not told the command waits: {said}");
}

/// How long a control waits for a compile the service runs in the background.
const WARM_WAIT: Duration = Duration::from_secs(60);

/// How long a control watches for a warm that must not happen.
const NO_WARM_FOR: Duration = Duration::from_secs(4);

/// The rules file the default configuration names.
const WARM_RULES_FILE: &str = "forbidden-strings.local.txt";

/// Every compiled artifact under a cache directory.
fn compiled_rules_in(cache: &Path) -> usize {
    let mut found: usize = 0;
    let mut pending: Vec<PathBuf> = vec![cache.to_path_buf()];
    while let Some(next) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(next) else { continue };
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                pending.push(entry.path());
            } else if entry.file_name() == "rules.bin" {
                found += 1;
            }
        }
    }
    return found;
}

/// Waits until the cache holds a compiled artifact,
/// at most `WARM_WAIT`.
fn artifact_appears(cache: &Path) -> bool {
    let began: Instant = Instant::now();
    while began.elapsed() < WARM_WAIT {
        if compiled_rules_in(cache) > 0 {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    return false;
}

/// A repository whose `main` holds one text of the rules file and whose branch `other` holds another;
/// the worktree is on `main`,
///  and no commit has been made through the wrapper,
/// so no compiled form of either text is anywhere.
fn repository_with_two_rules_texts(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = base_repository(fixture);
    std::fs::write(repo.join(WARM_RULES_FILE), "FIRST_PLANTED_RULE_TEXT\n").expect("rules text");
    super::support::git(fixture, &repo, &["add", WARM_RULES_FILE]);
    super::support::git(fixture, &repo, &["commit", "--quiet", "--message=first rules"]);
    super::support::git(fixture, &repo, &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join(WARM_RULES_FILE), "SECOND_PLANTED_RULE_TEXT\n").expect("rules text");
    super::support::git(fixture, &repo, &["commit", "--quiet", "--all", "--message=second rules"]);
    super::support::git(fixture, &repo, &["checkout", "--quiet", "main"]);
    return repo;
}

/// A service with its own cache directory in the environment,
/// serving `repo`.
fn service_with_cache(fixture: &Fixture, repo: &Path, name: &str, cache: &Path) -> (Held, PathBuf) {
    let places: Places = places_for(fixture, name, repo);
    let instance: String = instance_argument(repo);
    let held: Held = spawn_service_with_environment(
        fixture.root.as_path(),
        &places,
        SAFETY_IDLE_SECONDS,
        &["--instance", instance.as_str()],
        &[("FORBIDDEN_STRINGS_CACHE_DIR", cache.display().to_string())],
    );
    return (held, places.socket);
}

/// A `warm` message makes the running service compile the repository's rules.
/// Why:
///  The client that sees a checkout or a merge change the rules text cannot compile it itself without making the next command wait;
///  it asks the service that already runs,
/// once,
/// and the service compiles on its own time.
#[test]
fn a_warm_message_makes_the_running_service_compile_the_rules() {
    let fixture: Fixture = fixture("warm-message");
    let repo: PathBuf = repository_with_two_rules_texts(&fixture);
    let cache: PathBuf = fixture.root.join("warm-cache");
    let name: String = format!("warm-message-{}", std::process::id());
    let (_service, socket): (Held, PathBuf) = service_with_cache(&fixture, &repo, name.as_str(), cache.as_path());
    assert_eq!(compiled_rules_in(&cache), 0);
    let git_dir: String = repo.join(".git").display().to_string();
    let line: String = format!("{{\"type\":\"warm\",\"version\":{},\"gitDir\":{git_dir:?},\"repository\":{:?}}}\n", git_policy_cli::service_protocol::PROTOCOL_VERSION, repo.display().to_string());
    let mut connection = std::os::unix::net::UnixStream::connect(&socket).expect("the service listens");
    std::io::Write::write_all(&mut connection, line.as_bytes()).expect("the message is sent");
    assert!(artifact_appears(&cache), "the service did not compile the rules after a warm message");
    remove_settled(&fixture);
}

/// A checkout that changes the rules text asks the running service to compile it;
/// one that does not,
/// asks nothing;
/// and with no
///  service running the checkout succeeds and starts none.
#[test]
fn a_checkout_that_changes_the_rules_text_asks_the_running_service_to_compile_it() {
    let fixture: Fixture = fixture("warm-checkout");
    let repo: PathBuf = repository_with_two_rules_texts(&fixture);
    let cache: PathBuf = fixture.root.join("warm-cache");
    let name: String = format!("warm-checkout-{}", std::process::id());
    let runtime: PathBuf = runtime_for(name.as_str());
    let (service, _socket): (Held, PathBuf) = service_with_cache(&fixture, &repo, name.as_str(), cache.as_path());
    let switch = |branch: &str, quiet: bool| {
        let mut command: Command = wrapped_commit(&fixture, &repo, &["checkout", "--quiet", branch]);
        command.env("XDG_RUNTIME_DIR", &runtime).env("FORBIDDEN_STRINGS_CACHE_DIR", &cache);
        let done = observe(&mut command, b"");
        assert_eq!(done.code, Some(0), "{}", String::from_utf8_lossy(&done.stderr));
        let _ = quiet;
    };
    // A switch that leaves the rules file as it is asks for nothing.
    switch("main", true);
    std::thread::sleep(NO_WARM_FOR);
    assert_eq!(compiled_rules_in(&cache), 0, "a checkout that changed no rules text made the service compile");
    // A switch that changes it asks, and the service compiles.
    switch("other", false);
    assert!(artifact_appears(&cache), "the service did not compile the rules after a checkout changed them");
    drop(service);
    // With no service running, the command still succeeds, and nothing starts one.
    std::fs::remove_dir_all(&cache).expect("remove the cache");
    switch("main", false);
    std::thread::sleep(NO_WARM_FOR);
    assert_eq!(compiled_rules_in(&cache), 0, "a service that was not running compiled the rules");
    remove_settled(&fixture);
}
