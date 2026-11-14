//! What:
//!  The lock files a killed wrapper command leaves behind,
//!  and what the next command does with each:
//!  cleared when the wrapper's own record proves the holder gone,
//!  kept when it does not,
//!  and never waited out to a ceiling that must not be reached.
//! Why:
//!  On this machine a process can be killed at any instruction.
//!  An audit of 0.6.1-alpha.14 ended a merge's process group while its checkout waited on a stalled download,
//!  and `index.lock` stood with a record naming a process that no longer existed:
//!  the next commit waited sixty seconds and was withdrawn.
//!  A lock whose holder is proven gone protects nothing,
//!  so the next command clears it and goes on;
//!  a lock whose holder is not proven gone keeps the bounded wait it always had,
//!  and the words say what is known.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! plantDeadHolderLock(repo, 'index.lock'); expect(runWrapped(repo, ['add', 'a.txt']).code).toBe(0);
//! ```

use super::support::{Fixture, Observed, executable, fixture, git, hermetic, own_group, porcelain, repository, run_wrapped, spawn_owned, stderr_of, wrapper_first_path};
use std::os::unix::process::CommandExt as _;
use git_policy_cli::lock_holder_records::{folder_of, identity_of_process, record_text};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// How long a command that must go on is given,
/// so a regression to the sixty second wait fails the control instead of passing slowly.
const GOES_ON_WITHIN: Duration = Duration::from_secs(20);

/// How long a control waits for a fixture's Git to reach the point where it holds a lock.
const HOLD_PATIENCE: Duration = Duration::from_secs(60);

/// The lock files a killed command can leave,
/// each with the PID file Git writes beside it under `core.lockfilePid`.
const EVERY_LOCK: [&str; 6] = ["index.lock", "HEAD.lock", "shallow.lock", "packed-refs.lock", "refs/heads/topic.lock", "reftable/tables.list.lock"];

/// What:
///  Start `program` with empty standard input and output,
///  so a helper that writes nothing never holds the test's own pipes.
/// Why:
///  A helper outliving the control is ended by `Helper`'s drop,
///  and a helper that writes into a pipe nobody reads would wait instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// spawn(program, { stdio: 'ignore' })
/// ```
fn quiet(program: &str) -> Command {
    let mut command: Command = Command::new(program);
    command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    return command;
}

/// What:
///  The identity of a process that ran and is now gone,
///  read while it still ran.
/// Why:
///  A record naming such an identity is exactly what a killed command leaves:
///  the birth is one this boot had,
/// and no process has it now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = deadHolderIdentity(); // 'linux:1234#<boot>#<scope>#<machine>'
/// ```
fn a_dead_holder() -> (i64, String) {
    // A program that waits for its own end, so its identity is readable until this control ends it: a helper that is
    //  already gone (`git --version`) made two of the unit controls fail for the machine's speed in a review of
    //  `d189162e3e`, and nothing here should depend on that.
    let mut child: Child = quiet("sleep").arg("600").spawn().expect("a helper starts");
    let pid: i64 = i64::from(child.id());
    let identity: String = identity_of_process(pid).expect("the helper's identity is readable while it waits");
    let _killed = child.kill();
    child.wait().expect("the helper ends");
    return (pid, identity);
}

/// What:
///  The PID file beside a lock,
/// as Git writes it.
/// Why:
///  The proof begins with the number Git recorded in it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pidFileOf('/repo/.git/index.lock') // '/repo/.git/index~pid.lock'
/// ```
fn pid_file_of(lock: &Path) -> PathBuf {
    let name: String = lock.file_name().expect("a lock has a name").to_string_lossy().into_owned();
    return lock.with_file_name(format!("{}~pid.lock", name.strip_suffix(".lock").expect("a lock ends in .lock")));
}

/// What:
///  Write the record a killed command would have left:
///  `holders` in one file of the repository's record folder.
/// Why:
///  A control plants the state a kill leaves,
///  so it can name a holder that is gone,
/// alive,
/// or a number now used by another process,
///  which no real kill can be made to produce on demand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeRecord(repo, [['child', pid, identity]]);
/// ```
fn plant_record(repo: &Path, holders: &[(&str, i64, String)], name: &str) {
    let folder: PathBuf = folder_of(repo.join(".git").as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the record folder");
    std::fs::write(folder.join(name), record_text(holders)).expect("the record");
}

/// What:
/// What:
///  Write the PID file beside `lock`,
/// again if need be,
///  until its birth is strictly after the lock's.
/// Why:
///  A control that means to see a removal must give the gate a proven order;
///  this volume gives two files written microseconds apart one birth tick most of the time,
///  so the file is made again after a pause until the two stamps differ,
///  and a file system that records no birth time fails the control naming what it said.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeAfterTheLock(lock, 'pid 42\n')
/// ```
fn write_after_the_lock(lock: &Path, content: &str) {
    let pid_file: PathBuf = pid_file_of(lock);
    let began: Instant = Instant::now();
    loop {
        let _ = std::fs::remove_file(pid_file.as_path());
        std::thread::sleep(Duration::from_millis(2));
        std::fs::write(pid_file.as_path(), content.as_bytes()).expect("the PID file");
        let lock_born = std::fs::symlink_metadata(lock).ok().and_then(|metadata| { return metadata.created().ok(); });
        let pid_born = std::fs::symlink_metadata(pid_file.as_path()).ok().and_then(|metadata| { return metadata.created().ok(); });
        if let (Some(lock_at), Some(pid_at)) = (lock_born, pid_born)
            && pid_at > lock_at
        {
            return;
        }
        assert!(
            began.elapsed() < Duration::from_secs(5),
            "this file system cannot show that {} was born after {}: the lock is {:?} and the PID file {:?}",
            pid_file.display(),
            lock.display(),
            lock_born,
            pid_born
        );
    }
}

///  Plant one lock file,
/// the PID file Git writes beside it,
///  and the record a killed command leaves.
/// Why:
///  One helper plants every lock this control judges,
///  so each lock is judged by the same state and the same proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantLock(repo, 'index.lock', { pid, identity });
/// ```
fn plant_lock(repo: &Path, name: &str, pid: i64, identity: Option<&String>, record_name: &str) {
    let lock: PathBuf = repo.join(".git").join(name);
    std::fs::create_dir_all(lock.parent().expect("the lock's folder")).expect("the lock's folder");
    // A Git writes the new index, the reference list or HEAD into the lock before it renames it, so a lock left by a kill
    //  holds bytes rather than nothing.
    std::fs::write(lock.as_path(), b"a half-written copy the dead holder never renamed\n").expect("the lock");
    // The gate names a holder only when the PID file is born strictly after the lock, which this volume gives two files
    //  written at once only sometimes: the helper makes the file again until the order is proven, and a file system that
    //  records no birth time fails the control naming what it said.
    write_after_the_lock(lock.as_path(), format!("pid {pid}\n").as_str());
    if let Some(recorded) = identity {
        plant_record(repo, &[("child", pid, recorded.clone())], record_name);
    }
}

/// What:
///  What a control asks of a lock after a command ran.
/// Why:
///  One shape says the same thing about every lock of the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type After = { lockStands: boolean; said: string; code: number | null };
/// ```
struct After {
    /// Whether the lock file still stands.
    lock_stands: bool,
    /// What the command wrote to standard error.
    said: String,
    /// Its exit status.
    code: Option<i32>,
}

/// What:
///  Run a wrapped command and report what became of `name`'s lock.
/// Why:
///  Every control of this file asks the same question,
///  and the elapsed time is the claim that matters:
/// a command that goes on spends a moment,
///  and one that waits spends the sixty seconds nobody asked for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const after = afterRunning(fixture, repo, ['add', 'a.txt'], 'index.lock');
/// ```
fn after_running(fixture: &Fixture, repo: &Path, arguments: &[&str], name: &str) -> (After, Duration) {
    return after_running_with(fixture, repo, arguments, name, &[]);
}

/// What:
///  `after_running` with variables for the wrapper's own process,
///  which a control uses to report birth times as another file system would.
/// Why:
///  The gate's answer depends on the two birth times,
///  and this volume gives them one tick or two;
///  a variable that fixes them makes the control state a fact instead of the machine's speed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// afterRunningWith(fixture, repo, ['status'], 'index.lock', [['CLI_GIT_TEST_ONLY_BIRTH_TIME', 'none']]);
/// ```
fn after_running_with(fixture: &Fixture, repo: &Path, arguments: &[&str], name: &str, environment: &[(&str, &str)]) -> (After, Duration) {
    let began: Instant = Instant::now();
    let mut command: Command = super::support::wrapped(fixture);
    for (variable, value) in environment {
        command.env(variable, value);
    }
    command.current_dir(repo).args(arguments);
    let observed: Observed = super::support::observe(&mut command, b"");
    let elapsed: Duration = began.elapsed();
    return (
        After {
            lock_stands: repo.join(".git").join(name).exists(),
            said: stderr_of(&observed),
            code: observed.code,
        },
        elapsed,
    );
}

/// What:
///  A fixture repository with one commit,
///  a staged change the next commit can land,
///  and a file an `add` can stage.
/// Why:
///  Every control needs a repository whose next commit is a real one,
///  and hooks off means the fixture's Git takes no hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const repo = stagedFixture('index-lock');
/// ```
fn staged_fixture(name: &str) -> (Fixture, PathBuf) {
    let fixture: Fixture = fixture(name);
    let repo: PathBuf = repository(&fixture, std::ffi::OsStr::new("repo"));
    std::fs::write(repo.join("staged.txt"), "the person's change\n").expect("the person's change");
    git(&fixture, repo.as_path(), &["add", "staged.txt"]);
    std::fs::write(repo.join("later.txt"), "a later change\n").expect("a later change");
    return (fixture, repo);
}

/// What:
///  Commit the staged change through the wrapper and require that it landed.
/// Why:
///  The commit is the command the audit's report says waited and was withdrawn,
///  so it is the command every control's ending is judged by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commit = runWrapped(repo, ['commit', '--message', 'next', '--', 'staged.txt']); expect(commit.code).toBe(0);
/// ```
fn commit_lands(fixture: &Fixture, repo: &Path, label: &str) -> Duration {
    let began: Instant = Instant::now();
    // The commit's work happens in the commit service, outside the command's own tree, so it runs under the commit bound the
    //  sibling commit controls use rather than the shared bound of a forwarded command.
    let observed: Observed = super::commit::run_commit(fixture, repo, &["commit", "--message", "next", "--", "staged.txt"]);
    let elapsed: Duration = began.elapsed();
    assert_eq!(
        observed.code,
        Some(0),
        "{label}: the commit must land now, not be withdrawn; standard output {:?}, standard error {:?}",
        String::from_utf8_lossy(observed.stdout.as_slice()),
        stderr_of(&observed)
    );
    assert!(elapsed < GOES_ON_WITHIN, "{label}: the commit took {elapsed:?}, which is the wait this change removes");
    assert!(!repo.join(".git/index.lock").exists(), "{label}: the lock must be gone after the commit");
    return elapsed;
}

/// What:
///  An index lock left by a merge whose checkout stalled,
///  with the wrapper and its whole process group killed at `SIGKILL`.
/// Why:
///  This is the state the audit of alpha.14 recorded (finding D8),
///  produced by a real kill rather than planted,
///  so the proof is exercised on the shape a kill really leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// killGroup(aMergeWithAStalledCheckout(repo)); // leaves index.lock + index~pid.lock + the wrapper's record
/// ```
fn a_merge_killed_while_its_checkout_stalled(fixture: &Fixture, repo: &Path) -> PathBuf {
    let ready: PathBuf = fixture.root.join("filter-started");
    std::fs::write(repo.join(".gitattributes"), "*.dat filter=slow\n").expect("the attributes");
    std::fs::write(repo.join("big.dat"), "from main\n").expect("the file");
    git(fixture, repo, &["add", ".gitattributes", "big.dat"]);
    git(fixture, repo, &["-c", "filter.slow.smudge=cat", "-c", "filter.slow.clean=cat", "commit", "--quiet", "--message=main"]);
    git(fixture, repo, &["checkout", "--quiet", "-b", "other"]);
    std::fs::write(repo.join("big.dat"), "from other\n").expect("the other branch's file");
    git(fixture, repo, &["add", "big.dat"]);
    git(fixture, repo, &["-c", "filter.slow.smudge=cat", "-c", "filter.slow.clean=cat", "commit", "--quiet", "--message=other"]);
    git(fixture, repo, &["checkout", "--quiet", "main"]);
    // The filter stalls inside the checkout, which is inside the index lock.
    git(fixture, repo, &["config", "filter.slow.smudge", format!("sh -c 'touch {}; sleep 600; cat'", ready.display()).as_str()]);
    git(fixture, repo, &["config", "filter.slow.clean", "cat"]);
    let mut command: Command = Command::new(fixture.root.join("bin/git"));
    hermetic(&mut command, fixture, wrapper_first_path(fixture).as_os_str());
    if let Some(runtime) = super::support::service_runtime(fixture) {
        command.env("XDG_RUNTIME_DIR", runtime);
    }
    command.current_dir(repo).args(["merge", "--no-edit", "other"]).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    // The merge leads its own process group, so the kill reaches the wrapper, its Git and the stalled filter together.
    command.process_group(0);
    let mut child: Child = command.spawn().expect("the merge starts");
    own_group(&child);
    let began: Instant = Instant::now();
    while !ready.exists() {
        assert!(began.elapsed() < HOLD_PATIENCE, "the merge never reached its stalled checkout");
        std::thread::sleep(Duration::from_millis(20));
    }
    // SAFETY: `killpg` takes a group number and a signal and touches no memory; the group is the one this control started.
    unsafe { libc::killpg(child.id() as libc::pid_t, libc::SIGKILL) };
    let _waited = child.wait();
    // The wrapper led the group, so the kill reached the wrapper, its Git and the stalled filter together.
    let lock: PathBuf = repo.join(".git/index.lock");
    assert!(lock.exists(), "the kill must leave the index lock the merge's Git held");
    git(fixture, repo, &["config", "filter.slow.smudge", "cat"]);
    return lock;
}

/// What:
///  The lock a killed merge left is kept
///  when its PID file and the lock were born in one tick,
///  which is what this volume gives the two files one Git writes at once,
///  and the next command says why instead of removing it.
/// Why:
///  The gate removes a lock only when its PID file is born strictly after it
///  (`Judge::pid_file_state`,
/// `PidFile::Unproven`).
///  A killed Git writes both files within microseconds,
///  so on this volume they carry one birth stamp (measured:
/// three samples of a checkout held by a stalled
///  filter,
/// one with identical stamps and two within 150 microseconds),
///  and nothing shows whether the file was written for this lock or by an earlier holder.
///  A review of `d189162e3e` found a live Git's editor-phase lock removed in exactly that state,
///  and the fix reads an unproven order as "keep".
///  The clearing itself is the control beside this one
///  (`every_lock_file_a_killed_command_can_leave_is_cleared_on_the_same_proof`),
/// whose fixture proves the order.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = aMergeKilledWhileItsCheckoutStalled(); expect(runWrapped(['status']).said).toContain('cannot be told apart');
/// expect(lock).toExist();
/// ```
#[test]
fn an_index_lock_a_killed_merge_left_is_kept_when_its_birth_order_is_unproven() {
    let fixture: Fixture = fixture("killed-merge-lock");
    let repo: PathBuf = super::support::repository(&fixture, std::ffi::OsStr::new("repo"));
    let lock: PathBuf = a_merge_killed_while_its_checkout_stalled(&fixture, repo.as_path());
    let pid_file: PathBuf = pid_file_of(lock.as_path());
    assert!(pid_file.exists(), "the kill leaves the PID file the lock's holder was named in");
    // A file system of one-second ticks, which this control states rather than waits for: the two files one Git writes at
    //  once then carry one birth stamp, and the order is not proven.
    let (after, elapsed) = after_running_with(&fixture, repo.as_path(), &["status", "--porcelain=v1"], "index.lock", &[("CLI_GIT_TEST_ONLY_BIRTH_TIME", "1000")]);
    assert!(after.lock_stands, "an order the file system does not prove keeps the lock: {}", after.said);
    assert!(after.said.contains("cannot be told apart") || after.said.contains("no birth time"), "the line says why nothing was removed: {}", after.said);
    assert!(!after.said.contains("cleared"), "nothing may claim this lock was cleared: {}", after.said);
    assert!(elapsed < GOES_ON_WITHIN, "the command that meets it still goes on at once: {elapsed:?}");
    let _ = pid_file;
    super::commit::remove_settled(&fixture);
    return;
}

/// What:
///  The lock a killed merge left is cleared,
/// and the next commit lands,
///  when the order the gate asks for is proven.
/// Why:
///  This is what the change exists for,
/// on the state a real group kill leaves:
///  the PID file names the Git the wrapper's own record names,
///  and with the two births told apart the next command clears the lock and the person's commit lands.
///  The control makes the order provable the way a file system of a finer clock would,
///  because this volume gives the two files one tick most of the time
///  (the control beside this one pins that half).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lock = aMergeKilledWhileItsCheckoutStalled(); rewritePidFileLater(lock); expect(runWrapped(['status']).cleared).toBe(true);
/// expect(commit(['-m', 'next']).code).toBe(0);
/// ```
#[test]
fn an_index_lock_a_killed_merge_left_is_cleared_and_the_next_commit_lands() {
    let fixture: Fixture = fixture("killed-merge-lock-proven");
    let repo: PathBuf = super::support::repository(&fixture, std::ffi::OsStr::new("repo"));
    let lock: PathBuf = a_merge_killed_while_its_checkout_stalled(&fixture, repo.as_path());
    let pid_file: PathBuf = pid_file_of(lock.as_path());
    let named: String = std::fs::read_to_string(pid_file.as_path()).expect("the PID file the kill left");
    // The same number the kill left, written once more so that its birth is strictly after the lock's.
    write_after_the_lock(lock.as_path(), named.as_str());
    let (after, elapsed) = after_running(&fixture, repo.as_path(), &["status", "--porcelain=v1"], "index.lock");
    assert!(!after.lock_stands, "a proven order and a dead recorded writer clear the lock: {}", after.said);
    assert!(!pid_file.exists(), "the PID file of a cleared lock goes with it");
    assert!(after.said.contains("cleared"), "one line must say what was cleared: {}", after.said);
    assert!(elapsed < GOES_ON_WITHIN, "the next command took {elapsed:?}");
    assert_eq!(after.code, Some(0), "the command must go on: {}", after.said);
    std::fs::write(repo.join("staged.txt"), "the person's change\n").expect("the person's change");
    git(&fixture, repo.as_path(), &["add", "staged.txt"]);
    commit_lands(&fixture, repo.as_path(), "after a killed merge");
    super::commit::remove_settled(&fixture);
}

/// What:
///  A lock whose file system records no birth time is kept,
///  and the line names the file and the fact that nothing shows the PID file was written for it.
/// Why:
///  This is the other file system the review named:
///  one whose `statx` gives no birth time at all.
///  Nothing can prove the order there,
///  so the pass must keep every lock whose PID file it cannot pin,
///  and tell the person what stands and what clears it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const after = afterRunningWith(fixture, repo, ['status'], 'index.lock', [['CLI_GIT_TEST_ONLY_BIRTH_TIME', 'none']]);
/// expect(after.said).toContain('no birth time');
/// ```
#[test]
fn a_lock_whose_file_system_records_no_birth_time_is_kept_and_named() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("no-birth-time-lock");
    let (pid, identity) = a_dead_holder();
    plant_lock(repo.as_path(), "index.lock", pid, Some(&identity), "record-no-birth-time");
    let (after, elapsed) = after_running_with(&fixture, repo.as_path(), &["status", "--porcelain=v1"], "index.lock", &[("CLI_GIT_TEST_ONLY_BIRTH_TIME", "none")]);
    assert_eq!(after.code, Some(0), "the command goes on: {}", after.said);
    assert!(after.lock_stands, "nothing proves the order, so the lock stays: {}", after.said);
    assert!(after.said.contains("no birth time"), "the line says the file system records none: {}", after.said);
    assert!(after.said.contains("removing the file"), "the line says what a person can do: {}", after.said);
    assert!(elapsed < GOES_ON_WITHIN, "the command took {elapsed:?}");
    super::commit::remove_settled(&fixture);
}

/// What:
///  Each lock file a killed command can leave,
///  with the record a kill leaves,
///  is cleared by the next command.
/// Why:
///  The index lock is not the only one:
///  `HEAD`,
/// the shallow file,
/// the packed references,
/// a reference and the reftable backend's list
///  are all filled and renamed the same way,
///  so the same proof and the same removal apply to every one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of EVERY_LOCK) { plantLock(repo, name, dead); expect(clear(name)).toBe('cleared'); }
/// ```
#[test]
fn every_lock_file_a_killed_command_can_leave_is_cleared_on_the_same_proof() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("every-lock-file");
    for (index, name) in EVERY_LOCK.iter().enumerate() {
        let (pid, identity) = a_dead_holder();
        plant_lock(repo.as_path(), name, pid, Some(&identity), format!("record-{index}").as_str());
        let (after, elapsed) = after_running(&fixture, repo.as_path(), &["status", "--porcelain=v1"], name);
        assert!(!after.lock_stands, "{name}: a lock of a proven dead holder must be cleared: {}", after.said);
        assert!(after.said.contains("cleared"), "{name}: the removal is told once: {}", after.said);
        assert!(elapsed < GOES_ON_WITHIN, "{name}: the command took {elapsed:?}");
        assert_eq!(after.code, Some(0), "{name}: the command must go on: {}", after.said);
    }
    super::commit::remove_settled(&fixture);
}

/// What:
///  A lock whose record names a process that still runs is kept,
///  and the words name the lock and what a person can do.
/// Why:
///  A live holder's lock protects its writer's work,
///  so nothing is removed,
///  and a command that cannot use the index says what stands in its way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const live = spawnOwned('sleep 600'); plantLock(repo, 'index.lock', live.pid); expect(add.code).not.toBe(0);
/// ```
#[test]
fn a_lock_whose_record_names_a_live_process_is_kept() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("live-holder-lock");
    let mut holder: Child = spawn_owned(quiet("sleep").arg("600")).expect("a live holder");
    let pid: i64 = i64::from(holder.id());
    let identity: String = identity_of_process(pid).expect("the live holder's identity");
    plant_lock(repo.as_path(), "index.lock", pid, Some(&identity), "record-live");
    let (after, _elapsed) = after_running(&fixture, repo.as_path(), &["add", "later.txt"], "index.lock");
    assert!(after.lock_stands, "a lock of a live holder must stay: {}", after.said);
    assert!(!after.said.contains("cleared"), "nothing may claim a live holder's lock was cleared: {}", after.said);
    assert!(after.said.contains("index.lock"), "the refusal must name the lock: {}", after.said);
    let _killed = holder.kill();
    let _waited = holder.wait();
    super::commit::remove_settled(&fixture);
}

/// What:
///  A forwarded command's own record is swept once that command has ended,
///  so the folder does not grow with every command run in a repository.
/// Why:
///  The writer put two spaces after `self` and the reader parsed one,
///  so a forwarded command's own record named nobody and the sweep never removed it:
///  every command left a file,
/// and past 512 files a lookup reads only the first 512
///  (a review of `675771e759`,
/// finding 1).
///  Eight forwarded commands must leave at most the records of the commands still running.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let n = 0; n < 8; n++) expect(runWrapped(['status']).code).toBe(0);
/// expect(countRecords(repo)).toBeLessThanOrEqual(1);
/// ```
#[test]
fn a_forwarded_commands_own_record_is_swept_so_the_folder_does_not_grow() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("record-folder-growth");
    let folder: PathBuf = git_policy_cli::lock_holder_records::folder_of(repo.join(".git").as_path());
    for _ in 0..8 {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
        assert_eq!(observed.code, Some(0), "a forwarded command goes on: {}", stderr_of(&observed));
    }
    let left: Vec<String> = std::fs::read_dir(folder.as_path()).map(|entries| { return entries.flatten().map(|entry| { return entry.file_name().to_string_lossy().into_owned(); }).collect(); }).unwrap_or_default();
    assert!(left.len() <= 1, "eight forwarded commands leave at most the record of the command that is running, not eight: {left:?}");
    super::commit::remove_settled(&fixture);
}

/// What:
///  A named pipe at a lock's PID-file name does not hold a command.
/// Why:
///  The pass reads a PID file on every command now,
///  and a read that opens a pipe waits for a writer that may never come
///  (a review of `675771e759`,
/// finding 2).
///  The read goes through `plain_file`,
/// which refuses everything but a plain file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantFifo('.git/index~pid.lock'); expect(runWrapped(['status']).code).toBe(0); // within the bound
/// ```
#[test]
fn a_fifo_at_a_pid_file_name_does_not_hold_the_command() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("fifo-pid-file");
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), b"a half-written index\n").expect("the lock");
    let fifo: PathBuf = pid_file_of(lock.as_path());
    let made: std::process::Output = Command::new("mkfifo").arg(fifo.as_path()).output().expect("mkfifo runs");
    assert!(made.status.success(), "the control makes a named pipe: {}", String::from_utf8_lossy(made.stderr.as_slice()));
    let (after, elapsed) = after_running(&fixture, repo.as_path(), &["status", "--porcelain=v1"], "index.lock");
    assert_eq!(after.code, Some(0), "the command goes on: {}", after.said);
    assert!(elapsed < GOES_ON_WITHIN, "the command took {elapsed:?}, which is a read that waited on a pipe");
    assert!(lock.exists(), "a lock whose PID file is not a plain file keeps its place: {}", after.said);
    super::commit::remove_settled(&fixture);
}

/// What:
///  A lock below the walk's depth bound keeps the record that explains it.
/// Why:
///  The sweep removes a record nothing needs,
///  and a lock the walk did not visit can still need one:
///  a record taken away leaves a lock no later pass can prove abandoned
///  (a review of `675771e759`,
/// finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantDeepRefLock(deadHolder); expect(runWrapped(['status']).code).toBe(0); expect(record).toExist();
/// ```
#[test]
fn a_lock_below_the_walks_depth_keeps_the_record_that_explains_it() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("deep-lock-record");
    let (pid, identity) = a_dead_holder();
    let deep: PathBuf = repo.join(".git/refs/heads/a/b/c/d/e/f/g/h/i.lock");
    std::fs::create_dir_all(deep.parent().expect("the lock's folder")).expect("the deep folders");
    std::fs::write(deep.as_path(), b"0000000000000000000000000000000000000000\n").expect("the deep lock");
    write_after_the_lock(deep.as_path(), format!("pid {pid}\n").as_str());
    plant_record(repo.as_path(), &[("child", pid, identity)], "record-deep-lock");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["status", "--porcelain=v1"]);
    assert_eq!(observed.code, Some(0), "the command goes on: {}", stderr_of(&observed));
    let folder: PathBuf = git_policy_cli::lock_holder_records::folder_of(repo.join(".git").as_path());
    let kept: bool = std::fs::read_dir(folder.as_path()).map(|entries| { return entries.flatten().any(|entry| { return entry.file_name().to_string_lossy().starts_with("record-deep-lock"); }); }).unwrap_or(false);
    assert!(kept, "a lock the walk could not see keeps the record that explains it");
    assert!(deep.exists(), "and the lock itself is left alone");
    super::commit::remove_settled(&fixture);
}

/// What:
///  A lock whose PID file names a number a live process has now is kept,
///  even when a record of a dead process names that number:
///  the number was used before,
///  and a record of that earlier process proves nothing about the program that has it now,
///  which may be the Git that wrote this very PID file.
/// Why:
///  Removing such a lock is the worst kind of defect the project knows:
///  the review of `d4a0803231` held a real `git commit --all` in its editor,
///  planted exactly this record,
///  and the removal made that live commit fail with
///  `fatal: repository has been updated, but unable to write new index file ... "git restore --staged :/" to recover`,
///  which for `commit --only <path>` also discards the only copy of that staging.
///  The control that used to assert the removal now asserts the lock stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reused = deadHolder(); plantLock(repo, 'index.lock', process.pid, reused.identity); expect(kept).toBe(true);
/// ```
#[test]
fn a_lock_whose_number_a_live_process_has_is_kept() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("reused-number-lock");
    let (_dead_pid, dead_identity) = a_dead_holder();
    // The number is this control's own process, which runs now; the identity is a process that ended.
    let live_number: i64 = i64::from(std::process::id());
    plant_lock(repo.as_path(), "index.lock", live_number, Some(&dead_identity), "record-reused");
    let (after, _elapsed) = after_running(&fixture, repo.as_path(), &["add", "later.txt"], "index.lock");
    assert!(after.lock_stands, "a number a live process has is never proved gone: {}", after.said);
    assert!(!after.said.contains("cleared"), "nothing may claim this lock was cleared: {}", after.said);
    assert!(after.said.contains("live process has now"), "the line says a live process has the number: {}", after.said);
    super::commit::remove_settled(&fixture);
}

/// What:
///  A real `git commit --all` held in its editor keeps its lock and finishes its commit
///  while a wrapper command meets the lock and a record of a dead process names the commit's number.
/// Why:
///  This is the review's finding 1 as a control:
///  the lock is a live Git's editor-phase lock,
///  the number a record names was used by a process that ended,
///  and the commit must not lose the index it wrote into that lock.
///  It is the case the fabricated lock of the rewritten control above cannot see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commit = heldEditorCommit(repo); plantRecord(commit.pid, deadIdentity); expect(runWrapped(['status']).code).toBe(0);
/// expect(lockStands).toBe(true); releaseEditor(); expect(commitExit).toBe(0);
/// ```
#[test]
fn a_live_commits_editor_phase_lock_is_kept_when_a_dead_record_names_its_number() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("live-editor-lock");
    // The person's own commit is real Git's, so the lock it holds must name itself: the wrapper turns lockfilePid on for its
    //  own Git, and this one is started directly.
    git(&fixture, repo.as_path(), &["config", "core.lockfilePid", "true"]);
    let ready: PathBuf = fixture.root.join("editor-ready");
    let let_go: PathBuf = fixture.root.join("editor-go");
    let editor: PathBuf = fixture.root.join("editor.sh");
    executable(
        editor.as_path(),
        format!(
            "#!/bin/sh\nprintf 'the live commit\\n' > \"$1\"\necho ready > '{}'\nwhile [ ! -e '{}' ]; do sleep 0.05; done\nexit 0\n",
            ready.display(),
            let_go.display()
        )
        .as_bytes(),
    );
    let lock: PathBuf = repo.join(".git/index.lock");
    let pid_file: PathBuf = pid_file_of(lock.as_path());
    let mut commit: Child = super::support::direct_held(&fixture)
        .current_dir(repo.as_path())
        .args(["commit", "--all"])
        .env("GIT_EDITOR", editor.as_path())
        .env("EDITOR", editor.as_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the live commit starts");
    let began: Instant = Instant::now();
    while !ready.exists() {
        assert!(began.elapsed() < HOLD_PATIENCE, "the live commit never opened its editor");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(lock.exists(), "the live commit holds index.lock through its editor");
    assert!(pid_file.exists(), "and it wrote the PID file that names it");
    let commit_pid: i64 = i64::from(commit.id());
    let (_dead_pid, dead_identity) = a_dead_holder();
    plant_record(repo.as_path(), &[("child", commit_pid, dead_identity)], "record-of-a-reused-number");
    let (after, _elapsed) = after_running(&fixture, repo.as_path(), &["status", "--porcelain=v1"], "index.lock");
    assert_eq!(after.code, Some(0), "the command that meets the lock goes on: {}", after.said);
    assert!(after.lock_stands, "the live commit's lock must stay: {}", after.said);
    assert!(pid_file.exists(), "the live commit's PID file must stay");
    assert!(!after.said.contains("cleared"), "nothing may claim the live commit's lock was cleared: {}", after.said);
    std::fs::write(let_go.as_path(), b"go\n").expect("let the editor finish");
    let ended: std::process::ExitStatus = commit.wait().expect("the live commit ends");
    assert_eq!(ended.code(), Some(0), "the live commit must finish with its index");
    let subject: String = String::from_utf8_lossy(super::support::git(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).stdout.as_slice()).into_owned();
    assert_eq!(subject.trim(), "the live commit", "the live commit landed");
    super::commit::remove_settled(&fixture);
}

/// The variable that makes this test binary hold a lock open,
/// non-dumpably,
/// for the control below.
const HOLD_NON_DUMPABLY: &str = "CLI_GIT_TEST_NON_DUMPABLE_LOCK";

/// What:
///  The child body:
/// hold the lock open for writing,
/// become non-dumpable,
/// and wait to be killed.
/// Why:
///   `PR_SET_DUMPABLE` is reset by `execve`,
///  so the holder must be a process this binary ran,
///  with no exec after the call:
///  a shell made non-dumpable before its own start is dumpable again after it.
///  Inert unless the control set the variable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!process.env.HOLD) return; const fd = open(lock, 'r+'); prctl(0); writeFileSync(ready); park();
/// ```
#[test]
fn a_non_dumpable_writer_holds_the_lock_for_the_control() {
    let Ok(lock) = std::env::var(HOLD_NON_DUMPABLY) else {
        return;
    };
    let file: std::fs::File = std::fs::OpenOptions::new().read(true).write(true).open(lock.as_str()).expect("open the lock for writing");
    // SAFETY: `prctl` takes integers and touches no memory of this process.
    let refused: libc::c_int = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0) };
    assert_eq!(refused, 0, "the holder must become non-dumpable");
    std::fs::write(format!("{lock}.ready"), b"ready\n").expect("the ready file");
    let _held = file;
    std::thread::park();
}

/// What:
///  A lock a non-dumpable writer of this account holds open is kept.
/// Why:
///  The review's finding 2:
///  the kernel closes `/proc/<pid>/fd` of a process of this account that is not dumpable,
///  and a scan that tolerates every `PID ` reason acts through it.
///  The kernel's read lease is refused while any program has the lock open for writing,
///  so the lock stays without reading any descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holder = nonDumpableWriter(lock); expect(lockStands).toBe(true);
/// ```
#[test]
fn a_lock_a_non_dumpable_writer_of_this_account_holds_open_is_kept() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("non-dumpable-writer");
    let (pid, identity) = a_dead_holder();
    plant_lock(repo.as_path(), "index.lock", pid, Some(&identity), "record-dead-writer");
    let lock: PathBuf = repo.join(".git/index.lock");
    let mut holder: Child = start_non_dumpable_holder(lock.as_path());
    assert!(unreadable_holder(lock.as_path(), i64::from(holder.id())), "the holder must be non-dumpable and hold the lock open");
    let (after, _elapsed) = after_running(&fixture, repo.as_path(), &["status", "--porcelain=v1"], "index.lock");
    assert_eq!(after.code, Some(0), "the command goes on: {}", after.said);
    assert!(after.lock_stands, "a writer this account cannot look into keeps the lock: {}", after.said);
    assert!(!after.said.contains("cleared"), "nothing may claim this lock was cleared: {}", after.said);
    let _killed = holder.kill();
    let _waited = holder.wait();
    super::commit::remove_settled(&fixture);
}

/// This test binary running the child body on `lock`.
/// The wrapper never starts itself;
/// this starts libtest,
/// so the ban on `current_exe` does not reach it.
#[allow(clippy::disallowed_methods, reason = "starts this test binary (libtest), not the wrapper; the ban on `current_exe` guards the wrapper's own start")]
fn start_non_dumpable_holder(lock: &Path) -> Child {
    let binary: PathBuf = std::env::current_exe().expect("this test binary");
    return Command::new(binary)
        .args(["--exact", "killed_holder_lock::a_non_dumpable_writer_holds_the_lock_for_the_control", "--nocapture", "--test-threads=1"])
        .env(HOLD_NON_DUMPABLY, lock)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the non-dumpable holder starts");
}

/// Whether the holder is there and this account cannot read its descriptor directory.
fn unreadable_holder(lock: &Path, pid: i64) -> bool {
    let ready: PathBuf = PathBuf::from(format!("{}.ready", lock.display()));
    let began: Instant = Instant::now();
    while began.elapsed() < Duration::from_secs(10) {
        if ready.exists() {
            return std::fs::read_dir(format!("/proc/{pid}/fd")).is_err();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    return false;
}

/// What:
///  A lock the wrapper did not take carries no record,
///  so nothing proves its holder gone and it is kept,
///  with the words naming what is known and what a person can do.
/// Why:
///  Real Git run directly,
/// an editor and an IDE write no record of the wrapper's,
///  and a number in a PID file is not a proof:
///  a Git of another PID namespace writes one this table has never had.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantLock(repo, 'index.lock', deadPid); /* no record */ expect(add.code).not.toBe(0); expect(lockStands).toBe(true);
/// ```
#[test]
fn a_lock_with_no_record_of_the_wrapper_is_kept() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("unrecorded-lock");
    let (pid, _identity) = a_dead_holder();
    plant_lock(repo.as_path(), "index.lock", pid, None, "record-unused");
    let (after, _elapsed) = after_running(&fixture, repo.as_path(), &["add", "later.txt"], "index.lock");
    assert!(after.lock_stands, "a lock no record explains must stay (exit {:?}): {}", after.code, after.said);
    assert!(!after.said.contains("cleared"), "nothing may claim an unproven lock was cleared: {}", after.said);
    assert!(after.said.contains("index.lock"), "the message must name the lock (exit {:?}): {}", after.code, after.said);
    assert!(after.said.contains("no record"), "the message must say no record of the wrapper's commands explains that number: {}", after.said);
    assert!(after.said.contains("removing the file"), "the message must say what a person can do: {}", after.said);
    super::commit::remove_settled(&fixture);
}

/// What:
///  Two commands that meet one abandoned lock at the same instant end correctly,
///  and the lock is cleared once:
/// the second finds it gone.
/// Why:
///  Each of them takes the exclusive `flock` of the lock file before its last look,
///  so one removal is the only one that happens,
///  and the other finds no lock to explain and stages its own file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [reader, writer] = await Promise.all([status(), add('a.txt')]); expect([reader.code, writer.code]).toEqual([0, 0]);
/// ```
/// A writer and a reader,
/// not two writers:
/// Git itself lets one index writer at a time hold `index.lock`,
/// so two simultaneous
///  `git add`s die with Git's own `File exists` whatever the wrapper does (`binary_tests.rs`,
///  `the_second_of_two_adds_at_once_...`).
/// The lock this control is about is the *dead* one,
/// and the reader meets it too.
#[test]
fn two_commands_meeting_one_dead_lock_clear_it_once_and_both_end_correctly() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("two-commands-one-lock");
    let (pid, identity) = a_dead_holder();
    plant_lock(repo.as_path(), "index.lock", pid, Some(&identity), "record-raced");
    std::fs::write(repo.join("first.txt"), "first\n").expect("the first file");
    std::fs::write(repo.join("second.txt"), "second\n").expect("the second file");
    let mut commands: Vec<Command> = Vec::new();
    for arguments in [vec!["status", "--porcelain=v1"], vec!["add", "first.txt"]] {
        let mut command: Command = super::support::bounded(&fixture, fixture.root.join("bin/git").as_path(), wrapper_first_path(&fixture).as_os_str());
        command.current_dir(repo.as_path()).args(arguments).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        commands.push(command);
    }
    let mut children: Vec<Child> = Vec::new();
    for mut command in commands {
        children.push(command.spawn().expect("a command starts"));
    }
    let mut said: String = String::new();
    let mut codes: Vec<Option<i32>> = Vec::new();
    for child in children {
        let output = child.wait_with_output().expect("a command ends");
        codes.push(output.status.code());
        said.push_str(String::from_utf8_lossy(output.stderr.as_slice()).as_ref());
    }
    assert_eq!(codes, vec![Some(0), Some(0)], "both commands must end correctly: {said}");
    assert!(!repo.join(".git/index.lock").exists(), "the lock must be gone");
    let cleared: usize = said.matches("cleared").count();
    assert!(cleared <= 1, "one removal is told once, and the other command finds no lock to clear: {said}");
    let staged: String = porcelain(&fixture, repo.as_path());
    assert!(staged.contains("first.txt"), "the writer's file is staged: {staged}");
    super::commit::remove_settled(&fixture);
}

/// What:
///  The index lock of a commit service that was killed is cleared by the next forwarded command,
///  whose notice of the lock is printed once rather than fifty times.
/// Why:
///  A second audit of alpha.14 found a `git add` printing the same notice fifty times in 110 ms,
///  because the retry loop ran Git again for each attempt without asking what the wait proved.
///  The service's own marker proves its holder gone,
/// so its own reclaimer removes the lock,
///  and the command goes on at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plantMarkedLock(repo, deadHolder); const add = runWrapped(repo, ['add', 'later.txt']); expect(add.code).toBe(0);
/// expect(add.stderr.match(/index\.lock/g).length).toBeLessThanOrEqual(2);
/// ```
#[test]
fn the_lock_of_a_killed_commit_service_is_cleared_and_its_notice_is_printed_once() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("killed-service-lock");
    let (pid, identity) = a_dead_holder();
    let lock: PathBuf = repo.join(".git/index.lock");
    std::fs::write(lock.as_path(), format!("cli-git-service v2 1 {pid} {identity}\n").as_bytes()).expect("the service's lock");
    let (after, elapsed) = after_running(&fixture, repo.as_path(), &["add", "later.txt"], "index.lock");
    assert_eq!(after.code, Some(0), "the add goes on once the dead service's lock is gone: {}", after.said);
    assert!(!after.lock_stands, "the killed service's lock must be cleared");
    let mentions: usize = after.said.matches("index.lock").count();
    assert!(mentions <= 2, "the lock is named once or twice, not fifty times ({mentions}): {}", after.said);
    assert!(elapsed < GOES_ON_WITHIN, "the add took {elapsed:?}");
    super::commit::remove_settled(&fixture);
}

/// What:
///  The words of a lock that cannot be proven gone are true:
///  they name the lock,
///  what the PID file holds,
///  and the fact that no record of the wrapper's commands names that number.
/// Why:
///  A message that says "no process here will release it" would be false,
///  because a Git of another PID namespace holds it and writes a number this table has never had.
///  The person deciding what to do needs the difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(message).toContain('no record'); expect(message).not.toContain('no process here will release it');
/// ```
#[test]
fn the_message_about_a_lock_that_cannot_be_proven_gone_says_what_is_known() {
    let (fixture, repo): (Fixture, PathBuf) = staged_fixture("unproven-message");
    let (pid, _identity) = a_dead_holder();
    plant_lock(repo.as_path(), "index.lock", pid, None, "record-unused");
    let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["merge", "--no-edit", "main"]);
    let said: String = stderr_of(&observed);
    assert!(said.contains("index.lock"), "the message names the lock: {said}");
    assert!(said.contains(format!("{pid}").as_str()), "the message names the number the PID file holds: {said}");
    assert!(said.contains("no record"), "the message says no record of the wrapper's commands names that number: {said}");
    assert!(!said.contains("no process here will release it"), "the message must not claim a proof it does not have: {said}");
    super::commit::remove_settled(&fixture);
}
