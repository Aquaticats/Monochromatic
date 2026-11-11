//! Controls for where the automatic push stops when a post-commit gate kept one of a branch's commits local.
//! Every case below runs real Git in a disposable repository with a bare remote,
//! because the questions this module asks are
//!  Git's own:
//! which commits a branch holds,
//! which of them the branch's upstream already holds,
//! and what one push
//!  actually delivers.
//! The verdicts are the ones a real gate wrote,
//! read back from the queue's own `blocked` state,
//! so a case here fails when
//!  the record,
//!  the question or the refspec is wrong.

use super::*;
use crate::auto_push::{AutoPushOutcome, AutoPushResult, push_once};
use crate::blocked_commit_lines::{UNRECORDED_REASON, held_back_line, named_reason, unanswered_line};
use crate::blocked_commit_questions::{ContextHistory, held_back_lines};
use crate::auto_push_single_flight::PushGit;
use crate::service_work::work_directory;
use crate::test_support::{fixture, git, git_context, git_text, remove};
use crate::work_item::{WorkKind, hold, state_directory};
use crate::work_item_holding::{BlockedVerdict, block};
/// The trait the service's push handler is reached through.
use crate::service_work::WorkHandlers;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Environment of every Git child here.
fn fixture_environment() -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let mut environment: Vec<(std::ffi::OsString, std::ffi::OsString)> = [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| return (std::ffi::OsString::from(name), std::ffi::OsString::from(value)))
    .collect();
    environment.extend(crate::test_support::scratch_environment());
    return environment;
}

/// The Git the queue pushes with,
/// for the repository at `root`.
fn queue_git(root: &Path) -> PushGit {
    return PushGit { real_git: PathBuf::from(crate::test_support::REAL_GIT), cwd: root.to_path_buf(), environment: fixture_environment(), lfs: false };
}

/// A bare remote,
/// a repository that tracks it,
/// and the branch the cases push.
struct World {
    /// The fixture root,
    /// removed when the case ends.
    root: PathBuf,
    /// The bare remote named `origin`.
    remote: PathBuf,
    /// The repository under test.
    repo: PathBuf,
}

/// The branch every case pushes.
const BRANCH: &str = "refs/heads/main";

/// The bare remote,
/// the repository that tracks it,
/// with its empty `base` commit already pushed.
fn world(name: &str) -> World {
    let root: PathBuf = fixture(name);
    let remote: PathBuf = root.join("remote.git");
    git(&root, &["init", "--quiet", "--bare", "--initial-branch=main", remote.to_str().expect("UTF-8")]);
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("repository directory");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("a.txt");
    git(&repo, &["add", "a.txt"]);
    git(&repo, &["commit", "--quiet", "--message=base"]);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("UTF-8")]);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    return World { root, remote, repo };
}

/// Write `text` to `a.txt` and commit it,
/// answering the commit's name.
fn commit(world: &World, text: &str) -> String {
    std::fs::write(world.repo.join("a.txt"), text).expect("a.txt");
    git(&world.repo, &["commit", "--quiet", "--message", text.trim(), "a.txt"]);
    return git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
}

/// Whether the bare remote holds `commit`.
/// The question is asked of the remote's own object store,
/// and "no" is an answer.
fn remote_has(world: &World, commit: &str) -> bool {
    let asked: std::process::Output = Command::new(crate::test_support::REAL_GIT)
        .current_dir(world.remote.as_path())
        .args(["cat-file", "-e", format!("{commit}^{{commit}}").as_str()])
        .output()
        .expect("real Git answers");
    return asked.status.success();
}

/// The commit the branch names on the remote.
fn remote_branch(world: &World) -> String {
    return git_text(world.remote.as_path(), &["rev-parse", BRANCH]);
}

/// What:
///  Record the verdict a post-commit gate wrote for `commit` on the world's branch,
/// the way `forget_the_push` does.
/// Why:
///   The state the push reads lives in the queue's own directory,
/// so the case writes it through the production step rather than by hand,
///  and reads it back through `blocked_verdicts` where the push reads it.
fn record_the_verdict(world: &World, commit: &str, reason: &str) -> Vec<BlockedVerdict> {
    let item = crate::work_item::WorkItem {
        kind: WorkKind::Push,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        branch_ref: String::from(BRANCH),
        oid: String::from(commit),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
    let work: PathBuf = work_directory(world.repo.join(".git").as_path());
    // The gate holds the push record before it decides, and blocks it when it refuses; the hold is what a case that only
    //  records a verdict reproduces.
    hold(work.as_path(), &item, "").expect("the push is held while the gate decides");
    block(work.as_path(), &item, reason);
    assert!(!state_directory(work.as_path(), crate::work_item::HELD).join(item.id()).exists(), "a blocked push is not held");
    return crate::work_item_holding::blocked_verdicts(work.as_path());
}

/// Push the branch the way the service does,
/// answering the result and the text the push printed.
fn push(world: &World, landed: &str, blocked: &[BlockedVerdict]) -> (AutoPushResult, String) {
    let mut err: Vec<u8> = Vec::new();
    let result: AutoPushResult = push_once(&queue_git(world.repo.as_path()), BRANCH, landed, blocked, &[], &mut err).expect("the push runs");
    return (result, String::from_utf8_lossy(err.as_slice()).into_owned());
}

/// Push the branch the way the service does,
/// awaited by a caller that is itself a task of the runtime,
/// answering the result.
fn await_push(world: &World, landed: &str, blocked: &[BlockedVerdict], undecided: &[String]) -> AutoPushResult {
    let mut err: Vec<u8> = Vec::new();
    let result: Result<AutoPushResult, crate::auto_push_single_flight::SingleFlightError> =
        crate::lazy_runtime::run_on_runtime(crate::auto_push::push_once_awaiting(&queue_git(world.repo.as_path()), BRANCH, landed, blocked, undecided, &mut err))
            .expect("the runtime runs the push");
    return result.expect("the push runs");
}

/// What:
///  A verdict on the branch's way out stops the push at the commit before it,
/// and says which commit and which check hold the branch back.
/// Why:
///   This is the whole promise of the owner's answer:
/// the blocked commit stays local,
/// every older commit still reaches
///  GitHub,
/// and no later commit drags the blocked one along.
#[test]
fn a_verdict_stops_the_push_before_the_blocked_commit() {
    let world: World = world("held-back-stops-before");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let blocked: String = commit(&world, "blocked\n");
    let landed: String = commit(&world, "landed\n");
    // The control the promise is measured against: with no verdict the whole branch goes out, so the remote can hold every one
    //  of these commits and this case can tell a stop from a push.
    let verdicts: Vec<BlockedVerdict> = record_the_verdict(&world, blocked.as_str(), "final-newline: the file does not end with a newline");
    assert_eq!(verdicts.len(), 1, "the gate kept one verdict");
    assert_eq!(verdicts.first().map(|verdict| return verdict.reason.as_str()), Some("final-newline: the file does not end with a newline"));
    let (result, shown) = push(&world, landed.as_str(), verdicts.as_slice());
    assert_eq!(result.outcome, AutoPushOutcome::HeldBack, "{shown}");
    assert_eq!(result.exit_code, 0, "{shown}");
    assert_eq!(remote_branch(&world), base, "the remote holds the commit before the blocked one and nothing past it");
    assert!(!remote_has(&world, blocked.as_str()), "the blocked commit stays local");
    assert!(!remote_has(&world, landed.as_str()), "the commit after it does not drag it along");
    assert!(shown.contains("is held back by the commit"), "{shown}");
    assert!(shown.contains(blocked.as_str()), "the line names the commit: {shown}");
    assert!(shown.contains("final-newline"), "the line names the check: {shown}");
    remove(world.root.as_path());
}

/// What:
///  With no verdict the same push publishes the whole branch.
/// Why:
///   The stop above is worth nothing unless the same repository sends everything when nothing holds it back;
/// this is
///  the positive control that the difference is the verdict and not the fixture.
#[test]
fn without_a_verdict_the_same_branch_is_pushed_whole() {
    let world: World = world("held-back-none");
    commit(&world, "blocked\n");
    let landed: String = commit(&world, "landed\n");
    let (result, shown) = push(&world, landed.as_str(), &[]);
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{shown}");
    assert_eq!(remote_branch(&world), landed, "the remote holds the branch's tip");
    remove(world.root.as_path());
}

/// What:
///  A verdict whose commit was amended away holds nothing back:
/// the branch's way out no longer runs through it.
/// Why:
///   The owner's answer says the push resumes once the commit is off the branch's way out,
/// and the branch must resume by
///  itself rather than wait for the verdict to be removed by hand.
#[test]
fn a_verdict_off_the_branchs_way_out_holds_nothing_back() {
    let world: World = world("held-back-amended-away");
    let blocked: String = commit(&world, "blocked\n");
    let verdicts: Vec<BlockedVerdict> = record_the_verdict(&world, blocked.as_str(), "final-newline: no final newline");
    // The person fixes the commit: the branch that replaces it does not descend from the blocked one.
    git(&world.repo, &["reset", "--quiet", "--hard", "HEAD~1"]);
    let landed: String = commit(&world, "fixed\n");
    let (result, shown) = push(&world, landed.as_str(), verdicts.as_slice());
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{shown}");
    assert_eq!(remote_branch(&world), landed, "the push resumed once the blocked commit was off the way out");
    assert!(!remote_has(&world, blocked.as_str()), "the replaced commit was never published");
    remove(world.root.as_path());
}

/// What:
///  A verdict whose commit Git already holds in the branch's remote holds nothing back.
/// Why:
///   A person's own `git push` is unchanged and may publish that commit deliberately;
/// once it is on the remote,
/// nothing of it
///  remains to withhold,
/// and a push that kept stopping would leave the branch permanently short.
#[test]
fn a_verdict_whose_commit_the_remote_holds_holds_nothing_back() {
    let world: World = world("held-back-already-there");
    let blocked: String = commit(&world, "blocked\n");
    let verdicts: Vec<BlockedVerdict> = record_the_verdict(&world, blocked.as_str(), "final-newline: no final newline");
    git(&world.repo, &["push", "--quiet", "origin", "main"]);
    let landed: String = commit(&world, "landed\n");
    let (result, shown) = push(&world, landed.as_str(), verdicts.as_slice());
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{shown}");
    assert_eq!(remote_branch(&world), landed, "a commit the remote already holds holds nothing back");
    remove(world.root.as_path());
}

/// What:
///  A verdict on a commit the branch does not hold holds nothing back,
/// so one branch's refusal never stops another branch's push.
/// Why:
///   The queue pushes whatever branch a record names,
/// and a verdict is about one commit on one branch.
#[test]
fn a_verdict_on_another_branch_holds_nothing_back() {
    let world: World = world("held-back-other-branch");
    let blocked: String = commit(&world, "blocked\n");
    let landed: String = commit(&world, "landed\n");
    let mut verdicts: Vec<BlockedVerdict> = record_the_verdict(&world, blocked.as_str(), "final-newline: no final newline");
    verdicts.first_mut().expect("one verdict").branch_ref = String::from("refs/heads/elsewhere");
    let (result, shown) = push(&world, landed.as_str(), verdicts.as_slice());
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{shown}");
    assert_eq!(remote_branch(&world), landed, "another branch's verdict decides nothing here");
    remove(world.root.as_path());
}

/// What:
///  The line a command prints names the branch,
/// the commit and the words of its verdict,
/// and goes once the commit is off the branch's way out.
/// Why:
///   Every command says which commit holds the branch back,
/// and the line must never outlive the state it describes:
/// a
///  line that still claimed a block after the commit was replaced would be read as a push that is not happening.
#[test]
fn the_line_names_the_branch_the_commit_and_its_verdict() {
    let world: World = world("held-back-line");
    let blocked: String = commit(&world, "blocked\n");
    let _landed: String = commit(&world, "landed\n");
    record_the_verdict(&world, blocked.as_str(), "final-newline: the file does not end with a newline");
    let git_dir: PathBuf = world.repo.join(".git");
    // The questions are asked with the same Git context a command holds, in the worktree the command runs in.
    let context: crate::transaction_git::GitContext = git_context();
    let history: ContextHistory<'_> = ContextHistory { context: &context, cwd: world.repo.as_path() };
    let lines: Vec<String> = held_back_lines(git_dir.as_path(), &history);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &String = lines.first().expect("one line");
    assert!(line.starts_with("main is held back by the commit "), "{line}");
    assert!(line.contains(blocked.as_str()), "{line}");
    assert!(line.contains("final-newline: the file does not end with a newline"), "{line}");
    assert!(line.contains("a person's `git push` still sends it"), "{line}");
    // The commit is replaced, so the way out no longer runs through it and the line goes.
    git(&world.repo, &["reset", "--quiet", "--hard", "HEAD~2"]);
    commit(&world, "fixed\n");
    assert_eq!(held_back_lines(git_dir.as_path(), &history), Vec::<String>::new(), "a replaced commit holds nothing back");
    remove(world.root.as_path());
}

/// What:
///  A verdict on a commit this repository does not hold holds nothing back.
/// Why:
///   A commit whose objects are gone cannot be published by any push of this repository,
/// so nothing of it can leak;
/// a verdict
///  that stopped the branch forever over a lost object would leave every later commit unpublished.
#[test]
fn a_verdict_on_a_commit_the_repository_does_not_hold_holds_nothing_back() {
    let world: World = world("held-back-lost-commit");
    let landed: String = commit(&world, "landed\n");
    let missing: String = String::from("f".repeat(40));
    let verdicts: Vec<BlockedVerdict> = vec![BlockedVerdict { branch_ref: String::from(BRANCH), commit: missing, reason: String::from("final-newline: no final newline") }];
    let (result, shown) = push(&world, landed.as_str(), verdicts.as_slice());
    assert_eq!(result.outcome, AutoPushOutcome::Pushed, "{shown}");
    assert_eq!(remote_branch(&world), landed, "a commit the repository does not hold decides nothing");
    remove(world.root.as_path());
}

/// What:
///  The decision sends nothing when a question about the branch could not be answered,
/// and says which commit it could not place.
/// Why:
///   Sending nothing loses nothing,
/// while sending a commit the gate refused cannot be undone;
/// the person is told why nothing was sent
///  and that their own push still sends the branch.
#[test]
fn an_unanswerable_question_sends_nothing() {
    let verdicts: Vec<BlockedVerdict> = vec![
        BlockedVerdict { branch_ref: String::from(BRANCH), commit: String::from("1").repeat(40), reason: String::from("final-newline: no final newline") },
        BlockedVerdict { branch_ref: String::from(BRANCH), commit: String::from("2").repeat(40), reason: String::from("final-newline: no final newline") },
    ];
    let history: Unanswered = Unanswered;
    let stop: WhereThePushStops = where_the_push_stops(&history, BRANCH, None, verdicts.as_slice(), &[]);
    let stopped: Stopped = match stop {
        WhereThePushStops::Unanswered(stopped) => stopped,
        other => panic!("an unanswerable question must send nothing, and the decision answered {other:?}"),
    };
    assert_eq!(stopped.pushable, None, "nothing is published");
    let line: String = unanswered_line(BRANCH, &stopped);
    assert!(line.starts_with("main may be held back by "), "{line}");
    assert!(line.contains("could not say whether it is still on main's way out"), "{line}");
    assert!(line.contains("sends nothing"), "{line}");
    assert!(line.contains("a person's `git push` still sends the branch"), "{line}");
}

/// What:
///  Every commit the push may not publish is named,
/// not only the first.
/// Why:
///   Two verdicts on one branch are two commits that stay local,
/// and a line that named one of them would read as though the other
///  went out.
#[test]
fn several_verdicts_are_all_named() {
    let first: String = String::from("1").repeat(40);
    let second: String = String::from("2").repeat(40);
    let stopped: Stopped = Stopped {
        commits: vec![
            Held { commit: first.clone(), reason: String::from("final-newline: no final newline") },
            Held { commit: second.clone(), reason: String::from("forbidden-strings: a rule matched") },
        ],
        pushable: Some(String::from("3").repeat(40)),
    };
    let line: String = held_back_line(BRANCH, &stopped);
    assert!(line.starts_with("main is held back by these commits"), "{line}");
    assert!(line.contains(&format!("{first} (final-newline: no final newline)")), "{line}");
    assert!(line.contains(&format!("{second} (forbidden-strings: a rule matched)")), "{line}");
    assert!(line.contains("publishes nothing past them"), "{line}");
}

/// What:
///  A verdict a record of an earlier release wrote is named with the words this release has for it.
/// Why:
///   Such a record holds the word `blocked` alone,
/// and an empty clause would read as no reason at all.
#[test]
fn a_verdict_with_no_words_is_named_with_its_own_sentence() {
    let stopped: Stopped = Stopped {
        commits: vec![Held { commit: String::from("1").repeat(40), reason: named_reason("  ") }],
        pushable: None,
    };
    let line: String = held_back_line(BRANCH, &stopped);
    assert!(line.contains(UNRECORDED_REASON), "{line}");
    assert_eq!(named_reason(" final-newline: no final newline "), "final-newline: no final newline");
}

/// What:
///  The history that answers nothing,
/// for the controls of the decision alone.
/// Why:
///   A Git that cannot relate two commits it holds is not a state a fixture can be put in safely,
/// so the questions are answered here
///  instead.
struct Unanswered;

impl BranchHistory for Unanswered {
    fn holds(&self, _of: &str, _commit: &str) -> Option<bool> {
        return None;
    }
    fn first_parent(&self, _commit: &str) -> Option<String> {
        return None;
    }
    fn merge_base(&self, _commits: &[String]) -> Option<String> {
        return None;
    }
}

/// What:
///  The push record of one commit of the world's branch,
///  as the queue holds it.
/// Why:
///  The service pushes the record's branch and its own directory's records,
///  so a case that drives the service's own push needs the record the commit left.
fn push_item_of(world: &World, commit: &str) -> crate::work_item::WorkItem {
    return crate::work_item::WorkItem {
        kind: WorkKind::Push,
        repository: world.repo.clone(),
        git_dir: world.repo.join(".git"),
        branch_ref: String::from(BRANCH),
        oid: String::from(commit),
        parent: String::new(),
        created: crate::iso_time::now_iso(),
        tries: 0,
    };
}

/// What:
///  A holder text naming a process that has ended,
///  spelled as `this_command` spells one.
/// Why:
///  A recorded pass is released only once the command that wrote it is proven gone,
///  and a child this control starts and waits for is the one such proof it can make without a race.
fn a_command_that_has_ended() -> String {
    let mut child: std::process::Child = crate::test_support::spawn_owned(std::process::Command::new("sleep").arg("30")).expect("a child starts");
    let pid: u32 = child.id();
    let birth: String = crate::process_identity::process_birth_identity(i64::from(pid)).expect("the table is readable").expect("the child runs");
    let _ended = child.kill();
    let _reaped = child.wait();
    return format!("{}/{pid}/{birth}", crate::claim_time::current_boot_id());
}

/// What:
///  A commit whose own post-commit check still runs is not published by any push of its branch,
///  even one made for another commit.
/// Why:
///   The queue holds a commit's push until its gate answers,
/// and a gate may refuse it;
///  a push of the branch sends every commit up to the tip,
/// so publishing the tip before that answer publishes a commit the gate may keep local.
#[test]
fn a_commit_whose_own_checks_have_not_decided_is_not_published() {
    let world: World = world("undecided-checks-hold-the-tip");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let undecided: String = commit(&world, "undecided\n");
    let work: PathBuf = work_directory(world.repo.join(".git").as_path());
    let item: crate::work_item::WorkItem = push_item_of(&world, undecided.as_str());
    // This control's own process holds the record, so the command whose checks run is provably alive.
    hold(work.as_path(), &item, crate::held_push_holder_and_verdict::this_command().as_str()).expect("the push is held while its checks run");

    let handlers: crate::service_work::GitHandlers = crate::service_work::GitHandlers::new(git_context());
    let verdict: crate::service_work::Verdict = handlers.push(std::slice::from_ref(&item));

    let crate::service_work::Verdict::Retry(shown) = verdict else { panic!("a push waiting for a running check is tried again, and the service answered {verdict:?}") };
    assert!(shown.contains(undecided.as_str()), "the line names the commit whose check runs: {shown}");
    assert!(shown.contains("has not decided"), "the line says the check has not decided: {shown}");
    assert_eq!(remote_branch(&world), base, "the automatic push stops before the commit whose checks still run");
    assert!(!remote_has(&world, undecided.as_str()), "the undecided commit is not published");
    remove(world.root.as_path());
}

/// What:
///  Releasing an older approved push does not publish a newer commit whose checks still run.
/// Why:
///   This is the recovery every command makes when the command that landed a commit died after recording its pass:
///  the older record becomes runnable and the service is woken,
/// while a newer commit's gate is still running.
///  The wake must not publish the newer commit.
#[test]
fn an_older_passed_record_does_not_publish_a_newer_undecided_commit() {
    let world: World = world("older-pass-newer-undecided");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let older: String = commit(&world, "older\n");
    let newer: String = commit(&world, "newer\n");
    let work: PathBuf = work_directory(world.repo.join(".git").as_path());
    let older_item: crate::work_item::WorkItem = push_item_of(&world, older.as_str());
    hold(work.as_path(), &older_item, "").expect("the push was held while its gate decided");
    // The command that landed `older` recorded its pass and died before it could promote the record.
    assert!(
        crate::work_item::record_verdict(work.as_path(), older_item.id().as_str(), a_command_that_has_ended().as_str(), crate::held_push_holder_and_verdict::Verdict::Passed, ""),
        "the older record holds the pass its command left"
    );
    let newer_item: crate::work_item::WorkItem = push_item_of(&world, newer.as_str());
    hold(work.as_path(), &newer_item, crate::held_push_holder_and_verdict::this_command().as_str()).expect("the newer commit's gate still runs");

    let said: Vec<String> = crate::service_handoff::held_push_notices(
        world.repo.join(".git").as_path(),
        &crate::held_push_commit_on_the_remote::NothingAsked,
        &crate::held_push_release_wakes_the_service::NothingWoken,
    );
    assert!(said.iter().any(|line| { return line.contains(older.as_str()); }), "the older pass is released and said: {said:?}");

    let handlers: crate::service_work::GitHandlers = crate::service_work::GitHandlers::new(git_context());
    let verdict: crate::service_work::Verdict = handlers.push(std::slice::from_ref(&older_item));

    assert!(matches!(verdict, crate::service_work::Verdict::Retry(_)), "the push stops and is tried again, and the service answered {verdict:?}");
    assert_eq!(remote_branch(&world), older, "the released record publishes only the commit whose gate approved it");
    assert!(remote_has(&world, older.as_str()), "the approved commit reached the remote");
    assert!(!remote_has(&world, newer.as_str()), "the commit whose checks still run was not published");
    assert!(state_directory(work.as_path(), crate::work_item::HELD).join(newer_item.id()).exists(), "the newer record is still held");
    assert_ne!(remote_branch(&world), base, "the base was published before the release, so this case tells a stop from a push");
    remove(world.root.as_path());
}

/// What:
///  A commit that lands after the service's push read the worktree's verdicts is not published by that push.
/// Why:
///   The push reads the held and blocked records and then pushes the branch,
///  and the branch can move in between:
///  a commit that lands there has its own gate still running,
///  so publishing it would publish a commit that gate may keep local,
///  and no later verdict could take it back.
/// The push therefore sends the commit the branch named before that reading,
///  never the branch's reference,
/// and the newer commit goes out with its own push once its gate answers.
///  The stand-in below lands the commit and puts its live record back at the push's first question about the upstream,
/// which is after
///  the service read the worktree.
#[test]
#[cfg(unix)]
fn a_commit_that_lands_after_the_verdicts_were_read_is_not_published() {
    let world: World = world("commit-after-the-reading");
    let first: String = commit(&world, "first\n");
    let racing: String = commit(&world, "racing\n");
    let work: PathBuf = work_directory(world.repo.join(".git").as_path());
    let racing_item: crate::work_item::WorkItem = push_item_of(&world, racing.as_str());
    hold(work.as_path(), &racing_item, crate::held_push_holder_and_verdict::this_command().as_str()).expect("the racing commit's gate still runs");
    // The record of the commit whose gate runs is put aside, so the service reads a worktree that holds none, and the branch is
    //  put back to what it was at that reading.
    let held: PathBuf = state_directory(work.as_path(), crate::work_item::HELD).join(racing_item.id());
    let staged: PathBuf = world.root.join("staged-held-record");
    std::fs::rename(held.as_path(), staged.as_path()).expect("the live record waits beside the worktree");
    git(&world.repo, &["update-ref", "refs/heads/main", first.as_str()]);
    let marker: PathBuf = world.root.join("stand-in-ran");
    let script: String = format!(
        "#!/bin/sh\ncase \"$*\" in\n  *'@{{upstream}}'*)\n    if [ ! -e '{marker}' ]; then\n      '{real}' -C '{repo}' update-ref refs/heads/main '{racing}'\n      mv '{staged}' '{held}'\n      : > '{marker}'\n    fi\n    ;;\nesac\nexec '{real}' \"$@\"\n",
        marker = marker.display(),
        real = crate::test_support::REAL_GIT,
        repo = world.repo.display(),
        racing = racing,
        staged = staged.display(),
        held = held.display(),
    );
    let stand_in: PathBuf = world.root.join("git-stand-in");
    crate::test_support::executable(stand_in.as_path(), script.as_bytes());
    let mut context: crate::transaction_git::GitContext = git_context();
    context.real_git = stand_in;
    // Every question the push asks starts the stand-in, so the one that lands the commit is the push's own question about the
    //  upstream, which it asks after the service read the worktree (`repo_read_hook.rs` answers a query in process otherwise).
    let handlers: crate::service_work::GitHandlers =
        crate::service_work::GitHandlers::new(context).with_push_environment(&[(crate::repo_read_hook::SWITCH_VARIABLE, "off")]);
    let first_item: crate::work_item::WorkItem = push_item_of(&world, first.as_str());
    let verdict: crate::service_work::Verdict = handlers.push(std::slice::from_ref(&first_item));

    assert!(marker.exists(), "the stand-in moved the branch and put the live record back after the reading");
    assert!(held.exists(), "the racing commit's own gate still runs");
    println!("post-reading commit: verdict={verdict:?}; remote={}", remote_branch(&world));
    assert!(matches!(verdict, crate::service_work::Verdict::Done), "the push ran and delivered the commit it was for: {verdict:?}");
    assert_eq!(remote_branch(&world), first, "the push sent the commit the branch named before the reading");
    assert!(remote_has(&world, first.as_str()), "the commit the push was for reached the remote");
    assert!(!remote_has(&world, racing.as_str()), "BUG: the push published the commit whose own gate had not answered");
    remove(world.root.as_path());
}

/// What:
///  A refused commit whose durable verdict could not be written is not published by a later push of its branch.
/// Why:
///   The gate writes its refusal into the held record first and the verdict every push reads second,
///  so a storage failure on the second leaves the record alone with the refusal.
/// Forgetting that record would leave neither,
///  `is_blocked` would read false,
///  and the branch's next push would publish the very commit the gate refused.
/// The record therefore stays,
///  and it is read as the verdict it states.
#[test]
fn a_refusal_whose_verdict_could_not_be_written_is_not_published_by_a_later_push() {
    let world: World = world("refusal-whose-verdict-could-not-be-written");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let refused: String = commit(&world, "refused\n");
    let later: String = commit(&world, "later\n");
    let work: PathBuf = work_directory(world.repo.join(".git").as_path());
    let refused_item: crate::work_item::WorkItem = push_item_of(&world, refused.as_str());
    hold(work.as_path(), &refused_item, crate::held_push_holder_and_verdict::this_command().as_str()).expect("the push is held while the gate decides");
    assert!(
        crate::work_item::record_refusal(work.as_path(), refused_item.id().as_str(), "final-newline: the file does not end with a newline"),
        "the gate's words are written into the record it holds"
    );
    // The one place a durable verdict lives cannot be used: its name is a regular file.
    std::fs::write(state_directory(work.as_path(), crate::work_item_holding::BLOCKED).as_path(), "not a directory")
        .expect("a file in the verdict directory's place");
    assert!(!block(work.as_path(), &refused_item, "final-newline: the file does not end with a newline"), "the verdict cannot be kept");
    assert!(
        state_directory(work.as_path(), crate::work_item::HELD).join(refused_item.id()).exists(),
        "the refusal survives only in the record the gate held"
    );

    // A later commit lands on the same branch, and the service pushes the branch for it.
    let later_item: crate::work_item::WorkItem = push_item_of(&world, later.as_str());
    let handlers: crate::service_work::GitHandlers = crate::service_work::GitHandlers::new(git_context());
    let verdict: crate::service_work::Verdict = handlers.push(std::slice::from_ref(&later_item));

    println!("unkept refusal: verdict={verdict:?}; refused={refused}; later={later}; remote={}", remote_branch(&world));
    assert!(!remote_has(&world, refused.as_str()), "BUG: the branch's push published the commit whose refusal could not be written beside it");
    assert_eq!(remote_branch(&world), base, "the branch's push stops before the refused commit, and publishes nothing past it");
    remove(world.root.as_path());
}

/// What:
///  The first push of a branch that has no upstream stops at the base below a blocked commit,
///  even though that push is also the one that would give the branch its upstream.
/// Why:
///  A branch with an `origin` and no configured upstream is pushed to `origin` under its own name and takes it as its upstream,
///  and Git sets an upstream only from a refspec that names a branch.
/// Naming the branch there is what published the commit the gate refused:
///  the push sent every commit up to the branch's tip,
///  and its result could still say the branch was held back.
/// The push that stops sends the commit below the refused one and leaves the upstream for a push that can name the branch.
#[test]
fn a_first_push_with_no_upstream_stops_at_the_base_below_a_blocked_commit() {
    let world: World = world("first-push-with-no-upstream");
    let first: String = commit(&world, "first\n");
    let blocked: String = commit(&world, "blocked\n");
    let landed: String = commit(&world, "landed\n");
    // The branch keeps its `origin` and loses only the upstream a previous push set.
    git(&world.repo, &["config", "--unset", "branch.main.remote"]);
    git(&world.repo, &["config", "--unset", "branch.main.merge"]);
    let verdicts: Vec<BlockedVerdict> = record_the_verdict(&world, blocked.as_str(), "final-newline: the file does not end with a newline");

    let (result, shown) = push(&world, landed.as_str(), verdicts.as_slice());
    let remote: String = remote_branch(&world);
    println!("first push: result={result:?}; shown={shown}; first={first}; blocked={blocked}; landed={landed}; remote={remote}");
    assert!(!remote_has(&world, blocked.as_str()), "BUG: the first push published the commit the gate refused");
    assert!(!remote_has(&world, landed.as_str()), "BUG: the first push published the commit past the refused one");
    // The push did run: the commit below the refused one reached the remote, so this case tells a stop from a push that never ran.
    assert_eq!(remote, first, "BUG: the first push did not stop at the commit below the refused one");
    assert_eq!(result.outcome, AutoPushOutcome::HeldBack, "{shown}");
    assert!(shown.contains(blocked.as_str()), "the line names the commit that holds the branch back: {shown}");
    remove(world.root.as_path());
}

/// What:
///  The awaited first push of a branch that has no upstream stops there too.
/// Why:
///  The service pushes through the awaited face,
///  and a face that named the branch while the other sent the commit below the refused one
///  would publish the refused commit for every commit the service pushes.
#[test]
fn the_awaited_first_push_with_no_upstream_stops_at_the_base_below_a_blocked_commit() {
    let world: World = world("awaited-first-push-with-no-upstream");
    let first: String = commit(&world, "first\n");
    let blocked: String = commit(&world, "blocked\n");
    let landed: String = commit(&world, "landed\n");
    git(&world.repo, &["config", "--unset", "branch.main.remote"]);
    git(&world.repo, &["config", "--unset", "branch.main.merge"]);
    let verdicts: Vec<BlockedVerdict> = record_the_verdict(&world, blocked.as_str(), "final-newline: the file does not end with a newline");

    let result: AutoPushResult = await_push(&world, landed.as_str(), verdicts.as_slice(), &[]);
    let remote: String = remote_branch(&world);
    println!("awaited first push: result={result:?}; first={first}; blocked={blocked}; landed={landed}; remote={remote}");
    assert!(!remote_has(&world, blocked.as_str()), "BUG: the awaited first push published the commit the gate refused");
    assert_eq!(remote, first, "BUG: the awaited first push did not stop at the commit below the refused one");
    assert_eq!(result.outcome, AutoPushOutcome::HeldBack, "{}", result.shown);
    remove(world.root.as_path());
}

/// What:
///  A push under `push.default=matching` sends nothing while the branch holds a commit its reading did not see,
///  and the retry that follows publishes the branch once the branch names the commit again.
/// Why:
///  `push.default=matching` leaves the refspec to Git,
///  so Git resolves the branch at push time:
///  a commit that landed after the reading goes out with no verdict at all,
///  and its own gate may still keep it local.
/// Nothing is sent then,
///  and the branch's retry plan runs the same matching push once the branch has settled,
///  which is the backup the commit needs.
#[test]
fn a_matching_push_does_not_publish_a_commit_that_landed_after_the_reading() {
    let world: World = world("matching-push-keeps-the-pinned-commit");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let pinned: String = commit(&world, "pinned\n");
    let racing: String = commit(&world, "racing\n");
    git(&world.repo, &["config", "push.default", "matching"]);

    // The verdicts were read while the branch named `pinned`; `racing` landed after that reading.
    let (result, shown) = push(&world, pinned.as_str(), &[]);
    let remote: String = remote_branch(&world);
    println!("matching push: result={result:?}; shown={shown}; pinned={pinned}; racing={racing}; remote={remote}");
    assert!(!remote_has(&world, racing.as_str()), "BUG: the matching push published the commit that landed after the reading");
    assert_eq!(remote, base, "BUG: the matching push published past the commit its reading ended at");
    assert_eq!(result.outcome, AutoPushOutcome::HeldBack, "{shown}");
    assert!(shown.contains("moved after this push read it"), "the line says why nothing was sent: {shown}");

    // The retry the service plans asks again with the branch's own tip, which Git's matching push may then send.
    let (retried, retried_shown) = push(&world, racing.as_str(), &[]);
    assert_eq!(retried.outcome, AutoPushOutcome::Pushed, "{retried_shown}");
    assert_eq!(remote_branch(&world), racing, "the retry published the branch once it named the commit the push read");
    remove(world.root.as_path());
}

/// What:
///  The awaited matching push sends nothing for a commit that landed after its reading either,
///  and its retry publishes the branch.
/// Why:
///  The service's push is the awaited face,
///  and the guard that keeps a matching push from publishing a commit with no verdict is a query of its own there.
#[test]
fn the_awaited_matching_push_does_not_publish_a_commit_that_landed_after_the_reading() {
    let world: World = world("awaited-matching-push-keeps-the-pinned-commit");
    let base: String = git_text(world.repo.as_path(), &["rev-parse", "HEAD"]);
    let pinned: String = commit(&world, "pinned\n");
    let racing: String = commit(&world, "racing\n");
    git(&world.repo, &["config", "push.default", "matching"]);

    let result: AutoPushResult = await_push(&world, pinned.as_str(), &[], &[]);
    println!("awaited matching push: result={result:?}; pinned={pinned}; racing={racing}");
    assert!(!remote_has(&world, racing.as_str()), "BUG: the awaited matching push published the commit that landed after the reading");
    assert_eq!(remote_branch(&world), base, "BUG: the awaited matching push published past the commit its reading ended at");
    assert_eq!(result.outcome, AutoPushOutcome::HeldBack, "{}", result.shown);

    let retried: AutoPushResult = await_push(&world, racing.as_str(), &[], &[]);
    assert_eq!(retried.outcome, AutoPushOutcome::Pushed, "{}", retried.shown);
    assert_eq!(remote_branch(&world), racing, "the retry published the branch once it named the commit the push read");
    remove(world.root.as_path());
}
