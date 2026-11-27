//! Controls for the notes of a commit whose standard error nobody drains (issue 6421):
//!  the command returns and the commit lands,
//!  and the lines it could not hand over are given up rather than waited for.
//! Why:
//!  A write to a pipe whose reader has stopped draining waits in the kernel until it resumes,
//!  and `std::io::stderr()` is one handle behind one lock for the whole process,
//!  so a note written on the commit's own thread can hold the commit for as long as the reader
//!  does not read.
//!  The owner's rule is "nothing should ever wait forever or indefinitely".
//! Every control runs the built executable on a disposable repository with the host's Git,
//!  with a standard error nobody reads:
//!  the read end of a full pipe kept open and never read,
//!  or a pipe whose read end is closed before the command starts.
//! The pipe is filled to its whole capacity, so not one byte of any line fits:
//!  a line the wrapper writes before the fix waits there and the commit never returns.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pipe = fullPipeNobodyDrains(); const run = spawn(wrapper, commit, { stderr: pipe.write });
//! expect(await waitBounded(run)).toBe(0); expect(pendingBytes(pipe)).toBe(pipe.capacity);
//! ```

use super::commit::{
    remove_settled,
    wrapped_commit,
};
use super::support::{
    Fixture,
    Observed,
    fixture,
    git,
    repository,
    run_wrapped,
};
/// The full pipe nobody drains, the closed pipe a reader left,
/// the bytes a pipe still holds and the bounded wait for a command given one,
///  which this control hands its commit instead of the pipes the shared reading drains.
use super::support_a_full_standard_error::{
    PipeNobodyDrains,
    pending_bytes,
    started_on_a_closed_pipe,
    started_on_a_full_pipe,
    wait_bounded,
};
use std::ffi::OsStr;
use std::os::fd::AsRawFd;
use std::path::{
    Path,
    PathBuf,
};
use std::process::{
    Child,
    Command,
    ExitStatus,
};
use std::time::Duration;

/// How long this control waits for the commit before it fails, in seconds.
/// Why:
///  The commit's own line ends it at `COMMIT_BOUND_SECONDS` (120 s),
///  so this bound is that one plus the grace in which a `SIGKILL`ed command is reaped;
///  a control that reached it fails with what it waited for rather than hanging the run.
const WAIT_SECONDS: u64 = 150;

/// What:
///  The text of one Git query in a repository,
///  run with real Git directly.
/// Why:
///  A control reads what landed from the repository itself, never from the wrapper's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitText = (fixture, repo, args) => exec(git, ['-C', repo, ...args]).stdout.trim();
/// ```
fn git_text(
    fixture: &Fixture,
    repository: &Path,
    arguments: &[&str],
) -> String {
    let output: std::process::Output = git(fixture, repository, arguments);
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// What:
///  A repository whose staged edit the pre-forward policies correct,
///  so the commit's own path writes a line to standard error.
/// Why:
///   Every note of the commit path must be given up when the reader is gone,
///  and a control with no note would prove only that a silent command is not held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aRepositoryWithAnEditThePoliciesCorrect = (fixture) => { ... write('a.txt', 'one\ntwo'); git add -- a.txt }
/// ```
fn a_repository_with_an_edit_the_policies_correct(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    std::fs::write(repo.join("a.txt"), b"one\n").expect("the tracked file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(
        fixture,
        repo.as_path(),
        &["commit", "--quiet", "--message=base"],
    );
    // The edit has no final newline, which the `final-newline` policy corrects and says so on standard
    //  error once the commit has landed.
    std::fs::write(repo.join("a.txt"), b"one\ntwo").expect("an edit without the final newline");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    return repo;
}

/// What:
///  A commit whose standard error is a full pipe nobody drains returns,
///  and the commit it was asked for is on the branch.
/// Why:
///   The notes of the commit path are lines a person can live without,
///  and the exit status already carries the result,
///  so no reader that stopped reading may hold the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('lands', () => { const pipe = fullPipe(); expect(waitBounded(spawn(commit, { stderr: pipe.write }))).toBe(0); expect(pendingBytes(pipe)).toBe(pipe.capacity) })
/// ```
#[test]
fn a_commit_lands_while_nobody_drains_its_full_standard_error() {
    let fixture: Fixture = fixture("commit-stalled-standard-error");
    let repo: PathBuf = a_repository_with_an_edit_the_policies_correct(&fixture);
    let mut command: Command = wrapped_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--message=stalled", "a.txt"],
    );
    // What: The command's standard error is the full pipe, and its other two streams go nowhere.
    // Why:  A captured pipe of this control's own would be drained by the wait, which is the reader
    //       this control removes.
    let (child, pipe): (Child, PipeNobodyDrains) = started_on_a_full_pipe(&mut command);
    let status: ExitStatus = wait_bounded(child, Duration::from_secs(WAIT_SECONDS), "the commit");
    assert_eq!(
        status.code(),
        Some(0),
        "the commit returned without landing: {status:?}"
    );
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        "stalled",
        "the commit landed"
    );
    assert_eq!(
        pending_bytes(pipe.read_end.as_raw_fd()),
        pipe.capacity,
        "the pipe was read by nobody, so every byte this control put in it is still there"
    );
    remove_settled(&fixture);
}

/// What:
///  The failed-push notice the command before this one could not say is shown once by this one,
///  and is gone after it.
/// Why:
///   A record is the only statement a failed push leaves,
///  so a command that could not say it must leave it where the next command finds it,
///  and a record shown twice would be noise on every command after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const first = runWrapped(fixture, repo, ['status', '--short']); expect(first.stderr).toContain('auto-push to origin failed'); const second = runWrapped(again); expect(second.stderr).not.toContain('auto-push to origin failed')
/// ```
fn the_notice_is_shown_once_by_the_command_after(
    fixture: &Fixture,
    repo: &Path,
) {
    let first: Observed = run_wrapped(fixture, repo, &["status", "--short"]);
    let said: String = String::from_utf8_lossy(first.stderr.as_slice()).into_owned();
    assert!(
        said.contains("auto-push to origin failed"),
        "the record the command before this one could not say is said now: {said}"
    );
    let second: Observed = run_wrapped(fixture, repo, &["status", "--short"]);
    let again: String = String::from_utf8_lossy(second.stderr.as_slice()).into_owned();
    assert!(
        !again.contains("auto-push to origin failed"),
        "a record whose words reached the person is taken: {again}"
    );
}

/// What:
///  Records one failed-push notice for the worktree whose commit comes next.
/// Why:
///   Every wrapped command shows the notices waiting for it before it runs,
///  and a commit's own thread showing one is the write this file's controls are about.
///  The record is made through the writer the service uses,
///  so the notice it words is the one a person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordPushFailure(`${repo}/.git`, 'refs/heads/main', head, 'refused', 'the remote said no');
/// ```
fn a_recorded_failed_push_notice(
    repo: &Path,
    commit: &str,
) {
    git_policy_cli::push_failure_notice::record_push_failure(
        repo.join(".git").as_path(),
        "refs/heads/main",
        commit,
        git_policy_cli::push_failure_notice::PushOutcome::Refused,
        "the remote said no",
    )
    .expect("one failed-push notice for the worktree");
}

/// What:
///  A commit one recorded failed-push notice waits for,
///  whose standard error is a pipe whose reader has gone,
///  returns, lands the commit,
///  and leaves that notice's record for the next command.
/// Why:
///   Every write to a pipe whose reader ended is refused with `EPIPE`,
///  which is the case a harness that closes its reader leaves,
///  and the writer thread is the only place that knows the line was refused:
///  a write that answered before the thread wrote would take the record with the line.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('closed', () => { recordPushFailure(head); expect(waitBounded(spawn(commit, { stderr: closedPipe() }))).toBe(0); expect(log('-1')).toBe('closed-reader'); expect(status().stderr).toContain('auto-push to origin failed') })
/// ```
#[test]
fn a_record_whose_line_a_closed_reader_refused_stays_for_the_next_command() {
    let fixture: Fixture = fixture("commit-closed-standard-error-notice");
    let repo: PathBuf = a_repository_with_an_edit_the_policies_correct(&fixture);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    a_recorded_failed_push_notice(repo.as_path(), head.trim());
    let mut command: Command = wrapped_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--message=closed-reader", "a.txt"],
    );
    let child: Child = started_on_a_closed_pipe(&mut command);
    let status: ExitStatus = wait_bounded(child, Duration::from_secs(WAIT_SECONDS), "the commit");
    assert_eq!(
        status.code(),
        Some(0),
        "the commit returned although no program read its standard error: {status:?}"
    );
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        "closed-reader",
        "the commit landed"
    );
    the_notice_is_shown_once_by_the_command_after(&fixture, repo.as_path());
    remove_settled(&fixture);
}

/// What:
///  A commit one recorded failed-push notice waits for,
///  whose standard error is a full pipe nobody drains,
///  returns, lands the commit,
///  and leaves that notice's record for the next command,
///  which shows it once.
/// Why:
///   The notice is shown before the commit runs,
///  so it used to be written on the commit's own thread,
///  and a reader that stopped draining held the commit for ever (issue 6421).
///  The line itself may be one a person can live without,
///  but the record behind it is not:
///  it is the only trace that a push did not land,
///  so it goes only once the thread that writes standard error says the line reached the person.
///  A line still waiting when the command gave up is not in the stream,
///  so the record stays, and the next command whose reader is there shows it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// test('notice', () => { recordPushFailure(head); const pipe = fullPipe(); expect(waitBounded(spawn(commit, { stderr: pipe.write }))).toBe(0); expect(log('-1')).toBe('stalled-notice'); expect(status().stderr).toContain('auto-push to origin failed'); expect(status().stderr).not.toContain('auto-push to origin failed') })
/// ```
#[test]
fn a_recorded_notice_holds_nobody_while_the_commit_lands_and_its_record_stays() {
    let fixture: Fixture = fixture("commit-stalled-standard-error-notice");
    let repo: PathBuf = a_repository_with_an_edit_the_policies_correct(&fixture);
    let head: String = git_text(&fixture, repo.as_path(), &["rev-parse", "HEAD"]);
    a_recorded_failed_push_notice(repo.as_path(), head.trim());
    let mut command: Command = wrapped_commit(
        &fixture,
        repo.as_path(),
        &["commit", "--message=stalled-notice", "a.txt"],
    );
    let (child, pipe): (Child, PipeNobodyDrains) = started_on_a_full_pipe(&mut command);
    let status: ExitStatus = wait_bounded(child, Duration::from_secs(WAIT_SECONDS), "the commit");
    assert_eq!(
        status.code(),
        Some(0),
        "the commit returned without landing: {status:?}"
    );
    assert_eq!(
        git_text(&fixture, repo.as_path(), &["log", "-1", "--format=%s"]).trim(),
        "stalled-notice",
        "the commit landed"
    );
    assert_eq!(
        pending_bytes(pipe.read_end.as_raw_fd()),
        pipe.capacity,
        "the pipe was read by nobody, so every byte this control put in it is still there"
    );
    the_notice_is_shown_once_by_the_command_after(&fixture, repo.as_path());
    remove_settled(&fixture);
}
