//! Controls for the manual-push published set and range against real repositories.
//! The TypeScript corpus is `src/policy-engine/manual-push-scope.unit.test.ts`;
//! the cases that apply to commit
//! pushes are reproduced here (tag,
//! tree and blob targets are not ported and fail closed).

use super::*;
use crate::auto_push_single_flight::PushGit;
use crate::candidate_prediction::{CandidateRequest, PushedScope, prepare_candidates};
use crate::manual_push_probe::PushUpdate;
use crate::test_support::{fixture, git, remove};
use std::path::{Path, PathBuf};

/// Environment of every Git child of these fixtures.
fn fixture_environment() -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = [
        ("PATH", "/usr/bin:/bin"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_GLOBAL", "/nonexistent-global-config"),
        ("GIT_AUTHOR_NAME", "Fixture"),
        ("GIT_AUTHOR_EMAIL", "fixture@example.invalid"),
        ("GIT_COMMITTER_NAME", "Fixture"),
        ("GIT_COMMITTER_EMAIL", "fixture@example.invalid"),
    ]
    .iter()
    .map(|(name, value)| {
        return (OsString::from(name), OsString::from(value));
    })
    .collect();
    // The per-user scratch directory of the child runners is made under a home beside the fixtures.
    environment.extend(crate::test_support::scratch_environment());
    return environment;
}

/// Coordinator Git for a fixture repository.
fn git_for(root: &Path) -> PushGit {
    return PushGit {
        real_git: PathBuf::from("/usr/bin/git"),
        cwd: root.to_path_buf(),
        environment: fixture_environment(),
        lfs: false,
    };
}

/// The commit `HEAD` names.
fn head_of(root: &Path) -> String {
    return String::from_utf8_lossy(&git(root, &["rev-parse", "HEAD"]).stdout)
        .trim()
        .to_owned();
}

/// A repository with a commit on `main` and a bare origin that has it published.
fn published_repository(name: &str) -> (PathBuf, String) {
    let root = fixture(name);
    let origin = root.join("origin.git");
    let repo = root.join("repo");
    git(
        &root,
        &["init", "--quiet", "--bare", origin.to_str().expect("utf-8")],
    );
    std::fs::create_dir_all(&repo).expect("repo");
    git(&repo, &["init", "--quiet", "--initial-branch=main"]);
    git(&repo, &["config", "user.email", "cli-git@example.invalid"]);
    git(&repo, &["config", "user.name", "monochromatic-git-policy-cli fixture"]);
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=base"],
    );
    git(
        &repo,
        &["remote", "add", "origin", origin.to_str().expect("utf-8")],
    );
    git(
        &repo,
        &["push", "--quiet", "origin", "HEAD:refs/heads/main"],
    );
    let base = head_of(&repo);
    return (repo, base);
}

/// An update to `origin`'s `main` from `local` over `prior`.
fn update(local: Option<&str>, prior: Option<&str>) -> PushUpdate {
    return PushUpdate {
        local_oid: local.map(str::to_owned),
        remote_oid: prior.map(str::to_owned),
        remote_name: "origin".to_owned(),
        remote_ref: "refs/heads/main".to_owned(),
    };
}

/// Object names are peeled only when Git has them locally and they are commits.
#[test]
fn only_locally_present_commits_are_peeled() {
    let (repo, base) = published_repository("mp-range-peel");
    let missing = "f".repeat(40);
    let peeled = peel_commits(&git_for(&repo), &[base.clone(), missing.clone()]).expect("peels");
    assert_eq!(peeled.get(&base), Some(&base));
    assert!(!peeled.contains_key(&missing));
    remove(repo.parent().expect("root"));
}

/// Tracking refs count only in their own remote's namespace,
/// matched literally.
#[test]
fn tracking_refs_are_matched_per_remote_literally() {
    let (repo, base) = published_repository("mp-range-tracking");
    git(&repo, &["update-ref", "refs/remotes/origin/main", &base]);
    git(&repo, &["update-ref", "refs/remotes/other/main", &base]);
    let tracked = tracking_commits(&git_for(&repo), &["origin".to_owned(), "ori*".to_owned()])
        .expect("lists");
    assert_eq!(tracked.get("origin"), Some(&vec![base.clone()]));
    assert_eq!(tracked.get("ori*"), Some(&Vec::new()));
    remove(repo.parent().expect("root"));
}

/// A prior destination value joins the published tips when it is a locally present commit.
#[test]
fn prior_destination_values_join_the_published_tips() {
    let (repo, base) = published_repository("mp-range-prior");
    let mut prior: BTreeMap<String, String> = BTreeMap::new();
    prior.insert(base.clone(), base.clone());
    let tracking: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let tips = published_tips(&[update(Some(&base), Some(&base))], &prior, &tracking);
    assert_eq!(tips.get("origin"), Some(&vec![base.clone()]));
    remove(repo.parent().expect("root"));
}

/// A commit whose base is published scans only the commits the remote lacks.
#[test]
fn a_known_base_scans_only_the_new_commits() {
    let (repo, base) = published_repository("mp-range-new");
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=one"],
    );
    let one = head_of(&repo);
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=two"],
    );
    let two = head_of(&repo);
    let scope = update_scope(
        &git_for(&repo),
        &update(Some(&two), Some(&base)),
        std::slice::from_ref(&base),
    )
    .expect("scopes");
    assert_eq!(scope, UpdateScope::Commits(vec![two.clone(), one.clone()]));
    remove(repo.parent().expect("root"));
}

/// With no published base,
/// the pushed tip's whole tree is the scope.
#[test]
fn an_unknown_base_scans_the_final_tree() {
    let (repo, _base) = published_repository("mp-range-final");
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=one"],
    );
    let tip = head_of(&repo);
    let scope = update_scope(&git_for(&repo), &update(Some(&tip), None), &[]).expect("scopes");
    assert_eq!(scope, UpdateScope::FinalTree(tip.clone()));
    remove(repo.parent().expect("root"));
}

/// A deletion publishes nothing and has no scope.
#[test]
fn a_deletion_has_no_scope() {
    let (repo, base) = published_repository("mp-range-delete");
    let scope = update_scope(
        &git_for(&repo),
        &update(None, Some(&base)),
        std::slice::from_ref(&base),
    )
    .expect("scopes");
    assert_eq!(scope, UpdateScope::None);
    remove(repo.parent().expect("root"));
}

/// A pushed object that is not a commit fails closed,
/// never an empty scope.
#[test]
fn a_non_commit_target_fails_closed() {
    let (repo, _base) = published_repository("mp-range-tree");
    let tree = String::from_utf8_lossy(&git(&repo, &["rev-parse", "HEAD^{tree}"]).stdout)
        .trim()
        .to_owned();
    let error = update_scope(&git_for(&repo), &update(Some(&tree), None), &[]).expect_err("a tree is not a commit");
    assert_eq!(error.cause, ManualPushCause::UnsupportedObject, "{error}");
    remove(repo.parent().expect("root"));
}

/// The pushed scope lists each new commit's own delta and each tip tree's blobs,
/// with no staged state.
#[test]
fn the_pushed_scope_lists_commit_deltas_and_tree_blobs() {
    let (repo, base) = published_repository("mp-range-candidates");
    git(
        &repo,
        &["commit", "--quiet", "--allow-empty", "--message=empty"],
    );
    std::fs::write(repo.join("pushed.txt"), "pushed content\n").expect("pushed file");
    git(&repo, &["add", "pushed.txt"]);
    git(&repo, &["commit", "--quiet", "-m", "pushed"]);
    let tip = head_of(&repo);
    let real_git = PathBuf::from("/usr/bin/git");
    let prefix: Vec<OsString> = vec![OsString::from("-C"), repo.as_os_str().to_os_string()];
    let overlay = fixture_environment();
    let commit_scope = PushedScope {
        commits: vec![
            crate::candidate_object::parse_object_id(tip.as_bytes()).expect("object name"),
        ],
        trees: Vec::new(),
    };
    let prepared = prepare_candidates(
        &real_git,
        &prefix,
        &overlay,
        &CandidateRequest::Pushed(commit_scope),
    )
    .expect("commit candidates");
    let paths: Vec<String> = prepared
        .version
        .candidates()
        .iter()
        .map(|candidate| {
            return String::from_utf8_lossy(&candidate.path).into_owned();
        })
        .collect();
    assert_eq!(paths, vec!["pushed.txt".to_owned()], "{paths:?}");
    let tree_scope = PushedScope {
        commits: Vec::new(),
        trees: vec![crate::candidate_object::parse_object_id(tip.as_bytes()).expect("object name")],
    };
    let whole = prepare_candidates(
        &real_git,
        &prefix,
        &overlay,
        &CandidateRequest::Pushed(tree_scope),
    )
    .expect("tree candidates");
    let mut all: Vec<String> = whole
        .version
        .candidates()
        .iter()
        .map(|candidate| {
            return String::from_utf8_lossy(&candidate.path).into_owned();
        })
        .collect();
    all.sort();
    assert_eq!(all, vec!["pushed.txt".to_owned()], "{all:?}");
    assert!(!base.is_empty());
    remove(repo.parent().expect("root"));
}

/// The range error prints as its diagnostic,
/// so a refused push names the failed query.
#[test]
fn the_range_error_displays_its_message() {
    let error = ManualPushRangeError {
        message: String::from("the range query failed"),
        cause: ManualPushCause::SelfExplained,
    };
    assert_eq!(error.to_string(), "the range query failed");
}

/// What:
///  A request list larger than a pipe is written while the answers are read,
/// so the query returns.
/// Why:
///   Git reads its requests and answers in one loop.
/// The wrapper wrote the whole list before it read anything,
/// so a
///  reply pipe that filled stopped Git reading the list,
/// and the write waited for the reply for good
/// (`doc/planning/cli-git-process-and-child-lifetime-route-findings-on-main.md`,
/// finding 9).
/// The stand-in fills its reply pipe and only then reads its requests,
/// which is what a Git answering a large batch
///  does;
/// the list is larger than a pipe,
/// and the wait is bounded,
/// so a deadlock is reported rather than hanging the run.
#[test]
fn a_request_list_larger_than_a_pipe_is_written_while_the_answers_are_read() {
    let root: PathBuf = fixture("manual-push-range-pipes");
    let program: PathBuf = root.join("stand-in-git");
    let pid_file: PathBuf = root.join("stand-in-pid");
    // The stand-in answers more than a pipe holds before it reads a request; it then reads the whole list and answers once.
    std::fs::write(
        &program,
        format!("#!/bin/sh\necho $$ > '{}'\nprintf '%300000s' ''\ncat > /dev/null\nprintf 'answered\\n'\n", pid_file.display()),
    )
    .expect("the stand-in");
    let mut mode = std::fs::metadata(&program).expect("the stand-in's metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o755);
    std::fs::set_permissions(&program, mode).expect("the stand-in runs");
    let git: PushGit = PushGit { real_git: program.clone(), cwd: root.clone(), environment: fixture_environment(), lfs: false };
    // One line per destination, far more than a pipe of 64 KiB holds.
    let requests: String = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n".repeat(2000);
    let (sender, receiver) = std::sync::mpsc::channel::<Result<Vec<u8>, ManualPushRangeError>>();
    // The query runs on a thread of its own, so a deadlock is reported by this control rather than hanging it.
    let querying = {
        let for_the_query: PushGit = git.clone();
        let list: String = requests;
        std::thread::Builder::new()
            .name(String::from("manual-push-range-control"))
            .spawn(move || {
                let arguments: Vec<OsString> = ["cat-file", "--batch-check"].iter().map(OsString::from).collect();
                let _abandoned = sender.send(run_git_bytes(&for_the_query, arguments.as_slice(), Some(list.as_str())));
            })
            .expect("the control's thread")
    };
    let answer: Result<Result<Vec<u8>, ManualPushRangeError>, std::sync::mpsc::RecvTimeoutError> = receiver.recv_timeout(Duration::from_secs(20));
    // The stand-in is ended whatever happened, so a query that deadlocked is released and the control's thread ends.
    let stand_in: Result<String, std::io::Error> = std::fs::read_to_string(&pid_file);
    if let Ok(text) = stand_in.as_ref()
        && let Ok(number) = text.trim().parse::<libc::pid_t>()
    {
        // SAFETY: the number is the stand-in's own, which it wrote; `kill` takes two integers and touches no memory.
        let _ended = unsafe { libc::kill(number, libc::SIGKILL) };
    }
    let _joined = querying.join();
    remove(root.as_path());
    let returned = answer.expect("the query returned; a full request pipe deadlocked it");
    let bytes: Vec<u8> = returned.expect("the query answers");
    // The stand-in's answer is what it wrote before it read the list, and the line it wrote after it.
    assert_eq!(bytes.len(), 300_000 + "answered\n".len(), "the query read every byte the stand-in wrote");
    assert!(bytes.ends_with(b"answered\n"), "the query read the answer that came after the request list: {} bytes", bytes.len());
}
