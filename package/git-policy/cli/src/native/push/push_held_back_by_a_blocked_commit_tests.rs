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
//!  the record, the question or the refspec is wrong.

use super::*;
use crate::auto_push::{AutoPushOutcome, AutoPushResult, push_once};
use crate::auto_push_single_flight::PushGit;
use crate::service_work::work_directory;
use crate::test_support::{fixture, git, git_context, git_text, remove};
use crate::work_item::{WorkKind, hold, state_directory};
use crate::work_item_holding::{BlockedVerdict, block};
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
    hold(work.as_path(), &item).expect("the push is held while the gate decides");
    block(work.as_path(), &item, reason);
    assert!(!state_directory(work.as_path(), crate::work_item::HELD).join(item.id()).exists(), "a blocked push is not held");
    return crate::work_item_holding::blocked_verdicts(work.as_path());
}

/// Push the branch the way the service does,
/// answering the result and the text the push printed.
fn push(world: &World, landed: &str, blocked: &[BlockedVerdict]) -> (AutoPushResult, String) {
    let mut err: Vec<u8> = Vec::new();
    let result: AutoPushResult = push_once(&queue_git(world.repo.as_path()), BRANCH, landed, blocked, &mut err).expect("the push runs");
    return (result, String::from_utf8_lossy(err.as_slice()).into_owned());
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
    assert_eq!(verdicts.first().map(|verdict| verdict.reason.as_str()), Some("final-newline: the file does not end with a newline"));
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
    let work: PathBuf = work_directory(world.repo.join(".git").as_path());
    // The questions are asked with the same Git context a command holds, in the worktree the command runs in.
    let context: crate::transaction_git::GitContext = git_context();
    let history: ContextHistory<'_> = ContextHistory { context: &context, cwd: world.repo.as_path() };
    let lines: Vec<String> = held_back_lines(work.as_path(), &history);
    assert_eq!(lines.len(), 1, "{lines:?}");
    let line: &String = lines.first().expect("one line");
    assert!(line.starts_with("main is held back by the commit "), "{line}");
    assert!(line.contains(blocked.as_str()), "{line}");
    assert!(line.contains("final-newline: the file does not end with a newline"), "{line}");
    assert!(line.contains("a person's `git push` still sends it"), "{line}");
    // The commit is replaced, so the way out no longer runs through it and the line goes.
    git(&world.repo, &["reset", "--quiet", "--hard", "HEAD~2"]);
    commit(&world, "fixed\n");
    assert_eq!(held_back_lines(work.as_path(), &history), Vec::<String>::new(), "a replaced commit holds nothing back");
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
    let stop: WhereThePushStops = where_the_push_stops(&history, BRANCH, None, verdicts.as_slice());
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
    fn upstream_of(&self, _branch_ref: &str) -> Option<String> {
        return None;
    }
}
