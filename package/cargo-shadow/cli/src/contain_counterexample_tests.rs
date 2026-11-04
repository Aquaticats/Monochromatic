//! What:
//! Round 7 tests built from the sixth review's own counterexamples to the containment of a bounded run (6a R1, R2, R3).
//! Why:
//! Round 5's process group and round 6's subreaper with a `/proc` scan failed one proof, that every process a command started
//! has stopped; each test here is a schedule the reviewer gave, written to fail on that design first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('containment counterexamples', () => { it('a zombie leader with a live thread', ...) })
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use rustix::process::{Pid, Signal, kill_process, test_kill_process};

use crate::bounded_run::{Bound, output_within};
use crate::crates::LockSnapshot;
use crate::journal::recover;
use crate::test_scratch::scratch;

/// The environment variable that tells the helper process which workspace to use.
const FIXTURE_ROOT: &str = "CARGO_SHADOW_CONTAIN_FIXTURE_ROOT";

/// The crate root of the workspace fixtures.
const CRATE_DIR: &str = "package/c/one";

/// How long a test waits for a file that a helper promised to make.
const FILE_WAIT: Duration = Duration::from_secs(30);

/// The C source of a process whose first thread ends while a second thread lives on.
/// The second thread says when the first thread is a zombie in `/proc` (the handshake), then lives on.
/// The comments are for a reader who knows only TypeScript.
const LEADER_SOURCE: &str = r#"
/* What: includes the thread library and the standard helpers. Why: pthread_create starts a second thread, sleep waits. */
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* The second thread: waits until /proc shows the first thread (the thread group leader) as a zombie, then creates the file named by
   the first argument, and lives on.
   In TS you'd write (pseudocode): while (!leaderIsZombie()) await sleep(1); writeFileSync(ready, ""); await sleep(600_000); */
static const char *ready_path;
static int leader_is_zombie(void) {
  char name[64];
  char text[512];
  snprintf(name, sizeof name, "/proc/%d/stat", (int) getpid());
  FILE *stat = fopen(name, "r");
  if (stat == NULL) { return 0; }
  size_t count = fread(text, 1, sizeof text - 1, stat);
  fclose(stat);
  text[count] = 0;
  char *after = strrchr(text, ')');
  return after != NULL && after[1] == ' ' && after[2] == 'Z';
}
static void *live_thread(void *argument) {
  (void) argument;
  while (!leader_is_zombie()) { usleep(1000); }
  FILE *out = fopen(ready_path, "w");
  if (out != NULL) { fputs("ready", out); fclose(out); }
  sleep(600);
  return NULL;
}

/* The first thread (the thread group leader) starts the second and then ends alone with pthread_exit.
   The process lives on in the second thread while /proc shows the leader as a zombie. */
int main(int count, char **arguments) {
  pthread_t thread;
  (void) count;
  ready_path = arguments[1];
  pthread_create(&thread, NULL, live_thread, NULL);
  pthread_exit(NULL);
}
"#;

/// A bound for the commands of these tests.
fn bound(limit: Duration) -> Bound {
    return Bound { limit, what: "the contained command".to_string(), rerun: "contained-command".to_string() };
}

/// Waits until a file exists, up to `FILE_WAIT`.
pub(crate) fn wait_for_file(path: &Path) {
    let deadline: Instant = Instant::now() + FILE_WAIT;
    while !path.exists() {
        assert!(Instant::now() < deadline, "{} did not appear", path.display());
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Reads a process id a helper wrote.
pub(crate) fn read_pid(path: &Path) -> i32 {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let Some(found) = std::fs::read_to_string(path).ok().and_then(|text| { return text.trim().parse::<i32>().ok(); }) {
            return found;
        }
        assert!(std::time::Instant::now() < deadline, "{} never held a process id", path.display());
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Whether a process with this id exists and is not a zombie.
pub(crate) fn is_running(pid: i32) -> bool {
    let Some(found) = Pid::from_raw(pid) else {
        return false;
    };
    if test_kill_process(found).is_err() {
        return false;
    }
    let stat: String = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
    let state: &str = stat.rsplit(')').next().unwrap_or("").trim_start();
    return !state.starts_with('Z');
}

/// Stops a helper process the test started and does not mind if it is gone.
pub(crate) fn stop_quietly(pid: i32) {
    if let Some(found) = Pid::from_raw(pid) {
        let _ = kill_process(found, Signal::KILL);
    }
}

/// Compiles the C source above with the system C compiler into `directory`.
pub(crate) fn compile_leader(directory: &Path) -> PathBuf {
    let source: PathBuf = directory.join("leader.c");
    let program: PathBuf = directory.join("leader");
    fs::write(&source, LEADER_SOURCE).expect("write the C source");
    let status = Command::new("cc").arg("-pthread").arg("-o").arg(&program).arg(&source).status().expect("a C compiler named cc must be on PATH: the tests that need a process whose first thread ends while another lives compile a small C fixture");
    assert!(status.success(), "the C fixture did not compile");
    return program;
}

/// Whether a process has a thread that is not a zombie (a zombie leader with a live thread has one).
pub(crate) fn has_live_thread(pid: i32) -> bool {
    let Ok(tasks) = fs::read_dir(format!("/proc/{pid}/task")) else {
        return false;
    };
    return tasks.flatten().any(|task| {
        let stat: String = fs::read_to_string(task.path().join("stat")).unwrap_or_default();
        return stat.rsplit(')').next().unwrap_or("").trim_start().chars().next().is_some_and(|state| { return state != 'Z'; });
    });
}

/// Round 7 (6a R1 A), round 8 with handshakes:
/// a thread group whose leader ended while another thread lives is not quiescent,
/// although `/proc` shows the leader as a zombie:
/// the run stops it before a normal exit returns.
/// The program waits for the fixture's own statement that its leader is a zombie, and after the run the process has no live thread.
#[test]
fn a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns() {
    if crate::cgroup_test_support::delegated_route().is_none() {
        return;
    }
    let directory = scratch();
    let leader: PathBuf = compile_leader(directory.path());
    let ready: PathBuf = directory.path().join("ready");
    let pid_file: PathBuf = directory.path().join("pid");
    let script: String = format!(
        "setsid {} {} </dev/null >/dev/null 2>&1 & echo $! > {}; while [ ! -e {} ]; do sleep 0.02; done; echo done",
        leader.display(),
        ready.display(),
        pid_file.display(),
        ready.display()
    );

    let output = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(30))).expect("runs");

    let pid: i32 = read_pid(&pid_file);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
    assert!(!has_live_thread(pid), "a thread of a process whose leader had ended is still alive after the run returned");
}

/// Round 7 (6a R1 B):
/// many short-lived parents that leave orphans while the run ends are all stopped.
/// This one passed before the change in most schedules:
/// the reviewer's schedule depends on the order in which `/proc` is read, so it is a regression test and no deterministic red.
#[test]
fn orphans_of_parents_that_exit_while_the_run_ends_are_all_stopped() {
    let directory = scratch();
    let pids: PathBuf = directory.path().join("pids");
    // Forty orphans: each double-forked child records its process id and lives on; the program waits until all forty have.
    let script: String = format!(
        "i=0; while [ $i -lt 40 ]; do ( ( echo $$ >> {pids}; exec sleep 600 ) & ) & i=$((i+1)); done; \
         while [ \"$(wc -l < {pids} 2>/dev/null || echo 0)\" -lt 40 ]; do sleep 0.02; done; echo done",
        pids = pids.display()
    );

    let output = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(30))).expect("runs");

    let recorded: Vec<i32> = fs::read_to_string(&pids).expect("the orphans' process ids").lines().filter_map(|line| { return line.trim().parse::<i32>().ok(); }).collect();
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
    assert_eq!(recorded.len(), 40);
    assert!(recorded.iter().all(|pid| { return !is_running(*pid); }), "an orphan of the run is still running after the run returned");
}

/// Round 7 (6a R3):
/// a child the caller had before the run, whose own child is adopted by the caller while the run is open,
/// is not part of the run and is not stopped.
#[test]
fn a_grandchild_adopted_during_the_run_that_the_run_did_not_start_survives() {
    let directory = scratch();
    let pid_file: PathBuf = directory.path().join("grandchild.pid");
    let script: String = format!("sleep 30 & echo $! > {}; sleep 1", pid_file.display());
    let mut earlier: Child = Command::new("sh").args(["-c", &script]).spawn().expect("a child the caller already had");
    wait_for_file(&pid_file);
    std::thread::sleep(Duration::from_millis(100));
    let grandchild: i32 = read_pid(&pid_file);

    output_within(Command::new("sleep").arg("2"), None, &bound(Duration::from_secs(20))).expect("runs");
    let alive: bool = is_running(grandchild);
    stop_quietly(grandchild);
    let _ = earlier.wait();

    assert!(alive, "a process the run did not start was stopped when its parent exited during the run");
}

/// Round 7 (6a R3):
/// an ordinary subprocess that another thread of the caller starts during the run is not part of the run.
#[test]
fn a_subprocess_another_thread_starts_during_the_run_survives() {
    let starter = std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(300));
        return Command::new("sleep").arg("30").spawn().expect("an unrelated child");
    });

    output_within(Command::new("sleep").arg("1.5"), None, &bound(Duration::from_secs(20))).expect("runs");
    let mut unrelated: Child = starter.join().expect("the starting thread");
    let alive: bool = is_running(i32::try_from(unrelated.id()).expect("a process id"));
    let _ = unrelated.kill();
    let _ = unrelated.wait();

    assert!(alive, "a process another thread started was stopped by a run it was not part of");
}

/// The helper process of the crash test below:
/// it opens a transaction and runs a command that starts a writer in a session of its own and then waits.
/// Run on its own it does nothing.
#[test]
#[ignore = "the helper process of a_killed_wrapper_with_an_escaped_writer_stops_the_recovery"]
fn wrapper_with_an_escaped_writer_fixture() {
    let Ok(root_text) = std::env::var(FIXTURE_ROOT) else {
        return;
    };
    let root: PathBuf = PathBuf::from(root_text);
    let snapshot: LockSnapshot = LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "original".to_string() };
    let _transaction = crate::journal::begin(&root, std::slice::from_ref(&snapshot), false).expect("begin");
    let directory: PathBuf = root.join("fixture");
    let script: String = format!(
        "echo $$ > {shell}; setsid sh -c 'exec </dev/null >/dev/null 2>&1; echo $$ > {writer}; : > {ready}; sleep 20; echo late > {target}' & \
         i=0; while [ ! -e {ready} ] && [ $i -lt 600 ]; do sleep 0.05; i=$((i+1)); done; exec sleep 120",
        shell = directory.join("shell.pid").display(),
        writer = directory.join("writer.pid").display(),
        ready = directory.join("ready").display(),
        target = directory.join("target").display()
    );
    let _ = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(300)));
}

/// Round 7 (6a R2, 6c R2):
/// the wrapper and the direct child die while a writer in another session lives:
/// a recovery that finds the journal must not restore the lockfile, because the writer may still change it.
#[test]
fn a_killed_wrapper_with_an_escaped_writer_stops_the_recovery() {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("crate dir");
    fs::create_dir_all(root.path().join("fixture")).expect("fixture dir");
    let lock: PathBuf = root.path().join(CRATE_DIR).join("Cargo.lock");
    fs::write(&lock, "original").expect("lock");
    let mut wrapper: Child = Command::new(std::env::current_exe().expect("this test executable"))
        .args(["--ignored", "--exact", "contain_counterexample_tests::wrapper_with_an_escaped_writer_fixture", "--nocapture"])
        .env(FIXTURE_ROOT, root.path())
        .spawn()
        .expect("the wrapper helper");
    wait_for_file(&root.path().join("fixture/ready"));
    fs::write(&lock, "unchecked output of the killed run").expect("the killed run's output");
    let shell: i32 = read_pid(&root.path().join("fixture/shell.pid"));
    let writer: i32 = read_pid(&root.path().join("fixture/writer.pid"));
    wrapper.kill().expect("kill the wrapper");
    let _ = wrapper.wait();
    stop_quietly(shell);
    std::thread::sleep(Duration::from_millis(300));

    let messages: Vec<String> = recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });
    let writer_alive: bool = is_running(writer);
    stop_quietly(writer);

    assert!(writer_alive, "the escaped writer died by itself; the test proves nothing");
    assert_eq!(fs::read_to_string(&lock).expect("lock"), "unchecked output of the killed run", "a recovery restored a file a live writer may change: {messages:?}");
}
