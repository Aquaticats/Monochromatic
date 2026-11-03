//! What:
//!  A commit of named paths made while another session's merge is unfinished (`MERGE_HEAD` exists), through the built
//!  wrapper and the resident service.
//! Why:
//!  The owner's rule (`SPEC.md`, "Merges and commits never fail"): "Commits should never fail." Real Git refuses a partial
//!  commit during a merge (`fatal: cannot do a partial commit during a merge.`), and in a checkout many sessions share, one
//!  person's unfinished merge must not make another session's unrelated commit fail. The commit lands on `HEAD` as an
//!  ordinary commit of its paths, and the merge is carried forward: `MERGE_HEAD`, the staged merge result and the merge message
//!  stay, and the commit that concludes the merge sits on the new head.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await mergeNoCommit(); await wrapped(['commit', '-m', 'x', '--', 'x.txt']); expect(mergeHeadExists()).toBe(true);
//! ```

use super::commit::{git_text, remove_settled};
use super::service_commit::{Running, start, through_service};
use super::support::{Fixture, fixture, git, repository};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::Output;

/// A repository where `other` changes `y.txt`, `main` changes `z.txt`, and `git merge --no-commit --no-ff other` is unfinished:
///  `y.txt` is staged by the merge, and `x.txt` is a tracked file the merge does not touch.
fn unfinished_merge(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["x.txt", "y.txt", "z.txt"] {
        std::fs::write(repo.join(file), "base\n").expect("file");
    }
    git(fixture, &repo, &["add", "--", "x.txt", "y.txt", "z.txt"]);
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(fixture, &repo, &["switch", "--quiet", "--create", "other"]);
    std::fs::write(repo.join("y.txt"), "other\n").expect("file");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=other"]);
    git(fixture, &repo, &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("z.txt"), "main\n").expect("file");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=main"]);
    git(fixture, &repo, &["merge", "--no-commit", "--no-ff", "other"]);
    return repo;
}

/// An unrelated commit of named paths during an unfinished merge lands as an ordinary commit, and the merge is carried on.
#[test]
fn a_path_commit_during_an_unfinished_merge_lands_and_the_merge_is_carried_forward() {
    let fixture: Fixture = fixture("commit-during-merge");
    let repo: PathBuf = unfinished_merge(&fixture, "repo");
    let name: String = format!("cdm-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let head_before: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let staged_before: String = git_text(&fixture, &repo, &["ls-files", "--stage", "--", "y.txt"]);
    std::fs::write(repo.join("x.txt"), "edited by an unrelated session\n").expect("file");
    let served: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(served.status.code(), Some(0), "{}", String::from_utf8_lossy(&served.stderr));
    let head_after: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    assert_ne!(head_after, head_before, "the commit landed");
    assert_eq!(git_text(&fixture, &repo, &["rev-parse", "HEAD^"]).trim(), head_before);
    assert_eq!(git_text(&fixture, &repo, &["diff-tree", "-r", "--name-only", "HEAD^", "HEAD"]).trim(), "x.txt", "only its path");
    assert!(repo.join(".git/MERGE_HEAD").exists(), "the merge is carried forward");
    assert_eq!(git_text(&fixture, &repo, &["ls-files", "--stage", "--", "y.txt"]), staged_before, "the merge result stays staged");
    let concluded: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=conclude the merge"]);
    assert_eq!(concluded.status.code(), Some(0), "{}", String::from_utf8_lossy(&concluded.stderr));
    let parents: Vec<String> = git_text(&fixture, &repo, &["rev-list", "--parents", "-n", "1", "HEAD"]).split_whitespace().skip(1).map(String::from).collect();
    assert_eq!(parents.len(), 2, "the merge commit");
    assert_eq!(parents[0], head_after, "it sits on the head that moved");
    assert_eq!(parents[1], git_text(&fixture, &repo, &["rev-parse", "other"]).trim());
    assert!(!repo.join(".git/MERGE_HEAD").exists());
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// During a merge that stopped on a conflict, an unrelated path lands like any other, and the merge is carried on.
#[test]
fn during_a_conflicted_merge_an_unrelated_path_lands_and_the_conflict_is_carried_on() {
    let fixture: Fixture = fixture("commit-during-conflict");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    for file in ["x.txt", "y.txt"] {
        std::fs::write(repo.join(file), "base\n").expect("file");
    }
    git(&fixture, &repo, &["add", "--", "x.txt", "y.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["switch", "--quiet", "--create", "other"]);
    std::fs::write(repo.join("y.txt"), "other\n").expect("file");
    git(&fixture, &repo, &["commit", "--quiet", "--all", "--message=other"]);
    git(&fixture, &repo, &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("y.txt"), "main\n").expect("file");
    git(&fixture, &repo, &["commit", "--quiet", "--all", "--message=main"]);
    let merged = super::support::direct(&fixture).current_dir(&repo).args(["merge", "--no-edit", "other"]).output().expect("git runs");
    assert_eq!(merged.status.code(), Some(1), "the merge conflicts");
    let name: String = format!("cdc-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    std::fs::write(repo.join("x.txt"), "edited by an unrelated session\n").expect("file");
    let landed: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=unrelated", "--", "x.txt"]);
    assert_eq!(landed.status.code(), Some(0), "{}", String::from_utf8_lossy(&landed.stderr));
    assert_eq!(git_text(&fixture, &repo, &["diff-tree", "-r", "--name-only", "HEAD^", "HEAD"]).trim(), "x.txt");
    assert!(repo.join(".git/MERGE_HEAD").exists(), "the conflicted merge is carried on");
    assert!(git_text(&fixture, &repo, &["ls-files", "--unmerged"]).contains("y.txt"), "the conflict is still the merge's to resolve");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The owner (2026-11-03): "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will be
///  committed." A commit that names the path a merge left in conflict lands, the committed blob holds the markers, the merge is
///  carried on, and standard error says in plain words that the path still has conflict markers, that they were committed, and
///  how to fix them in a follow-up commit.
#[test]
fn a_commit_naming_a_conflicted_path_lands_with_its_markers_and_warns() {
    let fixture: Fixture = fixture("commit-conflicted-path");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("y.txt"), "base\n").expect("file");
    git(&fixture, &repo, &["add", "--", "y.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["switch", "--quiet", "--create", "other"]);
    std::fs::write(repo.join("y.txt"), "other\n").expect("file");
    git(&fixture, &repo, &["commit", "--quiet", "--all", "--message=other"]);
    git(&fixture, &repo, &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("y.txt"), "main\n").expect("file");
    git(&fixture, &repo, &["commit", "--quiet", "--all", "--message=main"]);
    let merged = super::support::direct(&fixture).current_dir(&repo).args(["merge", "--no-edit", "other"]).output().expect("git runs");
    assert_eq!(merged.status.code(), Some(1), "the merge conflicts");
    let name: String = format!("ccp-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let before: String = git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim().to_owned();
    let landed: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=conflicted on purpose", "--", "y.txt"]);
    let stderr: String = String::from_utf8_lossy(&landed.stderr).into_owned();
    assert_eq!(landed.status.code(), Some(0), "a conflicted path is committed: {stderr}");
    assert_ne!(git_text(&fixture, &repo, &["rev-parse", "HEAD"]).trim(), before, "a commit landed");
    assert!(git_text(&fixture, &repo, &["show", "HEAD:y.txt"]).contains("<<<<<<<"), "the committed blob holds the markers");
    assert!(stderr.contains("y.txt") && stderr.contains("conflict markers") && stderr.contains("committed"), "the warning names the path: {stderr}");
    assert!(stderr.contains("follow-up commit"), "the warning says how to fix it: {stderr}");
    assert!(repo.join(".git/MERGE_HEAD").exists(), "the merge is carried on");
    assert_eq!(git_text(&fixture, &repo, &["ls-files", "--unmerged"]), "", "the committed path is no longer unmerged in the index");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
