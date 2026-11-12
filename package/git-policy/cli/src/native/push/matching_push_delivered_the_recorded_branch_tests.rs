//! Controls for the judgement of a matching push's status:
//! the ref a branch is sent to,
//! which pushes leave the answer to the remote,
//! the reading of the remote's own listing,
//! and that a remote which cannot be asked proves nothing.

use super::*;
use crate::test_support::{fixture, git, git_output, remove};
use std::path::PathBuf;

/// A branch of `refs/heads` is sent to the ref of its own name.
#[test]
fn a_branch_of_refs_heads_is_sent_to_the_ref_of_its_own_name() {
    assert_eq!(remote_ref_of("refs/heads/main"), Some(String::from("refs/heads/main")));
    assert_eq!(remote_ref_of("refs/heads/topic/one"), Some(String::from("refs/heads/topic/one")));
}

/// A ref no matching push sends has no answer to ask for.
#[test]
fn a_ref_outside_refs_heads_is_sent_nowhere() {
    assert_eq!(remote_ref_of("refs/tags/v1"), None);
    assert_eq!(remote_ref_of("HEAD"), None);
}

/// Only a matching push that failed leaves the recorded branch's fate to the remote.
#[test]
fn only_a_matching_push_that_failed_asks_the_remote() {
    assert!(asks_the_remote(true, 1));
    assert!(!asks_the_remote(true, 0));
    assert!(!asks_the_remote(false, 1));
    assert!(!asks_the_remote(false, 0));
}

/// A listing answers with the object of the ref it was asked about,
/// and nothing for a ref the listing does not name.
#[test]
fn the_listing_answers_with_the_object_of_the_ref_it_names() {
    let listing: &str = "3f786850e387550fdab836ed7e6dc881de23001b\trefs/heads/main\n89e6c98d92887913ead7cdc37f937e1cc95807c6\trefs/heads/topic\n";
    assert_eq!(object_named_by(listing, "refs/heads/topic"), Some(String::from("89e6c98d92887913ead7cdc37f937e1cc95807c6")));
    assert_eq!(object_named_by(listing, "refs/heads/other"), None);
    assert_eq!(object_named_by("", "refs/heads/main"), None);
    assert_eq!(object_named_by("not a listing\n", "refs/heads/main"), None);
}

/// The lines a reader is shown name the refs the remote refused,
/// and a push the remote accepted has none.
#[test]
fn the_refused_refs_are_the_lines_that_say_so() {
    let refused: &str = "To /nowhere/remote.git\n   3f78685..89e6c98  main -> main\n ! [rejected]        topic -> topic (non-fast-forward)\n ! [remote rejected] other -> other (hook declined)\nerror: failed to push some refs\n";
    assert_eq!(
        refused_lines(refused),
        vec![" ! [rejected]        topic -> topic (non-fast-forward)", " ! [remote rejected] other -> other (hook declined)"]
    );
    assert!(refused_lines("To /nowhere/remote.git\n   abc..def  main -> main\n").is_empty());
}

/// A remote's push URLs are its own lines,
/// one per URL,
/// and a remote with none names nothing.
#[test]
fn the_push_urls_are_the_lines_that_name_one() {
    assert_eq!(urls_of("https://one.example.invalid/r.git\n"), vec!["https://one.example.invalid/r.git"]);
    assert_eq!(urls_of("  /first/path.git  \n/second/path.git\n"), vec!["/first/path.git", "/second/path.git"]);
    assert!(urls_of("\n  \n").is_empty());
}

/// Every push URL must prove the commit's arrival,
/// by holding it or a commit that contains it,
/// and no URL proves nothing.
#[test]
fn every_push_url_must_prove_the_delivery() {
    let yes: Option<bool> = Some(true);
    let no: Option<bool> = Some(false);
    assert!(every_url_proved_it(std::slice::from_ref(&yes)), "one URL that holds it");
    assert!(every_url_proved_it(&[yes, yes]), "two URLs that both hold it");
    assert!(!every_url_proved_it(&[yes, no]), "one URL whose ref does not hold it");
    assert!(!every_url_proved_it(&[yes, None]), "one URL whose ancestry could not be answered");
    assert!(!every_url_proved_it(&[]), "no URL at all");
}

/// A remote the repository names answers with its own push URLs,
/// and one it does not name is the URL the push went to,
/// so that URL is the one asked.
#[test]
fn the_urls_are_the_listing_or_the_remote_itself() {
    assert_eq!(
        urls_the_push_went_to(Some("/first/path.git\n/second/path.git\n"), "origin"),
        vec!["/first/path.git", "/second/path.git"]
    );
    assert_eq!(urls_the_push_went_to(None, "/somewhere/published.git"), vec!["/somewhere/published.git"]);
    assert!(urls_the_push_went_to(Some("\n  \n"), "origin").is_empty(), "a listing that names nothing asks nothing");
}

/// A repository with `main` published to a local bare remote,
/// and the commit `main` holds there.
fn published(name: &str) -> (PathBuf, PushGit, String) {
    let root: PathBuf = fixture(name);
    let remote: PathBuf = root.join("remote.git");
    let repo: PathBuf = root.join("repo");
    git(&root, &["init", "--bare", "--quiet", "--initial-branch=main", "remote.git"]);
    git(&root, &["init", "--quiet", "--initial-branch=main", "repo"]);
    git(&repo, &["commit", "--quiet", "--allow-empty", "--message=seed"]);
    git(&repo, &["remote", "add", "origin", remote.to_str().expect("utf-8")]);
    git(&repo, &["push", "--quiet", "--set-upstream", "origin", "main"]);
    let landed: String = String::from_utf8_lossy(&git(&repo, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    let push_git: PushGit = PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: repo,
        environment: crate::test_support::scratch_environment(),
        lfs: false,
    };
    return (root, push_git, landed);
}

/// The commit the remote's ref names is a delivery the remote proved.
#[test]
fn the_commit_the_remote_names_was_delivered() {
    let (root, push_git, landed): (PathBuf, PushGit, String) = published("mpdtr-delivered");
    let push: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "origin", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    assert!(delivered_although_git_failed(&push_git, &push).expect("the question is put"), "the remote names the commit");
    remove(&root);
}

/// What:
///  A remote's branch that holds the commit as one of its ancestors proves the delivery,
///  although the ref names a child of it.
/// Why:
///  A matching push names no refspec,
/// so what it carried is the branch's own tip,
/// which can be a child of the commit the wrapper fixed before the push;
///  and another client can move the branch past that commit between the push and this question.
/// The commit is on the remote either way,
/// which is what the backup promises,
/// and an answer that demanded the ref name the commit itself left the push a failure and its retry planned for good.
#[test]
fn a_remote_naming_a_child_of_the_commit_proves_the_delivery() {
    let (root, push_git, landed): (PathBuf, PushGit, String) = published("mpdtr-contains-locally");
    git(&push_git.cwd, &["commit", "--quiet", "--allow-empty", "--message=later work"]);
    git(&push_git.cwd, &["push", "--quiet", "origin", "HEAD:refs/heads/main"]);
    let advanced: String = String::from_utf8_lossy(&git(&push_git.cwd, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    assert!(
        git_output(&push_git.cwd, &["merge-base", "--is-ancestor", landed.as_str(), advanced.as_str()]).status.success(),
        "the remote's branch holds the commit as an ancestor"
    );
    let push: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "origin", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    assert!(delivered_although_git_failed(&push_git, &push).expect("the question is put"), "the remote's branch holds the commit");
    remove(&root);
}

/// What:
///  A remote whose branch holds the commit beyond an object this repository never fetched proves the delivery,
///  and the question leaves the repository as it found it.
/// Why:
///  Another client's commit is not among this repository's objects,
/// so the branch's own ref is fetched into the reconcile's private ref to answer;
///  that ref is scaffolding and is deleted,
/// and `FETCH_HEAD` is the one file every fetch in a repository shares,
/// which another command's `git pull` reads,
/// so the fetch writes none.
#[test]
fn a_remote_whose_branch_holds_an_unfetched_child_proves_the_delivery() {
    let (root, push_git, landed): (PathBuf, PushGit, String) = published("mpdtr-contains-remotely");
    let elsewhere: PathBuf = root.join("elsewhere");
    // Another client of the remote, which is the bare repository `published` made beside the clone under test.
    git(&root, &["clone", "--quiet", root.join("remote.git").to_str().expect("utf-8"), elsewhere.to_str().expect("utf-8")]);
    git(&elsewhere, &["commit", "--quiet", "--allow-empty", "--message=another client"]);
    let advanced: String = String::from_utf8_lossy(&git(&elsewhere, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    git(&elsewhere, &["push", "--quiet", "origin", "HEAD:refs/heads/main"]);
    // The positive control: the commit that holds the one this push sent is not here until the question fetches it.
    assert!(
        !git_output(&push_git.cwd, &["cat-file", "-e", format!("{advanced}^{{commit}}").as_str()]).status.success(),
        "another client's commit is not in this repository"
    );
    let push: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "origin", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    assert!(delivered_although_git_failed(&push_git, &push).expect("the question is put"), "the remote's branch holds the commit");
    assert!(
        git_output(&push_git.cwd, &["cat-file", "-e", format!("{advanced}^{{commit}}").as_str()]).status.success(),
        "the question fetched the object it needed to answer"
    );
    assert_eq!(
        String::from_utf8_lossy(&git(&push_git.cwd, &["for-each-ref", "--format=%(refname)", "refs/cli-git/auto-push/"]).stdout).trim(),
        "",
        "the private fetch ref the question made is gone"
    );
    assert!(!push_git.cwd.join(".git").join("FETCH_HEAD").exists(), "the file every fetch shares is left alone");
    remove(&root);
}

/// A remote that names another object proves nothing,
/// and neither does a push whose status already says what became of the branch.
#[test]
fn a_remote_naming_another_object_proves_nothing() {
    let (root, push_git, landed): (PathBuf, PushGit, String) = published("mpdtr-not-delivered");
    git(&push_git.cwd, &["commit", "--quiet", "--allow-empty", "--message=unpublished"]);
    let unpublished: String = String::from_utf8_lossy(&git(&push_git.cwd, &["rev-parse", "HEAD"]).stdout).trim().to_owned();
    let behind: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "origin", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: unpublished.as_str() };
    assert!(!delivered_although_git_failed(&push_git, &behind).expect("the question is put"), "a commit the remote does not name");
    let done: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "origin", branch_ref: "refs/heads/main", exit_code: 0, landed_oid: landed.as_str() };
    assert!(!delivered_although_git_failed(&push_git, &done).expect("the question is put"), "a push Git reported as done");
    let named: MatchingPush<'_> = MatchingPush { git_matched: false, remote: "origin", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    assert!(!delivered_although_git_failed(&push_git, &named).expect("the question is put"), "a push that named its refspec");
    remove(&root);
}

/// A remote that cannot be asked proves nothing,
/// so the push keeps the failure it was.
#[test]
fn a_remote_that_cannot_be_asked_proves_nothing() {
    let (root, push_git, landed): (PathBuf, PushGit, String) = published("mpdtr-unreachable");
    let missing: PathBuf = root.join("missing.git");
    git(&push_git.cwd, &["remote", "add", "nowhere", missing.to_str().expect("utf-8")]);
    let unreachable: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "nowhere", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    assert!(!delivered_although_git_failed(&push_git, &unreachable).expect("the question is put"));
    remove(&root);
}

/// The awaited face answers what the synchronous one answers,
/// for a delivery and for a question that proves nothing.
#[test]
fn the_awaited_face_answers_as_the_synchronous_one() {
    let (root, push_git, landed): (PathBuf, PushGit, String) = published("mpdtr-awaited");
    let missing: PathBuf = root.join("missing.git");
    git(&push_git.cwd, &["remote", "add", "nowhere", missing.to_str().expect("utf-8")]);
    let delivered: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "origin", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    let unproven: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "nowhere", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    let answered = crate::lazy_runtime::run_on_runtime(async {
        let one: bool = delivered_although_git_failed_awaiting(&push_git, &delivered).await.expect("the question is put");
        let other: bool = delivered_although_git_failed_awaiting(&push_git, &unproven).await.expect("the question is put");
        return (one, other);
    })
    .expect("the runtime answers");
    assert_eq!(answered, (true, false));
    remove(&root);
}

/// What:
///  The awaited face proves a delivery the remote's own object is needed for,
///  and leaves the file every fetch shares alone as the synchronous one does.
/// Why:
///  The service's push is a task of the runtime and asks this face,
/// so the fetch that proves an ancestry beyond this repository's objects is made as a task:
///  a synchronous query there would be refused and the delivery would stay unproven,
/// which is the false failure this face's control exists for.
#[test]
fn the_awaited_face_proves_a_delivery_that_needs_the_remotes_own_object() {
    let (root, push_git, landed): (PathBuf, PushGit, String) = published("mpdtr-awaited-contains");
    let elsewhere: PathBuf = root.join("elsewhere");
    git(&root, &["clone", "--quiet", root.join("remote.git").to_str().expect("utf-8"), elsewhere.to_str().expect("utf-8")]);
    git(&elsewhere, &["commit", "--quiet", "--allow-empty", "--message=another client"]);
    git(&elsewhere, &["push", "--quiet", "origin", "HEAD:refs/heads/main"]);
    let push: MatchingPush<'_> = MatchingPush { git_matched: true, remote: "origin", branch_ref: "refs/heads/main", exit_code: 1, landed_oid: landed.as_str() };
    let answered = crate::lazy_runtime::run_on_runtime(async {
        return delivered_although_git_failed_awaiting(&push_git, &push).await.expect("the question is put");
    })
    .expect("the runtime answers");
    assert!(answered, "the remote's branch holds the commit this push sent");
    assert!(!push_git.cwd.join(".git").join("FETCH_HEAD").exists(), "the file every fetch shares is left alone");
    assert_eq!(
        String::from_utf8_lossy(&git(&push_git.cwd, &["for-each-ref", "--format=%(refname)", "refs/cli-git/auto-push/"]).stdout).trim(),
        "",
        "the private fetch ref the question made is gone"
    );
    remove(&root);
}
