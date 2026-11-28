//! What:
//! What the stop of a scope reports in the cases issue 6528 could not tell apart:
//! a member that has exited and only waits to be reaped,
//! a member that is still alive when the stop's bound passes,
//! a member that stands in a group below the scope,
//! a listing that could not be read,
//! a scope whose directory was gone,
//! and a listing that was cut at its own bound.
//! Why:
//! The install step refused a command after printing that a process cargo started "could not be stopped",
//! and the round that met it read the process from `ps` as a zombie (issue 6528).
//! A zombie holds no descriptor and can write nothing,
//! so the stop must count it as stopped;
//! a member that is really alive past the bound must be named,
//! with the time the stop waited and the state of what it was waiting for,
//! and the run must be treated as one that cannot be settled (the README's
//! "install and update then stop with the lockfiles neither rolled back nor accepted and the journal kept").
//! The kernel's `populated` word counts the groups below the scope,
//! so a scope whose own `cgroup.procs` is empty can still hold a live process
//! (`doc/troubleshooting/a-cgroup-does-not-count-a-member-that-has-exited.md`),
//! and the refusal must name that process rather than say nothing it listed can write.
//! Every control that runs a scope uses a delegated one and says so,
//! so none passes on a host that cannot give one.
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
/// Starts the fixture and waits for it,
/// and holds a child that a control wants to leave as one that has exited and waits to be reaped.
use std::process::{
    Child,
    Command,
};
/// The bound of the contained run,
/// the time the stop is given to show the scope empty,
/// and how long reading its members may take.
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
/// the reads of the scope a test looks at afterwards,
/// and how long a clean-up waits for a group to empty.
use crate::cgroup_test_support::{
    STATE_WAIT,
    assert_scope_record,
    scope_directory_of,
    this_test_needs_a_delegated_scope,
    wait_until,
    wait_up_to,
};
/// The report's own words,
/// and the listing it is built from.
use crate::control_group_stop::report::{
    Listing,
    MemberListing,
    MemberStanding,
    StopReport,
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
/// `NESTED_SOURCE` is the C source of the fixture whose only live member stands in a group below the scope.
/// Why:
/// The kernel's `populated` word counts a group below the scope,
/// so this fixture makes the case the refusal used to misword:
/// the scope's own `cgroup.procs` is empty and `populated` is 1
/// (measured 2026-11-27 in `doc/troubleshooting/a-cgroup-does-not-count-a-member-that-has-exited.md`).
/// The program forks a member that makes `nested` below the scope,
/// moves itself into it,
/// and only then reports itself;
/// the program ends,
/// so the member in `nested` is all that is left and the direct child was reaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const NESTED_SOURCE = `
///   // fork a member, mkdir <scope>/nested, move the member in, report it, then end`
/// ```
const NESTED_SOURCE: &str = r#"
/* What: includes the calls that read /proc, make a directory, fork and sleep.
   Why: the fixture reads its own scope, makes a group below it and moves a member into it. */
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

/* What: writes the path of this process's scope below /sys/fs/cgroup into `out`, or an empty text when it cannot.
   Why: a group below the scope is made by its path, and only the version 2 line of /proc/self/cgroup names it. */
static void scope_path(char *out, size_t size) {
  FILE *in = fopen("/proc/self/cgroup", "r");
  char line[1024];
  out[0] = 0;
  if (in == NULL) {
    return;
  }
  while (fgets(line, sizeof line, in) != NULL) {
    if (strncmp(line, "0::", 3) == 0) {
      int written = snprintf(out, size, "/sys/fs/cgroup%s", line + 3);
      if (written > 0 && (size_t) written < size) {
        char *end = strchr(out, '\n');
        if (end != NULL) {
          *end = 0;
        }
      }
      break;
    }
  }
  fclose(in);
}

/* What: moves this process into the group whose cgroup.procs is `procs`, by writing its own id there.
   Why: a process moves itself into a group below its own when the group is delegated to its user, which is what makes the
   scope's own list empty while the kernel still counts a member. */
static int move_self_into(const char *procs) {
  FILE *out = fopen(procs, "w");
  if (out == NULL) {
    return -1;
  }
  int written = fprintf(out, "%d\n", (int) getpid());
  fclose(out);
  return written > 0 ? 0 : -1;
}

/* What: makes `nested` below the scope, moves into it and reports itself there, then lives until it is killed.
   Why: this is the member a refusal must name as one that stands below the scope and can still write. */
static void stand_below_the_scope(const char *ready_path) {
  char scope[1024];
  scope_path(scope, sizeof scope);
  if (scope[0] == 0) {
    _exit(2);
  }
  char nested[1200];
  snprintf(nested, sizeof nested, "%s/nested", scope);
  if (mkdir(nested, 0755) != 0 && errno != EEXIST) {
    _exit(3);
  }
  char procs[1300];
  snprintf(procs, sizeof procs, "%s/cgroup.procs", nested);
  if (move_self_into(procs) != 0) {
    _exit(4);
  }
  char partial[1200];
  snprintf(partial, sizeof partial, "%s.part", ready_path);
  FILE *out = fopen(partial, "w");
  if (out != NULL) {
    fprintf(out, "%d\n%s\n", (int) getpid(), nested);
    fclose(out);
    rename(partial, ready_path);
  }
  for (;;) {
    sleep(600);
  }
}

/* What: starts the member below the scope, waits until it has reported itself, and then ends.
   Why: the program must end so the run's stop begins, and the member it left is the only live one.
   The member's parent is this program, so it is re-parented when this program ends and keeps running. */
int main(int count, char **arguments) {
  (void) count;
  pid_t member = fork();
  if (member == 0) {
    freopen("/dev/null", "r", stdin);
    freopen("/dev/null", "w", stdout);
    freopen("/dev/null", "w", stderr);
    stand_below_the_scope(arguments[1]);
    _exit(0);
  }
  while (access(arguments[1], F_OK) != 0) {
    usleep(1000);
  }
  return 0;
}
"#;

/// What:
/// `fn compile_fixture(directory: &Path, name: &str, source: &str) -> PathBuf` builds one fixture in `directory` and answers
/// its path.
/// Why:
/// Each fixture is a program of the test that needs it,
/// so two tests never share one binary,
/// and its schedule is made before the run that is judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function compileFixture(directory: string, name: string, source: string): string
/// ```
fn compile_fixture(
    directory: &Path,
    name: &str,
    source: &str,
) -> PathBuf {
    let source_path: PathBuf = directory.join(format!("{name}.c"));
    let program: PathBuf = directory.join(name);
    fs::write(&source_path, source).expect("write the C source");
    let status = Command::new("cc")
        .arg("-o")
        .arg(&program)
        .arg(&source_path)
        .status()
        .expect(
            "a C compiler named cc must be on PATH: the controls of a report compile small C fixtures",
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

/// Asks the manager to stop the scope unit a test's run made,
/// through the package's own retirement,
/// so no unit of this test stays loaded once its members are gone.
fn retire(scope: &Path) {
    let route = crate::cgroup_test_support::delegated_route();
    let unit: String = scope
        .file_name()
        .map(|name| return name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let _ = crate::control_group::retire::retire_unit(
        &route.systemctl,
        &unit,
        crate::control_group::retire::RETIRE_BOUND,
    );
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
    retire(scope);
}

/// What:
/// `fn empty_the_nested_group(scope: &Path, nested: &Path, bound: Duration)` writes `cgroup.kill` on the scope,
/// waits up to `bound` for the group below it to list no process,
/// and fails with what that write answered and the state of every process the group still lists.
/// Why:
/// The write is the one step of the clean-up that can fail without a word,
/// and a member still listed after it is told from one the kill never reached by its own state:
/// a member that has exited or gone was slow to leave,
/// and one alive in any other state is a kill that did not take it (issue 6650).
/// The failure used to say only that the group's end did not happen in time,
/// which told the two apart for nobody.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function emptyTheNestedGroup(scope: string, nested: string, bound: number): void  // throws with the write's answer and each member
/// ```
fn empty_the_nested_group(
    scope: &Path,
    nested: &Path,
    bound: Duration,
) {
    // What:     `killed` is what the kernel answered the write that kills the scope's members.
    // Why:      A write the kernel refused and a write it took leave the same group behind, and only this record tells them apart.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let killed: Error | undefined
    // try { writeFileSync(path.join(scope, 'cgroup.kill'), '1') } catch (error) { killed = error }
    // ```
    let killed: std::io::Result<()> = fs::write(scope.join("cgroup.kill"), "1");
    // What:     `emptied` says whether the group listed no process before the bound passed.
    // Why:      A wait that answers instead of asserting is what lets the failure be built from what is still there.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const emptied = waitUpTo(bound, () => readFileSync(path.join(nested, 'cgroup.procs'), 'utf8').trim() === '')
    // ```
    let emptied: bool = wait_up_to(bound, &|| {
        return fs::read_to_string(nested.join("cgroup.procs")).is_ok_and(|text| {
            return text.trim().is_empty();
        });
    });
    assert!(
        emptied,
        "{}",
        a_nested_group_that_did_not_empty(scope, nested, bound, &killed)
    );
}

/// What:
/// `fn a_nested_group_that_did_not_empty(scope: &Path, nested: &Path, bound: Duration, killed: &std::io::Result<()>) -> String` is
/// the sentence a clean-up fails with when the group below the scope still lists a process at its bound:
/// what the `cgroup.kill` write answered,
/// every process the group lists with its state,
/// and how to read the two cases apart.
/// Why:
/// The control of issue 6650 failed at its twenty-second bound in four of the ten whole-suite runs on record,
/// and its message named neither the write's answer nor the members left,
/// so a reader could not tell a kill that never arrived from a member that was slow to leave
/// (a same-vendor second read of `0.1.5-alpha.18`, 2026-11-28).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aNestedGroupThatDidNotEmpty(scope: string, nested: string, bound: number, killed: unknown): string
/// ```
fn a_nested_group_that_did_not_empty(
    scope: &Path,
    nested: &Path,
    bound: Duration,
    killed: &std::io::Result<()>,
) -> String {
    // What:     `written` says whether the kernel took the kill or refused it, with the refusal's own words.
    // Why:      A refused write is the first case a reader has to be able to rule out.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const written = killed === undefined ? 'the kernel took it' : `it was refused (${killed})`
    // ```
    let written: String = match killed {
        Ok(()) => format!(
            "the kernel took the write of {}/cgroup.kill",
            scope.display()
        ),
        Err(error) => format!(
            "the write of {}/cgroup.kill was refused ({error})",
            scope.display()
        ),
    };
    // What:     `listed` is every process the group below the scope still lists, each with its state, or the words that say why none
    //           could be read.
    // Why:      The state is the whole difference between a member that was leaving and one the kill never reached.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const listed = whatAGroupStillLists(nested)
    // ```
    let listed: String = what_a_group_still_lists(nested);
    return format!(
        "the nested group {} still listed a process at the bound of {bound:?}: {written}, and every process it lists: {listed}. A member that has exited or is no longer there was slow to leave; one alive in any other state is a member the kill did not take",
        nested.display()
    );
}

/// What:
/// `fn what_a_group_still_lists(group: &Path) -> String` reads a group's `cgroup.procs` and names every process it lists with the state
/// of that process.
/// Why:
/// A count would not say whether the members were leaving or were never killed,
/// and the ids alone would send a reader to `/proc` by hand while the state is one read away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatAGroupStillLists(group: string): string
/// ```
fn what_a_group_still_lists(group: &Path) -> String {
    // What:     `listed` is the group's cgroup.procs as text, one process id per line.
    // Why:      That file is the kernel's own list of the processes the group holds.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let listed: string
    // try { listed = readFileSync(path.join(group, 'cgroup.procs'), 'utf8') } catch (error) { return `nothing could be read (${error})` }
    // ```
    let listed: String = match fs::read_to_string(group.join("cgroup.procs")) {
        Ok(text) => text,
        Err(error) => {
            return format!(
                "nothing, because its cgroup.procs could not be read ({error})"
            );
        }
    };
    // Every process the message names, one entry each.
    let mut named: Vec<String> = Vec::new();
    for pid in listed.split_whitespace() {
        named.push(what_a_process_shows(pid));
    }
    if named.is_empty() {
        return "no process at all, so the bound passed on a group that was already empty".to_string();
    }
    return named.join("; ");
}

/// What:
/// `fn what_a_process_shows(pid: &str) -> String` is one process the group listed, read from `/proc/<pid>/stat`:
/// whether it is alive,
/// has exited and waits to be reaped,
/// or is no longer there at all.
/// Why:
/// Those three are what tell a kill that never arrived from a member that was slow to leave,
/// and the state letter of the stat file is the kernel's own word for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whatAProcessShows(pid: string): string
/// ```
fn what_a_process_shows(pid: &str) -> String {
    // What:     `stat` is that process's stat file, whose second field is its command name in brackets and whose third is its state
    //           letter.
    // Why:      A read that fails is the kernel saying no process has that id any more, which is the slow-to-leave case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let stat: string
    // try { stat = readFileSync(`/proc/${pid}/stat`, 'utf8') } catch (error) { return `${pid} is no longer there (${error})` }
    // ```
    let stat: String = match fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(text) => text.trim().to_string(),
        Err(error) => {
            return format!(
                "{pid} has no process any more (/proc/{pid}/stat could not be read: {error}), so the group's listing has not caught up with a member that left"
            );
        }
    };
    // What:     `after_name` is that file from its closing bracket on, whose first word is the state letter.
    // Why:      A command name may hold a bracket and spaces (`(a) b`), so the split is taken at the last closing bracket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const afterName = stat.slice(stat.lastIndexOf(')') + 1)
    // ```
    let after_name: &str = match stat.rsplit_once(')') {
        Some((_, rest)) => rest,
        None => {
            return format!(
                "{pid} is listed and its stat file has no command name: {stat:?}"
            );
        }
    };
    // What:     `state` is the letter the kernel gives that process: `R` running, `S` sleeping, `D` in an uninterruptible sleep, `T`
    //           stopped, `Z` exited and waiting to be reaped, `I` idle.
    // Why:      The message reads the state, so the letter is the fact it is built from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const state = afterName.trim().split(/\s+/)[0]
    // ```
    let state: &str = after_name.split_whitespace().next().unwrap_or("");
    // What:     `reading` is what that letter means for a reader of the failure.
    // Why:      A zombie can write nothing and is only waiting for its parent, where any other state is a process still able to write.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reading = state === 'Z' ? 'has exited and waits to be reaped, so it can write nothing' : 'is alive, so the kill did not take it'
    // ```
    let reading: &str = match state {
        "Z" => "has exited and waits to be reaped, so it can write nothing",
        "X" => "is dead",
        "R" | "S" | "D" | "I" | "T" | "t" => "is alive, so the kill did not take it",
        _ => "is in a state this test does not know",
    };
    return format!("{pid} {reading} (state {state:?} of /proc/{pid}/stat: {stat})");
}

/// Stops a scope a test left behind whose `nested` group this test made,
/// removes that group,
/// stops the scope's unit,
/// and waits until the scope itself is gone.
/// Why the group is removed here:
/// the manager removes a scope's directory only when nothing is below it,
/// so a group this test made would keep the scope of the host behind;
/// and a scope whose directory the test removed first stays an active unit,
/// so the unit is stopped after the group is gone,
/// when there is nothing left for its own removal to fail on.
fn clean_up_a_nested_scope(
    scope: &Path,
    nested: &Path,
) {
    empty_the_nested_group(scope, nested, STATE_WAIT);
    // The group is empty now, so the directory below the scope can be removed; a removal that is still refused is retried.
    let removed: bool = wait_up_to(Duration::from_secs(20), &|| {
        return !nested.exists() || fs::remove_dir(nested).is_ok();
    });
    assert!(
        removed,
        "the nested group {} could not be removed",
        nested.display()
    );
    // The manager removes a scope it could not remove while the group was below it only when it is asked to end the unit, so the
    // package's own retirement is asked last.
    retire(scope);
    let gone: bool = wait_up_to(Duration::from_secs(20), &|| {
        return !scope.exists();
    });
    assert!(gone, "the scope {} could not be removed", scope.display());
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
    let program: PathBuf = compile_fixture(directory.path(), "zombie", ZOMBIE_SOURCE);
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
        said.contains("it is alive, so it can still write"),
        "the refusal does not name the live member with its state: {said}"
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

/// A live writer in the scope's own group,
/// started by a payload that ends once the writer is ready,
/// so the stop that follows is the one this test judges.
fn leave_a_writer(ready: &Path) -> String {
    return format!(
        "setsid sh -c 'exec </dev/null >/dev/null 2>&1; : > {ready}; exec sleep 600' & while [ ! -e {ready} ]; do sleep 0.02; done; exit 0",
        ready = ready.display()
    );
}

/// Issue 6528's review,
/// Medium 1:
/// the scope's own `cgroup.procs` is empty while the kernel still reports the scope populated,
/// because its only live member stands in a group below it.
/// The refusal must name that member as one that may still write,
/// and must say where it stands,
/// because `populated` counts the groups below the scope and the lockfiles hang on what can still write.
#[test]
fn a_member_in_a_group_below_the_scope_is_named_where_it_stands_and_as_one_that_may_still_write() {
    this_test_needs_a_delegated_scope();
    let directory = scratch();
    let program: PathBuf = compile_fixture(directory.path(), "nested", NESTED_SOURCE);
    let ready: PathBuf = directory.path().join("nested-member");
    let root = repository();
    let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path())
        .into_iter()
        .next()
        .expect("a journal");

    // The stop reports at once that a member could not be stopped, so the report is made while the member below the scope
    // really stands there (`report_now`), and nothing was killed that could have emptied the group first.
    arm(Point::ContainmentStopStuck);
    let result = output_within(
        Command::new(&program).arg(&ready),
        None,
        &bound(Duration::from_secs(60)),
    );

    let groups: String = journal.join("groups").display().to_string();
    assert_scope_record(&groups);
    let scope: PathBuf = scope_directory_of(&groups);
    let own: String = fs::read_to_string(scope.join("cgroup.procs")).unwrap_or_default();
    let nested: PathBuf = scope.join("nested");
    let readiness: String = fs::read_to_string(&ready).unwrap_or_default();
    let member: i32 = readiness
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .parse::<i32>()
        .unwrap_or(0);
    let member_group: String =
        fs::read_to_string(format!("/proc/{member}/cgroup")).unwrap_or_default();
    let populated: String = fs::read_to_string(scope.join("cgroup.events")).unwrap_or_default();
    clean_up_a_nested_scope(&scope, &nested);

    // The case really is the one under test: the scope's own group is empty, the kernel still reports it populated, and the
    // member really stands in the group below the scope.
    assert!(
        member > 0 && readiness.lines().nth(1) == Some(nested.display().to_string().as_str()),
        "the fixture did not report a member below the scope, so the test shows nothing: {readiness:?}"
    );
    assert!(
        own.trim().is_empty(),
        "the scope's own group still lists a member, so the case is not the one under test: {own:?}"
    );
    assert!(
        populated.contains("populated 1"),
        "the kernel did not report the scope populated, so the test shows nothing: {populated:?}"
    );
    assert!(
        crate::cgroup_test_support::v2_path_of_cgroup_file(&member_group).ends_with("/nested"),
        "the member does not stand in the group below the scope: {member_group:?}"
    );
    let error: RunError = result.expect_err("a member that may still write is not a stopped run");
    let said: String = error.to_string();
    assert!(
        matches!(error, RunError::NotQuiescent { .. }),
        "a member below the scope was reported as another failure: {said}"
    );
    assert!(
        said.contains(&member.to_string()),
        "the refusal does not name the member below the scope ({member}): {said}"
    );
    assert!(
        said.contains("in nested"),
        "the refusal does not say that the member stands in a group below the scope: {said}"
    );
    assert!(
        said.contains("it is alive, so it can still write"),
        "the refusal does not say that the member may still write: {said}"
    );
    assert!(
        said.contains("a member of the scope could not be shown stopped"),
        "the refusal does not open by naming a member it could not show stopped: {said}"
    );
    assert!(
        !said.contains("not a process of this scope's own group"),
        "the refusal still says the member is not a process of the scope's own group instead of naming it: {said}"
    );
    drop(transaction);
}

/// Issue 6528's review,
/// Medium 1:
/// the listing of the members failed,
/// so the refusal must say that and must not say what its members can do.
/// A listing that could not be read is not a list of members that cannot write.
#[test]
fn a_refusal_whose_members_could_not_be_read_says_nothing_about_what_they_can_do() {
    this_test_needs_a_delegated_scope();
    let root = repository();
    let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path())
        .into_iter()
        .next()
        .expect("a journal");
    let ready: PathBuf = root.path().join("writer-ready");
    // The stop reports at once, so the listing is the report's own and the failure lands there.
    arm(Point::ContainmentStopStuck);
    arm(Point::ContainmentInspection);

    let result = output_within(
        Command::new("sh").args(["-c", &leave_a_writer(&ready)]),
        None,
        &bound(Duration::from_secs(60)),
    );

    let groups: String = journal.join("groups").display().to_string();
    assert_scope_record(&groups);
    let scope: PathBuf = scope_directory_of(&groups);
    let listed: String = fs::read_to_string(scope.join("cgroup.procs")).unwrap_or_default();
    clean_up(&scope);
    assert!(
        ready.exists() && !listed.trim().is_empty(),
        "no writer was left in the scope, so the test shows nothing"
    );
    let error: RunError = result.expect_err("a run whose members could not be read is not stopped");
    let said: String = error.to_string();
    assert!(
        said.contains("the scope's members could not be read"),
        "the refusal does not say that the members could not be read: {said}"
    );
    // What:     `.matches("members could not be read")` counts the times the message says the members could not be read.
    // Why:      The opening clause and the sentence are one message, so the fact is said once (issue 6528's second review, Low 1).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const saidAgain = said.split('members could not be read').length - 1;
    // ```
    let said_again: usize = said.matches("members could not be read").count();
    assert_eq!(
        said_again, 1,
        "the refusal says the members could not be read {said_again} times: {said}"
    );
    assert!(
        !said.contains("can still write") && !said.contains("can write nothing"),
        "the refusal says what the members it could not read can do: {said}"
    );
    drop(transaction);
}

/// Issue 6528's review,
/// Medium 1,
/// last case:
/// the scope vanished between the last `populated` read and the listing,
/// so the refusal must say its directory was gone and must not say anything about members it never read.
/// The window is too short to produce from outside
/// (a removal before that read makes the stop report the scope empty at once),
/// so a test build makes the listing find the scope gone,
/// and the test judges the words the refusal carries;
/// the member is a real one that still stands in the scope,
/// so the report is made of real facts.
#[test]
fn a_refusal_of_a_scope_that_was_gone_when_its_members_were_read_says_so() {
    this_test_needs_a_delegated_scope();
    let root = repository();
    let transaction = begin(root.path(), &[snapshot()], false).expect("begin");
    let journal: PathBuf = journal_dirs(root.path())
        .into_iter()
        .next()
        .expect("a journal");
    let ready: PathBuf = root.path().join("writer-ready");
    arm(Point::ContainmentStopStuck);
    arm(Point::ContainmentScopeGoneAtListing);

    let result = output_within(
        Command::new("sh").args(["-c", &leave_a_writer(&ready)]),
        None,
        &bound(Duration::from_secs(60)),
    );

    let groups: String = journal.join("groups").display().to_string();
    assert_scope_record(&groups);
    let scope: PathBuf = scope_directory_of(&groups);
    clean_up(&scope);
    assert!(
        ready.exists(),
        "no writer was left in the scope, so the test shows nothing"
    );
    assert!(
        !crate::failpoints::fires(Point::ContainmentScopeGoneAtListing),
        "the listing never ran, so the test shows nothing"
    );
    let error: RunError = result.expect_err("a run whose scope vanished is not stopped");
    let said: String = error.to_string();
    assert!(
        said.contains("directory was gone when its members were read"),
        "the refusal does not say the scope was gone when its members were read: {said}"
    );
    // What:     `.matches("directory was gone")` counts the times the message says the directory was gone.
    // Why:      The opening clause and the sentence are one message a person reads, so the fact is said once (issue 6528's
    //           second review, Low 1).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const said = said.matches('directory was gone').count();
    // ```
    let directory_gone: usize = said.matches("directory was gone").count();
    assert_eq!(
        directory_gone, 1,
        "the refusal says the directory was gone {directory_gone} times: {said}"
    );
    assert!(
        !said.contains("can still write") && !said.contains("can write nothing"),
        "the refusal says what the members it never read can do: {said}"
    );
    // The listing checks that the directory is absent and not that this process still sees the parent the scope was
    // planned in, so the clause may not claim that no member is left to stop.
    assert!(
        said.contains("a directory that is gone also matches a scope that was never made"),
        "the refusal does not say that the absence it read was not verified: {said}"
    );
    assert!(
        !said.contains("so no member of it is left to stop"),
        "the refusal claims no member is left on an absence it did not verify: {said}"
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
    use crate::control_group_stop::LISTING_BOUND;
    use crate::control_group_stop::report::standing_of_within;
    let within_the_listing_bound = || {
        return Instant::now() + LISTING_BOUND;
    };
    // A process that runs: its threads are alive.
    let mut running: std::process::Child = Command::new("sleep")
        .arg("600")
        .spawn()
        .expect("starts a process that runs");
    let running_pid: i32 = i32::try_from(running.id()).expect("an id that fits");
    assert_eq!(
        standing_of_within(running_pid, within_the_listing_bound()),
        MemberStanding::Alive
    );
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
        standing_of_within(leader_pid, within_the_listing_bound()),
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
        return standing_of_within(ending_pid, within_the_listing_bound())
            == MemberStanding::Exited;
    });
    assert!(!MemberStanding::Exited.can_still_write());
    ending.wait().expect("reaps it");
    assert_eq!(
        standing_of_within(ending_pid, within_the_listing_bound()),
        MemberStanding::Gone,
        "a reaped process was not read as gone"
    );

    // A number that names no process this one can see, which is how `cgroup.procs` writes a process of another pid namespace,
    // is never read as gone: it is not proven to hold nothing.
    let unseeable: MemberStanding = standing_of_within(0, within_the_listing_bound());
    assert!(
        unseeable.can_still_write() && unseeable.text().contains("another pid namespace"),
        "a listed number that names no visible process was read as gone: {}",
        unseeable.text()
    );
}

/// A report of a scope that was not shown empty,
/// built from the facts a case names.
fn report_of(
    populated: Option<bool>,
    listing: Listing,
) -> StopReport {
    return StopReport::new(QUIESCE_BOUND, false, populated, listing);
}

/// A member in the scope's own group.
fn at_home(
    pid: i32,
    standing: MemberStanding,
) -> MemberListing {
    return MemberListing {
        pid,
        below: None,
        standing,
    };
}

/// A member in a group below the scope,
/// by that group's path below the scope.
fn below(
    pid: i32,
    group: &str,
    standing: MemberStanding,
) -> MemberListing {
    return MemberListing {
        pid,
        below: Some(group.to_string()),
        standing,
    };
}

/// The control of the sentence a refusal carries:
/// it states how long the stop waited,
/// names every member that was left with its state and the group it stands in,
/// and says which of them can still write.
/// The report is built from the same facts the stop hands it,
/// so the words are the stop's own reading and not a second one.
#[test]
fn the_sentence_of_a_report_states_the_wait_and_the_state_of_every_member() {
    let report: StopReport = report_of(
        Some(true),
        Listing::Read(vec![
            at_home(4242, MemberStanding::Alive),
            at_home(4243, MemberStanding::Exited),
            below(4244, "nested", MemberStanding::Alive),
        ]),
    );

    let said: String = report.sentence(Path::new("/sys/fs/cgroup/a.scope"));

    assert!(
        said.contains(&format!("{:?}", QUIESCE_BOUND)),
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
        said.contains("4244 in nested"),
        "the sentence does not say which group below the scope the member stands in: {said}"
    );
    assert!(
        said.contains("/sys/fs/cgroup/a.scope"),
        "the sentence does not name the scope: {said}"
    );
    assert!(
        matches!(
            report.leftover(),
            crate::control_group_stop::report::Leftover::Writer
        ),
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

/// Issue 6528's review,
/// Low 5:
/// a listing whose members all ended says so,
/// and the refusal does not claim a member that can still write.
#[test]
fn a_refusal_whose_members_all_ended_says_no_member_it_listed_can_still_write() {
    let report: StopReport = report_of(
        Some(true),
        Listing::Read(vec![
            at_home(5001, MemberStanding::Gone),
            at_home(5002, MemberStanding::Exited),
        ]),
    );
    let said: String = report.sentence(Path::new("/sys/fs/cgroup/b.scope"));

    assert_eq!(
        report.headline(),
        "every group of the scope was read, and no member it listed can still write"
    );
    assert!(
        said.contains("it listed 2 members: 5001 (it has ended and been reaped, so it can write nothing), 5002"),
        "the sentence does not name both members that ended: {said}"
    );
}

/// Issue 6528's review,
/// Medium 1:
/// a listing that read every group and found no member says that,
/// and the refusal does not say what such a member can do.
#[test]
fn a_refusal_that_listed_no_member_says_so() {
    let report: StopReport = report_of(Some(true), Listing::Read(Vec::new()));
    let said: String = report.sentence(Path::new("/sys/fs/cgroup/c.scope"));

    assert_eq!(
        report.headline(),
        "every group of the scope was read, and none of them listed a member"
    );
    // What:     `said` is the sentence that follows that opening clause, and it may not say the same thing again.
    // Why:      The two clauses are one message, so the listing that found no member is said once (issue 6528's second review, Low 1).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const saysItAgain = said.includes('listed no member');
    // ```
    let says_it_again: bool = said.contains("listed no member");
    assert!(
        !says_it_again,
        "the sentence says again what the opening clause says, that no member was listed: {said}"
    );
    assert!(
        said.contains("the kernel last reported it populated"),
        "the sentence does not say what the kernel last reported: {said}"
    );
    assert!(
        !said.contains("can still write") && !said.contains("can write nothing"),
        "the sentence says what a member it did not read can do: {said}"
    );
}

/// Issue 6528's review,
/// Low 4:
/// a listing that was cut at its own bound says which bound it hit and names what it read before that,
/// so a member it never reached is not counted as one that cannot write.
#[test]
fn a_refusal_whose_listing_was_cut_states_the_bound_it_hit() {
    use crate::control_group_stop::LISTING_BOUND;
    let cut: StopReport = report_of(Some(true), Listing::Cut(Vec::new()));
    let cut_said: String = cut.sentence(Path::new("/sys/fs/cgroup/d.scope"));
    assert_eq!(
        cut.headline(),
        format!("the listing of the scope's members stopped at its own bound of {LISTING_BOUND:?}")
    );
    assert!(
        cut_said.contains("no member was read"),
        "the sentence does not say the cut kept the listing from reading one member: {cut_said}"
    );

    // A listing cut after it read one member that had ended names it and still says the rest were not read.
    let partly: StopReport = report_of(
        Some(true),
        Listing::Cut(vec![at_home(6001, MemberStanding::Gone)]),
    );
    let partly_said: String = partly.sentence(Path::new("/sys/fs/cgroup/e.scope"));
    assert!(
        partly_said.contains("it listed 1 member: 6001"),
        "the sentence does not name the member it read before the cut: {partly_said}"
    );
}

/// Issue 6528's review,
/// Low 3:
/// a scope whose absence was verified is worded as absent,
/// never as a kernel that reported it empty,
/// and no wait is stated for it.
#[test]
fn a_scope_proved_absent_is_worded_as_gone_and_not_as_the_kernel_speaking() {
    let report: StopReport = StopReport::absent(Duration::from_micros(12));
    let said: String = report.sentence(Path::new("/sys/fs/cgroup/f.scope"));

    assert!(
        said.contains("was gone when the stop looked, and its absence was verified"),
        "the sentence does not say the scope's directory was gone: {said}"
    );
    assert!(
        !said.contains("kernel") && !said.contains("waited"),
        "the sentence claims a word or a wait the kernel never gave: {said}"
    );
    assert!(report.empty(), "a report of an absent scope is not empty");
}

/// The control of the sentence a clean-up fails with when a group below the scope did not empty (issue 6650):
/// it names what the `cgroup.kill` write answered,
/// and the state of every process the group still lists,
/// so "the kill never arrived" and "the member was slow to leave" are told apart.
/// Why it is a control and not a comment:
/// the control of issue 6650 failed at its twenty-second bound in four of the ten whole-suite runs on record,
/// and its message named neither the write's answer nor the members left
/// (a same-vendor second read of `0.1.5-alpha.18`, 2026-11-28).
/// The group here is a plain directory holding a `cgroup.procs` file,
/// because the read under control is a read of that file and of `/proc`,
/// and the bound is fifty milliseconds because the subject is the message and not the wait.
/// The group holds a directory where the kernel's `cgroup.kill` file would be,
/// so the write is refused and the refusal's own words are part of the message.
/// The three members are the three kinds a reader has to tell apart:
/// this test's own process, which is alive;
/// a child that has exited and is not reaped, so it is a zombie;
/// and an id no process of this machine can have, since the kernel hands out no such id.
#[test]
fn a_nested_group_that_did_not_empty_names_the_kill_s_answer_and_each_member_left() {
    // The fixture directory of this control.
    let directory = scratch();
    // What:     `group` is the directory that stands for the group below a scope: it holds a `cgroup.procs` file and no `cgroup.kill`.
    // Why:      A plain directory is what makes the kill's write fail, and the failure's own words are part of the message.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const group = path.join(dir, 'a-group-that-did-not-empty')
    // ```
    let group: PathBuf = directory.path().join("a-group-that-did-not-empty");
    fs::create_dir(&group).expect("make the group");
    // What:     `directory` at `cgroup.kill`'s own name is what makes the write a kernel would take here be refused.
    // Why:      A refusal is the first case the message has to name, and its own words are what a reader needs.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // mkdirSync(path.join(group, 'cgroup.kill'))
    // ```
    fs::create_dir(group.join("cgroup.kill")).expect("make a directory where the kernel's kill file would be");
    // What:     `live` is a member that is alive: this test's own process, whose state the message must read as able to write.
    // Why:      A kill that did not take a member is the case the message exists for.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const live = process.pid
    // ```
    let live: u32 = std::process::id();
    // What:     `exited` is a child that ends at once and is not reaped until this control has read the message, so it is a zombie
    //           of this test while the message is built, and nothing is left behind afterwards.
    // Why:      A member that has exited and waits to be reaped is the "slow to leave" case the old message could not name,
    //           and only a parent that has not waited yet can show it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const exited = spawn('true')   // and only wait for it at the end
    // ```
    let mut exited: Child = Command::new("true")
        .spawn()
        .expect("start a member that exits at once");
    // The id the kernel gave that child.
    let exited_id: u32 = exited.id();
    // What:     `gone` is an id no process of this machine can have, because the kernel hands out no id that wide.
    // Why:      A group's listing can lag behind a member that has left, and the message must say so rather than print an id alone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const gone = 0xFFFFFFFF
    // ```
    let gone: u32 = u32::MAX;
    // What:     `format!` is the listing the kernel would write: one process id per line.
    // Why:      The read under control takes the file as the kernel's own list, so the fixture writes exactly that shape.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // writeFileSync(path.join(group, 'cgroup.procs'), `${live}\n${exitedId}\n${gone}\n`)
    // ```
    let listing: String = format!("{live}\n{exited_id}\n{gone}\n");
    fs::write(group.join("cgroup.procs"), listing).expect("write the group's listing");
    // What:     `showed` says whether the child has really become a zombie before the message is built.
    // Why:      A child that is still running would be read as alive, and the control would then say nothing about a zombie.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const showed = waitUpTo(10_000, () => readFileSync(`/proc/${exitedId}/stat`, 'utf8').includes(') Z '))
    // ```
    let showed: bool = wait_up_to(Duration::from_secs(10), &|| {
        return fs::read_to_string(format!("/proc/{exited_id}/stat"))
            .is_ok_and(|stat| return stat.contains(") Z "));
    });
    assert!(
        showed,
        "the member that exited never showed as a zombie, so this control would say nothing about one"
    );
    // What:     `outcome` is the clean-up's failure, caught so the message it leaves can be read.
    // Why:      The message is the subject, and a test that failed on it would not print it to a reader.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let outcome: unknown
    // try { emptyTheNestedGroup(group, group, 50) } catch (error) { outcome = error }
    // ```
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        empty_the_nested_group(&group, &group, Duration::from_millis(50));
    }));
    // What:     `failed` is that panic's payload, which a group that still lists a live process must leave.
    // Why:      A clean-up that returned here would have waited out a listing it could read, which the fixture never writes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const failed = outcome ?? panic('the clean-up did not fail on a group that lists a live process')
    // ```
    let failed = outcome.expect_err("the clean-up did not fail on a group that lists a live process");
    // What:     `said` is the message that payload carries.
    // Why:      Every assertion below is about the words a reader of a failing run would see.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const said = String(failed)
    // ```
    let said: String = match failed.downcast_ref::<String>() {
        Some(text) => text.to_string(),
        None => panic!("the failure is not a message this control can read"),
    };
    assert!(
        said.contains("was refused"),
        "the message does not say what the write of cgroup.kill answered: {said}"
    );
    assert!(
        said.contains("cgroup.kill"),
        "the message does not name the write that was refused: {said}"
    );
    assert!(
        said.contains(&format!("{live} is alive, so the kill did not take it")),
        "the message does not name the live member as one the kill did not take: {said}"
    );
    assert!(
        said.contains(&format!("{exited_id} has exited and waits to be reaped")),
        "the message does not name the member that exited as one that was leaving: {said}"
    );
    assert!(
        said.contains(&format!("{gone} has no process any more")),
        "the message does not say that an id the group still lists has no process behind it: {said}"
    );
    assert!(
        said.contains(&group.display().to_string()),
        "the message does not name the group that did not empty: {said}"
    );
    println!("the clean-up failed with: {said}");
    // What:     `wait()` reaps the member that had exited, so the zombie this control read is gone when it ends.
    // Why:      A zombie is the state under control, not something a test may leave behind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await exited.wait()
    // ```
    exited.wait().expect("reap the member that had exited");
}
