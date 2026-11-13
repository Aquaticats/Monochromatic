//! What:
//!  Binary-level controls for a typed push of a branch that is not checked out,
//! and for every shape of that push:
//! by name,
//! with `HEAD` detached,
//! from a linked worktree,
//! with two sides named differently,
//! with several refspecs,
//! with a tag,
//! with the branch checked out in another worktree,
//! and with a conflict.
//! Why:
//!  A typed push of `feat/cli-git-worktree-copy` while `main` was checked out published a merge whose subject,
//! body and report named `main`,
//! a wrong statement that cannot be corrected without rewriting the published history
//! (`doc/handover/cli-git-rust-implementation.md`,
//! "A typed push of a branch that was not checked out published a merge labelled with the checked-out branch").
//!  Every control here asserts the merge commit's message,
//! the line the wrapper prints,
//! and where every branch and every worktree is afterwards,
//! because the two wrong names and the branch left behind its own remote were each one of those three.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pushed = await wrapped(['push', 'origin', 'refs/heads/feat/topic:refs/heads/feat/topic']); // the merge names feat/topic
//! ```

use super::support::{Fixture, Observed, fixture, git, remove, repository, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A repository,
/// its bare origin,
/// a second clone that wins the race,
/// and the two tips that diverged.
struct World {
    /// The fixture that holds everything and gives every command its own home.
    fixture: Fixture,
    /// The bare origin.
    origin: PathBuf,
    /// The pushing repository.
    repo: PathBuf,
    /// The clone another session pushes from.
    rival: PathBuf,
    /// The tip of `feat/topic` in the pushing repository, which the origin lacks.
    local_tip: String,
    /// The tip of `feat/topic` on the origin, which the pushing repository lacks.
    remote_tip: String,
}

/// The commit a ref names in a repository.
fn tip(fixture: &Fixture, repo: &Path, name: &str) -> String {
    let named = git(fixture, repo, &["rev-parse", "--verify", name]);
    return String::from_utf8_lossy(&named.stdout).trim().to_owned();
}

/// The branch a worktree has checked out,
/// or the word for a detached `HEAD`.
fn checked_out(fixture: &Fixture, worktree: &Path) -> String {
    let observed: std::process::Output = super::support::direct(fixture)
        .current_dir(worktree)
        .args(["symbolic-ref", "--quiet", "HEAD"])
        .output()
        .expect("real Git answers");
    if !observed.status.success() {
        return String::from("DETACHED");
    }
    return String::from_utf8_lossy(&observed.stdout).trim().to_owned();
}

/// The files a worktree shows as changed,
/// with untracked files.
fn changes(fixture: &Fixture, worktree: &Path) -> String {
    let listed = git(fixture, worktree, &["status", "--porcelain=v1", "--untracked-files=all"]);
    return String::from_utf8_lossy(&listed.stdout).trim().to_owned();
}

/// The message of the commit a ref names, and its parents.
fn message_of(fixture: &Fixture, repo: &Path, name: &str) -> String {
    let shown = git(fixture, repo, &["log", "--max-count=1", "--format=%P%n%B", name]);
    return String::from_utf8_lossy(&shown.stdout).trim_end().to_owned();
}

/// The last line a caller sees on standard error,
/// which is the line a caller that reads only the last line gets.
fn last_line(observed: &Observed) -> String {
    return stderr_of(observed).trim_end().lines().last().unwrap_or_default().to_owned();
}

/// Write `name`,
/// commit it,
/// and say nothing of policies.
fn committing(fixture: &Fixture, repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("fixture file");
    git(fixture, repo, &["add", "--", name]);
    git(fixture, repo, &["commit", "--quiet", &format!("--message={name}"), "--", name]);
}

/// What:
///  A repository whose `feat/topic` and whose origin's `feat/topic` hold a commit each that the other lacks,
///  with `feat/topic` checked out.
/// Why:
///   That is the state the reported push met,
/// and every control starts from it.
fn diverged(name: &str) -> World {
    let fixture: Fixture = fixture(name);
    let origin: PathBuf = fixture.root.join("origin.git");
    git(&fixture, fixture.root.as_path(), &["init", "--quiet", "--bare", "--initial-branch=main", "origin.git"]);
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    committing(&fixture, repo.as_path(), "base.txt", "base\n");
    git(&fixture, repo.as_path(), &["remote", "add", "origin", origin.to_str().expect("utf-8")]);
    git(&fixture, repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "main"]);
    git(&fixture, repo.as_path(), &["switch", "--quiet", "--create", "feat/topic"]);
    committing(&fixture, repo.as_path(), "feature.txt", "feature one\n");
    committing(&fixture, repo.as_path(), "shared.txt", "shared line\n");
    git(&fixture, repo.as_path(), &["push", "--quiet", "--set-upstream", "origin", "feat/topic"]);
    let rival: PathBuf = fixture.root.join("rival");
    git(&fixture, fixture.root.as_path(), &["clone", "--quiet", origin.to_str().expect("utf-8"), "rival"]);
    git(&fixture, rival.as_path(), &["switch", "--quiet", "feat/topic"]);
    committing(&fixture, rival.as_path(), "rival.txt", "rival line\n");
    git(&fixture, rival.as_path(), &["push", "--quiet", "origin", "feat/topic"]);
    committing(&fixture, repo.as_path(), "local.txt", "local line\n");
    let local_tip: String = tip(&fixture, repo.as_path(), "refs/heads/feat/topic");
    let remote_tip: String = tip(&fixture, origin.as_path(), "refs/heads/feat/topic");
    return World {
        fixture,
        origin,
        repo,
        rival,
        local_tip,
        remote_tip,
    };
}

/// The case as it happened:
/// `feat/topic` is pushed by an explicit refspec while `main` is checked out,
/// and the merge and the line name `feat/topic`,
/// which follows it,
/// while `main` and the worktree stay where they were.
#[test]
fn a_typed_push_of_a_branch_that_is_not_checked_out_is_named_after_that_branch() {
    let world: World = diverged("push-not-checked-out-the-case");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let head_before: String = checked_out(fixture, world.repo.as_path());
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/feat/topic"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    // The published commit is a merge of the two tips.
    assert_eq!(
        message_of(fixture, world.origin.as_path(), published.as_str()).lines().next(),
        Some(format!("{published} {} {}", world.local_tip, world.remote_tip).as_str()),
        "the merge joins the pushed tip and the remote tip"
    );
    let message: String = message_of(fixture, world.origin.as_path(), published.as_str());
    // What: every name in the message is the branch that was pushed, and never the checked-out branch.
    assert!(
        message.contains("Merge origin/feat/topic into feat/topic"),
        "the subject names the pushed branch: {message}"
    );
    assert!(!message.contains("main"), "no line of the message names the checked-out branch: {message}");
    assert!(
        message.contains("Created by monochromatic-git-policy-cli push because origin/feat/topic had commits feat/topic did not have."),
        "the body says what happened in a typed push: {message}"
    );
    assert!(
        !message.contains("advanced after the commit landed"),
        "no commit of the person's was involved: {message}"
    );
    assert!(
        message.contains(
            format!(
                "The first parent is feat/topic at {} and the second origin/feat/topic at {}; nothing was forced.",
                world.local_tip, world.remote_tip
            )
            .as_str()
        ),
        "a reader a year later finds the two tips: {message}"
    );
    assert!(
        message.contains(format!("git revert -m 1 {published}").as_str()),
        "the message says how to undo the merge: {message}"
    );
    assert!(!message.contains("Published to"), "the subject already names the ref it landed on: {message}");
    // The line says what was merged with what, where it was published, and where the branch is now.
    let line: String = last_line(&observed);
    assert!(
        line.starts_with("monochromatic-git-policy-cli: push: origin/feat/topic had commits feat/topic did not have, so the merge of feat/topic and origin/feat/topic was published to origin/feat/topic, and feat/topic is now at it"),
        "the last line answers all three questions: {line}"
    );
    assert!(line.ends_with(format!("(merge {published}).").as_str()), "the merge is named last: {line}");
    // Where every branch and the worktree is now.
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), published, "the pushed branch follows its merge");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before, "no other branch moved");
    assert_eq!(checked_out(fixture, world.repo.as_path()), head_before, "the checkout is where it was");
    assert_eq!(changes(fixture, world.repo.as_path()), "", "the worktree was not written");
    assert_eq!(tip(fixture, world.origin.as_path(), "refs/heads/main"), main_before, "the origin's main is untouched");
    remove(fixture);
}

/// The same push by branch name behaves the same way.
#[test]
fn a_push_naming_the_branch_is_named_after_it() {
    let world: World = diverged("push-not-checked-out-by-name");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let observed: Observed = run_wrapped(fixture, world.repo.as_path(), &["push", "origin", "feat/topic"]);
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    let message: String = message_of(fixture, world.origin.as_path(), published.as_str());
    assert!(message.contains("Merge origin/feat/topic into feat/topic"), "{message}");
    assert!(!message.contains("main"), "{message}");
    assert!(last_line(&observed).contains("and feat/topic is now at it"), "{}", last_line(&observed));
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), published);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_eq!(checked_out(fixture, world.repo.as_path()), String::from("refs/heads/main"));
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// A push made with `HEAD` detached names the ref it publishes to and says that no local branch follows it.
#[test]
fn a_push_with_head_detached_names_no_local_branch() {
    let world: World = diverged("push-not-checked-out-detached");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "--detach", "main"]);
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/feat/topic"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    let message: String = message_of(fixture, world.origin.as_path(), published.as_str());
    assert!(message.contains("Merge origin/feat/topic into feat/topic"), "{message}");
    assert!(!message.contains("main"), "{message}");
    let line: String = last_line(&observed);
    assert!(line.contains("and feat/topic is now at it"), "the pushed branch still follows its merge: {line}");
    assert!(!line.contains("HEAD"), "the line does not talk about the detached checkout: {line}");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), published);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_eq!(checked_out(fixture, world.repo.as_path()), String::from("DETACHED"), "the checkout stays detached");
    assert_eq!(tip(fixture, world.repo.as_path(), "HEAD"), main_before, "the detached head stays where it was");
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// A push typed in a linked worktree names and follows the branch it sends,
/// never the branch that worktree has checked out.
#[test]
fn a_push_from_a_linked_worktree_names_the_branch_it_sends() {
    let world: World = diverged("push-not-checked-out-linked");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let linked: PathBuf = fixture.root.join("linked");
    git(
        fixture,
        world.repo.as_path(),
        &["worktree", "add", "--quiet", "-b", "side", linked.to_str().expect("utf-8"), "main"],
    );
    let observed: Observed = run_wrapped(
        fixture,
        linked.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/feat/topic"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    let message: String = message_of(fixture, world.origin.as_path(), published.as_str());
    assert!(message.contains("Merge origin/feat/topic into feat/topic"), "{message}");
    assert!(!message.contains("side"), "{message}");
    assert!(last_line(&observed).contains("and feat/topic is now at it"), "{}", last_line(&observed));
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), published);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/side"), main_before, "the branch the worktree shows did not move");
    assert_eq!(checked_out(fixture, linked.as_path()), String::from("refs/heads/side"));
    assert_eq!(changes(fixture, linked.as_path()), "");
    remove(fixture);
}

/// A refspec whose two sides are named differently says both names:
/// the branch that was pushed,
/// and the ref the merge was published to.
#[test]
fn a_push_whose_two_sides_are_named_differently_names_both() {
    let world: World = diverged("push-not-checked-out-two-names");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    // `origin/other` starts where the pushed branch and `origin/feat/topic` parted, so both sides hold something
    //  the other lacks.
    git(fixture, world.repo.as_path(), &["push", "--quiet", "origin", "refs/heads/feat/topic~1:refs/heads/other"]);
    git(fixture, world.rival.as_path(), &["fetch", "--quiet", "origin"]);
    git(fixture, world.rival.as_path(), &["switch", "--quiet", "--create", "other", "--track", "origin/other"]);
    committing(fixture, world.rival.as_path(), "other.txt", "other line\n");
    git(fixture, world.rival.as_path(), &["push", "--quiet", "origin", "other"]);
    let other_before: String = tip(fixture, world.origin.as_path(), "refs/heads/other");
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/other"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/other");
    let message: String = message_of(fixture, world.origin.as_path(), published.as_str());
    assert!(message.contains("Merge origin/other into feat/topic"), "{message}");
    assert!(!message.contains("main"), "{message}");
    assert!(message.contains("Published to origin/other."), "the body names the ref it landed on: {message}");
    assert!(
        message.contains(
            format!(
                "The first parent is feat/topic at {} and the second origin/other at {other_before}; nothing was forced.",
                world.local_tip
            )
            .as_str()
        ),
        "{message}"
    );
    let line: String = last_line(&observed);
    assert!(
        line.contains("origin/other had commits feat/topic did not have, so the merge of feat/topic and origin/other was published to origin/other, and feat/topic is now at it"),
        "{line}"
    );
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), published, "the pushed branch follows the merge");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_ne!(published, other_before, "the origin's other moved to the merge");
    assert_eq!(checked_out(fixture, world.repo.as_path()), String::from("refs/heads/main"));
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// Several refspecs in one command are not integrated:
/// Git's own report stands,
/// nothing is published,
/// and no branch or worktree moves.
#[test]
fn several_refspecs_keep_gits_own_report() {
    let world: World = diverged("push-not-checked-out-several");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    git(fixture, world.repo.as_path(), &["push", "--quiet", "origin", "refs/heads/feat/topic~1:refs/heads/other"]);
    git(fixture, world.rival.as_path(), &["fetch", "--quiet", "origin"]);
    git(fixture, world.rival.as_path(), &["switch", "--quiet", "--create", "other", "--track", "origin/other"]);
    committing(fixture, world.rival.as_path(), "other.txt", "other line\n");
    git(fixture, world.rival.as_path(), &["push", "--quiet", "origin", "other"]);
    git(fixture, world.repo.as_path(), &["fetch", "--quiet", "origin"]);
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "--create", "feat/other", "origin/other"]);
    committing(fixture, world.repo.as_path(), "mine-other.txt", "mine other\n");
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let local_before: String = tip(fixture, world.repo.as_path(), "refs/heads/feat/topic");
    let other_local_before: String = tip(fixture, world.repo.as_path(), "refs/heads/feat/other");
    let remote_before: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    let other_remote_before: String = tip(fixture, world.origin.as_path(), "refs/heads/other");
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "feat/topic:refs/heads/feat/topic", "feat/other:refs/heads/other"],
    );
    let said: String = stderr_of(&observed);
    assert_ne!(observed.code, Some(0), "{said}");
    assert!(!said.contains("monochromatic-git-policy-cli: push:"), "no merge is reported: {said}");
    assert_eq!(tip(fixture, world.origin.as_path(), "refs/heads/feat/topic"), remote_before);
    assert_eq!(tip(fixture, world.origin.as_path(), "refs/heads/other"), other_remote_before);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), local_before);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/other"), other_local_before);
    assert_eq!(checked_out(fixture, world.repo.as_path()), String::from("refs/heads/main"));
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// A tag the origin already holds is not a branch and is never reconciled.
#[test]
fn a_tag_push_keeps_gits_own_report() {
    let world: World = diverged("push-not-checked-out-tag");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    git(fixture, world.repo.as_path(), &["tag", "v1", "main"]);
    git(fixture, world.repo.as_path(), &["push", "--quiet", "origin", "v1"]);
    git(fixture, world.repo.as_path(), &["tag", "--force", "v1", "feat/topic"]);
    let tag_before: String = tip(fixture, world.origin.as_path(), "refs/tags/v1");
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let observed: Observed = run_wrapped(fixture, world.repo.as_path(), &["push", "origin", "v1"]);
    let said: String = stderr_of(&observed);
    assert_ne!(observed.code, Some(0), "{said}");
    assert!(said.contains("already exists"), "{said}");
    assert!(!said.contains("monochromatic-git-policy-cli: push:"), "{said}");
    assert_eq!(tip(fixture, world.origin.as_path(), "refs/tags/v1"), tag_before);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), world.local_tip);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// A branch checked out in another clean worktree follows the merge there,
/// and that worktree's own checkout moves with it.
#[test]
fn a_branch_checked_out_in_another_clean_worktree_follows_its_merge() {
    let world: World = diverged("push-not-checked-out-elsewhere-clean");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let linked: PathBuf = fixture.root.join("linked");
    git(
        fixture,
        world.repo.as_path(),
        &["worktree", "add", "--quiet", linked.to_str().expect("utf-8"), "feat/topic"],
    );
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/feat/topic"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    let message: String = message_of(fixture, world.origin.as_path(), published.as_str());
    assert!(message.contains("Merge origin/feat/topic into feat/topic"), "{message}");
    assert!(!message.contains("main"), "{message}");
    let line: String = last_line(&observed);
    assert!(
        line.contains(
            format!(
                "and feat/topic and the worktree at {} are now at it",
                std::fs::canonicalize(linked.as_path()).expect("the linked worktree").display()
            )
            .as_str()
        ) || line.contains("and feat/topic and the worktree at "),
        "{line}"
    );
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), published);
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_eq!(checked_out(fixture, linked.as_path()), String::from("refs/heads/feat/topic"));
    assert_eq!(tip(fixture, linked.as_path(), "HEAD"), published, "the other worktree's checkout followed");
    assert_eq!(changes(fixture, linked.as_path()), "", "and it shows the merge's files with nothing pending");
    assert!(linked.join("rival.txt").exists(), "the merge's file is in that worktree");
    assert_eq!(checked_out(fixture, world.repo.as_path()), String::from("refs/heads/main"));
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// A branch checked out in another worktree with a change the merge does not write follows,
/// and that work keeps its bytes.
#[test]
fn a_branch_checked_out_elsewhere_follows_beside_work_the_merge_does_not_write() {
    let world: World = diverged("push-not-checked-out-elsewhere-dirty");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(
        fixture,
        world.repo.as_path(),
        &["worktree", "add", "--quiet", linked.to_str().expect("utf-8"), "feat/topic"],
    );
    std::fs::write(linked.join("feature.txt"), "an edit the merge does not touch\n").expect("edit");
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/feat/topic"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), published);
    assert_eq!(tip(fixture, linked.as_path(), "HEAD"), published);
    assert_eq!(
        std::fs::read_to_string(linked.join("feature.txt")).expect("the edit"),
        "an edit the merge does not touch\n",
        "the person's edit is intact"
    );
    assert_eq!(changes(fixture, linked.as_path()), "M feature.txt");
    assert!(
        last_line(&observed).contains("that worktree's uncommitted changes were left as they are"),
        "{}",
        last_line(&observed)
    );
    assert!(linked.join("rival.txt").exists(), "the merge was installed there too");
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// A branch checked out in another worktree with a change the merge writes is not moved,
/// and the reason and the way forward are reported.
#[test]
fn a_branch_checked_out_elsewhere_is_left_when_its_work_is_in_the_merges_way() {
    let world: World = diverged("push-not-checked-out-elsewhere-blocked");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let linked: PathBuf = fixture.root.join("linked");
    git(
        fixture,
        world.repo.as_path(),
        &["worktree", "add", "--quiet", linked.to_str().expect("utf-8"), "feat/topic"],
    );
    std::fs::write(linked.join("rival.txt"), "an edit the merge would overwrite\n").expect("edit");
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/feat/topic"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    let line: String = last_line(&observed);
    assert!(
        line.contains("and feat/topic is not at it: the worktree at "),
        "the line names the worktree and the reason: {line}"
    );
    assert!(
        line.contains("has uncommitted changes to files the merge changes: rival.txt"),
        "{line}"
    );
    assert!(line.contains(format!("git merge --ff-only {published}").as_str()), "the line names the step that finishes it: {line}");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), world.local_tip, "the branch is where it was");
    assert_eq!(tip(fixture, linked.as_path(), "HEAD"), world.local_tip);
    assert_eq!(
        std::fs::read_to_string(linked.join("rival.txt")).expect("the edit"),
        "an edit the merge would overwrite\n",
        "the person's edit is intact"
    );
    assert!(!linked.join("local.txt").exists() || changes(fixture, linked.as_path()) == "M rival.txt");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_eq!(changes(fixture, world.repo.as_path()), "");
    remove(fixture);
}

/// A merge that conflicts publishes nothing and changes nothing local,
/// and the sentence names the branch the merge was into.
#[test]
fn a_conflicting_merge_publishes_nothing_and_moves_nothing() {
    let world: World = diverged("push-not-checked-out-conflict");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    // Both sides change the same line of one file differently.
    std::fs::write(world.rival.join("shared.txt"), "shared line from the rival\n").expect("rival edit");
    git(fixture, world.rival.as_path(), &["commit", "--quiet", "--all", "--message=rival shared"]);
    git(fixture, world.rival.as_path(), &["push", "--quiet", "origin", "feat/topic"]);
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "feat/topic"]);
    std::fs::write(world.repo.join("shared.txt"), "shared line from here\n").expect("local edit");
    git(fixture, world.repo.as_path(), &["commit", "--quiet", "--all", "--message=here shared"]);
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    let local_before: String = tip(fixture, world.repo.as_path(), "refs/heads/feat/topic");
    let remote_before: String = tip(fixture, world.origin.as_path(), "refs/heads/feat/topic");
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "refs/heads/feat/topic:refs/heads/feat/topic"],
    );
    let said: String = stderr_of(&observed);
    assert_ne!(observed.code, Some(0), "{said}");
    let line: String = last_line(&observed);
    assert!(
        line.contains("not published: merging origin/feat/topic into feat/topic conflicts in: shared.txt"),
        "the sentence names the pushed branch: {line}"
    );
    assert!(!line.contains("into main"), "{line}");
    assert!(line.contains("Nothing was pushed and no branch or worktree was moved"), "{line}");
    assert_eq!(tip(fixture, world.origin.as_path(), "refs/heads/feat/topic"), remote_before, "nothing was published");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/feat/topic"), local_before, "the pushed branch did not move");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before);
    assert_eq!(checked_out(fixture, world.repo.as_path()), String::from("refs/heads/main"));
    assert_eq!(changes(fixture, world.repo.as_path()), "", "nothing local changed");
    remove(fixture);
}

/// What:
///  A pushed branch that happens to share the checked-out branch's tip is not the checked-out branch,
///  and the checked-out branch is not moved over the merge.
/// Why:
///  The reconcile learned the branch from the command's checkout,
///  so this state moved `main` to a merge of `topic` and `origin/other`
///  and wrote the merge's files into the checkout.
#[test]
fn a_checked_out_branch_at_the_pushed_tip_is_not_moved_over_another_branchs_merge() {
    let world: World = diverged("push-not-checked-out-same-tip");
    let fixture: &Fixture = &world.fixture;
    git(fixture, world.repo.as_path(), &["switch", "--quiet", "main"]);
    // `origin/other` starts at the base and is advanced by the rival, so `topic` (which is `main`) has a commit it lacks.
    git(fixture, world.repo.as_path(), &["push", "--quiet", "origin", "refs/heads/main:refs/heads/other"]);
    git(fixture, world.rival.as_path(), &["fetch", "--quiet", "origin"]);
    git(fixture, world.rival.as_path(), &["switch", "--quiet", "--create", "other", "--track", "origin/other"]);
    committing(fixture, world.rival.as_path(), "other.txt", "other line\n");
    git(fixture, world.rival.as_path(), &["push", "--quiet", "origin", "other"]);
    committing(fixture, world.repo.as_path(), "main.txt", "main line\n");
    git(fixture, world.repo.as_path(), &["branch", "topic", "main"]);
    let main_before: String = tip(fixture, world.repo.as_path(), "refs/heads/main");
    let observed: Observed = run_wrapped(
        fixture,
        world.repo.as_path(),
        &["push", "origin", "topic:refs/heads/other"],
    );
    let said: String = stderr_of(&observed);
    assert_eq!(observed.code, Some(0), "{said}");
    let published: String = tip(fixture, world.origin.as_path(), "refs/heads/other");
    let message: String = message_of(fixture, world.origin.as_path(), published.as_str());
    assert!(message.contains("Merge origin/other into topic"), "{message}");
    assert!(!message.contains("main"), "the merge names the pushed branch and no other: {message}");
    let line: String = last_line(&observed);
    assert!(
        line.contains("origin/other had commits topic did not have"),
        "the line names the pushed branch: {line}"
    );
    assert!(line.contains("and topic is now at it"), "{line}");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/topic"), published, "the pushed branch follows its merge");
    assert_eq!(tip(fixture, world.repo.as_path(), "refs/heads/main"), main_before, "the checked-out branch is not moved");
    assert_eq!(checked_out(fixture, world.repo.as_path()), String::from("refs/heads/main"));
    assert_eq!(changes(fixture, world.repo.as_path()), "", "the checkout was not written");
    assert!(!world.repo.join("other.txt").exists(), "the merge's file is not in this checkout");
    remove(fixture);
}
