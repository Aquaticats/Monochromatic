//! What:
//!  Controls for the two reads of a branch's gate records that a push makes,
//!  and the measurement of what reading them off the runtime's worker costs.
//! Why:
//!   A push is a task of the runtime,
//! and these two reads list and read the worktree's `blocked` and `held` records with `std::fs`:
//! a disk or a mount that stopped answering held the worker the runtime needs,
//! and on the current-thread runtime the build falls back to it held the whole service.
//! The records must be read off the worker and within a ceiling,
//! and a read that reaches the ceiling must end that push's round with an answer the next look can act on:
//! the commit is neither lost nor reported as made,
//! and the service's own retry plan tries it again
//! (`gate_records_read_off_the_runtime.rs`).
//!  A record whose read does not return is a FIFO at a record's name in a disposable fixture,
//! held open by a writer this control owns and closed when the control lets it go:
//! it shows a read that never answers and what the push does then,
//! and nothing about a mount whose server stopped,
//! which needs a mount of its own to show.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! using record = aRecordThatDoesNotAnswer(join(work, 'blocked', id)); const wait = round(gitDir); expect(elapsed).toBeLessThan(CEILING + SLACK)
//! ```

#![cfg(unix)]

use super::*;
use crate::parallel_budget::HostResources;
use crate::push_retry::PushRetries;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use crate::work_item::{WorkKind, WorkItem, enqueue, state_directory};
use crate::work_item_holding::BLOCKED;
use crate::service_work::{GitHandlers, work_directory};
use crate::service_work_run_round::run_planned_round;
use crate::claim_time::ClaimTime;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The ceiling the controls give a gate record's read,
/// short enough that a read this control holds open cannot answer inside it,
/// and long enough that a read of a quiet disk always does.
const THE_CONTROLS_CEILING: Duration = Duration::from_millis(200);

/// How much longer than the ceiling a round may take and still be one that ended at it:
/// the place, the thread and the settling of an ordinary push on this host,
/// measured well under a second.
const THE_CONTROLS_SLACK: Duration = Duration::from_secs(2);

/// The first pause before the push is tried again, in the controls.
const PAUSE: Duration = Duration::from_millis(800);

/// The words every control's pushes are given,
/// so a push that asks a terminal for a password fails instead of waiting for one.
const NO_PROMPT: [(&str, &str); 2] = [("GIT_TERMINAL_PROMPT", "0"), ("GIT_ASKPASS", "/bin/true")];

/// What:
///  A repository tracking a bare remote of its own,
/// with one commit landed and its push queued,
/// which is the state one round of the service acts on.
/// Why:
///   A push's gate records are read after Git answered which commit the branch names,
/// so the control needs real Git, a real remote and a real queued item,
/// and every fixture has its own bare remote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stage = stage('some-name'); // repo + remote + enqueued push item
/// ```
struct Stage {
    /// The fixture directory, removed when the control ends.
    root: PathBuf,
    /// The working repository.
    repo: PathBuf,
    /// The bare remote named `origin`.
    remote: PathBuf,
    /// The commit the queued push is owed for.
    landed: String,
}

/// What:
///  The push record of one landed commit of `repo`.
/// Why:
///   A record's id is its file name, so a control that plants a FIFO at a record's name plants it at this one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const item = { kind: 'push', repository: root, gitDir: join(repo, '.git'), branchRef: 'refs/heads/main', oid: landed, parent: '', created: '', tries: 0 };
/// ```
fn push_item(stage: &Stage) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: stage.repo.clone(),
        git_dir: stage.repo.join(".git"),
        branch_ref: String::from("refs/heads/main"),
        oid: stage.landed.clone(),
        parent: String::new(),
        created: String::from("2026-11-14T00:00:00.000Z"),
        tries: 0,
    };
}

/// What:
///  The stage of one control:
///  a repository, a bare remote, the commit the remote already holds,
///  and the record of the push it is owed.
/// Why:
///   The remote's own ref file is what every control reads to learn whether a commit reached it,
/// and the setup's own push is the positive control that this reading sees one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// git init --bare remote.git; git remote add origin remote.git; commit; push --set-upstream; commit; enqueue(pushItem)
/// ```
fn stage(name: &str) -> Stage {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    let remote: PathBuf = root.join("remote.git");
    git(root.as_path(), &["init", "--bare", "--quiet", remote.to_str().expect("utf-8")]);
    git(repo.as_path(), &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("file");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=one"]);
    git(repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let base: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    std::fs::write(repo.join("a.txt"), b"two\n").expect("file");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=two"]);
    let landed: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]).trim().to_owned();
    let queued: Stage = Stage { root, repo, remote, landed };
    enqueue(work_directory(queued.repo.join(".git").as_path()).as_path(), &push_item(&queued)).expect("the push record");
    assert_eq!(tip_of(queued.remote.as_path()), base, "the control's reading of the remote's ref shows the push the setup made");
    return queued;
}

/// What:
///  The branch tip a bare remote holds,
///  read from its own ref file.
/// Why:
///   A control waits for a push by asking this,
/// and starting Git for it costs a process each look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(join(remote, 'refs/heads/main'), 'utf8').trim()
/// ```
fn tip_of(remote: &Path) -> String {
    return std::fs::read_to_string(remote.join("refs").join("heads").join("main")).map(|text| { return text.trim().to_owned(); }).unwrap_or_default();
}

/// How many records of a push that did not land this repository holds for the person.
fn notices(stage: &Stage) -> usize {
    return std::fs::read_dir(crate::push_failure_notice::notice_directory(stage.repo.join(".git").as_path())).map_or(0, |entries| { return entries.flatten().count(); });
}

/// What the person was told about this repository's pushes,
/// every record of them in one text.
fn told(stage: &Stage) -> String {
    return crate::test_support::shown_notices(crate::push_failure_notice::push_failures(stage.repo.join(".git").as_path())).join("\n");
}

/// One round over the stage's Git directory,
/// the way the service's work loop runs it,
/// answering when to look at the repository again.
fn round_of(stage: &Stage, retries: &mut PushRetries) -> Option<Duration> {
    let handlers: Arc<GitHandlers> = Arc::new(GitHandlers::new(git_context()).with_push_environment(&NO_PROMPT));
    let git_dir: PathBuf = stage.repo.join(".git");
    let asked: Vec<(PathBuf, Duration)> = run_planned_round(&HostResources::read(), std::slice::from_ref(&git_dir), &handlers, retries, &ClaimTime::read());
    return asked.into_iter().find(|(directory, _)| { return *directory == git_dir; }).map(|(_, wait)| { return wait; });
}

/// What:
///  An empty state directory of the stage's work tree,
///  made so a control can plant a record in it.
/// Why:
///   The reads list the directory before they open anything,
/// so the directory the record stands in must be there for the listing to answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mkdirSync(join(work, 'held'), { recursive: true })
/// ```
fn a_state_directory(stage: &Stage, state: &str) -> PathBuf {
    let directory: PathBuf = state_directory(work_directory(stage.repo.join(".git").as_path()).as_path(), state);
    std::fs::create_dir_all(directory.as_path()).expect("the state directory");
    return directory;
}

/// What:
///  `path` as one pipe of this fixture,
///  in the fixture's own directory.
/// Why:
///   `mkfifo` is the only way to make one;
/// the path is handed over as the raw bytes the file system takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = Buffer.from(path); if (mkfifo(name, 0o600) !== 0) throw new Error(String(errno))
/// ```
fn make_pipe(path: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let name: std::ffi::CString = std::ffi::CString::new(path.as_os_str().as_bytes()).expect("a path without a NUL byte");
    // SAFETY:
    // `name` is a live, NUL-terminated C string for the whole call,
    // and `mkfifo` reads it and nothing else;
    // its failure is inspected through the error it left.
    let made: libc::c_int = unsafe { libc::mkfifo(name.as_ptr(), 0o600) };
    assert_eq!(made, 0, "a pipe at {}: {}", path.display(), std::io::Error::last_os_error());
}

/// What:
///  A record whose read does not return:
/// a FIFO at `record`, and a writer of this control's own that holds it open,
/// both let go when this value is dropped.
/// Why:
///   A gate record that does not answer is what a mount that stopped answering does to a read,
/// and a FIFO is the one way to make that in a fixture this test can end by itself:
/// the read opens the pipe and waits for its writer,
/// the writer writes nothing,
///  and closing it answers the read with the end of its input.
/// The writer gives up after `PATIENCE` as well,
/// so a control that fails before its own end leaves no thread waiting on this pipe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { writer, letGo } = aRecordThatDoesNotAnswer(join(blocked, id)); // letGo() closes the pipe and removes the record
/// ```
struct ARecordThatDoesNotAnswer {
    /// The record's path,
    /// which is a pipe while this value lives.
    record: PathBuf,
    /// Lets the writer go,
    /// which answers the read with the end of its input.
    let_go: Option<std::sync::mpsc::Sender<()>>,
    /// The writer's own thread.
    writer: Option<std::thread::JoinHandle<()>>,
    /// Told once the writer holds the pipe open,
    /// so the control knows the read has opened it too.
    opened: std::sync::mpsc::Receiver<()>,
}

impl ARecordThatDoesNotAnswer {
    /// What:
    ///  Plant the pipe at `record` and start the writer that holds it open.
    /// Why:
    ///   The writer's open completes only when a reader opens the pipe,
    /// so the plant is answered as soon as the read a control waits for has begun.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// mkfifo(record); writer = spawn(() => { const file = open(record, 'w'); opened(); recv(); close(file) });
    /// ```
    fn at(record: &Path) -> ARecordThatDoesNotAnswer {
        make_pipe(record);
        let (told, opened) = std::sync::mpsc::channel::<()>();
        let (let_go, waits) = std::sync::mpsc::channel::<()>();
        let held_open: PathBuf = record.to_path_buf();
        let writer: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            let Ok(file) = std::fs::OpenOptions::new().write(true).open(held_open.as_path()) else { return };
            let _told = told.send(());
            let _held = waits.recv_timeout(crate::parallel_test_support::PATIENCE);
            drop(file);
        });
        return ARecordThatDoesNotAnswer { record: record.to_path_buf(), let_go: Some(let_go), writer: Some(writer), opened };
    }

    /// What:
    ///  Whether the read this control waits for has opened the pipe,
    ///  without waiting for it.
    /// Why:
    ///   A control that asserts the pipe was opened says the round reached the record,
    /// which is what makes the elapsed time its wait rather than a push that never got there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const reached = tryRecv(opened)
    /// ```
    fn reached(&self) -> bool {
        return self.opened.try_recv().is_ok();
    }

    /// What:
    ///  End the record:
    ///  take its name away and let the writer go,
    ///  so the read that waited returns and the next round finds no such record.
    /// Why:
    ///   A pipe with no writer left is one a later read would wait on again,
    /// so the record goes with the writer that held it open,
    /// and the next round reads a directory with nothing in it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// unlinkSync(record); letGo(); join(writer)
    /// ```
    fn answer(mut self) {
        let _removed = std::fs::remove_file(self.record.as_path());
        self.let_go_from_the_writer();
    }

    /// Lets the writer go and waits for it,
    /// which the end of this value needs too.
    fn let_go_from_the_writer(&mut self) {
        drop(self.let_go.take());
        if let Some(writer) = self.writer.take() {
            let _joined = writer.join();
        }
    }
}

impl Drop for ARecordThatDoesNotAnswer {
    fn drop(&mut self) {
        let _removed = std::fs::remove_file(self.record.as_path());
        self.let_go_from_the_writer();
    }
}

/// What:
///  A gate record that does not answer ends the push's round within the ceiling,
///  nothing of the branch is sent,
///  the person is told,
///  the push is kept for the next try,
///  and the round after the record answers delivers it.
/// Why:
///   The two reads sit on the path of every push,
/// and a record that cannot be read may be the refusal that keeps this commit local:
/// a push decided from an empty list would publish it.
/// Without a ceiling the round waits for as long as the storage does,
/// and the claim's renewal keeps the item this instance's,
/// so nothing else acts on it either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using record = aRecordThatDoesNotAnswer(join(work, 'blocked', id)); const wait = round(); expect(elapsed < CEILING + SLACK); expect(tip).toBe(base); record.answer(); sleep(wait); round(); expect(tip).toBe(landed)
/// ```
#[test]
fn a_verdict_record_that_does_not_answer_ends_the_round_within_the_ceiling_and_the_push_is_tried_again() {
    // A verdict the gate kept stops the push before the commit it names, so its record is the one whose read a push may not
    //  mistake for "no verdict".
    let stage: Stage = stage("gate-record-does-not-answer-blocked");
    let record: PathBuf = a_state_directory(&stage, BLOCKED).join(push_item(&stage).id());
    let unanswered: ARecordThatDoesNotAnswer = ARecordThatDoesNotAnswer::at(record.as_path());
    let _ceiling = hold_the_gate_record_reads_to(THE_CONTROLS_CEILING);
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let began: Instant = Instant::now();
    let asked_again: Option<Duration> = round_of(&stage, &mut retries);
    let took: Duration = began.elapsed();
    assert!(unanswered.reached(), "the round reached the record, so what it waited for was that record's read");
    assert!(took < THE_CONTROLS_CEILING + THE_CONTROLS_SLACK, "the round ended within the ceiling and not when the record answered: it took {took:?}");
    assert_ne!(tip_of(stage.remote.as_path()), stage.landed, "nothing of the branch was sent: a verdict nobody read may be the one that keeps this commit local");
    assert_eq!(notices(&stage), 1, "the person is told the push did not land");
    let said: String = told(&stage);
    assert!(said.contains("were not read") || said.contains("gave no answer"), "the record says what did not answer: {said}");
    assert!(said.contains("Nothing of this branch was sent"), "the record says nothing was sent: {said}");
    assert!(said.contains("tries this push again"), "the record says the service tries the push again, and when: {said}");
    assert!(!retries.directories().is_empty(), "the retry plan keeps the push for the next try");
    let next_look: Duration = asked_again.expect("the service says when to look at the repository again");
    assert!(next_look <= PAUSE + THE_CONTROLS_SLACK, "the wake is for the retry: {next_look:?}");
    // The record answers, and the round the plan asked for delivers the commit that was kept.
    unanswered.answer();
    std::thread::sleep(next_look);
    let _after: Option<Duration> = round_of(&stage, &mut retries);
    assert_eq!(tip_of(stage.remote.as_path()), stage.landed, "the round after the record answered sent the commit that was kept");
    assert_eq!(notices(&stage), 0, "the record of the failure is gone once the push landed");
    remove(stage.root.as_path());
}

/// What:
///  A held record that does not answer ends the push's round the same way.
/// Why:
///   The push reads the held records twice on its path:
/// once for a refusal a held record still states,
/// and once for the commits of the branch whose gate has not decided.
/// A held record is the only statement that a commit may not be published yet,
/// so a push that sent past a held record it never read would publish a commit whose gate still runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using record = aRecordThatDoesNotAnswer(join(work, 'held', id)); const wait = round(); expect(elapsed < CEILING + SLACK); expect(tip).toBe(base); record.answer(); sleep(wait); round(); expect(tip).toBe(landed)
/// ```
#[test]
fn a_held_record_that_does_not_answer_ends_the_round_within_the_ceiling_and_the_push_is_tried_again() {
    let stage: Stage = stage("gate-record-does-not-answer-held");
    let record: PathBuf = a_state_directory(&stage, crate::work_item::HELD).join(push_item(&stage).id());
    let unanswered: ARecordThatDoesNotAnswer = ARecordThatDoesNotAnswer::at(record.as_path());
    let _ceiling = hold_the_gate_record_reads_to(THE_CONTROLS_CEILING);
    let mut retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let began: Instant = Instant::now();
    let asked_again: Option<Duration> = round_of(&stage, &mut retries);
    let took: Duration = began.elapsed();
    assert!(unanswered.reached(), "the round reached the record, so what it waited for was that record's read");
    assert!(took < THE_CONTROLS_CEILING + THE_CONTROLS_SLACK, "the round ended within the ceiling and not when the record answered: it took {took:?}");
    assert_ne!(tip_of(stage.remote.as_path()), stage.landed, "nothing of the branch was sent: a gate nobody read may still hold this commit");
    assert_eq!(notices(&stage), 1, "the person is told the push did not land");
    let said: String = told(&stage);
    assert!(said.contains("were not read") || said.contains("gave no answer"), "the record says what did not answer: {said}");
    assert!(said.contains("tries this push again"), "the record says the service tries the push again, and when: {said}");
    assert!(!retries.directories().is_empty(), "the retry plan keeps the push for the next try");
    let next_look: Duration = asked_again.expect("the service says when to look at the repository again");
    assert!(next_look <= PAUSE + THE_CONTROLS_SLACK, "the wake is for the retry: {next_look:?}");
    unanswered.answer();
    std::thread::sleep(next_look);
    let _after: Option<Duration> = round_of(&stage, &mut retries);
    assert_eq!(tip_of(stage.remote.as_path()), stage.landed, "the round after the record answered sent the commit that was kept");
    assert_eq!(notices(&stage), 0, "the record of the failure is gone once the push landed");
    remove(stage.root.as_path());
}

/// What:
///  The read of the stored verdicts is a job of the blocking pool awaited within the ceiling,
///  even on the current-thread runtime the build falls back to.
/// Why:
///   That runtime has one thread, so a read made on it holds the service itself;
/// the answer is the ceiling's words and not the record's,
///  and it comes within the ceiling rather than when the record answers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using record = aRecordThatDoesNotAnswer(join(work, 'blocked', id)); const began = now(); const answer = await blockedVerdictsWithinAwaiting(CEILING, work); expect(answer).toBeErr(); expect(elapsed).toBeLessThan(CEILING + SLACK)
/// ```
#[test]
fn the_verdict_records_are_read_off_the_runtime_and_within_the_ceiling() {
    let root: PathBuf = fixture("gate-records-read-off-the-runtime-blocked");
    let work: PathBuf = root.join("work");
    let blocked: PathBuf = state_directory(work.as_path(), BLOCKED);
    std::fs::create_dir_all(blocked.as_path()).expect("the blocked directory");
    // The current-thread runtime is the one the build falls back to when the system refuses threads; it has no other worker for a
    //  read to be carried by, so a read that ran on its task would hold this whole call. It is made before the record, so a control
    //  that fails drops the record and closes its pipe before this runtime waits for the job that read it.
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
    let record: PathBuf = blocked.join(push_item_of(&root).id());
    let unanswered: ARecordThatDoesNotAnswer = ARecordThatDoesNotAnswer::at(record.as_path());
    let began: Instant = Instant::now();
    let answered = runtime.block_on(blocked_verdicts_within_awaiting(THE_CONTROLS_CEILING, work.as_path()));
    let took: Duration = began.elapsed();
    assert!(unanswered.reached(), "the read opened the record, so what it waited for was that record");
    assert!(took < THE_CONTROLS_CEILING + THE_CONTROLS_SLACK, "the read was given up at the ceiling and not when the record answered: it took {took:?}");
    let said: String = answered.expect_err("a record that did not answer leaves the verdicts unread");
    assert!(said.contains("gave no answer within 200ms"), "the words name the ceiling the read reached: {said}");
    unanswered.answer();
    let after = runtime.block_on(blocked_verdicts_within_awaiting(THE_CONTROLS_CEILING, work.as_path()));
    assert_eq!(after, Ok(Vec::new()), "the record answered, so the verdicts are read: no record is left to state one");
    remove(root.as_path());
}

/// What:
///  The read of the commits whose gate still runs is a job of the blocking pool awaited within the ceiling too.
/// Why:
///   It is the other read on the push's path,
/// and a push that sends past a commit whose gate has not decided publishes a commit the gate may keep local.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using record = aRecordThatDoesNotAnswer(join(work, 'held', id)); const answer = await undecidedCommitsWithinAwaiting(CEILING, work, branch); expect(answer).toBeErr()
/// ```
#[test]
fn the_held_records_are_read_off_the_runtime_and_within_the_ceiling() {
    let root: PathBuf = fixture("gate-records-read-off-the-runtime-held");
    let work: PathBuf = root.join("work");
    let held: PathBuf = state_directory(work.as_path(), crate::work_item::HELD);
    std::fs::create_dir_all(held.as_path()).expect("the held directory");
    // Made before the record, so a control that fails closes the record's pipe before this runtime waits for the job that read it.
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().expect("a runtime for the control");
    let record: PathBuf = held.join(push_item_of(&root).id());
    let unanswered: ARecordThatDoesNotAnswer = ARecordThatDoesNotAnswer::at(record.as_path());
    let began: Instant = Instant::now();
    let answered = runtime.block_on(undecided_commits_within_awaiting(THE_CONTROLS_CEILING, work.as_path(), "refs/heads/main"));
    let took: Duration = began.elapsed();
    assert!(unanswered.reached(), "the read opened the record, so what it waited for was that record");
    assert!(took < THE_CONTROLS_CEILING + THE_CONTROLS_SLACK, "the read was given up at the ceiling and not when the record answered: it took {took:?}");
    let said: String = answered.expect_err("a record that did not answer leaves the undecided commits unread");
    assert!(said.contains("gave no answer within 200ms"), "the words name the ceiling the read reached: {said}");
    unanswered.answer();
    let after = runtime.block_on(undecided_commits_within_awaiting(THE_CONTROLS_CEILING, work.as_path(), "refs/heads/main"));
    assert_eq!(after, Ok(Vec::new()), "the record answered and names no undecided commit");
    remove(root.as_path());
}

/// What:
///  The push record of a work tree that has no repository behind it,
///  which is all a record's name needs.
/// Why:
///   The two controls above plant a pipe at a record's name and read the directory,
/// so nothing of them runs Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const item = { kind: 'push', repository: root, gitDir: root, branchRef: 'refs/heads/main', oid: OID, parent: '', created: '', tries: 0 };
/// ```
fn push_item_of(root: &Path) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: root.to_path_buf(),
        git_dir: root.to_path_buf(),
        branch_ref: String::from("refs/heads/main"),
        oid: String::from("0123456789abcdef0123456789abcdef01234567"),
        parent: String::new(),
        created: String::new(),
        tries: 0,
    };
}

/// What:
///  A push whose gate records are readable is held or sent exactly as it was before the reads were made jobs of the pool.
/// Why:
///   The two reads answer the same things either way:
/// a verdict the gate kept stops the push before the commit it names,
/// and a push with no verdict against it sends the branch.
/// A control that only showed the ceiling would not show that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const sent = stageWithANoOpVerdict(); round(sent); expect(tipOf(sent.remote)).toBe(sent.landed); const heldBack = stageWithAVerdictOnItsTip(); round(heldBack); expect(tipOf(heldBack.remote)).not.toBe(heldBack.landed)
/// ```
#[test]
fn a_push_with_readable_gate_records_is_sent_or_held_back_as_it_was() {
    let sent: Stage = stage("gate-records-readable-one-sent");
    let other: WorkItem = WorkItem { branch_ref: String::from("refs/heads/other"), oid: String::from("1111111111111111111111111111111111111111"), ..push_item(&sent) };
    crate::work_item::hold(work_directory(sent.repo.join(".git").as_path()).as_path(), &other, "4321").expect("a held record of another branch");
    let refused_elsewhere: WorkItem = WorkItem { branch_ref: String::from("refs/heads/other"), oid: String::from("2222222222222222222222222222222222222222"), ..push_item(&sent) };
    let work_sent: PathBuf = work_directory(sent.repo.join(".git").as_path());
    crate::work_item::hold(work_sent.as_path(), &refused_elsewhere, "4321").expect("a held record the gate refused");
    crate::work_item_holding::block(work_sent.as_path(), &refused_elsewhere, "a check of another branch refused it");
    let mut sent_retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let _one_round: Option<Duration> = round_of(&sent, &mut sent_retries);
    assert_eq!(tip_of(sent.remote.as_path()), sent.landed, "a push whose records name other commits is sent");
    assert_eq!(notices(&sent), 0, "a push that landed leaves no record for the person");

    let held_back: Stage = stage("gate-records-readable-one-held-back");
    let refused: WorkItem = push_item(&held_back);
    let work_held: PathBuf = work_directory(held_back.repo.join(".git").as_path());
    crate::work_item::hold(work_held.as_path(), &refused, "4321").expect("the push is held while the gate decides");
    crate::work_item_holding::block(work_held.as_path(), &refused, "a check of this commit refused it");
    let mut held_back_retries: PushRetries = PushRetries::with_first_pause(PAUSE);
    let _another_round: Option<Duration> = round_of(&held_back, &mut held_back_retries);
    assert_ne!(tip_of(held_back.remote.as_path()), held_back.landed, "a verdict on this commit keeps the push local");
    let said: String = told(&held_back);
    assert!(said.contains("a check of this commit refused it"), "the person is told what the gate said: {said}");
    remove(sent.root.as_path());
    remove(held_back.root.as_path());
}

//region What the two reads cost a push, measured against the reads made on the worker

// What:
//  What the two reads of a push's gate records cost on this path,
//  as the ratio of the arm that awaits them on the pool to the arm that makes them in the push's task.
// Why:
//   `wait_ceilings.rs`'s `AFTER_COMMIT_STATE_CEILING` and this module's own ceiling cite the wait they grant,
// and the change that bounded these reads cites the ratio below:
// the ceiling buys a free worker for the cost of a place of the process's bound and the hop to it.
// Run:
// `mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture the_two_reads_a_push_makes_cost_this_on_this_machine`.
//
//  Measured on 2026-11-16 on this host,
// three runs, 400 rounds each, the two arms interleaved round by round,
// reading a work tree that holds ten gate records:
// five verdicts, and five commits of the pushed branch whose gate still runs.
// - in the push's own task: 712 to 722 us at the middle, 957 to 1,094 us at the 99th percentile,
//   and 1,097 to 1,128 us at the slowest;
// - awaited as jobs of the pool: 768 to 787 us at the middle, 1,113 to 1,166 us at the 99th percentile,
//   and 1,417 to 1,844 us at the slowest;
// - the ratio of the two middles: 1.08 to 1.09,
//   and of the two 99th percentiles 1.07 to 1.17.
// The control arm's own middle moved 1.4% between runs,
// so the difference is the change and not the host's load.
// The middle is what a reader compares:
// the largest samples of both arms move with this host's load and say nothing about the change.

/// How many rounds of the measurement,
/// each reading the same handful of records once per arm,
/// so each middle is worth comparing.
const ROUNDS: usize = 400;

/// How many records of each state the measured work tree holds:
/// a handful, which is what a push of an ordinary branch reads.
const RECORDS_OF_EACH_STATE: usize = 5;

/// The ceiling the bounded arm is given:
/// far past what a read that answers takes,
/// so the arm measures the step and never its own ceiling.
const GENEROUS: Duration = Duration::from_secs(30);

/// What:
///  A work tree holding a handful of gate records of both states,
///  the one every round of the measurement reads.
/// Why:
///   The cost of the change is what the pool's place and thread add to reading these files,
///  so the work belongs to the fixture and never to the round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let n = 0; n < RECORDS; n++) { hold(work, item(n)); block(work, item(n)); }
/// ```
fn a_work_tree_with_a_handful_of_gate_records(root: &Path) -> PathBuf {
    let work: PathBuf = root.join("work");
    let held: PathBuf = state_directory(work.as_path(), crate::work_item::HELD);
    let blocked: PathBuf = state_directory(work.as_path(), BLOCKED);
    std::fs::create_dir_all(held.as_path()).expect("the held directory");
    std::fs::create_dir_all(blocked.as_path()).expect("the blocked directory");
    for number in 0..RECORDS_OF_EACH_STATE {
        // A verdict a gate kept, on a branch of its own: its record is what the push reads first.
        let refused: WorkItem = an_item_of(root, format!("refs/heads/topic-{number}"), format!("{number:040x}"));
        crate::work_item::hold(work.as_path(), &refused, "4321").expect("a held record");
        crate::work_item_holding::block(work.as_path(), &refused, "a check of that commit refused it");
        // A commit of the pushed branch whose gate still runs: its record is what the push reads second, and it stays held.
        let undecided: WorkItem = an_item_of(root, String::from("refs/heads/main"), format!("{:040x}", RECORDS_OF_EACH_STATE + number));
        crate::work_item::hold(work.as_path(), &undecided, "4321").expect("a held record whose gate still runs");
    }
    return work;
}

/// The push record of one commit of `branch` in the work tree under `root`.
fn an_item_of(root: &Path, branch: String, oid: String) -> WorkItem {
    return WorkItem {
        kind: WorkKind::Push,
        repository: root.to_path_buf(),
        git_dir: root.to_path_buf(),
        branch_ref: branch,
        oid,
        parent: String::new(),
        created: String::from("2026-11-14T00:00:00.000Z"),
        tries: 0,
    };
}

/// The middle,
/// the 99th percentile and the largest of `micros`,
/// with the count.
fn spread(mut micros: Vec<u128>) -> (u128, u128, u128, usize) {
    micros.sort_unstable();
    let at = |per_thousand: usize| -> u128 { return micros[(micros.len() * per_thousand / 1000).min(micros.len() - 1)]; };
    return (at(500), at(990), *micros.last().expect("at least one sample"), micros.len());
}

/// The text of one arm's spread,
/// in microseconds.
fn words(label: &str, micros: Vec<u128>) -> String {
    let (median, p99, largest, count): (u128, u128, u128, usize) = spread(micros);
    return format!("{label}: n={count} median={median}us p99={p99}us max={largest}us");
}

/// The ratio of two middles,
/// to two decimals,
/// so a cost is read as the multiple it is.
fn ratio(before: u128, after: u128) -> String {
    return format!("{:.2}", after as f64 / before.max(1) as f64);
}

/// What:
///  One reading of each arm per round,
///  with the arm that goes first alternating,
///  so a machine that slows or speeds during the run costs each arm the same.
/// Why:
///   The two arms are what the change is,
/// and a ratio of two spreads taken at different times is a ratio of those times.
///  Both arms read the same files of the same work tree:
/// one as this task did before the change,
/// in the body of the push's task,
/// and one as the push reads them now,
/// as a job of the blocking pool awaited within a ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let n = 0; n < ROUNDS; n++) { if (n % 2 === 0) { control(); bounded(); } else { bounded(); control(); } }
/// ```
async fn rounds(work: &Path, branch: &str) -> (Vec<u128>, Vec<u128>) {
    let mut control: Vec<u128> = Vec::with_capacity(ROUNDS);
    let mut bounded: Vec<u128> = Vec::with_capacity(ROUNDS);
    for number in 0..ROUNDS {
        if number % 2 == 0 {
            control.push(on_the_tasks_thread(work, branch));
            bounded.push(on_a_job_of_the_pool(work, branch).await);
        } else {
            bounded.push(on_a_job_of_the_pool(work, branch).await);
            control.push(on_the_tasks_thread(work, branch));
        }
    }
    return (control, bounded);
}

/// The two reads as the push's task made them before the change:
/// in the body of the task, timed around both.
fn on_the_tasks_thread(work: &Path, branch: &str) -> u128 {
    let began: Instant = Instant::now();
    let verdicts: Vec<crate::work_item_holding::BlockedVerdict> = crate::work_item_holding::blocked_verdicts(work);
    let undecided: Vec<String> = crate::work_item_holding::undecided_commits(work, branch);
    let took: u128 = began.elapsed().as_micros();
    // The reads' answers are read here so neither is optimised away, and a wrong answer is a fixture's fault.
    assert_eq!(verdicts.len(), RECORDS_OF_EACH_STATE, "the control arm read every verdict record");
    assert_eq!(undecided.len(), RECORDS_OF_EACH_STATE, "the control arm read the held records of the pushed branch");
    return took;
}

/// The two reads as the push makes them now:
/// as jobs of the blocking pool, awaited within a ceiling, timed around both.
async fn on_a_job_of_the_pool(work: &Path, branch: &str) -> u128 {
    let began: Instant = Instant::now();
    let verdicts: Vec<crate::work_item_holding::BlockedVerdict> =
        blocked_verdicts_within_awaiting(GENEROUS, work).await.expect("the verdict records answer");
    let undecided: Vec<String> = undecided_commits_within_awaiting(GENEROUS, work, branch).await.expect("the held records answer");
    let took: u128 = began.elapsed().as_micros();
    assert_eq!(verdicts.len(), RECORDS_OF_EACH_STATE, "the bounded arm read every verdict record");
    assert_eq!(undecided.len(), RECORDS_OF_EACH_STATE, "the bounded arm read the held records of the pushed branch");
    return took;
}

/// What:
///  What the two reads of a push's gate records cost,
///  as the ratio of the arm that awaits them on the pool to the arm that makes them on the task.
/// Why:
///   The change buys a bound and a thread that stays free with the cost of a place of the process's bound and
///  the hop to it,
/// and the cost is the ratio of the two middles, interleaved round by round.
/// The middle is what a reader compares:
/// the largest samples of both arms move with this host's load and say nothing about the change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const root = scratch(); const [control, bounded] = await runOnRuntime(rounds(work, branch)); console.log(ratioOfMiddles)
/// ```
#[test]
#[ignore = "a measurement for the record, not a control"]
fn the_two_reads_a_push_makes_cost_this_on_this_machine() {
    let root: PathBuf = fixture("gate-records-cost");
    let work: PathBuf = a_work_tree_with_a_handful_of_gate_records(root.as_path());
    let (control, bounded): (Vec<u128>, Vec<u128>) =
        crate::lazy_runtime::run_on_runtime(rounds(work.as_path(), "refs/heads/main")).expect("the runtime runs the measurement");
    let control_words: String = words("the reads made in the push's own task", control.clone());
    let bounded_words: String = words("the reads awaited as jobs of the pool", bounded.clone());
    let (control_middle, control_p99, control_largest, _): (u128, u128, u128, usize) = spread(control);
    let (bounded_middle, bounded_p99, bounded_largest, _): (u128, u128, u128, usize) = spread(bounded);
    println!("{control_words}");
    println!("{bounded_words}");
    println!("cost of the bound, median: {}", ratio(control_middle, bounded_middle));
    println!("cost of the bound, p99: {}", ratio(control_p99, bounded_p99));
    println!("cost of the bound, largest: {}", ratio(control_largest, bounded_largest));
    remove(root.as_path());
}

//endregion
