//! Controls for the awaited Git steps of a landing:
//! they run what the synchronous steps run,
//! choose the requests the synchronous ones
//!  choose,
//! keep the side reference the synchronous ones keep,
//! and do all of it inside the runtime,
//! where the
//!  synchronous faces cannot start Git at all.

use super::*;
use crate::service_protocol::CommitRequest;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::path::PathBuf;

/// What:
///  One request of the batch a control lands,
///  with the base it was prepared on and the path it changes.
/// Why:
///  The fold the two faces share reads the base,
/// the captured stamp and the paths of a request,
/// so a control
///  makes those itself and needs no preparation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { id, repository, commonDir, gitDir, reference: 'refs/heads/main', base, changes: [{ path, blob: null, mode: 0o100644, ... }], message, author: null, committer, amend: false, tree: null, commit: null, cleanup: null, index, scratch, captured, holder: '' };
/// ```
fn a_request(repository: &Path, base: &str, path: &str, captured: u64) -> CommitRequest {
    let common: PathBuf = repository.join(".git");
    return CommitRequest {
        id: format!("request-{captured}-{path}"),
        repository: repository.to_path_buf(),
        common_dir: common.clone(),
        git_dir: common.clone(),
        reference: crate::ref_name::RefName::from_text("refs/heads/main"),
        base: base.to_string(),
        changes: vec![crate::service_protocol::Change { path: PathBuf::from(path), blob: None, mode: 0o100644 }],
        message: String::from("a control\n"),
        author: None,
        committer: String::from("A Control <control@example.invalid> 1793535889 +0000"),
        amend: false,
        tree: None,
        commit: None,
        cleanup: None,
        index: common.join("index"),
        scratch: common.join("monochromatic-git-policy-cli-service-a-control"),
        captured,
        holder: String::new(),
    };
}

/// What:
///  One step's answer through both faces,
///  so a control compares them without asking which is which.
/// Why:
///  The two faces run one command for one person,
/// so a difference here is a step that reads or writes another
///  repository once the landing is a task.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [sync, awaited] = bothSteps();
/// ```
fn both_run(context: &GitContext, repo: &Path, arguments: &[&str]) -> (Result<GitOutput, FailureCode>, Result<GitOutput, FailureCode>) {
    let synchronous: Result<GitOutput, FailureCode> = run(context, repo, arguments, None);
    let awaited: Result<GitOutput, FailureCode> = crate::lazy_runtime::run_on_runtime(run_awaiting(context, repo, arguments, None)).expect("the runtime");
    return (synchronous, awaited);
}

/// What:
///  The awaited steps run what the synchronous steps run,
///  and they run it inside the runtime,
///  where the synchronous
///  steps cannot start Git at all.
/// Why:
///  A step that ran nothing would read an empty answer as an answer,
/// and one that ran another command would read
///  another repository's names;
/// the text and the bytes are the two readings a landing makes of a step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.equal(await checkedAwaiting(git, repo, ['rev-parse', '--verify', '--quiet', 'HEAD']), checked(git, repo, [...]));
/// ```
#[test]
fn the_awaited_steps_run_what_the_synchronous_ones_run() {
    let root: PathBuf = fixture("land-steps-awaiting-runs");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let (synchronous, awaited) = both_run(&context, repo.as_path(), &["rev-parse", "--verify", "--quiet", "HEAD"]);
    let output: GitOutput = synchronous.clone().expect("the step runs");
    assert!(output.succeeded(), "the synchronous step asks a question Git answers");
    assert_eq!(
        awaited.expect("the awaited step runs").stdout,
        output.stdout,
        "the two faces run one step for one commit"
    );
    let synchronous_text: String = checked(&context, repo.as_path(), &["symbolic-ref", "--quiet", "HEAD"], None).expect("the branch");
    let awaited_text: String = crate::lazy_runtime::run_on_runtime(checked_awaiting(&context, repo.as_path(), &["symbolic-ref", "--quiet", "HEAD"], None))
        .expect("the runtime")
        .expect("the branch");
    assert_eq!(awaited_text, synchronous_text, "the two faces read the same text of a step that must succeed");
    let listing: Vec<&str> = vec!["log", "-1", "--format=%H"];
    let synchronous_bytes: Vec<u8> = checked_bytes(&context, repo.as_path(), listing.as_slice(), None).expect("the listing");
    let awaited_bytes: Vec<u8> = crate::lazy_runtime::run_on_runtime(checked_bytes_awaiting(&context, repo.as_path(), listing.as_slice(), None))
        .expect("the runtime")
        .expect("the listing");
    assert_eq!(awaited_bytes, synchronous_bytes, "the two faces read the same bytes of a listing");
    remove(root.as_path());
}

/// What:
///  A step Git cannot run is the failure the synchronous step stands for,
///  and a step that must succeed says so the same way.
/// Why:
///  The code is what the client is told,
/// so a face that answered another code would report another problem than the one
///  that stopped the step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const blind = { ...gitContext, realGit: '/nonexistent/git' }; assert.deepEqual(await runAwaiting(blind, repo, args), run(blind, repo, args));
/// ```
#[test]
fn a_step_git_cannot_run_is_the_answer_both_faces_give() {
    let root: PathBuf = fixture("land-steps-awaiting-refused");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let mut context: GitContext = git_context();
    context.real_git = PathBuf::from("/nonexistent-git-for-a-control");
    let (synchronous, awaited) = both_run(&context, repo.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(synchronous, Err(FailureCode::Unavailable), "a Git that cannot be run is a step that cannot run");
    assert_eq!(awaited, synchronous, "both faces answer a step that could not run the same way");
    let synchronous_checked: Result<String, FailureCode> = checked(&context, repo.as_path(), &["rev-parse", "HEAD"], None);
    let awaited_checked: Result<String, FailureCode> = crate::lazy_runtime::run_on_runtime(checked_awaiting(&context, repo.as_path(), &["rev-parse", "HEAD"], None)).expect("the runtime");
    assert_eq!(awaited_checked, synchronous_checked, "both faces answer a step that must succeed the same way");
    remove(root.as_path());
}

/// What:
///  The requests one batch lands are the ones both faces choose,
///  for a request whose paths changed since its
///  base,
///  one that an earlier member overlaps and one that captured the worktree later.
/// Why:
///  A request left in or out decides what lands together,
/// so a face that folded the listing differently would land
///  one request against another's work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// assert.deepEqual(await requestsThatLandAwaiting(git, repo, requests, old), requestsThatLand(git, repo, requests, old));
/// ```
#[test]
fn the_awaited_requests_that_land_are_the_ones_the_synchronous_face_chooses() {
    let root: PathBuf = fixture("land-steps-awaiting-landing");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let old: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    std::fs::write(repo.join("changed.txt"), "one\n").expect("a file");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=the branch moved"]);
    // A request prepared on the branch as it was, whose path the branch changed since.
    let behind: CommitRequest = a_request(repo.as_path(), old.as_str(), "changed.txt", 10);
    // Two requests of one path: the later capture holds the earlier one's edit, so both land.
    let earlier: CommitRequest = a_request(repo.as_path(), git_text(repo.as_path(), &["rev-parse", "HEAD"]).as_str(), "earlier.txt", 20);
    let later: CommitRequest = a_request(repo.as_path(), git_text(repo.as_path(), &["rev-parse", "HEAD"]).as_str(), "earlier.txt", 30);
    let requests: Vec<CommitRequest> = vec![behind, earlier, later];
    let tip: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let synchronous: Vec<(usize, &CommitRequest)> = requests_that_land(&context, repo.as_path(), requests.as_slice(), tip.as_str()).expect("the landing requests");
    let awaited: Vec<(usize, &CommitRequest)> =
        crate::lazy_runtime::run_on_runtime(requests_that_land_awaiting(&context, repo.as_path(), requests.as_slice(), tip.as_str()))
            .expect("the runtime")
            .expect("the landing requests");
    let positions = |landing: &[(usize, &CommitRequest)]| -> Vec<usize> { return landing.iter().map(|(index, _request)| { return *index; }).collect() };
    assert_eq!(positions(&synchronous), vec![1, 2], "a request whose path the branch changed since is left out, and one that captured later lands");
    assert_eq!(positions(&awaited), positions(&synchronous), "the two faces choose one set of requests to land");
    remove(root.as_path());
}
