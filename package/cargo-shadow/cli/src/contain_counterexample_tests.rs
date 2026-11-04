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

use rustix::process::{Pid, Signal, kill_process};

use crate::bounded_run::{Bound, output_within};
use crate::cgroup_test_support::{assert_ran_in_recorded_scope, wrapper_group};
use crate::crates::LockSnapshot;
use crate::journal::{begin, recover};
use crate::journal_support::journal_dirs;
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
#include <sys/syscall.h>
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
  /* The thread id is written whole under another name and renamed, so a reader that sees the file sees a complete id (9b 3.7). */
  char partial[1024];
  snprintf(partial, sizeof partial, "%s.part", ready_path);
  FILE *out = fopen(partial, "w");
  if (out != NULL) {
    fprintf(out, "%ld\n", (long) syscall(SYS_gettid));
    fclose(out);
    rename(partial, ready_path);
  }
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

/// Whether a process with this id is running (neither ended nor a zombie that waits to be reaped).
/// The inspection fails closed (certification, second pass): it reads through the pinned procfs and a pidfd, and one that cannot tell
/// (a procfs that numbers processes as another namespace does, an unreadable entry) stops the test instead of reading as "gone".
pub(crate) fn is_running(pid: i32) -> bool {
    return crate::control_group::process_is_running(pid).unwrap_or_else(|reason| { panic!("whether process {pid} runs cannot be told: {reason}"); });
}

/// Asserts that the wrapper (this process) can no longer wait for the child with this id: a checked `waitpid` gives `ECHILD`, which
/// is what a successful reap leaves behind.
/// Why:
/// Killing a child, or seeing it exit, does not show that it was reaped: a zombie still answers a wait (certification group 1).
pub(crate) fn assert_reaped(pid: i32, what: &str) {
    use rustix::process::{WaitOptions, waitpid};
    let result = waitpid(Pid::from_raw(pid), WaitOptions::NOHANG);
    assert!(matches!(result, Err(rustix::io::Errno::CHILD)), "{what}: the direct child {pid} can still be waited for, so it was not reaped: {result:?}");
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

/// Whether the thread with this id is alive (not a zombie, not gone).
/// Only a thread that is gone (`ENOENT`) or a zombie is "not alive"; anything the helper cannot read or parse stops the test,
/// because an unknown state is not a stopped one (8b 2.2).
pub(crate) fn thread_is_alive(tid: i32) -> bool {
    // A procfs that numbers threads as another namespace does would read a live thread as absent: the test environment is refused.
    if let Err(reason) = crate::control_group::procfs_matches_namespace() {
        panic!("the thread oracle cannot be trusted here: {reason}");
    }
    let stat: String = match fs::read_to_string(format!("/proc/{tid}/stat")) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(3) => return false,
        Err(error) => panic!("/proc/{tid}/stat cannot be read ({error}): the state of the thread is unknown, not stopped"),
    };
    let state: char = stat.rsplit(')').next().and_then(|rest| { return rest.trim_start().chars().next(); }).unwrap_or_else(|| { panic!("/proc/{tid}/stat cannot be parsed: {stat:?}") });
    return state != 'Z' && state != 'X';
}

/// The thread id the fixture's worker wrote into its ready file, once the file holds it.
pub(crate) fn worker_tid(ready: &Path) -> i32 {
    return read_pid(ready);
}

/// The control of the oracle (8b 2.2):
/// the fixture run on its own, with no run around it, shows the worker alive behind its zombie leader,
/// and the same helper reports it gone once the process is killed.
#[test]
fn the_thread_oracle_sees_a_live_worker_behind_a_zombie_leader_and_its_end() {
    let directory = scratch();
    let leader: PathBuf = compile_leader(directory.path());
    let ready: PathBuf = directory.path().join("ready");
    let mut process = Command::new(&leader).arg(&ready).spawn().expect("starts the fixture");
    let tid: i32 = worker_tid(&ready);
    let leader_state: char = fs::read_to_string(format!("/proc/{}/stat", process.id())).expect("the leader's stat").rsplit(')').next().expect("a state").trim_start().chars().next().expect("a letter");

    let alive_before: bool = thread_is_alive(tid);
    process.kill().expect("kill the fixture");
    let _ = process.wait();
    let alive_after: bool = thread_is_alive(tid);

    assert_eq!(leader_state, 'Z', "the fixture's leader is not a zombie, so the control shows nothing");
    assert!(alive_before, "the oracle did not see the live worker");
    assert!(!alive_after, "the oracle did not see the worker end");
}

/// Round 7 (6a R1 A), round 8 with handshakes:
/// a thread group whose leader ended while another thread lives is not quiescent,
/// although `/proc` shows the leader as a zombie:
/// the run stops it before a normal exit returns.
/// The program waits for the fixture's own statement that its leader is a zombie, and after the run the process has no live thread.
#[test]
fn a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns() {
    zombie_leader_run(false);
}

/// Certification group 1 (A1): the same fixture driven through the timeout boundary, with the same readiness handshake, worker oracle
/// and recorded-scope assertion: the program waits after the fixture's leader is a zombie, the bound passes, and the run is stopped
/// with the live worker behind the zombie.
#[test]
fn a_zombie_leader_with_a_live_writer_thread_is_stopped_at_the_bound() {
    zombie_leader_run(true);
}

/// The run of the zombie-leader fixture: to a normal exit (`at_the_bound` false) or to the bound (true).
fn zombie_leader_run(at_the_bound: bool) {
    if crate::cgroup_test_support::delegated_route().is_none() {
        return;
    }
    let directory = scratch();
    let leader: PathBuf = compile_leader(directory.path());
    let ready: PathBuf = directory.path().join("ready");
    let pid_file: PathBuf = directory.path().join("pid");
    let group_file: PathBuf = directory.path().join("group");
    let direct_file: PathBuf = directory.path().join("direct");
    let workspace = recorded_workspace();
    let transaction = begin(workspace.path(), &[LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "lock".to_string() }], false).expect("begin");
    let journal: PathBuf = journal_dirs(workspace.path()).into_iter().next().expect("a journal");
    let wrapper: String = wrapper_group();
    let finish: &str = if at_the_bound { "sleep 600" } else { "echo done" };
    let script: String = format!(
        "echo $$ > {direct}; cat /proc/self/cgroup > {group}; setsid {leader} {ready} </dev/null >/dev/null 2>&1 & echo $! > {pid}; while [ ! -e {ready} ]; do sleep 0.02; done; {finish}",
        direct = direct_file.display(),
        group = group_file.display(),
        leader = leader.display(),
        ready = ready.display(),
        pid = pid_file.display(),
    );
    let limit: Duration = if at_the_bound { Duration::from_secs(8) } else { Duration::from_secs(30) };

    let result = output_within(Command::new("sh").args(["-c", &script]), None, &bound(limit));

    let tid: i32 = worker_tid(&ready);
    if at_the_bound {
        assert!(matches!(result, Err(crate::bounded_run::RunError::TimedOut { .. })), "the program was not stopped at the bound: {result:?}");
    } else {
        assert_eq!(String::from_utf8_lossy(&result.expect("runs").stdout).trim(), "done");
    }
    assert!(!thread_is_alive(tid), "the worker thread of a process whose leader had ended is still alive after the run returned (at the bound: {at_the_bound})");
    let _ = read_pid(&pid_file);
    assert_ran_in_recorded_scope(&journal.join("groups").display().to_string(), &fs::read_to_string(&group_file).expect("the program's group"), &wrapper);
    assert_reaped(read_pid(&direct_file), if at_the_bound { "at the bound" } else { "after a normal exit" });
    drop(transaction);
}

/// A disposable workspace with the crate root the journal of a test snapshots.
fn recorded_workspace() -> tempfile::TempDir {
    let workspace = scratch();
    fs::create_dir_all(workspace.path().join(CRATE_DIR)).expect("dir");
    fs::write(workspace.path().join(CRATE_DIR).join("Cargo.lock"), "lock").expect("lock");
    return workspace;
}

/// Round 7 (6a R1 B):
/// many short-lived parents that leave orphans while the run ends are all stopped.
/// This one passed before the change in most schedules:
/// the reviewer's schedule depends on the order in which `/proc` is read, so it is a regression test and no deterministic red.
#[test]
fn orphans_of_parents_that_exit_while_the_run_ends_are_all_stopped() {
    if crate::cgroup_test_support::delegated_route().is_none() {
        return;
    }
    let directory = scratch();
    let pids: PathBuf = directory.path().join("pids");
    let group_file: PathBuf = directory.path().join("group");
    let workspace = recorded_workspace();
    let transaction = begin(workspace.path(), &[LockSnapshot { crate_dir: CRATE_DIR.to_string(), source: "lock".to_string() }], false).expect("begin");
    let journal: PathBuf = journal_dirs(workspace.path()).into_iter().next().expect("a journal");
    let wrapper: String = wrapper_group();
    // Forty orphans: each double-forked child records its process id and lives on; the program waits until all forty have.
    let script: String = format!(
        "cat /proc/self/cgroup > {group}; i=0; while [ $i -lt 40 ]; do ( ( sh -c 'echo $$ >> {pids}; exec sleep 600' ) & ) & i=$((i+1)); done; \
         while [ \"$(wc -l < {pids} 2>/dev/null || echo 0)\" -lt 40 ]; do sleep 0.02; done; echo done",
        pids = pids.display(),
        group = group_file.display()
    );

    let output = output_within(Command::new("sh").args(["-c", &script]), None, &bound(Duration::from_secs(30))).expect("runs");

    let recorded: Vec<i32> = fs::read_to_string(&pids).expect("the orphans' process ids").lines().filter_map(|line| { return line.trim().parse::<i32>().ok(); }).collect();
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "done");
    let distinct: std::collections::BTreeSet<i32> = recorded.iter().copied().collect();
    assert_eq!(recorded.len(), 40);
    assert_eq!(distinct.len(), 40, "the orphans did not record their own process ids: {recorded:?}");
    let alive: Vec<(i32, String)> = recorded.iter().filter(|pid| { return is_running(**pid); }).map(|pid| { return (*pid, fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default()); }).collect();
    assert!(alive.is_empty(), "an orphan of the run is still running after the run returned: {alive:?}");
    assert_ran_in_recorded_scope(&journal.join("groups").display().to_string(), &fs::read_to_string(&group_file).expect("the program's group"), &wrapper);
    drop(transaction);
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
    // The recorded launcher (the shell, which is the program) is established gone before the recovery, by the launcher question and
    // not by an elapsed sleep (certification group 1): an unreaped zombie launcher would itself be alive and block, so the refusal
    // below is attributable to the scope's member only once the launcher is classified gone.
    let journal_for_launcher: PathBuf = crate::journal_support::journal_dirs(root.path()).into_iter().next().expect("a journal");
    let launcher_record: String = fs::read_to_string(journal_for_launcher.join("groups")).expect("the records");
    let Some(Ok(crate::control_group::Record::Launcher { pid: launcher_pid, start: launcher_start, pid_namespace, path: recorded_scope })) =
        launcher_record.lines().map(crate::control_group::parse_record).find(|parsed| { return matches!(parsed, Ok(crate::control_group::Record::Launcher { .. })); })
    else {
        panic!("no launcher record: {launcher_record:?}");
    };
    assert_eq!(launcher_pid, shell, "the recorded launcher is not the shell that ran the program");
    crate::cgroup_test_support::wait_until("the recorded launcher to be classified gone", &|| { return crate::control_group::launcher_state(launcher_pid, launcher_start, pid_namespace) == crate::control_group::LauncherState::Gone; });

    let messages: Vec<String> = recover(root.path(), &|_before, _live| { return vec!["dep@1.0.1 refused".to_string()]; });
    let writer_alive: bool = is_running(writer);
    stop_quietly(writer);
    // The scope is stopped as a whole, so the writer's own children do not outlive the test.
    if let Some(journal) = crate::journal_support::journal_dirs(root.path()).first()
        && let Ok(text) = fs::read_to_string(journal.join("groups"))
        && let Some(Ok(crate::control_group::Record::Scope { path, .. })) = text.lines().next().map(crate::control_group::parse_record)
    {
        let _ = fs::write(path.join("cgroup.kill"), "1");
    }

    assert!(writer_alive, "the escaped writer died by itself; the test proves nothing");
    assert_eq!(fs::read_to_string(&lock).expect("lock"), "unchecked output of the killed run", "a recovery restored a file a live writer may change: {messages:?}");
    // The refusal is for the intended reason, the journal is retained, and the run used a delegated scope.
    assert!(
        messages.iter().any(|message| { return message.contains("a process in the scope") && message.contains(&recorded_scope.display().to_string()) && message.contains("still running"); }),
        "the recovery did not refuse because a member of the recorded scope lives: {messages:?}"
    );
    let journals: Vec<PathBuf> = crate::journal_support::journal_dirs(root.path());
    assert_eq!(journals.len(), 1, "the journal was not retained: {journals:?}");
    crate::cgroup_test_support::assert_scope_record(&journals[0].join("groups").display().to_string());
}
