//! What:
//!  Controls of the ending a run may give a filling's clean,
//!  and of the reader that says how a filling ended.
//! Why:
//!   A control that shows what a failed filling is reported as rests on the stand-in ending the clean exactly as the runner
//! words an ending from outside its own bounds,
//! and on the reader naming the state and the words the filling came to.
//! A stand-in that ended the wrong run,
//!  or one that passed every run through and ended nothing,
//! would make such a control pass for no reason at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(signalOf(standIn, ['clean'], { CARGO_BUILD_BUILD_DIR: staging })).toBe(9); expect(run(standIn, ['--version']).stdout).toContain('real');
//! ```

use super::support::{
    ENDED_WITH_SIGKILL, Fixture, SERVICE_LOG_FILE, a_filling_that_did_not_run_to_its_end, a_filling_that_did_not_run_to_its_end_watching,
    end_the_fills_clean_within, executable, fixture,
};
use git_policy_cli::work_item::{CLAIMED, DONE, FAILED, HELD, PENDING, RECOVERING, WorkItem, WorkKind, enqueue, state_directory};
use std::os::unix::process::ExitStatusExt as _;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

/// A stamp of a filling that was handed over long before any control here runs,
/// so a record that carries it is an earlier filling of the same worktree path.
const AN_EARLIER_FILLING: &str = "2026-01-01T00:00:00.000Z";

/// How long a control waits for the reader to answer before it says the reader did not.
///  The reader's own bound is the harness's patience, which is far longer; a control that waits this long has already
/// proved that the reader is not bounded by it.
const A_READERS_ANSWER_PATIENCE: Duration = Duration::from_secs(3);


/// What a stand-in's real program is in this control:
/// it says which of its runs reached it,
/// so a run the stand-in should have passed through is told apart from one it ended.
const A_REAL_PROGRAM: &str = "#!/bin/sh\nprintf 'reached the real program: %s\\n' \"$*\"\n";

/// The fixture's own `cargo` name,
/// which the stand-in takes over.
fn cargo_in(fixture: &Fixture) -> PathBuf {
    return fixture.root.join("bin/cargo");
}

/// What:
///  The end of a run of a stand-in `cargo`,
///  with standard error kept for the failure.
/// # Panics
///  When the program cannot be started.
fn run_cargo(fixture: &Fixture, arguments: &[&str], build_directory: Option<&str>) -> std::process::Output {
    let mut command: Command = Command::new(cargo_in(fixture));
    command.args(arguments).env_clear().env("PATH", "/usr/bin:/bin").env("HOME", fixture.root.join("home"));
    if let Some(staging) = build_directory {
        command.env(git_policy_cli::package_repair_prewarm::BUILD_DIRECTORY_VARIABLE, staging);
    }
    return command.output().expect("the stand-in cargo runs");
}

/// What:
///  A stand-in ends a filling's clean with `SIGKILL`, fails the resolution that follows it,
///  and is the real program for every other run.
/// Why:
///   This is the ending the gate's line names (`stopped by signal 9`) and the state it leaves (nothing displaced),
/// so each of the three runs is asserted:
/// the stand-in's own words end by the signal whatever it was doing,
/// a run of the real program is passed through untouched before that,
/// and the resolution after a killed clean is refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(run(['--version']).stdout).toContain('reached the real program'); expect(signalOf(run(['clean'], staging))).toBe(9); expect(run(['metadata']).status).toBe(101);
/// ```
#[test]
fn a_fillings_clean_ends_with_the_signal_and_every_other_run_reaches_the_real_program() {
    let fixture: Fixture = fixture("filling-clean-ended");
    let real: PathBuf = fixture.root.join("real-cargo");
    executable(real.as_path(), A_REAL_PROGRAM.as_bytes());
    std::fs::create_dir_all(fixture.root.join("bin")).expect("the fixture's bin directory");
    std::os::unix::fs::symlink(real.as_path(), cargo_in(&fixture)).expect("the fixture's Cargo link");
    end_the_fills_clean_within(&fixture, Some(ENDED_WITH_SIGKILL));
    // A run that is not the filling's clean is the real program's, and it says so.
    let passed: std::process::Output = run_cargo(&fixture, &["--version"], None);
    assert!(passed.status.success(), "a run that is not a filling's clean reaches the real program: {:?}", passed.status);
    assert!(
        String::from_utf8_lossy(passed.stdout.as_slice()).contains("reached the real program: --version"),
        "the real program is the one that ran: {}",
        String::from_utf8_lossy(passed.stdout.as_slice())
    );
    // A clean of a build directory that is not a filling's staging copy is passed through as well.
    let elsewhere: std::process::Output = run_cargo(&fixture, &["clean", "--locked"], Some(fixture.root.join("worktree/target").to_str().expect("utf-8")));
    assert!(elsewhere.status.success(), "a clean of a worktree's own build directory is the real program's: {:?}", elsewhere.status);
    // The filling's clean is the one that ends by the signal, which is the ending the runner words `stopped by signal 9`.
    let staging: PathBuf = fixture.root.join("deeper/stale").join(format!("target{}", git_policy_cli::package_repair_prewarm::STAGING_SUFFIX));
    let killed: std::process::Output = run_cargo(&fixture, &["clean", "--locked", "--package", "app"], Some(staging.to_str().expect("utf-8")));
    assert_eq!(killed.status.signal(), Some(libc::SIGKILL), "the filling's clean ends with the signal the runner reports as its own kind of ending: {:?}", killed.status);
    // The resolution after that clean is refused, which is the second half of the gate's line: nothing displaced, nothing kept.
    let resolved: std::process::Output = run_cargo(&fixture, &["metadata", "--offline", "--format-version", "1"], None);
    assert_eq!(resolved.status.code(), Some(101), "the resolution after a killed clean is refused: {:?} {}", resolved.status, String::from_utf8_lossy(resolved.stderr.as_slice()));
}

/// What:
///  A run that asks for no ending keeps the fixture's Cargo exactly as the fixture linked it.
/// Why:
///   Every run of the suite whose variable is unset must reach the toolchain's Cargo,
/// and a stand-in written by accident would end the clean of a filling some other control waits for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endTheFillsCleanWithin(fixture, undefined); expect(readlink(bin/cargo)).toBe(real);
/// ```
#[test]
fn a_run_that_asks_for_no_ending_keeps_the_real_cargo_linked() {
    let fixture: Fixture = fixture("filling-clean-not-ended");
    let real: PathBuf = fixture.root.join("real-cargo");
    executable(real.as_path(), A_REAL_PROGRAM.as_bytes());
    std::fs::create_dir_all(fixture.root.join("bin")).expect("the fixture's bin directory");
    std::os::unix::fs::symlink(real.as_path(), cargo_in(&fixture)).expect("the fixture's Cargo link");
    end_the_fills_clean_within(&fixture, None);
    assert_eq!(std::fs::read_link(cargo_in(&fixture)).expect("the fixture's Cargo is still the link it was"), real, "a run that asks for no ending leaves the real Cargo linked");
    let passed: std::process::Output = run_cargo(&fixture, &["--version"], None);
    assert!(passed.status.success(), "the real program still answers: {:?}", passed.status);
}

/// What:
///  A filling whose record stands in `failed` is a failure that quotes the service's own words,
///  and one that settled in `done` is none.
/// Why:
///   This is the sentence a person reads instead of an empty folder,
/// so it must name where the record stands and carry the line the service wrote about this worktree:
/// a sentence that named the bytes without the reason is the failure the gate of 2026-11-15T06:00-05:00 printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(said).toContain('failed'); expect(said).toContain('could not be filled'); move(record, 'done'); expect(said).toBeUndefined();
/// ```
#[test]
fn a_filling_that_did_not_run_to_its_end_quotes_where_it_stands_and_what_the_service_said() {
    let fixture: Fixture = fixture("filling-outcome");
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the repository's Git directory");
    let worktree: PathBuf = fixture.root.join("deeper/stale");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    // The record an add hands the service for this worktree's filling, and the moment that add was taken up.
    let started: SystemTime = SystemTime::now();
    let item: WorkItem = WorkItem {
        kind: WorkKind::Prewarm,
        repository: worktree.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from(git_policy_cli::package_repair_prewarm::PREWARM_REFERENCE),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::from("[]"),
        created: git_policy_cli::iso_time::now_iso(),
        tries: 0,
    };
    let work: PathBuf = git_policy_cli::service_work::work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("the record is queued");
    let id: String = item.id();
    // The service's own line about this filling, as its log keeps it.
    let line: String = format!(
        "monochromatic-git-policy-cli-service: the Cargo build directory of {} could not be filled: the copy of {}/target.monochromatic-prewarm could not be cleaned (stopped by signal 9: ) at_ms=1\n",
        worktree.join("target").display(),
        worktree.display()
    );
    std::fs::write(fixture.root.join(SERVICE_LOG_FILE), line.as_bytes()).expect("the service's log");
    // The state a consumer that gave up on a filling leaves it in.
    std::fs::create_dir_all(state_directory(work.as_path(), FAILED).as_path()).expect("the failed state");
    std::fs::rename(state_directory(work.as_path(), PENDING).join(id.as_str()), state_directory(work.as_path(), FAILED).join(id.as_str())).expect("the record moves to failed");
    let said: String = a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path(), started).expect("a filling whose record failed did not run to its end");
    assert!(said.contains(FAILED), "the sentence names where the record stands: {said}");
    assert!(said.contains("could not be filled"), "the sentence quotes the service's own reason: {said}");
    assert!(said.contains("stopped by signal 9"), "and how the clean ended: {said}");
    // The same record settled in `done` is a filling that ran to its end, and a control reads its bytes.
    std::fs::create_dir_all(state_directory(work.as_path(), DONE).as_path()).expect("the done state");
    std::fs::rename(state_directory(work.as_path(), FAILED).join(id.as_str()), state_directory(work.as_path(), DONE).join(id.as_str())).expect("the record settles");
    assert!(
        a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path(), started).is_none(),
        "a record that settled in done is a filling that ran to its end"
    );
}

/// What:
///  A filling the queue still holds is waited for, not called a failure:
///  a record in `held` that settles in `done` while the reader waits is a filling that ran.
/// Why:
///   A filling the drain is running stands in `held`, which the wait a commit control uses does not count, so a reader that
/// read that state as an answer would call a working filling a failure, which is the window the gate of
/// 2026-11-15T06:00-05:00 met.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// move(record, 'held'); after(200, () => move(record, 'done')); expect(reader()).toBeUndefined();
/// ```
#[test]
fn a_filling_the_queue_holds_is_waited_for_rather_than_called_a_failure() {
    let fixture: Fixture = fixture("filling-held-then-done");
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    std::fs::create_dir_all(git_dir.as_path()).expect("the repository's Git directory");
    let worktree: PathBuf = fixture.root.join("deeper/stale");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    let started: SystemTime = SystemTime::now();
    let item: WorkItem = WorkItem {
        kind: WorkKind::Prewarm,
        repository: worktree.clone(),
        git_dir: git_dir.clone(),
        branch_ref: String::from(git_policy_cli::package_repair_prewarm::PREWARM_REFERENCE),
        oid: String::from("89abcdef0123456789abcdef0123456789abcdef"),
        parent: String::from("[]"),
        created: git_policy_cli::iso_time::now_iso(),
        tries: 0,
    };
    let work: PathBuf = git_policy_cli::service_work::work_directory(git_dir.as_path());
    enqueue(work.as_path(), &item).expect("the record is queued");
    let id: String = item.id();
    std::fs::create_dir_all(state_directory(work.as_path(), HELD).as_path()).expect("the held state");
    std::fs::rename(state_directory(work.as_path(), PENDING).join(id.as_str()), state_directory(work.as_path(), HELD).join(id.as_str())).expect("the drain holds it");
    let settling: PathBuf = state_directory(work.as_path(), DONE);
    let held: PathBuf = state_directory(work.as_path(), HELD).join(id.as_str());
    let settler = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(200));
        std::fs::create_dir_all(settling.as_path()).expect("the done state");
        std::fs::rename(held.as_path(), settling.join(id.as_str())).expect("the record settles");
    });
    let said: Option<String> = a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path(), started);
    settler.join().expect("the thread that settles the record ends");
    assert!(said.is_none(), "a filling the queue held while it ran and settled in done did not fail: {said:?}");
}

/// What:
///  One worktree's prewarm record as the add hands it over,
///  with the stamp that makes it this filling's record and not an earlier one's.
/// Why:
///   The record's id is its object and branch, and its object is the worktree and the moment of the hand-over,
/// so a control that writes two records for one path writes them with two stamps and gets two ids.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const item = { kind: 'prewarm', repository: worktree, gitDir, ref: PREWARM_REFERENCE, oid: objectNameOf(worktree, created), created };
/// ```
fn a_prewarm_record(git_dir: &Path, worktree: &Path, created: &str) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Prewarm,
        repository: worktree.to_path_buf(),
        git_dir: git_dir.to_path_buf(),
        branch_ref: String::from(git_policy_cli::package_repair_prewarm::PREWARM_REFERENCE),
        oid: git_policy_cli::package_repair_prewarm::object_name_of(worktree, created),
        parent: String::from("[]"),
        created: String::from(created),
        tries: 0,
    };
}

/// What:
///  A fixture whose repository has a queue, a worktree to fill, and the service's line about that filling's failure,
///  with the record that names `created` in `state`.
/// Why:
///   Every control here is about what the reader makes of the queue alone,
/// so the queue is written by hand and nothing is run:
///  a control that started a service to produce these states would be a control of the service.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { fixture, gitDir, worktree, work } = aQueue(); writeRecord(work, 'failed', aPrewarmRecord(gitDir, worktree, created));
/// ```
fn a_queue(name: &str) -> (Fixture, PathBuf, PathBuf, PathBuf) {
    let fixture: Fixture = fixture(name);
    let git_dir: PathBuf = fixture.root.join("repo/.git");
    let worktree: PathBuf = fixture.root.join("deeper/stale");
    std::fs::create_dir_all(git_dir.as_path()).expect("the repository's Git directory");
    std::fs::create_dir_all(worktree.as_path()).expect("the worktree");
    let work: PathBuf = git_policy_cli::service_work::work_directory(git_dir.as_path());
    for state in [PENDING, RECOVERING, CLAIMED, HELD, DONE, FAILED] {
        std::fs::create_dir_all(state_directory(work.as_path(), state)).expect("a state of the queue");
    }
    std::fs::write(
        fixture.root.join(SERVICE_LOG_FILE),
        format!(
            "monochromatic-git-policy-cli-service: the Cargo build directory of {} could not be filled: the copy of {}/target.monochromatic-prewarm could not be cleaned (stopped by signal 9: ) at_ms=1\n",
            worktree.join("target").display(),
            worktree.display()
        ),
    )
    .expect("the service's log");
    return (fixture, git_dir, worktree, work);
}

/// What:
///  Put a record in `state` of the queue.
fn write_record(work: &Path, state: &str, item: &WorkItem) {
    enqueue(work, item).expect("the record is queued");
    let id: String = item.id();
    if state != PENDING {
        std::fs::rename(state_directory(work, PENDING).join(id.as_str()), state_directory(work, state).join(id.as_str())).expect("the record moves to its state");
    }
}

/// What:
///  A named pipe at `path`,
///  which nothing writes to.
/// Why:
///   A queue directory is a place anything with write access can put anything,
/// and an entry that is not a regular file is one a reader must not open:
/// opening a pipe for reading waits for a writer that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(path, 0o600); // no writer: a reader of it never returns
/// ```
fn a_fifo(path: &Path) {
    let text: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("a path without a NUL byte");
    // SAFETY: `mkfifo` takes a NUL-terminated path and a mode and touches no memory of this process.
    let made: libc::c_int = unsafe { libc::mkfifo(text.as_ptr(), 0o600) };
    assert_eq!(made, 0, "a named pipe at {}: {}", path.display(), std::io::Error::last_os_error());
}

/// What:
///  A queue entry that is not a regular file is not opened, so the reader answers within its bound.
/// Why:
///   A pipe with no writer keeps a reader that opens it for good,
/// and the deadline of this reader is looked at between looks,
/// so an opened pipe means a control hangs instead of failing
///  (the review of 2026-11-15, the first of its five findings:
///  with the harness patience shortened to 50 ms the reader was still blocked after 250 ms).
/// The reader is asked on a thread of its own so this control can fail rather than wait: a pipe is exactly the state in
/// which the answer never comes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkfifo(join(work, 'done/gate')); expect(await within(3s, read())).toBeDefined();
/// ```
#[test]
fn a_queue_entry_that_is_not_a_regular_file_does_not_keep_the_reader_waiting() {
    // The whole scenario is the reading thread's, because a fixture belongs to the thread that made it and a reader that
    //  never answers leaves that thread holding it.
    let (sender, answers) = std::sync::mpsc::channel::<Option<String>>();
    let reader: std::thread::JoinHandle<()> = std::thread::spawn(move || {
        let (fixture, git_dir, worktree, work) = a_queue("filling-with-a-pipe-in-the-queue");
        let started: SystemTime = SystemTime::now();
        a_fifo(state_directory(work.as_path(), DONE).join("gate").as_path());
        a_fifo(state_directory(work.as_path(), FAILED).join("gate").as_path());
        write_record(work.as_path(), FAILED, &a_prewarm_record(git_dir.as_path(), worktree.as_path(), git_policy_cli::iso_time::now_iso_never_the_same_twice().as_str()));
        let said: Option<String> = a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path(), started);
        sender.send(said).expect("the control receives the reader's answer");
    });
    let answered: Option<String> = answers
        .recv_timeout(A_READERS_ANSWER_PATIENCE)
        .unwrap_or_else(|_| { return panic!("the reader did not answer within {A_READERS_ANSWER_PATIENCE:?} although its bound is the harness's patience: a queue entry that is not a regular file is being opened"); });
    assert!(answered.is_some(), "the filling failed, so the reader answers with its reason: {answered:?}");
    assert!(answered.expect("the sentence").contains(FAILED), "the sentence names the state the record stands in");
    reader.join().expect("the reader ends");
}

/// What:
///  A record that settles to `done` while the look is reading the states is the answer the reader gives.
/// Why:
///   The queue moves a record from an open state to `done` or `failed` in one rename,
/// so a look that reads the settled states before the open ones can miss it entirely and call a healthy filling
///  "no state of the work directory".
/// The look is told each state as it reaches it, and this control settles the record exactly then,
/// which makes a moment that no wait could hit certain
///  (the review of 2026-11-15, the second of its five findings, reproduced with a pipe as the gate).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readWatching((state) => { if (state === 'claimed') settle(); }); expect(said).toBeUndefined();
/// ```
#[test]
fn a_record_that_settles_during_the_look_is_the_answer() {
    let (fixture, git_dir, worktree, work) = a_queue("filling-settled-during-the-look");
    let started: SystemTime = SystemTime::now();
    let item: WorkItem = a_prewarm_record(git_dir.as_path(), worktree.as_path(), git_policy_cli::iso_time::now_iso_never_the_same_twice().as_str());
    write_record(work.as_path(), CLAIMED, &item);
    let id: String = item.id();
    let claimed: PathBuf = state_directory(work.as_path(), CLAIMED).join(id.as_str());
    let done: PathBuf = state_directory(work.as_path(), DONE).join(id.as_str());
    let mut settled: bool = false;
    let mut looking = |state: &str| {
        // The service settles the record once, after the look has read a state it was not in and before the look reaches
        //  `claimed`, where it stands: a look that reads the settled states first misses it entirely, and one that reads the
        //  states in the order records move through them meets `done` after `claimed`.
        if state == RECOVERING && !settled {
            settled = true;
            std::fs::rename(claimed.as_path(), done.as_path()).expect("the record settles to done");
        }
    };
    let said: Option<String> = a_filling_that_did_not_run_to_its_end_watching(&fixture, git_dir.as_path(), worktree.as_path(), started, &mut looking);
    assert!(done.is_file(), "the record is in done, so the filling ran to its end");
    assert!(said.is_none(), "a filling that settled in done did not fail: {said:?}");
}

/// What:
///  An earlier record for the same worktree path does not decide what became of the current filling.
/// Why:
///   A worktree removed and added again at one path leaves the earlier filling's record in the queue,
/// and that record names the same path,
///  so a reader that takes any record of the path answers for the wrong fill:
/// an earlier `done` record hides a current failure,
///  and an earlier `failed` record makes a current healthy fill fail.
/// The record this control means is the one the add before it wrote, which its stamp tells from the earlier one's
///  (the review of 2026-11-15, the third of its five findings).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeOlder(done); writeCurrent(failed); expect(said).toContain('failed'); writeOlder(failed); writeCurrent(done); expect(said).toBeUndefined();
/// ```
#[test]
fn an_earlier_record_of_the_same_path_does_not_decide_the_current_filling() {
    let (fixture, git_dir, worktree, work) = a_queue("filling-with-an-earlier-record");
    let earlier: WorkItem = a_prewarm_record(git_dir.as_path(), worktree.as_path(), AN_EARLIER_FILLING);
    let started: SystemTime = SystemTime::now();
    let current: WorkItem = a_prewarm_record(git_dir.as_path(), worktree.as_path(), git_policy_cli::iso_time::now_iso_never_the_same_twice().as_str());
    assert_ne!(earlier.id(), current.id(), "two hand-overs of one path are two records");
    // An earlier filling that ran to its end, and the current one that failed.
    write_record(work.as_path(), DONE, &earlier);
    write_record(work.as_path(), FAILED, &current);
    let said: Option<String> = a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path(), started);
    let words: String = said.unwrap_or_else(|| { return panic!("the current filling failed, so the reader says so, and an earlier done record does not hide it"); });
    assert!(words.contains(FAILED), "the sentence names the current filling's state: {words}");
    assert!(words.contains("stopped by signal 9"), "and quotes the service's reason: {words}");
    // The other way round: an earlier failure, and a current filling that ran to its end.
    let later: WorkItem = a_prewarm_record(git_dir.as_path(), worktree.as_path(), git_policy_cli::iso_time::now_iso_never_the_same_twice().as_str());
    assert!(git_policy_cli::iso_time::instant_of_iso(later.created.as_str()).expect("a stamp") >= started, "the current filling's record is the one the add taken up at the moment wrote");
    std::fs::remove_file(state_directory(work.as_path(), FAILED).join(current.id().as_str())).expect("the previous control state goes");
    std::fs::remove_file(state_directory(work.as_path(), DONE).join(earlier.id().as_str())).expect("the previous control state goes");
    write_record(work.as_path(), FAILED, &earlier);
    write_record(work.as_path(), DONE, &later);
    let said: Option<String> = a_filling_that_did_not_run_to_its_end(&fixture, git_dir.as_path(), worktree.as_path(), started);
    assert!(said.is_none(), "the current filling ran to its end, and an earlier failed record does not make it fail: {said:?}");
}
