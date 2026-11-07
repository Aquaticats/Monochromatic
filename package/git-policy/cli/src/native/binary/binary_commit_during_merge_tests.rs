//! What:
//!  A commit of named paths made while another session's merge is unfinished (`MERGE_HEAD` exists),
//! through the built
//!  wrapper and the resident service.
//! Why:
//!  The owner's rule (`SPEC.md`,
//! "Merges and commits never fail"):
//! "Commits should never fail."
//! Real Git refuses a partial
//!  commit during a merge (`fatal: cannot do a partial commit during a merge.`),
//! and in a checkout many sessions share,
//! one
//!  person's unfinished merge must not make another session's unrelated commit fail.
//! The commit lands on `HEAD` as an
//!  ordinary commit of its paths,
//! and the merge is carried forward:
//! `MERGE_HEAD`,
//! the staged merge result and the merge message
//!  stay,
//! and the commit that concludes the merge sits on the new head.
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

/// A repository where `other` changes `y.txt`,
/// `main` changes `z.txt`,
/// and `git merge --no-commit --no-ff other` is unfinished:
///  `y.txt` is staged by the merge,
/// and `x.txt` is a tracked file the merge does not touch.
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

/// An unrelated commit of named paths during an unfinished merge lands as an ordinary commit,
/// and the merge is carried on.
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

/// During a merge that stopped on a conflict,
/// an unrelated path lands like any other,
/// and the merge is carried on.
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

/// The owner (2026-11-03):
/// "Committing a conflicted path is fine.
/// Warn and commit anyway,
/// even if conflict markers will be
///  committed."
/// A commit that names the path a merge left in conflict lands,
/// the committed blob holds the markers,
/// the merge is
///  carried on,
/// and standard error says in plain words that the path still has conflict markers,
/// that they were committed,
/// and
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

/// A repository whose merge of `other` into `main` stopped on a conflict in `y.txt`.
fn conflicted_merge(fixture: &Fixture, name: &str) -> PathBuf {
    return conflicted_merge_with_attributes(fixture, name, None);
}

/// The same,
/// with `.gitattributes` holding `attributes` from the base commit on,
/// so Git writes the markers its lines ask for.
fn conflicted_merge_with_attributes(fixture: &Fixture, name: &str, attributes: Option<&str>) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new(name));
    git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    std::fs::write(repo.join("y.txt"), "base\n").expect("file");
    git(fixture, &repo, &["add", "--", "y.txt"]);
    if let Some(lines) = attributes {
        std::fs::write(repo.join(".gitattributes"), lines).expect("attributes");
        git(fixture, &repo, &["add", "--", ".gitattributes"]);
    }
    git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(fixture, &repo, &["switch", "--quiet", "--create", "other"]);
    std::fs::write(repo.join("y.txt"), "other\n").expect("file");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=other"]);
    git(fixture, &repo, &["switch", "--quiet", "main"]);
    std::fs::write(repo.join("y.txt"), "main\n").expect("file");
    git(fixture, &repo, &["commit", "--quiet", "--all", "--message=main"]);
    let merged = super::support::direct(fixture).current_dir(&repo).args(["merge", "--no-edit", "other"]).output().expect("git runs");
    assert_eq!(merged.status.code(), Some(1), "the merge conflicts");
    return repo;
}

/// The owner (2026-11-03,
/// question tool):
/// "Warn and finish the merge (Recommended)".
/// A plain `git commit` with no paths while
///  files are still unmerged lands as the merge commit with the markers in those files,
/// instead of Git's "Committing is not
///  possible because you have unmerged files",
/// and the warning is worded as the named-path warning is.
#[test]
fn a_plain_commit_with_unmerged_files_finishes_the_merge_with_their_markers_and_warns() {
    let fixture: Fixture = fixture("plain-commit-conflicted");
    let repo: PathBuf = conflicted_merge(&fixture, "repo");
    let name: String = format!("pcc-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let landed: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=finish the merge"]);
    let stderr: String = String::from_utf8_lossy(&landed.stderr).into_owned();
    assert_eq!(landed.status.code(), Some(0), "the merge is finished: {stderr}");
    assert_eq!(git_text(&fixture, &repo, &["rev-list", "--parents", "--max-count=1", "HEAD"]).split_whitespace().count(), 3, "a merge commit: one commit and two parents");
    assert!(git_text(&fixture, &repo, &["show", "HEAD:y.txt"]).contains("<<<<<<<"), "the committed blob holds the markers");
    assert!(!repo.join(".git/MERGE_HEAD").exists(), "the merge is concluded");
    assert_eq!(git_text(&fixture, &repo, &["ls-files", "--unmerged"]), "", "nothing is unmerged any more");
    assert!(stderr.contains("y.txt") && stderr.contains("conflict markers") && stderr.contains("committed"), "the warning names the file: {stderr}");
    assert!(stderr.contains("follow-up commit"), "the warning says how to fix it: {stderr}");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The owner (2026-11-03):
/// "Committing a conflicted path is fine.
/// Warn and commit anyway".
/// `git commit -a` stages the working
///  copy of every tracked file,
/// an unmerged one included,
/// so it commits the markers too,
/// and it warns for each such file in the
///  same words as a plain commit.
#[test]
fn a_commit_with_all_over_unmerged_files_commits_their_markers_and_warns() {
    let fixture: Fixture = fixture("all-commit-conflicted");
    let repo: PathBuf = conflicted_merge(&fixture, "repo");
    let name: String = format!("acc-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let landed: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--no-enforce-only", "-a", "--message=finish the merge with -a"]);
    let stderr: String = String::from_utf8_lossy(&landed.stderr).into_owned();
    assert_eq!(landed.status.code(), Some(0), "the merge is finished: {stderr}");
    assert!(git_text(&fixture, &repo, &["show", "HEAD:y.txt"]).contains("<<<<<<<"), "the committed blob holds the markers");
    assert!(!repo.join(".git/MERGE_HEAD").exists(), "the merge is concluded");
    assert!(stderr.contains("y.txt") && stderr.contains("conflict markers") && stderr.contains("committed"), "the warning names the file: {stderr}");
    assert!(stderr.contains("follow-up commit"), "the warning says how to fix it: {stderr}");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The line a named-path commit prints when it leaves the merge open:
/// that the merge is still in progress and the commands that
///  finish it.
fn says_the_merge_is_open(stderr: &str) -> bool {
    return stderr.contains("merge is still in progress") && stderr.contains("git merge --continue") && stderr.contains("git commit");
}

/// A path the person resolved by editing,
/// with no marker line left and not staged,
/// committed by name during the merge:
/// the commit
///  lands and the merge is carried on (the SPEC design),
/// but no marker is claimed,
/// since the committed blob has none,
/// and the
///  one line says the merge is still open and how to finish it.
#[test]
fn a_resolved_but_unstaged_path_committed_by_name_makes_no_marker_claim_and_says_how_to_finish() {
    let fixture: Fixture = fixture("commit-resolved-unstaged");
    let repo: PathBuf = conflicted_merge(&fixture, "repo");
    std::fs::write(repo.join("y.txt"), "resolved by hand\n").expect("the resolution");
    let name: String = format!("cru-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let landed: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=resolve y", "--", "y.txt"]);
    let stderr: String = String::from_utf8_lossy(&landed.stderr).into_owned();
    assert_eq!(landed.status.code(), Some(0), "{stderr}");
    assert_eq!(git_text(&fixture, &repo, &["show", "HEAD:y.txt"]), "resolved by hand\n", "the committed blob is the resolution");
    assert!(repo.join(".git/MERGE_HEAD").exists(), "the merge is carried on");
    assert!(!stderr.contains("conflict markers"), "no marker is claimed for a clean blob: {stderr}");
    assert!(says_the_merge_is_open(stderr.as_str()), "the merge is said to be open, with the commands that finish it: {stderr}");
    let finished: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--no-edit"]);
    assert_eq!(finished.status.code(), Some(0), "the line's remedy finishes the merge: {}", String::from_utf8_lossy(&finished.stderr));
    assert!(!repo.join(".git/MERGE_HEAD").exists(), "the merge is concluded");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A path that really holds marker lines keeps the claim,
/// and the merge-open line comes too,
/// whether or not markers were committed.
#[test]
fn a_path_committed_by_name_with_marker_lines_keeps_the_claim_and_says_the_merge_is_open() {
    let fixture: Fixture = fixture("commit-markers-open");
    let repo: PathBuf = conflicted_merge(&fixture, "repo");
    let name: String = format!("cmo-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let landed: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=markers on purpose", "--", "y.txt"]);
    let stderr: String = String::from_utf8_lossy(&landed.stderr).into_owned();
    assert_eq!(landed.status.code(), Some(0), "{stderr}");
    assert!(stderr.contains("y.txt still has conflict markers, and they were committed"), "{stderr}");
    assert!(stderr.contains("follow-up commit"), "the fix of the file is said: {stderr}");
    assert!(says_the_merge_is_open(stderr.as_str()), "{stderr}");
    assert!(!stderr.contains("The merge in progress is not finished by this commit. To fix it"), "the follow-up commit is not offered as the way to finish the merge: {stderr}");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// The marker size comes from the `conflict-marker-size` attribute as Git reads it:
/// a path with size 9 that holds nine-character
///  markers is claimed,
/// and a path with size 9 that holds only seven-character lines is not.
#[test]
fn the_claim_follows_the_conflict_marker_size_attribute() {
    for (label, rewrite) in [("nine", false), ("seven", true)] {
        let fixture: Fixture = fixture(format!("commit-marker-size-{label}").as_str());
        let repo: PathBuf = conflicted_merge_with_attributes(&fixture, "repo", Some("y.txt conflict-marker-size=9\n"));
        assert!(std::fs::read_to_string(repo.join("y.txt")).expect("file").contains("<<<<<<<<< "), "Git wrote markers of the size the attribute asks for");
        if rewrite {
            // Seven-character lines are not markers for size 9, whatever they look like.
            std::fs::write(repo.join("y.txt"), "text about merges\n<<<<<<< is how a marker starts\n=======\n>>>>>>> is how it ends\n").expect("file");
        }
        let name: String = format!("cms-{label}-{}", std::process::id());
        let mut service: Running = start(&fixture, name.as_str());
        let landed: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--message=by name", "--", "y.txt"]);
        let stderr: String = String::from_utf8_lossy(&landed.stderr).into_owned();
        assert_eq!(landed.status.code(), Some(0), "{label}: {stderr}");
        assert_eq!(stderr.contains("y.txt still has conflict markers"), !rewrite, "{label}: the claim follows the size the attribute gives: {stderr}");
        let _killed = service.kill();
        let _waited = service.wait();
        remove_settled(&fixture);
    }
}
