//! What:
//!  The facts one record of a deleted branch carries,
//!  against real Git:
//!  a branch whose commit
//!       nothing else reaches answers "not merged",
//!  a branch `HEAD` already contains answers "merged",
//!  and a name that
//!       does not exist answers nothing at all.
//! Why:
//!  The line exists so the branch can be restored from it,
//!  which needs the commit,
//!  and so a reader can see
//!  whether the commits were already safe elsewhere.

use super::{branch_commit, deletion_facts, merged_into_head};
use crate::discard_audit_facts::AuditFact;
use crate::test_support::{fixture, git, isolated_overlay, remove, repository};
use crate::transaction_git::GitContext;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

/// A context that reads the fixture repository through `-C`.
fn context(repo: &std::path::Path) -> GitContext {
    let mut global_prefix: Vec<OsString> = vec![
        OsString::from("-C"),
        repo.as_os_str().to_os_string(),
    ];
    global_prefix.shrink_to_fit();
    return GitContext {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        overlay: isolated_overlay(),
        global_prefix,
        common_dir: None,
    };
}

/// The value of one named fact,
/// or empty text when the record carries none.
fn value(facts: &[AuditFact], name: &'static str) -> String {
    for fact in facts {
        if fact.name() != name {
            continue;
        }
        return match fact {
            AuditFact::Text(_, text) => text.clone(),
            AuditFact::Flag(_, flag) => String::from(if *flag { "true" } else { "false" }),
        };
    }
    return String::new();
}

/// A repository with one branch `main` already contains and one it does not.
#[test]
fn real_git_answers_the_commit_and_whether_head_contains_it() {
    let root: PathBuf = fixture("branch-delete-audit");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=base"]);
    git(repo.as_path(), &["branch", "merged"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "unmerged"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=unmerged"]);
    let commit: String = String::from_utf8_lossy(
        git(repo.as_path(), &["rev-parse", "unmerged"]).stdout.as_slice(),
    ).trim().to_owned();
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    let reading: GitContext = context(repo.as_path());

    // The unmerged branch: named with its commit, and `HEAD` does not contain it.
    assert_eq!(branch_commit(&reading, Some(repo.as_path()), OsStr::new("unmerged"), false).as_deref(), Some(commit.as_str()));
    assert_eq!(merged_into_head(&reading, Some(repo.as_path()), commit.as_str()), Some(false));
    let unmerged: Vec<AuditFact> =
        deletion_facts(&reading, Some(repo.as_path()), OsStr::new("unmerged"), false).expect("the branch exists");
    assert_eq!(value(&unmerged, "branch"), "unmerged");
    assert_eq!(value(&unmerged, "reference"), "refs/heads/unmerged");
    assert_eq!(value(&unmerged, "commit"), commit);
    assert_eq!(value(&unmerged, "merged"), "false");

    // The merged branch: the same fields say the commits are already in `HEAD`.
    let merged: Vec<AuditFact> =
        deletion_facts(&reading, Some(repo.as_path()), OsStr::new("merged"), false).expect("the branch exists");
    assert_eq!(value(&merged, "merged"), "true");
    assert_ne!(value(&merged, "commit"), commit);

    // A name with no local branch answers nothing, so the record writes no line for it.
    assert_eq!(branch_commit(&reading, Some(repo.as_path()), OsStr::new("no-such-branch"), false), None);
    assert_eq!(deletion_facts(&reading, Some(repo.as_path()), OsStr::new("no-such-branch"), false), None);
    // Without a known worktree the invocation's own options select the repository.
    assert_eq!(branch_commit(&reading, None, OsStr::new("unmerged"), false).as_deref(), Some(commit.as_str()));
    remove(root.as_path());
}

/// Real Git:
///  a deletion that names the previous branch records the branch Git expanded it to,
///  with that
///       branch's own commit,
///  and a name no reference answers still writes no line.
#[test]
fn real_git_records_the_branch_a_previous_branch_expression_names() {
    let root: PathBuf = fixture("branch-delete-expression");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=base"]);
    git(repo.as_path(), &["checkout", "--quiet", "-b", "lost"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=lost"]);
    let commit: String =
        String::from_utf8_lossy(git(repo.as_path(), &["rev-parse", "lost"]).stdout.as_slice()).trim().to_owned();
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    let reading: GitContext = context(repo.as_path());
    // The control: Git resolves the expression the record reads.
    let symbolic: String = String::from_utf8_lossy(
        git(repo.as_path(), &["rev-parse", "--symbolic-full-name", "@{-1}"]).stdout.as_slice(),
    )
    .trim()
    .to_owned();
    assert_eq!(symbolic, "refs/heads/lost");
    let expression: &std::ffi::OsStr = std::ffi::OsStr::new("@{-1}");
    assert_eq!(branch_commit(&reading, Some(repo.as_path()), expression, false).as_deref(), Some(commit.as_str()));
    let facts: Vec<AuditFact> =
        deletion_facts(&reading, Some(repo.as_path()), expression, false).expect("the branch exists");
    assert_eq!(value(&facts, "branch"), "lost");
    assert_eq!(value(&facts, "reference"), "refs/heads/lost");
    assert_eq!(value(&facts, "commit"), commit);
    // A name no reference answers writes no line, and the typed reference is asked about first.
    let missing: &std::ffi::OsStr = std::ffi::OsStr::new("@{-9}");
    assert_eq!(branch_commit(&reading, Some(repo.as_path()), missing, false), None);
    assert_eq!(deletion_facts(&reading, Some(repo.as_path()), missing, false), None);
    remove(root.as_path());
}

/// Real Git:
///  `HEAD` is no branch name,
///  so a deletion of it records no branch,
///  while a reference literally named `refs/heads/HEAD` is a branch of its own that the record keeps.
/// Why:
///  The line exists so a person can restore what the deletion took,
///  and a line naming the branch `HEAD` resolves to would restore a branch no deletion touched.
#[test]
fn real_git_records_no_branch_for_a_revision_that_is_no_branch_name() {
    let root: PathBuf = fixture("branch-delete-revision-not-branch");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=base"]);
    let reading: GitContext = context(repo.as_path());
    // The control: general revision resolution answers the branch `HEAD` points at, and that is no branch named `HEAD`.
    let symbolic: String = String::from_utf8_lossy(
        git(repo.as_path(), &["rev-parse", "--symbolic-full-name", "HEAD"]).stdout.as_slice(),
    )
    .trim()
    .to_owned();
    assert_eq!(symbolic, "refs/heads/main", "the control needs HEAD to point at main");
    // Git refuses `git branch -D HEAD` in this repository, so the record names no branch and no commit.
    assert_eq!(branch_commit(&reading, Some(repo.as_path()), OsStr::new("HEAD"), false), None);
    assert_eq!(deletion_facts(&reading, Some(repo.as_path()), OsStr::new("HEAD"), false), None);
    // A reference literally named `refs/heads/HEAD` is a branch: Git deletes it, and the record keeps it.
    let commit: String = String::from_utf8_lossy(git(repo.as_path(), &["rev-parse", "main"]).stdout.as_slice())
        .trim()
        .to_owned();
    git(repo.as_path(), &["update-ref", "refs/heads/HEAD", "main"]);
    let facts: Vec<AuditFact> =
        deletion_facts(&reading, Some(repo.as_path()), OsStr::new("HEAD"), false).expect("the branch exists");
    assert_eq!(value(&facts, "branch"), "HEAD");
    assert_eq!(value(&facts, "reference"), "refs/heads/HEAD");
    assert_eq!(value(&facts, "commit"), commit);
    remove(root.as_path());
}

/// Real Git:
///  a remote-tracking deletion expands the upstream mark and nothing else,
///  so the record names the upstream reference Git took and no other.
/// Why:
///  Git expands a deletion operand with `copy_branchname` under the namespace's own kind,
///  and the remote-tracking kind interprets a branch mark only:
///  `@{-1}` is a local branch,
///  which that deletion never expands,
///  and a fully qualified operand is no mark at all.
#[test]
fn real_git_records_only_the_remote_tracking_mark_a_deletion_expands() {
    let root: PathBuf = fixture("branch-delete-remote-mark");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=base"]);
    let commit: String = String::from_utf8_lossy(git(repo.as_path(), &["rev-parse", "main"]).stdout.as_slice())
        .trim()
        .to_owned();
    // An upstream stored as a remote-tracking reference, which is what a remote-tracking deletion expands `@{u}` to.
    let place: String = repo.display().to_string();
    git(repo.as_path(), &["config", "remote.origin.url", place.as_str()]);
    git(repo.as_path(), &["config", "remote.origin.fetch", "+refs/heads/*:refs/remotes/origin/*"]);
    git(repo.as_path(), &["config", "branch.main.remote", "origin"]);
    git(repo.as_path(), &["config", "branch.main.merge", "refs/heads/main"]);
    git(repo.as_path(), &["update-ref", "refs/remotes/origin/main", commit.as_str()]);
    let reading: GitContext = context(repo.as_path());
    // The control: Git itself expands the mark to the reference the record must name.
    let resolved: String = String::from_utf8_lossy(
        git(repo.as_path(), &["rev-parse", "--symbolic-full-name", "@{u}"]).stdout.as_slice(),
    )
    .trim()
    .to_owned();
    assert_eq!(resolved, "refs/remotes/origin/main", "the control needs @{{u}} to name the remote-tracking ref");
    let facts: Vec<AuditFact> = deletion_facts(&reading, Some(repo.as_path()), OsStr::new("@{u}"), true)
        .expect("the upstream reference exists");
    assert_eq!(value(&facts, "branch"), "origin/main");
    assert_eq!(value(&facts, "reference"), "refs/remotes/origin/main");
    assert_eq!(value(&facts, "commit"), commit);
    // A branch the worktree visited before is no remote-tracking mark: Git deletes nothing, even when a
    //  remote-tracking reference of that name is there for a wrong reader to claim.
    git(repo.as_path(), &["checkout", "--quiet", "-b", "lost"]);
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    git(repo.as_path(), &["update-ref", "refs/remotes/lost", commit.as_str()]);
    assert_eq!(deletion_facts(&reading, Some(repo.as_path()), OsStr::new("@{-1}"), true), None);
    // A fully qualified operand is no mark either, and Git prefixes it again and finds nothing.
    assert_eq!(
        deletion_facts(&reading, Some(repo.as_path()), OsStr::new("refs/remotes/origin/main"), true),
        None
    );
    remove(root.as_path());
}

/// Real Git:
///  a branch whose name is not UTF-8 keeps its own reference and commit in the record,
///  because the
///       name reaches Git as bytes.
#[cfg(unix)]
#[test]
fn real_git_records_a_branch_name_that_is_not_utf8() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("branch-delete-non-utf8");
    let repo: PathBuf = repository(root.as_path(), "repo");
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=base"]);
    let name: std::ffi::OsString = std::ffi::OsString::from_vec(b"lost-\xff".to_vec());
    git(repo.as_path(), &[OsStr::new("branch"), name.as_os_str()]);
    let commit: String = String::from_utf8_lossy(
        git(repo.as_path(), &[OsStr::new("rev-parse"), name.as_os_str()]).stdout.as_slice(),
    )
    .trim()
    .to_owned();
    let reading: GitContext = context(repo.as_path());
    let facts = deletion_facts(&reading, Some(repo.as_path()), name.as_os_str(), false).expect("the branch exists");
    assert_eq!(value(&facts, "commit"), commit);
    // The whole identity, byte for byte: `refs/heads/lost-` and the byte 0xff.
    let mut reference_bytes: Vec<u8> = b"refs/heads/".to_vec();
    reference_bytes.extend_from_slice(b"lost-\xff");
    assert_eq!(
        value(&facts, "referenceHex"),
        crate::path_bytes_in_records::hex_of_bytes(reference_bytes.as_slice())
    );
    // The text field beside it holds replacement characters, so it is no name to restore from.
    assert_eq!(value(&facts, "reference"), "refs/heads/lost-\u{fffd}");
    // The control: the same reference spelled as bytes is the branch Git holds.
    let listed: String = String::from_utf8_lossy(
        git(repo.as_path(), &["for-each-ref", "--format=%(refname)", "refs/heads"]).stdout.as_slice(),
    )
    .trim()
    .to_owned();
    assert!(listed.contains("lost-"), "{listed}");
    remove(root.as_path());
}
