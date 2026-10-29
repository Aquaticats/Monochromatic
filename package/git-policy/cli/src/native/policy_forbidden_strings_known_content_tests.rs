//! What:
//!  Controls for the forbidden-strings policy's rule that content already in a parent commit's tree is not
//!       scanned again:
//!  a pure rename or a copy of an existing blob is allowed,
//!  any changed byte, any new file and any
//!       root commit is scanned,
//!  and a merge or push is judged against the parents that were already published.
//! Why:
//!  A rename re-lists its file as an addition,
//!  so scanning the bytes again refused moves of files that
//!      already sit in history,
//!  while skipping too much would let new secrets through.
//!  Every control uses real Git on a disposable repository and a rules file holding one needle.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await check(renamed)).toEqual([]); expect(await check(renamedAndEdited)).toHaveLength(1);
//! ```

use super::{
    CandidateRequest, LifecycleContent, PolicyFinding, PolicyOutcome, ScannerSettings,
    add, check, content_match, git, needle, repository, run_isolated, settings,
};
use crate::candidate_object::{ObjectId, parse_object_id};
use crate::candidate_prediction::PushedScope;
use crate::event_path::EventPath;
use crate::policy_forbidden_strings::FORBIDDEN_STRING_CODE;
use crate::scanner_test_support::rules_file;
use crate::test_support::git_text;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The outcome that reports nothing.
fn clean() -> PolicyOutcome {
    return PolicyOutcome::Findings(Vec::new());
}

/// Stage everything and commit it through the fixture Git.
fn commit_all(repo: &Path, message: &str) {
    git(repo, &["add", "--all"]);
    git(repo, &["commit", "--quiet", "--message", message]);
}

/// The object name `revision` resolves to in `repo`.
fn oid(repo: &Path, revision: &str) -> ObjectId {
    let text: String = git_text(repo, &["rev-parse", revision]);
    return parse_object_id(text.as_bytes()).expect("an object name");
}

/// The candidates of one landed commit.
fn landed(repo: &Path, revision: &str) -> LifecycleContent {
    return LifecycleContent::Requested(CandidateRequest::Landed(oid(repo, revision)));
}

/// The candidates of a push of these commits and tip trees.
fn pushed(commits: Vec<ObjectId>, trees: Vec<ObjectId>) -> LifecycleContent {
    return LifecycleContent::Requested(CandidateRequest::Pushed(PushedScope { commits, trees }));
}

/// A repository whose `HEAD` holds `a.txt` with the needle on line 2,
///  and the settings that scan for that needle.
fn repository_with_known_file(work: &Path) -> (PathBuf, ScannerSettings, String) {
    let repo: PathBuf = repository(work, "repo");
    let rules: PathBuf = rules_file(work);
    let body: String = format!("first\n{}\n", needle());
    std::fs::write(repo.join("a.txt"), body.as_bytes()).expect("known file");
    commit_all(repo.as_path(), "known");
    return (
        repo,
        settings(None, Some(rules.as_os_str()), false),
        body,
    );
}

/// A rename of an unchanged file and a copy of an existing blob are allowed;
///  the same rename with one byte changed,
///  a new file and a new pathname that itself matches are not.
#[test]
fn a_rename_or_copy_of_content_in_head_is_not_scanned_again() {
    run_isolated(
        "policy_forbidden_strings::tests::known_content::a_rename_or_copy_of_content_in_head_is_not_scanned_again",
        "forbidden-known-head",
        known_in_head,
    );
}

/// The body of `a_rename_or_copy_of_content_in_head_is_not_scanned_again`.
fn known_in_head(work: &Path) {
    let (repo, chosen, body): (PathBuf, ScannerSettings, String) = repository_with_known_file(work);
    std::fs::create_dir_all(repo.join("moved")).expect("moved");
    std::fs::create_dir_all(repo.join("changed")).expect("changed");
    std::fs::create_dir_all(repo.join("dir")).expect("dir");
    // A pure rename: the same bytes at a new path.
    std::fs::rename(repo.join("a.txt"), repo.join("moved/a.txt")).expect("rename");
    assert_eq!(
        check(repo.as_path(), &add(&["moved/a.txt"]), &chosen),
        clean()
    );
    // The same rename with one byte changed: the content is new, so it is scanned.
    std::fs::write(
        repo.join("changed/a.txt"),
        body.replacen("first", "firsT", 1).as_bytes(),
    )
    .expect("changed");
    assert_eq!(
        check(repo.as_path(), &add(&["changed/a.txt"]), &chosen),
        PolicyOutcome::Findings(vec![content_match("changed/a.txt", 2)])
    );
    // A new file with a forbidden string.
    std::fs::write(
        repo.join("fresh.txt"),
        format!("brand new\n{}\n", needle()).as_bytes(),
    )
    .expect("fresh");
    assert_eq!(
        check(repo.as_path(), &add(&["fresh.txt"]), &chosen),
        PolicyOutcome::Findings(vec![content_match("fresh.txt", 2)])
    );
    // A copy of a blob that `HEAD` holds, to a new path.
    std::fs::write(repo.join("copied.txt"), body.as_bytes()).expect("copied");
    assert_eq!(
        check(repo.as_path(), &add(&["copied.txt"]), &chosen),
        clean()
    );
    // Known bytes at a pathname that matches a rule: the pathname is new, so it is still reported, masked.
    let named: String = format!("dir/{}.txt", needle());
    std::fs::write(repo.join(&named), body.as_bytes()).expect("named");
    assert_eq!(
        check(repo.as_path(), &add(&[named.as_str()]), &chosen),
        PolicyOutcome::Findings(vec![PolicyFinding {
            code: FORBIDDEN_STRING_CODE,
            message: String::from("Forbidden string matched in pathname segment 2 (rule 0)."),
            path: Some(EventPath::from_git_bytes("dir/[REDACTED]".as_bytes())),
            location: None,
            fix_available: false,
        }])
    );
}

/// The commit path judges staged entries against the base the commit was recorded on:
///  a staged rename is allowed and a staged rename with a changed byte is not.
#[test]
fn staged_entries_are_compared_with_the_recorded_base() {
    run_isolated(
        "policy_forbidden_strings::tests::known_content::staged_entries_are_compared_with_the_recorded_base",
        "forbidden-known-staged",
        known_staged,
    );
}

/// The body of `staged_entries_are_compared_with_the_recorded_base`.
fn known_staged(work: &Path) {
    let (repo, chosen, body): (PathBuf, ScannerSettings, String) = repository_with_known_file(work);
    let base: ObjectId = oid(repo.as_path(), "HEAD");
    std::fs::create_dir_all(repo.join("moved")).expect("moved");
    git(repo.as_path(), &["mv", "a.txt", "moved/a.txt"]);
    let staged = |path: &str| -> LifecycleContent {
        let mut pathspec: OsString = OsString::from(":(top,literal)");
        pathspec.push(path);
        return LifecycleContent::Requested(CandidateRequest::StagedAtBase {
            base: Some(base.clone()),
            pathspecs: vec![pathspec],
        });
    };
    assert_eq!(
        check(repo.as_path(), &staged("moved/a.txt"), &chosen),
        clean()
    );
    std::fs::write(
        repo.join("moved/a.txt"),
        body.replacen("first", "firsT", 1).as_bytes(),
    )
    .expect("edit");
    git(repo.as_path(), &["add", "moved/a.txt"]);
    assert_eq!(
        check(repo.as_path(), &staged("moved/a.txt"), &chosen),
        PolicyOutcome::Findings(vec![content_match("moved/a.txt", 2)])
    );
}

/// A repository with no commit has no parent tree, and a root commit has no parent:
///  both scan every candidate.
#[test]
fn a_root_commit_and_an_unborn_branch_are_scanned() {
    run_isolated(
        "policy_forbidden_strings::tests::known_content::a_root_commit_and_an_unborn_branch_are_scanned",
        "forbidden-known-root",
        known_root,
    );
}

/// The body of `a_root_commit_and_an_unborn_branch_are_scanned`.
fn known_root(work: &Path) {
    let rules: PathBuf = rules_file(work);
    let chosen: ScannerSettings = settings(None, Some(rules.as_os_str()), false);
    let repo: PathBuf = work.join("unborn");
    std::fs::create_dir(&repo).expect("repository directory");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(
        repo.join("a.txt"),
        format!("first\n{}\n", needle()).as_bytes(),
    )
    .expect("a");
    assert_eq!(
        check(repo.as_path(), &add(&["a.txt"]), &chosen),
        PolicyOutcome::Findings(vec![content_match("a.txt", 2)])
    );
    commit_all(repo.as_path(), "root");
    assert_eq!(
        check(repo.as_path(), &landed(repo.as_path(), "HEAD"), &chosen),
        PolicyOutcome::Findings(vec![content_match("a.txt", 2)])
    );
}

/// A merge whose result holds a file that only one parent has is allowed;
///  new content brought by the merge commit itself is not;
///  a push judges a commit against the parents that were already published, not against its pushed ancestors.
#[test]
fn a_merge_is_judged_against_every_parent_and_a_push_against_the_published_ones() {
    run_isolated(
        "policy_forbidden_strings::tests::known_content::a_merge_is_judged_against_every_parent_and_a_push_against_the_published_ones",
        "forbidden-known-merge",
        known_merge,
    );
}

/// The body of `a_merge_is_judged_against_every_parent_and_a_push_against_the_published_ones`.
fn known_merge(work: &Path) {
    let repo: PathBuf = repository(work, "repo");
    let rules: PathBuf = rules_file(work);
    let chosen: ScannerSettings = settings(None, Some(rules.as_os_str()), false);
    let a_match: PolicyOutcome = PolicyOutcome::Findings(vec![content_match("a.txt", 2)]);
    // `left` adds the needle file, `right` adds a clean file, and the merge has both.
    git(repo.as_path(), &["switch", "--quiet", "--create", "left"]);
    std::fs::write(
        repo.join("a.txt"),
        format!("first\n{}\n", needle()).as_bytes(),
    )
    .expect("a");
    commit_all(repo.as_path(), "left");
    // The branch moves when the merge lands on it, so the commit that introduced the file is kept by name.
    let introducing: ObjectId = oid(repo.as_path(), "HEAD");
    git(repo.as_path(), &["switch", "--quiet", "--create", "right", "main"]);
    std::fs::write(repo.join("other.txt"), b"clean\n").expect("other");
    commit_all(repo.as_path(), "right");
    git(repo.as_path(), &["switch", "--quiet", "left"]);
    git(repo.as_path(), &["merge", "--quiet", "--no-ff", "--no-edit", "right"]);
    // The file is new against `right` and known to `left`.
    assert_eq!(
        check(repo.as_path(), &landed(repo.as_path(), "HEAD"), &chosen),
        clean()
    );
    // The commit that introduced the file is scanned on its own.
    assert_eq!(
        check(
            repo.as_path(),
            &LifecycleContent::Requested(CandidateRequest::Landed(introducing.clone())),
            &chosen
        ),
        a_match
    );
    // A push of the merge alone has both parents already published.
    assert_eq!(
        check(
            repo.as_path(),
            &pushed(vec![oid(repo.as_path(), "HEAD")], Vec::new()),
            &chosen
        ),
        clean()
    );
    // A push of the merge and the commit that introduced the file: that parent is part of the push, so it vouches for nothing.
    assert_eq!(
        check(
            repo.as_path(),
            &pushed(
                vec![introducing.clone(), oid(repo.as_path(), "HEAD")],
                Vec::new()
            ),
            &chosen
        ),
        a_match
    );
    // A pushed tip with no known published commit has no published parents to vouch: its tree is scanned whole.
    assert_eq!(
        check(
            repo.as_path(),
            &pushed(
                vec![oid(repo.as_path(), "HEAD")],
                vec![oid(repo.as_path(), "HEAD")]
            ),
            &chosen
        ),
        a_match
    );
    // A merge commit that brings new content of its own: only that content is reported.
    git(repo.as_path(), &["switch", "--quiet", "--create", "evil", introducing.as_str()]);
    git(
        repo.as_path(),
        &["merge", "--quiet", "--no-ff", "--no-commit", "right"],
    );
    std::fs::write(
        repo.join("evil.txt"),
        format!("novel\n{}\n", needle()).as_bytes(),
    )
    .expect("evil");
    git(repo.as_path(), &["add", "evil.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--no-edit"]);
    assert_eq!(
        check(repo.as_path(), &landed(repo.as_path(), "HEAD"), &chosen),
        PolicyOutcome::Findings(vec![content_match("evil.txt", 2)])
    );
}
