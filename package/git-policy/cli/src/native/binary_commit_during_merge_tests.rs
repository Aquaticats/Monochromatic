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
