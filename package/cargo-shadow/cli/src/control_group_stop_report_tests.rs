//! What:
//! What the stop of a scope reports in the two cases issue 6528 could not tell apart:
//! a member that has exited and only waits to be reaped,
//! and a member that is still alive when the stop's bound passes.
//! Why:
//! The install step refused a command after printing that a process cargo started "could not be stopped",
//! and the round that met it read the process from `ps` as a zombie (issue 6528).
//! A zombie holds no descriptor and can write nothing,
//! so the stop must count it as stopped;
//! a member that is really alive past the bound must be named,
//! with the time the stop waited and the state of what it was waiting for,
//! and the run must be treated as one that cannot be settled (the README's
//! "install and update then stop with the lockfiles neither rolled back nor accepted and the journal kept").
//! Both controls here use a delegated scope and say so,
//! so neither passes on a host that cannot give one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('what the stop reports', () => { it('a member that has exited is counted as stopped', ...) })
//! ```

/// Reads the fixture's ready file and the journal's records.
use std::fs;
/// Owns the path of the compiled fixture and of the ready file.
use std::path::{
    Path,
    PathBuf,
};
/// Starts the fixture and waits for it.
use std::process::Command;
/// The bound of the contained run,
/// and the time the stop is given to show the scope empty.
use std::time::{
    Duration,
    Instant,
};

/// The bounded runner the fixture is started through,
/// and the error a run answers when its scope cannot be shown empty.
use crate::bounded_run::{
    Bound,
    QUIESCE_BOUND,
    RunError,
    output_within,
};
/// The proof that this host can give the tests a delegated scope,
/// and the reads of the scope a test looks at afterwards.
use crate::cgroup_test_support::{
    assert_scope_record,
    scope_directory_of,
    this_test_needs_a_delegated_scope,
    wait_until,
};
/// The lockfile snapshot a transaction is begun over.
use crate::crates::LockSnapshot;
/// The fence of a test build that makes a real member survive every signal the stop sends.
use crate::failpoints::{
    Point,
    arm,
};
/// The transaction that records the scope of a run in its journal.
use crate::journal::begin;
/// The journals a workspace holds.
use crate::journal_support::journal_dirs;
/// The short-lived scratch repository of one test.
use crate::test_scratch::scratch;

/// The crate root every case uses.
const CRATE_DIR: &str = "package/s/one";

/// What:
/// `ZOMBIE_SOURCE` is the C source of the fixture whose member has exited and is not reaped by its parent.
/// Why:
/// A shell reaps its background children,
/// so only a program that forks and never waits can leave a member that has exited and waits to be reaped
/// (measured 2026-11-27:
/// `sh -c 'sleep 0.2 & sleep 600'` reaped the `sleep`,
/// and a zombie of its parent's leaving is reaped by the manager).
/// The fixture's holder prints the member's process id only after `/proc` shows it as a zombie,
/// so the run's stop happens while the member really is a zombie.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ZOMBIE_SOURCE = `
///   // fork a child that exits at once, never await it, then report and sleep`
/// ```
const ZOMBIE_SOURCE: &str = r#"
/* What: includes the process calls (fork, _exit, usleep, sleep) and the file calls the ready file needs.
   Why: the fixture forks its member and its holder, and tells the test when the member is a zombie. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

/* What: reads the state letter of a process from its stat file, or 0 when it cannot.
   Why: the holder must show the member as a zombie and not merely as started. */
static char state_of(pid_t process) {
  char path[64];
  char text[512];
  snprintf(path, sizeof path, "/proc/%d/stat", (int) process);
  FILE *stat = fopen(path, "r");
  if (stat == NULL) {
    return 0;
  }
  size_t count = fread(text, 1, sizeof text - 1, stat);
  fclose(stat);
  text[count] = 0;
  char *after = strrchr(text, ')');
  if (after == NULL || after[1] != ' ') {
    return 0;
  }
  return after[2];
}

/* What: forks the member that exits at once, waits until it is a zombie, writes its id and then lives until it is killed.
   Why: this is the member the issue saw: exited, not reaped by its parent, holding nothing. */
static void hold_an_exited_member(const char *ready_path) {
  pid_t member = fork();
  if (member == 0) {
    _exit(0);
  }
  while (state_of(member) != 'Z') {
    usleep(1000);
  }
  /* What: records the group this process is in, beside the member's id.
     Why: the test compares it with the scope the journal recorded, so the member that exited is shown to be in that scope. */
  char group_partial[1024];
  snprintf(group_partial, sizeof group_partial, "%s.group.part", ready_path);
  FILE *group_in = fopen("/proc/self/cgroup", "r");
  FILE *group_out = fopen(group_partial, "w");
  if (group_in != NULL && group_out != NULL) {
    char group_text[1024];
    size_t group_count = fread(group_text, 1, sizeof group_text - 1, group_in);
    fwrite(group_text, 1, group_count, group_out);
  }
  if (group_in != NULL) {
    fclose(group_in);
  }
  if (group_out != NULL) {
    fclose(group_out);
    char group_path[1024];
    snprintf(group_path, sizeof group_path, "%s.group", ready_path);
    rename(group_partial, group_path);
  }
  char partial[1024];
  snprintf(partial, sizeof partial, "%s.part", ready_path);
  FILE *out = fopen(partial, "w");
  if (out != NULL) {
    fprintf(out, "%d\n%d\n", (int) member, (int) getpid());
    fclose(out);
    rename(partial, ready_path);
  }
  for (;;) {
    sleep(600);
  }
}

/* What: starts the holder, waits until the holder has reported its exited member, and then ends.
   Why: the stop of a scope after a normal exit is the path the issue's refusal came from,
   and the program must not end before its scope holds the member that has exited. */
int main(int count, char **arguments) {
  (void) count;
  pid_t holder = fork();
  if (holder == 0) {
    freopen("/dev/null", "r", stdin);
    freopen("/dev/null", "w", stdout);
    freopen("/dev/null", "w", stderr);
    hold_an_exited_member(arguments[1]);
    _exit(0);
  }
  while (access(arguments[1], F_OK) != 0) {
    usleep(1000);
  }
  return 0;
}
"#;

/// What:
/// `fn compile_zombie_fixture(directory: &Path) -> PathBuf` builds the fixture in `directory` and answers its path.
/// Why:
/// The fixture is a program of this test alone,
/// so two tests never share one binary,
/// and its schedule is made before the run that is judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compileZombieFixture(directory: string): string
/// ```
fn compile_zombie_fixture(directory: &Path) -> PathBuf {
    let source: PathBuf = directory.join("zombie.c");
    let program: PathBuf = directory.join("zombie");
    fs::write(&source, ZOMBIE_SOURCE).expect("write the C source");
    let status = Command::new("cc")
        .arg("-o")
        .arg(&program)
        .arg(&source)
        .status()
        .expect(
            "a C compiler named cc must be on PATH: the control of an exited member compiles a small C fixture",
        );
    assert!(status.success(), "the C fixture did not compile");
    return program;
}

/// A disposable repository whose crate root holds a lockfile.
fn repository() -> tempfile::TempDir {
    let root = scratch();
    fs::create_dir_all(root.path().join(CRATE_DIR)).expect("dir");
    fs::write(root.path().join(CRATE_DIR).join("Cargo.lock"), "original").expect("lock");
    return root;
}

/// The snapshot of the lockfile of a case.
fn snapshot() -> LockSnapshot {
    return LockSnapshot {
        crate_dir: CRATE_DIR.to_string(),
        source: "original".to_string(),
    };
}

/// A bound for the contained run of these tests.
fn bound(limit: Duration) -> Bound {
    return Bound {
        limit,
        what: "the contained command".to_string(),
        rerun: "contained-command".to_string(),
    };
}

/// Stops a scope a test left behind,
/// through its recorded path,
/// and waits until it is empty or gone.
fn clean_up(scope: &Path) {
    let _ = fs::write(scope.join("cgroup.kill"), "1");
    wait_until("the scope's end", &|| {
        return !scope.exists()
            || fs::read_to_string(scope.join("cgroup.events")).is_ok_and(|text| {
                return text.contains("populated 0");
            });
    });
}

/// The control issue 6528 asks for first:
/// a member that has exited and is not reaped by its parent is counted as stopped,
/// so the run shows quiescence,
/// its scope is recorded as settled,
/// and nothing is reported as a writer that may remain.
/// The fixture says the member was a zombie before the run's program ended,
/// so the stop met a member that had exited,
/// and the run's own answer is the subject.
#[test]
fn a_member_that_has_exited_and_waits_to_be_reaped_is_counted_as_stopped() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let program: PathBuf = compile_zombie_fixture(directory.path());
    let ready: PathBuf = directory.path().join("exited-member");
    let root = repository();
    let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path())
        .into_iter()
        .next()
        .expect("a journal");

    let result = output_within(
        Command::new(&program).arg(&ready),
        None,
        &bound(Duration::from_secs(60)),
    );

    let groups: String = journal.join("groups").display().to_string();
    assert_scope_record(&groups);
    let scope: PathBuf = scope_directory_of(&groups);
    let record: String = fs::read_to_string(&groups).expect("the record");
    let settled: bool = record.lines().any(|line| {
        return line.starts_with("settled ");
    });
    clean_up(&scope);
    assert!(
        ready.exists(),
        "the fixture never saw its member as a zombie, so the stop met no exited member"
    );
    let member: i32 = fs::read_to_string(&ready)
        .expect("the member's id")
        .lines()
        .next()
        .expect("a first line")
        .trim()
        .parse::<i32>()
        .expect("a process id");
    assert!(
        result.is_ok(),
        "a member that had exited and held nothing kept the run from showing quiescence: {result:?} \
         (member {member})"
    );
    assert!(
        !transaction.writers_may_remain(),
        "the scope reported writers that may remain although its member had exited"
    );
    assert!(
        settled,
        "the run did not record the end of its scope: {record:?}"
    );
    // The member that exited was in the very scope the journal recorded, and not merely in some group of this host.
    crate::cgroup_test_support::assert_ran_in_recorded_scope(
        &groups,
        &fs::read_to_string(directory.path().join("exited-member.group"))
            .expect("the holder's group"),
        &crate::cgroup_test_support::wrapper_group(),
    );
    drop(transaction);
}

/// The control issue 6528 asks for second,
/// at the stop's own level:
/// a member that is still alive when the stop's bound passes is named in the report,
/// with the time the stop waited and the state of what it was waiting for,
/// and the run is one whose writers may remain.
/// A test build makes the member survive every signal the stop sends
/// (`Point::ContainmentUnkillableMember`),
/// which is what a member in an uninterruptible wait does on a loaded host:
/// the stop really runs to its deadline and really finds the member there.
#[test]
fn a_member_alive_past_the_stop_bound_is_named_with_the_time_the_stop_waited() {
    this_test_needs_a_delegated_scope();
    let root = repository();
    let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path())
        .into_iter()
        .next()
        .expect("a journal");
    let ready: PathBuf = root.path().join("writer-ready");
    let script: String = format!(
        "setsid sh -c 'exec </dev/null >/dev/null 2>&1; : > {ready}; exec sleep 600' & while [ ! -e {ready} ]; do sleep 0.02; done; exit 0",
        ready = ready.display()
    );
    arm(Point::ContainmentUnkillableMember);
    let started: Instant = Instant::now();

    let result = output_within(
        Command::new("sh").args(["-c", &script]),
        None,
        &bound(Duration::from_secs(60)),
    );
    let elapsed: Duration = started.elapsed();

    let groups: String = journal.join("groups").display().to_string();
    assert_scope_record(&groups);
    let scope: PathBuf = scope_directory_of(&groups);
    let record: String = fs::read_to_string(&groups).expect("the record");
    clean_up(&scope);
    assert!(
        ready.exists(),
        "the member was not running before the stop, so the test shows nothing"
    );
    assert!(
        !crate::failpoints::fires(Point::ContainmentUnkillableMember),
        "the member survived no signal, so the stop never met it"
    );
    let error: RunError = result.expect_err("a member that is alive past the bound is not stopped");
    let said: String = error.to_string();
    assert!(
        matches!(error, RunError::NotQuiescent { .. }),
        "a member that outlives the bound was reported as another failure: {said}"
    );
    assert!(
        elapsed >= QUIESCE_BOUND,
        "the stop answered before its bound of {QUIESCE_BOUND:?}: {elapsed:?}"
    );
    assert!(
        said.contains("waited"),
        "the refusal does not say how long the stop waited: {said}"
    );
    assert!(
        said.contains("alive") || said.contains("live"),
        "the refusal does not say what it was waiting for: {said}"
    );
    assert!(
        transaction.writers_may_remain(),
        "the scope was not told that writers may remain although a member outlived the bound"
    );
    assert!(
        !record.lines().any(|line| {
            return line.starts_with("settled ");
        }),
        "the run recorded the end of its scope although a member outlived the bound: {record:?}"
    );
    drop(transaction);
}
/// The unit control of the distinction issue 6528 asks for:
/// the state of one process's threads tells a member that has exited from one that is alive,
/// and the case the stop's own comment warns about is never read as gone:
/// a leader that exited while another thread of the same process still runs can still write.
/// Each case is a real process this test started,
/// so no state is assumed from the text of a document.
#[test]
fn the_state_of_a_process_tells_an_exited_thread_from_a_live_one() {
    use crate::control_group_stop::report::{
        MemberStanding,
        standing_of,
    };
    // A process that runs: its threads are alive.
    let mut running: std::process::Child = Command::new("sleep")
        .arg("600")
        .spawn()
        .expect("starts a process that runs");
    let running_pid: i32 = i32::try_from(running.id()).expect("an id that fits");
    assert_eq!(standing_of(running_pid), MemberStanding::Alive);
    running.kill().expect("kills the process that runs");
    running.wait().expect("reaps it");

    // A process whose leader exited while another thread of it lives: it can still write.
    let directory = scratch();
    let leader: PathBuf = crate::contain_counterexample_tests::compile_leader(directory.path());
    let ready: PathBuf = directory.path().join("worker");
    let mut leader_child: std::process::Child = Command::new(&leader)
        .arg(&ready)
        .spawn()
        .expect("starts the leader fixture");
    crate::cgroup_test_support::wait_until("the leader fixture's worker", &|| {
        return ready.exists();
    });
    let leader_pid: i32 = i32::try_from(leader_child.id()).expect("an id that fits");
    let leader_state: char = fs::read_to_string(format!("/proc/{leader_pid}/stat"))
        .expect("the leader's stat")
        .rsplit(')')
        .next()
        .expect("a state")
        .trim_start()
        .chars()
        .next()
        .expect("a letter");
    assert_eq!(
        leader_state, 'Z',
        "the fixture's leader is not a zombie, so the case shows nothing"
    );
    assert_eq!(
        standing_of(leader_pid),
        MemberStanding::ThreadLives,
        "a leader that exited behind a live thread was not read as one that can still write"
    );
    leader_child.kill().expect("kills the leader fixture");
    leader_child.wait().expect("reaps it");

    // A process that has exited and is not reaped yet: it can write nothing.
    let mut ending: std::process::Child = Command::new("sleep")
        .arg("0.1")
        .spawn()
        .expect("starts a process that ends");
    let ending_pid: i32 = i32::try_from(ending.id()).expect("an id that fits");
    crate::cgroup_test_support::wait_until("the process's end without its reaping", &|| {
        return standing_of(ending_pid) == MemberStanding::Exited;
    });
    assert!(!MemberStanding::Exited.can_still_write());
    ending.wait().expect("reaps it");
    assert_eq!(
        standing_of(ending_pid),
        MemberStanding::Gone,
        "a reaped process was not read as gone"
    );
}

/// The control of the sentence a refusal carries:
/// it states how long the stop waited,
/// names every member that was left with its state,
/// and says which of them can still write.
/// The report is built from the same facts the stop hands it,
/// so the words are the stop's own reading and not a second one.
#[test]
fn the_sentence_of_a_report_states_the_wait_and_the_state_of_every_member() {
    use crate::control_group_stop::report::{
        MemberListing,
        MemberStanding,
        StopReport,
    };
    let report: StopReport = StopReport::new(
        QUIESCE_BOUND + Duration::from_millis(3),
        false,
        Some(true),
        vec![
            MemberListing {
                pid: 4242,
                standing: MemberStanding::Alive,
            },
            MemberListing {
                pid: 4243,
                standing: MemberStanding::Exited,
            },
        ],
        None,
    );

    let said: String = report.sentence(Path::new("/sys/fs/cgroup/a.scope"));

    assert!(
        said.contains(&format!("{:?}", QUIESCE_BOUND + Duration::from_millis(3))),
        "the sentence does not state the time the stop waited: {said}"
    );
    assert!(
        said.contains("4242") && said.contains("it is alive, so it can still write"),
        "the sentence does not name the live member with its state: {said}"
    );
    assert!(
        said.contains("4243")
            && said.contains("it has exited and only waits to be reaped, so it can write nothing"),
        "the sentence does not tell the exited member from the live one: {said}"
    );
    assert!(
        said.contains("/sys/fs/cgroup/a.scope"),
        "the sentence does not name the scope: {said}"
    );
    assert!(
        report.may_still_write(),
        "a report with a live member was read as one that cannot still write"
    );
    let unknown: MemberStanding = MemberStanding::Unknown("the read failed".to_string());
    assert!(unknown.can_still_write());
    assert!(
        unknown.text().contains("may still write"),
        "an unreadable member was read as one that cannot write: {}",
        unknown.text()
    );
}
